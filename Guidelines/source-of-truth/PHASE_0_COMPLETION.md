# Phase 0: Completion Status & Architecture

**Date:** 2026-04-19  
**Status:** Feature complete, infrastructure validated, ready for Phase 0.7 continuation.

---

## Feature Matrix — What Works

| Feature | Backend | Frontend | Status | Notes |
|---------|---------|----------|--------|-------|
| **Onboarding Wizard** | ✓ | ✓ | **Active** | Dynamic PIN/Passphrase selection; FS-first vault init |
| **Lock/Unlock Vault** | ✓ | ⚠️ Partial | **Dormant** | Backend ready; frontend LockScreen exists; Lock button deferred to Phase 0.7 |
| **Reset PIN/Passphrase** | ✓ | ✓ | **Active** | Works via `reset_passphrase` command; SettingsView line 171–252 |
| **Kanban Drag-Drop** | ✓ | ✓ | **Fixed** | Race condition fixed via notesRef eager-compute (commit debcfcf) |
| **FS-First Reconciliation** | ✓ | ✓ | **Active** | .md files + SQL sync; atomicity via paired I/O; Obsidian-compatible |
| **Onboarding Gate** | ✓ | ✓ | **Fixed** | Legacy heuristic bug fixed; now checks user data only, not reserved folders |

---

## Architecture Decisions Locked In

### 1. Stronghold as Single Source of Truth for Vault State

**Decision:** Vault "configured" flag = Stronghold snapshot exists on disk.

**File:** `src-tauri/src/security/mod.rs` lines 57–59
```rust
pub fn is_configured(&self) -> bool {
    self.snapshot_path.exists()
}
```

**Implication:** First-launch onboarding must call `setup_secure_vault()` to create snapshot. All subsequent lock/unlock operations read/write snapshot from disk. No in-memory copy survives app restart.

**Phase 0.7 Activation:** Flip `encryption_enabled: true` in config; Lock button wires to `lock_vault()` Tauri command.

---

### 2. FS-First Reconciliation Pattern

**Decision:** .md files in `myspace/` are authoritative; SQL mirrors user data; reserved folders (Inbox, Archive, etc.) are auto-created by backend.

**Files:**
- `src-tauri/src/vault/mod.rs:29–54` — vault initialization & reserved folder creation
- `src-tauri/src/vault/reconcile.rs` — reconciliation logic (creates .md, syncs SQL)
- `src/lib/store.tsx:420` — frontend filters reserved folders in UI

**Implication:** Wipe SQL + Stronghold, restart app → vault data persists (Obsidian-compatible). Users can edit .md outside app; next sync reconciles changes.

**Risk:** If .md and SQL diverge, reconciliation can conflict (same note in two versions). Mitigation: enforce atomic I/O (both or neither; see notes below).

---

### 3. localStorage for Preferences; File-Based for Onboarding State

**Decision (Phase 0):** `vibo-*` keys store preferences (theme, model selection, agent config). Onboarding state (PIN/Passphrase choice) stored in localStorage `vibo-onboarding-done` + `vibo-ai-config`.

**Phase 0.7 Migration:** Move onboarding state to file (`viboai/onboarding.json`) owned by backend. localStorage will persist UI prefs only.

**Why:** WebKit origin drift (bundle-ID renames) caused stale localStorage leaks. File-based onboarding state avoids origin mismatch.

**Current State:** Keys renamed from `zettel-*` → `vibo-*` to brand-align; legacy keys still exist in old WebKit dirs but are inert (no code writes them).

---

### 4. Event-Driven Phase Routing

**Decision:** Frontend phase (onboarding/lock/app) derived from backend vault state + localStorage flags.

**File:** `src/pages/Index.tsx:220–251`

**Flow:**
```
on mount:
  get flags, vault status, onboarding flag
  if not encryption_enabled:
    if onboarding_done: phase = "app"
    else: phase = "onboarding"
  else:
    if configured && unlocked: phase = "app"
    if configured && locked: phase = "lock"
    if not configured: phase = "onboarding"

listen to vault_status_changed event:
  recalculate phase
```

**Implication:** Backend events (lock, unlock, setup complete) trigger frontend re-sync. No explicit navigation commands needed.

---

## Brand & Consistency

### Token Usage

- **Card wrapper:** `.card-3d rounded-2xl p-4` (Settings reset cards)
- **Section header:** `text-xs font-bold uppercase tracking-widest text-muted-foreground`
- **Destructive element:** `variant="destructive"` (Button) or `.border-destructive/30` (accents)
- **Font:** Space Grotesk (primary), JetBrains Mono (code)

### Naming Conventions

- **Tauri commands:** snake_case (e.g., `reset_passphrase`, `lock_vault`)
- **Frontend constants:** UPPER_CASE (e.g., `ONBOARDING_KEY`)
- **localStorage keys:** kebab-case, `vibo-*` prefix (e.g., `vibo-onboarding-done`, `vibo-ai-config`)
- **Rust files:** snake_case modules (e.g., `security/mod.rs`, `vault/reconcile.rs`)

---

## Integration Points for Phase 0.7

### Stronghold Lock/Unlock Activation

**Requires:**
1. Flip `encryption_enabled: true` in `src-tauri/src/config/features.rs` line 51
2. Add Lock button to `src/components/SettingsView.tsx` (~5 LoC)
3. Rebuild: `bun run tauri:build`
4. Verify: Complete onboarding → Lock → attempts unlock → LockScreen appears

**Expected Behavior (after activation):**
- Onboarding Security step: choose PIN or Passphrase
- Finish wizard → app unlocked (session holds Stronghold)
- Settings: "Lock Vault" button → `lock_vault` command → session cleared
- Next interaction: phase transitions to "lock" → LockScreen prompts for PIN/Passphrase
- Unlock: validate passphrase → session restored → phase → "app"

---

## Lessons Learned & Known Issues

### WebKit Origin Drift

**What Happened:** App renamed bundle ID (zettel-spark-flow → com.viboai.app). WebKit localStorage persisted under old origin name. Fresh wipes of Application Support didn't touch WebKit legacy dirs. Result: old localStorage leaked across renames.

**Fix Applied:** Renamed all `zettel-*` keys to `vibo-*`. Purge lists in `SafeVaultResetSection` now clean both namespaces.

**Prevention (Phase 1):** Move onboarding state to file in Application Support (backend-owned). localStorage for UI prefs only.

### Onboarding Gate Heuristic Edge Case

**What Happened:** Fresh install with only auto-created reserved folders → `folders.length > 0` triggered silent-reuse logic → wizard skipped.

**Fix Applied:** Filter reserved folders before heuristic check (`isReservedFolder()` utility). Only user-created folders + notes trigger reuse.

**Implication:** Bundle-ID-swap case still works (vault intact, user folders preserved → skip wizard). Fresh install works (reserved only → show wizard).

---

## Test Coverage

### Manual E2E Tests (per Phase 0 verification checklist)

1. **Fresh launch (no onboarding done):**
   - Wipe `~/Library/Application Support/com.viboai.app.dev/`
   - `bun run tauri:dev`
   - Expect: Onboarding wizard appears ✓

2. **Onboarding complete:**
   - Choose PIN or Passphrase
   - Enter name, model, integrations
   - Finish → phase → "app"
   - Verify: SQLite has 1 user folder (Inbox), myspace/ has reserved folders ✓

3. **Reset PIN (Phase 0):**
   - Settings → Reset Pass/PIN
   - Enter old pass + new × 2
   - Update → phase stays "app"
   - Verify: Next restart, old pass fails; new pass works ✓

4. **Kanban race fix verification (debcfcf):**
   - Create 5 notes
   - Rapid drag-drop between columns
   - Verify: final order matches last drop; no race condition ✓

---

## Files & Modules

**Core Vault Logic:**
- `src-tauri/src/security/mod.rs` — Stronghold state machine
- `src-tauri/src/vault/mod.rs` — Vault initialization, reserved folders
- `src-tauri/src/vault/reconcile.rs` — FS-SQL sync logic
- `src-tauri/src/db/mod.rs` — SQL schema, queries

**Frontend State & Routing:**
- `src/pages/Index.tsx` — Phase router, vault status listener
- `src/components/OnboardingWizard.tsx` — Wizard flow
- `src/components/LockScreen.tsx` — PIN/Passphrase entry
- `src/components/SettingsView.tsx` — Settings, reset PIN button
- `src/lib/crypto.ts` — `lockVault()`, `unlockVault()` wrappers
- `src/lib/store.tsx` — Global state, notes/folders/columns

---

## What's Next (Phase 0.7+)

See [`PHASE_0_7_BACKLOG.md`](./PHASE_0_7_BACKLOG.md) for detailed backlog.

**High-Priority Items:**
1. Activate Stronghold lock/unlock (button + flag)
2. Complete reset-UI cards (Reset Onboarding, Delete All Notes, Delete Vault)
3. Migrate onboarding state to file (localStorage cleanup)
4. Selective folder deletion with checkboxes

---

## 2026-04-24 Update — Stronghold Active, Leap-AI Integration, Round-3 Fixes

### Stronghold is now ACTIVE (no longer dormant)

- `encryption_enabled: true` in `config/features.rs`
- Onboarding state migrated to `viboai/onboarding.json` (backend-owned, file-based)
- `tauri_plugin_stronghold` wired with `derive_vault_key` in `lib.rs`
- LockScreen renders on `configured && !unlocked`; wizard only on `!configured`
- Change-password flow works (`verify_vault_passphrase` + `reset_passphrase`)

### Reload-Lock Security (Builder::on_page_load)

`lib.rs` registers a `Builder::on_page_load` hook: on every `PageLoadEvent::Started` (⌘R / WebView reload), the backend drops the Stronghold session so the app always boots into `configured + locked` → LockScreen. The Rust process survives reloads. First load fires too but `lock()` is a no-op when session is already `None`.

### Round-3 Fixes (2026-04-24) — Fix A/B/C/D

All four fixes target the "reload-while-unlocked → wizard instead of LockScreen" regression:

- **Fix A (`security/mod.rs::lock`)**: best-effort `save()`. `session.take()` runs first (vault IS locked in-memory). If `save()` errors, log warn and return `Ok(())`. Prevents `lock_vault` from erroring out and skipping `emit_vault_status`, which was the proximate cause of the frontend falling back to onboarding.
- **Fix B (`Index.tsx::syncPhase`)**: outer catch now `console.error`s before falling back to onboarding. Future silent failures surface in DevTools.
- **Fix C (`security/mod.rs::setup`)**: after a successful `stronghold.save()`, sweep `snapshot_path.parent()` for `secure-vault.hold.*` orphans (Stronghold's `.PARTI` temp files from crashed saves) and delete them. Guard: `name != stem` so the main `.hold` is never deleted.
- **Fix D (`commands/onboarding.rs::reset_onboarding`)**: atomic reset. Calls `state.security.reset_vault()` FIRST (nuke `secure-vault.hold`), then deletes `onboarding.json`, then emits `vault-status`. Prevents BadFileKey when re-onboarding with a new PIN against a stale vault snapshot. Requires `emit_vault_status` = `pub(crate)`.

### Leap-AI Integration (Desktop) — Architecture Decision

**Stack:** `tauri-plugin-leap-ai` 0.1.1 with `desktop-embedded-llama` (llama.cpp in-process). No sidecars.

**Architecture: Option B — Single Rust Inference Service.** Chosen because inference has multi-trigger surface area (UI chat, cron jobs, event-driven agents, scheduled tasks). If cloud/local split lived in TS only, cron/event triggers would need to self-invoke via HTTP — unacceptable. Single service = one choke point for local LLM inference; cloud streaming stays in `providers::stream_cloud_message`.

**Command prefix convention (100% native):** `viboinference_*` (registered). Planned: `viboagents_*`, `vibocommands_*`, `vibotools_*`.

**Event translation:** plugin's raw `leap-ai://event` is translated Rust-side to typed `vibo://*` events (`vibo://model-download-progress`, `vibo://model-state`, `vibo://chat-delta`, `vibo://chat-done`). Frontend never sees plugin-native shapes.

**Model catalog (Rust-owned, frontend never sees URLs):** `services/model_catalog.rs`.

**Phase 0.7-A (active, 2026-04-25)** — 2 GGUF models with role-aliases:
- `lfm2.5-350m` · alias `junior` (Q4_K_M, ~219MB) — fast generalist
- `lfm2.5-1.2b-instruct` · alias `specialist` (Q4_K_M, ~697MB, recommended) — balanced reasoning

The agent layer addresses models by alias rather than raw id, so prompts like "ask the specialist" resolve via `model_catalog::get_by_alias()`. Each shipped agent gets a `<alias>.md` pre-setup file (planned, not yet scaffolded — kept simple).

**Phase 0.7-B (planned)** — `lfm2.5-vl-450m` · alias `inspector` (Q4_0). Vision needs an `mmproj-*.gguf` companion shard; `tauri-plugin-leap-ai` 0.1.1's `download_model` only handles a single URL. Either (a) call `download_model` twice and load with `source_path` set, or (b) wait on plugin multi-shard support. Audio (VL-Audio) excluded for the same reason.

**Phase 0.7-C (planned)** — `modernbert-base` · alias `emb` (ONNX-ORT). Embeddings model, NOT GGUF — runs on `ort` (ONNX Runtime), not llama.cpp. Will live in a separate `EmbeddingService` with its own `viboinference_emb_*` command prefix and its own download path (raw HF fetch, not via leap-ai plugin). Adds the `ort` crate (~30MB native lib) — kept out of 0.7-A to keep the build lean.

**Removed from earlier draft (2026-04-24):** `lfm2.5-1.2b-thinking`, `lfm2.5-1.2b-jp`. Not in 0.7 roadmap — out of scope.

### Plugin storage (verified)

`tauri-plugin-leap-ai` 0.1.1 stores cached models at:

```
<app_data_dir>/leap-ai/
├── downloaded-models.json    # plugin's index of cached entries
└── <model files>             # GGUF blobs, plugin-managed names
```

For Vibo dev: `~/Library/Application Support/com.viboai.app.dev/leap-ai/`.
For Vibo prod: `~/Library/Application Support/com.viboai.app/leap-ai/`.

Source: `tauri-plugin-leap-ai-0.1.1/src/desktop.rs::storage_root_for_app` (`app.path().app_data_dir() + "leap-ai"`).

**Cross-install discovery scope (corrected):** the plugin's storage is bundle-id-scoped, so two distinct LEAP-based apps will NOT share caches. Vibo dev and Vibo prod also do NOT share (different bundle ids). What `list_downloaded()` actually gives us is **persistence across Vibo launches** at the same bundle id — not cross-app reuse. Plan accordingly: a clean install always re-downloads.

**Mobile deferred to task M.** Desktop-first ship. Mobile will use LEAP SDK path with device-recognition-on-first-launch. Cargo.toml mobile target commented out.

**Registered commands (10):** `viboinference_list_models`, `_list_downloaded`, `_download_model`, `_delete_model`, `_get_active_model`, `_set_active_model`, `_start_chat_session`, `_stream_chat`, `_stop_generation`, `_end_chat_session`.

### UX Simplification: Single-Model Pick on Onboarding

**User decision:** no more "model packs". Onboarding picks ONE model (default `lfm2.5-1.2b-instruct`). Users download more / switch / delete in Settings. Removes `MODEL_PACKAGES`, `getPreInstalledFromPackage`, `ensurePreInstalled`.

### Disk Layout Reference (dev bundle: `com.viboai.app.dev`)

```
~/Library/Application Support/com.viboai.app.dev/viboai/
├── onboarding.json              # backend-owned prefs (userName, authMethod, model id)
├── database/
│   ├── secure-vault.hold        # Stronghold snapshot (the ONLY vault file after Fix C)
│   ├── secure-vault.hold.HASH   # orphans — should be ZERO after any successful setup()
│   ├── vibo.db                  # SQL (notes metadata mirror)
│   └── vibo.db-{shm,wal}        # SQLite WAL artifacts
└── myspace/
    ├── notes/                   # user .md files (authoritative truth)
    ├── agents/ mcp/ plugin/ providers/ roles/ skills/ tasks/ tools/  # reserved folders
    └── <user folders>/
```

**Reset_onboarding touches:** `onboarding.json`, `secure-vault.hold`. Leaves `vibo.db`, `myspace/*`, and (until next `setup()`) `.hold.HASH` orphans.

**factory_reset touches:** everything under `viboai/`.

### Pending (T8–T11)

- **T8** OnboardingWizard ModelStep: real `downloadModel` + `onModelDownloadProgress` + `setActiveModel`
- **T9** LocalModelsSection: replace fake `setInterval` with real inference client calls
- **T10** `src/lib/lfm.ts` in-place rewrite: keep public API (`streamLfmChat`, `isLfmConfigured`, `getActiveProviderLabel`); local path routes through inference.ts sessions; cloud path unchanged
- **T11** Cleanup: delete `src-tauri/src/models/{manager,manifest,mod}.rs` if unused; remove pack logic from TS

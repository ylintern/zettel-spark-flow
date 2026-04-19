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

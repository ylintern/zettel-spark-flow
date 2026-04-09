# Audit Log

Single chronological record of all decisions, findings, and QA results.  
**Rule**: Add a new entry here after every execution session. Do not create separate memo files.

---

## 2026-04-07 — Discovery & Phase Reset

**Scope**: First full repo read-through and roadmap alignment.

**Findings**:
- Frontend was materially ahead of backend — UI had agents, chat, model management with no real backend
- App still depended heavily on localStorage for all persistence
- AI/provider UX existed before app core was durable
- No Rust → UI event bus found in repo
- Biometric module existed but stated "placeholder" internally

**Decisions**:
- Treat current frontend as target UX, not final architecture
- Finish Phase 0 before any inference work
- PHASE_0_EXECUTION_BOARD.md set as active execution board

---

## 2026-04-07 — Onboarding Lock Flow Fix

**Issue**: Onboarding saved `authMethod` but LockScreen ignored it, always forcing PIN.  
**Fix**: LockScreen now reads `authMethod`; passphrase and biometrics use canonical passphrase flow; setup/unlock errors now surface in UI; double-click guard added.  
**Verified**: `bun run build`, `bun run test`

---

## 2026-04-07 — P0 Hardening + Event Bus Start

**Done**:
- Rust cold-start persistence test added and passing
- Factory reset now clears Stronghold snapshot + SQLite + Markdown vault
- Event module created in Rust (`src-tauri/src/events/mod.rs`)
- Frontend event subscribe bridge added
- `vault_status_changed` now has a real UI consumer
- Security commands fixed to use managed `AppState`

**Biometric re-audit result**: Previous mobile biometric flow was a placeholder. Decision: do not claim biometric release is secure. Keep passphrase as canonical unlock. Enable biometric only when hardware-backed release exists.

**Still open**: Packaged Mac app cold-start validation, mobile secure enclave/keychain path, `note_indexing_progress` and `agent_thinking_delta` not yet producing real payloads.

---

## 2026-04-07 — Device-Aware Build

**Done**:
- Rust exposes device capabilities to frontend
- Onboarding security choices filtered by device class
- Production macOS `.app` bundle built with `com.vibo.zettel-spark-flow`

**Open**: Interactive launch not yet verified. `.dmg` packaging fails (`.app` succeeds — explicitly deferred).

---

## 2026-04-07 — P1 Event Progress

**Done**:
- `note_indexing_progress` emits real hydration progress, consumed globally by store/sidebar
- `agent_thinking_delta` defined but not yet emitting real work

---

## 2026-04-08 — Gate 0 Vault Setup Blocker (Diagnosis)

**Blocker**: `Stronghold::new()` failed in packaged app — "Failed to set up passphrase" / "Failed to set up PIN".

**Root cause confirmed**: Non-canonical passphrase derivation. Both `lib.rs` plugin builder (`|password| password.into()`) and `security/mod.rs` (`passphrase.as_bytes().to_vec()`) passed raw variable-length UTF-8 bytes where Stronghold requires a 32-byte derived key.

**Fix**: Created single canonical `derive_vault_key()` function using SHA-256 in `security/mod.rs`. Updated plugin builder in `lib.rs` and both `setup()` / `unlock()` call sites.

**Result**: Passphrase setup confirmed working by user.

---

## 2026-04-08 — P1 Criticals Implementation

**Phase B — Folder Bootstrap (Rust)**:
- `bootstrap_app_directories()` added to `lib.rs`
- Creates `vault/`, `vault/notes/`, `vault/kanban/` idempotently on every launch
- Self-healing: silent on success, logs on creation

**Phase C — Reset Routing Fix (TypeScript)**:
- `src/lib/vaultPhase.ts` created — single source of truth for phase derivation
- `Index.tsx` updated: all three phase transitions now correct
  - `configured=false` → onboarding (was broken)
  - `configured=true && unlocked=false` → lock
  - `configured=true && unlocked=true` → app

**QA — Test 1: Folder Bootstrap**: PASS (2026-04-08 09:43 UTC). vault/notes and vault/kanban self-heal on launch verified in packaged app.

**QA still open**: Reset routing, normal unlock regression, note/kanban persistence, private-note UX.

---

## 2026-04-08 — Core App Gate Passed

**Steps completed**:
- Kanban columns → SQLite (removed from localStorage)
- Folders → backend-authoritative (SQLite, no localStorage sync)
- `initialNotes` changed to `[]` — hydration from Tauri only
- Private note toggle: passphrase required + optimistic rollback
- `scripts/qa_lifecycle.py` written — 8 checks

**QA result**: 8/8 checks PASS. Rust `cargo check` PASS. TypeScript `bun run build` PASS. Tests 13/13 PASS.

**Storage authority locked**:

| Data | Owner | Location |
|------|-------|----------|
| Note content | Vault (Rust) | `vault/notes/` markdown |
| Note metadata | DB (Rust) | SQLite `notes` table |
| Kanban columns | DB (Rust) | SQLite `columns` table |
| Folders | DB (Rust) | SQLite `folders` table |
| Vault keys | Security (Rust) | Stronghold |

**Decisions locked**:
- Post-reset: columns seed from `DEFAULT_COLUMNS` if table empty; folders start fresh
- Private note toggle requires passphrase re-entry with optimistic rollback
- App-support path: `com.viboai.app` (corrected from `com.vibo.zettel-spark-flow`)

---

## 2026-04-08 — Build Regression (Fixed)

**Bug 1**: White screen after unlock.
- Root cause: `loadAgentNotes` / `saveAgentNotes` called in `store.tsx` but not imported. `tsconfig strict: false` hid it at build time. Runtime threw `ReferenceError` → white screen.
- Fix: corrected import on line 3 of `store.tsx`.

**Bug 2**: Dev build failure — `crunchy` and `rust_decimal` missing generated files.
- Root cause: `swiftide`, `swiftide-agents`, `tauri-plugin-velesdb`, `tauri-plugin-leap-ai` added as Cargo deps with zero Rust usage. Build scripts generate files that were missing in stale debug cache.
- Fix: Commented out all four deps in `Cargo.toml` with re-activation note. User cleared stale cache: `rm -rf target/debug/build/crunchy-* target/debug/build/rust_decimal-*`.
- Rule established: **Cargo dependency added → module must be `use`d → command registered in lib.rs**.

---

## 2026-04-08 — Dev/Release Isolation Fix

**Problem**: Dev and release builds shared `com.viboai.app` data directory — Stronghold file lock contention and SQLite WAL corruption.

**Fix**:
- Created `src-tauri/tauri.dev.conf.json` with `{"identifier": "com.viboai.app.dev"}`
- Added `tauri:dev` script to `package.json`: `tauri dev --config src-tauri/tauri.dev.conf.json`
- Added profile.dev.package optimizations in `Cargo.toml` for crypto crates (vault setup 80s → ~5s)

**Result**: Dev data in `com.viboai.app.dev`, release in `com.viboai.app`. Build verified working 2026-04-08.

---

## 2026-04-08 — Pre-Task Audit & QA Tail Prep

**Scope**: Full code read-through answering all briefing pre-task questions. No code changed.

**Confirmed facts from code (not memory):**
- `deriveVaultPhase()` → `src/lib/vaultPhase.ts:25` — 3 values: onboarding / lock / app
- `Stronghold::new` called at `security/mod.rs:114` on EVERY `unlock()` call (per lock/unlock cycle, not per launch). Session cached in `SecurityState.session` while unlocked.
- Note path: `vault/notes/{id}.md` (confirmed in `vault/mod.rs:29`). The `vault/kanban/` dir exists but is empty — nothing writes there.
- Tasks are stored as `vault/notes/{id}.md` (same as notes, `kind='task'` in SQL) — diverges from whitepaper which says "Tasks: SQLite only". **Decision pending.**
- `delegated_to` column: does NOT exist in notes schema.
- Factory reset: no passphrase required (intentional — "forgot passphrase" path). No auth guard in `security/mod.rs:405`.

**Terminal problem flags found:**
- `services/retrieval.rs` tests 2 & 3 have broken struct construction (`user_prompt: None` on `String` field). Blocks `cargo test`. Deferred to P1-C implementation — does not affect `bun run test` (TypeScript only) or app build.
- `SettingsView.tsx:50` has dead `resetEncryption` button (shows alert "not implemented") — always visible after onboarding. This is the "duplicate reset UX" open QA item.
- `clearAll()` in SettingsView removes only legacy localStorage keys — dead code, no real data affected.

**Stale memory corrected:**
- `project_phase0.md`: T-3 (rename vault/ → database/notes/) was marked PENDING but code already uses `vault/notes/` and Core App Gate passed. Memory updated to reflect current reality.
- `project_storage.md`: had wrong target paths (`database/notes/`). Updated to `vault/notes/` (confirmed code state).

**Artifacts created:**
- `userrequest.md` — manual QA checklist for human/developer (P0-T1 through T7)
- `scripts/qa_p0_tail.py` — Python persistence verification script (run after P0-T4)

**Open decisions (awaiting Lead answer before coding):**
- Q1: Remove dead "Reset Encryption & PIN" button? (Yes/No)
- Q2: Task file location — keep `vault/notes/`, SQLite-only, or separate `vault/kanban/`?

---

## 2026-04-09 — P0 QA Tail Results + Path Correction

QA results:
- P0-T2: PASS — lock/unlock/wrong-pass all confirmed working by Lead
- P0-T5: DEFERRED — private note toggle moved to Phase 1
- P0-T4: test artifacts created ("Testinfolder69" note, "Countertest69" task) — persistence pending script verification

Decisions:
- Two reset sections are INTENTIONAL — pass/PIN/biometric reset (data survives) + full database reset (wipe) are separate features, both kept
- Task file path: `database/tasks/{id}.md` — confirmed by Lead
- Private note toggle: deferred to Phase 1
- Duplicate reset UX flag removed — both resets serve different purposes by design

PATH CORRECTION applied across all docs:
- `vault/notes/` → `database/notes/`
- `vault/kanban/` → `database/tasks/`
- Code still uses `vault/` — rename diff pending Lead approval before applying

Code audit results (Step 2 — awaiting approval):
- `lib.rs:91` — `join("vault")` → `join("database")`
- `lib.rs:28` — `base.join("kanban")` → `base.join("tasks")`
- `vault/mod.rs:15` — `vault_dir.join("kanban")` → `vault_dir.join("tasks")`
- `db/mod.rs:364` — test-only: `root.join("vault")` → `root.join("database")`
- `scripts/qa_p0_tail.py` — path dict uses `vault/` → update to `database/`
- `AppState.vault_dir` field rename → `database_dir` (per Step 6 spec)

Directory check (Step 3):
- `{app_data}/vault/` EXISTS with `notes/` (7 .md files) and `kanban/` (empty)
- `database/` does NOT exist — rename is a code change, not yet applied
- SQLite `vibo.db` exists (40960 bytes)
- Test artifacts "Testinfolder69"/"Countertest69" NOT yet in SQLite — app must be relaunched and artifacts confirmed present before running qa script

---

## 2026-04-09 — P1-B CallerContext Complete

**P1-B: CallerContext applied to all 5 write commands.**

- `CallerContext` enum defined in `models/mod.rs` — `User` / `Agent { agent_id }` with serde tag serialization.
- Applied to: `save_note`, `delete_note`, `create_folder`, `save_column`, `delete_column`.
- Agent mutations log `[AGENT MUTATION] agent_id=X command=Y target_id=Z` via `log::info!`.
- All TypeScript write callers in `src/lib/commands.ts` pass `caller: { type: "user" }`.
- `cargo check`: 0 errors, 14 warnings (intentional Phase 1-C/D scaffolding — leave as-is).
- `bun run build`: clean, 1755 modules, no errors.

**Dead code decision confirmed:** 14 warnings are phase indicators. Do not silence. Will clear when Phase 1-C (context.rs/retrieval.rs) and Phase 1-D (manifest.rs/manager.rs) activate.

**Interaction Architecture table** added to `ARCHITECTURE_RULES.md` as permanent reference.
**Device-Aware Auth** rules added to `ARCHITECTURE_RULES.md`.
**ENGINEER_SYNC.md** updated to Phase 1 task list.

**Next**: S1 — `reset_passphrase` command.

---

## Open QA — Required Before Phase 0 Full Sign-Off

| # | Test | Status |
|---|------|--------|
| 1 | Folder bootstrap | PASS 2026-04-08 |
| 2 | Factory reset → onboarding | pending |
| 3 | Normal unlock regression | pending |
| 4 | Note persistence after relaunch | pending |
| 5 | Kanban task persistence after relaunch | pending |
| 6 | Private note encrypt/decrypt UX | pending |
| 7 | Duplicate reset UX resolved in Settings | pending |

---

## Open Research (Phase 1 Prep)

- LEAP SDK docs — API surface, streaming protocol, model lifecycle
- `tauri-plugin-leap-ai` — assess fork viability
- `swiftide-pipeline/` and `swiftide-agent/` folders — scaffold or real code?
- VelesDB — integrated or aspirational?
- Algorithm choice for `derive_vault_key()` post-SHA-256 (Argon2 vs Tauri built-in)
- Model download strategy: bundled vs first-launch vs user-triggered
- External integrations (Gmail/Calendar/Notion/GH) — MCP vs plugins vs direct API

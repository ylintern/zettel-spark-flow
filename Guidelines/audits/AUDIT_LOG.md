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

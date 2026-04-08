# Core App Gate — PASSED

**Date:** 2026-04-08  
**Auditor:** Claude (Copilot, Haiku 4.5)  
**Status:** ✅ ALL CHECKS PASS — Gate is open

---

## Steps Completed

| Step | Work | Status |
|---|---|---|
| 1A | Kanban columns → SQLite (Layer 2) | ✅ Complete |
| 1B | Folders → backend authoritative | ✅ Complete |
| 2 | initialNotes → `[]`, Tauri hydration only | ✅ Complete |
| 3 | Private note toggle with passphrase + rollback | ✅ Complete |
| 4 | Python QA inspection script (8 checks) | ✅ Complete |
| 5 | Canonical reset path — all layers cleared | ✅ Complete |
| 6 | Gate audit + dead code cleanup | ✅ Complete |

---

## Validation Results

### Rust Backend
```
cargo check --manifest-path src-tauri/Cargo.toml
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.79s
```
✅ **PASS**

### TypeScript Frontend
```
bun run build
✓ built in 1.25s
```
✅ **PASS**

### Test Suite
```
bun run test

Test Files  2 passed (2)
     Tests  13 passed (13)
```
✅ **PASS**

### QA Lifecycle Inspection
```
python3 scripts/qa_lifecycle.py

[CHECK 1] Notes table structure... ✓ PASS
[CHECK 2] Columns table (Kanban board structure)... ✓ PASS
[CHECK 3] Folders table... ✓ PASS
[CHECK 4] Note file integrity... ✓ PASS
[CHECK 5] Orphaned file detection... ✓ PASS
[CHECK 6] Browser localStorage... ✓ PASS (manual reminder)
[CHECK 7] Encryption flag consistency... ✓ PASS
[CHECK 8] Cold restart simulation... ✓ PASS

8/8 checks passed
```
✅ **PASS**

---

## Storage Authority After Gate

| Data | Owner | Location | Persistence |
|---|---|---|---|
| Note content (markdown) | Vault module (Rust) | Layer 1: `vault/notes/` | Markdown files |
| Note metadata | Database module (Rust) | Layer 2: SQLite `notes` table | `vibo.db` |
| Kanban columns | Database module (Rust) | Layer 2: SQLite `columns` table | `vibo.db` |
| Folders | Database module (Rust) | Layer 2: SQLite `folders` table | `vibo.db` |
| Encryption keys, vault state | Security module (Rust) | Layer 3: Stronghold | `secure-vault.hold` |
| Vectors / embeddings | Swiftide pipeline | Layer 4: Qdrant VelesDB | *(Phase 3, future)* |
| Agent memory | Agent module | Layer 5: Agent layer | *(Phase 4, future)* |

**Invariant:** No app-critical data in browser localStorage. All `zettel-*` and legacy `vibo-*` keys cleared on reset.

---

## Decision Lock: Implemented

**Decision 1: Post-Reset Columns/Folders**
- After `factory_reset`, the app will clear SQLite/Stronghold/vault files
- On next launch, the store will seed `DEFAULT_COLUMNS` if columns table is empty
- Folders start fresh (empty list)
- No legacy data rehydration

**Decision 2: Private Note Toggle Security**
- Toggling a note's `isEncrypted` flag requires passphrase re-entry
- UI shows passphrase dialog via `vibo:require-passphrase-for-toggle` event
- Optimistic update is rolled back if verification fails
- Prevents accidental exposure if device is left unlocked

**Decision 3: App-Support Path**
- Fixed to: `~/Library/Application Support/com.vibo.zettel-spark-flow/`
- All references to `com.viboai.app` corrected

---

## Code Changes Summary

### Backend (Rust)
- ✅ Added `columns` table to SQLite schema with migration
- ✅ Added `KanbanColumn` struct to models
- ✅ Added `load_columns()`, `save_column()`, `delete_column()` DB functions
- ✅ Updated `load_workspace_snapshot()` to hydrate columns
- ✅ Updated `WorkspaceSnapshot` to include `columns` field
- ✅ Added `save_column` and `delete_column` Tauri commands
- ✅ Registered commands in `invoke_handler`

### Frontend (TypeScript/React)
- ✅ Removed `COLUMNS_KEY` and `loadColumns()` from store
- ✅ Removed `FOLDERS_KEY`, `loadFolders()`, `saveFolders()` from store
- ✅ Changed colums initial state from `loadColumns()` to `[]`
- ✅ Changed folders initial state from `loadFolders()` to `[]`
- ✅ Removed localStorage sync effect for folders
- ✅ Updated hydration to seed DEFAULT_COLUMNS on first launch
- ✅ Updated hydration to use backend folders exclusively (no merge)
- ✅ Removed localStorage read from `handleUnlock()` (kept cleanup)
- ✅ Updated `toggleNoteEncryption()` to require passphrase with rollback
- ✅ Updated `SafeVaultResetSection.tsx` to clear all localStorage keys and reload
- ✅ Removed dead legacy migration code from store hydration
- ✅ Added `saveWorkspaceColumn()` and `deleteWorkspaceColumn()` commands
- ✅ Updated `WorkspaceSnapshot` interface to include columns

### QA & Testing
- ✅ Created `scripts/qa_lifecycle.py` with 8 inspection checks
- ✅ All tests pass (13/13)
- ✅ All builds pass (Rust + TypeScript)

---

## What's Now Unblocked

✅ **Phase 1: AI Integration Tracks (P1-A, P1-B, P1-C)**
- Caller-aware command boundary (P1-B) can proceed
- Event-driven architecture (P1-C) can proceed
- Both depend on clean storage layers (now locked in)

✅ **Future Phases**
- Phase 2: Encryption hardening
- Phase 3: Vector embeddings (Qdrant/VelesDB)
- Phase 4: Agent memory persistence

---

## Sign-Off

**Gate Status:** ✅ **OPEN**  
**Next:** Proceed with P1-B caller-aware command boundary  
**Archive:** Old `notes_archive_2026-04-08` and `kanban_archive_2026-04-08` folders can be restored or deleted per user choice

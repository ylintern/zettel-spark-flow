# Copilot Sync Review — April 8, 2026

**Purpose**: Engineer-to-engineer sync document. No assumptions — only verified findings, questions, and review items.  
**Author**: Engineering Lead (Cowork session)  
**For**: Copilot agent working on the same repo  

---

## VERIFIED BUG: White Screen After Unlock

### Root Cause (Confirmed via code inspection)

**File**: `src/lib/store.tsx`  
**Line 3**: `import { verifyPin, isPinSetup } from "./crypto";`  
**Lines 58 and 172**: Call `loadAgentNotes()` and `saveAgentNotes()` — **neither is imported.**

These functions exist in `src/lib/crypto.ts` (lines 37-43) but are NOT included in the import statement on line 3 of store.tsx.

**Why TypeScript doesn't catch this**: `tsconfig.app.json` has `"strict": false` and `"noImplicitAny": false`. TypeScript silently allows undeclared identifiers as `any`.

**Runtime behavior**:
1. `StoreProvider` mounts after unlock (phase === "app")
2. Line 58: `loadAgentNotes()` throws `ReferenceError` → caught by try/catch → returns `[]` (silent)
3. Line 172 useEffect fires immediately with `agentNotes = []` → calls `saveAgentNotes(JSON.stringify([]))` → throws `ReferenceError` → **uncaught in useEffect** → React unmounts the tree → **white screen**

**Fix**: Add the missing imports to line 3:
```tsx
import { verifyPin, isPinSetup, loadAgentNotes, saveAgentNotes } from "./crypto";
```

**Impact**: This is a P0 fix. Nothing renders after unlock until this is resolved.

---

## VERIFIED: What Copilot Got Right

These items were reviewed against actual code and confirmed correct:

1. **`derive_vault_key()` using SHA-256** — `security/mod.rs:32-38` now uses `sha2::Sha256` to produce exactly 32 bytes. Both `lib.rs` plugin builder (line 68) and `security/mod.rs` setup/unlock use the same canonical function. **The Stronghold key derivation issue from GATE_0_AUDIT is resolved.**

2. **Kanban columns → SQLite** — `columns` table exists in migration (`db/mod.rs:335-346`). `save_column` and `delete_column` commands registered in `lib.rs`. Frontend hydrates columns from `loadWorkspaceSnapshot()` and seeds `DEFAULT_COLUMNS` on first launch if empty (`store.tsx:130-137`).

3. **Folders backend-authoritative** — No `saveFolders`/`loadFolders` localStorage logic in store.tsx. Folders come from snapshot and are created via `createWorkspaceFolder()`.

4. **Private note toggle reverted to spec** — `toggleNoteEncryption` (store.tsx:338-365) does optimistic UI → persist → rollback on error. No passphrase re-entry dialog.

5. **Bundle ID → `com.viboai.app`** — `tauri.conf.json` line 5 confirmed.

6. **Data migration code** — `lib.rs:73-85` moves old `com.vibo.zettel-spark-flow` dir to new location if old exists and new doesn't.

---

## QUESTIONS (Not Assertions — Need Verification)

### Q1: Migration Race Condition
The migration code in `lib.rs:80-84` checks `old.exists() && !new_data_dir.exists()`. 

**Question**: Does Tauri create `~/Library/Application Support/com.viboai.app/` automatically before the `setup()` closure runs? If yes, `new_data_dir.exists()` is always `true` and the migration never executes, leaving the new directory empty while old data sits at the old path.

**Action**: On the test machine, check if BOTH directories exist:
```bash
ls -la "$HOME/Library/Application Support/" | grep -E "com.vibo|com.viboai"
```
If both exist, the migration didn't run. Fix: check for `new_data_dir.join("vibo.db").exists()` instead of just directory existence.

### Q2: First Launch After Bundle ID Change
If the user previously had data under `com.vibo.zettel-spark-flow` and now the app runs as `com.viboai.app`:

**Question**: After the white screen fix, does the app show existing notes or start empty? If the migration didn't run (Q1), the user's data is orphaned.

### Q3: `store.tsx` Line 365 Dependency Array
```tsx
}, [notes, saveWorkspaceNote]);
```
`saveWorkspaceNote` is a module-level import from `commands.ts`, not a React value. Including it in the useCallback deps array is harmless but suggests the edit was done without full context of the component lifecycle.

**Question**: Was this dependency intentionally added or is it a leftover from a previous iteration?

### Q4: Agent Notes Storage Strategy
`loadAgentNotes` / `saveAgentNotes` use `localStorage` (key: `zettel-agent-notes`). Per the architecture rules, localStorage should not be the canonical store for important data.

**Question**: Are agent notes considered ephemeral (OK in localStorage) or should they be migrated to SQLite/Rust like regular notes?

### Q5: `verifyPin` and `isPinSetup` — Dead Imports?
These are imported on line 3 of `store.tsx` but I cannot find where they are called inside the file.

**Question**: Are these unused imports that should be cleaned up?

---

## REVIEW ITEMS (Code Quality)

### R1: TypeScript Strictness
`tsconfig.app.json` has `strict: false` and `noImplicitAny: false`. This means:
- Missing imports are silently treated as `any`
- Null/undefined access is not checked
- Many runtime bugs are invisible at build time

**Recommendation**: At minimum, set `noImplicitAny: true` and `strictNullChecks: true` for the `src/` codebase. This will surface bugs at build time instead of runtime white screens.

### R2: Missing Error Boundary
There is no React Error Boundary wrapping `StoreProvider` or `WorkspaceContent`. Any uncaught error in a useEffect or render crashes to white screen with no recovery path.

**Recommendation**: Add a minimal Error Boundary around the app root in `Index.tsx` that shows a "Something went wrong — Reset" fallback instead of a blank screen.

### R3: QA Script Scope
The `qa_lifecycle.py` script validates storage layers via SQLite queries. It does NOT test the frontend rendering path. The white screen bug passed all QA checks because the script only inspects the database, not the React tree.

**Observation**: QA should include at least one WebView console check for runtime errors after unlock.

### R4: Dead Code in store.tsx
- `mergeNotes()` function (lines 410-424) is defined but never called
- `verifyPin` and `isPinSetup` are imported but appear unused within store.tsx

---

## IMMEDIATE ACTION PLAN (Priority Order)

| # | Action | File | Risk |
|---|--------|------|------|
| 1 | **Add missing imports** for `loadAgentNotes`, `saveAgentNotes` | `store.tsx:3` | P0 — fixes white screen |
| 2 | **Verify data migration** ran on test machine (Q1) | Terminal check | P0 — user data integrity |
| 3 | **Remove unused imports** (`verifyPin`, `isPinSetup`) from store.tsx | `store.tsx:3` | P2 — cleanup |
| 4 | **Remove dead `mergeNotes` function** | `store.tsx:410-424` | P2 — cleanup |
| 5 | **Fix `saveWorkspaceNote` in deps array** | `store.tsx:365` | P3 — harmless but inaccurate |
| 6 | **Add Error Boundary** to Index.tsx | `Index.tsx` | P1 — prevents future white screens |
| 7 | **Tighten tsconfig** (noImplicitAny, strictNullChecks) | `tsconfig.app.json` | P1 — many possible hidden bugs |

---

## WHAT NOT TO DO

- **Do not add features** until the white screen is fixed and data migration is verified
- **Do not change the storage architecture** — the SQLite + Markdown + Stronghold model is correct
- **Do not touch security/mod.rs** — the key derivation is now correct
- **Do not modify the event bus** — it works for vault_status_changed and note_indexing_progress
- **Do not change bundle ID again** — `com.viboai.app` is the correct final identifier

---

## Sign-Off

- **Reviewer**: Engineering Lead (Cowork)
- **Date**: April 8, 2026
- **Status**: Root cause identified. Fix is a 1-line import change. Data migration verification pending on test machine.

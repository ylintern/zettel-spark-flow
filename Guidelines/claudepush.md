# CLAUDEPUSH — Phase 0 Completion Briefing
**Owner:** Dev Lead (Claude)  
**Date:** 2026-04-08  
**Status:** REVIEW — do not execute

---

## 0. Overview Mindmap

```
Phase 0 Completion
│
├── T-1: Fix Build (stale cache)
│     └── crunchy + rust_decimal artifacts missing
│
├── T-2: Verify Password Persistence [manual QA, no code]
│     └── Stronghold snapshot → lock screen on reopen?
│
├── T-3: Rename Data Paths
│     ├── vault/notes/  →  database/notes/
│     └── vault/kanban/ →  database/tasks/
│
├── T-4: Notes CRUD [depends on T-3]
│     ├── create / edit / delete → .md files
│     └── bootstrap database/notes/original/
│
├── T-5: Tasks/Kanban CRUD [depends on T-3]
│     ├── status = kanban column (todo/in-progress/done)
│     ├── delegated_to field (new DB column)
│     ├── date stamps (created_at, updated_at)
│     └── bootstrap database/tasks/original/
│
└── T-6: Data Isolation Audit [depends on T-4 + T-5]
      ├── notes view: kind == "note" only
      ├── tasks view: kind == "task" only
      └── audit_data_dirs() IPC command
```

---

## 1. Decision Tree — Where to Fix Each Problem

```
Build fails?
 └─ YES: stale cargo artifacts?
         ├─ YES → delete target/debug/build/crunchy-* and rust_decimal-*
         │         then: cargo build
         └─ NO  → read full error output → escalate

Password not persisting after restart?
 └─ Check: does secure-vault.hold file exist in app_local_data_dir?
           ├─ NO  → vault setup didn't save → check security/mod.rs save_snapshot path
           └─ YES → check is_vault_configured() is reading same path
                    └─ paths differ? → fix state.rs AppState snapshot_path init

Notes showing in kanban / tasks showing in notes?
 └─ Fix: store.tsx must expose separate filtered arrays
         notes = workspace.filter(n => n.kind == "note")
         tasks = workspace.filter(n => n.kind == "task")
         Each view component consumes ONLY its array

File not found after create/edit?
 └─ Check: vault/mod.rs note_path() — does it use new database/ prefix?
           └─ NO → update note_path() and task_path() functions
```

---

## 2. Task Index (TODOs)

| # | Task | Depends On | Files Touched | Status |
|---|------|-----------|---------------|--------|
| T-1 | Fix stale cargo cache | — | `target/debug/build/` (delete) | PENDING |
| T-2 | Verify password persistence | T-1 (build works) | None (manual QA) | PENDING |
| T-3 | Rename data paths | T-2 pass | `lib.rs`, `vault/mod.rs`, `state.rs` | PENDING |
| T-4 | Notes CRUD | T-3 | `vault/mod.rs`, `commands/workspace.rs`, `store.tsx` | PENDING |
| T-5 | Tasks/Kanban CRUD + delegated_to | T-3 | `db/mod.rs` (migration), `models/mod.rs`, `vault/mod.rs`, `store.tsx` | PENDING |
| T-6 | Data isolation audit | T-4, T-5 | `commands/workspace.rs` (new cmd), `store.tsx` | PENDING |

---

## 3. DOs and DON'Ts

### ✅ DO
- Clean only the specific broken Cargo build dirs (targeted, not full `cargo clean`)
- Use existing `kind` DB column to separate notes from tasks — it's already there
- Keep notes and tasks in the SAME `notes` SQLite table (unified schema, different kind)
- Bootstrap `original/` subfolders idempotently (safe to run every launch)
- Add `delegated_to` as a nullable column — don't break existing rows
- Emit `vault_status_changed` events consistently after any lock/unlock state change

### ❌ DON'T
- Don't run `cargo clean` — it rebuilds everything from scratch (10–20 min wasted)
- Don't create a separate DB table for tasks — the `kind` field is the correct separation
- Don't store the user's passphrase anywhere — Stronghold uses a derived key (SHA-256 hash)
- Don't hardcode `vault/notes/` paths anywhere new — always resolve from AppState
- Don't show tasks in notes view, ever — enforce at the store layer, not the component layer
- Don't skip audit confirmation steps before marking a task complete

---

## 4. Current vs Target Data Paths

| Item | Current Path | Target Path |
|------|-------------|-------------|
| Notes files | `{app_data}/vault/notes/{id}.md` | `{app_data}/database/notes/{id}.md` |
| Tasks files | `{app_data}/vault/kanban/{id}.md` | `{app_data}/database/tasks/{id}.md` |
| SQLite DB | `{app_data}/vibo.db` | `{app_data}/vibo.db` ✅ (unchanged) |
| Stronghold vault | `{app_data}/secure-vault.hold` | `{app_data}/secure-vault.hold` ✅ (unchanged) |
| Notes bootstrap folder | (none) | `{app_data}/database/notes/original/` |
| Tasks bootstrap folder | (none) | `{app_data}/database/tasks/original/` |

> **Note:** `{app_data}` on macOS dev = `~/Library/Application Support/com.viboai.app/`

---

## 5. Database Schema — Changes Needed for T-5

```sql
-- EXISTING (already in db/mod.rs migration)
CREATE TABLE notes (
  id         TEXT PRIMARY KEY,
  title      TEXT NOT NULL,
  file_path  TEXT NOT NULL UNIQUE,
  folder     TEXT,
  column_id  TEXT,
  position   INTEGER NOT NULL DEFAULT 0,
  kind       TEXT NOT NULL CHECK(kind IN ('note','task')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  is_encrypted INTEGER NOT NULL DEFAULT 0
);

-- NEW MIGRATION NEEDED (T-5)
ALTER TABLE notes ADD COLUMN delegated_to TEXT;
-- Valid values: 'user' | 'agent-a' | 'agent-b' | 'agent-c' | NULL
```

---

## 6. Rust Model Change — T-5

```rust
// models/mod.rs — add field to WorkspaceNote
pub struct WorkspaceNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub column: Option<String>,
    pub position: i64,
    pub is_kanban: bool,
    pub created_at: String,
    pub updated_at: String,
    pub folder: Option<String>,
    pub is_encrypted: Option<bool>,
    pub delegated_to: Option<String>,  // ← NEW
}
```

---

## 7. Frontend Store Change — T-6

```typescript
// store.tsx — expose filtered derived arrays
const notes = workspace.filter(n => n.kind === "note")   // ← notes view only
const tasks = workspace.filter(n => n.kind === "task")   // ← kanban view only
```

---

## 8. Audit Command — T-6

```rust
// commands/workspace.rs — new IPC command
#[tauri::command]
pub async fn audit_data_dirs(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let notes_dir = state.notes_dir.join("original");
    let tasks_dir = state.tasks_dir.join("original");
    Ok(serde_json::json!({
        "notes_dir_exists":     state.notes_dir.exists(),
        "tasks_dir_exists":     state.tasks_dir.exists(),
        "notes_original_exists": notes_dir.exists(),
        "tasks_original_exists": tasks_dir.exists(),
    }))
}
```

---

## 9. Build Fix — Exact Commands

```bash
# Step 1: delete only the broken build artifact dirs
rm -rf src-tauri/target/debug/build/crunchy-*
rm -rf src-tauri/target/debug/build/rust_decimal-*

# Step 2: attempt build
cd src-tauri && cargo build 2>&1 | tail -40
```

> **Why targeted?** The error `couldn't read .../build/crunchy-xxx/out/lib.rs` means the OUT_DIR was deleted but Cargo cached the fact it ran the build script. Deleting the build dir forces Cargo to re-run the build script cleanly.

---

## 10. Phase 0 Complete Gate — Checklist

```
[ ] T-1: cargo build exits 0
[ ] T-2: lock screen shows on app reopen (not onboarding)
[ ] T-3: database/notes/ and database/tasks/ directories created on launch
[ ] T-4: create/edit/delete note → .md file in database/notes/
[ ] T-4: database/notes/original/ folder exists in folder list
[ ] T-5: create/move/delete task → .md file in database/tasks/
[ ] T-5: delegated_to field works (user/agent-a/b/c)
[ ] T-5: 3 default columns bootstrapped (todo, in-progress, done)
[ ] T-5: database/tasks/original/ folder exists in folder list
[ ] T-6: notes view shows ZERO tasks
[ ] T-6: kanban view shows ZERO notes
[ ] T-6: audit_data_dirs() returns all true on fresh launch
```

---

## 11. Interconnection Map

```
T-1 (build fix)
  └─► T-2 (manual QA: password persistence)
        └─► T-3 (rename paths: vault/ → database/)
              ├─► T-4 (notes CRUD + original folder)
              ├─► T-5 (tasks CRUD + delegated_to + original folder)
              └─► T-6 (data isolation audit command)
                    └─► Phase 0 COMPLETE ✓
```

---

## 12. Research Links (for review)

- [Tauri app_local_data_dir docs](https://tauri.app/reference/javascript/api/namespacepath/#applocaldata)  
- [Stronghold Tauri plugin](https://github.com/tauri-apps/tauri-plugin-stronghold) — snapshot save/load lifecycle
- [SQLite ALTER TABLE](https://www.sqlite.org/lang_altertable.html) — adding `delegated_to` column safely
- [ripgrep: searching Rust kind field usage](vscode://search?query=kind%3D%3D) — use Grep in Claude Code

---

## 13. Agents Review Protocol

Before executing each task:
1. Agent reads the specific section above
2. Agent reads the specific source files listed in the task row (Section 2)
3. Agent proposes exact diff / exact command
4. Dev Lead (you) approves
5. Agent executes single task
6. Audit confirmation runs
7. Move to next task

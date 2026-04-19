# Folder Architecture — Physical-First + Reserved Names

**Status:** Locked — Phase 0.5
**Date:** 2026-04-17

## 5 Locked Decisions

1. **Hybrid dual truth, physical-first.**
   - `.md` files + subdirs under `viboai/myspace/` are the **primary** store.
   - SQLite (`viboai/database/vibo.db`) is a **secondary** index.
   - Every write/edit/save/delete first touches FS, then mirrors into SQL via a transaction. On SQL failure the FS write is compensated (rolled back).
   - Obsidian-style: users can edit the vault in Obsidian and the app will (Phase 0.5+ via `notify` watcher) reconcile.

2. **Option B — user folders are real physical subdirs.**
   - User creates folder "Research" ⇒ `fs::create_dir_all(myspace/Research)` + SQL row. Notes placed in it live at `myspace/Research/{uuid}.md`.
   - Rolled out stage-by-stage: notes first, then tasks.
   - No UI regressions — same sidebar/picker shape; only data model tightens.

3. **Reserved folder names.**
   - Never allowed as user-created folders. Never shown in notes/tasks pickers or the notebook sidebar.
   - **Type defaults:** `notes`, `tasks`
   - **Infrastructure (Phase 1+):** `agents`, `skills`, `roles`, `providers`, `tools`, `mcp`, `plugin`
   - Enforced in **backend** (`vault::is_reserved_folder` → `create_folder` rejects) and **frontend** (`src/lib/constants.ts` → `isReservedFolder`; store exposes `userFolders = folders.filter(!isReservedFolder)`).

4. **Two top-level trees under `viboai/`.**
   - `viboai/myspace/` — Obsidian-compatible vault. User-visible `.md` files + subdirs.
   - `viboai/database/` — internal SQLite + future RAG/Swiftide/VelesDB artifacts. **Never surfaced in UI.**
   - The two trees do not cross.

5. **"No folder" is never a valid state.**
   - Every `.md` lives in a physical subdir of `myspace/`.
   - Default at creation time: `is_kanban ? "tasks" : "notes"`.
   - `write_note` computes the subdir from `note.folder`, falling back to the default — never empty.

## UI Filter Rule (by view type)

| Context | Folder picker shows |
|---|---|
| Notes editor (`!isKanban`) | `["notes", ...userFolders]` — pre-selected to `note.folder ?? "notes"` |
| Tasks editor (`isKanban`) | `["tasks", ...userFolders]` — pre-selected to `note.folder ?? "tasks"` |
| Kanban quick-add | No prompt — always `folder: "tasks"` |
| Notebook sidebar | Default "notes" group (implicit) + `userFolders` |

Reserved infrastructure names (`agents`, `skills`, `roles`, `providers`, `tools`, `mcp`, `plugin`) never appear in any folder picker or sidebar. They exist on disk for Phase 1+ wiring.

## Invariants (for agents inheriting this project)

| Rule | Detail |
|---|---|
| Physical-first | FS is truth. SQL mirrors FS on every write/delete. Startup snapshot ensures any SQL-known folder has a physical dir (recovery path). |
| Reserved list | See §3. Backend rejects. Frontend filters. |
| Two-tree | `myspace/` = user. `database/` = internal. Never mix. |
| Default folder | `is_kanban ? "tasks" : "notes"`. Never null. |
| Folder = physical | User folder "foo" ⇒ `myspace/foo/` exists. |

## Dev vs Prod launch (Phase 0.5 addendum)

- **Canonical dev command:** `bun run tauri:dev` — ALWAYS use this.
  - Loads `src-tauri/tauri.dev.conf.json` which overrides the bundle identifier to `com.viboai.app.dev`.
  - Runs `scripts/dev-prelaunch.sh` automatically via the `pretauri:dev` npm hook (kills stale tauri/vite/app processes + frees port 8080).
  - Cargo profile: debug. This is the reason dev feels slow vs a `tauri build` release (especially crypto) — it has nothing to do with the identifier.
- **Never** run `bun run tauri dev` (without the colon). That skips the config override, writes data into the prod bundle `com.viboai.app`, and resets WebKit localStorage (making onboarding re-appear).
- Production builds (future): `bun run tauri:build` → release profile + identifier `com.viboai.app`.
- Data roots on macOS:
  - Dev: `~/Library/Application Support/com.viboai.app.dev/`
  - Prod: `~/Library/Application Support/com.viboai.app/`
- Vite uses `strictPort: true` on 8080 as a loud-fail safety net — the prelaunch script ensures the port is free before vite binds it.

## Out of scope (deferred)

- `notify`-based FS watcher for Obsidian cross-process edits → Phase 0.5 follow-up
- Folder rename / merge UI → Phase 1
- Folder-level encryption toggle → Phase 2/3
- Swiftide / VelesDB artifacts under `database/` → Phase 3+

## Code Pointers

- Backend: `src-tauri/src/vault/mod.rs` — `RESERVED_FOLDER_NAMES`, `is_reserved_folder`, `ensure_vault_dirs`, `ensure_folder_dir`, `write_note`
- Backend: `src-tauri/src/commands/workspace.rs::create_folder` — reserved guard + `fs::create_dir_all` before SQL
- Backend: `src-tauri/src/db/mod.rs::init_default_folders` — no-op (defaults are physical, not SQL rows); `load_workspace_snapshot` — defensive mkdir recovery
- Frontend: `src/lib/constants.ts` — `RESERVED_FOLDER_NAMES`, `isReservedFolder`, `defaultFolderFor`
- Frontend: `src/lib/store.tsx` — `userFolders` derived; `addNote` sets default folder
- Frontend: `src/components/NoteEditor.tsx`, `NewNoteDialog.tsx`, `NotebookView.tsx`, `KanbanView.tsx`

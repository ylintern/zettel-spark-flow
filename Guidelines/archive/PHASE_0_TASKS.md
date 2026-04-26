# Phase 0: Implementation Tasks
## Executable Checklist (25 Tasks, 5 EPICs)

**Timeline:** 7-8 days  
**Owner:** Engineering Team  
**Status:** Ready to assign  

---

## EPIC 1: Validation & Setup (0.5 days)

### Task 1.1: Validate Crate Selections ⬜
- **Owner:** Engineering Lead
- **Deliverable:** Decision locked on YAML parser + dependencies
- [ ] Review gray_matter-rs vs serde_yaml options
- [ ] Confirm tauri-plugin-fs and tauri-plugin-dialog are suitable
- [ ] Check for conflicts or missing dependencies
- **Acceptance:** Crates selected, Cargo.toml plan drafted

### Task 1.2: Validate Vault Directory Structure ⬜
- **Owner:** Product + Engineering
- **Deliverable:** Final directory layout confirmed
- [ ] Decide: keep .vibo-meta or pure Obsidian?
- [ ] Any other folders needed?
- [ ] Lock structure in code comments
- **Acceptance:** Directory structure decided

### Task 1.3: Answer 5 Decisions ⬜
- **Owner:** Product/Engineering Lead
- **Deliverable:** All decisions locked
- [ ] YAML parser: gray_matter-rs or serde_yaml?
- [ ] Vault metadata: keep .vibo-meta/config.json or not?
- [ ] List performance: simple (read files) or indexed (index.json)?
- [ ] Concurrent edits: last-write-wins or conflict prompt?
- [ ] Tags/Folders: keep UI or defer to Phase 1?
- **Acceptance:** All 5 answered in writing

### Task 1.4: Create Phase 0 Branch ⬜
- **Owner:** Engineering Lead
- **Deliverable:** Branch created, ready for work
- [ ] Create branch: `phase-0/filesystem-first` from main
- [ ] Document decisions in commit message
- [ ] Push branch to remote
- **Acceptance:** Branch exists, all engineers can pull it

---

## EPIC 2: Vault Initialization (1.5 days)

| Task | Owner | Status | Deps |
|------|-------|--------|------|
| 2.1: Add Tauri plugins to Cargo.toml | Eng | ⬜ | 1.1 |
| 2.2: Implement vault.rs module | Eng | ⬜ | 1.2 |
| 2.3: Expose vault commands (IPC) | Eng | ⬜ | 2.2 |
| 2.4: Wire OnboardingWizard → init_vault | FE | ⬜ | 2.3 |
| 2.5: Test end-to-end (Desktop) | QA | ⬜ | 2.4 |

### Details:

**2.1: Add Tauri Plugins to Cargo.toml**
- [ ] Add tauri-plugin-fs
- [ ] Add tauri-plugin-dialog
- [ ] Add YAML parser crate (from 1.1 decision)
- [ ] Add uuid crate
- [ ] Run `cargo build` successfully
- **Acceptance:** Cargo.toml updated, build passes

**2.2: Implement vault.rs Module**
- [ ] Create `src-tauri/src/vault.rs`
- [ ] `init_vault(vault_path) → VaultMetadata`
  - [ ] Creates /notes, /tasks, .vibo-meta directories
  - [ ] Generates vault_id
  - [ ] Returns metadata
- [ ] `get_vault_path() → String`
- [ ] `validate_vault(path) → bool`
- [ ] Proper error handling
- **Acceptance:** Module compiles, logic documented

**2.3: Expose Vault Commands (IPC)**
- [ ] Create `src-tauri/src/commands/vault.rs`
- [ ] `#[tauri::command] init_vault_cmd(...)`
- [ ] `#[tauri::command] get_vault_meta(...)`
- [ ] Register in main.rs tauri builder
- [ ] Test with invoke() from TSX
- **Acceptance:** Commands accessible via invoke()

**2.4: Wire OnboardingWizard → init_vault**
- [ ] Update `src/components/OnboardingWizard.tsx`
- [ ] Add "Select Vault" button (native file picker)
- [ ] Call `invoke('init_vault_cmd', { path })`
- [ ] Handle response, store vault_id in localStorage
- [ ] Show success message
- [ ] Handle errors gracefully
- **Acceptance:** User can select directory, vault created on disk

**2.5: Test End-to-End (Desktop)**
- [ ] Run app, go through onboarding
- [ ] Select test directory (~test-vault/)
- [ ] Verify /notes, /tasks exist on disk
- [ ] Close + reopen app, vault still accessible
- [ ] Test on macOS, Windows, Linux
- **Acceptance:** All tests pass

---

## EPIC 3: Note CRUD (2 days)

| Task | Owner | Status | Deps |
|------|-------|--------|------|
| 3.1: Lock YAML Schema | Eng | ⬜ | 1.3 |
| 3.2: Implement note.rs | Eng | ⬜ | 2.2 |
| 3.3: Expose note commands (IPC) | Eng | ⬜ | 3.2 |
| 3.4: Wire TSX components | FE | ⬜ | 3.3 |
| 3.5: Test end-to-end (Notes) | QA | ⬜ | 3.4 |

### Details:

**3.1: Lock YAML Schema**
- [ ] Finalize all fields (title, id, type, folder, tags, created, modified)
- [ ] Test with Obsidian desktop (parse without errors)
- [ ] Create schema documentation
- [ ] Lock in code comment in note.rs
- **Acceptance:** Schema locked, tested with Obsidian

**3.2: Implement note.rs**
- [ ] Create `src-tauri/src/note.rs`
- [ ] `create_note(title, folder) → NoteMetadata`
  - [ ] Generate UUID
  - [ ] Create YAML frontmatter
  - [ ] Write {uuid}.md to /notes/
  - [ ] Return metadata
- [ ] `read_note(note_id) → NoteContent`
  - [ ] Read .md file
  - [ ] Parse YAML frontmatter
  - [ ] Extract body markdown
  - [ ] Return structured data
- [ ] `update_note(note_id, content) → Result<()>`
  - [ ] Parse new content
  - [ ] Update frontmatter (timestamps, etc.)
  - [ ] Rewrite .md safely
- [ ] `delete_note(note_id) → Result<()>`
  - [ ] Delete {uuid}.md
- [ ] `list_notes(folder?) → Vec<NoteMetadata>`
  - [ ] Scan /notes directory
  - [ ] Parse metadata from each .md
  - [ ] Filter by folder if provided
  - [ ] Return sorted list
- **Acceptance:** All functions compile, error handling in place

**3.3: Expose Note Commands (IPC)**
- [ ] Create `src-tauri/src/commands/note.rs`
- [ ] Wrap each function in `#[tauri::command]`
- [ ] Register all commands in main.rs
- **Acceptance:** Commands accessible via invoke()

**3.4: Wire TSX Components**
- [ ] Update `NewNoteDialog.tsx`
  - [ ] Call `invoke('create_note', { title, folder })`
- [ ] Update `NoteEditor.tsx`
  - [ ] On mount: `invoke('read_note', { id })`
  - [ ] On save: `invoke('update_note', { id, content })`
  - [ ] On delete: `invoke('delete_note', { id })`
- [ ] Update `NotebookView.tsx`
  - [ ] On mount: `invoke('list_notes', { folder })`
  - [ ] Populate UI with returned metadata
- [ ] Test all interactions
- **Acceptance:** UI calls Rust commands, data flows correctly

**3.5: Test End-to-End (Notes)**
- [ ] Create note via UI button
  - [ ] Verify .md file appears on disk
  - [ ] Verify YAML frontmatter is valid
- [ ] Edit note, save
  - [ ] Verify file updated on disk
  - [ ] Verify modified timestamp changed
- [ ] Delete note
  - [ ] Verify file deleted from disk
- [ ] Open vault in Obsidian desktop
  - [ ] Verify notes parse without errors
  - [ ] Verify markdown renders correctly
- [ ] Manually edit .md in text editor
  - [ ] Reload ViBo
  - [ ] Verify changes visible in UI
- **Acceptance:** All tests pass

---

## EPIC 4: Kanban/Task CRUD (1.5 days)

| Task | Owner | Status | Deps |
|------|-------|--------|------|
| 4.1: Implement task.rs | Eng | ⬜ | 3.2 |
| 4.2: Expose task commands (IPC) | Eng | ⬜ | 4.1 |
| 4.3: Wire KanbanView.tsx | FE | ⬜ | 4.2 |
| 4.4: Test end-to-end (Kanban) | QA | ⬜ | 4.3 |

### Details:

**4.1: Implement task.rs**
- [ ] Create `src-tauri/src/task.rs`
- [ ] Similar to note.rs but for tasks in /tasks/
- [ ] `create_task(title, column, position) → TaskMetadata`
- [ ] `read_task(task_id) → TaskContent`
- [ ] `update_task(task_id, content) → Result<()>`
- [ ] `delete_task(task_id) → Result<()>`
- [ ] `list_tasks() → Vec<TaskMetadata>`
- [ ] `move_task(task_id, dest_column, position) → Result<()>`
  - [ ] Update column and position in frontmatter
- **Acceptance:** All functions compile

**4.2: Expose Task Commands (IPC)**
- [ ] Create `src-tauri/src/commands/task.rs`
- [ ] Register in main.rs
- **Acceptance:** Commands accessible

**4.3: Wire KanbanView.tsx**
- [ ] On mount: `invoke('list_tasks', {})`
- [ ] On drag-drop: `invoke('move_task', { task_id, dest_column, position })`
- [ ] On create: `invoke('create_task', { title, column, position })`
- [ ] On delete: `invoke('delete_task', { id })`
- [ ] Re-render after each operation
- **Acceptance:** Kanban board syncs with filesystem

**4.4: Test End-to-End (Kanban)**
- [ ] Create task via Kanban UI
  - [ ] Verify .md created in /tasks
  - [ ] Verify frontmatter has column + position
- [ ] Drag task between columns
  - [ ] Verify column + position updated
  - [ ] Verify file rewritten on disk
- [ ] Open vault in Obsidian Kanban plugin
  - [ ] Verify board renders
  - [ ] Verify task positions correct
- [ ] Delete task
  - [ ] Verify file deleted
- **Acceptance:** Tests pass

---

## EPIC 5: Cleanup & Validation (1.5 days)

| Task | Owner | Status | Deps |
|------|-------|--------|------|
| 5.1: Remove old code | Eng | ⬜ | 4.4 |
| 5.2: Verify no TS fs access | Eng | ⬜ | 4.4 |
| 5.3: Test Obsidian compatibility | QA | ⬜ | 4.4 |
| 5.4: Document Phase 0 manifest | Eng | ⬜ | 4.4 |
| 5.5: Code review & merge | Eng | ⬜ | 5.4 |

### Details:

**5.1: Remove Old Code**
- [ ] Identify SQLite-related code (if present)
- [ ] Archive to separate branch or document
- [ ] Remove from active codebase
- [ ] Verify no compilation errors
- **Acceptance:** Old code cleaned, no dangling references

**5.2: Verify No TS Filesystem Access**
- [ ] Run: `grep -r "from ['\"]fs" src/`
- [ ] Run: `grep -r "require.*fs" src/`
- [ ] Run: `grep -r "fs\." src/` (only comments)
- [ ] Run: `grep -r "node:" src/` (none expected)
- **Acceptance:** 0 filesystem access in TSX

**5.3: Test Obsidian Compatibility**
- [ ] Create sample vault in ViBo
- [ ] Open in Obsidian desktop
  - [ ] YAML parses without errors
  - [ ] Markdown renders correctly
  - [ ] Frontmatter fields preserved
- [ ] Try Obsidian mobile (if available)
- [ ] Try Kanban plugin (if vault has tasks)
- [ ] Manually edit .md in Obsidian, reload ViBo
  - [ ] Changes visible
- **Acceptance:** All compatibility tests pass

**5.4: Document Phase 0 Manifest**
- [ ] Create `Guidelines/current-phase/PHASE_0_MANIFEST.md`
- [ ] Lock YAML schema
- [ ] Document file structure
- [ ] Document IPC command surface
- [ ] List Obsidian compatibility notes
- [ ] List known Phase 0 limitations
- **Acceptance:** Manifest complete, reviewed

**5.5: Code Review & Merge**
- [ ] Full code review (Rust + TSX)
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Commit messages clear
- [ ] Merge `phase-0/filesystem-first` → `main`
- [ ] Tag release: `v0.1.0-phase-0` (optional)
- **Acceptance:** Phase 0 merged, ready for Phase 1

---

## Task Dependencies & Critical Path

```
EPIC 1 (Validation)
    ↓
├─→ EPIC 2 (Vault) → EPIC 2.5 (Test)
├─→ EPIC 3 (Notes) → EPIC 3.5 (Test)
└─→ EPIC 4 (Tasks) → EPIC 4.4 (Test)
    ↓
EPIC 5 (Cleanup & Merge)
```

**Parallel After EPIC 1:** Can run EPIC 2, 3, 4 in parallel (different modules)  
**Critical Path:** EPIC 1 → any of 2/3/4 → 5

---

## Timeline Estimate

| EPIC | Days | Critical |
|------|------|----------|
| 1: Validation | 0.5 | No |
| 2: Vault | 1.5 | No |
| 3: Notes | 2.0 | **Yes** |
| 4: Kanban | 1.5 | No |
| 5: Cleanup | 1.5 | No |
| **Total** | **7-8** | Solid |

---

## Success Criteria: All Checklist Items Complete

When all checkboxes ☑️ are checked:

✅ Physical .md files created in user vault  
✅ YAML frontmatter valid + Obsidian-compatible  
✅ All file I/O in Rust (zero TS filesystem)  
✅ TSX UI unchanged (same UX, different backend)  
✅ Obsidian can read vault without errors  
✅ Files portable (move vault to any device)  
✅ IPC command surface clean  
✅ Code reviewed + merged to main  

**→ Phase 0 Complete**

---

## Next Phase

Once Phase 0 merged:
- **Phase 1:** Encryption (Stronghold, AES-256)
- **Phase 2:** Chat/Inference (local or cloud)
- **Phase 3:** Swiftide Indexing
- **Phase 4:** Agents + Tool Calls

See [Roadmap Overview](../README.md) for full vision.

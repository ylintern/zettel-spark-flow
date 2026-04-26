# Phase 0: Filesystem-First Architecture
## Strategy & Design Document

**Status:** Ready for Engineering Review  
**Timeline:** 7-8 days  
**Owner:** Engineering Team  

---

## Overview

Phase 0 establishes the foundation: physical .md files as single source of truth, Rust backend handling all I/O, TSX UI as presentation layer only.

**User Flow:** TSX UI → invoke() → Tauri backend → Physical .md files on disk

**Goal:** Notes and tasks are created/edited/deleted as Obsidian-compatible .md files. User can take vault to another device and open it in Obsidian unchanged.

---

## Architecture Decision (Locked)

### ✅ Single Source of Truth: Physical .md Files
- **Before:** Abstract vault + SQLite + optional files = fragile
- **After:** Physical files = canonical, Rust reads/writes them
- **Benefit:** Portable, Obsidian-compatible, simple

### ✅ TSX as Dumb Presentation Layer
- No filesystem access
- Only calls Rust via invoke()
- Displays what's on disk
- No business logic

### ✅ File Format: Obsidian-Compatible
- UUID.md filenames
- Standard YAML frontmatter (Obsidian ignores unknown fields)
- Regular markdown body
- No proprietary extensions

---

## File Structure (Phase 0 Output)

```
{vault_dir}/
├── notes/
│   ├── 550e8400-e29b-41d4.md
│   │   ├── Frontmatter (YAML)
│   │   │   ├── title: Note Title
│   │   │   ├── id: uuid
│   │   │   ├── type: note
│   │   │   ├── folder: optional-folder
│   │   │   ├── tags: []
│   │   │   ├── created: ISO8601
│   │   │   └── modified: ISO8601
│   │   └── Body (Markdown)
│   └── {more notes}
├── tasks/
│   ├── {uuid}.md (similar structure, type: task)
│   └── {more tasks}
└── .vibo-meta/ (optional, for vault config)
    └── config.json
```

---

## User Flows (Phase 0 Only)

### Flow 1: Create Note
```
TSX: User clicks "New Note" → enters title
  ↓
invoke('create_note', { title, folder })
  ↓
Rust: Generate UUID, write {uuid}.md with frontmatter
  ↓
Return: NoteMetadata { id, title, created, ... }
  ↓
TSX: Update state, render in notebook view
```

### Flow 2: Edit & Save Note
```
TSX: User opens note
  ↓
invoke('read_note', { note_id })
  ↓
Rust: Read .md, parse frontmatter + body
  ↓
Return: { frontmatter, body }
  ↓
TSX: Display content
  ↓
User edits, auto-save fires:
invoke('update_note', { note_id, content })
  ↓
Rust: Parse frontmatter, rewrite .md with updated timestamp
  ↓
Return: Result<()>
```

### Flow 3: Delete Note
```
TSX: User clicks delete, confirms
  ↓
invoke('delete_note', { note_id })
  ↓
Rust: Delete {uuid}.md
  ↓
TSX: Remove from state, close editor
```

### Flow 4: List Notes
```
TSX: Mount NotebookView
  ↓
invoke('list_notes', { folder })
  ↓
Rust: Scan /notes directory, parse all .md files
  ↓
Return: Vec<NoteMetadata>
  ↓
TSX: Render list
```

### Flow 5-8: Kanban Tasks (Similar)
- Create task → file in /tasks
- Move task → update column + position in frontmatter
- Delete task → delete file
- List tasks → scan /tasks directory

---

## YAML Frontmatter Schema (Locked)

Every .md file starts with:

```yaml
---
title: (string, required)
id: (uuid, auto-generated)
type: note | task
folder: (optional string)
tags: (array, optional)
created: (ISO8601 timestamp)
modified: (ISO8601 timestamp)
---
```

**Obsidian-native:** title, tags  
**ViBo-extended:** id, type, folder, modified (Obsidian ignores unknowns)

---

## Tauri IPC Command Surface

### Vault Commands
```rust
init_vault(path: String) → VaultMetadata
get_vault_meta() → VaultMetadata
```

### Note Commands
```rust
create_note(title, folder) → NoteMetadata
read_note(note_id) → NoteContent { frontmatter, body }
update_note(note_id, content) → Result<()>
delete_note(note_id) → Result<()>
list_notes(folder?) → Vec<NoteMetadata>
```

### Task Commands
```rust
create_task(title, column, position) → TaskMetadata
read_task(task_id) → TaskContent
update_task(task_id, content) → Result<()>
delete_task(task_id) → Result<()>
list_tasks() → Vec<TaskMetadata>
move_task(task_id, dest_column, position) → Result<()>
```

---

## Technical Requirements

### Req A: Vault Filesystem Foundation
- User selects directory via native file picker (tauri-plugin-dialog)
- Rust creates /notes, /tasks, .vibo-meta/ directories
- Persist vault path for future sessions

### Req B: Markdown File Creation
- Rust generates UUID for each note
- Creates YAML frontmatter with proper formatting
- Writes {uuid}.md to disk safely
- Returns metadata to TSX

### Req C: YAML Parsing
- Parse frontmatter without destroying markdown body
- Support safe updating of frontmatter
- Handle edge cases (special characters, quotes, etc.)

### Req D: File I/O
- All filesystem operations in Rust
- Zero Node.js fs access in TSX
- Proper error handling and propagation

### Req E: Obsidian Compatibility
- Files readable by Obsidian (test with Obsidian desktop)
- YAML frontmatter valid and parseable
- Markdown body unmodified
- Files portable (user can move vault to another device)

---

## Crate Selection (Engineering to Validate)

### Mobile Filesystem Access
- **tauri-plugin-fs** — Read/write files ✓
- **tauri-plugin-dialog** — Native folder picker ✓

### YAML Parsing (Choose One)
- [ ] **gray_matter-rs** — JavaScript-like, drop-in replacement
- [ ] **serde_yaml** + manual parsing — Rust-native approach

### Other Dependencies
- **uuid** — Generate note/task IDs
- **serde** — Serialize/deserialize
- **chrono** — Timestamps

---

## Engineering Decision Points

| Decision | Option A | Option B |
|----------|----------|----------|
| YAML Parser | gray_matter-rs (drop-in) | serde_yaml + regex (Rusty) |
| Vault Metadata | Keep .vibo-meta/config.json | Pure Obsidian (no .vibo-meta) |
| List Performance | Read all files each time | Maintain .vibo-meta/index.json |
| Concurrent Edits | Last-write-wins (Phase 0) | User conflict prompt (Phase 1) |
| Tags/Folders | Keep existing UI | Defer to Phase 1 |

---

## Success Criteria

- ✅ Physical .md files created in user-selected vault directory
- ✅ Files have valid YAML frontmatter and Obsidian-compatible format
- ✅ User can create/edit/delete notes entirely via TSX UI
- ✅ All file I/O in Rust (zero Node.js fs in TSX)
- ✅ Obsidian can open vault without errors
- ✅ Files portable (user can move vault to any device)
- ✅ IPC command surface clean and documented
- ✅ No encryption, no inference, no agents (Phase 1+)

---

## What Phase 0 Does NOT Include

- ❌ Encryption (Phase 1)
- ❌ File watcher (auto-detect external edits)
- ❌ Chat/inference (Phase 2)
- ❌ Swiftide indexing (Phase 3)
- ❌ Agents (Phase 4)
- ❌ Tag/folder creation UI (use existing read-only for now)
- ❌ Performance optimization (simple file reads acceptable)

---

## Next Steps

1. **Engineering:** Review this document
2. **Engineering:** Validate 5 decision points above
3. **Engineering:** Decide on crates
4. **Lead:** Create branch `phase-0/filesystem-first`
5. **Team:** Execute PHASE_0_TASKS.md

---

## Related Documents

- [PHASE_0_TASKS.md](PHASE_0_TASKS.md) — Execution checklist (25 tasks, 5 EPICs)
- [PHASE_0_CONFLICTS.md](PHASE_0_CONFLICTS.md) — Architecture conflict analysis
- [PHASE_0_MANDATE_AUDIT.md](PHASE_0_MANDATE_AUDIT.md) — Validation checklist

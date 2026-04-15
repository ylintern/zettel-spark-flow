CRITICAL CONFLICTS: User Memories vs. Phase 0 Mandate
Status: BLOCKING ISSUES IDENTIFIED
Authority: Product Coherence Review

🚨 CONFLICT 1: SQLite Metadata vs. Markdown Source of Truth
What User Memories Say:

SQLite schema covers notes, note_tags (many-to-many), folders, kanban/tasks, and settings. load_workspace_snapshot reads notes + tags + content; save_note upserts metadata and writes markdown.

Interpretation:

Notes have a dual representation:

Filesystem: UUID.md with YAML frontmatter + body
SQLite: metadata row in notes table


On load, the app reads both and reconciles them

What Phase 0 Mandate Says:

All filesystem logic MUST be in Rust... Rust must write a file formatted exactly like this

Interpretation:

Implied: Filesystem is the single source of truth
SQLite is... what? Secondary? Caching layer? Unspecified.

The Problem:
If they diverge, which wins?
Scenario:

User opens note in ViBo (reads from SQLite metadata)
User closes ViBo
User opens note in Obsidian on another device (modifies .md file directly)
User reopens ViBo
Does ViBo read from SQLite (stale) or re-sync from filesystem (correct)?

Mandate Ambiguity:

Does Phase 0 include a file watcher that syncs filesystem changes → SQLite?
If yes: Is it recursive? Does it trigger during app backgrounding?
If no: ViBo data becomes inconsistent with Obsidian edits.

INTERROGATION:
[ ] SQLite is primary source of truth (filesystem is cache)
    - Implication: Obsidian changes are lost on next ViBo session
    - Violates "Obsidian compatible"

[ ] Filesystem is primary source of truth (SQLite is cache)
    - Implication: Must implement file watcher + sync logic
    - Not mentioned in mandate (scope creep?)

[ ] Bidirectional sync (complex conflict resolution)
    - Implication: Last-write-wins? Timestamp-based? User prompt?
    - Adds significant complexity to Phase 0

🚨 CONFLICT 2: Encryption Within Phase 0?
What User Memories Say:

Private notes encrypted with AES-256-GCM in Rust before disk write, prefixed !vibo-encrypted:v1:
SecurityState uses Stronghold vault locked until explicit unlock

Interpretation:

Encryption is a core VIBO feature
Notes are stored encrypted on disk
User must unlock the vault to read

What Phase 0 Mandate Says:

Phase 0 Scope Lock – Vault, Notes, and Kanban Tasks ONLY
(Silent on encryption)

Interpretation:

Encryption status: Phase 0 or Phase 1?
If Phase 0: Must include vault unlock ceremony
If Phase 1: Phase 0 notes are stored unencrypted, then encrypted in Phase 1?

The Problem:
If encryption is Phase 0, the mandate is incomplete. If Phase 1, users are storing plaintext notes.
Mandate Gaps:

No mention of vault setup (passphrase/biometric unlock)
No mention of SecurityState initialization
No mention of Stronghold integration
No mention of !vibo-encrypted:v1: prefix handling

INTERROGATION:
[ ] Encryption is Phase 0 (core feature)
    - Missing from mandate: SecurityState setup, passphrase input, vault unlock
    - Adds ~200 lines Rust + UI complexity (biometric prompt, passphrase entry)
    - Timeline impact: +3 days

[ ] Encryption is Phase 1 (deferred)
    - Phase 0 stores plaintext markdown (security risk for real users)
    - User memories are misleading (say encryption is core, but it's deferred)
    - On Phase 1 cutover, must retrofit encryption (complex file migration)

[ ] Encryption is Phase 0 (but note content only, not vault setup)
    - Partial: Note bodies encrypted, metadata in SQLite plaintext
    - Confusing: Which fields are encrypted? Tags? Folder? Timestamps?

🚨 CONFLICT 3: Kanban Column Counts
What Phase 0 Mandate Says:
markdown## To Do 1
- [ ] Explore ViBo features

## In Progress 0
Interpretation:

Column header format: ## ColumnName Count
The count is... what? Task count? Auto-updated?

What User Memories Say:
(Silent on this specific format)
The Problem:
Are these counts authoritative, or just display hints?
Scenario:

User creates a Kanban board with "To Do 1" (1 task in column)
User drags task to "In Progress"
Should the file become:

## To Do 0 and ## In Progress 1? (Auto-update)
## To Do 1 and ## In Progress 0? (Counts drift)
## To Do and ## In Progress? (Remove counts entirely)



Mandate Ambiguity:

No algorithm specified for updating counts
Obsidian Kanban plugin documentation doesn't require counts
If counts drift, does Obsidian Kanban still parse it? Or does it break?

INTERROGATION:
[ ] Counts are auto-synced (counts always match actual tasks)
    - Implementation: After every move_task, parse column, recount, rewrite
    - Risk: Off-by-one errors if task parsing is buggy
    - Timeline: +1 day

[ ] Counts are hints (never synced, can drift)
    - Implementation: Ignore counts during parsing; generate on export
    - Risk: User confusion (counts don't match)
    - Implication: Obsidian Kanban might break if counts are wrong

[ ] Remove counts entirely (simplify)
    - Implementation: `## ColumnName` (no count suffix)
    - Benefit: Simpler parsing, less error-prone
    - Tradeoff: Breaks Obsidian Kanban plugin compatibility (if plugin expects counts)

🚨 CONFLICT 4: YAML Frontmatter: Fields and Scope
What Phase 0 Mandate Says:
markdown---
id: 550e8400-e29b-41d4
title: Welcome to ViBo
created: 2026-04-15T18:30:00Z
tags: []
---
Fields in mandate: id, title, created, tags (4 fields)
What User Memories Say:

Notes stored as notes/{id}.md with YAML frontmatter (id, kind, title, column, position, is_encrypted, folder, tags)

Fields in memories: id, kind, title, column, position, is_encrypted, folder, tags (8 fields)
The Problem:
Which is the actual schema?
Mandate Gaps (vs. Memories):

 kind – note type (e.g., "note", "task", "idea")? Mandatory?
 column – which Kanban column? Only for tasks?
 position – order within column? Needed for drag-drop stability?
 is_encrypted – flag for encryption? Part of Phase 0?
 folder – which folder? Mandatory?
 updated – last modified timestamp? Needed for list_notes()?

INTERROGATION:
YAML Schema (Choose One):

[ ] Minimal (mandate): id, title, created, tags
    - Implication: Simplest, easiest to parse, but loses Kanban metadata
    - Problem: How is column/position stored if not in frontmatter?
              (Answer: Implied in markdown structure of .md file, not frontmatter)

[ ] Extended (memories): id, kind, title, column, position, is_encrypted, folder, tags, updated
    - Implication: Complete, but larger files, more parsing surface area
    - Problem: Some fields (column, position) might be redundant
              (they're also embedded in markdown structure)

[ ] Hybrid: id, title, folder, tags, updated (plus optional encryption/kind fields)
    - Recommendation: Include folder + updated for Phase 0
    - Column/position: Store in markdown structure only (not redundant)
    - Encryption: Phase 1 (add is_encrypted field later)

🚨 CONFLICT 5: File Browser Access (iOS Sandboxing)
What Phase 0 Mandate Says (Acceptance Criterion 4.2):

I can close ViBo, open an external file browser on the phone, go to the Vault folder, open /Notes, and physically see the .md file there.

Interpretation:

Vault files are accessible to other apps (Obsidian, Files app, etc.)

What Mobile Reality Says:

iOS: App files are sandboxed in the app container. To access them from other apps, you must:

Use "Files" app via UIDocumentPickerViewController (manual sharing)
OR store vault in iCloud Drive (shared across apps)
OR use "File Sharing" (iTunes, manual setup)


Android: Files in getExternalFilesDir() are accessible to other apps (less strict sandbox)

The Problem:
The mandate assumes universal filesystem access that iOS does NOT provide by default.
Mandate Gaps:

No mention of iOS file sharing setup
No mention of iCloud Drive integration
No mention of requiring "File Sharing" capability in Info.plist

INTERROGATION:
[ ] Vault path is app-private (inside app container)
    - iOS acceptable, but breaks Obsidian cross-app access
    - Android works fine
    - Workaround: Use Obsidian sync (cloud), not local folder access

[ ] Vault path is shared (iCloud Drive / Android external storage)
    - iOS: Requires `NSUbiquitousContainers` + CloudKit setup (non-trivial)
    - Android: Works, but requires explicit user permission grant
    - Timeline impact: +5 days for iOS setup, +1 day for Android

[ ] Vault path supports manual sharing (iTunes/Files app)
    - iOS: Requires `UIFileSharingEnabled` in Info.plist
    - User must manually export .md files to access in Obsidian
    - Clunky but functional
    - Timeline impact: +1 day for iOS setup
CRITICAL DECISION: Which file access model is VIBO committed to?

🚨 CONFLICT 6: Tags and Folders
What Phase 0 Mandate Says:
markdowntags: []
Interpretation:

Tags exist
Format: empty array []

What User Memories Say:

note_tags (many-to-many)

Interpretation:

Tags are stored in a separate SQLite table
Each note can have multiple tags
Tags are queryable/indexable

What Phase 0 Mandate Does NOT Specify:

Can users CREATE new tags? Or only apply existing ones?
Are tags displayed in the UI?
Can notes be filtered by tag?
Are tags stored in markdown only, or synced to SQLite?

Similarly for Folders:

Mandate is silent
Memories mention folder field
Are folders represented as:

Subdirectories? (/Notes/ProjectA/noteID.md)
Or metadata field only? (folder: "ProjectA" in YAML)



INTERROGATION:
TAGS:
[ ] Full implementation (create, apply, filter, display)
    - Timeline: +2 days for CRUD + UI

[ ] Minimal implementation (apply existing tags only, no creation/filtering)
    - Timeline: +0.5 days
    - Note: Doesn't match "collaborative" user story

[ ] Deferred to Phase 1
    - Phase 0 scope: ignore tags field

FOLDERS:
[ ] Full implementation (nested folder UI, subdirectories on disk)
    - Timeline: +2 days

[ ] Metadata-only (folder field in YAML, no UI)
    - Timeline: +0.5 days
    - Note: User can't navigate folders in app, but files are organized on disk

[ ] Deferred to Phase 1
    - Phase 0: all notes in flat /Notes directory

🚨 CONFLICT 7: Kanban Board Persistence
What Phase 0 Mandate Says:

invoke('get_kanban_board', { id: UUID }) -> JSON
invoke('move_task', { board_id, task_id, source_col, dest_col })

Interpretation:

Kanban boards are separate entities from notes
move_task updates the board file on disk

What User Memories Say:
(Silent on whether tasks are separate files or embedded in notes)
What Real Obsidian Does:

Obsidian Kanban plugin stores board state in a .md file (like you're doing)
But it also supports inline board syntax within note content

The Problem:
Are tasks in VIBO:

Embedded in a note as checklist items?
Stored in separate /Tasks/board-uuid.md files?
Stored in both places (redundant)?

Mandate Gaps:

No clarity on task lifecycle
No clarity on how tasks relate to notes
No clarity on whether a task can exist without a board

INTERROGATION:
[ ] Model A: Tasks are separate objects stored in /Tasks/board-id.md
    - Implementation: move_task rewrites the board file
    - Benefit: Clean separation, Obsidian compatible
    - Risk: If user deletes the board file, all tasks are lost

[ ] Model B: Tasks are embedded in notes as checklists
    - Implementation: move_task updates the parent note file
    - Benefit: Tasks stay with notes
    - Risk: Complex to parse; breaks Kanban plugin compatibility

[ ] Model C: Dual representation (both files and SQLite)
    - Implementation: complex sync
    - Benefit: Queryable via SQLite
    - Risk: Sync errors, dual-write problem

SUMMARY: 7 UNRESOLVED CONFLICTS
ConflictMandateMemoriesStatus1. SQLite vs. FilesystemFilesystem primaryDual representation⛔ BLOCKING2. Encryption in Phase 0?SilentCore feature⛔ BLOCKING3. Kanban column countsAssumed auto-syncNot specified⚠️ NEEDS SPEC4. YAML schema4 fields8 fields⚠️ MISMATCH5. iOS file accessAssumed universalRequires setup⚠️ NEEDS PLAN6. Tags/Folders CRUDAssumed existSilent on CRUD⚠️ SCOPE UNCLEAR7. Task storage modelSeparate /Tasks filesNot specified⚠️ NEEDS SPEC

RECOMMENDATION
Do NOT code Phase 0 until these are resolved.
Create a revised Phase 0 mandate that:

Chooses SQLite vs. filesystem source of truth (one only)
Decides: Encryption Phase 0 or Phase 1
Specifies Kanban column count logic
Locks YAML schema (mandate + memories reconciled)
Specifies iOS file access strategy
Defines tags/folders scope (full, minimal, or deferred)
Defines task storage model (separate or embedded)

Once these are locked, engineering can code confidently.

NEXT STEPS

Share actual repo code (current implementation status)
Answer the 7 interrogations above
I will create a revised mandate that reflects decisions
Engineering signs off on revised mandate before coding begins

> **STATUS (2026-04-26 review):** ✅ Audit completed and acted on. Mandate vs. code reconciliation locked in by Phase 0 sign-off (2026-04-19) and 2026-04-24 Stronghold activation. Current state of truth: `Guidelines/source-of-truth/PHASE_0_COMPLETION.md`. Marked for archive.

# PHASE 0 MANDATE AUDIT
**Status:** ✅ Resolved — historical
**Purpose:** Compare stated mandate against actual codebase to identify gaps, conflicts, and risks

---

## SECTION A: INTERROGATING THE MANDATE ITSELF
*(These critiques apply regardless of repo state)*

### 1.1 The Vault Selection UX
**Mandate Claim:**
> *Use tauri-plugin-dialog to allow the user to select or create a base directory (e.g., Documents/ViBo).*

**Critical Questions:**
- [ ] On iOS, users cannot directly access filesystem dialogs the way Android can. `tauri-plugin-dialog` uses native APIs. Does it work on iOS, or does this require a fallback UX (e.g., "Create Vault" button that creates a Documents/ViBo folder automatically)?
- [ ] Once a Vault is selected, can the user later *change* it? Or is it locked forever? If changeable, what happens to app state if they switch vaults mid-session?
- [ ] Does the app have permission to write to the user's Documents folder? Or must it use `NSDocumentDirectory` (iOS) / `getExternalFilesDir()` (Android)? These are sandboxed, not the universal Documents folder.

**Current Repo State:** `_______________________` (Need: check src/commands/vault setup logic)

---

### 1.2 The Markdown File Format
**Mandate Claim:**
```markdown
---
id: 550e8400-e29b-41d4
title: Welcome to ViBo
created: 2026-04-15T18:30:00Z
tags: []
---
```

**Critical Questions:**
- [ ] Is the YAML frontmatter delimiter EXACTLY `---\n` (3 dashes + newline)? Or does the code also accept other delimiters (which would fail Obsidian compatibility)?
- [ ] The mandate shows `id`, `title`, `created`, `tags`. But the user memories mention `kind`, `column`, `position`, `is_encrypted`, `folder`, `timestamps`. Are there MORE fields? If so, which is the source of truth?
- [ ] What happens if a user manually edits the YAML in Obsidian (e.g., adds a custom field)? Does ViBo preserve it on next update?
- [ ] **CRITICAL:** The mandate does NOT mention the `!vibo-encrypted:v1:` prefix for encrypted notes. Is encryption in or out of Phase 0? The user memories say it's a core feature, but Phase 0 mandate is silent.

**Current Repo State:** `_______________________` (Need: check src/storage/note format)

---

### 1.3 The Kanban Board Format
**Mandate Claim:**
```markdown
---
kanban-plugin: basic
title: Project Board
---

## To Do 1
- [ ] Explore ViBo features

## In Progress 0
```

**Critical Questions:**
- [ ] The `kanban-plugin: basic` field—is this required for Obsidian Kanban plugin compatibility? Or is it just metadata ViBo uses internally?
- [ ] The `1`, `0` suffixes in column headers ("To Do 1", "In Progress 0")—are these task *counts* that must be auto-updated? If so, when? After every drag-drop? This is error-prone if the count drifts.
- [ ] **MAJOR ISSUE:** How does ViBo store which task belongs in which column if a user deletes a column header? The mandate doesn't explain the mapping.
- [ ] Can tasks span multiple Kanban boards? Or is each task bound to exactly one board?

**Current Repo State:** `_______________________` (Need: check src/commands/kanban parsing logic)

---

### 1.4 The "No Direct DOM Filesystem Access" Constraint
**Mandate Claim:**
> *A code review confirms that fs.readFile or fs.writeFile do not exist anywhere in the TSX/Bun frontend code.*

**Critical Questions:**
- [ ] Does the current codebase use any Node.js runtime filesystem APIs in the frontend (e.g., `require('fs')`)?
- [ ] Are there any webpack/Bun imports of `node:fs` modules that get bundled by mistake?
- [ ] Does Tauri's API (`__TAURI__.invoke`) actually exist and is it wired correctly, or is the frontend still making localhost HTTP calls that would break on mobile webview?

**Current Repo State:** `_______________________` (Need: grep -r "fs\." src/face/ and check tauri.conf.json)

---

## SECTION B: MANDATE COMPARISON CHECKLIST
*(Complete these by examining actual repo code)*

### 2.1 Rust Commands (Tauri IPC Layer)
**Expected Commands (from mandate):**
- `create_note(title: String) -> UUID`
- `read_note(id: UUID) -> { frontmatter, content }`
- `update_note(id: UUID, content: String) -> Result`
- `list_notes() -> Array<{ id, title, updated }>`
- `get_kanban_board(id: UUID) -> JSON`
- `move_task(board_id, task_id, source_col, dest_col) -> Result`

**Questions:**
- [ ] Do these commands exist in the current codebase?
- [ ] What commands EXIST that are NOT in the mandate? Are they necessary, or Phase 1 scope creep?
- [ ] Are there commands that ARE in the mandate but MISSING from the codebase?

**Current Repo State:** 
```
Commands Found: ___________________
Commands Missing: ___________________
Extra Commands (not in mandate): ___________________
```

---

### 2.2 Filesystem Layout
**Expected Structure:**
```
/ViBo (Vault root)
├── /Notes
│   ├── 550e8400-e29b-41d4.md
│   ├── 550e8400-e29b-41d5.md
└── /Tasks
    ├── project-board-uuid.md
```

**Questions:**
- [ ] Does the current code actually create this structure on vault init?
- [ ] Are there OTHER folders (/.vibo, /metadata, etc.)? If so, why?
- [ ] Are files named exactly `{UUID}.md`, or with slugified titles?

**Current Repo State:**
```
Vault structure matches mandate: [ ] Yes [ ] No [ ] Partially
Deviations: ___________________
```

---

### 2.3 YAML Frontmatter Parsing
**Expected Library:** `gray_matter` (or similar)

**Questions:**
- [ ] What crate is actually used for YAML parsing?
- [ ] Does it preserve unknown fields (for forward compatibility with Obsidian custom fields)?
- [ ] Does it correctly handle edge cases:
  - [ ] Empty YAML blocks?
  - [ ] YAML with special characters (e.g., `title: "I said \"hello\"")`)?
  - [ ] Tabs vs. spaces in YAML indentation?

**Current Repo State:**
```
YAML crate used: ___________________
Edge cases handled: [ ] Comprehensive [ ] Partial [ ] None
```

---

### 2.4 Encryption (Implicit in mandate or Phase 1?)
**User Memories State:**
> *Private notes encrypted with AES-256-GCM in Rust before disk write, prefixed `!vibo-encrypted:v1:`*

**But Mandate Says:**
> *Phase 0 scope: Notes and Kanban Tasks ONLY*

**Questions:**
- [ ] Is encryption implemented in Phase 0 or Phase 1?
- [ ] If Phase 0: Can notes be marked as private/encrypted via the TSX UI?
- [ ] If Phase 0: Does the `create_note` command accept an `is_encrypted` flag?
- [ ] If Phase 0: Where is the encryption key stored? In Stronghold? In a simple passphrase?

**Current Repo State:** `_______________________`

---

### 2.5 SQLite Metadata vs. Markdown Source of Truth
**User Memories State:**
> *SQLite (WAL mode) for metadata... markdown files for note bodies*

**Mandate Says:**
> *All filesystem logic MUST be in Rust. ... Rust must write a file formatted exactly like this*

**Critical Conflict:**
- [ ] If metadata is in SQLite AND files are in markdown, which is the source of truth?
- [ ] If a user manually edits a `.md` file in Obsidian and ViBo hasn't synced, does ViBo see stale data from SQLite?
- [ ] When `list_notes()` is called, does it:
  - [ ] Read filesystem `.md` files directly (slow on large vaults)?
  - [ ] Query SQLite metadata (fast, but can go stale)?
  - [ ] Do both and compare (expensive)?

**Current Repo State:** `_______________________`

---

## SECTION C: ARCHITECTURE CONFLICTS
*(These are red flags in the mandate + memories)*

### 3.1 The Dual-Write Problem
**Scenario:**
1. User creates note in ViBo → writes to disk + inserts into SQLite
2. User opens same note in Obsidian on another device → modifies `.md` file directly
3. User returns to ViBo → which version is authoritative?

**Questions:**
- [ ] Is there a file-watcher on the Vault directory that syncs changes back into SQLite?
- [ ] If yes, does it handle conflicts (e.g., if user edits both ViBo and Obsidian simultaneously)?
- [ ] If no, ViBo can become inconsistent with the actual filesystem.

**Current Repo State:** `_______________________` (Need: check for fs watcher, sync logic)

---

### 3.2 The Kanban Column Count Problem
**Scenario:**
```markdown
## To Do 1
- [ ] Task A

## In Progress 0
```

After moving Task A to In Progress, should this become:
```markdown
## To Do 0
## In Progress 1
```

**Questions:**
- [ ] Does the mandate require auto-updating the counts?
- [ ] If yes: When does the update happen? Immediately? Or on next file read?
- [ ] If no: The counts can drift, confusing users. Is this acceptable?

**Current Repo State:** `_______________________`

---

### 3.3 The Mobile Permissions Problem
**Scenario:**
iOS sandboxing means:
- App can write to `Documents/` (inside the app container)
- App CANNOT write to `Documents/` (the shared system folder)

**Questions:**
- [ ] Does the mandate account for this?
- [ ] Is the "Vault" path expected to be inside the app container or the system Documents folder?
- [ ] If system Documents: This breaks on iOS unless handled via file sharing (iTunes, Files app).

**Current Repo State:** `_______________________`

---

## SECTION D: ACCEPTANCE CRITERIA VALIDATION
*(The 4 criteria from mandate—are they achievable?)*

### 4.1 Creation
> *I can open the app, create a Vault, and tap "New Note".*

**Questions:**
- [ ] Does the TSX UI have a "New Note" button?
- [ ] Is the `create_note` command wired to it?
- [ ] Does the command actually create a `.md` file on disk?

**Current Repo State:** `[ ] Done  [ ] Partial  [ ] Missing`

---

### 4.2 Verification
> *I can close ViBo, open an external file browser on the phone, go to the Vault folder, open /Notes, and physically see the .md file there.*

**Questions:**
- [ ] Is the Vault path accessible via the iOS/Android file browser?
- [ ] Or is it locked in the app container (which requires iTunes/Files app to access)?

**Current Repo State:** `[ ] Done  [ ] Partial  [ ] Missing`

---

### 4.3 Cross-App Compatibility
> *I can open that exact .md file in the mobile Obsidian app, and it renders perfectly.*

**Questions:**
- [ ] Has this been tested with Obsidian iOS/Android?
- [ ] Does the YAML frontmatter parse correctly in Obsidian?
- [ ] Does the Kanban board render as a valid Obsidian board?

**Current Repo State:** `[ ] Done  [ ] Partial  [ ] Missing`

---

### 4.4 No Direct DOM Filesystem Access
> *A code review confirms that fs.readFile or fs.writeFile do not exist anywhere in the TSX/Bun frontend code.*

**Questions:**
- [ ] Have you run `grep -r "fs\." src/face/` to check?
- [ ] Have you checked webpack/Bun bundles for embedded Node modules?

**Current Repo State:** `[ ] Done  [ ] Partial  [ ] Missing`

---

## SECTION E: MISSING FROM MANDATE (But in User Memories)

### 5.1 Encryption
- User memories: *Private notes encrypted with AES-256-GCM in Rust*
- Mandate: Silent
- **Question:** Phase 0 or Phase 1?

### 5.2 Tags
- User memories: *tag` field in frontmatter*
- Mandate: Silent on tag CRUD
- **Question:** Can users create/edit tags in Phase 0?

### 5.3 Folders
- User memories: *`folder` field in frontmatter*
- Mandate: Silent
- **Question:** Can notes be organized into folders?

### 5.4 Vault Security / Passphrase
- User memories: *Stronghold vault locked until explicit unlock*
- Mandate: Silent
- **Question:** Must users authenticate to access the vault in Phase 0?

---

## NEXT STEPS

**Share the following with me:**

1. **Repo path/structure** (or zip of src/commands and src/face)
2. **Answers to Section B checklist** (which commands exist, which don't)
3. **Current Cargo.toml** (to see which crates are actually used)
4. **A screenshot of the current Phase 0 UI** (to see what's already built)

Once you provide these, I will:
- [ ] Identify 3–5 critical gaps between mandate and code
- [ ] Flag architectural conflicts (dual-write, SQLite vs. filesystem source of truth)
- [ ] Create a **revised Phase 0 mandate** that reflects reality
- [ ] Provide a **step-by-step integration plan** to close gaps

---

**Bottom line:** The mandate is *aspirational*. Your code is *actual*. Let's align them.

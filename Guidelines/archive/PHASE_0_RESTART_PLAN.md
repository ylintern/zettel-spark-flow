# Phase 0 Restart Execution Plan

**Status**: `[~]` IN PROGRESS  
**Last Updated**: 2026-04-14

---

## Objective

Full audit completed. Phase 0 restart plan approved (Option B: rebuild, test, then delete old implementations).

Key mandates:
- Storage folder: `~/ViboAI/`
- Obsidian format compliance (wiki-links, markdown, tasks as `.md`)
- Existing tasks: delete and rebuild with Obsidian format
- Autocomplete: disable (not a priority)
- Agent notes: DEFERRED (no agent functionality yet)

---

## Task Execution Board

| # | Task | Agent | Status | Dependencies |
|---|------|-------|--------|--------------|
| A1 | Create PHASE_0_RESTART_PLAN.md | Architect | ✅ DONE | - |
| A2 | Plugin Index - Create src-tauri/src/plugins/ structure | Agent 1 | ✅ DONE | A1 |
| A3 | Task Format Migration - Backend + Frontend | Agent 2 | ✅ DONE | A2 |
| A4 | Wiki-link Alias Support + Autocomplete Disable | Agent 3 | ✅ DONE | A3 |
| A5 | Onboarding - Storage folder selection | Agent 4 | ⏸️ DEFERRED | A2 |
| A6 | Frontmatter Standardization | Agent 5 | ✅ DONE | A3 |
| A7 | Run QA tests and verify persistence | Agent 6 | ⏸️ PENDING | A3, A5 |

---

## Category A: Plugin Index

### A2: Create src-tauri/src/plugins/ structure

**Purpose**: Document all active and planned Tauri plugins for the project.

**Structure**:
```
src-tauri/src/plugins/
├── mod.rs           # Plugin index - lists all active + planned
├── active/          # Currently in use
│   ├── log.rs       # tauri-plugin-log
│   ├── fs.rs        # tauri-plugin-fs
│   ├── sql.rs       # tauri-plugin-sql
│   ├── stronghold.rs # tauri-plugin-stronghold
│   ├── dialog.rs    # tauri-plugin-dialog
│   ├── os.rs        # tauri-plugin-os
│   ├── clipboard.rs # tauri-plugin-clipboard-manager
│   ├── autostart.rs # tauri-plugin-autostart (desktop)
│   ├── haptics.rs   # tauri-plugin-haptics (mobile)
│   └── shortcuts.rs # tauri-plugin-global-shortcut (desktop)
└── planned/         # Future plugins
    ├── biometric.rs # tauri-plugin-biometric
    ├── leap_ai.rs   # tauri-plugin-leap-ai
    └── velesdb.rs   # tauri-plugin-velesdb
```

**Active Plugins (from lib.rs)**:
- `tauri-plugin-log` - Logging (debug only)
- `tauri-plugin-fs` - File system access
- `tauri-plugin-sql` - SQLite database
- `tauri-plugin-stronghold` - Secure vault
- `tauri-plugin-dialog` - Native dialogs
- `tauri-plugin-os` - OS detection
- `tauri-plugin-clipboard-manager` - Clipboard
- `tauri-plugin-autostart` - Desktop auto-start
- `tauri-plugin-global-shortcut` - Desktop shortcuts
- `tauri-plugin-haptics` - Mobile haptics

**Instructions for Agent 1**:
1. Create the folder structure
2. For each active plugin, create a `.md` file with:
   - Plugin name and version
   - Purpose in this app
   - Configuration used
   - Known issues or notes
3. For planned plugins, add to `planned/` with status: "RESEARCH", "PENDING", "READY"
4. Report back with file list created

---

## Category B: Obsidian Format Compliance

### B1: Task Format Migration

**Current**: YAML frontmatter with custom `Status:`, `Priority:` fields  
**Target**: Obsidian-native `- [ ]` checkbox format in markdown body

**Backend changes** (`src-tauri/src/vault/mod.rs`):
1. Remove YAML Status/Priority fields from frontmatter
2. Add `type: "task"` to frontmatter
3. Add optional Obsidian properties: `due`, `scheduled`, `priority`
4. Task body should be: `- [ ]` (unchecked) or `- [x]` (checked)

**Frontend changes** (`src/lib/store.tsx`, `src/components/NoteEditor.tsx`):
1. Update kanbanTemplate to: `## Task\n\n- [ ] \n\n### Description\n\n`
2. Add checkbox parsing for preview mode
3. Update KanbanView to detect checkbox state from `.md` content

**Instructions for Agent 2**:
1. Read `src-tauri/src/vault/mod.rs` - find `render_markdown()` function
2. Read `src/lib/store.tsx:218` - find kanbanTemplate
3. Create Obsidian format implementation
4. Test: create task → verify checkbox format in `.md` file
5. Report: what changed, what files modified

---

### B2: Wiki-link Alias Support

**Current**: `[[Title]]` format only  
**Target**: `[[target|display]]` Obsidian-compatible format

**Changes** (`src/lib/wiki-links.ts`, `src/components/NoteEditor.tsx`):
1. Update `extractWikiLinks()` to parse `[[target|display]]`
2. Return `{ target, display }` object instead of just string
3. Update rendering to show display text, link to target
4. Update autocomplete to show `[display] → target` format

**Instructions for Agent 3 (Part 1)**:
1. Read `src/lib/wiki-links.ts` - find `extractWikiLinks()` function
2. Read `src/components/NoteEditor.tsx` - find wiki-link rendering
3. Implement alias parsing
4. Report: regex pattern used, rendering changes

---

### B3: Autocomplete Disable

**Current**: Triggers on `[[` typing  
**Target**: Disabled (not a priority)

**Changes** (`src/components/NoteEditor.tsx`):
1. Set default `showLinkSuggest` to `false`
2. Comment out trigger logic in `handleContentChange()`

**Instructions for Agent 3 (Part 2)**:
1. Read `src/components/NoteEditor.tsx:27-41`
2. Comment out autocomplete trigger (lines 33-40)
3. Report: what was commented out

---

### B4: Frontmatter Standardization

**Current**: `created_at: "2026-04-07T00:00:00.000Z"`  
**Target**: `created: 2026-04-07T00:00:00Z` (ISO 8601, Obsidian style)

**Changes** (`src-tauri/src/vault/mod.rs`):
1. Change `created_at` → `created`
2. Change `updated_at` → `modified`
3. Add `type: "note" | "task"` to frontmatter

**Instructions for Agent 5**:
1. Read `src-tauri/src/vault/mod.rs` - find YAML frontmatter generation
2. Update field names to match Obsidian convention
3. Report: fields changed, format used

---

## Category C: Onboarding + Storage

### C1: Storage Folder Selection

**Current**: Uses `app_local_data_dir()` (auto-generated by Tauri)  
**Target**: User selects `~/ViboAI/` or custom path during onboarding

**Backend changes** (`src-tauri/src/commands/workspace.rs`, `src-tauri/src/lib.rs`):
1. Add `select_vault_folder()` command using tauri-plugin-dialog
2. Update bootstrap to use user-selected path
3. Store path in Stronghold (not localStorage)

**Frontend changes** (`src/components/OnboardingWizard.tsx`):
1. Add new step: "Choose where to store your notes"
2. Show folder picker with default `~/ViboAI/`
3. Allow change before finalizing

**Instructions for Agent 4**:
1. Read `src-tauri/src/lib.rs` - find bootstrap functions
2. Read `src-tauri/src/commands/workspace.rs` - find existing commands
3. Read `src/components/OnboardingWizard.tsx` - find step structure
4. Plan: add folder selection step + backend command
5. Report: planned implementation approach

---

## Category D: QA Completion

### D1: Run Pending Tests

Per `PHASE_0_EXECUTION_BOARD.md`:

| Test | Command | Expected |
|------|---------|----------|
| Reset routing → onboarding | Manual | Settings → Reset → confirm → expect onboarding |
| Note persistence after relaunch | `python3 scripts/qa_p0_tail.py --dev` | Note exists after restart |
| Task persistence after relaunch | Same script | Task + checkboxes exist |

**Instructions for Agent 6**:
1. Run `python3 scripts/qa_p0_tail.py --dev`
2. Verify note persistence
3. Verify task persistence with checkbox format
4. Report: test results, any failures

---

## Cross-File Dependencies

Before each agent executes, verify:

| Agent | Must Check |
|-------|------------|
| Agent 1 | Cargo.toml - verify plugin versions |
| Agent 2 | vault/mod.rs + store.tsx - ensure format match |
| Agent 3 | wiki-links.ts + NoteEditor.tsx + KnowledgeGraph.tsx |
| Agent 4 | lib.rs + workspace.rs + OnboardingWizard.tsx + tauri.conf.json |
| Agent 5 | vault/mod.rs + db/mod.rs - ensure field names match |
| Agent 6 | All above - ensure nothing broken |

---

## Success Criteria

- [ ] Plugin index created with all active + planned plugins
- [ ] Tasks use Obsidian checkbox format (`- [ ]` / `- [x]`)
- [ ] Wiki-links support aliases (`[[target|display]]`)
- [ ] Autocomplete disabled
- [ ] Frontmatter uses Obsidian field names
- [ ] Onboarding has storage folder selection step
- [ ] QA tests pass for persistence

---

## Notes

- Agent notes in localStorage: DEFERRED (no agent functionality yet)
- Storage folder selection: DEFERRED (complex, needs careful implementation)
- Existing tasks: will be deleted when migration happens
- Storage folder default: `~/ViboAI/`

---

## Completed Tasks Summary

### A2: Plugin Index
- Created `src-tauri/src/plugins/` with:
  - `README.md` - Main index
  - `ACTIVE.md` - List of 10 active plugins
  - `PLANNED.md` - List of 2 planned plugins
  - `active/` folder with 10 plugin documentation files
  - `planned/` folder with leap_ai.md and velesdb.md

### A3: Task Format Migration
**Backend** (`src-tauri/src/vault/mod.rs`):
- Changed frontmatter from `kind:` to `type:` (Obsidian-native)
- Added checkbox handling: prepends `- [ ]` if not present in task body

**Frontend** (`src/lib/store.tsx`):
- Updated kanbanTemplate from custom YAML to: `## Task\n\n- [ ] \n`

### A4: Wiki-link Alias Support + Autocomplete Disable
**Frontend** (`src/lib/wiki-links.ts`):
- Updated regex: `/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g` to capture aliases
- Added `WikiLink` interface: `{ target: string, display: string | null }`
- Updated `extractWikiLinks()` to return `WikiLink[]`
- Updated `getBacklinks()` and `buildGraph()` to use `target` field

**Frontend** (`src/components/NoteEditor.tsx`):
- Disabled autocomplete by commenting out trigger logic
- Updated `renderContent()` to parse alias format
- Updated click handler to navigate using `target`

**Verified**: KnowledgeGraph works without changes (uses `target` for edges)

### A6: Frontmatter Standardization
**Backend** (`src-tauri/src/vault/mod.rs`):
- Changed `created_at` → `created`
- Changed `updated_at` → `modified`

### Fixes Applied During Implementation
- Fixed capabilities: Removed invalid leap-ai permissions, added dialog:default
- Fixed Rust: Added Serialize derive to ProviderMessage struct
- Fixed duplicate files: Removed duplicate models folder in providers/
# Phase 0 Smoke Tests & QA Scripts

**Status**: `[~]` IN PROGRESS  
**Last Updated**: 2026-04-14

---

## Existing Scripts

| Script | Purpose | Location |
|--------|---------|----------|
| `qa_lifecycle.py` | Core lifecycle tests (8 checks) | `scripts/qa_lifecycle.py` |
| `qa_p0_tail.py` | Phase 0 tail verification | `scripts/qa_p0_tail.py` |

---

## New Smoke Tests to Create

### Category A: Onboarding & Vault

| Test ID | Name | Description | Automated | Manual |
|---------|------|--------------|-----------|--------|
| A1 | First Launch | App launches without crash | ❌ | ✅ |
| A2 | Onboarding Flow | Can complete all onboarding steps | ❌ | ✅ |
| A3 | Vault Setup | Can set passphrase during onboarding | ❌ | ✅ |
| A4 | Vault Unlock | Can unlock with correct passphrase | ❌ | ✅ |
| A5 | Wrong Passphrase | Wrong passphrase shows error | ❌ | ✅ |
| A6 | Factory Reset | Settings → Reset clears all data | ❌ | ✅ |
| A7 | Reset → Onboarding | After reset, shows onboarding again | ❌ | ✅ |

### Category B: Notes (Obsidian Format)

| Test ID | Name | Description | Automated | Manual |
|---------|------|--------------|-----------|--------|
| B1 | Create Note | Can create new note | ❌ | ✅ |
| B2 | Edit Note | Can edit note title and content | ❌ | ✅ |
| B3 | Delete Note | Can delete note | ❌ | ✅ |
| B4 | Note Persistence | Note survives app restart | ❌ | ✅ |
| B5 | Wiki-link Creation | `[[Note Title]]` syntax works | ❌ | ✅ |
| B6 | Wiki-link Click | Clicking wiki-link navigates | ❌ | ✅ |
| B7 | Wiki-link Alias | `[[target\|display]]` works | ❌ | ✅ |
| B8 | Tags | Can add/remove tags | ❌ | ✅ |
| B9 | Folders | Can assign note to folder | ❌ | ✅ |
| B10 | Frontmatter Format | `.md` has Obsidian `type:` field | ✅ | ❌ |
| B11 | Markdown File Exists | Note stored in `database/notes/{id}.md` | ✅ | ❌ |

### Category C: Tasks (Obsidian Format)

| Test ID | Name | Description | Automated | Manual |
|---------|------|--------------|-----------|--------|
| C1 | Create Task | Can create new task in Kanban | ❌ | ✅ |
| C2 | Move Task | Can drag task between columns | ❌ | ✅ |
| C3 | Task Checkbox | Task shows `- [ ]` checkbox | ❌ | ✅ |
| C4 | Toggle Checkbox | Can toggle checkbox state | ❌ | ✅ |
| C5 | Task Persistence | Task survives app restart | ❌ | ✅ |
| C6 | Task File Format | `.md` uses `- [ ]` / `- [x]` format | ✅ | ❌ |
| C7 | Task Type Field | Frontmatter has `type: task` | ✅ | ❌ |

### Category D: Security

| Test ID | Name | Description | Automated | Manual |
|---------|------|--------------|-----------|--------|
| D1 | Encrypt Note | Can mark note as encrypted | ❌ | ✅ |
| D2 | Decrypt Note | Can unmark encryption | ❌ | ✅ |
| D3 | Encrypted Content | Encrypted note shows cipher in `.md` | ✅ | ❌ |
| D4 | Biometrics Available | Shows biometric option on mobile | ❌ | ✅ |

### Category E: Views & Navigation

| Test ID | Name | Description | Automated | Manual |
|---------|------|--------------|-----------|--------|
| E1 | Dashboard | Dashboard view loads | ❌ | ✅ |
| E2 | Notebook | Notebook view shows notes list | ❌ | ✅ |
| E3 | Kanban | Kanban shows columns and cards | ❌ | ✅ |
| E4 | Graph | Knowledge graph renders | ❌ | ✅ |
| E5 | Settings | Settings view accessible | ❌ | ✅ |

### Category F: Build & Performance

| Test ID | Name | Description | Automated | Manual |
|---------|------|--------------|-----------|--------|
| F1 | Dev Build | `bun run tauri:dev` starts | ✅ | ❌ |
| F2 | Prod Build | `bun run tauri:build` succeeds | ✅ | ❌ |
| F3 | Frontend Build | `bun run build` succeeds | ✅ | ❌ |
| F4 | Rust Check | `cargo check` passes | ✅ | ❌ |
| F5 | Cold Start Time | App starts in < 5 seconds | ❌ | ✅ |

---

## Automated Test Scripts to Create

### Script 1: `scripts/smoke_tests.py`

**Purpose**: Run all automated tests (AUTO column = ✅)

**Tests to automate**:
- F1, F2, F3, F4: Build tests
- B10, B11, B6, B7: File format checks
- C6, C7: Task format checks
- D3: Encryption check

```python
#!/usr/bin/env python3
"""
Phase 0 Smoke Tests - Automated
Runs build checks, file format validation, etc.
"""

import subprocess
import sys
from pathlib import Path

def run_build_test():
    """Test that frontend builds"""
    print("\n[BUILD] Testing frontend build...")
    result = subprocess.run(["bun", "run", "build"], capture_output=True, text=True)
    return result.returncode == 0

def run_rust_check():
    """Test that Rust compiles"""
    print("\n[RUST] Running cargo check...")
    result = subprocess.run(
        ["cargo", "check"],
        cwd=Path("src-tauri"),
        capture_output=True,
        text=True
    )
    return result.returncode == 0

def check_frontmatter_format():
    """Verify Obsidian frontmatter format"""
    # Check that .md files have correct format
    pass

def check_task_format():
    """Verify task checkbox format"""
    pass

def main():
    results = []
    results.append(("Build", run_build_test()))
    results.append(("Rust Check", run_rust_check()))
    
    passed = sum(1 for _, r in results if r)
    print(f"\n{passed}/{len(results)} automated tests passed")
    return 0 if passed == len(results) else 1

if __name__ == "__main__":
    sys.exit(main())
```

---

### Script 2: `scripts/test_wiki_links.py`

**Purpose**: Test wiki-link functionality

**Tests**:
- Create note with `[[Link]]`
- Create note with `[[Target|Display]]` alias
- Verify links render in preview
- Verify click navigation works

---

### Script 3: `scripts/test_task_format.py`

**Purpose**: Validate task format compliance

**Checks**:
- Task `.md` files in `database/tasks/` have `- [ ]` or `- [x]`
- Frontmatter has `type: task`
- No custom YAML fields like `Status:` or `Priority:`

---

### Script 4: `scripts/test_obsidian_compat.py`

**Purpose**: Verify Obsidian compatibility

**Checks**:
- Frontmatter uses `created` / `modified` (not `created_at` / `updated_at`)
- Frontmatter uses `type:` (not `kind:`)
- Tags in array format
- Folders as string or null

---

## Manual Test Checklist

### For Auditor - Run These Manually

```
□ A1: First Launch - App opens without error
□ A2: Onboarding - Complete all steps
□ A3: Vault Setup - Set passphrase
□ A4: Vault Unlock - Enter correct passphrase
□ A5: Wrong Passphrase - Enter wrong, see error
□ A6: Factory Reset - Settings → Reset
□ A7: Reset Routing - After reset, go to onboarding

□ B1: Create Note - Click add note
□ B2: Edit Note - Type content, save
□ B3: Delete Note - Click delete
□ B4: Note Persistence - Close app, reopen, note exists
□ B5: Wiki-link - Type [[Test]], see it become link
□ B6: Wiki-link Click - Click link, navigate
□ B7: Wiki-link Alias - Type [[Target|Display]], see "Display"
□ B8: Tags - Add tag #test, see badge
□ B9: Folders - Create folder, move note

□ C1: Create Task - Add task in Kanban
□ C2: Move Task - Drag to different column
□ C3: Task Checkbox - See - [ ] in content
□ C4: Toggle Checkbox - Click checkbox, see - [x]
□ C5: Task Persistence - Reload app, task exists

□ D1: Encrypt Note - Toggle lock icon
□ D2: Decrypt Note - Toggle lock icon again
□ D4: Biometrics - (Mobile only) See biometric option

□ E1-5: All views load correctly
```

---

## How to Run

### Run Automated Tests
```bash
# Build tests
python3 scripts/smoke_tests.py

# File format tests
python3 scripts/qa_p0_tail.py --dev

# Wiki-link tests
python3 scripts/test_wiki_links.py

# Task format tests
python3 scripts/test_task_format.py
```

### Run Manual Tests
1. Open app: `bun run tauri:dev`
2. Follow manual checklist above
3. Note any failures in audit log

---

## Current Test Results

| Category | Status |
|----------|--------|
| Build (F1-F4) | ❓ Not run |
| File Format (B10-B11, C6-C7) | ❓ Not run |
| Manual Tests | ❓ Not run |

---

## Next Steps for Auditor

1. Run `python3 scripts/smoke_tests.py` - verify build works
2. Run `python3 scripts/qa_p0_tail.py --dev` - verify persistence
3. Run manual tests in checklist
4. Report results back to update this doc
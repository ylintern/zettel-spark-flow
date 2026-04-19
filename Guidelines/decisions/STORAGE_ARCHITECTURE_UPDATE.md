# Storage Architecture: Update to Phase 0 Completion

**Updated:** 2026-04-19  
**Previous:** Initial FS-first design (Phase 0 kickoff)  
**Status:** Design validated, one bug fixed, migration planned.

---

## What Was Built

### FS-First Reconciliation (Working ✓)

**Pattern:** .md files in `myspace/` are authoritative. SQL mirrors user data. On startup, reconcile FS → SQL.

**Files:**
- `src-tauri/src/vault/reconcile.rs` — walks .md tree, syncs SQL
- `src-tauri/src/vault/mod.rs:29–54` — reserved folder creation

**Result:** Users can edit .md outside the app; next sync brings SQL up to date. Obsidian-compatible.

**Verified:** ✓ Create note in app → .md appears in Finder; edit in Finder → restart app → SQL updated.

---

### Atomicity Issue & Fix

**Original Design Risk:** Save to .md, then SQL. If process crashes between, data diverges.

**Fix Applied (commit debcfcf, kanban race):** Use notesRef eager-compute pattern.
```ts
const notesRef = useRef<Note[]>(initialNotes);
useEffect(() => { notesRef.current = notes; }, [notes]);

const moveNote = (id, newColumn) => {
  const updated = notesRef.current.map(n => n.id === id ? {...n, column: newColumn} : n);
  setNotes(updated);
  persistNote(...); // Always called after state update
};
```

**Result:** Both .md and SQL writes happen atomically (within same turn). No half-baked state.

**Still True:** No file-level locking (two processes could corrupt). Single-instance app assumption holds (tauri:dev + build processes don't run concurrently). Production releases won't have this issue.

---

## localStorage Key Migration — Lesson Learned

### Problem: WebKit Origin Drift

Bundle ID changed (zettel-spark-flow → com.viboai.app). 

**What Happened:**
- WKWebView localStorage scoped by bundle identifier
- Old WebKit dir: `~/Library/WebKit/com.vibo.zettel-spark-flow/`
- New WebKit dir: should be `~/Library/WebKit/com.viboai.app.dev/` (but browser might use generic "app")
- Fresh wipes of Application Support didn't touch old WebKit origins
- Result: old localStorage (onboarding flags, agent config) leaked across renames

**Impact:** Fresh install detection failed because old `zettel-onboarding-done=true` persisted.

### Solution Applied (Phase 0)

Renamed all storage keys from `zettel-*` to `vibo-*`:
- `zettel-onboarding-done` → `vibo-onboarding-done`
- `zettel-ai-config` → `vibo-ai-config`
- `zettel-agent-notes` → `vibo-agent-notes`
- `zettel-tor-enabled` → `vibo-tor-enabled`

**Files Updated:**
- `src/components/OnboardingWizard.tsx` (3 keys)
- `src/components/SettingsView.tsx` (1 key)
- `src/lib/crypto.ts` (1 key)
- `src/pages/Index.tsx` (1 key)
- `src/components/settings/SafeVaultResetSection.tsx` (purge list)

**Also Deleted:**
- Dead migration code in `src-tauri/src/lib.rs:54–70` (referenced `com.vibo.zettel-spark-flow` path)

**Defensive Cleanup:**
- SafeVaultResetSection now purges both `zettel-*` and `vibo-*` keys so future resets clean everything

### Prevention: Phase 0.7+ Plan

**Move onboarding state from localStorage to file.**

**Rationale:**
- Files in Application Support are keyed by bundle ID (same for all origins)
- localStorage is keyed by WebKit origin (fragile across bundle renames)
- Backend-owned file = simpler state machine, no cross-origin confusion

**Implementation:**
- Create `viboai/onboarding.json` in vault dir
- Tauri commands: `read_onboarding()`, `write_onboarding()`, `reset_onboarding()`
- Frontend: async calls to Tauri instead of localStorage read/write
- Keep localStorage for non-onboarding prefs (theme, model, etc.)

---

## Current Storage Layout (Phase 0 End State)

### Application Support
```
~/Library/Application Support/com.viboai.app.dev/
  viboai/
    database/
      vibo.db (SQLite)
      vibo.db-shm, vibo.db-wal (WAL files)
      secure-vault.hold (Stronghold snapshot)
    myspace/
      Inbox/ (reserved, auto-created)
      Archive/ (reserved, auto-created)
      user-folder-1/ (user-created)
      note-1.md (FS-first)
      note-2.md
      subfolder/
        note-3.md
```

### WebKit (localStorage)
```
~/Library/WebKit/app/  or  ~/Library/WebKit/com.viboai.app.dev/
  WebsiteData/Default/.../LocalStorage/localstorage.sqlite3
    vibo-onboarding-done = "true"
    vibo-ai-config = {...}
    vibo-agent-notes = "[]"
    vibo-tor-enabled = "false"
    vibo_theme = "dark"
    ... (other prefs)
```

### Legacy (Inert, Can Be Cleaned)
```
~/Library/WebKit/com.vibo.zettel-spark-flow/  (OLD origin, unused now)
  WebsiteData/.../LocalStorage/localstorage.sqlite3
    zettel-onboarding-done, zettel-ai-config, etc. (DEAD DATA)
```

---

## Constraints & Assumptions

### Single-Instance App

- `bun run tauri:dev` + cargo build do NOT run concurrently
- Production release won't have dev build running alongside
- **Assumption:** At most one instance of app is reading/writing vault at a time

**If Violated:** Races on .md + SQL writes could cause corruption. Mitigation: add file-level locking (defer to Phase 1 if needed).

### Obsidian Compatibility

- Users can read/edit .md files outside app
- App's SQL mirrors FS; next sync reconciles
- `.md` format: YAML frontmatter + markdown body (see `src-tauri/src/vault/frontmatter.rs`)

**Limitation:** If user edits .md to break YAML, sync will error. No validation before write.

**Mitigation (Phase 1):** Validate YAML before sync; offer user choice (skip/fix).

---

## Testing Verification

### FS-SQL Sync
```bash
# 1. Create note in app
bun run tauri:dev
# Settings: Create note "Test"
# Verify: ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/Test.md exists

# 2. Edit .md outside app
vim ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/Test.md
# Add content, save

# 3. Restart app
# Verify: Note shows updated content; SQL was reconciled
```

### localStorage Isolation
```bash
# 1. Clear WebKit
rm -rf ~/Library/WebKit/com.vibo.zettel-spark-flow/
rm -rf ~/Library/WebKit/com.viboai.app.dev/

# 2. Restart app
bun run tauri:dev
# Verify: No old data persists; onboarding wizard appears (fresh state)
```

---

## Future Improvements (Phase 1+)

- [ ] File-level locking on vault dir (prevent concurrent writes)
- [ ] YAML validation before sync (handle user edits gracefully)
- [ ] Atomic transaction markers (.md + SQL in single TX)
- [ ] Backup snapshots before destructive operations (delete all notes)
- [ ] Cloud sync / conflict resolution (if multi-device)

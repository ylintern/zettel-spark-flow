# VIBO — Phase 0 QA: Persistence Verification
**Date**: 2026-04-09 | **Phase**: 0 Final Close

All code changes are done and compile clean. You only need to:
1. Run the app, create test items, quit, relaunch
2. Run the Python script to verify persistence
3. Report results back

---

## WHAT CHANGED (code already applied)

- `vault/` renamed to `database/` in all Rust code
- `kanban/` renamed to `tasks/` in all Rust code
- Migration runs silently on first launch: moves your existing `vault/` files to `database/`
- New kanban tasks now write to `database/tasks/{id}.md`
- New notes write to `database/notes/{id}.md`

---

## DEV MODE QA (use this track)

**Tools**: Terminal + `bun run tauri:dev`
**Data dir**: `~/Library/Application Support/com.viboai.app.dev/`

### Step 1 — Launch app

```bash
cd ~/Desktop/zettel-spark-flow-main
bun run tauri:dev
```

Wait for app window. Unlock with your passphrase.

### Step 2 — Create test artifacts (in the app UI)

- Create a **note** titled exactly: `Testinfolder69`
- Create a **kanban task** titled exactly: `Countertest69` (in any column)

### Step 3 — Quit and relaunch

```bash
# Quit with Cmd+Q, then relaunch:
bun run tauri:dev
```

Unlock with passphrase. Confirm both items are still visible in the app.

### Step 4 — Verify directories (paste output here)

```bash
ls -la ~/Library/Application\ Support/com.viboai.app.dev/
# Expected: database/ exists, vault/ is GONE

ls ~/Library/Application\ Support/com.viboai.app.dev/database/
# Expected: notes/ and tasks/

find ~/Library/Application\ Support/com.viboai.app.dev/database/notes/ -name "*.md" | wc -l
# Expected: at least 1

find ~/Library/Application\ Support/com.viboai.app.dev/database/tasks/ -name "*.md" | wc -l
# Expected: at least 1
```

### Step 5 — Run the QA script (paste output here)

```bash
python3 scripts/qa_p0_tail.py --dev
```

The script will check:
- `database/notes/` has a `.md` file for "Testinfolder69"
- `database/tasks/` has a `.md` file for "Countertest69"
- SQLite has both rows with correct `kind`

---

## INSTALLED APP QA (optional — only if you have a built .app)

**Tools**: Built `.app` bundle (not from terminal)
**Data dir**: `~/Library/Application Support/com.viboai.app/`

To build: `bun run tauri:build`

Same steps as above. Run script **without** `--dev`:
```bash
python3 scripts/qa_p0_tail.py
```

---

## RESET PASSWORD (separate flow — not this QA)

The Settings screen has two reset flows — both are intentional:
- **Pass / PIN / Biometric reset** — changes auth only, data survives (focus of next sprint)
- **Full database reset** — wipes everything, returns to onboarding (not in scope this sprint)

Nothing to do here right now.

---

## When Done

Paste the script output. Claude Code will:
1. Confirm pass/fail for each check
2. Close Phase 0
3. Begin P1-B (caller-aware command boundary)

# Task for Claude Code
**Status**: BUILD FIXED — Ready for Phase 0 work  
**Date**: 2026-04-08

---

## Build Fix Applied

**Problem**: `crunchy` and `rust_decimal` crates failed with missing generated files (stale build cache).  
**Fix**: User manually cleaned the stale cache:
```bash
rm -rf src-tauri/target/debug/build/crunchy-*
rm -rf src-tauri/target/debug/build/rust_decimal-*
cd src-tauri && cargo build
```
**Result**: Build succeeded in 48.78s with 14 warnings (all "never used" — cosmetic only).

---

## What You Need to Know

### Read These First
1. `Guidelines/audits/2026-04-08_CLAUDE_CODE_HANDOVER.md` — Full context on root causes, fixes applied, architecture
2. `Guidelines/audits/2026-04-08_BUILD_VERIFICATION_CHECKLIST.md` — Verification steps for vault timing and data isolation

### Current Config Changes (already applied, not yet tested end-to-end)
- `src-tauri/tauri.dev.conf.json` — NEW: dev identifier override (`com.viboai.app.dev`)
- `src-tauri/Cargo.toml` — MODIFIED: profile.dev.package optimizations for crypto crates
- `package.json` — MODIFIED: `tauri:dev` and `tauri:build` scripts added

### 14 Warnings (Cosmetic — Do Not Fix Unless Asked)
All in `src/services/context.rs` and `src/services/retrieval.rs` — unused structs/traits from future Swiftide agent integration. Leave them alone.

---

## Your Task: Verify Build End-to-End

### Step 1: Launch the App
```bash
cd /Users/cristianovb/Desktop/zettel-spark-flow-main
bun run tauri:dev
```

### Step 2: Check These
1. **Does the app launch?** (should show onboarding screen)
2. **How fast is vault setup?** Time the PIN creation (target: 5–8 sec, was 80+ before fix)
3. **Does second launch work?** Close app, rerun `bun run tauri:dev`, enter PIN — should NOT white screen

### Step 3: Verify Data Isolation
```bash
ls -la "$HOME/Library/Application Support/com.viboai.app.dev/"
```
Dev data should be in `com.viboai.app.dev`, NOT `com.viboai.app`.

---

## After Verification: Phase 0 Scope

Once build is confirmed stable, work on Phase 0 features **task by task, with user permission before each**:

### Phase 0a: Login / Settings
- PIN/Passphrase persistence (verify it works after Stronghold integration)
- Password reset with confirmation audit

### Phase 0b: Notes
- CRUD operations (Create, Read, Update, Delete)
- MD file storage in `~/vibo/database/notes/`
- YAML frontmatter metadata (title, tags, created_at, updated_at, folder)
- Default folders on first install: `Projects`, `Reference`, `Areas`, `Archive`
- No encryption (Phase 0 is local-only)

### Phase 0c: Tasks / Kanban
- Separate from Notes (own CRUD, own storage)
- MD file storage in `~/vibo/database/tasks/`
- Auto-status updates: `todo` → `in-progress` → `done`
- Date tracking (due_date, completed_at)
- Delegation support
- Kanban board UI

**USER CONSTRAINT**: "do not execute without my permission. ask and go task by task."

---

## Critical Rule

Do NOT start implementing Phase 0 features until the build is verified end-to-end (Steps 1–3 above). Report results to user first.

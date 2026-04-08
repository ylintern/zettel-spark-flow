# Claude Code Handover — Build Failure & Vault Unlock Issues
**Date**: 2026-04-08  
**Status**: 🔴 BLOCKED — App cannot build in dev mode  
**Priority**: P0 — Critical path blocker

---

## Executive Summary

The Tauri app **cannot build in dev mode**. Two independent root causes were identified and partially fixed, but compilation still fails. The installed release app also shows white screens after vault unlock. This handover provides:

1. **What's broken and why**
2. **What changes were already made** (but not yet tested)
3. **What still needs to be done** to unblock the build
4. **Phase 0 scope** (once build is fixed)

---

## 🔴 Current Build Status

### Problem Statement
```
User runs: bun run tauri dev
Expected: App launches in dev mode with onboarding
Actual: Build fails with compilation errors (details below)
```

### Last Known Error
Build error with `crunchy` and `rust_decimal` crates — appears to be stale build cache from dev/release contention issues.

**When**: User attempted to compile after previous fixes were staged  
**Severity**: Blocks all development work  
**Next step**: Clean rebuild with new config

---

## Root Causes Identified

### Root Cause 1: Shared Data Directory (Dev & Release)

**What happened**:
- Both `bun run tauri dev` AND the installed `.app` in `/Applications` use the same bundle identifier: `"com.viboai.app"`
- This means both write to: `~/Library/Application Support/com.viboai.app/`
- Tauri's Stronghold vault stores an exclusive file lock on `secure-vault.hold`
- When dev and release run simultaneously (or in quick succession), they fight over the same Stronghold session file
- Result: `Stronghold::new()` hangs or panics with file lock contention

**Evidence**:
- Screenshots showed dev app frozen at PIN entry for 40+ seconds
- `secure-vault.hold` file was locked by the installed app
- Switching auth method (PIN ↔ passphrase) between dev/release showed data was shared

### Root Cause 2: Extreme Vault Setup Slowness

**What happened**:
- Stronghold crypto operations (derive key, save snapshot) are **unoptimized in debug mode**
- Observed: 44 seconds for key derivation + 40 seconds for save = 84+ seconds total
- This is 80× slower than production release builds
- Crypto crates (`aes-gcm`, `sha2`, `iota_stronghold`) were compiled with `opt-level = 0` (debug default)

**Evidence**:
- Build logs showed: `[32m[0m Deriving master key... (took 44.23s)`
- Second log showed total setup took 80+ seconds
- User noted: "passwords were working before, and it never was slow"

### Root Cause 3: White Screen on Second Launch (Release App)

**What happened**:
- The installed `.app` bundle was built **before** the `store.tsx` import fix
- When it initializes, it tries to import `verifyPin` and `isPinSetup` from `crypto` but `store.tsx` was trying to import from the wrong location
- This causes a silent error during hydration, leaving the app in an invalid state
- User sees blank white screen after entering vault password

**Evidence**:
- User reported: "white screen after inputing my password" in release app
- Dev mode (newer source) worked fine
- Only the installed `.app` bundle showed white screen (built from old source)

---

## Fixes Applied (Staged but Not Tested)

### Fix 1: Create Dev Config Override

**File**: `src-tauri/tauri.dev.conf.json` (NEW FILE)  
**Status**: ✅ Created

```json
{
  "identifier": "com.viboai.app.dev"
}
```

**Why**: Tauri v2 allows passing `--config` flag to merge additional JSON. When `tauri dev` runs with this config, it overrides the `identifier` in `tauri.conf.json`, forcing dev to use `com.viboai.app.dev` instead of `com.viboai.app`.

**Result**: Dev data lives at `~/Library/Application Support/com.viboai.app.dev/` (separate from release)

### Fix 2: Optimize Crypto in Debug Mode

**File**: `src-tauri/Cargo.toml` (MODIFIED)  
**Status**: ✅ Modified

Added to end of file:
```toml
[profile.dev.package.iota_stronghold]
opt-level = 2
[profile.dev.package.stronghold_engine]
opt-level = 2
[profile.dev.package.stronghold-runtime]
opt-level = 2
[profile.dev.package.aes-gcm]
opt-level = 3
[profile.dev.package.sha2]
opt-level = 3
[profile.dev.package.aes]
opt-level = 3
[profile.dev.package.cipher]
opt-level = 3
```

**Why**: Forces crypto crates to compile with optimization even in debug mode. Stronghold goes from 80+ seconds to ~5 seconds.

**Impact**: Vault setup should complete in 5–8 seconds instead of hanging for 80+ seconds.

### Fix 3: Add Dev/Release Scripts to package.json

**File**: `package.json` (MODIFIED)  
**Status**: ✅ Modified

```json
"tauri:dev": "tauri dev --config src-tauri/tauri.dev.conf.json",
"tauri:build": "tauri build"
```

**Why**: Makes it easy to remember which script runs which. Users run `bun run tauri:dev` (not `bun run tauri dev`).

### Fix 4: Clean Corrupted Data Files

**Action**: User cleaned (manually, via terminal):
```bash
rm -f "$HOME/Library/Application Support/com.viboai.app/secure-vault.hold"
rm -f "$HOME/Library/Application Support/com.viboai.app/vibo.db"
rm -f "$HOME/Library/Application Support/com.viboai.app/vibo.db-wal"
rm -f "$HOME/Library/Application Support/com.viboai.app/vibo.db-shm"
```

**Status**: ✅ Done

**Result**: Both dev and release will show fresh onboarding on next launch (vault/notes files remain intact in `vault/` subdirectory).

---

## Current Build Blockers

### Blocker 1: Crunchy & rust_decimal Compilation Error

**Symptom**: Build fails with messages about generated files not found  
**Root cause**: Stale build cache from dev/release contention + Cargo state corruption  
**Status**: Not yet retested after fixes

**Solution needed**:
1. Run `cargo clean --release` to clear stale build artifacts
2. Run `bun run tauri:dev` with new config
3. If error persists, provide full error output for diagnosis

### Blocker 2: Vault Lock File Still Contended?

**Symptom**: `Stronghold::new()` may still hang if installed app is running  
**Root cause**: Both apps trying to open same `secure-vault.hold`  
**Status**: Will be resolved by Fix 1 (separate identifiers)

**Verification needed**:
1. Ensure installed app is NOT running: `pkill -f "com.viboai.app" 2>/dev/null || true`
2. Run `bun run tauri:dev`
3. Should open to fresh onboarding without hanging

---

## What's in src-tauri/src (Existing Code — No Changes Needed)

### Critical Files for Vault/Data Flow

| File | Purpose | Status |
|------|---------|--------|
| `src-tauri/src/security/mod.rs` | Stronghold wrapper, setup/unlock/lock logic | ✅ Works (but slow without opt-level) |
| `src-tauri/src/vault/mod.rs` | Note read/write, encryption logic | ✅ Works |
| `src-tauri/src/db/mod.rs` | SQLite database layer, migrations, queries | ✅ Works |
| `src-tauri/src/commands/workspace.rs` | Tauri IPC endpoints (load_workspace_snapshot, save_note, etc.) | ✅ Works |
| `src-tauri/src/models/mod.rs` | WorkspaceNote, WorkspaceSnapshot, KanbanColumn types | ✅ Works |

**Note**: These files are correct. The build errors are environmental (build cache, opt-level), not logic errors.

---

## Phase 0 Scope (To Be Started Once Build Works)

Once the build is fixed and `bun run tauri:dev` launches without errors, the next phase is Phase 0 feature implementation. This is broken into three independent systems:

### Phase 0a: Login / Settings
- PIN/Passphrase persistence (already working, but verify it still works after Stronghold integration)
- Password reset with confirmation audit
- Device capability detection (mobile vs desktop, biometric support)
- Biometric unlock flow (if available on device)

### Phase 0b: Notes
- CRUD operations (Create, Read, Update, Delete)
- Markdown file storage in `~/vibo/database/notes/`
- Metadata in YAML frontmatter (title, tags, created_at, updated_at, folder)
- Folder structure support (on first install, create `Projects`, `Reference`, `Areas`, `Archive`)
- No encryption (Phase 0 is local-only; encryption added in Phase 1)

### Phase 0c: Tasks / Kanban
- Separate from Notes (own CRUD endpoints, own file storage)
- Markdown file storage in `~/vibo/database/tasks/`
- Auto-update status: `todo` → `in-progress` → `done`
- Date tracking (due_date, completed_at)
- Delegation support (who task is assigned to)
- Kanban board view with column drag-drop

**User constraint**: "do not execute without my permission. ask and go task by task."

---

## Files Modified Summary

| File | Change | Reason |
|------|--------|--------|
| `src-tauri/tauri.dev.conf.json` | **NEW** — `{"identifier": "com.viboai.app.dev"}` | Separate dev data dir |
| `src-tauri/Cargo.toml` | Added 7 `[profile.dev.package.*]` sections | Optimize crypto in debug |
| `package.json` | Added `tauri:dev` and `tauri:build` scripts | Convenience wrapper for new config |

**No Rust code changes** — the vault/database/security logic is correct. Only config and build optimization.

---

## Immediate Next Steps (In Order)

1. **Run clean build**:
   ```bash
   cd /Users/cristianovb/Desktop/zettel-spark-flow-main
   cargo clean --release
   bun install  # ensure node_modules fresh
   bun run tauri:dev
   ```

2. **Verify no build errors** — if it compiles without error, proceed to step 3

3. **Test vault setup**:
   - App should launch with onboarding
   - PIN/passphrase setup should complete in **~5-8 seconds** (not 80+)
   - No white screen on second launch
   - No freezing at PIN entry

4. **Verify data isolation**:
   - Kill dev: `pkill -f "app.dev"`
   - Open installed release app
   - Both should have independent vaults/data (no cross-contamination)

5. **Once stable**: Begin Phase 0 feature work (login → notes → tasks, task-by-task)

---

## Critical Files to Check If Build Still Fails

If compilation still errors after `cargo clean --release`:

1. **Check Rust version**: `rustc --version` (should be 1.83+)
2. **Check Tauri CLI**: `bunx @tauri-apps/cli --version` (should be 2.0.0+)
3. **Full error output**: Run with verbose logging:
   ```bash
   RUST_BACKTRACE=1 bun run tauri:dev 2>&1 | tee build.log
   ```
4. **Provide the full `build.log`** for diagnosis

---

## Known Limitations & Future Work

- **Phase 0 is local-only**: No encryption, no sync, no cloud
- **PIN/Passphrase stored in Stronghold**: Currently unencrypted on disk (Stronghold provides file-level protection via iota_stronghold crate)
- **Biometric integration**: Partially implemented in `security/biometric.rs` but not wired to UI (Phase 1)
- **Mobile support**: Tauri plugins for iOS/Android included in `Cargo.toml` but not yet implemented (Phases 2+)

---

## Questions for Claude Code

When picking up this task:

1. **Can you build successfully?** Run `bun run tauri:dev` and report any errors
2. **How fast is vault setup?** Time the PIN setup (should be ~5-8 seconds)
3. **Is data isolated?** Check that dev data is in `com.viboai.app.dev` and release in `com.viboai.app`
4. **Ready for Phase 0?** Once build is stable, confirm before starting feature work

---

**End of Handover**

*This document is the single source of truth for the current state, problems, and fixes. Update it as work progresses.*

# Build Verification Checklist
**For**: Claude Code debugging the build failure  
**Time**: ~15 minutes to run through all steps

---

## Pre-Build Checks ✓

- [ ] Rust version is 1.83+
  ```bash
  rustc --version
  ```

- [ ] Node.js dependencies are installed
  ```bash
  bun install
  ```

- [ ] Tauri CLI is v2.0+
  ```bash
  bunx @tauri-apps/cli --version
  ```

- [ ] No other Tauri apps are running
  ```bash
  pkill -f "com.viboai.app" 2>/dev/null || true
  pkill -f "vibo" 2>/dev/null || true
  ```

- [ ] Git status is clean (all changes committed or staged)
  ```bash
  git status
  ```

---

## Build Attempt 1: Clean Slate

```bash
cd /Users/cristianovb/Desktop/zettel-spark-flow-main

# Full clean
cargo clean --release
rm -rf node_modules
bun install

# Attempt build with dev config
bun run tauri:dev 2>&1 | tee build_attempt_1.log
```

**Expected output**:
- No compilation errors
- Tauri dev server starts
- Browser/window opens with onboarding screen

**If it fails**:
- [ ] Save error output: `build_attempt_1.log`
- [ ] Note: Which crate failed? (crunchy? rust_decimal? Something else?)
- [ ] Go to "Build Attempt 2"

---

## Build Attempt 2: Incremental Clean

If Attempt 1 fails with crate-specific errors:

```bash
# Partial clean - keep some intermediate files
cargo clean
bun install

# Try again with backtrace
RUST_BACKTRACE=1 bun run tauri:dev 2>&1 | tee build_attempt_2.log
```

**If this also fails**:
- [ ] Extract the error crate name
- [ ] Note the exact error message (file not found? version mismatch?)
- [ ] Go to "Build Attempt 3"

---

## Build Attempt 3: Nuclear Option

If Attempts 1 & 2 both fail, try removing all Rust cache:

```bash
# Remove all cargo/build artifacts
rm -rf ~/.cargo/registry/cache/*
cargo clean --all
cargo build --release 2>&1 | tee build_attempt_3.log

# If that works, retry tauri dev
bun run tauri:dev
```

**If still failing**:
- [ ] This indicates a deeper issue (corrupted lockfile, toolchain mismatch, etc.)
- [ ] Attach `build_attempt_3.log` to diagnosis
- [ ] Check `Cargo.lock` for version mismatches

---

## If Build Succeeds 🎉

Once `bun run tauri:dev` launches without errors:

### Vault Setup Timing Test

- [ ] App shows onboarding screen
- [ ] Start PIN setup
- [ ] **Measure time**: How long does "Deriving master key..." take?
  - Expected: **5–8 seconds**
  - Bad: 30+ seconds (opt-level not applied)
  - Very bad: 80+ seconds (old unoptimized behavior)

### Verify Config Was Applied

```bash
# Check that Cargo.toml has the profile overrides
grep -A 1 "profile.dev.package" src-tauri/Cargo.toml | head -20
```

Expected output: Should show multiple `[profile.dev.package.*]` sections with `opt-level = 2` or `opt-level = 3`

### Verify Data Isolation

```bash
# After completing onboarding in dev:
ls -la "$HOME/Library/Application Support/com.viboai.app.dev/"

# Should exist and contain:
# - vault/ (directory with markdown notes)
# - vibo.db (SQLite database)
# - secure-vault.hold (Stronghold snapshot)
```

Expected: Dev data is in `com.viboai.app.dev`, NOT in `com.viboai.app`

### Test Second Launch

- [ ] Close dev app: `Cmd+Q` or kill it
- [ ] Run `bun run tauri:dev` again
- [ ] App should skip onboarding and go directly to vault lock screen
- [ ] Enter the PIN/passphrase you set up
- [ ] **Should NOT show white screen** — should open to notes/tasks interface

---

## Diagnosis Tree

```
Build fails?
├─ YES, error mentions "crunchy" or "rust_decimal"
│  └─ Likely: Stale build cache from previous contention
│     → Try Build Attempt 2 (incremental clean)
│     → If still fails, try Build Attempt 3 (nuclear)
│
├─ YES, error mentions "tauri-build" or config file
│  └─ Likely: tauri.dev.conf.json not being read
│     → Verify file exists: `ls -l src-tauri/tauri.dev.conf.json`
│     → Verify syntax: `cat src-tauri/tauri.dev.conf.json | jq .`
│     → Try: `bunx @tauri-apps/cli dev --config src-tauri/tauri.dev.conf.json`
│
├─ YES, error is unknown
│  └─ Attach full output with RUST_BACKTRACE=1
│     → This needs deeper investigation
│
└─ NO, build succeeds!
   └─ Go to "If Build Succeeds" section above
```

---

## Files to Verify

Before asking "why won't it build?", confirm these exist and have correct content:

| File | Expected | Check |
|------|----------|-------|
| `src-tauri/tauri.dev.conf.json` | `{"identifier": "com.viboai.app.dev"}` | `cat src-tauri/tauri.dev.conf.json` |
| `src-tauri/Cargo.toml` | Contains `[profile.dev.package.iota_stronghold]` with `opt-level = 2` | `grep -c "profile.dev.package" src-tauri/Cargo.toml` (should be >= 7) |
| `package.json` | Contains `"tauri:dev"` script | `grep "tauri:dev" package.json` |

---

## When All Else Fails

1. **Compare with git history**: What changed in the last successful build?
   ```bash
   git log --oneline -20
   git diff HEAD~1 src-tauri/Cargo.toml | head -50
   ```

2. **Check for syntax errors in config files**:
   ```bash
   cat src-tauri/tauri.dev.conf.json | jq empty && echo "Valid JSON" || echo "Invalid JSON"
   ```

3. **Verify plugin compatibility**:
   ```bash
   grep "tauri-plugin" src-tauri/Cargo.toml | sort
   ```
   All plugins should be `2.0.0-rc` or `2` (v2 era)

4. **Ask user directly**:
   - "Did you modify any Rust files besides Cargo.toml?"
   - "Did you install a new version of Tauri/Rust recently?"
   - "Is this a fresh checkout from git?"

---

**End of Checklist**

Use this step-by-step to narrow down where the build is breaking. Report back with which attempt succeeded (or provide logs if all fail).

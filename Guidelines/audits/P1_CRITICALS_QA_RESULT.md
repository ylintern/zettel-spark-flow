# P1 Criticals - QA Result
Date: 2026-04-08  
Status: PARTIAL - TEST 1 PASSED, APP-CORE QA STILL OPEN

⚠️ This memo now reflects real progress, not just planned QA.
Do not treat this as final sign-off.

## Policy Update
Changed QA approach from destructive shell deletion to non-destructive Finder-based testing:
- No `rm`, `cargo clean`, `git clean` commands
- Use Finder to move folders to archive names instead
- Preserve evidence for audits
- Manual comparison of recreated vs archived state

## Code Implementation Status: PARTIAL PASS

### Phase B: Folder Bootstrap
- [x] `bootstrap_app_directories()` added to [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
- [x] Creates `vault/`, `vault/notes/`, `vault/kanban/` idempotently
- [x] Wired into `setup()` before DB init
- [x] Compiles cleanly with zero errors

### Phase C: Reset Routing
- [x] [src/lib/vaultPhase.ts](src/lib/vaultPhase.ts) created (single source of truth)
- [x] [src/pages/Index.tsx](src/pages/Index.tsx) updated to handle all three phases
- [x] `configured=false -> onboarding` ✅ (was broken, now fixed)
- [x] `configured=true && unlocked=false -> lock` ✅
- [x] `configured=true && unlocked=true -> app` ✅
- [x] TypeScript builds cleanly

### Gate 0 Fix (Locked In)
- [x] SHA-256 canonical key derivation in [src-tauri/src/security/mod.rs](src-tauri/src/security/mod.rs)
- [x] Plugin builder uses `derive_vault_key()` from [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
- [x] All Stronghold operations use same 32-byte derivation
- [x] Passphrase setup verified working (user reported "password is working")

### Build Verification
- [x] TypeScript build reported successful
- [x] Tauri macOS `.app` bundle exists
- [x] Packaged app launch reported working
- [ ] Rust verification should be treated as stable only after the current environment noise is no longer conflicting across reports

## Manual QA

### Test 1: Folder Bootstrap (Using Finder Archive)

**Pre-Rename Check:**
1. In Finder, navigate to: `~/Library/Application Support/com.vibo.zettel-spark-flow/vault/`
2. Verify these archive names do NOT already exist:
   - `notes_archive_2026-04-08`
   - `kanban_archive_2026-04-08`
   - If they do exist, use unique suffix: `notes_archive_2026-04-08_01`, etc.

**Closure:**
1. Close app completely (use Cmd+Q, or verify in Activity Monitor it is closed)

**Archive Step:**
1. In Finder, rename (do NOT delete):
   - `notes/` → `notes_archive_2026-04-08`
   - `kanban/` → `kanban_archive_2026-04-08`
2. Verify both archived folders are visible in Finder

**Relaunch:**
1. Open app: `open /Users/cristianovb/Desktop/zettel-spark-flow-main/src-tauri/target/release/bundle/macos/vibo.app`
2. Wait 5 seconds for app to fully launch
3. Navigate back to `~/Library/Application Support/com.vibo.zettel-spark-flow/vault/` in Finder

**Expected Outcome (PASS):**
- [x] Fresh `notes/` directory exists (newly created)
- [x] Fresh `kanban/` directory exists (newly created)
- [x] Archive folders still present: `notes_archive_2026-04-08`, `kanban_archive_2026-04-08`
- [x] App launches without crash or error prompt
- [x] Workspace may be empty (old notes were in archived folder) — this is acceptable

**Result:** ✅ **PASS** — Executed 2026-04-08 at 09:43 UTC. Bootstrap folder recreation working correctly.

**Optional Stronger Check (Confirmation):**
1. Create a new note in app: "Test bootstrap recreation"
2. Close app (Cmd+Q)
3. Relaunch app
4. [ ] New test note persists (was written to fresh `notes/` folder)

### Conclusion
- Folder bootstrap behavior is working as intended in the packaged app.
- The backend self-heals `vault/notes` and `vault/kanban` on launch.
- This item can be marked done.

### Test 2: Reset Routing (Manual App Interaction)
1. If vault already set up: skip to step 4
2. Otherwise: Set up vault with PIN/passphrase
3. Create at least one note and one task
4. Settings → "Reset Encryption & PIN" → confirm twice
5. Observe app behavior

**Expected:**
- [ ] After reset succeeds, app transitions to **ONBOARDING** (not lock screen)
- [ ] Passphrase input cleared, ready for new setup
- [ ] Old notes and tasks are gone
6. Set new passphrase through onboarding → enters workspace
7. Create new note, close app completely, relaunch

**Expected on relaunch:**
- [ ] Lock screen shown (vault is configured)
- [ ] Can unlock with new passphrase and access new note

### Review
- Backend event path appears correct.
- Root phase derivation is correct in code.
- Remaining risk is product-flow consistency:
  - settings still expose a stale "Reset Encryption & PIN" button in one place
  - safe reset section exists separately
  - reset semantics need one canonical user-facing path

### Test 3: Normal Unlock Regression
1. Close app, relaunch

**Expected:**
- [ ] Lock screen shown (NOT onboarding)

2. Enter correct passphrase/PIN

**Expected:**
- [ ] App enters workspace normally

3. Close app, relaunch, enter wrong credential

**Expected:**
- [ ] Error shown, stays on lock screen

### Review
- This is still required before we declare the secure entry flow stable.
- It is especially important after the passphrase derivation changes.

### Test 4: Semantic Copy (Code Review, Not Destructive)
- [x] Search [SettingsView.tsx](src/components/SettingsView.tsx) for stale reset copy
  - Found: "Reset Encryption & PIN"
- [ ] Confirm no UI implies "change passphrase" feature exists
- [ ] Confirm lock icon only appears when `configured == true`

## App-Core Gaps Found During Review

### 1. Simple Notes / Kanban Still Need Real User QA
- Folder bootstrap passed, but this does not prove end-to-end durability for:
  - creating a normal note
  - editing a normal note
  - creating a kanban task
  - persisting those after close/relaunch

### 2. Settings Reset UX Is Split
- `SafeVaultResetSection.tsx` has the real destructive reset flow
- `SettingsView.tsx` still shows a separate stale button:
  - `Reset Encryption & PIN`
  - current behavior: alert placeholder
- Conclusion:
  - reset capability exists
  - reset UX contract is not clean yet

### 3. Private Note Encryption UX Contract Is Incomplete
- Private notes are encrypted in Rust on save
- UI can toggle note encryption directly from `NoteEditor`
- Current toggle does not ask for passphrase confirmation
- Current UX appears to rely only on vault unlocked state, not explicit user confirmation
- Conclusion:
  - storage/security path exists
  - user-facing encrypt/decrypt contract is still not mature enough to close

## Known Issues
- Semantic copy drift:
  - `Reset Encryption & PIN`
- Duplicate reset entrypoints in settings UX
- Private-note lock/encrypt UX needs explicit policy and QA

## Result Summary

**Code Status:**
- `[x]` Folder bootstrap implementation
- `[x]` Phase derivation implementation
- `[~]` Reset flow implementation
- `[~]` Private-note encryption UX
- `[~]` Simple note / kanban persistence still needs user QA

**QA Status:**
- `[x]` Test 1 complete: folder bootstrap PASS
- ✅ Non-destructive approach LOCKED IN
- `[ ]` Test 2: reset routing
- `[ ]` Test 3: normal unlock regression
- `[ ]` Simple note + kanban create/edit/relaunch QA
- `[ ]` Private note encrypt/decrypt UX QA

## Decision Gate

Do not proceed to model manager yet.

Next required order:
1. finish reset-routing QA
2. finish normal unlock regression QA
3. verify simple note + kanban persistence in real use
4. define and test private-note encrypt/decrypt UX contract
5. only then resume deeper Phase 1 integration

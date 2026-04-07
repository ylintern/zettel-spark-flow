# P0 Runtime Audit

## Purpose
Document and validate cold-start behavior, race conditions, and persistence guarantees during Phase 0.

## Cold Start Validation Checklist

### ✅ Tauri Startup Sequence
1. **App init** → `src-tauri/src/lib.rs::run()`
   - Create app data directory
   - Initialize SQLite pool
   - Create SecurityState with vault path
   - Load plugins (Stronghold, etc.)

2. **SecurityState ready** → vault snapshot path exists, session is None (locked)
   - Vault is **not** automatically unlocked on startup
   - User must unlock with passphrase or biometric

3. **Frontend mounts** → ReactDOM.render(App)
   - Initial state must **wait** for backend readiness
   - Component should check `is_vault_configured()` before showing lock screen

4. **Workspace load** → `load_workspace_snapshot()`
   - Fetches all notes from vault directory
   - Reads SQLite metadata

### ✅ Expected State After Cold Start
- [ ] Vault snapshot file exists on disk: `{app_data_dir}/secure-vault.hold`
- [ ] SQLite database exists: `{app_data_dir}/vibo.db`
- [ ] Note markdown files exist in vault directory
- [ ] SecurityState is locked (session.is_none())
- [ ] UI shows lock screen if vault is configured
- [ ] Normal notes are visible without unlock (unencrypted)

## Race Condition Checks

### Race 1: Tauri Setup vs Frontend Mount
**Scenario**: Frontend components mount before app state is ready
**Impact**: Commands fail with "AppState not ready"
**Mitigation**: 
- AppState is attached in `setup()` hook before plugins finish
- Frontend must await `invoke('is_vault_configured')` before rendering sensitive routes
- Use React.Suspense or similar to defer route load

**Status**: ✅ Implemented in AppSidebar check

### Race 2: Vault Lock While Reading
**Scenario**: User clicks "lock vault" while frontend is fetching notes
**Impact**: In-flight read returns "Vault locked" error
**Mitigation**:
- Frontend should debounce lock/unlock actions
- Pending note reads should timeout gracefully
- Display "Vault locked" message, don't crash

**Status**: 🟡 Needs frontend error boundary

### Race 3: Multiple Unlock Attempts
**Scenario**: User rapidly clicks unlock button with different passphrases
**Impact**: Multiple Stronghold sessions might queue
**Mitigation**:
- `SecurityState::unlock()` overwrites session Mutex
- Add unlock debounce in command handler (50ms)
- Return "already unlocking" if session transition in progress

**Status**: 🟡 Consider adding debounce

### Race 4: Private Note Read While Vault Locked
**Scenario**: Private note markdown is loaded, but vault locks before decrypt
**Impact**: Decrypt fails, note shows encrypted marker
**Mitigation**:
- Decrypt only happens in command handler, not on read
- Always check `is_vault_unlocked()` before decrypt attempt
- UI shows "this note is encrypted" instead of raw ciphertext

**Status**: ✅ Implemented in `decrypt_note_content()` guard

## File Persistence Validation

### Write Path: Note Save
```
Frontend: save_note("content") 
  ↓ 
Tauri Command: save_note()
  1. Encrypt if is_encrypted && vault unlocked
  2. Write to markdown file
  3. Update SQLite metadata
  4. Return success
  ↓
Frontend: update local cache, show "Saved"
```

**Failure modes**:
- [ ] File write fails (disk full, permissions) → return OS error, user sees "Save error"
- [ ] SQLite update fails → rollback? or recover on next load?
- [ ] Encrypt fails (vault locked) → return VaultLocked, show unlock prompt

**Action items**:
- [ ] Add transaction-like behavior: if SQLite fails, could we delete the markdown file?
- [ ] Document encryption failure mode

### Write Path: Secret Store
```
Frontend: store_secret("api_key", "value")
  ↓
Tauri Command: store_secret()
  1. Check vault unlocked
  2. Insert into Stronghold store
  3. Call stronghold.save() to write snapshot
  ↓
Frontend: show "API key saved"
```

**Validation**:
- [ ] Stronghold snapshot persists to `secure-vault.hold`
- [ ] Snapshot is locked (encrypted) on disk
- [ ] Next startup: `unlock_vault(passphrase)` loads snapshot correctly

## Restart Validation Tests

### Test 1: Normal Notes Persist
```bash
1. Launch app
2. Create note: "Hello, World!"
3. Force kill app (or close)
4. Launch app again
5. Verify note exists in workspace
```

**Expected**: ✅ Note appears in sidebar

**If fails**:
- [ ] Check vault directory exists
- [ ] Check SQLite has note metadata
- [ ] Check markdown file content

### Test 2: Vault Unlocks After Passphrase
```bash
1. Launch app, setup vault with passphrase "test"
2. Store secret: key="api", value="secret123"
3. Force kill app
4. Launch app
5. Vault status should be "locked"
6. Enter passphrase "test"
7. Verify secret retrieval works
```

**Expected**: ✅ Can unlock and retrieve secret

**If fails**:
- [ ] Check secure-vault.hold exists
- [ ] Check passphrase is correctly hashed
- [ ] Check Stronghold plugin loading

### Test 3: Private Notes Decrypt After Unlock
```bash
1. Unlock vault with passphrase
2. Create private note: "Secret thought"
3. Force kill app
4. Launch app (vault is locked)
5. Note shows encrypted marker (!vibo-encrypted:v1:...)
6. Enter passphrase to unlock
7. Note content becomes readable
```

**Expected**: ✅ Note decrypts correctly

**If fails**:
- [ ] Check note encryption key is in Stronghold
- [ ] Check markdown file has correct prefix
- [ ] Check AES-256-GCM decryption parameters

### Test 4: Invalid Passphrase Rejected
```bash
1. Setup vault with passphrase "correct"
2. Force kill app
3. Try to unlock with "wrong"
4. Should get "Invalid passphrase" error
```

**Expected**: ✅ Error message, no access

**If fails**:
- [ ] Check Stronghold validation
- [ ] Check error handling in unlock_vault command

## Error Handling Log

### Critical Errors (must fail gracefully)
- ❌ Vault locked during private note read → show "Encrypt lock" icon
- ❌ Stronghold snapshot corrupted → show "Vault corrupted, contact support"
- ❌ SQLite database corrupted → show "Database error, try reset vault"
- ❌ Disk full during write → show "Storage full, delete notes or data"

### Non-Critical Errors (warn but continue)
- ⚠️ Note metadata missing (file exists but no SQL row) → reconstruct from file
- ⚠️ Race condition: vault locked during in-flight read → retry after unlock
- ⚠️ API key store empty → show "No secrets stored"

## Signed Off
- **Phase 0 Verification**: P0-04, P0-05 Implemented and Smoke-Tested
- **Next Gate**: P0-06 (Mobile Biometrics) and P0-07 (Persistence Tests) must pass before Phase 1 kickoff
- **Audit Date**: 2026-04-07

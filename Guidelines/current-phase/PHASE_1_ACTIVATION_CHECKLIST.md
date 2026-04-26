> **STATUS (2026-04-26 review):** ✅ All Part A + B + C + D + E steps shipped 2026-04-24. See `Guidelines/source-of-truth/PHASE_0_COMPLETION.md` "2026-04-24 Update". This file is historical — kept for traceability. Marked for archive.

**Outcome:** Encryption activated 2026-04-24 with Fix A/B/C/D + reload-lock via `Builder::on_page_load`. Onboarding state migrated to `viboai/onboarding.json`. Part G (Secrets UI) and Part H (Biometric) remain explicitly deferred.

# Phase 1: Encryption Activation Checklist

**Timeline:** Post-Phase 0.6 (when ready)
**Objective:** Flip encryption flag, activate vault lifecycle, enable user secrets
**Effort:** ~1 day (flag + testing) + 3 days (encryption UX hardening)

---

## Part A: Trivial Flag Flip (30 min)

### A1. Enable Encryption in Backend

- [ ] **File:** `src-tauri/src/config/features.rs` (line 59)
  - Change: `FeatureFlags::phase_0()` → `FeatureFlags { encryption_enabled: true, ... }`
  - Rebuild: `cargo build`

### A2. Verify Compilation

- [ ] No new compiler errors
- [ ] No existing unit test failures
- [ ] `cargo test` passes

### A3. Build + Launch

- [ ] `bun run tauri:dev` compiles successfully
- [ ] Tauri window opens (native, not web)
- [ ] Dashboard loads (no blank UI)

---

## Part B: User Flow Testing (E2E, 2–3 hours)

### B1. Fresh Install (Onboarding with Vault Setup)

- [ ] **Scenario:** First launch, clean vault
  - [ ] Onboarding wizard appears
  - [ ] Step 3: "Security" asks for auth method (PIN, Passphrase, Biometrics)
  - [ ] User selects **Passphrase**
  - [ ] Continue to finish
  - [ ] **EXPECT:** LockScreen appears (not skipped like Phase 0)
  - [ ] **User enters passphrase** from onboarding
  - [ ] **EXPECT:** Workspace unlocks, `secure-vault.hold` created on disk
  - [ ] **Verify:** `ls ~/Library/Application\ Support/com.viboai.app.dev/viboai/database/secure-vault.hold` exists

### B2. Plaintext → Encrypted Note

- [ ] Create new note, toggle **lock icon** in NoteEditor
  - [ ] `is_private: true` set in store
  - [ ] Save note
  - [ ] **Verify:** File on disk starts with `!vibo-encrypted:v1:`
  - [ ] Check YAML frontmatter: `is_encrypted: true`
  - [ ] Check SQL: `is_encrypted = 1`

### B3. Encrypted Note Roundtrip

- [ ] Close app (Cmd+Q)
- [ ] Relaunch (`bun run tauri:dev`)
- [ ] **EXPECT:** LockScreen appears, vault is locked
  - [ ] Enter passphrase
  - [ ] Workspace unlocks
  - [ ] Encrypted note content readable and unmodified
  - [ ] Edit, save → re-encrypted with new nonce
  - [ ] Relaunch again → decrypt succeeds

### B4. Plaintext & Encrypted Mixed

- [ ] Create plaintext note (is_private=false)
- [ ] Create encrypted note (is_private=true)
- [ ] Both visible in Notebook view
- [ ] Plaintext readable without unlock (Phase 2 concern: disable this? Or keep plaintext notes accessible?)
  - [ ] **Decision point:** Should plaintext notes be readable before unlock? (TBD based on UX)

### B5. Wrong Passphrase Error

- [ ] Close app
- [ ] Relaunch
- [ ] LockScreen: enter **wrong** passphrase
  - [ ] **EXPECT:** Error toast "Invalid passphrase"
  - [ ] LockScreen still displayed
  - [ ] Try again with correct passphrase → works

### B6. Settings: Reset Passphrase

- [ ] Open Settings → Encryption card
  - [ ] Current passphrase field (masked)
  - [ ] New passphrase field
  - [ ] Confirm new passphrase
  - [ ] Click "Update Passphrase"
  - [ ] **EXPECT:** Success toast
  - [ ] Close app, relaunch
  - [ ] Enter **old** passphrase → fails
  - [ ] Enter **new** passphrase → succeeds

### B7. Settings: Vault Reset (Destructive)

- [ ] Open Settings → Vault Reset button
  - [ ] Warning dialog: "All encrypted data will be deleted"
  - [ ] Confirmation checkbox required
  - [ ] Confirm
  - [ ] **EXPECT:** `secure-vault.hold` deleted, all encrypted notes become unreadable
  - [ ] Relaunch → Onboarding appears again (vault not configured)
  - [ ] **VERIFY:** Plaintext notes still exist (but were never encrypted)

### B8. Biometric Button (Mobile Only)

- [ ] Desktop: LockScreen should NOT show Biometric button
- [ ] (Skip for now; Phase 2+ on actual iOS/Android device)

---

## Part C: Database & File Integrity (1 hour)

### C1. SQL is_encrypted Column

- [ ] Check existing notes table:
  ```bash
  sqlite3 ~/Library/Application\ Support/com.viboai.app.dev/viboai/database/vibo.db \
    "SELECT id, title, is_encrypted FROM notes LIMIT 5;"
  ```
  - [ ] Plaintext notes show `is_encrypted = 0`
  - [ ] Encrypted notes show `is_encrypted = 1`

### C2. Frontmatter is_encrypted Field

- [ ] Read encrypted note file:
  ```bash
  cat ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/notes/<uuid>.md
  ```
  - [ ] YAML block includes `is_encrypted: true`
  - [ ] Body is `!vibo-encrypted:v1:...`

### C3. Plaintext Note Integrity

- [ ] Read plaintext note:
  ```bash
  cat ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/notes/<other-uuid>.md
  ```
  - [ ] YAML block includes `is_encrypted: false`
  - [ ] Body is readable markdown

### C4. Nonce Randomization

- [ ] Edit encrypted note 3 times, save each time
- [ ] Extract encrypted body from file after each save:
  ```bash
  cat file.md | grep '!vibo-encrypted' | cut -d: -f3
  ```
  - [ ] **Three different nonce values** (Base64 after `v1:`)
  - [ ] Ciphertext also differs (authenticated encryption)

---

## Part D: Event Wiring & Phase Transitions (45 min)

### D1. Vault Status Events (Console Inspection)

- [ ] **First launch:** Onboarding → passphrase entry → setup
  - [ ] Browser DevTools Console: search for `vault_status_changed`
  - [ ] **EXPECT:** Event with `{ configured: true, unlocked: true, reason: "setup" }`

- [ ] **Relaunch:** LockScreen → unlock
  - [ ] **EXPECT:** Event with `{ configured: true, unlocked: false, reason: (none) }` on lock
  - [ ] Then `{ configured: true, unlocked: true, reason: "unlock" }` after user enters passphrase

### D2. Phase Derivation

- [ ] Check Index.tsx logic:
  - [ ] `phase = "onboarding"` → no vault file exists
  - [ ] `phase = "lock"` → vault exists, user hasn't unlocked
  - [ ] `phase = "app"` → vault unlocked, workspace loaded

### D3. Silent-Reuse (Encryption Enabled)

- [ ] **Scenario:** User completes onboarding + vault setup
- [ ] Close app, relaunch
  - [ ] **EXPECT:** LockScreen appears (phase = "lock")
  - [ ] **NOT onboarding** (onboarding is skipped if vault exists)
- [ ] Change `zettel-onboarding-done` localStorage key to false manually
- [ ] Relaunch
  - [ ] **EXPECT:** LockScreen still appears (phase = "lock" takes precedence)
  - [ ] **NOT onboarding** (vault state overrides localStorage)

---

## Part E: UI Polish & Error Handling (2 hours)

### E1. Error Messages

- [ ] **Invalid passphrase:** Clear, specific error (not generic "unlock failed")
  - [ ] Localize for i18n (future)

- [ ] **Vault not configured:** Onboarding prompt or error message
  - [ ] Should not happen (TBD: error handling for corrupted state)

- [ ] **Encryption failed on note save:** Surface to user (log if non-fatal)
  - [ ] Don't lose note data on encryption error

### E2. Settings UX

- [ ] Encryption settings card:
  - [ ] Only visible if `encryption_enabled === true` ✓ (already gated)
  - [ ] Status indicator: "AES-256 Active" vs. "No Passphrase Set" (if vault reset)
  - [ ] Reset Passphrase button highlights danger (red color or warning icon)

- [ ] Lock icon in NoteEditor:
  - [ ] Only visible if `encryption_enabled === true` ✓ (already gated)
  - [ ] Icon feedback: locked = 🔒, unlocked = 🔓 (already in place)
  - [ ] Disabled state if vault locked? (TBD: UX decision)

### E3. Onboarding Step Naming

- [ ] Step 3 ("Security") should clarify:
  - [ ] "Choose how to unlock your vault"
  - [ ] Show device capabilities for biometrics (not yet supported)

---

## Part F: Documentation Updates (1 hour)

### F1. User Guide

- [ ] Add "Encryption Setup" section:
  - [ ] First launch: choose passphrase
  - [ ] Passphrase is **non-recoverable** (no reset without vault loss)
  - [ ] Plaintext notes are readable without unlock (Phase 1 behavior)
  - [ ] Encrypted notes require unlock

### F2. Architecture Docs

- [ ] Update FOLDER_ARCHITECTURE.md to reference Phase 1:
  - [ ] Vault file location: `viboai/database/secure-vault.hold`
  - [ ] Encryption algorithm: AES-256-GCM
  - [ ] Key derivation: SHA-256(passphrase)

### F3. Troubleshooting

- [ ] "Passphrase lost" → Vault reset + recreate vault
- [ ] "Encrypted notes unreadable" → Check `secure-vault.hold` exists
- [ ] "Biometric not working" → Phase 2+ feature (not available yet)

---

## Part G: Secrets Integration (Phase 1.5+, Optional Now)

### G1. Expose Secret Commands via UI

- [ ] Create Settings subpage: "Secrets"
- [ ] Button: "Add Secret"
  - [ ] Name: "openrouter_api_key"
  - [ ] Value: (masked input)
  - [ ] Save → calls `store_secret(name, value)`

### G2. Secret Retrieval

- [ ] Query secrets on demand (inference initialization)
  - [ ] `get_secret("openrouter_api_key")` → decrypted value
  - [ ] Used in API calls (Tauri command, not exposed to frontend)

### G3. Secret Deletion

- [ ] Settings → Secrets → Delete button
  - [ ] `delete_secret(name)`

---

## Part H: Biometric Integration (Phase 2+, Not Now)

### H1. Research Required

- [ ] [ ] Tauri biometric plugin compatibility (if exists)
- [ ] [ ] iOS Secure Enclave integration (Apple Security framework)
- [ ] [ ] Android BiometricPrompt integration (androidx.biometric)

### H2. Implementation (Deferred)

- [ ] Replace `verify_biometric_and_unlock()` stub
- [ ] Test on real iOS/Android devices
- [ ] Fallback to passphrase entry if biometric fails

---

## Part I: Testing Checklist

### I1. Unit Tests (Backend)

- [ ] Test `encrypt_note_content()` + `decrypt_note_content()` roundtrip
  - [ ] Special characters: é, 中文, emoji
  - [ ] Long content (10K+ characters)
  - [ ] Empty content

- [ ] Test `setup()`, `unlock()`, `lock()` vault state machine
  - [ ] Invalid passphrase rejection
  - [ ] Multiple unlock/lock cycles

- [ ] Test reconciliation with encrypted notes:
  - [ ] Foreign `.md` file with `is_encrypted: true` → adoption

### I2. Integration Tests (Full Flow)

- [ ] Onboarding → Passphrase → Vault Creation
- [ ] Note creation → Toggle is_private → Encrypt → Decrypt
- [ ] Passphrase change → Re-encrypt all notes with new key

### I3. Stress Tests

- [ ] 100 encrypted notes → lock → unlock → all readable
- [ ] Rapid encryption toggle (private → plaintext → private)
- [ ] Concurrent note edits during vault lock (should queue until unlock)

---

## Part J: Rollback Plan (If Needed)

### J1. Quick Disable

- [ ] Revert flag in `src-tauri/src/config/features.rs:59`
- [ ] Rebuild, relaunch
- [ ] Phase 0 behavior restored (LockScreen skipped)
- [ ] Encrypted notes become unreadable (data not lost, just gated)

### J2. Data Recovery

- [ ] If `secure-vault.hold` corrupted:
  - [ ] User cannot unlock vault
  - [ ] Encrypted notes are permanently unreadable (no recovery key)
  - [ ] Plaintext notes remain readable
  - [ ] Recommend vault reset + re-entry of notes

---

## Sign-Off Template

| Milestone | Owner | Status | Date | Notes |
|-----------|-------|--------|------|-------|
| A. Flag flip | Dev | ✅ | 2026-04-24 | `encryption_enabled: true` |
| B. E2E testing | QA | ✅ | 2026-04-24 | All B1–B7 passing |
| C. Database checks | Dev | ✅ | 2026-04-24 | is_encrypted column live |
| D. Event wiring | Dev | ✅ | 2026-04-24 | + reload-lock via `Builder::on_page_load` |
| E. UI polish | Design/Dev | ✅ | 2026-04-24 | LockScreen + Settings card live |
| F. Docs updated | Tech Writer | 🟡 | 2026-04-26 | Source-of-truth updated; user-facing guide pending |
| G. Secrets (optional) | Dev | ⬜ | — | Phase 1.5 — still deferred |
| H. Biometric (Phase 2) | Dev | ⬜ | — | Deferred — awaits hardware plugin |
| Phase 1 Complete | Product | ✅ | 2026-04-24 | Released as part of Phase 0.7-A activation batch |

---

## Estimated Timeline

| Task | Hours | Owner | Dependency |
|------|-------|-------|------------|
| A. Flag flip + build | 0.5 | Dev | None |
| B. E2E testing | 3 | QA | A |
| C. Database checks | 1 | Dev | A |
| D. Event verification | 0.75 | Dev | B |
| E. UI polish | 2 | Design/Dev | B |
| F. Docs | 1 | Tech Writer | B, E |
| **Total (Phase 1)** | **~8 hours** | — | — |
| G. Secrets integration | 4 | Dev | F (optional) |
| H. Biometric (Phase 2) | 16+ | Dev | F (deferred) |

---

## Go/No-Go Criteria

### Go (Proceed to Production)

✅ All of B1–B8 pass  
✅ C1–C4 database checks pass  
✅ D1–D3 event wiring verified  
✅ E1–E3 UI polished  
✅ F1–F3 docs updated  
✅ No regressions in Phase 0 features (Notebook, Kanban, reconciliation)

### No-Go (Extend Testing or Rollback)

❌ Encryption/decryption fails on any character set  
❌ Passphrase entry rejects valid input  
❌ Encrypted notes become corrupted on edit/reload  
❌ Vault lock/unlock state machine hangs or deadlocks  
❌ E2E test failure with no clear root cause

---

**Prepared:** 2026-04-19  
**Next Review:** Post-implementation  
**Revision:** 1.0

# TO_DO: Phase 0 → Phase 0.7 → Phase 1 Checklist

**Last Updated:** 2026-04-19  
**Status:** Phase 0 feature-complete; Phase 0.7 backlog prioritized; Phase 1 deferred.

---

## Quick Checklist (Phase 0.7 High Priority)

### Stronghold Lock/Unlock Activation
- [ ] Flip `encryption_enabled: true` in `src-tauri/src/config/features.rs:51`
- [ ] Add Lock Vault button to `src/components/SettingsView.tsx` (~5 LoC)
- [ ] Test E2E: Onboarding → PIN → Settings → Lock → LockScreen → Unlock
- [ ] Verify: Wrong PIN fails; correct PIN unlocks

### Reset Operations UI Cards (Steps 2–6)
- [ ] Step 2: Reset Onboarding card (`ResetOnboardingSection.tsx` + `reset_onboarding()` command)
- [ ] Step 3: Delete All Notes card (`ClearAllNotesSection.tsx` + `clear_all_notes()` command)
- [ ] Step 4: Delete Vault with keep-files checkbox (`SafeVaultResetSection.tsx` refactor + `factory_reset(keep_vault_files: bool)`)
- [ ] Step 5: Zone layout (Account & Security / Danger Zone sections)
- [ ] Step 6: Placeholder card (Delete Specific Folders — disabled, Phase 0.8 label)

### Onboarding State Migration (localStorage → File)
- [ ] Create `src-tauri/src/commands/onboarding.rs` with `read_onboarding()`, `write_onboarding()`, `reset_onboarding()`
- [ ] Refactor `src/components/OnboardingWizard.tsx` to use async Tauri calls
- [ ] Test: Fresh install → onboarding file created; reset → file deleted; restart → wizard re-appears

### Testing & Verification
- [ ] All E2E smoke tests pass (onboarding PIN, onboarding Passphrase, lock/unlock, reset PIN, kanban DnD)
- [ ] Bundle-ID WebKit origin issue resolved (no legacy localStorage leaks)
- [ ] FS-SQL sync verified (create note in app → .md in Finder; edit .md → restart → SQL updated)
- [ ] Obsidian compatibility verified (vault opens in Obsidian with correct frontmatter)
- [ ] Kanban race condition stable (rapid drag-drop → final order matches last drop)

### Documentation & Handoff
- [ ] Commit all code changes to GitHub
- [ ] Push to main branch
- [ ] Verify team has access to Guidelines/ documentation

---

## Detailed Task Matrix (Phase 0.7)

| Task | Owner | Status | Est. Hours | Priority | Acceptance Criteria | Notes |
|------|-------|--------|-----------|----------|---------------------|-------|
| **Stronghold Activation** | — | Pending | 2 | HIGH | Lock button visible; lock→LockScreen; unlock works | Just 1-line flag flip + 5 LoC button; infrastructure ready |
| Step 2: Reset Onboarding | — | Pending | 3 | HIGH | Auth via old pass; snapshot deleted; wizard re-runs | Calls `reset_onboarding()` backend command |
| Step 3: Delete All Notes | — | Pending | 2 | HIGH | Auth via pass; SQL cleared; myspace/ user folders wiped | Preserves reserved folders (Inbox, Archive) |
| Step 4: Delete Vault + Checkbox | — | Pending | 3 | HIGH | Checkbox toggles behavior: wipe all vs. keep .md files | Refactor existing `SafeVaultResetSection` |
| Step 5: Zone Layout | — | Pending | 1.5 | HIGH | Cards grouped into 2 sections; danger zone red accents | Pure UI refactoring in SettingsView |
| Step 6: Placeholder Card | — | Pending | 0.5 | HIGH | Disabled, labeled "Phase 0.8"; styled consistently | Stub component; no backend work |
| Onboarding State Migration | — | Pending | 4 | HIGH | File-based state; async Tauri calls; no localStorage for onboarding | Move `vibo-onboarding-done`, `vibo-ai-config` to file |
| E2E Testing | — | In Progress | 3 | HIGH | All smoke tests + matrix passing | Use native Tauri window; verify .md in Finder |
| GitHub Commit & Push | — | Pending | 0.5 | HIGH | All changes on main; CI passes | Documentation + code |

---

## Phase 0.7 Test Matrix

| Scenario | Setup | Action | Verify | Owner | Status |
|----------|-------|--------|--------|-------|--------|
| **Onboarding → PIN** | Fresh install (wipe vault) | Complete wizard, choose PIN "0000", finish | LockScreen respects PIN; can unlock & re-lock | — | Pending |
| **Onboarding → Passphrase** | Fresh install | Complete wizard, choose Passphrase "test1234", finish | LockScreen respects passphrase; can unlock & re-lock | — | Pending |
| **Lock After Setup** | Complete onboarding | Settings → Lock Vault button → Enter correct PIN | Phase transitions to "lock"; LockScreen appears | — | Pending |
| **Reset PIN** | Locked vault with PIN "0000" | Settings → Reset Pass → old "0000", new "1111" → Restart | Old PIN fails; new PIN works | — | Pending |
| **Delete All Notes** | 5 notes in vault | Settings → Delete All Notes → confirm → restart | SQL notes table empty; myspace/ user folders wiped; Inbox/Archive intact | — | Pending |
| **Delete Vault (Wipe All)** | Vault with data | Settings → Delete Vault → uncheck "keep files" → confirm | All deleted; next restart → fresh wizard | — | Pending |
| **Delete Vault (Keep Files)** | Vault with data | Settings → Delete Vault → check "keep files" → confirm | .md files remain in myspace/; SQL wiped; next restart → fresh wizard | — | Pending |
| **Kanban DnD Stability** | 5 notes, 3 folders | Rapid drag-drop between folders (last = final) | SQL + .md match last drop position; no race | — | Pending |
| **Obsidian Compat** | Create 3 notes in app | Open myspace/ in Obsidian | Notes visible with correct frontmatter; edit in Obsidian; restart app | — | Pending |
| **Reset Onboarding** | Existing vault, onboarding done | Settings → Reset Onboarding → confirm → restart | Wizard re-runs from Step 1; prior vault data preserved | — | Pending |
| **FS-SQL Sync** | Create note in app | Edit .md outside app in Finder; restart | SQL note reflects edits from .md | — | Pending |
| **WebKit Origin Cleanup** | Fresh install | Check browser dev tools localStorage | Only `vibo-*` keys present; no `zettel-*` leaks | — | Verified (Phase 0) |

---

## Known Issues & Edge Cases

| Issue | Severity | Root Cause | Mitigation | Phase |
|-------|----------|-----------|-----------|-------|
| WebKit origin drift (bundle-ID renames) | **FIXED** | `zettel-*` keys leaked across bundle-ID renames | Renamed all keys to `vibo-*`; legacy cleanup in reset | Phase 0 ✓ |
| Kanban race condition (atomic I/O) | **FIXED** | setState() + persintNote() not atomic | notesRef eager-compute pattern (commit debcfcf) | Phase 0 ✓ |
| Onboarding gate false positive | **FIXED** | Reserved folders (auto-created) triggered silent-reuse | Filter reserved folders before heuristic; check user data only | Phase 0 ✓ |
| YAML parse crashes on user edits | **KNOWN** | User edits .md with broken YAML syntax | Parser skips + logs error (Phase 1 fix: validate + prompt) | Phase 1 |
| Stronghold key derivation mismatch | **OBSERVED** | If key derivation differs between frontend/backend | Verify SHA-256 + 32-byte output matches | Ongoing |
| No file-level vault locking | **ASSUMPTION** | Concurrent app instances could corrupt .md + SQL | Single-instance app assumption holds (dev + build don't run concurrently) | Phase 1 |

---

## Phase 1 Medium-Priority Backlog

| Feature | Status | Est. Hours | Priority | Notes |
|---------|--------|-----------|----------|-------|
| Selective Folder Deletion | Pending | 6 | MEDIUM | Checkbox UI + batch SQL deletion + FS cleanup |
| Folder Rename/Move (3-dot menu) | Pending | 4 | MEDIUM | Context menu + Tauri commands |
| Encryption Key Rotation | Pending | 8 | MEDIUM | Complex; re-encrypt snapshot without re-setup |
| Cloud Backup Before Reset | Blocked | — | MEDIUM | Blocks on cloud integration (Phase 2) |
| File-Level Vault Locking | Pending | 3 | LOW | Prevent concurrent write corruption |
| YAML Validation on Sync | Pending | 2 | LOW | Graceful handling of user-edited .md files |
| Atomic Transaction Markers | Pending | 4 | LOW | Ensure .md + SQL writes in single TX |

---

## Integration Points for Phase 0.7 Activation

**Stronghold Lock/Unlock:**
1. File: `src-tauri/src/config/features.rs:51` — change `encryption_enabled: false` → `true`
2. File: `src/components/SettingsView.tsx:189` — add Lock Vault button
3. Test: `bun run tauri:dev` → complete onboarding → Settings → Lock → enter PIN → unlock

**Reset Operations (6 Steps):**
1. Backend commands in `src-tauri/src/commands/security.rs` + `src-tauri/src/commands/workspace.rs`
2. Frontend cards in `src/components/settings/`
3. Test each step via UI; verify SQL + FS state after each operation

**Onboarding State Migration:**
1. Create `src-tauri/src/commands/onboarding.rs`
2. Refactor `src/components/OnboardingWizard.tsx` to call Tauri instead of localStorage
3. Test: Fresh install → file-based state; reset → file deleted; re-run wizard

---

## Commands for Testing & Debugging

```bash
# Fresh start (clean slate)
rm -rf ~/Library/Application\ Support/com.viboai.app.dev
rm -rf ~/Library/WebKit/com.vibo.zettel-spark-flow
rm -rf ~/Library/WebKit/app
bun run tauri:dev

# Verify vault state
sqlite3 ~/Library/Application\ Support/com.viboai.app.dev/viboai/database/vibo.db \
  "SELECT COUNT(*) as notes, COUNT(DISTINCT folder_id) as folders FROM notes;"

# List myspace contents
ls -la ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/

# Check WebKit localStorage
sqlite3 ~/Library/WebKit/app/WebsiteData/Default/*/LocalStorage/localstorage.sqlite3 \
  "SELECT key FROM ItemTable WHERE key LIKE '%vibo%' OR key LIKE '%zettel%';"

# Inspect Stronghold snapshot
ls -la ~/Library/Application\ Support/com.viboai.app.dev/viboai/database/secure-vault.hold

# Run tests
bun run test

# Type-check
bun run build

# Rebuild Tauri
bun run tauri:build
```

---

## Sign-Off Checklist (Before Phase 0.7 Handoff)

- [ ] All Phase 0 features verified working (feature matrix green)
- [ ] E2E smoke tests documented and passing
- [ ] No `zettel-*` references remain in code or config
- [ ] Stronghold activation recipe documented and tested
- [ ] Reset operations UI complete and tested
- [ ] Onboarding state migration (file-based) complete
- [ ] Guidelines documentation (PHASE_0_COMPLETION, PHASE_0_7_BACKLOG, etc.) up-to-date
- [ ] Handoff summary (HANDOFF_SUMMARY_2026-04-19.md) provided to team
- [ ] Memory updated with Phase 0 completion status
- [ ] All commits pushed to GitHub
- [ ] Team has questions answered before Phase 0.7 starts

---

## Questions Before Phase 0.7 Kickoff

1. **Stronghold Activation Scope:** Just Lock button + flag flip, or full lock/unlock flow test?
2. **Reset Operations Order:** Execute Steps 2–6 in sequence, or are there dependencies?
3. **Onboarding Migration Timing:** Phase 0.7 or defer to Phase 1?
4. **CI/CD Integration:** Do we have automated E2E tests, or manual only?
5. **Bundle-ID Stability:** Will the app bundle ID change again? (Affects localStorage strategy)

See detailed backlog in [`Guidelines/current-phase/PHASE_0_7_BACKLOG.md`](Guidelines/current-phase/PHASE_0_7_BACKLOG.md).

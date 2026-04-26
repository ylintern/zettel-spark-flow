# Phase 0.7 & Beyond — Backlog

**Created:** 2026-04-19
**Last reviewed:** 2026-04-26 — Stronghold activation done; T9/T10/T12 done; T8/T11 + Reset-UI Steps 2–6 still pending.
**Deferred from Phase 0:** Stronghold activation, advanced reset operations, selective deletion.

---

## High Priority — Phase 0.7 (likely Q2 2026)

### 1. Activate Stronghold Lock/Unlock

**Status:** Infrastructure ready; needs UI + feature flag.

**Tasks:**
- [x] Flip `encryption_enabled: true` in `src-tauri/src/config/features.rs:51` — done 2026-04-24
- [x] Add "Lock Vault" button to `src/components/SettingsView.tsx` (call `lockVault()` from crypto.ts) — done 2026-04-24
- [x] Test E2E: Onboarding → choose PIN/Passphrase → finish → unlock → settings → lock → LockScreen appears — done 2026-04-24
- [x] Verify: wrong PIN → error; correct PIN → unlock → app — done 2026-04-24

> ✅ **Section complete 2026-04-24.** See `Guidelines/source-of-truth/PHASE_0_COMPLETION.md` "2026-04-24 Update" — also includes Fix A/B/C/D and reload-lock via `Builder::on_page_load`.

**Files to modify:**
- `src-tauri/src/config/features.rs` (1-line change)
- `src/components/SettingsView.tsx` (~5 LoC addition)

**Test commands:**
```bash
bun run tauri:dev
# Complete onboarding with PIN "0000"
# Settings → Lock Vault button → LockScreen appears
# Enter wrong PIN → error message
# Enter "0000" → unlocks → back to app
```

**Acceptance Criteria:**
- Lock button visible when vault is unlocked
- Lock button disabled while processing
- LockScreen prompts for PIN/Passphrase after lock
- Correct secret unlocks; incorrect fails with error

---

### 2. Complete Reset Operations UI (from reset-UI plan)

**Status:** ResetPassSection created (Step 1); remaining 5 steps pending.

**Step 2: Reset Onboarding Card**
- Input: old pass (auth)
- Action: delete Stronghold snapshot + clear onboarding localStorage keys
- Effect: next restart → full wizard re-runs
- File: NEW `src/components/settings/ResetOnboardingSection.tsx`
- Backend: NEW command `reset_onboarding()` in `src-tauri/src/commands/security.rs`

**Step 3: Delete All Notes Card**
- Input: pass (auth)
- Action: wipe myspace/ contents (user folders) + clear SQL notes table
- Effect: vault skeleton recreated on next launch
- File: NEW `src/components/settings/ClearAllNotesSection.tsx`
- Backend: NEW command `clear_all_notes()` in `src-tauri/src/commands/workspace.rs`

**Step 4: Delete Vault (Full Wipe) with Checkbox**
- Input: pass (auth) — or skip for unrecoverable wipe
- Checkbox: "Keep folders & notes on disk (Obsidian compatibility)"
- Action: wipe everything OR wipe SQL + Stronghold only (keep .md)
- File: REFACTOR `src/components/settings/SafeVaultResetSection.tsx` (add checkbox)
- Backend: EXTEND `factory_reset()` with `keep_vault_files: bool`

**Step 5: Zone Layout**
- Group Cards 1–2 in "Account & Security" section
- Group Cards 3–4 in "Danger Zone" section with red accents
- File: REFACTOR `src/components/SettingsView.tsx`

**Step 6: Placeholder Card**
- "Delete Specific Folders" — disabled, "Coming in Phase 0.8" label
- File: NEW `src/components/settings/DeleteFoldersSection.tsx` (stub)

**Timeline:** 1 step per day + user testing = ~1 week

---

### 3. Onboarding State Migration (localStorage → File)

**Problem:** localStorage scoped by WebKit origin. Bundle-ID renames break persistence.

**Solution:** Move `vibo-onboarding-done` + `vibo-ai-config` to file in vault dir.

**Files:**
- NEW: `src-tauri/src/commands/onboarding.rs`
  - `read_onboarding()` → Option<OnboardingState>
  - `write_onboarding(state)` → atomic write
  - `reset_onboarding()` → delete file (used by reset card)

- REFACTOR: `src/components/OnboardingWizard.tsx`
  - Replace `localStorage.getItem(AI_CONFIG_KEY)` with `await read_onboarding()`
  - Replace `localStorage.setItem(...)` with `await write_onboarding(...)`

**Timeline:** ~4 hours (+ testing)

---

## Medium Priority — Phase 1 (Q3 2026)

### 4. Selective Folder Deletion with Checkboxes

**Feature:** Allow user to pick which folders to delete (keep others + their notes).

**Files:**
- REFACTOR: `src/components/settings/DeleteFoldersSection.tsx` (enable)
- NEW: `src-tauri/src/commands/workspace.rs` function `delete_folders(ids: Vec<String>)`

**Complexity:** Requires folder listing UI, batch SQL deletion, FS cleanup.

### 5. Folder Rename/Move 3-Dot Menu

**Feature:** Right-click folder → rename or move to another.

**Files:**
- `src/components/DashboardView.tsx` — context menu
- `src-tauri/src/commands/workspace.rs` — `rename_folder()`, `move_folder()`

### 6. Encryption Key Rotation

**Feature:** Re-encrypt Stronghold snapshot with new passphrase without re-setup.

**Complexity:** High. Requires re-deriving vault key, re-encrypting snapshot.

### 7. Cloud Backup Before Reset

**Feature:** Before wipe, offer to backup notes to cloud (if provider configured).

**Blocks:** Cloud integration not yet implemented.

---

## Known Blockers & Constraints

| Blocker | Severity | Resolution |
|---------|----------|-----------|
| Encryption flag dormant (Phase 0) | Medium | Flip flag in Phase 0.7; unlock flow already implemented |
| Onboarding state in localStorage | Low | Defer to Phase 0.7 migration |
| Selective folder deletion not implemented | Low | Phase 1 scope |
| Cloud backup integration missing | Medium | Blocks Phase 1 backup feature |
| Bundle-ID WebKit drift lessons | Info | Document; prevent in Phase 1 (use file-based state) |

---

## Testing Strategy for Phase 0.7

### Smoke Test Suite
```bash
# 1. Fresh install
rm -rf ~/Library/Application\ Support/com.viboai.app.dev
bun run tauri:dev
# Verify: Wizard appears, complete with PIN "0000"

# 2. Lock/Unlock cycle
# Settings → Lock Vault → Enter "0000" → Unlock → back to app

# 3. Reset PIN
# Settings → Reset Pass → Old "0000", New "1111", Confirm → Restart
# Verify: Old PIN fails, new PIN works

# 4. Kanban operations
# Create 5 notes → rapid drag-drop → verify final order matches
```

### E2E Test Matrix
| Scenario | Setup | Action | Verify |
|----------|-------|--------|--------|
| Onboarding → PIN | Fresh install | Choose PIN, finish | LockScreen respects PIN |
| Onboarding → Passphrase | Fresh install | Choose passphrase, finish | LockScreen respects passphrase |
| Lock after setup | Complete onboarding | Settings → Lock | Phase transitions to lock |
| Reset PIN | Locked vault | Settings → Reset | Old pass invalid, new pass works |
| Drag-drop reliability | 5 notes, 3 folders | Rapid DnD | Final order = last drop |
| Obsidian compat | Fresh vault | Create note in app | .md visible in Finder |

---

## Handoff Notes for Next Team

- **Stronghold state machine** is straightforward (lock = clear session; unlock = load snapshot)
- **Phase routing** driven by events; keep that pattern
- **localStorage** now brand-aligned (`vibo-*`); migrate to file in Phase 0.7
- **FS-first architecture** works; .md is source of truth
- **Testing:** Always verify E2E on native Tauri window (not localhost); SQLite state matters

See [`READY_FOR_HANDOFF.md`](../agents/READY_FOR_HANDOFF.md) for deep-dive code architecture.

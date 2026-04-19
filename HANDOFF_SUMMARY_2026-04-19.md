# Phase 0 Handoff Summary — April 19, 2026

**For:** Senior Engineer / Tech Lead taking over Phase 0.7 work  
**From:** Phase 0 completion audit  
**Status:** Feature-complete, infrastructure validated, ready for activation & continuation.

---

## Executive Summary

**Phase 0 is feature-complete.** All core infrastructure for onboarding, vault security, note persistence, and PIN/Passphrase management is implemented and tested. The primary work ahead is UI additions (Lock button, reset operation cards) and a onboarding state migration (localStorage → file). No breaking changes expected.

**Key Findings:**
- ✅ Stronghold lock/unlock backend: **fully implemented**, dormant (flag-gated)
- ✅ Onboarding wizard: **fully functional**, dynamic PIN/Passphrase selection
- ✅ FS-first reconciliation: **working**, .md files authoritative, SQL mirrors
- ✅ Reset PIN/Passphrase: **active**, verified in SettingsView
- ⚠️ Lock button in Settings: **missing**, trivial 5-line addition
- ⚠️ Reset operations UI (Steps 2–6): **pending**, 1–2 weeks estimated
- ⚠️ Onboarding state migration: **pending**, move from localStorage to file (~4 hours)

**Phase 0.7 Activation Recipe:**
1. Flip `encryption_enabled: true` in config (1 line)
2. Add Lock button to Settings (~5 LoC)
3. Test E2E: Onboarding → choose PIN → Settings → Lock → LockScreen → Unlock
4. Rebuild, verify, ship.

**Critical Lesson Learned:** WebKit localStorage scoped by bundle identifier. When app bundle renamed (zettel-spark-flow → com.viboai.app), old localStorage origin leaked across versions. **Solution:** Renamed all `zettel-*` keys to `vibo-*` namespace. Phase 0.7 plan moves onboarding state to file to avoid future origin drift.

---

## Architecture Deep-Dive

### 1. Stronghold State Machine (Backend)

**File:** `src-tauri/src/security/mod.rs`

**State Model:**
```rust
pub struct SecurityState {
    snapshot_path: PathBuf,
    session: Mutex<Option<Stronghold>>,  // None = locked; Some = unlocked
}
```

**Transitions:**
- **Locked** → **Unlocked:** `unlock(passphrase)` loads snapshot from disk, derives key (SHA-256), validates, stores in session
- **Unlocked** → **Locked:** `lock()` saves snapshot, clears session (session = None)
- **Query state:** `is_unlocked()` checks if session is Some; `is_configured()` checks if snapshot file exists

**Key Detail:** Session is **in-memory only**. On app restart, session = None (locked). Must unlock via LockScreen. No persistent session tokens.

**Tauri Commands (Already Implemented):**
```
setup_secure_vault(secret: String) → Creates snapshot, initializes session
unlock_vault(passphrase: String) → Loads snapshot, validates, unlocks
lock_vault() → Saves snapshot, clears session, emits vault_status_changed event
is_vault_unlocked() → Returns boolean
is_vault_configured() → Returns boolean
reset_passphrase(current: String, newPassphrase: String) → Changes encryption key
factory_reset() → Wipes snapshot + SQL (full reset)
```

All commands emit `vault_status_changed` event to notify frontend of state changes.

### 2. Phase Routing (Frontend State Machine)

**File:** `src/pages/Index.tsx:220–251`

**Logic:**
```
on mount:
  flags = await getFeatureFlags()
  vault_status = await isVaultConfigured(), isVaultUnlocked()
  onboarding_done = localStorage.getItem("vibo-onboarding-done")
  
  if not flags.encryption_enabled:
    # Phase 0 mode (encryption dormant, feature flag off)
    if onboarding_done: phase = "app"
    else: phase = "onboarding"
  else:
    # Phase 2/3 mode (encryption active, feature flag on)
    if not vault_status.configured: phase = "onboarding"
    else if vault_status.unlocked: phase = "app"
    else: phase = "lock"

listen to vault_status_changed event:
  # Backend emits after lock/unlock/setup
  recalculate phase, re-sync UI
```

**Key Points:**
- No explicit navigation commands; phase derived from backend state + localStorage
- Events drive sync; frontend is reactive to backend changes
- Feature flag gates entire encryption flow (easy on/off for gradual rollout)

### 3. FS-First Reconciliation Pattern

**Files:** `src-tauri/src/vault/mod.rs`, `src-tauri/src/vault/reconcile.rs`

**Architecture:**
```
myspace/ (user-visible vault)
├── Inbox/ (reserved, auto-created)
├── Archive/ (reserved, auto-created)
├── user-folder-1/
└── note.md (YAML frontmatter + markdown body)

vibo.db (SQL cache)
├── notes table (mirrors .md metadata)
├── columns table (view state)
└── folders table (folder structure)
```

**Sync Logic (on app startup):**
1. Walk `myspace/` directory → list all `.md` files
2. Parse YAML frontmatter from each `.md`
3. Upsert to SQL `notes` table
4. Any SQL note NOT in FS → marked deleted or warn user
5. **Result:** SQL mirrors FS state exactly

**Key:** `.md is source of truth.` If SQL diverges, reconciliation overwrites SQL from .md. Users can edit `.md` outside app (in Finder or Obsidian); next app restart syncs changes.

**Atomicity Fix (commit debcfcf):** notesRef eager-compute pattern ensures React state + persist I/O happen together, not split across turns.

### 4. Onboarding Flow

**File:** `src/components/OnboardingWizard.tsx`

**Steps:**
1. **Model Selection** — user chooses local model, configures endpoint
2. **Integrations** — enable cloud providers (optional)
3. **Security** — user picks auth method from dynamic list:
   - PIN: 4–6 digits
   - Passphrase: 8+ characters
   - Selection stored in OnboardingConfig.authMethod
4. **Name & Finish** — user enters app name

**Flow:**
- Wizard finish → `handleOnboardingComplete()` → checks `encryption_enabled` flag
  - If flag ON: advance to "lock" phase → LockScreen prompts for secret again → `setupSecureVault(secret)` → creates Stronghold snapshot
  - If flag OFF (Phase 0): advance to "app" phase directly
- localStorage updated: `vibo-onboarding-done = "true"` + AI config saved

**Auth Method Handling:**
- Frontend stores choice in OnboardingConfig
- LockScreen reads `authMethod` to determine input type (PIN keyboard vs. text)
- Backend doesn't store method; Stronghold just validates passphrase (works for both PIN + passphrase — both are strings)

### 5. localStorage Key Namespace

**Phase 0 Solution:** Renamed all keys from `zettel-*` to `vibo-*`.

**Files Updated:**
- `src/components/OnboardingWizard.tsx` (3 keys)
- `src/components/SettingsView.tsx` (1 key)
- `src/lib/crypto.ts` (1 key)
- `src/pages/Index.tsx` (1 key)
- `src/components/settings/SafeVaultResetSection.tsx` (purge list)

**Current Keys:**
```
vibo-onboarding-done = "true" | "false"
vibo-ai-config = JSON{model, integrations, ...}
vibo-agent-notes = "[]"
vibo-tor-enabled = "false"
vibo_theme = "dark" | "light"
```

**Why This Matters:** WKWebView localStorage is scoped by bundle identifier. When bundle ID changed (zettel-spark-flow → com.viboai.app.dev), old localStorage in WebKit/com.vibo.zettel-spark-flow/ origin leaked across. Fresh installs inherited old preferences. Solution: renamed keys + legacy cleanup in reset operations.

**Phase 0.7 Plan:** Move `vibo-onboarding-done` + `vibo-ai-config` from localStorage to file (`viboai/onboarding.json`). Files are keyed by bundle ID in Application Support, not by WebKit origin. Avoids future renames causing drift.

---

## Code Patterns to Reuse

### 1. Tauri Command Wrapper Pattern

**File:** `src/lib/commands.ts`

**Pattern:**
```ts
export async function isVaultConfigured(): Promise<boolean> {
  return tauriInvoke<boolean>("is_vault_configured");
}

export async function resetPassphrase(old: string, newPass: string) {
  return tauriInvoke<void>("reset_passphrase", {
    current: old,
    newPassphrase: newPass,
    caller: { type: "user" }
  });
}

function tauriInvoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  return invoke(cmd, args || {}) as Promise<T>;
}
```

**Benefits:** Typed, consistent error handling, easy to mock/test.

**For Phase 0.7:** Add wrappers for:
- `lockVault()` → calls `lock_vault`
- `unlockVault(passphrase)` → calls `unlock_vault`
- `clearAllNotes()` → calls `clear_all_notes`
- `resetOnboarding(passphrase)` → calls `reset_onboarding`

### 2. Event Listener Pattern

**Frontend:**
```ts
useEffect(() => {
    let dispose = () => {};
    void onVaultStatusChanged((payload: VaultStatus) => {
        setPhase(deriveVaultPhase(payload));
    }).then(unlisten => {
        dispose = () => void unlisten();
    });
    return () => dispose();
}, [flags.encryption_enabled]);
```

**Backend (Rust):**
```rust
#[tauri::command]
pub async fn lock_vault(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    state.security.lock()?;
    emit_vault_status(&app, &state.security, "lock")?;
    Ok(())
}

fn emit_vault_status(app: &AppHandle, security: &SecurityState, reason: &str) {
    let payload = VaultStatusChangedEvent {
        configured: security.is_configured(),
        unlocked: security.is_unlocked(),
        reason: reason.to_string(),
    };
    let _ = app.emit("vault_status_changed", &payload);
}
```

**Pattern:** After state change, backend emits event. Frontend listens, re-derives phase. No callback RPC.

### 3. Factory Reset Pattern

**File:** `src/components/settings/SafeVaultResetSection.tsx:35–75`

```ts
const handleConfirmReset = async () => {
    try {
        await factoryReset();  // Backend: wipes Stronghold, SQL
        
        const KEYS_TO_PURGE = [
            "zettel-notes", "zettel-columns", "zettel-agent-notes",  // Legacy
            "vibo-notes", "vibo-columns", "vibo-agent-notes",  // Current
        ];
        KEYS_TO_PURGE.forEach(key => localStorage.removeItem(key));
        
        window.location.reload();  // Full app restart
    } catch (err) {
        setState({ error: err.message, loading: false });
    }
};
```

**Pattern:** Backend destructive op → purge localStorage → reload (clean state).

### 4. Smart Heuristic Pattern (Onboarding Gate)

**File:** `src/pages/Index.tsx:230–235`

```ts
const hasUserData = snap.folders
    ?.filter(f => !isReservedFolder(f.name))
    .length! > 0 || snap.notes!.length > 0;

const shouldSkipOnboarding = onboardingDone && hasUserData;
```

**Pattern:** Filter reserved folders (auto-created) before heuristic check. Only user-created folders + notes trigger "existing data" detection.

---

## Integration Points for Phase 0.7

### Stronghold Lock/Unlock Activation

**Steps:**
1. **Config flag flip** (1 line):
   - File: `src-tauri/src/config/features.rs:51`
   - Change: `encryption_enabled: false` → `true`

2. **Lock button in Settings** (~5 LoC):
   - File: `src/components/SettingsView.tsx:189` (Encryption card)
   - Code:
     ```tsx
     {flags.encryption_enabled && vaultStatus.unlocked && (
       <Button
         variant="default"
         size="sm"
         onClick={() => lockVault()}
         disabled={isLocking}
       >
         {isLocking ? "Locking..." : "Lock Vault"}
       </Button>
     )}
     ```

3. **Test** (manual E2E):
   ```bash
   bun run tauri:dev
   # Complete onboarding with PIN "0000"
   # Settings → Lock Vault button → enter "0000" → LockScreen appears
   # Try wrong PIN → error
   # Enter "0000" again → unlock → back to app
   ```

**Expected Behavior After Activation:**
- Onboarding Security step: choose PIN or Passphrase
- Finish wizard → app unlocked (session holds Stronghold)
- Settings: "Lock Vault" button appears
- Click Lock → `lock_vault` command → session cleared → phase → "lock"
- LockScreen prompts for PIN/Passphrase → correct entry unlocks → phase → "app"

### Reset Operations UI (Steps 2–6)

**Step 2: Reset Onboarding Card**
- Backend: `reset_onboarding()` command (delete snapshot + clear onboarding file)
- Frontend: `ResetOnboardingSection.tsx` component
- Auth: user enters current passphrase
- Effect: snapshot deleted, wizard re-runs on restart

**Step 3: Delete All Notes Card**
- Backend: `clear_all_notes()` command (wipe myspace/ user folders + SQL notes)
- Frontend: `ClearAllNotesSection.tsx` component
- Auth: user enters passphrase
- Effect: SQL notes cleared; reserved folders (Inbox, Archive) remain

**Step 4: Delete Vault (with Checkbox)**
- Refactor: `SafeVaultResetSection.tsx` — add checkbox "Keep folders & notes on disk (Obsidian compatibility)"
- Backend: extend `factory_reset(keep_vault_files: bool)`
- Two modes:
  - Unchecked: full wipe (current behavior)
  - Checked: wipe SQL + Stronghold only; .md files stay in Finder

**Step 5: Zone Layout**
- Group Cards 1–2 into "Account & Security" section
- Group Cards 3–4 into "Danger Zone" section with red accents
- Pure UI refactoring in `SettingsView.tsx`

**Step 6: Placeholder Card**
- Component: `DeleteFoldersSection.tsx` (disabled stub)
- Label: "Delete Specific Folders — Coming in Phase 0.8"
- Styled consistently with other cards

### Onboarding State Migration (localStorage → File)

**Problem:** localStorage tied to WebKit origin (bundle ID). Renames cause leaks.

**Solution:** Move onboarding state to file in vault dir.

**Implementation:**
1. Backend: Create `src-tauri/src/commands/onboarding.rs`
   ```rust
   #[tauri::command]
   pub async fn read_onboarding(state: State<AppState>) -> Result<Option<OnboardingState>, String> {
       // Read from viboai/onboarding.json
   }

   #[tauri::command]
   pub async fn write_onboarding(state: State<AppState>, config: OnboardingState) -> Result<(), String> {
       // Write to viboai/onboarding.json atomically
   }

   #[tauri::command]
   pub async fn reset_onboarding(state: State<AppState>) -> Result<(), String> {
       // Delete file (used by Reset Onboarding card)
   }
   ```

2. Frontend: Refactor `OnboardingWizard.tsx`
   - Replace `localStorage.getItem("vibo-ai-config")` → `await readOnboarding()`
   - Replace `localStorage.setItem(...)` → `await writeOnboarding(...)`

3. Test: Fresh install → file created; reset → file deleted; restart → wizard re-appears

---

## Testing Strategy

### Unit Tests

**Location:** `src/test/` (use `bun run test`)

- `persistence.test.ts` — FS/SQL sync scenarios
- `task_format.test.ts` — YAML frontmatter parsing
- `wiki_links.test.ts` — link resolution

### E2E Manual Tests (Always Required)

**Setup:** `bun run tauri:dev` (on native Tauri window, not localhost)

**Test 1: Fresh Onboarding (PIN)**
```bash
rm -rf ~/Library/Application\ Support/com.viboai.app.dev
bun run tauri:dev
# → Wizard appears
# → Step 1: choose local model
# → Step 2: skip integrations
# → Step 3: choose PIN, enter "0000"
# → Step 4: enter name, finish
# → App shows Inbox, can create notes
✓ PASS if: .md created in myspace/; SQL has note; can re-lock/unlock
```

**Test 2: Lock/Unlock Cycle**
```bash
# Onboarding complete with PIN "0000"
# Settings → Lock Vault button
# → LockScreen appears
# → Enter "0000" → unlocks → back to app
# → Settings → Lock again
✓ PASS if: phase transitions correctly; LockScreen respects PIN
```

**Test 3: Reset PIN**
```bash
# Settings → Reset Pass/PIN
# Old: "0000", New: "1111", Confirm
# Restart app → LockScreen
# → Enter "0000" → fail
# → Enter "1111" → success
✓ PASS if: old secret invalid, new secret works
```

**Test 4: Kanban DnD (Race Fix Verification)**
```bash
# Create 5 notes in 3 different folders
# Rapidly drag note between folders (last drop = final state)
# Restart app
# Verify: note is in folder of last drop
✓ PASS if: final SQL/FS matches last drop; no race
```

**Test 5: Obsidian Compat**
```bash
# Create note "test" in app
# Finder: ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/
# Open test.md in text editor, add text
# Restart app
# Verify: note shows updated content
✓ PASS if: .md edits sync back to SQL on app restart
```

### Test Matrix (Before Each Release)

| Scenario | PIN? | Passphrase? | Lock Works? | Reset Works? | Notes OK? | Kanban OK? |
|----------|------|-------------|------------|--------------|----------|-----------|
| Fresh onboarding | ✓ | ✓ | ✓* | ✓ | ✓ | ✓ |
| Existing app | ✓ | ✓ | ✓* | ✓ | ✓ | ✓ |
| After reset PIN | ✓ | ✓ | ✓* | ✓ | ✓ | ✓ |

*Lock = depends on encryption_enabled flag; Phase 0 has it OFF (dormant).

---

## Known Gotchas & Debugging

### 1. WebKit localStorage Leaks

**Symptom:** Fresh install doesn't show wizard; remembers old prefs.

**Root Cause:** Old WebKit origin (`~/Library/WebKit/com.vibo.zettel-spark-flow/`) still has `zettel-*` keys.

**Debug:**
```bash
# Check both WebKit dirs
sqlite3 ~/Library/WebKit/com.vibo.zettel-spark-flow/WebsiteData/Default/*/LocalStorage/localstorage.sqlite3 \
  "SELECT key FROM ItemTable;" | grep -E "vibo-|zettel-"
  
sqlite3 ~/Library/WebKit/app/WebsiteData/Default/*/LocalStorage/localstorage.sqlite3 \
  "SELECT key FROM ItemTable;" | grep -E "vibo-|zettel-"

# If keys exist, wipe old dir:
rm -rf ~/Library/WebKit/com.vibo.zettel-spark-flow/
```

### 2. Kanban Race (Pre-debcfcf)

**Symptom:** Drag note to column A, immediately to column B, restart. Note is in A (not B).

**Root Cause:** React setState() + persintNote() not atomic.

**Fix (commit debcfcf):** notesRef eager-compute pattern ensures state + persist happen together.

**Prevention:** When reintroducing note moves, use ref to read current state before dispatch.

### 3. Stronghold Key Derivation Mismatch

**Symptom:** After onboarding, LockScreen rejects correct PIN.

**Root Cause:** Key derivation inconsistent (different hash, encoding).

**Prevention:** `derive_vault_key()` uses SHA-256, 32-byte output. Frontend crypto.ts must match.

**Check:** If adding new auth method, verify key derivation matches backend.

### 4. FS Sync Crashes on YAML Parse

**Symptom:** App panics on startup after user edits .md.

**Root Cause:** User broke YAML syntax (missing quotes, bad indent).

**Current:** Parser fails, sync errors (app doesn't start).

**Phase 1 Fix:** Catch parse error, skip note, warn user.

### 5. Onboarding Gate False Positive

**Symptom:** Fresh install with only reserved folders (auto-created) triggers silent-reuse logic, skipping wizard.

**Root Cause:** Heuristic checked `folders.length > 0` without filtering reserved folders.

**Fixed in Phase 0:** Applied `isReservedFolder()` filter. Now checks user data only.

---

## Key Files & Modules

**Core Vault Logic:**
- `src-tauri/src/security/mod.rs` — Stronghold state machine, lock/unlock
- `src-tauri/src/vault/mod.rs` — Vault initialization, reserved folders
- `src-tauri/src/vault/reconcile.rs` — FS-SQL sync logic
- `src-tauri/src/db/mod.rs` — SQL schema, queries
- `src-tauri/src/lib.rs` — App setup, plugin init, command registration

**Frontend State & Routing:**
- `src/pages/Index.tsx` — Phase router, vault status listener
- `src/components/OnboardingWizard.tsx` — Wizard flow (4 steps)
- `src/components/LockScreen.tsx` — PIN/Passphrase entry
- `src/components/SettingsView.tsx` — Settings UI, reset PIN button (add Lock button here)
- `src/lib/crypto.ts` — `lockVault()`, `unlockVault()`, `setupSecureVault()` wrappers
- `src/lib/commands.ts` — Tauri command wrappers
- `src/lib/store.tsx` — Global state, notes/folders/columns, reserved folder filtering

**Settings & Reset:**
- `src/components/settings/ResetPassSection.tsx` — Reset PIN/Passphrase
- `src/components/settings/SafeVaultResetSection.tsx` — Full vault wipe (refactor for Step 4)
- NEW in Phase 0.7:
  - `src/components/settings/ResetOnboardingSection.tsx` (Step 2)
  - `src/components/settings/ClearAllNotesSection.tsx` (Step 3)
  - `src/components/settings/DeleteFoldersSection.tsx` (Step 6, stub)

---

## Phase 0.7 Activation Checklist

- [ ] **Stronghold Activation (2 hours):**
  - [ ] Flip `encryption_enabled: true` in config (1 line)
  - [ ] Add Lock button to Settings (~5 LoC)
  - [ ] Test E2E: onboarding PIN → lock → unlock
  - [ ] Rebuild + verify + ship

- [ ] **Reset Operations UI (1–2 weeks):**
  - [ ] Step 2: Reset Onboarding card
  - [ ] Step 3: Delete All Notes card
  - [ ] Step 4: Delete Vault with keep-files checkbox
  - [ ] Step 5: Zone layout (Account & Security / Danger Zone)
  - [ ] Step 6: Placeholder card (Delete Folders)
  - [ ] Test each via E2E; verify SQL + FS state

- [ ] **Onboarding State Migration (4–6 hours):**
  - [ ] Create `src-tauri/src/commands/onboarding.rs`
  - [ ] Refactor `OnboardingWizard.tsx` to use Tauri calls
  - [ ] Test: fresh install → file-based state; reset → wizard re-runs

- [ ] **Final Testing & Verification:**
  - [ ] All E2E tests pass (test matrix green)
  - [ ] No `zettel-*` references remain in code
  - [ ] Kanban DnD stable
  - [ ] Obsidian compat verified
  - [ ] Lock/unlock cycles work
  - [ ] Reset operations work
  - [ ] FS-SQL sync works

- [ ] **Documentation & Handoff:**
  - [ ] Guidelines updated
  - [ ] TO_DO.md kept current
  - [ ] Team questions answered
  - [ ] All commits pushed to GitHub

---

## Next Steps & Questions

**Before Phase 0.7 Kickoff:**
1. Is Stronghold activation in scope, or defer to later sprint?
2. How many reset-UI steps per sprint (1 per day? 2?)
3. Is onboarding migration Phase 0.7 or Phase 1?
4. Do we have CI/CD for automated E2E, or manual only?
5. Will bundle ID change again? (Affects localStorage strategy)

**Resources:**
- Detailed backlog: [`Guidelines/current-phase/PHASE_0_7_BACKLOG.md`](Guidelines/current-phase/PHASE_0_7_BACKLOG.md)
- Code architecture deep-dive: [`Guidelines/agents/READY_FOR_HANDOFF.md`](Guidelines/agents/READY_FOR_HANDOFF.md)
- Phase 0 feature completeness: [`Guidelines/source-of-truth/PHASE_0_COMPLETION.md`](Guidelines/source-of-truth/PHASE_0_COMPLETION.md)
- Storage architecture lessons: [`Guidelines/decisions/STORAGE_ARCHITECTURE_UPDATE.md`](Guidelines/decisions/STORAGE_ARCHITECTURE_UPDATE.md)

---

## Summary

Phase 0 has delivered a robust foundation: Stronghold integration, FS-first architecture, onboarding flow, and reset operations. Phase 0.7 is primarily UI additions + one state migration (localStorage → file). No new cryptographic complexity, no new backend systems — just refinement of existing infrastructure.

The team taking over Phase 0.7 can proceed with confidence. Code is clean, architecture is documented, and test procedures are defined. Good luck!

**Questions? Start with `Guidelines/agents/READY_FOR_HANDOFF.md` for deep technical details.**

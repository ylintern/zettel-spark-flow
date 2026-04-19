# Ready for Handoff — Phase 0.7 Implementation Guide

**Audience:** Senior engineer / tech lead picking up Phase 0.7 work  
**Date:** 2026-04-19  
**Scope:** Architecture overview, code patterns, testing strategy, gotchas.

---

## Architecture Map

### Phase Routing (Frontend State Machine)

**File:** `src/pages/Index.tsx:220–251`

```
on mount:
  flags = await getFeatureFlags()
  vault_status = await isVaultConfigured(), isVaultUnlocked()
  onboarding_done = localStorage.getItem("vibo-onboarding-done")
  
  if not flags.encryption_enabled:
    # Phase 0 mode (encryption dormant)
    if onboarding_done: phase = "app"
    else: phase = "onboarding"
  else:
    # Phase 2/3 mode (encryption active)
    if not vault_status.configured: phase = "onboarding"
    else if vault_status.configured && vault_status.unlocked: phase = "app"
    else if vault_status.configured && !vault_status.unlocked: phase = "lock"

listen to vault_status_changed event:
  # When backend emits lock/unlock/setup events
  recalculate phase, re-sync frontend
```

**Key Points:**
- **No explicit navigation:** phase is derived from backend state
- **Events drive sync:** if backend lock_vault() → vault_status_changed → phase="lock"
- **Feature flag gates encryption:** encryption_enabled=false skips lock screen entirely

### Stronghold State Machine (Backend)

**File:** `src-tauri/src/security/mod.rs:40–135`

```rust
pub struct SecurityState {
    snapshot_path: PathBuf,
    session: Mutex<Option<Stronghold>>,  // None = locked; Some = unlocked
}

pub fn is_configured(&self) -> bool {
    self.snapshot_path.exists()  // Snapshot on disk?
}

pub fn is_unlocked(&self) -> bool {
    self.session.lock().unwrap().is_some()  // Session in memory?
}

pub fn lock(&self) -> Result<(), SecurityError> {
    let mut session = self.session.lock().unwrap();
    if let Some(stronghold) = session.take() {
        stronghold.save()?;  // Persist any changes
    }
    // session is now None; is_unlocked() returns false
    Ok(())
}

pub fn unlock(&self, passphrase: &str) -> Result<(), SecurityError> {
    // Derive key from passphrase (SHA-256, 32 bytes)
    let vault_key = derive_vault_key(&passphrase);
    // Load snapshot from disk, validate key
    let stronghold = Stronghold::new(&self.snapshot_path, vault_key)?;
    // Store in session
    *self.session.lock().unwrap() = Some(stronghold);
    Ok(())
}
```

**State:** Locked = `session=None`; Unlocked = `session=Some(Stronghold)`

**Persistence:** Session is in-memory. On app restart, `session=None` (locked). Must unlock via LockScreen.

### FS-First Reconciliation

**Files:** `src-tauri/src/vault/reconcile.rs`, `src/lib/store.tsx:143–180`

```
on app start:
  walk myspace/ → list all .md files
  for each .md:
    parse YAML frontmatter
    read/upsert to SQL `notes` table
  for each SQL note NOT in FS:
    (user deleted from Finder?)
    mark deleted in SQL or warn user

result: SQL mirrors FS
```

**Key:** .md is source of truth. SQL is cache.

**Risk:** If .md YAML corrupt, parser fails. Mitigation: skip + log error (Phase 1).

### Onboarding Flow

**File:** `src/components/OnboardingWizard.tsx:40–700`

```
Step 1: Model Selection
  user picks local model, configures endpoint

Step 2: Integrations
  user enables cloud providers (optional)

Step 3: Security
  user chooses auth method: PIN (4+ digits) or Passphrase (8+ chars)
  stored in OnboardingConfig.authMethod

Step 4: Name & Finish
  user enters name
  onComplete() → handleOnboardingComplete in Index.tsx
  
handleOnboardingComplete:
  if encryption_enabled:
    advance to "lock" phase (LockScreen asks for PIN/Passphrase again)
    LockScreen calls setupSecureVault(secret)
    backend creates Stronghold snapshot
  else:
    advance to "app" phase directly
```

**Auth Method Handling:**
- Frontend stores choice in `OnboardingConfig`
- `LockScreen.tsx:71` reads `authMethod` to determine input type
- Backend doesn't store method; Stronghold just validates passphrase (works for both PIN + passphrase)

---

## Code Patterns to Reuse

### 1. Tauri Command Wrapper Pattern

**File:** `src/lib/commands.ts:1–50`

```ts
import { invoke } from "@tauri-apps/api/core";

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

**Pattern:** Typed wrappers around `invoke()` for type safety + consistency.

**For Phase 0.7:**
- Add `lockVault()`, `unlockVault()` wrappers
- Add `clearAllNotes()`, `resetOnboarding()` wrappers

### 2. Event Listener Pattern

**File:** `src-tauri/src/lib.rs:156–170` (backend), `src/pages/Index.tsx:268–300` (frontend)

**Backend (Rust):**
```rust
#[tauri::command]
pub async fn lock_vault(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    state.security.lock()?;
    emit_vault_status(&app, &state.security, "lock")?;  // ← Event
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

**Frontend (React):**
```ts
useEffect(() => {
    let dispose = () => {};
    void onVaultStatusChanged((payload: VaultStatus) => {
        // payload.unlocked changed → re-derive phase
        setPhase(deriveVaultPhase(payload));
    }).then(unlisten => {
        dispose = () => void unlisten();
    });
    return () => dispose();
}, [flags.encryption_enabled]);
```

**Pattern:** Backend emits after state change; frontend listens and re-syncs. No explicit RPC callback.

### 3. localStorage Key Management

**File:** `src/components/OnboardingWizard.tsx:580–584`

```ts
const ONBOARDING_KEY = "vibo-onboarding-done";
const AI_CONFIG_KEY = "vibo-ai-config";

export function isOnboardingDone(): boolean {
  return localStorage.getItem(ONBOARDING_KEY) === "true";
}

export function getAIConfig(): OnboardingConfig {
  const raw = localStorage.getItem(AI_CONFIG_KEY);
  return raw ? JSON.parse(raw) : DEFAULT_CONFIG;
}
```

**Pattern:** Constants for keys, typed getters/setters, defaults.

**Phase 0.7 Plan:** Replace with async Tauri calls to file-based state.

### 4. Factory Reset Pattern

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

---

## Testing Strategy

### Unit Tests (Git in `src/test/`)

- `persistence.test.ts` — FS/SQL sync scenarios
- `task_format.test.ts` — YAML frontmatter parsing
- `wiki_links.test.ts` — link resolution

**Run:** `bun run test`

### E2E Manual Tests (Always Required)

**Setup:** `bun run tauri:dev`

```bash
# Test 1: Fresh Onboarding (PIN)
rm -rf ~/Library/Application\ Support/com.viboai.app.dev
bun run tauri:dev
# → Wizard appears
# → Choose PIN, enter "0000", finish
# → App shows Inbox, can create notes
✓ PASS if: .md created in myspace/; SQL has note; can re-lock/unlock

# Test 2: Reset PIN
# Settings → Reset Pass → old "0000", new "1111"
# Restart app
# LockScreen: enter "0000" → fail; enter "1111" → success
✓ PASS if: old secret invalid, new secret works

# Test 3: Kanban DnD (race fix verification)
# Create 5 notes in different folders
# Rapidly drag note between 3 folders (last drop = final state)
# Restart app
# Verify: note is in folder of last drop
✓ PASS if: final SQL/FS matches last drop; no race

# Test 4: Obsidian Compat
# Create note "test" in app
# Open Finder: ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/
# Find test.md, open in editor, add text
# Restart app
# Verify: note shows updated content
✓ PASS if: .md edits sync back to SQL on app restart
```

### Test Matrix (Before Each Release)

| Scenario | PIN? | Passphrase? | Lock Works? | Reset Works? | Notes OK? | Kanban OK? |
|----------|------|-------------|------------|--------------|----------|-----------|
| Fresh onboarding | ✓ | ✓ | ✓/✗* | ✓/✓* | ✓ | ✓ |
| Existing app | ✓ | ✓ | ✓/✗* | ✓/✓* | ✓ | ✓ |
| After reset PIN | ✓ | ✓ | ✓/✗* | ✓/✓* | ✓ | ✓ |

*Lock = depends on encryption_enabled flag (Phase 0.7 activation)

---

## Known Gotchas & Debugging

### 1. WebKit localStorage Leaks

**Symptom:** Fresh install doesn't show wizard; still has old prefs.

**Root Cause:** Old WebKit origin dir still has `zettel-*` or `vibo-*` keys.

**Debug:**
```bash
# Check both WebKit dirs
sqlite3 ~/Library/WebKit/com.vibo.zettel-spark-flow/WebsiteData/Default/*/LocalStorage/localstorage.sqlite3 \
  "SELECT key FROM ItemTable;" | grep -E "vibo-|zettel-"
  
sqlite3 ~/Library/WebKit/app/WebsiteData/Default/*/LocalStorage/localstorage.sqlite3 \
  "SELECT key FROM ItemTable;" | grep -E "vibo-|zettel-"

# If keys exist in old dir, wipe it:
rm -rf ~/Library/WebKit/com.vibo.zettel-spark-flow/
rm -rf ~/Library/WebKit/app/
```

### 2. Kanban Race (Pre-debcfcf)

**Symptom:** Drag note to column A, immediately to column B, restart. Note is in A (not B).

**Root Cause:** React setState() + persintNote() not atomic. State updates but persist lags.

**Fixed in debcfcf:** notesRef eager-compute pattern.

**Check:** If reintroducing note moves, use ref to read current state before dispatch.

### 3. Stronghold Key Derivation

**Symptom:** After onboarding, LockScreen rejects correct PIN.

**Root Cause:** Key derivation inconsistent (different hash, encoding).

**Prevention:** `derive_vault_key()` in `src-tauri/src/security/mod.rs:10–42` uses SHA-256, 32-byte output. Frontend `crypto.ts` must match.

**Check:** If adding new auth method, verify key derivation matches backend.

### 4. FS Sync Crashes on YAML Parse

**Symptom:** App panics on startup after user edits .md.

**Root Cause:** User broke YAML syntax (missing quotes, bad indent).

**Current:** Parser fails, sync error (app doesn't start).

**Phase 1 Fix:** Catch parse error, skip note, warn user.

---

## Integration Checklist for Phase 0.7

- [ ] **Stronghold Activation:**
  - [ ] Flip `encryption_enabled: true` in config
  - [ ] Add Lock button to Settings
  - [ ] Test: Onboarding → setup PIN → lock → LockScreen → unlock → app

- [ ] **Reset Operations (Steps 2–6 of reset-UI plan):**
  - [ ] Reset Onboarding card (delete snapshot, re-wizard)
  - [ ] Delete All Notes card (SQL + FS)
  - [ ] Delete Vault refactor (with keep-files checkbox)
  - [ ] Zone layout (Account & Security / Danger Zone)
  - [ ] Placeholder card (Phase 0.8 stub)

- [ ] **Onboarding State Migration:**
  - [ ] Move to file-based (`viboai/onboarding.json`)
  - [ ] Tauri commands: read, write, reset
  - [ ] Frontend: async calls instead of localStorage

- [ ] **Testing:**
  - [ ] All E2E tests pass (see test matrix above)
  - [ ] Kanban DnD stable
  - [ ] Obsidian compat verified
  - [ ] Lock/unlock cycles work

---

## Commands & Shortcuts

```bash
# Build + Run
bun run tauri:dev

# Type-check frontend
bun run build

# Check Rust compilation
cargo check -p app_lib

# Run tests
bun run test

# Inspect SQLite state
sqlite3 ~/Library/Application\ Support/com.viboai.app.dev/viboai/database/vibo.db \
  "SELECT id, title, column FROM notes LIMIT 10;"

# Count notes
sqlite3 ~/Library/Application\ Support/com.viboai.app.dev/viboai/database/vibo.db \
  "SELECT COUNT(*) FROM notes;"

# List myspace contents
ls ~/Library/Application\ Support/com.viboai.app.dev/viboai/myspace/
```

---

## Questions Before Starting Phase 0.7?

**Review and clarify:**
1. **Stronghold Activation Scope:** Just Lock button + flag flip, or full lock/unlock flow test?
2. **Reset Operations Order:** Sequence of Steps 2–6? Any dependencies?
3. **Onboarding Migration Timing:** Phase 0.7 or Phase 1?
4. **CI/CD Integration:** Do we have automated E2E tests, or manual only?

See [`PHASE_0_7_BACKLOG.md`](../current-phase/PHASE_0_7_BACKLOG.md) for detailed backlog prioritization.

# Architecture Rules Rsearch/draft

## Permanent Rules
- 2 flows:
  - user flow
  - agent flow
- cross-device first:
  - desktop
  - iOS
  - Android
- no sidecars
- adequated flow only:
  - UI
  - Tauri commands
  - Rust services
  - tools / inference
- background or agent work must be able to push state back to UI through an explicit event bus? tauri leap ai plugin has api or commands that cover this?

## Rust Structure Rule
- create folders under `src-tauri/src/` by real concern
- examples:
  - `commands/`
  - `tools/`
  - `mcp/`
  - `agents/`
  - `plugins/`

## Memory Rule
- app memory and agent memory must not be tangled casually
- Phase 0 builds durable app memory first:
  - markdown notes
  - SQL metadata /velesdb ?
  - secure secrets
- private note plaintext must not be the canonical disk form
- browser crypto must not be the long-term owner of private notes or provider keys
- placeholder biometric authorization must not be presented as secure vault release
- later phases may consume this memory
- later phases should not redefine it

## Caller Rule
- user-triggered commands and agent-triggered actions must be distinguishable
- when agent flow mutates user data, record audit context

## Decision Rule
- short memo after decisions, audits, smoke tests, or scripts
- keep memo in the relevant category folder
- update `current-phase` and `source-of-truth` at the end of every execution

## UX Contract Rule
- onboarding choices that affect security or storage must match the first real runtime flow
- do not let UI-only options drift ahead of the implemented lock/security path
- device-specific security options must come from backend capability truth, not frontend assumptions
- app root phase must derive from backend vault truth only:
  - `configured=false` -> onboarding
  - `configured=true && unlocked=false` -> lock
  - `configured=true && unlocked=true` -> app

## Bootstrap Rule
- app-owned storage directories should self-heal on launch
- bootstrap should be idempotent
- missing working directories are recreated silently and logged

---

## Interaction Architecture — Phase 1+

Every feature shipped must follow exactly one row from this table.

| Trigger | Who initiates | Frontend call | Tauri layer | Plugin / Service | Rust backend | Mobile ✅ |
|---------|--------------|---------------|-------------|------------------|--------------|-----------|
| Read data | Human | `invoke('get_notes')` | `#[tauri::command]` | `tauri-plugin-sql` | `db::query_notes()` | ✅ |
| Write data | Human | `invoke('save_note', {…, caller:{type:"user"}})` | `#[tauri::command]` | `tauri-plugin-fs` + sql | `vault::write_note()` | ✅ |
| Unlock app | Human | `invoke('unlock_vault', { passphrase })` | `#[tauri::command]` | `tauri-plugin-stronghold` | `security::unlock()` | ✅ |
| Reset pass/PIN | Human | `invoke('reset_passphrase', {current, new, caller})` | `#[tauri::command]` | `tauri-plugin-stronghold` | `security::reset_passphrase()` | ✅ |
| Biometric unlock | Human (mobile) | `invoke('verify_biometric_and_unlock')` | `#[tauri::command]` | `tauri-plugin-biometric` | `security::biometric_unlock()` | ✅ Mobile only |
| Store API key | Human | `invoke('store_secret', {key, value, caller})` | `#[tauri::command]` | `tauri-plugin-stronghold` | `secrets::store()` | ✅ |
| Get provider status | Human | `invoke('get_provider_status', { provider })` | `#[tauri::command]` | `tauri-plugin-stronghold` | returns bool only | ✅ |
| Device capabilities | App mount | `invoke('get_device_capabilities')` | `#[tauri::command]` | — | `biometric::capabilities()` | ✅ |
| Rust → UI event | Rust backend | — (no invoke) | `app.emit()` | `tauri-plugin-log` | `events::emit_*()` | ✅ |
| Export notes | Human | `invoke('export_notes', {caller})` | `#[tauri::command]` | `tauri-plugin-fs` + dialog | `export::to_json()` | ✅ |

**Hard rules from this table:**
- Human clicks → `invoke()` → `#[tauri::command]` — always, no exceptions
- Plugins are called from Rust only — never from TypeScript directly
- Events flow Rust → UI only — reverse direction always uses `invoke()`
- Secrets never pass through frontend state — Stronghold only
- Every write command carries `CallerContext` — `User` for human, `Agent { id }` for agent (Phase 1-B)
- All paths must work on mobile — desktop-only APIs need a fallback

---

## Device-Aware Auth — Permanent Rule

| Device | Auth method | Plugin | Status |
|--------|------------|--------|--------|
| Desktop (macOS/Windows/Linux) | Passphrase or PIN | `tauri-plugin-stronghold` | ✅ Working |
| Mobile (iOS/Android) | Biometrics + PIN fallback | `tauri-plugin-biometric` | Commands exist, needs mobile test |

**Auth scope now:** locks/unlocks the app only. No note content decryption.
**Auth scope later (Phase 2):** decrypt private notes on demand, unlock API keys from Stronghold.
**Do not couple these two concerns. They ship in different phases.**

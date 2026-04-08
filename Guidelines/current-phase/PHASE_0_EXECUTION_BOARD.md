# Phase 0 Execution Board

**Status**: `[~]` SIGN-OFF TAIL — BUILD FIXED, CORE GATE PASSED, QA GAPS REMAIN  
**Last Updated**: 2026-04-08

---

## Objective

Make the app fully functional before inference:
- Connect Rust/Tauri to the existing frontend without breaking roadmap direction
- Replace browser-only persistence with durable app-owned persistence
- Secure vault with hardware-backed unlock path

---

## Gate Status

| Gate | Status | Evidence |
|------|--------|----------|
| Build compiles (dev) | `[x]` | `bun run tauri:dev` — fixed 2026-04-08 after stale cache clear |
| White screen regression | `[x]` | `store.tsx` import fix confirmed working |
| SHA-256 key derivation | `[x]` | `derive_vault_key()` in `security/mod.rs`, confirmed by user |
| Core App QA (8 checks) | `[x]` | `scripts/qa_lifecycle.py` — 8/8 PASS, 2026-04-08 |
| Folder bootstrap | `[x]` | PASS — vault/notes and vault/kanban self-heal on launch |
| Reset routing | `[ ]` | Code done, manual QA not yet run |
| Normal unlock regression | `[ ]` | Required after key derivation changes |
| Note + kanban persistence | `[ ]` | End-to-end create/edit/relaunch not confirmed |
| Private note encrypt UX | `[ ]` | Storage path works; UX contract incomplete |
| Biometrics (hardware-backed) | `[!]` | Explicitly deferred — placeholder only |
| DMG packaging | `[!]` | .app bundle works, .dmg fails — explicitly deferred |

---

## What Is Confirmed Working

- `[x]` Dev and release use separate data directories (`com.viboai.app.dev` / `com.viboai.app`)
- `[x]` Crypto crates optimized in debug mode (vault setup ~5s not 80s)
- `[x]` Stronghold-backed passphrase with canonical SHA-256 derivation
- `[x]` Vault phase derives from backend truth: `configured` + `unlocked` → onboarding / lock / app
- `[x]` Factory reset clears Stronghold, SQLite, Markdown vault
- `[x]` Folder bootstrap is idempotent and self-healing on every launch
- `[x]` Kanban columns in SQLite (removed from localStorage)
- `[x]` Folders backend-authoritative (removed from localStorage)
- `[x]` Event bus module exists — `vault_status_changed` and `note_indexing_progress` wired
- `[x]` Device capabilities exposed to frontend (biometric availability from Rust)

---

## Open QA — Required Before Phase 0 Full Sign-Off

| # | Test | How |
|---|------|-----|
| 1 | Reset routing → onboarding | Settings → Reset → confirm → expect onboarding, not lock screen |
| 2 | Normal unlock regression | Close app, relaunch, enter correct passphrase → workspace |
| 3 | Wrong passphrase stays on lock | Enter wrong PIN/passphrase → error, no phase change |
| 4 | Note persistence after relaunch | Create note → close → relaunch → note still there |
| 5 | Kanban task persistence | Create task → close → relaunch → task still there |
| 6 | Private note toggle UX | Toggle encrypt on a note → confirm passphrase required |
| 7 | Duplicate reset UX | Confirm only one reset entrypoint visible in Settings |

---

## What Is NOT Phase 0

- Biometric hardware-backed unlock (deferred)
- DMG packaging fix (deferred)
- Agent notes migration from localStorage (decision pending)
- Inference / model manager / LEAP / Swiftide (Phase 1+)

---

## Handoff Gate To Phase 1

- Phase 1 infrastructure work is **allowed to start now** (event bus, caller-aware commands)
- Phase 0 is **not fully closed** until QA items 1–7 above pass
- Do not claim Phase 0 signed-off until those items have audit entries

---

## Storage Authority (Locked)

| Data | Owner | Location |
|------|-------|----------|
| Note content (markdown) | Vault module (Rust) | `vault/notes/` |
| Note metadata | DB module (Rust) | SQLite `notes` table |
| Kanban columns | DB module (Rust) | SQLite `columns` table |
| Folders | DB module (Rust) | SQLite `folders` table |
| Encryption keys, vault state | Security module (Rust) | Stronghold `secure-vault.hold` |
| Vectors / embeddings | Swiftide pipeline | Phase 3, not yet |
| Agent memory | Agent module | Phase 4, not yet |

**Invariant**: No app-critical data in browser localStorage.

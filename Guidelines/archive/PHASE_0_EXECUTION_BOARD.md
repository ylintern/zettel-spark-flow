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
| Build compiles (dev) | `[x]` | `bun run tauri:dev` — 39.87s, 14 dead_code warnings (cosmetic) |
| White screen regression | `[x]` | `store.tsx` import fix confirmed working |
| SHA-256 key derivation | `[x]` | `derive_vault_key()` in `security/mod.rs`, confirmed by user |
| Core App QA (8 checks) | `[x]` | `scripts/qa_lifecycle.py` — 8/8 PASS, 2026-04-08 |
| Folder bootstrap | `[x]` | PASS — notes and kanban self-heal on launch |
| Normal unlock regression | `[x]` | PASS — confirmed by Lead 2026-04-09 (lock/unlock/wrong-pass) |
| Reset routing | `[ ]` | Code done, manual QA not yet run |
| Note + task persistence | `[ ]` | Test artifacts created — persistence script pending |
| Private note encrypt UX | `[!]` | DEFERRED — moved to Phase 1 |
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
| 2 | ~~Normal unlock regression~~ | ✅ PASS 2026-04-09 |
| 3 | ~~Wrong passphrase stays on lock~~ | ✅ PASS 2026-04-09 |
| 4 | Note persistence after relaunch | Run `python3 scripts/qa_p0_tail.py --dev` after relaunch |
| 5 | Task persistence after relaunch | Same script — checks "Countertest69" in `database/tasks/` |
| 6 | ~~Private note toggle UX~~ | 🚫 DEFERRED Phase 1 |
| 7 | ~~Duplicate reset UX~~ | ✅ RESOLVED — two resets are intentional by design |

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
| Note content (markdown) | Vault module (Rust) | `database/notes/{id}.md` |
| Task content (markdown) | Vault module (Rust) | `database/tasks/{id}.md` |
| Note/task metadata | DB module (Rust) | SQLite `notes` table (`kind='note'` or `kind='task'`) |
| Kanban columns | DB module (Rust) | SQLite `columns` table |
| Folders | DB module (Rust) | SQLite `folders` table |
| Encryption keys, vault state | Security module (Rust) | Stronghold `secure-vault.hold` |
| Vectors / embeddings | Swiftide pipeline | Phase 3, not yet |
| Agent memory | Agent module | Phase 4, not yet |

**Invariant**: No app-critical data in browser localStorage.

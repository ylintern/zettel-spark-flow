# Engineer Sync

**Last Updated**: 2026-04-08  
**Status**: Build fixed. Core gate passed. QA tail open.

---

## What Is Working Now

- Dev and release builds use separate data dirs (`com.viboai.app.dev` / `com.viboai.app`)
- Crypto crates optimized in debug mode — vault setup ~5s (was 80s+)
- Vault phase derives from backend truth: `configured` + `unlocked` → onboarding / lock / app
- SHA-256 canonical key derivation in `security/mod.rs` — confirmed working
- Stronghold-backed passphrase/PIN setup and unlock
- Factory reset clears Stronghold + SQLite + Markdown vault
- Folder bootstrap self-heals `vault/notes/` and `vault/kanban/` on every launch
- Kanban columns and folders are backend-authoritative (SQLite, no localStorage)
- Event bus exists: `vault_status_changed` and `note_indexing_progress` wired end-to-end
- Device capabilities exposed to frontend from Rust
- `scripts/qa_lifecycle.py` — 8/8 checks pass

---

## What Is Still Open

| Item | Status | Blocker? |
|------|--------|----------|
| Reset routing → onboarding | `[ ]` QA pending | Yes — Phase 0 close |
| Normal unlock regression | `[ ]` QA pending | Yes — Phase 0 close |
| Note persistence after relaunch | `[ ]` QA pending | Yes — Phase 0 close |
| Kanban persistence after relaunch | `[ ]` QA pending | Yes — Phase 0 close |
| Private note encrypt/decrypt UX | `[ ]` QA pending | Yes — Phase 0 close |
| Duplicate reset UX (Settings) | `[ ]` Review pending | Minor |
| `agent_thinking_delta` event wired | `[ ]` Phase 1 | No |
| Caller-aware command boundary | `[ ]` Phase 1 | No |
| React Error Boundary on Index.tsx | `[ ]` Hygiene | No |
| tsconfig strict mode | `[ ]` Hygiene | No |
| Agent notes localStorage → SQLite | `[ ]` Decision pending | No |
| Biometrics hardware-backed | `[!]` Deferred | No |

---

## Storage Authority

| Data | Owner | Location |
|------|-------|----------|
| Note content (markdown) | Vault module (Rust) | `vault/notes/` |
| Note metadata | DB module (Rust) | SQLite `notes` table |
| Kanban columns | DB module (Rust) | SQLite `columns` table |
| Folders | DB module (Rust) | SQLite `folders` table |
| Vault keys / secrets | Security module (Rust) | Stronghold `secure-vault.hold` |
| Vectors / embeddings | Swiftide pipeline | Phase 3 — not yet |
| Agent memory | Agent module | Phase 4 — not yet |

**Invariant**: No app-critical data in browser localStorage.

---

## Key Files

| File | Role |
|------|------|
| `src-tauri/src/security/mod.rs` | Stronghold wrapper, `derive_vault_key()`, setup/unlock/lock |
| `src-tauri/src/vault/mod.rs` | Note read/write, encryption |
| `src-tauri/src/db/mod.rs` | SQLite migrations, all queries |
| `src-tauri/src/commands/workspace.rs` | Tauri IPC endpoints |
| `src-tauri/src/events/mod.rs` | Event bus definitions |
| `src-tauri/src/lib.rs` | App setup, bootstrap, plugin registration |
| `src/lib/vaultPhase.ts` | Single source of truth for phase derivation |
| `src/lib/store.tsx` | Frontend state, hydration from backend |
| `src/pages/Index.tsx` | Root — phase routing, vault_status_changed listener |
| `scripts/qa_lifecycle.py` | 8-check QA lifecycle inspector |
| `src-tauri/tauri.dev.conf.json` | Dev identifier override (`com.viboai.app.dev`) |

---

## Active Rules

- Private notes encrypt in Rust before disk write
- Cloud/API secrets use secure vault commands only
- Biometric implementation is untrusted until hardware-backed release exists
- `Cargo.toml` dependency added → module must be `use`d → command registered in `lib.rs`
- Phase derivation computed only by `deriveVaultPhase()` in `vaultPhase.ts`
- `swiftide`, `swiftide-agents`, `tauri-plugin-velesdb`, `tauri-plugin-leap-ai` commented out until Phase 1–3 modules are written

---

## Next Recommended Order

1. Run Phase 0 QA tail (tests 2–7 in `PHASE_0_EXECUTION_BOARD.md`)
2. Once QA passes, proceed to P1-B caller-aware command boundary
3. Then P1-C context bundle service
4. Then P1-D LEAP runtime hookup
5. Do not start model manager or Swiftide until app-core gate is fully passed

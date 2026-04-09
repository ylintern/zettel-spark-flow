# Engineer Sync

**Last Updated**: 2026-04-09
**Status**: Phase 0 closed. Settings sprint (S1–S5) complete. P1-B CallerContext complete. P1-D LEAP next.

---

## What Is Working Now

- Dev and release builds use separate data dirs (`com.viboai.app.dev` / `com.viboai.app`)
- Crypto crates optimized in debug mode — vault setup ~5s (was 80s+)
- Vault phase derives from backend truth: `configured` + `unlocked` → onboarding / lock / app
- SHA-256 canonical key derivation in `security/mod.rs` — confirmed working
- Stronghold-backed passphrase/PIN setup, unlock, reset (`reset_passphrase` ✅)
- Factory reset clears Stronghold + SQLite + Markdown vault
- Folder bootstrap self-heals `database/notes/` and `database/tasks/` on every launch
- Path migration complete: `vault/` → `database/`, `kanban/` → `tasks/` — 8 files migrated, zero data loss
- Kanban columns and folders are backend-authoritative (SQLite, no localStorage)
- Event bus: `vault_status_changed` and `note_indexing_progress` wired end-to-end
- Device capabilities exposed to frontend via `get_device_capabilities`
- Biometric commands registered (mobile path, commands exist — hardware test pending)
- Stronghold secret commands: `store_secret`, `get_secret`, `delete_secret`, `get_provider_status` ✅
- Factory reset: `factory_reset`, `reset_vault_and_secrets` — functional
- CallerContext on all 5 write commands — `save_note`, `delete_note`, `create_folder`, `save_column`, `delete_column` ✅
- Agent mutations log `[AGENT MUTATION] agent_id=X command=Y` ✅
- Cloud providers: Anthropic / Ollama (Cloud) / Local visible; OpenRouter/Gemini/Kimi/MiniMax hidden ✅
- `get_provider_status` returns bool only — secrets never in frontend state ✅
- Appearance toggle — `vibo_theme` persists across relaunch ✅
- Export notes: `export_notes` Rust command + tauri-plugin-dialog save dialog ✅
- Biometrics section reads from `is_biometric_available` on mount — not hardcoded ✅
- Cloud key loads: parallel `Promise.all` (not sequential) ✅
- `scripts/qa_p0_tail.py` — 6/6 checks pass

---

## What Is Still Open — Phase 1

| Item | Task | Status | Blocker? |
|------|------|--------|----------|
| agent_thinking_delta real emission | P1-D | `[ ]` Needs LEAP wired | Yes |
| LEAP pre-flight (5 questions) | P1-D | `[ ]` Answer before code | Yes |
| tauri-plugin-leap-ai registration | P1-D | `[ ]` Cargo.toml + capabilities + lib.rs | Yes |
| Model download UI (catalog-driven) | P1-D | `[ ]` After plugin registered | After pre-flight |
| Model load + createConversation | P1-D | `[ ]` | After download UI |
| generate → onLeapEvent → UI stream | P1-D | `[ ]` Gate for P1-C | Core P1-D |
| build_context Rust command | P1-C | `[ ]` retrieval.rs hot path | After P1-D stream |
| Cloud inference via tauri-plugin-http | P1-cloud | `[ ]` Anthropic + Ollama + Local | After P1-C |
| Swiftide agent + #[tool] functions | P1-tools | `[ ]` | After P1-cloud |
| Biometrics hardware-backed | Phase 2 | `[ ]` Deferred | — |
| Private note encryption | Phase 2 | `[ ]` Deferred | — |

---

## Storage Authority

| Data | Owner | Location |
|------|-------|----------|
| Note content (markdown) | Vault module (Rust) | `database/notes/{id}.md` |
| Task content (markdown) | Vault module (Rust) | `database/tasks/{id}.md` |
| Note/task metadata | DB module (Rust) | SQLite `notes` table (`kind='note' or 'task'`) |
| Kanban columns | DB module (Rust) | SQLite `columns` table |
| Folders | DB module (Rust) | SQLite `folders` table |
| Vault keys / secrets / API keys | Security module (Rust) | Stronghold `secure-vault.hold` |
| Vectors / embeddings | Swiftide pipeline | Phase 3 — not yet |
| Agent memory | Agent module | Phase 4 — not yet |

**Invariant**: No app-critical data in browser localStorage.
**Allowed localStorage**: `vibo_theme` (appearance only).

---

## Key Files

| File | Role |
|------|------|
| `src-tauri/src/security/mod.rs` | Stronghold wrapper, derive_vault_key, setup/unlock/lock/reset/get_provider_status |
| `src-tauri/src/vault/mod.rs` | Note read/write, task routing (database/notes vs database/tasks) |
| `src-tauri/src/db/mod.rs` | SQLite migrations, all queries |
| `src-tauri/src/commands/workspace.rs` | Tauri IPC: save_note, delete_note, create_folder, save_column, delete_column, export_notes |
| `src-tauri/src/events/mod.rs` | Event bus definitions |
| `src-tauri/src/lib.rs` | App setup, bootstrap, plugin registration, invoke_handler |
| `src-tauri/src/models/mod.rs` | CallerContext type |
| `src/lib/vaultPhase.ts` | Single source of truth for phase derivation |
| `src/lib/store.tsx` | Frontend state, hydration from backend |
| `src/lib/models.ts` | CLOUD_PROVIDERS, cloudProviderSecretKey, loadCloudKeys (parallel) |
| `src/pages/Index.tsx` | Root — phase routing, vault_status_changed listener, vibo_theme on mount |
| `src/components/SettingsView.tsx` | Settings: passphrase reset, appearance, export, cloud providers |
| `Guidelines/source-of-truth/PHASE_1_INFERENCE_PLAN.md` | Source of truth for P1-D through Phase 2 |

---

## Active Rules

- Private notes: encrypt in Rust before disk write (Phase 2)
- Cloud/API secrets: Stronghold only — never frontend state
- Biometric: untrusted until hardware-backed release path exists
- `Cargo.toml` dependency added → module must be `use`d → command registered in `lib.rs`
- Phase derivation: computed only by `deriveVaultPhase()` in `vaultPhase.ts`
- Tauri commands: Human clicks → `invoke()` only — never LLM-initiated
- Swiftide `#[tool]`: LLM-initiated only — never Human-triggered
- `swiftide`, `swiftide-agents`, `tauri-plugin-velesdb`, `tauri-plugin-leap-ai` commented out until Phase 1 modules written
- 14 dead-code warnings are intentional phase indicators (retrieval.rs, manifest.rs, manager.rs) — do not silence

---

## Next Recommended Order

1. Answer LEAP pre-flight 5 questions (`PHASE_1_INFERENCE_PLAN.md` Section 3)
2. P1-D: Register LEAP plugin, update Cargo.toml platform split, update capabilities
3. P1-D: Model download UI → load → createConversation → generate → stream → UI
4. P1-C: build_context command (retrieval.rs hot path, FTS first, velesdb decision at gate)
5. P1-cloud: Cloud providers via tauri-plugin-http
6. P1-tools: Swiftide agent + #[tool] functions
7. Do not build InferenceProvider trait before step 3 is complete

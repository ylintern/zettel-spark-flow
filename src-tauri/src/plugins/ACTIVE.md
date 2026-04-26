# Active Plugins

Single index of every plugin / native crate currently linked into Vibo. **Last reviewed 2026-04-26.** Source of truth for status: `src-tauri/Cargo.toml` + `src-tauri/src/lib.rs` Builder calls.

| Plugin / crate | Crate | Version | Scope | Doc |
|---|---|---|---|---|
| Logging | `tauri-plugin-log` | 2.0.0-rc | cross-platform | [`active/log.md`](active/log.md) |
| SQLite | `tauri-plugin-sql` | 2.0.0-rc (`sqlite` feat) | cross-platform | [`active/sql.md`](active/sql.md) |
| Filesystem | `tauri-plugin-fs` | 2 | cross-platform | [`active/fs.md`](active/fs.md) |
| Stronghold (vault) | `tauri-plugin-stronghold` | 2.0.0-rc | cross-platform | [`active/stronghold.md`](active/stronghold.md) |
| Native dialogs | `tauri-plugin-dialog` | 2.0.0-rc | cross-platform | [`active/dialog.md`](active/dialog.md) |
| OS info | `tauri-plugin-os` | 2 | cross-platform | [`active/os.md`](active/os.md) |
| Clipboard | `tauri-plugin-clipboard-manager` | 2.0.0-beta.0 | cross-platform | [`active/clipboard.md`](active/clipboard.md) |
| Auto-launch | `tauri-plugin-autostart` | 2 | desktop-only | [`active/autostart.md`](active/autostart.md) |
| Global shortcuts | `tauri-plugin-global-shortcut` | 2.0.0-rc | desktop-only | [`active/shortcuts.md`](active/shortcuts.md) |
| **Local inference** | `tauri-plugin-leap-ai` | 0.1.1 (`desktop-embedded-llama` feat) | desktop-only | [`active/leap-ai.md`](active/leap-ai.md) |
| **GGUF runtime** | `llama-cpp-2` (rust-crate) | 0.1 | desktop-only | [`active/llama-cpp.md`](active/llama-cpp.md) |
| Haptics | `tauri-plugin-haptics` | 2.0.0-rc | mobile-only | [`active/haptics.md`](active/haptics.md) |

**Total: 10 Tauri plugins + 1 direct Rust crate (`llama-cpp-2`).**

## Plugin-shaped Rust crates also linked

These are *not* Tauri plugins but ship in the same binary with their own surface:

| Crate | Version | Purpose |
|---|---|---|
| `iota_stronghold` | 2.1 | Underlying engine for `tauri-plugin-stronghold` |
| `aes-gcm` | 0.10 | Per-note AES-256-GCM (encrypted note bodies) |
| `sqlx` | 0.8 (sqlite + tokio) | SQL access used by `tauri-plugin-sql` |
| `reqwest` | 0.12 (rustls) | Cloud-LLM streaming in `providers::stream_cloud_message` |
| `minijinja` | 2 | Prompt templates for the agent loop (planned) |
| `walkdir` | 2 | Vault reconciliation (`vault/reconcile.rs`) |

## Wiring

All plugins are initialized in `src-tauri/src/lib.rs`:

- Cross-platform plugins are added unconditionally inside the Builder chain.
- Desktop-only plugins are gated by `cfg(not(any(target_os = "android", target_os = "ios")))` blocks in `Cargo.toml` and `cfg(desktop)` in `lib.rs`.
- Mobile-only plugins are gated by `cfg(any(target_os = "android", target_os = "ios"))` in `Cargo.toml` and `cfg(mobile)` in `lib.rs`.
- `tauri-plugin-log` is debug-only via `debug_assertions`.

Permissions live in `src-tauri/capabilities/default.json` — current grant: `core:default`, `sql:default`, `fs:default`, `stronghold:default`, `clipboard-manager:default`, `dialog:default`, `os:default`, `log:default` (plus `leap-ai:*` allow-list documented in [`active/leap-ai.md`](active/leap-ai.md)).

## Recently activated

- **2026-04-24:** `tauri-plugin-stronghold` moved from compiled-but-dormant → ACTIVELY USED (encryption flag flipped; LockScreen live).
- **2026-04-24:** `tauri-plugin-leap-ai` moved from `planned/` → `active/` (was previously commented out due to a Tauri 2.0.0-rc.* / 2.10.x mismatch — now resolved).
- **2026-04-25:** `llama-cpp-2` direct linkage added (commit `61447f3`).

---
*See individual files in [`active/`](active/) for per-plugin API + command details.*

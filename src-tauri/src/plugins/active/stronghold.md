---
plugin: tauri-plugin-stronghold
version: 2.0.0-rc
scope: cross-platform
status: active
since: 2026-04-24
phase: 0.7-A
---

# tauri-plugin-stronghold

Secure secret storage plugin using IOTA Stronghold.

> **Activation note (2026-04-24):** Plugin moved from compiled-but-dormant → ACTIVELY USED. `encryption_enabled: true` flipped in `src-tauri/src/config/features.rs`. LockScreen wired. Reload-lock via `Builder::on_page_load` (every WebView reload drops the session, forcing the user back to the lock screen). See `Guidelines/source-of-truth/PHASE_0_COMPLETION.md` "2026-04-24 Update".
>
> **Snapshot path:** `<app_data_dir>/viboai/database/secure-vault.hold`. Key derivation: SHA-256 → 32-byte vault key (`security::derive_vault_key`). Session lifecycle: `lock = drop session`, `unlock = load snapshot`, `reload = lock`. Fix C (2026-04-24) sweeps `secure-vault.hold.PARTI` orphans on every successful `setup()`.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-stronghold |
| Version | 2.0.0-rc |
| Status | ACTIVE (since 2026-04-24) |

## Purpose

Provides secure encrypted storage for sensitive data (API keys, passwords, secrets). Uses IOTA Stronghold for cryptographic protection.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (lines 105-107)
.plugin(
    tauri_plugin_stronghold::Builder::new(|password| derive_vault_key(&password)).build(),
)
```

**Initialization:**
- Custom password hash function via `derive_vault_key`
- Universal plugin (all platforms)
- Handles vault unlock/lock

## Platform Support

- macOS: Yes
- Windows: Yes
- Linux: Yes
- iOS: Yes
- Android: Yes

## Dependencies

```toml
# Cargo.toml (line 41)
tauri-plugin-stronghold = "2.0.0-rc"

# Also (line 29)
iota_stronghold = "2.1"
```

## Permissions

```json
// capabilities/default.json (line 12)
"stronghold:default"
```

## Vault Path

```rust
// src-tauri/src/lib.rs (line 138)
let secure_vault_path = app_data_dir.join("secure-vault.hold");
```

## Usage Notes

- Custom key derivation function required
- 32-byte password hash requirement
- Used for secure credential storage
- Integrates with custom security module

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition
- `src-tauri/src/security/` - Security module
- `src-tauri/src/vault.rs` - Vault module

---
*Last updated: 2026-04-14*

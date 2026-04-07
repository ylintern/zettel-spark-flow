# P0-05 Stronghold Memo

## Decision
- Stronghold is now the secure backend owner for:
  - vault unlock state
  - cloud/API secrets
  - private note encryption key

## Implementation
- Rust security module:
  - `src-tauri/src/security/mod.rs`
- commands added:
  - `setup_secure_vault`
  - `unlock_vault`
  - `lock_vault`
  - `is_vault_unlocked`
  - `is_vault_configured`
  - `store_secret`
  - `get_secret`
  - `delete_secret`

## Private Notes Rule
- frontend sends private note content in plaintext
- Rust encrypts before writing `.md`
- if vault is locked:
  - backend returns `Vault locked`
  - UI goes back to lock flow

## Smoke
- `bun run build`
- `cargo check`

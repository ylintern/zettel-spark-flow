# P0 + P1 Memo

## Date
- 2026-04-07

## Scope
- P0 hardening
- P1 event bus start

## Done
- Rust cold-start persistence test added and passing
- factory reset now clears:
  - Stronghold snapshot
  - SQLite db + wal/shm
  - Markdown vault folder
- event module created in Rust
- frontend event subscribe bridge added
- `vault_status_changed` now has a real UI consumer
- security commands fixed to use managed `AppState`

## Re-audit Result
- biometric unlock was not hardware-backed
- previous mobile biometric flow was a placeholder
- hardening decision:
  - do not pretend biometric release is secure
  - keep passphrase as canonical unlock path
  - only enable biometric unlock when secure hardware release is real

## Verified
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `bun run build`

## Not Closed Yet
- packaged Mac app cold-start validation still missing
- mobile biometric secure enclave / keychain / keystore path still missing
- `note_indexing_progress` and `agent_thinking_delta` are defined but not yet producing real payloads

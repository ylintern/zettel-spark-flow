# Engineer Sync

## What Changed
- cold-start persistence test now exists in Rust and passes
- factory reset now clears:
  - Stronghold snapshot
  - SQLite db + wal/shm
  - Markdown vault
- event bus module now exists
- `vault_status_changed` has a real frontend subscription path
- security commands now use managed `AppState` instead of orphan state types

## Current Boundary
- private notes encrypt in Rust before disk write
- cloud/API secrets use secure vault commands
- biometric implementation is intentionally treated as untrusted until a hardware-backed release path exists
- tests pass, but packaged Mac app runtime proof is still missing
- legacy browser-encrypted private notes are not migrated

## Files Touched This Pass
- `src-tauri/src/state.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/events/mod.rs`
- `src-tauri/src/security/mod.rs`
- `src-tauri/src/security/biometric.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/vault/mod.rs`
- `src-tauri/src/commands/workspace.rs`
- `src/lib/commands.ts`
- `src/pages/Index.tsx`
- `src/components/settings/SafeVaultResetSection.tsx`
- `src/components/settings/BiometricsSection.tsx`

## Why This Matters
- frontend and backend are finally close enough to connect inference without rewriting storage
- but the next blocker is still not models, it is integration infrastructure
- streaming and agent state now have an event path to build on, but only vault status is wired end-to-end so far

## Next Recommended Slice
- caller-aware command boundary
- context bundle service
- LEAP runtime bridge
- real producers for:
  - `note_indexing_progress`
  - `agent_thinking_delta`

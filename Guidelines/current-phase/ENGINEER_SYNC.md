# Engineer Sync

## What Changed
- app launch now bootstraps missing vault subdirectories idempotently:
  - `vault/`
  - `vault/notes/`
  - `vault/kanban/`
- root app phase now derives from backend vault truth through `src/lib/vaultPhase.ts`
- frontend now handles all 3 vault states correctly:
  - `configured=false` -> onboarding
  - `configured=true && unlocked=false` -> lock
  - `configured=true && unlocked=true` -> app
- factory reset routing now clears root secure session state and returns to onboarding via `vault_status_changed`
- vault helpers now recreate both `notes` and `kanban` directories after destructive reset paths
- cold-start persistence test now exists in Rust and passes
- factory reset now clears:
  - Stronghold snapshot
  - SQLite db + wal/shm
  - Markdown vault
- event bus module now exists
- `vault_status_changed` has a real frontend subscription path
- security commands now use managed `AppState` instead of orphan state types
- lock/setup flow now respects `authMethod` chosen during onboarding
- setup/unlock errors now surface in the UI instead of failing silently
- Rust now exposes device capabilities to the frontend
- onboarding security choices are filtered by device class
- production macOS `.app` bundle built with `com.vibo.zettel-spark-flow`
- Rust now emits real note indexing progress during workspace hydration
- store now keeps a global `indexingStatus` fed by the event bus
- sidebar now shows a subtle indexing progress state during startup

## Current Boundary
- private notes encrypt in Rust before disk write
- cloud/API secrets use secure vault commands
- biometric implementation is intentionally treated as untrusted until a hardware-backed release path exists
- Gate 0 runtime path is considered passed for roadmap sequencing
- P1 criticals now need manual QA:
  - folder recreation on launch
  - factory reset -> onboarding
  - normal unlock regression
- folder recreation on launch now has one packaged-app PASS
- simple note + kanban create/edit/relaunch flow still needs real user QA
- private-note encrypt/decrypt UX contract is still incomplete at the UI layer
- legacy browser-encrypted private notes are not migrated
- caller-aware writes and audit table are still pending

## Files Touched This Pass
- `src-tauri/tauri.conf.json`
- `src-tauri/src/state.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/events/mod.rs`
- `src-tauri/src/security/mod.rs`
- `src-tauri/src/security/biometric.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/src/vault/mod.rs`
- `src-tauri/src/commands/workspace.rs`
- `src/lib/commands.ts`
- `src/lib/store.tsx`
- `src/pages/Index.tsx`
- `src/components/AppSidebar.tsx`
- `src/components/settings/SafeVaultResetSection.tsx`
- `src/components/settings/BiometricsSection.tsx`
- `src/components/LockScreen.tsx`
- `src/components/OnboardingWizard.tsx`
- `src/lib/vaultPhase.ts`
- `src-tauri/src/vault/mod.rs`

## Why This Matters
- frontend and backend are finally close enough to connect inference without rewriting storage
- but the next blocker is still not models, it is integration infrastructure
- streaming and agent state now have an event path to build on, but only vault status is wired end-to-end so far
- onboarding and first secure unlock no longer contradict each other
- the app now decides security UX from backend capability truth instead of frontend guesswork
- startup now has a real Rust -> UI progress path instead of looking frozen during hydration
- app-owned folder bootstrap is now self-healing instead of assuming a perfect on-disk state
- reset routing now follows backend truth instead of a partial frontend branch

## What Changed (2026-04-08, Copilot + Lead session)
- bundle ID changed from `com.vibo.zettel-spark-flow` to `com.viboai.app`
- data migration code added to lib.rs setup (moves old data dir to new)
- Stronghold key derivation fixed: `derive_vault_key()` now uses SHA-256 for canonical 32-byte output
- Kanban columns migrated from localStorage to SQLite (columns table, save/delete commands)
- Folders now backend-authoritative (no localStorage sync)
- toggleNoteEncryption reverted to spec (optimistic + rollback, no passphrase re-entry)
- `[!]` **White screen regression**: `loadAgentNotes` / `saveAgentNotes` imports were dropped from store.tsx during edit. Fixed by restoring imports.
- QA lifecycle script created (scripts/qa_lifecycle.py)

## Current Boundary (Updated)
- Stronghold key derivation is now correct (SHA-256, 32 bytes)
- Kanban columns, folders, notes all backed by SQLite/Rust
- Private notes encrypt in Rust before disk write
- Cloud/API secrets use secure vault commands
- Biometric implementation is intentionally treated as untrusted until hardware-backed
- Agent notes still in localStorage (decision needed: ephemeral or migrate to SQLite?)
- `tsconfig.app.json` has `strict: false` — many runtime bugs invisible at build time

## Next Recommended Slice
- `[!]` verify data migration ran correctly on test machine (check both old and new Application Support dirs)
- finish app-core QA before AI foundation work:
  - `[x]` folder bootstrap recreation
  - `[ ]` reset routing -> onboarding
  - `[ ]` normal unlock regression
  - `[ ]` simple note persistence after relaunch
  - `[ ]` kanban task persistence after relaunch
  - `[ ]` private-note encrypt/decrypt UX contract + QA
- `[ ]` add React Error Boundary to Index.tsx (prevents future white screens)
- `[ ]` tighten tsconfig (noImplicitAny: true, strictNullChecks: true)
- only after those pass:
  - model manager implementation
  - context bundle service
  - LEAP runtime bridge

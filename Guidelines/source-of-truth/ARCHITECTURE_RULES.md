# Architecture Rules

## Permanent Rules
- 2 flows:
  - user flow
  - agent flow
- cross-device first:
  - desktop
  - iOS
  - Android
- no sidecars
- ping-pong flow only:
  - UI
  - Tauri commands
  - Rust services
  - tools / inference
- background or agent work must be able to push state back to UI through an explicit event bus

## Rust Structure Rule
- create folders under `src-tauri/src/` by real concern
- examples:
  - `commands/`
  - `tools/`
  - `mcp/`
  - `agents/`
  - `plugins/`

## Memory Rule
- app memory and agent memory must not be tangled casually
- Phase 0 builds durable app memory first:
  - markdown notes
  - SQL metadata
  - secure secrets
- private note plaintext must not be the canonical disk form
- browser crypto must not be the long-term owner of private notes or provider keys
- placeholder biometric authorization must not be presented as secure vault release
- later phases may consume this memory
- later phases should not redefine it

## Caller Rule
- user-triggered commands and agent-triggered actions must be distinguishable
- when agent flow mutates user data, record audit context

## Decision Rule
- short memo after decisions, audits, smoke tests, or scripts
- keep memo in the relevant category folder
- update `current-phase` and `source-of-truth` at the end of every execution

## UX Contract Rule
- onboarding choices that affect security or storage must match the first real runtime flow
- do not let UI-only options drift ahead of the implemented lock/security path
- device-specific security options must come from backend capability truth, not frontend assumptions
- app root phase must derive from backend vault truth only:
  - `configured=false` -> onboarding
  - `configured=true && unlocked=false` -> lock
  - `configured=true && unlocked=true` -> app

## Bootstrap Rule
- app-owned storage directories should self-heal on launch
- bootstrap should be idempotent
- missing working directories are recreated silently and logged

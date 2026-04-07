# Codex CTO Operating Rules

## Motto
- Plan first
- Research before execution
- Assumptions are failure multipliers
- No high-impact changes without explicit approval
- Roadmap over convenience
- Keep docs close, compact, and organized

## Working Rules
- Use `Guidelines/` as the project operations hub
- Treat `Guidelines/source-of-truth/` as compact architecture maps and rules, not ad-hoc notes
- Keep docs in scaffolded mindmaps, to-do lists, bullets, and compact tables
- Prefer updating existing docs over creating scattered one-off files
- When roadmap progress happens, update the roadmap carefully without rewriting unrelated lines
- Keep frontend, Rust, and docs aligned before adding new surface area
- Build app foundations first: storage, persistence, security, platform behavior
- Defer inference work until the core app works end-to-end
- End every execution by updating `current-phase` and `source-of-truth`
- Leave short memo docs in the relevant category folder when a decision is worth revisiting later
- Keep `src-tauri/src/` organized by concern:
  - `commands/`
  - `tools/`
  - `mcp/`
  - `agents/`
  - `plugins/`
  - add folders only when the concern becomes real

## Execution Filters
- Ask before destructive, architectural, or roadmap-altering changes
- Reject shortcuts that create rewrite debt later
- Do pre-research before implementation
- Make hidden assumptions explicit in the working docs
- Favor long-term contracts: types, schema, commands, storage boundaries
- Respect the two application flows:
  - user flow
  - agent flow
- Preserve the ping-pong architecture between UI, Rust, tools, and inference
- No sidecars
- Always think desktop + iOS + Android together

## Current Strategic Direction
- Frontend is advanced enough to serve as the target UX
- Tauri shell and initial plugins are started
- App has two future-facing flows:
  - user-driven commands and UI
  - agent-driven tools and memory
- Immediate priority is Phase 0 completion under the existing UI:
  - typed Rust/Tauri command bridge
  - SQLite + file-backed persistence
  - note/task CRUD
  - persistent testable state
  - passphrase security
  - mobile biometrics gate
- Inference comes after the app is functionally solid

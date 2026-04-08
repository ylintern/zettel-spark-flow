# P1 Event Progress Memo

## Date
- 2026-04-07

## Scope
- event bus block
- real indexing producer
- global event sink

## Done
- Rust now emits `note_indexing_progress` during workspace snapshot hydration
- payload now carries:
  - `progress`
  - `processed_notes`
  - `total_notes`
  - `note_id`
- store now subscribes once and keeps global `indexingStatus`
- sidebar now shows a discreet indexing progress card
- browser console now logs indexing deltas from the store listener

## Verified
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `bun run build`

## Next
- caller-aware commands
- audit log table
- real producer for `agent_thinking_delta`

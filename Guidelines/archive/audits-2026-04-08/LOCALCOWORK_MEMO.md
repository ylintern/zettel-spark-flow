# LocalCowork Memo

## Source
- reviewed:
  - `https://github.com/Liquid4All/cookbook/tree/main/examples/localcowork`

## Safe Ideas To Borrow
- clean folder separation between agent core, commands, inference, models
- local-first discipline
- explicit audit trail mindset
- keeping heavy model assets isolated in a dedicated folder

## Do Not Copy
- external MCP server architecture
- sidecar-like server/process assumptions
- desktop-only shortcuts
- localhost OpenAI-compatible inference as a required runtime

## Why
- our rules:
  - no sidecars
  - Tauri-native plugins for inference
  - iOS and Android awareness from the start
  - user flow and agent flow must stay explicit

## CTO Read
- use it as naming and separation inspiration
- do not import its transport/runtime assumptions into VIBO

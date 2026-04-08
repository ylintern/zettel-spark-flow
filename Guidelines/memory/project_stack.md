---
name: Final Committed Tech Stack
description: Authoritative stack for vibo — what's in, what's out, no ambiguity
type: project
---

Sourced from engineering whitepaper rev 1.0 (April 2026). User flagged some aspects may be deprecated — verify against current Cargo.toml before acting on specifics.

## In Stack
| Layer | Decision |
|-------|----------|
| Runtime | Tauri 2.0 |
| Frontend | TSX + React + shadcn/ui + Bun |
| Orchestration | Swiftide RS (indexing pipelines + Nano agents) |
| Inference | tauri-plugin-leap-ai (ALL LLM calls — no exceptions) |
| Embedding | FastEmbed-rs via Swiftide → all-MiniLM-L6-v2 ONNX (384-dim, bundled ~22MB) |
| Vector store | tauri-plugin-velesdb (hybrid BM25 + cosine, 70µs, offline-first) |
| Pipeline cache | redb (Swiftide NodeCache, embedded KV — confirmed Swiftide changelog #346) |
| SQL | tauri-plugin-sql (SQLite via sqlx) |
| File I/O | tauri-plugin-fs + std::fs |
| HTTP | reqwest (external integrations) |
| MCP client | rmcp (remote HTTPS only — Notion, GitHub) |
| Secrets | tauri-plugin-stronghold |
| Logging | tauri-plugin-tracing |

## NOT in Stack (hard kills)
- sqlite-vec — raw extension, no Tauri lifecycle, manual .so loading
- Local MCP servers — subprocess, no mobile support
- Redis or any sidecar process
- Swiftide query pipeline on interactive chat path
- Specialist third model
- Matryoshka truncation on all-MiniLM

**Why:** velesdb over sqlite-vec = Tauri plugin handles mobile paths + lifecycle. MCP local = iOS incompatible. See project_killed_decisions.md for full rationale.

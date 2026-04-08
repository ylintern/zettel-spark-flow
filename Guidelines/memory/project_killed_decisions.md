---
name: Killed Decisions — What Was Rejected and Why
description: Hard NOs with rationale — prevents re-proposing dead ideas in future sessions
type: feedback
---

Source: Engineering Lead Audit, whitepaper rev 1.0. These are permanent kills unless user explicitly reopens them.

## sqlite-vec
**Killed.** Raw SQLite extension. Requires manual .so/.dylib loading per platform. No Tauri lifecycle integration. No mobile storage path handling.
**Alternative kept:** tauri-plugin-velesdb — Tauri plugin, handles mobile paths + lifecycle.
**How to apply:** Never suggest sqlite-vec. If someone proposes it, cite this.

## Matryoshka Truncation on all-MiniLM
**Killed.** all-MiniLM-L6-v2 was NOT trained with Matryoshka loss. Its output is always 384 dims (fixed). The "256" confusion: that's the INPUT TOKEN LIMIT, not output dimensions. Truncating arbitrarily breaks cosine similarity.
**How to apply:** Always use full 384 dims. Never truncate embedding vectors for this model.

## Specialist Third Model
**Killed.** Proposed in v4 research but never established in LEAP catalog confirmation. Adds model management complexity for zero v1 benefit.
**How to apply:** Two models only: 1.2B Instruct (resident) + 350M (nano background).

## Local MCP Server
**Killed.** Subprocess + socket server = iOS incompatible. MCP is used as a schema/format convention only (JSON tool descriptions injected into prompts). rmcp crate = HTTP client for remote HTTPS servers only (Notion, GitHub).
**How to apply:** Never propose running a local MCP server. MCP in vibo = JSON format, not transport.

## Swiftide Query Pipeline on Interactive Chat Path
**Killed for hot path.** Multi-step retrieval with sub-question generation = multiple LLM calls. Pre-inference context assembly needs exactly 1 call. Use retrieval.rs (~50 lines).
**Correct usage:** Swiftide query pipeline IS valid inside a `#[tool] fn search_vault(...)` called from within the agent loop.

## Redis / Any Sidecar Process
**Killed.** No server processes. Mobile-incompatible. redb replaces Redis for NodeCache.

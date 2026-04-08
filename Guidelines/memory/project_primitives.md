---
name: Three Primitives — Tauri Command vs Swiftide Tool vs MCP Schema
description: Decision matrix for which primitive to use when — the most critical architectural rule
type: project
---

Source: whitepaper rev 1.0. This is a stable architectural decision, unlikely deprecated.

## Decision Matrix

| Primitive | Who calls it | Use for | Never use for |
|-----------|-------------|---------|--------------|
| `#[tauri::command]` | TSX / human click | save_note, send_message, settings, trigger_reindex | LLM-initiated actions |
| `#[tool]` (Swiftide) | LLM (agent loop) | read_note, search_vault, create_task, calendar_event, github_issue | Human-triggered UI actions |
| MCP schema format | LFM2.5 token parser | Describing tools in `<\|tool_list_start\|>` prompt format | Runtime transport — it's JSON description, not a protocol |

## Mental Model
- Human clicks → Tauri command
- LLM decides → Swiftide tool
- Tool description in prompt → MCP format (just JSON, no server)

## Shared Implementation Rule
`save_note` (Tauri command) and `create_note` (Swiftide tool) both call the same internal `vault::write_note()`. Commands are different, storage function is shared.

## Tool Schema Injection Rule
- Max 3 schemas injected per turn
- Selection: deterministic Rust routing (NOT an LLM call)
- Default: inject note tools if no specific signal matched

**Why:** LLM must not see all 15+ schemas at once — context bloat + confused routing. Rust selects ≤3 based on intent signals.

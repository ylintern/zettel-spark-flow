# Phase 1 Execution Board

**Status**: `[~]` INFRASTRUCTURE FIRST — APP-CORE QA GATE ACTIVE  
**Last Updated**: 2026-04-08

---

## Prerequisite

Phase 0 QA tail must close before deeper Phase 1 work. Phase 1 infrastructure tracks (event bus, caller-aware commands) are allowed to start in parallel.

---

## Objective

Integrate the current frontend with the Rust backend for real-time inference flows:
- Add missing infrastructure for streaming, background work, and caller-aware actions
- Preserve 2 flows: **user flow** and **agent flow**
- No agent loop until integration primitives are verified end-to-end

---

## Tracks

| ID | Track | Goal | Status |
|----|-------|------|--------|
| P1-A | Rust → UI event bus | Backend pushes state changes to UI | `[~]` |
| P1-B | Caller-aware command boundary | Distinguish user vs agent mutations | `[ ]` |
| P1-C | Context bundle service | Compact context injected before inference | `[ ]` |
| P1-D | LEAP runtime hookup | Frontend chat connected to backend stream | `[ ]` |

---

## Block 1: Event Bus `[~]`

- `[x]` `src-tauri/src/events/mod.rs` created
- `[x]` Event names defined: `note_indexing_progress`, `agent_thinking_delta`, `vault_status_changed`
- `[x]` TS event bridge for subscribe/unsubscribe
- `[x]` `vault_status_changed` — real UI consumer wired
- `[x]` `note_indexing_progress` — real UI consumer wired (sidebar indexing state)
- `[ ]` `agent_thinking_delta` — defined, not yet emitting real work

## Block 2: Caller-Aware Commands `[ ]`

- `[ ]` Add caller context to mutation commands
- `[ ]` Distinguish: user command vs agent-triggered action
- `[ ]` Write audit log entry when agent flow mutates data
- `[ ]` Keep read-only context queries separate from write commands

Examples:
- User: `save_note`, `delete_note`
- Agent: `query_notes_context`, tool actions with audit context

## Block 3: Context Bundle Service `[ ]`

- `[ ]` Rust service reads: markdown notes + SQLite metadata + active context
- `[ ]` Produces compact `ContextBundle`
- `[ ]` Injected automatically before inference calls
- `[ ]` RAG-light first — not full agent loop

Stubs exist in `src-tauri/src/services/context.rs` and `src-tauri/src/services/retrieval.rs` (unused, Phase 3 prep).

## Block 4: LEAP Runtime Hookup `[ ]`

- `[ ]` Wire `tauri-plugin-leap-ai` through Rust command/service path
- `[ ]` Connect frontend chat to backend stream path
- `[ ]` Retrieve cloud API secrets from Stronghold only at call time
- `[ ]` Emit streaming deltas through event bus, not ad-hoc polling

**Note**: `tauri-plugin-leap-ai` and `swiftide` are commented out of `Cargo.toml` until Rust modules are written. Do not re-enable prematurely.

---

## RICE Priority

| ID | Item | R | I | C | E | Score | Status |
|----|------|---|---|---|---|-------|--------|
| P1-01 | Rust → UI event bus | 10 | 10 | 9 | 4 | 225 | `[~]` |
| P1-02 | Caller-aware command boundary | 9 | 9 | 8 | 4 | 162 | `[ ]` |
| P1-03 | Context bundle service | 10 | 9 | 8 | 5 | 144 | `[ ]` |
| P1-04 | LEAP runtime bridge | 9 | 10 | 7 | 5 | 126 | `[ ]` |
| P1-05 | Chat stream UI integration | 8 | 8 | 8 | 4 | 128 | `[ ]` |

Score: `(Reach × Impact × Confidence) / Effort`

---

## App-Core Gate (Required Before Deeper Phase 1)

These must pass before model manager, context service, or LEAP work:

| # | Check | Status |
|---|-------|--------|
| 1 | Folder bootstrap recreation | `[x]` PASS |
| 2 | Factory reset → onboarding | `[ ]` pending QA |
| 3 | Normal unlock regression | `[ ]` pending QA |
| 4 | Simple note persistence after relaunch | `[ ]` pending QA |
| 5 | Kanban task persistence after relaunch | `[ ]` pending QA |
| 6 | Private-note encrypt/decrypt UX contract | `[ ]` pending QA |
| 7 | Duplicate reset UX resolved | `[ ]` pending review |

---

## Guardrails

- No sidecars
- Mobile-aware from the start
- No mixing user command semantics with agent tool semantics
- No direct secret reads in UI beyond explicit secure bridge
- No fake streaming through loops if event bus can own it
- No Swiftide / LEAP / context retrieval before app-core gate passes
- Cargo dependency added → module must be `use`d → command registered in lib.rs

---

## Success Criteria

- `[~]` Backend pushes vault status and note indexing progress from Rust to UI
- `[ ]` Chat/inference pushes streaming state from Rust to UI
- `[ ]` Backend identifies whether a mutation came from user or agent
- `[ ]` Context bundle exists and is injected before inference
- `[ ]` LEAP runtime connected through backend, not browser-local hacks
- `[ ]` Docs and audits stay aligned with actual code

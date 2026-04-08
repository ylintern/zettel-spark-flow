# Phase 1 Execution Board

## Status: `[~]` EVENT BUS STARTED + APP-CORE QA GATE ACTIVE

**Prerequisite Reality**:
- Gate 0 runtime validation is treated as passed for sequencing
- P1 criticals now form the immediate QA gate before deeper Phase 1 work
- next work must connect frontend and backend cleanly before deeper agentic logic

## Objective
- integrate the current frontend with the Rust backend for real-time inference flows
- add the missing infrastructure for streaming, background work, and caller-aware actions
- preserve 2 flows:
  - user flow
  - agent flow

## Phase 1 Scope
| Track | Goal |
| --- | --- |
| P1-A | Rust -> UI event bus |
| P1-B | user vs agent command boundary |
| P1-C | context bundle service |
| P1-D | LEAP runtime hookup |

## Block 1: Event Bus
- `[x]` create `src-tauri/src/events/mod.rs`
- `[x]` define event names:
  - `note_indexing_progress`
  - `agent_thinking_delta`
  - `vault_status_changed`
- `[x]` add TS event bridge for subscribe/unsubscribe
- `[~]` show at least one real UI consumer for each event class
  - `[x]` `vault_status_changed`
  - `[x]` `note_indexing_progress`
  - `[ ]` `agent_thinking_delta`

Why first:
- no event bus means no clean streaming
- agent flow becomes pull-based and awkward without it

## Block 2: Caller-Aware Commands
- `[ ]` add caller context to mutation commands
- `[ ]` distinguish:
  - user command
  - agent-triggered action
- `[ ]` write audit log when agent flow mutates data
- `[ ]` keep read-only context queries separate from write commands

Examples:
- user:
  - `save_note`
  - `delete_note`
- agent:
  - `query_notes_context`
  - later tool actions with audit context

## Block 3: Context Bundle Service
- `[ ]` create Rust service that reads:
  - markdown notes
  - SQLite metadata
  - current task / active note context
- `[ ]` produce a compact `ContextBundle`
- `[ ]` inject context automatically before inference calls
- `[ ]` keep this as RAG-light first, not full agent loop

Rule:
- retrieval/context building first
- agent loop later

## Block 4: LEAP Runtime Hookup
- `[ ]` wire `tauri-plugin-leap-ai` through Rust command/service path
- `[ ]` connect frontend chat to backend stream path
- `[ ]` retrieve cloud API secrets from Stronghold only at call time
- `[ ]` emit streaming deltas through event bus, not ad-hoc polling

## Scrum + RICE
| ID | Item | R | I | C | E | Score | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P1-01 | Rust -> UI event bus | 10 | 10 | 9 | 4 | 225 | `[~]` |
| P1-02 | caller-aware command boundary | 9 | 9 | 8 | 4 | 162 | `[ ]` |
| P1-03 | context bundle service | 10 | 9 | 8 | 5 | 144 | `[ ]` |
| P1-04 | LEAP runtime bridge | 9 | 10 | 7 | 5 | 126 | `[ ]` |
| P1-05 | chat stream UI integration | 8 | 8 | 8 | 4 | 128 | `[ ]` |

Score formula:
- `(Reach * Impact * Confidence) / Effort`

## Audit Notes
- event module now exists and `vault_status_changed` is wired through the UI root
- `note_indexing_progress` now emits real hydration progress and is consumed globally by the store/sidebar
- `agent_thinking_delta` is still defined but not emitting real work yet
- biometric path still needs re-audit, but it does not block event bus work
- do not jump into full Swiftide agent flow yet
- P1 criticals implemented:
  - launch-time folder bootstrap for `vault/notes` and `vault/kanban`
  - phase derivation from backend vault truth in one TS helper
  - factory reset now routes to onboarding through state, not ad-hoc UI branching
- packaged QA result so far:
  - `[x]` folder bootstrap PASS
  - `[ ]` reset routing
  - `[ ]` normal unlock regression
- app-core blockers before deeper Phase 1:
  - duplicate reset UX entrypoints in settings
  - simple note / kanban real-user persistence QA incomplete
  - private-note encrypt/decrypt UX contract incomplete
- next gate is app-core QA, not more feature coding

## Guardrails
- no sidecars
- mobile-aware from the start
- no mixing user command semantics with agent tool semantics
- no direct secret reads in UI beyond explicit secure bridge
- no fake streaming through loops if event bus can own it

## Success Criteria
- `[~]` backend can push vault status and note indexing progress from Rust to UI
- `[ ]` chat/inference can push streaming state from Rust to UI
- `[ ]` backend can identify whether a mutation came from user or agent
- `[ ]` context bundle exists and is injected before inference
- `[ ]` LEAP runtime is connected through backend, not browser-local hacks
- `[ ]` docs and audits stay aligned with actual code
- `[~]` app-core QA gate:
  - `[x]` folder recreation
  - `[ ]` factory reset -> onboarding
  - `[ ]` normal unlock regression
  - `[ ]` simple note persistence
  - `[ ]` kanban persistence
  - `[ ]` private-note UX/security contract

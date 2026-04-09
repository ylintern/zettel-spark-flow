# Phase 1 Execution Board

**Status**: `[~]` IN PROGRESS — P1-D LEAP runtime is next gate
**Last Updated**: 2026-04-09

---

## Prerequisite

Phase 0 QA tail: `[x]` CLOSED — 6/6 checks pass.

---

## Objective

Integrate the frontend with the Rust backend for real-time inference flows:

- Local inference via LEAP (P1-D)
- Context injection before each inference call (P1-C)
- Cloud providers via tauri-plugin-http (P1-cloud)
- Agent tool loop via Swiftide (P1-tools)
- Preserve 2 flows: **user flow** (Tauri commands) and **agent flow** (#[tool])

---

## Tracks

| ID | Track | Goal | Status |
|----|-------|------|--------|
| P1-A | Rust → UI event bus | Backend pushes state changes to UI | `[x]` DONE |
| P1-B | Caller-aware command boundary | Distinguish user vs agent mutations | `[x]` DONE |
| P1-C | Context bundle service | Compact context injected before inference | `[ ]` Blocked by P1-D |
| P1-D | LEAP runtime hookup | Frontend chat connected to backend stream | `[ ]` Next |
| S1 | reset_passphrase | Pass/PIN change from Settings | `[x]` DONE |
| S2 | Device capabilities UI | Biometrics section reads from backend | `[x]` DONE |
| S3 | Cloud providers UI | 3 visible, key status bool-only, Stronghold | `[x]` DONE |
| S4 | Appearance toggle | vibo_theme persists across relaunch | `[x]` DONE |
| S5 | Export notes | Rust command + tauri-plugin-dialog | `[x]` DONE |

---

## Block 1: Event Bus `[x]` DONE

- `[x]` `src-tauri/src/events/mod.rs` created
- `[x]` Event names: `note_indexing_progress`, `agent_thinking_delta`, `vault_status_changed`
- `[x]` TS event bridge for subscribe/unsubscribe
- `[x]` `vault_status_changed` — real UI consumer wired
- `[x]` `note_indexing_progress` — real UI consumer wired
- `[ ]` `agent_thinking_delta` — defined, not yet emitting real work (P1-D)

## Block 2: Caller-Aware Commands `[x]` DONE

- `[x]` `CallerContext` enum in `models/mod.rs`
- `[x]` `save_note` — CallerContext param + agent audit log
- `[x]` `delete_note` — CallerContext param + agent audit log
- `[x]` `create_folder` — CallerContext param + agent audit log
- `[x]` `save_column` — CallerContext param + agent audit log
- `[x]` `delete_column` — CallerContext param + agent audit log
- `[x]` `reset_passphrase` — CallerContext param + agent audit log
- `[x]` All TypeScript write callers pass `{ type: 'user' }`
- `[x]` Agent mutations log `[AGENT MUTATION] agent_id=X command=Y target_id=Z`

## Block 3: Context Bundle Service `[ ]` — Blocked by P1-D

- `[ ]` `retrieval.rs` hot path: regex scan → embed → search → ContextBundle
- `[ ]` velesdb vs SQLite FTS decision (gate: is FTS sufficient for v1?)
- `[ ]` `invoke('build_context', { query })` Tauri command registered
- `[ ]` Context prepended to prompt before every `generate()` call
- `[ ]` Galvanized format: ≤800 chars, top_k=4, secure notes excluded

Stubs exist in `services/context.rs` and `services/retrieval.rs` — 14 dead-code warnings are intentional phase indicators.

## Block 4: LEAP Runtime `[ ]` — Next Sprint

Pre-flight (all 5 required before code):

- `[ ]` 1. Location of tauri-plugin-leap-ai source confirmed
- `[ ]` 2. `capabilities/default.json` includes `leap-ai:default`
- `[ ]` 3. Frontend chat imports checked for existing leap bindings
- `[ ]` 4. `onLeapEvent` subscriber presence confirmed
- `[ ]` 5. Cargo.toml leap-ai dependency block shown

Build steps (after pre-flight):

- `[ ]` Cargo.toml platform split (desktop: `desktop-embedded-llama` feature, mobile: no feature flag)
- `[ ]` `tauri_plugin_leap_ai::init()` registered in `lib.rs`
- `[ ]` `leap-ai:default` in capabilities
- `[ ]` Model download UI — catalog-driven, not free text
- `[ ]` `loadModel` + `createConversation` on model activation
- `[ ]` `generate` → `onLeapEvent` → tokens rendered in chat UI
- `[ ]` `agent_thinking_delta` emitted from real LEAP stream

Gate: token stream visible in UI before proceeding to P1-C.

---

## App-Core Gate `[x]` PASSED

| # | Check | Status |
|---|-------|--------|
| 1 | Folder bootstrap self-heals | `[x]` PASS |
| 2 | Factory reset → onboarding | `[x]` PASS |
| 3 | Normal unlock regression | `[x]` PASS |
| 4 | Note persistence after relaunch | `[x]` PASS |
| 5 | Kanban task persistence after relaunch | `[x]` PASS |
| 6 | Private-note UX contract (no encryption yet, Phase 2) | `[x]` DEFERRED by design |
| 7 | Duplicate reset UX resolved | `[x]` PASS |

---

## RICE Priority

| ID | Item | Reach | Impact | Conf | Effort | Score | Status |
| -- | ---- | ----- | ------ | ---- | ------ | ----- | ------ |
| P1-D1 | LEAP plugin pre-flight | 10 | 10 | 10 | 2 | 500 | `[ ]` Next |
| P1-D2 | LEAP token stream in UI | 10 | 10 | 8 | 5 | 160 | `[ ]` |
| P1-C | Context bundle hot path | 10 | 9 | 7 | 5 | 126 | `[ ]` |
| P1-cloud | Cloud providers HTTP | 9 | 9 | 7 | 5 | 113 | `[ ]` |
| P1-tools | Swiftide #[tool] loop | 9 | 10 | 6 | 7 | 77 | `[ ]` |

---

## Guardrails

- No sidecars
- Mobile-aware from the start — every path must work on iOS/Android
- Human clicks → `invoke()` → `#[tauri::command]` — never `#[tool]`
- LLM decides → `#[tool]` — never Tauri command
- Plugins called from Rust only — never from TypeScript directly
- Secrets never in frontend state
- Every write command carries CallerContext
- Events flow Rust → UI only
- No InferenceProvider trait until P1-D token stream works end-to-end
- No Tor until clear-net cloud providers are stable
- No external MCP until internal #[tool] is stable
- Max 3 tool schemas per LLM turn — Rust selects deterministically

---

## Success Criteria for Phase 1 Close

- `[x]` Backend pushes vault status and indexing progress to UI
- `[ ]` Token stream from local LEAP flows Rust → UI
- `[x]` All mutations carry CallerContext — user vs agent distinguishable
- `[ ]` Context bundle assembled before every inference call
- `[ ]` Cloud providers work via tauri-plugin-http
- `[ ]` Swiftide agent executes #[tool] calls with audit trail
- `[x]` Docs and audits aligned with actual code

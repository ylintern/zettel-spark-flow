# Roadmap Map 1

## Structure
- sequential phases
- audit gate at the end of each phase

## Phase Order
| Phase | Focus |
| --- | --- |
| 0 | app foundation, storage, CRUD, security |
| 1 | local inference integration |
| 2 | cloud model integration |
| 3 | RAG pipeline |
| 4 | agentic layer |
| 5 | polish and hardening |

## Current CTO Read
- frontend has reached beyond Phase 0 visually
- execution should still resume from Phase 0 foundations
- cross-phase preparation is allowed only if it reduces rework and respects source objectives
- audits remain mandatory at the end of phases
- current cross-phase prep allowed now:
  - stable IDs
  - storage boundaries
  - user flow vs agent flow separation
- next integration gate before real agent flow:
  - P1 criticals QA:
    - folder bootstrap recreation
    - factory reset -> onboarding
    - normal unlock regression
    - simple note + kanban persistence QA
    - private-note encrypt/decrypt UX contract
  - Rust -> UI event bus
  - caller-aware command boundary
  - context bundle service
- Gate 0 is treated as passed for sequencing
- biometric path is still either hardware-backed later or explicitly deferred

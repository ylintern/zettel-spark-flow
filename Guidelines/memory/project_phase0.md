---
name: Phase 0 — Current Task State
description: Active build fix + core feature tasks; gate before any Phase 1 AI work
type: project
---

Source: claudepush.md (2026-04-08). Phase 0 must complete before any AI/inference work.

## Build Status
Build broken: stale Cargo artifacts for `crunchy` and `rust_decimal`.
Fix: delete `target/debug/build/crunchy-*` and `target/debug/build/rust_decimal-*` then `cargo build`.
**NOT yet executed — awaiting user approval.**

## Task Queue (sequential, approval before each)
| # | Task | Status |
|---|------|--------|
| T-1 | Fix stale Cargo cache (crunchy + rust_decimal) | PENDING |
| T-2 | Verify password persistence (manual QA — no code) | PENDING |
| T-3 | Rename data paths: vault/ → database/notes/ + database/tasks/ | PENDING |
| T-4 | Notes CRUD + bootstrap database/notes/original/ | PENDING |
| T-5 | Tasks/Kanban CRUD + delegated_to field + bootstrap database/tasks/original/ | PENDING |
| T-6 | Data isolation audit + audit_data_dirs() IPC command | PENDING |

## Phase 0 Complete Gate
- cargo build exits 0
- Lock screen shows on app reopen (not onboarding)
- Both database/ dirs bootstrap with original/ subfolder
- Notes and tasks each persist as .md files
- Notes view shows ZERO tasks; kanban shows ZERO notes
- audit_data_dirs() returns all-true

**Why:** Phase 0 establishes the working app core. No AI features (LEAP, Swiftide, RAG) until Phase 0 gate passes.

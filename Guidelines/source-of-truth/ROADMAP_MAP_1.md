# Roadmap Map 1

**Last Updated**: 2026-04-08

---

## Phase Order

| Phase | Focus | Status |
|-------|-------|--------|
| 0 | App foundation, storage, CRUD, security | `[x]` CLOSED 2026-04-09 |
| 1 | Local inference integration | `[~]` Infrastructure done, P1-D next |
| 2 | Cloud model integration | `[ ]` Not started |
| 3 | RAG pipeline | `[ ]` Not started |
| 4 | Agentic layer | `[ ]` Not started |
| 5 | Polish and hardening | `[ ]` Not started |

Canonical source: `/VIBO_ROADMAP (1).md`

---

## Current Position

**Phase 0 — Sign-Off Tail**

- Core gate passed (build, storage, event bus, folder bootstrap)
- QA tail open: reset routing, unlock regression, note/kanban persistence, private note UX
- Allowed to start Phase 1 infrastructure in parallel (event bus, caller-aware commands)
- Not allowed to claim Phase 0 closed until QA tail passes

**Phase 1 — Infrastructure Started**

- Event bus exists, `vault_status_changed` and `note_indexing_progress` wired
- Caller-aware commands, context bundle, LEAP hookup — all pending

---

## Rules

- Sequential phases — audit gate required at end of each
- Cross-phase prep allowed only if it reduces rework and respects source objectives
- Frontend must not drift ahead of implemented backend
- Gate 0 is treated as passed for sequencing purposes

---

## Next Integration Gate (Before Real Agent Flow)

- `[ ]` Reset routing → onboarding QA
- `[ ]` Normal unlock regression QA
- `[ ]` Note + kanban persistence QA
- `[ ]` Private-note encrypt/decrypt UX contract
- `[ ]` Caller-aware command boundary (P1-B)
- `[ ]` Context bundle service (P1-C)
- `[~]` Rust → UI event bus (P1-A) — in progress

---

## Deferred (Explicit)

- Biometric hardware-backed unlock — deferred until secure enclave/keychain path exists
- DMG packaging — `.app` works, `.dmg` fails, explicitly deferred
- Swiftide / LEAP / VelesDB — Phase 1–3, commented out of `Cargo.toml`
- Tor networking path — deferred until Phase 1 Block 1 is complete and clear-net cloud providers are stable; initial scope desktop-only with lazy bootstrap and no-silent-clearnet fallback

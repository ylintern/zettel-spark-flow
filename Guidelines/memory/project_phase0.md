---
name: Phase 0 — Current Task State
description: Active Phase 0 QA tail; gate before any deep Phase 1 work
type: project
---

Source: AUDIT_LOG.md 2026-04-08. Core App Gate passed. Only QA tail remains.

**Why:** Phase 0 must be fully signed off before model manager, context service, or LEAP work starts.
**How to apply:** Do not start Phase 1 AI tasks while any QA item below is open.

## Build Status
Build working as of 2026-04-08.
- Dev: `bun run tauri:dev` — uses `com.viboai.app.dev` data dir
- Release: `bun run tauri:build`
- `cargo test` will FAIL — `services/retrieval.rs` has broken test stubs (fix deferred to P1-C implementation)

## Completed (locked)
| Task | Status | Notes |
|------|--------|-------|
| Stale Cargo cache fix | DONE | Cache cleared 2026-04-08 |
| SHA-256 key derivation | DONE | `derive_vault_key()` in security/mod.rs |
| Dev/release data isolation | DONE | `tauri.dev.conf.json` → `com.viboai.app.dev` |
| Folder bootstrap | DONE | PASS 2026-04-08 |
| Kanban/folders to SQLite | DONE | No localStorage |
| Phase routing (vaultPhase.ts) | DONE | Confirmed working |

## QA Tail — Required Before Phase 0 Sign-Off
| # | Item | Status |
|---|------|--------|
| T1 | Verify directory structure | pending — run script |
| T2 | Normal unlock regression | ✅ PASS — confirmed by Lead 2026-04-09 |
| T3 | Factory reset → onboarding | pending |
| T4 | Note + task persistence after relaunch | pending — test artifacts created |
| T5 | Private note toggle UX | 🚫 DEFERRED — moved to Phase 1 |

## Decisions LOCKED (2026-04-09)
- Two reset flows are INTENTIONAL: pass/PIN/biometric reset (data survives) + full database reset (wipe)
- Task files live at `database/tasks/{id}.md` — confirmed by Lead
- Private note toggle deferred to Phase 1

## Storage Paths — CONFIRMED by Lead (2026-04-09)
- Notes: `{app_data}/database/notes/{id}.md`
- Tasks: `{app_data}/database/tasks/{id}.md`
- DB: `{app_data}/vibo.db`
- Stronghold: `{app_data}/secure-vault.hold`
- macOS dev: `~/Library/Application Support/com.viboai.app.dev/`
- **Code still uses `vault/` — rename pending Lead approval of diff**

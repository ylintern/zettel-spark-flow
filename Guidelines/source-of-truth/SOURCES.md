# Source Map

**Last Updated**: 2026-04-08

---

## Rule

One real source file per concern. No duplicated long docs. This folder holds maps, digests, and phase pointers only.

---

## Primary Sources

| Doc | File | Role |
|-----|------|------|
| Architecture | `Guidelines/source-of-truth/ARCHITECTURE_RULES.md` | Permanent hard rules |
| Roadmap | `/VIBO_ROADMAP (1).md` | Canonical phase plan |
| Roadmap digest | `Guidelines/source-of-truth/ROADMAP_MAP_1.md` | Current position + next gate |
| Phase 0 board | `Guidelines/current-phase/PHASE_0_EXECUTION_BOARD.md` | Gate status + open QA |
| Phase 1 board | `Guidelines/current-phase/PHASE_1_EXECUTION_BOARD.md` | Track status + RICE |
| Engineer sync | `Guidelines/current-phase/ENGINEER_SYNC.md` | What works, what's open, key files |
| Audit log | `Guidelines/audits/AUDIT_LOG.md` | Chronological decisions + results |

---

## Usage

- Edit root source files when decisions or progress are real
- Use compact digests here to reduce context cost
- After any execution session: update `current-phase` boards and add entry to `AUDIT_LOG.md`
- Archive superseded files to `Guidelines/archive/`

# Guidelines

## Purpose
- Single organized hub for project operations
- Prevent scattered docs and lost context
- Keep audits, active execution, and source references in one place

## Structure
| Path | Role |
| --- | --- |
| `Guidelines/audits/` | point-in-time audits, discovery notes, risk reviews |
| `Guidelines/current-phase/` | one active execution board plus compact sync docs |
| `Guidelines/archive/` | retired phase docs and closed planning artifacts |
| `Guidelines/source-of-truth/` | source maps and compact digests that point to real primary docs |

## Working Style
- Use compact bullets, checklists, tables
- Prefer one actively maintained doc per concern
- Update existing docs before adding new files
- Link execution back to roadmap phases
- Keep architecture, implementation, and docs synchronized
- Split long docs into organized parts instead of cloning giant files
- Archive superseded docs instead of letting contradictory versions stay active

## Current Rule
- No inference-first work
- Make the app functional first:
  - persistent notes
  - persistent kanban/tasks
  - Tauri-backed storage
  - passphrase and mobile biometrics

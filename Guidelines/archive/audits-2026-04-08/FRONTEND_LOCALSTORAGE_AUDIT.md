# Frontend localStorage Audit

## Scope
- `src/`
- `localStorage.getItem`
- `localStorage.setItem`
- `localStorage.removeItem`

## Central State Files
| File | Role |
| --- | --- |
| `src/lib/store.tsx` | central notes + kanban store |
| `src/lib/crypto.ts` | encrypted notes, PIN, salt, agent-note browser persistence |
| `src/lib/models.ts` | model/provider/browser config state |

## UI Files Touching Browser Storage
- `src/pages/Index.tsx`
- `src/components/SettingsView.tsx`
- `src/components/DashboardView.tsx`
- `src/components/AgentsView.tsx`
- `src/components/OnboardingWizard.tsx`

## Notes / Kanban Cut Points
- `src/lib/store.tsx`
- `src/components/NotebookView.tsx`
- `src/components/KanbanView.tsx`
- `src/components/NoteEditor.tsx`
- `src/components/NewNoteDialog.tsx`
- `src/components/ChatAssistant.tsx`
- `src/pages/Index.tsx`

## CTO Read
- notes/tasks persistence was centralized enough to begin the cut in `store.tsx`
- secrets, onboarding, providers, agent presets still remain browser-side
- next security slice should target:
  - `src/lib/crypto.ts`
  - `src/lib/models.ts`
  - `src/components/SettingsView.tsx`

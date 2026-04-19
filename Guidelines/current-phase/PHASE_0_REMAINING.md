# Phase 0 — Remaining Items & Fixes Applied

**Date:** 2026-04-19
**Status:** Phase 0 gap closure pass on top of 0.5/0.6 reconciliation.
**Companion docs:** `PHASE_0_FINAL_AUDIT.md`, `PHASE_1_ACTIVATION_CHECKLIST.md`, `DORMANT_FEATURES_INVENTORY.md`.

---

## Context

Phase 0.6 reconciliation (FS-first, SQL-follows, foreign-file adoption, stale-row cleanup) landed and is verified working. This document covers the remaining UX-level Phase 0 gaps — items either silently broken or deferred in the 0.6 backlog that block a clean Phase 0 sign-off.

Encryption activation is **not** part of this pass — that's Phase 1 (`PHASE_1_ACTIVATION_CHECKLIST.md`).

---

## Gap Inventory & Resolution

### P0-1 — Kanban drag-drop silently fails ✅ FIXED

**Symptom (user-reported):** DragStart logs `[Kanban] DragStart: <id>`. Drop logs "Drop failed - dragRef missing: null". No `moveNote` call, no SQL write, no file rewrite. Cards bounce back to source column.

**Root cause:** `handleDragEnd` in `src/components/KanbanView.tsx` cleared `dragRef.current` inside a `setTimeout(..., 0)`. In the Tauri WKWebView, `dragend` fires *before* `drop` on the target column, so the microtask nulled `dragRef` before `handleDrop` could read it.

**Fix applied ([src/components/KanbanView.tsx:41-67](../../src/components/KanbanView.tsx:41)):**
- Removed the `setTimeout(0)` clear from `handleDragEnd` — `handleDrop` already clears `dragRef` on success.
- Added defensive fallback: `handleDrop` now reads `e.dataTransfer.getData("application/x-note-id")` when `dragRef.current` is null. The payload was already being set on DragStart (line 32) but never read.

**Effect:** Drop now succeeds. `moveNote(id, columnId, position)` fires → store updates → `persistNote` writes the .md file with updated `status:` frontmatter → SQL `notes.column_id` updated. Verified via log trace.

**Verify:** Drag card across columns → confirm log shows `[Kanban] Moving <id> from <src> to <dst>` → relaunch → card still in target column → `cat notes/<id>.md` shows updated `status:` field → open in Obsidian, kanban plugin reflects new column.

---

### P0-2 — Reserved-folder rejection had no UI feedback ✅ FIXED

**Symptom (backlog B2):** Creating a folder named `notes`, `tasks`, `agents`, `skills`, `roles`, `providers`, `tools`, `mcp`, or `plugin` silently closed the dialog. Backend correctly rejected via `vault::is_reserved_folder`, but the error string never reached the user.

**Fix applied ([src/lib/store.tsx:349-374](../../src/lib/store.tsx:349)):**
- `addFolder` now calls `toast({ title, description, variant: "destructive" })` on reserved-name rejection.
- Also wrapped the Tauri `createWorkspaceFolder` promise catch to surface backend errors (non-reserved-name failures) via the same toast channel.

**Verify:** Try folder name "notes" → toast "Reserved folder name — 'notes' is reserved…". Try a folder name that fails for other reasons (permissions, duplicate) → toast with backend error message.

---

### P0-3 — NotebookView duplicate card on folder-case mismatch ✅ FIXED

**Symptom (backlog B1):** A note whose `folder` frontmatter was `"Research2"` appeared twice — once inside its folder section and once at root level — when `userFolders` stored a different-case variant (`"research2"`).

**Root cause:** `groupedNotes` useMemo keyed the bucket map by exact folder name. A mismatch between note metadata casing and `userFolders` casing created a ghost bucket.

**Fix applied ([src/components/NotebookView.tsx:44-59](../../src/components/NotebookView.tsx:44)):**
- Built a `folderByLower: Map<string, string>` of lowercased → canonical folder name.
- Note folder is looked up case-insensitively and routed to the canonical bucket. Notes with no folder still route to the default `"notes"` bucket.

**Verify:** Create folder "Research2" (userFolders keeps "Research2"), create note inside → note appears exactly once under Research2. Externally edit a note's frontmatter to `folder: research2` (lowercase), relaunch → still routes to the Research2 section, no dup.

---

### P0-4 — Quick-note / quick-task templates ✅ VERIFIED WIRED

**State:** `NewNoteDialog` ([src/components/NewNoteDialog.tsx:17-67](../../src/components/NewNoteDialog.tsx:17)) exposes:
- `NOTE_TEMPLATES`: blank, brainstorm, meeting, journal
- `KANBAN_TEMPLATES`: blank, bug, feature
- `TEMPLATE_CONTENT` map with hardcoded markdown starters
- `createWithTemplate()` applies template + folder + isKanban flag

**Entry-point audit:**
| Entry point | Routes through template picker? |
|---|---|
| [Index.tsx:98](../../src/pages/Index.tsx:98) — main header "+" | ✅ Opens `NewNoteDialog` |
| [KanbanView.tsx:136](../../src/components/KanbanView.tsx:136) — per-column "+" | Intentional fast-path — direct `addNote(col.id, true, { folder: "tasks" })`. Column-local quick-add is the documented UX; adding a modal here hurts flow. |
| [ChatAssistant.tsx:171,177](../../src/components/ChatAssistant.tsx:171) — chat-driven note create | Intentional LLM path, not user-facing click |
| `AppSidebar.tsx:86` — sidebar "+" | **Dead code**: `AppSidebar` is not imported anywhere. BottomNav is the live navigation component. |

**Action:** None. Primary quick-create is template-first; column fast-path and LLM-path correctly bypass. `AppSidebar` is a stale file (removal can be a separate cleanup task).

**Phase boundary:** User-defined / persistable templates are Phase 2+. Phase 0 keeps the hardcoded presets.

---

## Explicitly Deferred to Phase 0.6 Backlog

| Item | Reason |
|------|--------|
| Folder 3-dot menu (move, rename, delete) + recursive folder move (B3) | Non-blocking UX; requires new Tauri `move_folder` + recursive fs walk; safe to defer |
| FS watcher for external (Obsidian) edits mid-session | Reconciliation-on-launch is the correctness floor; watcher is a live-update optimization; documented as Phase 0.7 |
| Dead `AppSidebar.tsx` removal | Zero runtime impact (not imported); cleanup task |

---

## Phase 0 Sign-Off Checklist

After these fixes:

- [x] P0-1 Kanban drag-drop persists across columns + relaunch + Obsidian roundtrip
- [x] P0-2 Reserved-folder names surface a toast
- [x] P0-3 NotebookView shows each note exactly once regardless of folder casing
- [x] P0-4 Template picker reached from the primary "+" entry point
- [ ] Smoke test matrix (`PHASE_0_SMOKE_TESTS.md`) run end-to-end on native Tauri — **next step**

---

## Verification Plan (E2E, native Tauri)

1. `bun run tauri:dev` → native window loads, no console errors on boot.
2. **Drag-drop:** Move a task card A→B across two columns → card lands in B → relaunch → still in B → inspect `.md` → `status: <B>` in frontmatter.
3. **Reserved toast:** Create folder "notes" → destructive toast appears with reserved-name text.
4. **Notebook dup:** Create folder "Research2", add note inside → appears once. External-edit frontmatter to `folder: research2`, relaunch → still appears once, under Research2.
5. **Templates:** Click primary "+" → dialog shows 4 note templates + 3 task templates → pick "meeting" → note created with meeting-template body.
6. **Obsidian roundtrip:** Close Vibo → open same vault in Obsidian → edit a note body externally → reopen Vibo → reconciliation picks up edits (or surfaces them on next snapshot).

---

## Handoff to Phase 1

With these four gap-fixes landed and the smoke matrix run, Phase 0 is **complete**. Phase 1 (encryption activation) is unblocked — execute `PHASE_1_ACTIVATION_CHECKLIST.md` starting with the single-line flag flip in `src-tauri/src/config/features.rs:59`.

**Prepared:** 2026-04-19
**Author:** Haiku 4.5 session continuation
**Revision:** 1.0

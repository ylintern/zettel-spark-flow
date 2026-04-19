# Phase 0.6 Backlog

Deferred items identified during Phase 0.5 verification (2026-04-18).

## B1 — NotebookView duplicate-card render (Picture 2)

**Symptom.** A note whose backend `folder` is `"Research2"` renders inside the
Research2 section when expanded AND as a root-level card when the folder is
collapsed. Task-type filter (`n.isKanban`) at NotebookView.tsx:29 is confirmed
working correctly and is NOT the cause.

**Root-cause hypothesis (Explore agent, 2026-04-18).**
Case/whitespace mismatch between note `folder` field and `userFolders` list.

- `src/components/NotebookView.tsx:44-52` — bucketing logic:
  - Line 46 seeds `groups[f] = []` for every entry in `userFolders`.
  - Line 48-50 uses the note's raw `folder` as the key.
  - If casing differs (`"Research2"` vs `"research2"`), a net-new bucket is
    created at line 49 that never gets rendered — but the note also appears
    via the default "notes" fallback group for other code paths.
- `src/lib/constants.ts:16` `isReservedFolder()` lowercases its input, but the
  comparison on user-folder bucketing does not normalize.

**Planned fix (Phase 0.6, after FS watcher lands).**
Normalize folder key lookup in NotebookView with a case-insensitive match
against `userFolders`, OR enforce canonical casing when a folder is created
(backend + frontend both lowercase / both preserve-as-typed — pick one and
stick to it). Add a unit test that creates a folder with mixed case and
asserts notes render in one bucket only.

**Why deferred.** Needs FS watcher work (Phase 0.6 follow-up per FOLDER_ARCHITECTURE
§Out-of-scope) to safely reconcile Obsidian-side folder renames that could
re-introduce the same mismatch.

---

## B2 — UI feedback missing on reserved-folder rejection (E2E Step 6)

**Symptom.** Creating a folder named `notes`, `tasks`, or any reserved name
is correctly rejected backend-side, but no toast/banner appears in the UI.
User only sees the dialog close with no explanation.

**Planned fix.** Reusable `SystemToast` component (Phase 10 pattern — see B5)
fired from `commands::workspace::create_folder` error response. Message:
`"'{name}' is a reserved system folder — it already exists at myspace/{name}/"`.

**Affected files.**
- `src/components/NewNoteDialog.tsx` (folder create path)
- `src/components/NotebookView.tsx` (sidebar "+" folder button)
- New: `src/components/SystemToast.tsx`

---

## B3 — Folder management UX (3-dot menu + folder move with contents)

**User request (E2E notes).**
1. Add `…` (3-dot) menu on each note card → quick menu for folder management
   (move, rename, delete).
2. Moving a folder should recursively move all contents (notes + subfolders) —
   never leave orphans.

**Design notes.** Use `radix-ui/react-dropdown-menu` (already in deps).
Backend: new command `move_folder(from, to)` that recursively walks + calls
`save_note` for each child. SQL foreign-key cascade already handles it.

**Affected files.**
- `src/components/NoteCard.tsx` (new 3-dot menu)
- `src-tauri/src/commands/workspace.rs` (new `move_folder` command)

---

## B4 — Silent-reuse + WebKit localStorage interaction (E2E Step 1)

**Observation.** User deleted `~/Library/Application Support/com.viboai.app.dev/`
but onboarding wizard did NOT appear on relaunch, even though vault was fresh.

**Root cause.** WebKit localStorage is scoped per-bundle-identifier and lives
at `~/Library/WebKit/com.viboai.app.dev/...` — NOT in Application Support.
So `rm -rf` on the Application Support dir doesn't wipe the
`zettel-onboarding-done` flag. User had completed onboarding in a prior session,
so the flag persisted across the vault wipe.

**Status.** Not a bug — actually desired (don't re-onboard returning users
who've cleared data). Document in verification script: truly clean state
requires also clearing WebKit localStorage via
`defaults delete com.viboai.app.dev` or DevTools.

**Action.** Add note to `E2E_VERIFICATION_STEPS.md` for future cold-start tests.

---

## B5 — Reusable system messaging / notifications (Phase 10)

**Vision.** Build once, reuse everywhere:
- Toasts (success/error/warn/info)
- Top-menu event slider (Tauri events → UI)
- Inline form errors

**Pattern.** Single `useSystemMessage()` hook + Radix Toast provider at root.
Tauri events forwarded via `events::emit_system_message`.

**Defer to Phase 10** — not blocking current work.

---

## B6 — Obsidian vault path is hidden from Finder

**Observation.** User couldn't browse to
`~/Library/Application Support/com.viboai.app.dev/` via Finder (hidden dir).
They created an Obsidian vault on Desktop instead.

**Options.**
1. **Symlink** `~/Desktop/viboai-myspace` → actual path on first run.
2. **Settings UI** → "Reveal vault in Finder" button.
3. **Configurable vault location** → user picks any folder during onboarding,
   app uses that as `myspace/` root (pairs with Obsidian's vault model).

**Recommended.** Option 2 as a quick win for Phase 0.6. Option 3 for Phase 1
(matches Obsidian UX, also unlocks user-chosen iCloud/Dropbox sync without
app-level sync code).

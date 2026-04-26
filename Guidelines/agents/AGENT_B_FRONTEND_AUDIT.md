> **STATUS (2026-04-26):** Historical audit. Phase 0 + 0.7-A complete; current state lives in `Guidelines/source-of-truth/PHASE_0_COMPLETION.md`. Findings here informed the ship; left for traceability.

# Agent B: Frontend State Audit

## Current State Flow

```
Tauri Backend (Rust)
    |
    | IPC: load_workspace_snapshot()
    v
store.tsx (Hydration)
    |
    |---> setNotes(mergedNotes)       (sorted by updatedAt)
    |---> setFolders(mergedFolders)    (sorted alphabetically)
    |---> setColumns(columnsToUse)    (seed DEFAULT_COLUMNS if empty)
    |
    v
Components (useStore hook)
    |---> KanbanView        (kanbanNotes filtered by isKanban)
    |---> NotebookView      (all notes with search/filter)
    |---> NoteEditor        (selected note editing)

Persistence Flow:
    Components -> updateNote() -> persistNote() -> saveWorkspaceNote(note)
                                                     |
                                                     v
                                               IPC: save_note(note)
```

### Hydration Sequence (store.tsx lines 112-165)

1. **Init**: `setIsHydrating(true)`, set indexing status to "starting"
2. **Load**: `loadWorkspaceSnapshot()` returns `{ notes, folders, columns }`
3. **Merge**: Notes sorted by `updatedAt` descending (newest first)
4. **Seed**: If `columns.length === 0`, seed with `DEFAULT_COLUMNS` and persist each
5. **Sync**: Folders created via `createWorkspaceFolder()` for each folder
6. **Complete**: `setIsHydrating(false)`, set state with loaded data

### Persistence Patterns

| Operation | Command | Debounce |
|-----------|---------|----------|
| addNote | saveWorkspaceNote | 0ms (immediate) |
| updateNote | saveWorkspaceNote | 350ms |
| moveNote | saveWorkspaceNote | 0ms |
| deleteNote | deleteWorkspaceNote | immediate |
| createFolder | createWorkspaceFolder | immediate |

---

## Commands Analysis

| Command | Current Use | Phase 0 Status |
|-----------|-------------|----------------|
| `load_workspace_snapshot` | Initial hydration of notes/folders/columns | **Needs fs version** - must scan vault folder for .md files |
| `save_note` | Persist note changes | **Keep** - writes to vault path |
| `delete_note` | Remove note | **Keep** - deletes from vault path |
| `create_folder` | Create folder in backend | **Needs review** - may become folder metadata only |
| `save_column` | Persist kanban column | **Keep** - vault metadata |
| `delete_column` | Remove column | **Keep** - vault metadata |
| `setup_secure_vault` | Initial vault creation | **Keep** - sets up encryption keys |
| `unlock_vault` | Unlock encrypted vault | **Keep** - unlock encryption |
| `lock_vault` | Lock vault | **Keep** - lock encryption |
| `is_vault_configured` | Check if vault exists | **Keep** |
| `is_vault_unlocked` | Check vault status | **Keep** |
| `export_notes` | Export all notes to JSON | **Keep** - reads from filesystem |
| `factory_reset` | Clear all data | **Keep** - clears vault |

---

## New Commands Needed

### 1. `select_vault_path()`
**Purpose**: Open native folder picker dialog  
**Frontend Use**: OnboardingWizard vault path selection step  
**Returns**: `Promise<string | null>` - selected path or cancelled  
**Related**: Already using `@tauri-apps/plugin-dialog` in SettingsView.tsx for export

### 2. `init_vault(path: string)`
**Purpose**: Initialize vault at selected path  
**Frontend Use**: After folder selection in onboarding  
**Behavior**:
- Create `.vibo/` subdirectory (or similar)
- Set up metadata files
- Create initial folder structure
- Initialize encryption keys
- Return vault metadata

### 3. `list_notes_fs(folder?: string)`
**Purpose**: Scan filesystem for .md files  
**Frontend Use**: Replace `load_workspace_snapshot` note loading  
**Parameters**:
- `folder`: Optional subfolder to scan (default: vault root)
- Recursive scan for `.md` files
**Returns**: `Promise<Note[]>` - parsed notes from filesystem

### 4. `list_folders_fs()`
**Purpose**: List all subdirectories in vault  
**Frontend Use**: Populate folder list in NotebookView  
**Returns**: `Promise<string[]>` - folder names

### 5. `move_note_file(noteId, fromFolder, toFolder)`
**Purpose**: Move note file between folders  
**Frontend Use**: `moveNoteToFolder()` in store.tsx  
**Alternative**: Could be handled by `save_note` with updated folder path

---

## OnboardingWizard Gap Analysis

### Current Flow (5 Steps)

1. **Welcome** - App intro, "Begin setup" button
2. **Model Package** - AI model selection (Instruct/Thinking pack)
3. **Integrations** - Calendar/Gmail connections
4. **Security** - Auth method (biometrics/PIN/passphrase)
5. **Name** - User name, finish setup

### Missing: Vault Path Selection

**Where to insert**: Between Step 1 (Welcome) and Step 2 (Model Package)

**New Step Requirements**:

```typescript
// New state in OnboardingWizard.tsx
const [vaultPath, setVaultPath] = useState<string | null>(null);
const [isSelectingPath, setIsSelectingPath] = useState(false);

// New Step Component
function VaultPathStep({ path, onSelect, onNext, onBack }: VaultPathStepProps) {
  const handleSelectFolder = async () => {
    setIsSelectingPath(true);
    try {
      const selected = await invoke<string | null>("select_vault_path");
      if (selected) {
        onSelect(selected);
      }
    } finally {
      setIsSelectingPath(false);
    }
  };
  
  return (
    <div className="vault-path-step">
      <h2>Choose vault location</h2>
      <p>Select a folder to store your encrypted notes</p>
      
      {path ? (
        <div className="selected-path">{path}</div>
      ) : (
        <button onClick={handleSelectFolder}>
          {isSelectingPath ? "Opening..." : "Select Folder"}
        </button>
      )}
      
      <NavArrows onBack={onBack} onNext={onNext} />
    </div>
  );
}
```

**OnboardingConfig Changes**:
```typescript
export interface OnboardingConfig {
  userName: string;
  tone: string;
  localModel: string;
  cloudFallback: string;
  integrations: string[];
  authMethod: "biometrics" | "pin" | "passphrase";
  cloudProviders: string[];
  torEnabled: boolean;
  vaultPath: string;  // <-- ADD THIS
}
```

**Finish Handler Update**:
```typescript
const finish = async (name: string) => {
  // Initialize vault before completing
  if (vaultPath) {
    await invoke("init_vault", { path: vaultPath });
  }
  
  // ... rest of finish logic
};
```

---

## Filesystem Access Audit

### Node.js fs Usage
```bash
# Grep results for any Node fs usage
grep -r "from ['\"]fs" src/ || echo "None found"
# Result: None found

grep -r "require.*fs" src/ || echo "None found"  
# Result: None found

grep -r "import.*from ['\"]node:fs" src/ || echo "None found"
# Result: None found

grep -r "import.*from ['\"]fs/promises" src/ || echo "None found"
# Result: None found
```

**Conclusion**: No direct Node.js filesystem access in frontend code.

### Tauri Dialog Usage

**Current Usage**: SettingsView.tsx (lines 88-104)
```typescript
const [json, { save }, { writeTextFile }] = await Promise.all([
  exportNotesCmd(),
  import("@tauri-apps/plugin-dialog"),
  import("@tauri-apps/plugin-fs"),
]);
const filePath = await save({
  defaultPath: "zettelkasten-export.json",
  filters: [{ name: "JSON", extensions: ["json"] }],
});
if (filePath) {
  await writeTextFile(filePath, json);
}
```

**Pattern for Vault Path Selection**:
- Use `@tauri-apps/plugin-dialog` with `open()` (not `save()`)
- Set `directory: true` option
- Call via Tauri command (backend handles the dialog)

---

## Component Impact Assessment

| Component | Impact Level | Changes Needed |
|-----------|--------------|----------------|
| **OnboardingWizard** | **High** | Add vault path selection step; new `vaultPath` state; call `init_vault` on finish |
| **store.tsx** | **High** | Change `loadWorkspaceSnapshot` to `list_notes_fs`; update hydration logic; handle vault path in context |
| **SettingsView** | **Medium** | Add "Change Vault Location" option; reuse dialog pattern from export |
| **KanbanView** | **Low** | No changes - uses store, unaware of data source |
| **NotebookView** | **Low** | No changes - uses store, unaware of data source |
| **NoteEditor** | **Low** | No changes - uses store, unaware of data source |
| **commands.ts** | **High** | Add new IPC commands; update type definitions |
| **types.ts** | **Low** | Add `vaultPath` to config types if needed |

### Detailed Component Changes

#### OnboardingWizard.tsx
**Lines to modify**: 597-598 (step state), 649-672 (finish function)

**New Step** (insert at line 678):
```typescript
{step === "vault" && (
  <VaultPathStep
    path={vaultPath}
    onSelect={setVaultPath}
    onNext={() => setStep("model")}
    onBack={() => setStep("welcome")}
  />
)}
```

#### store.tsx
**Lines to modify**: 112-165 (hydration effect)

**New Pattern**:
```typescript
const hydrate = async () => {
  try {
    // Phase 0: Scan filesystem instead of loading snapshot
    const notes = await listNotesFs();  // New command
    const folders = await listFoldersFs();  // New command
    const columns = await loadColumns();  // Keep existing or new command
    
    // ...rest of hydration logic
  }
};
```

#### commands.ts
**New Commands to Add** (after line 190):
```typescript
export async function selectVaultPath(): Promise<string | null> {
  return tauriInvoke<string | null>("select_vault_path");
}

export async function initVault(path: string): Promise<void> {
  return tauriInvoke<void>("init_vault", { path });
}

export async function listNotesFs(folder?: string): Promise<Note[]> {
  return tauriInvoke<Note[]>("list_notes_fs", { folder });
}

export async function listFoldersFs(): Promise<string[]> {
  return tauriInvoke<string[]>("list_folders_fs");
}
```

---

## Migration Strategy

### Phase 0: Filesystem-First Migration

**Step 1**: Add new commands to `commands.ts`
- `selectVaultPath()` - for folder picker
- `initVault()` - for vault initialization
- `listNotesFs()` - for scanning .md files
- `listFoldersFs()` - for folder discovery

**Step 2**: Modify `OnboardingWizard.tsx`
- Insert vault path selection step
- Call `initVault()` before completing onboarding
- Store vault path in `OnboardingConfig`

**Step 3**: Modify `store.tsx`
- Replace `loadWorkspaceSnapshot()` with filesystem scan
- Keep same React state structure (notes, folders, columns)
- Update persistence to write to filesystem

**Step 4**: Test backward compatibility
- Ensure existing vaults still load
- Graceful migration path for old data

### Backward Compatibility Notes

Current `load_workspace_snapshot` returns:
```typescript
interface WorkspaceSnapshot {
  notes: Note[];
  folders: string[];
  columns: KanbanColumn[];
}
```

Phase 0 will build this from:
- `list_notes_fs()` -> `snapshot.notes`
- `list_folders_fs()` -> `snapshot.folders`
- `load_columns()` (or keep existing) -> `snapshot.columns`

The store.tsx hydration logic remains the same - only the source changes.

---

## Summary

The frontend architecture is well-suited for Phase 0 migration:

1. **Clean separation**: Components use store, store uses commands, commands use Tauri IPC
2. **No direct fs access**: All filesystem operations go through Tauri commands
3. **Centralized hydration**: Single point of change in store.tsx effect
4. **Clear extension points**: OnboardingWizard has clear step structure for adding vault selection

**Key files to modify**:
1. `src/lib/commands.ts` - Add new IPC commands
2. `src/components/OnboardingWizard.tsx` - Add vault path selection step
3. `src/lib/store.tsx` - Change hydration to use filesystem commands

**Estimated effort**: Medium (2-3 days) - mostly adding new commands and one new UI step.

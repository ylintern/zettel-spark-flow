# Agent A: Rust Architecture Audit

## Executive Summary

This audit analyzes the Rust codebase in `src-tauri/src/` to prepare for Phase 0 "Filesystem-First" migration. The current architecture uses SQLite as the primary source of truth with filesystem storage as a secondary concern. The goal is to reverse this: filesystem becomes primary, SQLite becomes optional/cache.

**Audit Date:** 2026-04-15
**Total Rust Files:** 16 source files
**Database Tables:** 4 core tables
**Critical Dependencies:** sqlx (SQLite), AES-GCM (encryption), Stronghold (secrets)

---

## Current Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (TSX)                                  │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  StoreProvider (src/lib/store.tsx)                                    │   │
│  │  - React Context for app state                                        │   │
│  │  - Local debounced persistence timers                                 │   │
│  │  - Agent notes stored in localStorage                                 │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ invoke()
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           COMMANDS LAYER                                     │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  commands/workspace.rs                                              │   │
│  │  ├─ load_workspace_snapshot() ──────► db::load_workspace_snapshot() │   │
│  │  ├─ save_note() ────────────────────► db::save_note()              │   │
│  │  ├─ delete_note() ──────────────────► db::delete_note()            │   │
│  │  ├─ create_folder() ────────────────► db::create_folder()         │   │
│  │  ├─ save_column() ──────────────────► db::save_column()           │   │
│  │  └─ delete_column() ────────────────► db::delete_column()         │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ calls
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DATABASE LAYER (Primary)                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  db/mod.rs                                                          │   │
│  │  ├─ SQLite with sqlx (WAL mode)                                    │   │
│  │  ├─ Tables: notes, note_tags, folders, columns                     │   │
│  │  └─ init_pool(), migrate()                                         │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ delegates file I/O
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         VAULT LAYER (Secondary)                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  vault/mod.rs                                                       │   │
│  │  ├─ Manual YAML string building (NOT parsed)                       │   │
│  │  ├─ write_note() ──────► creates markdown files                    │   │
│  │  ├─ read_note() ───────► reads markdown, extracts body            │   │
│  │  └─ Encryption/decryption via security layer                         │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      │ filesystem
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        FILESYSTEM (myspace/viboai/)                          │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  ~/Library/Application Support/com.viboai.zettel-spark-flow/         │   │
│  │  └── viboai/myspace/                                                │   │
│  │      ├── notes/        (*.md with YAML frontmatter)                 │   │
│  │      └── tasks/        (*.md for kanban items)                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Observation: Source of Truth is Inverted

The current architecture treats **SQLite as the source of truth** and filesystem as a side-effect. This is the opposite of what we want:

- **Current:** DB query → get file_path → read file → return content
- **Target:** Read file → parse metadata → return content (DB optional for indexing)

---

## SQLite Schema Analysis

### Complete Table Inventory

| Table | Purpose | Columns | Phase 0 Fate | Notes |
|-------|---------|---------|--------------|-------|
| **notes** | Primary note metadata storage | id, title, file_path, folder, column_id, position, kind, created_at, updated_at, is_encrypted | **Remove** | All data can be derived from filesystem |
| **note_tags** | Many-to-many tag relationships | note_id, tag | **Remove** | Tags stored in YAML frontmatter |
| **folders** | Folder registry for UI | name, created_at | **Remove** | Can scan directories |
| **columns** | Kanban board columns | id, title, order, created_at | **Keep (Migrate)** | UI-only state, no filesystem representation |

### Detailed Schema

```sql
-- notes table (PRIMARY ENTITY)
CREATE TABLE notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  file_path TEXT NOT NULL UNIQUE,  -- Path relative to vault_dir
  folder TEXT,                      -- Virtual folder for organization
  column_id TEXT NOT NULL DEFAULT 'inbox',  -- Kanban column reference
  position INTEGER NOT NULL DEFAULT 0,      -- Sort order in column
  kind TEXT NOT NULL CHECK (kind IN ('note', 'task')),
  created_at TEXT NOT NULL,         -- ISO 8601 timestamp
  updated_at TEXT NOT NULL,         -- ISO 8601 timestamp
  is_encrypted INTEGER NOT NULL DEFAULT 0
);

-- note_tags table (RELATIONSHIP)
CREATE TABLE note_tags (
  note_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  PRIMARY KEY (note_id, tag),
  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- folders table (REGISTRY)
CREATE TABLE folders (
  name TEXT PRIMARY KEY,
  created_at TEXT NOT NULL
);

-- columns table (UI STATE)
CREATE TABLE columns (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  "order" INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);
```

### SQLite Dependency Analysis

**Critical SQL Dependencies in `db/mod.rs`:**

1. **CRUD Operations:**
   - `INSERT INTO notes ... ON CONFLICT(id) DO UPDATE` (upsert)
   - `SELECT ... FROM notes ORDER BY updated_at DESC` (query with sort)
   - `DELETE FROM notes WHERE id = ?` (delete)
   - `DELETE FROM note_tags WHERE note_id = ?` (cascade delete)

2. **Join/Relationship Queries:**
   - Tags loaded separately via `load_tags()` function
   - No complex JOINs currently (simplifies migration)

3. **Transaction Scope:**
   - Individual operations (no multi-table transactions)
   - WAL mode enabled (good for concurrent access)

**Migration Complexity: MEDIUM**
- No complex queries to migrate
- Simple CRUD patterns
- Foreign key relationships exist but are manageable

---

## Vault Module Analysis

### Current Implementation: `vault/mod.rs`

#### What's Working

1. **File Organization:**
   ```rust
   notes/{note_id}.md      // Regular notes
   tasks/{note_id}.md      // Kanban items
   ```

2. **YAML Frontmatter Format (Obsidian-compatible):**
   ```yaml
   ---
   id: "note-uuid"
   type: "note" | "task"
   title: "Note Title"
   column: "inbox"
   position: 1234567890
   is_encrypted: false
   created: "2026-04-15T10:00:00.000Z"
   modified: "2026-04-15T10:00:00.000Z"
   folder: "Projects"  # or null
   tags:
     - "tag1"
     - "tag2"
   ---
   ```

3. **Encryption Integration:**
   - Content encrypted via AES-256-GCM
   - Encrypted payload stored with prefix: `!vibo-encrypted:v1:`
   - Nonce + ciphertext base64 encoded
   - Works at content level, not file level

#### What's Broken / Technical Debt

1. **Manual YAML String Building (CRITICAL):**
   ```rust
   // Current: String concatenation
   fn yaml_string(value: &str) -> String {
       format!("{value:?}")  // Uses Debug trait quoting
   }
   
   // PROBLEM: Not real YAML serialization
   // - No escaping rules beyond Rust's Debug
   // - No YAML library validation
   // - Could produce invalid YAML with special characters
   ```

2. **No YAML Parsing (CRITICAL):**
   ```rust
   // Current: Simple string extraction
   fn extract_body(markdown: &str) -> String {
       if let Some(stripped) = markdown.strip_prefix("---\n") {
           if let Some(idx) = stripped.find("\n---\n") {
               return stripped[(idx + 5)..].trim_start_matches('\n').to_string();
           }
       }
       markdown.to_string()
   }
   ```
   - **NO METADATA PARSING** - Cannot read YAML back into structured data
   - **Only extracts body** - Frontmatter is discarded on read
   - This is why DB is required - can't reconstruct note from file alone

3. **Missing Features:**
   - No folder nesting (flat structure)
   - No file watching
   - No conflict resolution
   - No sync support

### Required Changes for Filesystem-First

```rust
// NEW: Proper YAML serialization with serde_yaml
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct NoteFrontmatter {
    id: String,
    #[serde(rename = "type")]
    note_type: String,
    title: String,
    column: String,
    position: i64,
    is_encrypted: bool,
    created: String,
    modified: String,
    folder: Option<String>,
    tags: Vec<String>,
}

// Read note from filesystem
fn read_note_fs(path: &Path) -> Result<WorkspaceNote> {
    let content = fs::read_to_string(path)?;
    let (frontmatter, body) = parse_frontmatter(&content)?;
    let metadata: NoteFrontmatter = serde_yaml::from_str(&frontmatter)?;
    
    Ok(WorkspaceNote {
        id: metadata.id,
        title: metadata.title,
        content: decrypt_if_needed(body, metadata.is_encrypted)?,
        tags: metadata.tags,
        column: metadata.column,
        position: metadata.position,
        is_kanban: metadata.note_type == "task",
        created_at: metadata.created,
        updated_at: metadata.modified,
        folder: metadata.folder,
        is_encrypted: Some(metadata.is_encrypted),
    })
}
```

---

## Module Dependencies Map

```
lib.rs
├── commands/
│   ├── mod.rs                    (workspace module re-export)
│   └── workspace.rs              [DEPENDS ON: db, models, state]
│
├── db/
│   └── mod.rs                    [DEPENDS ON: models, security, vault]
│       - init_pool()             [SQLITE]
│       - migrate()               [SQLITE]
│       - load_workspace_snapshot() [SQLITE PRIMARY]
│       - save_note()             [SQLITE PRIMARY + vault]
│       - delete_note()           [SQLITE PRIMARY + vault]
│       - create_folder()         [SQLITE]
│       - save_column()           [SQLITE]
│       - delete_column()         [SQLITE]
│       - init_default_folders()  [SQLITE]
│
├── vault/
│   └── mod.rs                    [DEPENDS ON: models, security, events]
│       - write_note()            [FILESYSTEM]
│       - read_note()             [FILESYSTEM]
│       - delete_note()           [FILESYSTEM]
│       - render_markdown()       [STRING BUILDING]
│       - extract_body()          [STRING EXTRACTION]
│
├── models/
│   ├── mod.rs                    [NO DEPENDENCIES - Pure Data]
│   │   - WorkspaceNote
│   │   - KanbanColumn
│   │   - WorkspaceSnapshot
│   │   - CallerContext
│   ├── manifest.rs               [NO DEPENDENCIES]
│   └── manager.rs                [NO DEPENDENCIES]
│
├── security/
│   ├── mod.rs                    [DEPENDS ON: db, events, vault]
│   │   - SecurityState           [STRONGHOLD + AES-GCM]
│   │   - encrypt_note_content()
│   │   - decrypt_note_content()
│   │   - factory_reset()         [CALLS: vault::reset_vault_dir, db::delete_database_files]
│   └── biometric.rs              [NO DB DEPS]
│
├── state.rs                      [DEPENDS ON: sqlx, security]
│   - AppState                    [HOLDS: SqlitePool]
│
├── events/
│   └── mod.rs                    [NO DEPENDENCIES]
│
├── providers/
│   └── mod.rs                    [DEPENDS ON: state]
│
└── services/
    ├── mod.rs
    ├── context.rs                [NO DB DEPS - Trait definitions]
    └── retrieval.rs              [MARKED: "TODO: SQLite queries"]
```

---

## Migration Path: Phase 0 Filesystem-First

### Step 1: Create New `fs_vault/` Module Structure

```
src/
├── fs_vault/
│   ├── mod.rs              # Module exports
│   ├── scanner.rs          # Directory traversal, file discovery
│   ├── parser.rs           # YAML frontmatter parsing (serde_yaml)
│   ├── writer.rs           # YAML serialization
│   ├── indexer.rs          # Optional: Build SQLite index
│   └── watcher.rs          # File system watching (notify crate)
```

### Step 2: Function Migration Matrix

| Current Function | Current Location | New Location | Priority |
|------------------|------------------|--------------|----------|
| `load_workspace_snapshot` | db/mod.rs | fs_vault/scanner.rs | **P0** |
| `write_note` | vault/mod.rs | fs_vault/writer.rs | **P0** |
| `read_note` | vault/mod.rs | fs_vault/parser.rs | **P0** |
| `delete_note` | db/mod.rs | fs_vault/mod.rs | **P0** |
| `render_markdown` | vault/mod.rs | fs_vault/writer.rs | **P0** |
| `extract_body` | vault/mod.rs | fs_vault/parser.rs | **P0** |
| `save_column` | db/mod.rs | db/mod.rs (keep) | P1 |
| `delete_column` | db/mod.rs | db/mod.rs (keep) | P1 |
| `create_folder` | db/mod.rs | fs_vault/scanner.rs | P1 |
| `load_tags` | db/mod.rs | fs_vault/parser.rs | P1 |
| `save_note` | db/mod.rs | fs_vault/writer.rs | **P0** |
| `init_default_folders` | db/mod.rs | fs_vault/scanner.rs | P1 |

### Step 3: Update Commands Layer

**Current Pattern (DB-Centric):**
```rust
#[tauri::command]
pub async fn save_note(note: WorkspaceNote, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db().map_err(|err| err.to_string())?;
    db::save_note(&db, &state.myspace_dir, &note, &state.security)
        .await
        .map_err(|err| err.to_string())
}
```

**Target Pattern (Filesystem-Centric):**
```rust
#[tauri::command]
pub async fn save_note(note: WorkspaceNote, state: State<'_, AppState>) -> Result<(), String> {
    // Primary: Write to filesystem
    fs_vault::write_note(&state.myspace_dir, &note, &state.security)
        .map_err(|err| err.to_string())?;
    
    // Secondary: Update index (optional, async)
    if let Some(index) = state.index().await {
        index.upsert_note(&note).await.ok(); // Non-critical
    }
    
    Ok(())
}
```

### Step 4: Deprecate `db/` Module

**Option A: Remove Completely (Aggressive)**
- Delete `db/mod.rs`
- Remove `sqlx` from dependencies
- Update all imports

**Option B: Convert to Cache (Conservative)**
- Rename `db/` to `cache/`
- Keep for fast queries/columns
- Optional index that can be rebuilt from FS

**Recommendation: Option B**
- Columns table has no FS representation
- Fast search requires indexing
- Allows gradual migration

---

## Risk Assessment

### High Risk Areas

| Risk | Description | Mitigation |
|------|-------------|------------|
| **Data Loss** | Migration could corrupt notes | Full backup before migration, checksum validation |
| **YAML Parsing** | Current manual YAML may have edge cases | Property-based testing, round-trip validation |
| **Encryption** | Encrypted notes need special handling | Test with encrypted fixtures, verify decrypt round-trip |
| **Column State** | Kanban columns only in DB | Must preserve or migrate to config file |

### Medium Risk Areas

| Risk | Description | Mitigation |
|------|-------------|------------|
| **Performance** | FS scan slower than DB query | Directory caching, async scanning, incremental updates |
| **Tag Consistency** | Tags in YAML vs DB | Single source of truth (YAML), DB as index only |
| **Folder Sync** | Folders in DB vs FS directories | Generate folders from directory scan |

### Low Risk Areas

| Risk | Description | Mitigation |
|------|-------------|------------|
| **UI Breakage** | Command API unchanged | Keep same command signatures |
| **Security** | Encryption logic stays same | No changes to crypto code |
| **Events** | Progress events still work | Emit events from new fs_vault module |

### Breaking Changes Required

1. **None in Public API** - Command signatures remain identical
2. **State Structure** - `AppState.db()` may become optional
3. **Error Messages** - May change (file-not-found vs db-error)
4. **Timing** - FS operations may be slower (acceptable)

---

## Testing Requirements

### Critical Test Scenarios

```rust
#[cfg(test)]
mod fs_vault_tests {
    // 1. Round-trip preservation
    #[test]
    fn note_round_trip_preserves_all_fields() {
        let original = create_test_note();
        let path = write_note(&temp_dir, &original, &mock_security).unwrap();
        let restored = read_note(&temp_dir, &path, &mock_security).unwrap();
        assert_eq!(original, restored);
    }
    
    // 2. Encrypted note round-trip
    #[test]
    fn encrypted_note_round_trip() {
        let mut note = create_test_note();
        note.is_encrypted = Some(true);
        // ... write, read, verify decrypt
    }
    
    // 3. Special characters in YAML
    #[test]
    fn yaml_special_characters_escaped() {
        let note = create_note_with_content("Has: colon and \"quotes\"");
        // ... verify valid YAML produced
    }
    
    // 4. Directory scanning
    #[test]
    fn scan_finds_all_notes() {
        // Create 100 notes
        // Verify scan finds all
        // Verify order correct
    }
    
    // 5. Concurrent writes
    #[test]
    fn concurrent_writes_no_corruption() {
        // Multiple threads writing different notes
        // Verify all persisted correctly
    }
}
```

### Integration Tests

1. **Cold Start Test** (exists in db/mod.rs:369)
   - Must work with FS-first
   - No SQLite dependency

2. **Migration Test**
   - Existing DB -> FS-first
   - Verify no data loss

3. **Large Dataset Test**
   - 1000+ notes
   - Performance benchmarks

---

## Dependencies to Add/Remove

### Add to Cargo.toml

```toml
[dependencies]
# YAML parsing/serialization
serde_yaml = "0.9"

# File system watching (optional Phase 0+)
notify = "6.1"

# Path utilities
walkdir = "2.5"

# DateTime for file metadata
chrono = { version = "0.4", features = ["serde"] }  # Already present
```

### Remove from Cargo.toml (Phase 2)

```toml
# After full migration:
# sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
# tauri-plugin-sql = { version = "2.0.0-rc", features = ["sqlite"] }
```

---

## Summary & Recommendations

### Phase 0 Implementation Order

1. **Week 1:** Create `fs_vault/` module skeleton
   - Implement proper YAML parsing with serde_yaml
   - Implement scanner for directory traversal
   - Unit tests for round-trip preservation

2. **Week 2:** Migrate write path
   - Move `write_note()` to `fs_vault/writer.rs`
   - Keep DB writes as secondary (dual-write)
   - Verify file output matches expected format

3. **Week 3:** Migrate read path
   - Implement `load_workspace_snapshot()` from FS
   - Add directory watcher for changes
   - Compare performance with DB approach

4. **Week 4:** Deprecate DB for notes
   - Stop writing to `notes` table
   - Keep DB for `columns` table
   - Migration script for existing users

### Files Requiring Modification

| File | Lines | Change Type |
|------|-------|-------------|
| `vault/mod.rs` | 200 | **DELETE** (replaced by fs_vault) |
| `db/mod.rs` | 429 | **MODIFY** (remove note ops) |
| `commands/workspace.rs` | 168 | **MODIFY** (update imports) |
| `state.rs` | 48 | **MODIFY** (optional DB pool) |
| `lib.rs` | 161 | **MODIFY** (add fs_vault module) |
| **NEW:** `fs_vault/mod.rs` | - | **CREATE** |
| **NEW:** `fs_vault/parser.rs` | - | **CREATE** |
| **NEW:** `fs_vault/writer.rs` | - | **CREATE** |
| **NEW:** `fs_vault/scanner.rs` | - | **CREATE** |
| `Cargo.toml` | 83 | **MODIFY** (add serde_yaml) |

### Success Criteria

- [ ] All existing tests pass
- [ ] New round-trip tests pass
- [ ] File output matches Obsidian-compatible YAML
- [ ] Performance within 20% of DB approach
- [ ] No SQLite dependency for read operations
- [ ] Backward compatibility with existing vaults

---

## Appendix: File Inventory

### Rust Source Files (16 total)

| File | Purpose | Size | SQLite Usage |
|------|---------|------|--------------|
| `lib.rs` | App entry, module tree | 161 lines | Pool init, migration |
| `main.rs` | Binary entry | 6 lines | None |
| `state.rs` | AppState with DB pool | 48 lines | **Core** |
| `db/mod.rs` | All DB operations | 429 lines | **Core** - All operations |
| `vault/mod.rs` | File I/O, YAML render | 200 lines | None (called by db) |
| `commands/mod.rs` | Command re-exports | 1 line | None |
| `commands/workspace.rs` | IPC handlers | 168 lines | Calls db::* |
| `models/mod.rs` | Data structures | 59 lines | None |
| `models/manifest.rs` | Model manifest (unused) | 167 lines | None |
| `models/manager.rs` | Model manager (unused) | 197 lines | None |
| `security/mod.rs` | Encryption, vault | 521 lines | Imports db for factory_reset |
| `security/biometric.rs` | Biometric unlock | 350 lines | None |
| `providers/mod.rs` | Cloud AI providers | 317 lines | None |
| `events/mod.rs` | Tauri events | 58 lines | None |
| `services/mod.rs` | Service re-exports | 2 lines | None |
| `services/context.rs` | Context traits | 115 lines | None |
| `services/retrieval.rs` | SQL-based RAG (TODO) | 145 lines | **Planned** - not implemented |

**Total SQLite-dependent lines:** ~650 lines in `db/mod.rs` + `state.rs`

---

*Report generated for Phase 0 Filesystem-First Migration*
*Target completion: 4 weeks*
*Risk level: MEDIUM (complex but well-isolated)*

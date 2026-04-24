# VIBO AI — Development Roadmap
`source of truth · rev 1.0 · engineering lead sign-off april 2026`

> This document is the canonical build plan. It updates only when architectural decisions change after deliberate discussion. All phases are sequential. No phase begins until the previous phase passes its completion gate.

---

## CONVENTIONS

```
[ ] = not started
[~] = in progress
[x] = complete
[!] = blocked / needs decision
[s] = skipped / deferred
```

**Test notation**
- `SMOKE` = manual checklist, runs in < 5 min
- `UNIT` = automated Rust `#[test]` or `#[tokio::test]`
- `INT` = integration test, requires running app
- `SCRIPT` = shell / bun test script
- `AUDIT` = engineering lead review gate — must be signed off before next phase

---

## PHASE MAP

```
PHASE 0  Foundation & Core Data Layer
  └── Stage 0.1  Project scaffold
  └── Stage 0.2  Database schema + migrations
  └── Stage 0.3  Notes CRUD (MD files + SQLite)
  └── Stage 0.4  Kanban tasks CRUD + status transitions
  └── Stage 0.5  Security layer (biometrics + passphrase)
  └── Stage 0.6  Settings persistence
  └── AUDIT 0    Phase gate

PHASE 1  Inference Integration
  └── Stage 1.1  tauri-plugin-leap-ai setup
  └── Stage 1.2  Model catalog + download + management
  └── Stage 1.3  Onboarding flow
  └── Stage 1.4  Inline chat + token streaming
  └── Stage 1.5  Basic SQL conversation memory
  └── Stage 1.6  App lifecycle (suspend/resume)
  └── AUDIT 1    Phase gate

PHASE 2  Cloud Model Integration
  └── Stage 2.1  Provider registry architecture
  └── Stage 2.2  API key management (encrypted)
  └── Stage 2.3  OAuth flows (Google)
  └── Stage 2.4  Model routing + switching UI
  └── Stage 2.5  Privacy + routing policy
  └── AUDIT 2    Phase gate

PHASE 3  Swiftide RAG Pipeline
  └── Stage 3.1  FastEmbed setup + model bundle
  └── Stage 3.2  Swiftide indexing pipeline
  └── Stage 3.3  velesdb integration
  └── Stage 3.4  retrieval.rs (digestion path)
  └── Stage 3.5  Context injection into chat
  └── Stage 3.6  RAG skills + full-text search
  └── AUDIT 3    Phase gate

PHASE 4  Swiftide Agentic Layer
  └── Stage 4.1  LeapBridge trait implementation
  └── Stage 4.2  swiftide-agents + tool definitions
  └── Stage 4.3  Nano background agent + task queue
  └── Stage 4.4  External integrations (MCP + REST)
  └── Stage 4.5  Agentic chat loop + tool call detection
  └── Stage 4.6  Context window management + loop guards
  └── AUDIT 4    Phase gate

PHASE 5  Polish, Hardening & Future Extensibility
  └── Stage 5.1  Export, backup, offline handling
  └── Stage 5.2  Notifications + reminders
  └── Stage 5.3  Plugin registry architecture
  └── Stage 5.4  Performance profiling + mobile hardening
  └── Stage 5.5  Accessibility + privacy settings
  └── AUDIT 5    Phase gate
```

---

## PHASE 0 — Foundation & Core Data Layer

**Goal:** A working Tauri app on all 5 targets (iOS, Android, macOS, Windows, Linux) with full notes + tasks CRUD, persistent storage, and security unlock. No AI yet. No ambiguity about where data lives.

**Prerequisites:**
- Rust toolchain installed (rustup)
- Bun installed
- Xcode + iOS simulator (for iOS)
- Android Studio + NDK (for Android)
- Node/Bun dependencies confirmed

---

### Stage 0.1 — Project Scaffold

**Goal:** Tauri 2.0 project with Bun frontend, compiles clean on all targets.

#### Task 0.1.1 — Initialize project

```
Steps:
  1. bun create tauri-app vibo --template react-ts
  2. cd vibo
  3. Edit package.json: replace vite with bun dev server config
  4. bun install
  5. Verify: bun run tauri dev → app opens on desktop

File structure target:
  vibo/
  ├── src/                    # TSX frontend
  │   ├── components/
  │   │   ├── ui/             # shadcn/ui components
  │   │   ├── notes/
  │   │   ├── tasks/
  │   │   └── chat/
  │   ├── hooks/
  │   ├── lib/
  │   │   ├── commands.ts     # all invoke() wrappers — typed
  │   │   └── events.ts       # all listen() wrappers — typed
  │   ├── store/              # Zustand or jotai state
  │   ├── App.tsx
  │   └── main.tsx
  ├── src-tauri/
  │   ├── src/
  │   │   ├── lib.rs          # plugin registration + command registration
  │   │   ├── commands/
  │   │   │   ├── mod.rs
  │   │   │   ├── notes.rs
  │   │   │   ├── tasks.rs
  │   │   │   ├── security.rs
  │   │   │   └── settings.rs
  │   │   ├── db/
  │   │   │   ├── mod.rs
  │   │   │   ├── migrations.rs
  │   │   │   └── schema.rs
  │   │   ├── vault/
  │   │   │   └── mod.rs      # shared file I/O functions
  │   │   └── models/         # Rust structs matching DB schema
  │   ├── capabilities/
  │   │   └── default.json
  │   ├── build.rs
  │   └── tauri.conf.json
  ├── models/                 # bundled ONNX model (Phase 3)
  ├── bunfig.toml
  └── package.json
```

#### Task 0.1.2 — Configure shadcn/ui

```
Steps:
  1. bunx shadcn@latest init
  2. Select: TypeScript, Tailwind CSS, default style
  3. Add components: button, input, textarea, card, dialog,
                     dropdown-menu, badge, separator, toast
  4. Verify: import { Button } from "@/components/ui/button" works
```

#### Task 0.1.3 — Configure mobile targets

```
Steps:
  1. bun run tauri android init
  2. bun run tauri ios init   (macOS only)
  3. Verify tauri.conf.json has bundle.identifier set
  4. Verify src-tauri/gen/ directories created for both platforms
  5. SMOKE: bun run tauri ios dev → app opens on simulator
  6. SMOKE: bun run tauri android dev → app opens on emulator
```

#### Task 0.1.4 — Capabilities baseline

```
src-tauri/capabilities/default.json:
{
  "identifier": "default",
  "description": "Default capabilities",
  "platforms": ["linux","macOS","windows","iOS","android"],
  "permissions": [
    "core:default",
    "sql:default",
    "fs:default"
  ]
}

Note: leap-ai:default added in Phase 1
      stronghold:default added in Phase 2 (or 0.5 for biometrics)
```

#### Task 0.1.5 — Typed command bridge (commands.ts)

```typescript
// src/lib/commands.ts
// ALL invoke() calls live here. Frontend never calls invoke() directly.
// This file is the typed contract between TSX and Rust.

import { invoke } from "@tauri-apps/api/core"

// Notes
export const createNote = (title: string, content: string, tags: string[]) =>
  invoke<string>("create_note", { title, content, tags })

export const readNote = (id: string) =>
  invoke<Note>("read_note", { id })

export const updateNote = (id: string, content: string, tags?: string[]) =>
  invoke<void>("update_note", { id, content, tags })

export const deleteNote = (id: string) =>
  invoke<void>("delete_note", { id })

export const listNotes = (filter?: NoteFilter) =>
  invoke<Note[]>("list_notes", { filter })

// Tasks
export const createTask = (payload: CreateTaskPayload) =>
  invoke<string>("create_task", { payload })

export const updateTask = (id: string, patch: TaskPatch) =>
  invoke<void>("update_task", { id, patch })

export const moveTask = (id: string, status: TaskStatus) =>
  invoke<void>("move_task", { id, status })

export const deleteTask = (id: string) =>
  invoke<void>("delete_task", { id })

export const listTasks = (projectId?: string) =>
  invoke<Task[]>("list_tasks", { projectId })

// Types defined in src/lib/types.ts
```

**SMOKE 0.1:**
- [ ] `bun run tauri dev` opens app on desktop
- [ ] `bun run tauri ios dev` opens on iOS simulator
- [ ] `bun run tauri android dev` opens on Android emulator
- [ ] No TypeScript errors on commands.ts

---

### Stage 0.2 — Database Schema + Migrations

**Goal:** SQLite schema established, migrated on first launch, never breaks on upgrade.

#### Task 0.2.1 — Add tauri-plugin-sql

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

#### Task 0.2.2 — Schema definition

```sql
-- src-tauri/src/db/schema.sql
-- Run at app startup via migration system

CREATE TABLE IF NOT EXISTS migrations (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TEXT NOT NULL
);

-- V1
CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  file_path TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  tags TEXT NOT NULL DEFAULT '[]',  -- JSON array
  is_secure INTEGER NOT NULL DEFAULT 0,
  project_id TEXT
);

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  color TEXT NOT NULL DEFAULT '#6366f1',
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'todo',  -- todo | in_progress | done | archived
  due_date TEXT,
  project_id TEXT,
  position INTEGER NOT NULL DEFAULT 0,  -- kanban ordering
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS conversation_sessions (
  id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  last_active TEXT NOT NULL,
  leap_conversation_id TEXT,
  model_id TEXT NOT NULL,
  model_source TEXT NOT NULL DEFAULT 'local'  -- local | cloud
);

CREATE TABLE IF NOT EXISTS conversation_messages (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES conversation_sessions(id),
  role TEXT NOT NULL,  -- user | assistant | tool
  content TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  has_tool_calls INTEGER NOT NULL DEFAULT 0,
  tool_name TEXT,
  tool_result TEXT
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS background_tasks (
  id TEXT PRIMARY KEY,
  task_type TEXT NOT NULL,  -- index_note | enhance_note | extract_task
  target_id TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',  -- pending | running | done | failed
  retry_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  error_msg TEXT
);

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_notes_project ON notes(project_id);
CREATE INDEX IF NOT EXISTS idx_messages_session ON conversation_messages(session_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_bg_tasks_status ON background_tasks(status);
```

#### Task 0.2.3 — Migration runner (Rust)

```rust
// src-tauri/src/db/migrations.rs
// Runs on every app launch. Idempotent. Safe on upgrade.

pub async fn run(db: &SqlitePool) -> Result<()> {
    sqlx::query(include_str!("schema.sql"))
        .execute(db)
        .await?;
    Ok(())
}
```

#### Task 0.2.4 — DB initialization in lib.rs

```rust
// src-tauri/src/lib.rs
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:vibo.db", migrations::get())
            .build())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::notes::create_note,
            commands::notes::read_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::list_notes,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::move_task,
            commands::tasks::delete_task,
            commands::tasks::list_tasks,
            commands::settings::get_setting,
            commands::settings::set_setting,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // DB migrations run here
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running vibo");
}
```

**UNIT 0.2:**
```rust
#[tokio::test]
async fn test_migrations_idempotent() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    migrations::run(&pool).await.unwrap();
    migrations::run(&pool).await.unwrap(); // second run must not fail
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);
}
```

---

### Stage 0.3 — Notes CRUD

**Goal:** Full notes lifecycle in Rust. TSX only calls typed commands.

#### Task 0.3.1 — Vault directory setup

```rust
// src-tauri/src/vault/mod.rs

pub fn vault_dir(app: &AppHandle) -> PathBuf {
    app.path().document_dir()
        .expect("document dir unavailable")
        .join("vibo")
        .join("vault")
}

pub fn ensure_vault_exists(app: &AppHandle) -> Result<()> {
    let dir = vault_dir(app);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

pub fn note_path(app: &AppHandle, id: &str) -> PathBuf {
    vault_dir(app).join(format!("{}.md", id))
}

pub fn write_note(path: &PathBuf, content: &str) -> Result<()> {
    std::fs::write(path, content)?;
    Ok(())
}

pub fn read_note_file(path: &PathBuf) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}
```

#### Task 0.3.2 — Notes commands (Rust)

```rust
// src-tauri/src/commands/notes.rs

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub file_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub is_secure: bool,
    pub project_id: Option<String>,
}

#[tauri::command]
pub async fn create_note(
    app: AppHandle,
    state: State<'_, DbState>,
    title: String,
    content: String,
    tags: Vec<String>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let file_path = vault::note_path(&app, &id);
    let file_path_str = file_path.to_string_lossy().to_string();

    // 1. Write MD file first
    vault::write_note(&file_path, &content)
        .map_err(|e| e.to_string())?;

    // 2. Insert SQLite metadata
    sqlx::query(
        "INSERT INTO notes (id, title, file_path, created_at, updated_at, tags)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&title)
    .bind(&file_path_str)
    .bind(&now)
    .bind(&now)
    .bind(serde_json::to_string(&tags).unwrap())
    .execute(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    // 3. Return id — IPC closes here, UI unlocked
    Ok(id)
}

#[tauri::command]
pub async fn read_note(
    app: AppHandle,
    state: State<'_, DbState>,
    id: String,
) -> Result<Note, String> {
    let row = sqlx::query_as::<_, NoteRow>(
        "SELECT * FROM notes WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let file_path = PathBuf::from(&row.file_path);
    let content = vault::read_note_file(&file_path)
        .map_err(|e| e.to_string())?;

    Ok(Note {
        id: row.id,
        title: row.title,
        content,
        file_path: row.file_path,
        created_at: row.created_at,
        updated_at: row.updated_at,
        tags: serde_json::from_str(&row.tags).unwrap_or_default(),
        is_secure: row.is_secure == 1,
        project_id: row.project_id,
    })
}

#[tauri::command]
pub async fn update_note(
    app: AppHandle,
    state: State<'_, DbState>,
    id: String,
    content: String,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    // Read existing row for file_path
    let row: NoteRow = sqlx::query_as("SELECT * FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_one(&*state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Write file
    vault::write_note(&PathBuf::from(&row.file_path), &content)
        .map_err(|e| e.to_string())?;

    // Update SQL
    if let Some(ref t) = tags {
        sqlx::query(
            "UPDATE notes SET updated_at = ?, tags = ? WHERE id = ?"
        )
        .bind(&now)
        .bind(serde_json::to_string(t).unwrap())
        .bind(&id)
        .execute(&*state.pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE notes SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(&id)
            .execute(&*state.pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_note(
    state: State<'_, DbState>,
    id: String,
) -> Result<(), String> {
    // Get file path before deleting row
    let row: NoteRow = sqlx::query_as("SELECT * FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_one(&*state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Delete file
    if let Err(e) = std::fs::remove_file(&row.file_path) {
        eprintln!("Warning: could not delete file {}: {}", row.file_path, e);
    }

    // Delete SQL row
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(&id)
        .execute(&*state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_notes(
    state: State<'_, DbState>,
    project_id: Option<String>,
    tags_filter: Option<Vec<String>>,
) -> Result<Vec<NoteListItem>, String> {
    // Returns metadata only — no file reads
    // Full content read only on read_note()
    let rows = sqlx::query_as::<_, NoteListItem>(
        "SELECT id, title, created_at, updated_at, tags, project_id
         FROM notes ORDER BY updated_at DESC"
    )
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows)
}
```

**UNIT 0.3:**
```rust
#[tokio::test]
async fn test_note_lifecycle() {
    let (app, pool) = test_setup().await;
    let id = create_note(app.clone(), pool.clone(),
        "Test".into(), "# Hello\nWorld".into(), vec![]).await.unwrap();
    let note = read_note(app.clone(), pool.clone(), id.clone()).await.unwrap();
    assert_eq!(note.content, "# Hello\nWorld");
    update_note(app.clone(), pool.clone(), id.clone(),
        "# Updated".into(), None).await.unwrap();
    let updated = read_note(app.clone(), pool.clone(), id.clone()).await.unwrap();
    assert_eq!(updated.content, "# Updated");
    delete_note(pool.clone(), id.clone()).await.unwrap();
    assert!(read_note(app, pool, id).await.is_err());
}
```

**SMOKE 0.3:**
- [ ] Create note → file appears in ~/Documents/vibo/vault/
- [ ] Create note → row appears in SQLite
- [ ] Edit note → file content updated, updated_at changes
- [ ] Delete note → file removed, row removed
- [ ] List notes → returns correct metadata
- [ ] Create 50 notes → list returns 50 in correct order

---

### Stage 0.4 — Kanban Tasks CRUD + Status Transitions

**Goal:** Tasks live in SQLite only (no MD file). Status moves (todo → in_progress → done) update position and trigger frontend.

#### Task 0.4.1 — Task commands (Rust)

```rust
// src-tauri/src/commands/tasks.rs

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Archived,
}

#[tauri::command]
pub async fn create_task(
    app: AppHandle,
    state: State<'_, DbState>,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    project_id: Option<String>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Get max position in todo column
    let max_pos: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), 0) FROM tasks WHERE status = 'todo'"
    )
    .fetch_one(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO tasks (id, title, description, status, due_date,
                            project_id, position, created_at, updated_at)
         VALUES (?, ?, ?, 'todo', ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&title).bind(&description).bind(&due_date)
    .bind(&project_id).bind(max_pos + 1000).bind(&now).bind(&now)
    .execute(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    // Emit event so all open windows update
    app.emit("task_created", &id).ok();

    Ok(id)
}

#[tauri::command]
pub async fn move_task(
    app: AppHandle,
    state: State<'_, DbState>,
    id: String,
    status: TaskStatus,
    position: Option<i64>,  // None = append to end of column
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let status_str = serde_json::to_string(&status).unwrap()
        .trim_matches('"').to_string();

    // Get or compute position
    let pos = match position {
        Some(p) => p,
        None => {
            let max: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(position), 0) FROM tasks WHERE status = ?"
            )
            .bind(&status_str)
            .fetch_one(&*state.pool)
            .await
            .map_err(|e| e.to_string())?;
            max + 1000
        }
    };

    sqlx::query(
        "UPDATE tasks SET status = ?, position = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&status_str).bind(pos).bind(&now).bind(&id)
    .execute(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    // Emit event → frontend Kanban board re-renders column
    app.emit("task_moved", serde_json::json!({"id": id, "status": status_str})).ok();

    Ok(())
}

#[tauri::command]
pub async fn list_tasks(
    state: State<'_, DbState>,
    project_id: Option<String>,
) -> Result<KanbanBoard, String> {
    let rows: Vec<Task> = sqlx::query_as(
        "SELECT * FROM tasks WHERE status != 'archived'
         ORDER BY status, position ASC"
    )
    .fetch_all(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(KanbanBoard {
        todo: rows.iter().filter(|t| t.status == TaskStatus::Todo).cloned().collect(),
        in_progress: rows.iter().filter(|t| t.status == TaskStatus::InProgress).cloned().collect(),
        done: rows.iter().filter(|t| t.status == TaskStatus::Done).cloned().collect(),
    })
}
```

#### Task 0.4.2 — Frontend Kanban event listener

```typescript
// src/hooks/useKanban.ts
import { listen } from "@tauri-apps/api/event"
import { useEffect } from "react"

export function useKanbanEvents(refetch: () => void) {
  useEffect(() => {
    const unlisten_created = listen("task_created", () => refetch())
    const unlisten_moved   = listen("task_moved",   () => refetch())
    const unlisten_updated = listen("task_updated", () => refetch())
    const unlisten_deleted = listen("task_deleted", () => refetch())

    return () => {
      unlisten_created.then(f => f())
      unlisten_moved.then(f => f())
      unlisten_updated.then(f => f())
      unlisten_deleted.then(f => f())
    }
  }, [refetch])
}
```

**SMOKE 0.4:**
- [ ] Create task → appears in Todo column
- [ ] Drag task to In Progress → status updates in SQL, frontend updates
- [ ] Drag task to Done → status updates
- [ ] Edit task description → updated_at changes
- [ ] Delete task → removed from board
- [ ] Create 20 tasks → positions maintained correctly across columns
- [ ] Reorder within column → positions swap correctly

**UNIT 0.4:**
```rust
#[tokio::test]
async fn test_task_status_transitions() {
    let (app, pool) = test_setup().await;
    let id = create_task(app.clone(), pool.clone(),
        "Task A".into(), None, None, None).await.unwrap();
    let task = get_task(&pool, &id).await;
    assert_eq!(task.status, TaskStatus::Todo);

    move_task(app.clone(), pool.clone(), id.clone(),
        TaskStatus::InProgress, None).await.unwrap();
    let task = get_task(&pool, &id).await;
    assert_eq!(task.status, TaskStatus::InProgress);

    move_task(app.clone(), pool.clone(), id.clone(),
        TaskStatus::Done, None).await.unwrap();
    let task = get_task(&pool, &id).await;
    assert_eq!(task.status, TaskStatus::Done);
}
```

---

### Stage 0.5 — Security Layer

**Goal:** Biometric lock on iOS/Android. Passphrase on desktop. App is unusable without unlock. Encrypted content remains encrypted at rest.

#### Task 0.5.1 — Research + decide biometrics plugin

```
Decision needed before implementation:
  Option A: tauri-plugin-biometric (community, check current status)
  Option B: Custom Tauri mobile plugin (Swift + Kotlin, LocalAuthentication iOS,
            BiometricPrompt Android)
  Option C: OS keychain access via tauri-plugin-stronghold

Recommendation:
  Phase 0: implement custom mobile plugin (simplest, most reliable)
  Phase 0: implement passphrase via Argon2 + AES-256-GCM for desktop
  Phase 2: upgrade to tauri-plugin-stronghold for key management

Decision must be recorded here before Stage 0.5 begins.
```

#### Task 0.5.2 — App lock state machine

```rust
// src-tauri/src/commands/security.rs

#[derive(Clone, Serialize)]
pub enum LockState {
    Locked,
    Unlocked,
    BiometricsUnavailable,
}

// App-level state
pub struct SecurityState {
    pub lock_state: Mutex<LockState>,
    pub session_key: Mutex<Option<[u8; 32]>>,  // AES key, in memory only
}

#[tauri::command]
pub async fn check_lock_state(
    state: State<'_, SecurityState>
) -> Result<LockState, String> {
    Ok(state.lock_state.lock().unwrap().clone())
}

#[tauri::command]
pub async fn authenticate_biometric(
    app: AppHandle,
    state: State<'_, SecurityState>,
) -> Result<bool, String> {
    // iOS: calls Swift plugin → LocalAuthentication.evaluatePolicy
    // Android: calls Kotlin plugin → BiometricPrompt
    // Desktop: returns BiometricsUnavailable → falls through to passphrase
    #[cfg(target_os = "ios")]
    let result = crate::plugins::biometric::authenticate_ios(&app).await?;
    #[cfg(target_os = "android")]
    let result = crate::plugins::biometric::authenticate_android(&app).await?;
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    let result = false; // Desktop uses passphrase

    if result {
        *state.lock_state.lock().unwrap() = LockState::Unlocked;
        app.emit("app_unlocked", ()).ok();
    }
    Ok(result)
}

#[tauri::command]
pub async fn authenticate_passphrase(
    app: AppHandle,
    state: State<'_, SecurityState>,
    passphrase: String,
) -> Result<bool, String> {
    // Load stored hash from app_settings
    // Verify with Argon2
    // If correct: derive session key, set Unlocked state
    // Never store passphrase in memory — only derived key
    todo!("Implement Argon2 verification")
}

#[tauri::command]
pub async fn lock_app(
    app: AppHandle,
    state: State<'_, SecurityState>,
) -> Result<(), String> {
    *state.lock_state.lock().unwrap() = LockState::Locked;
    // Zero session key from memory
    if let Ok(mut key) = state.session_key.lock() {
        if let Some(ref mut k) = *key {
            k.fill(0);
        }
        *key = None;
    }
    app.emit("app_locked", ()).ok();
    Ok(())
}
```

#### Task 0.5.3 — Desktop passphrase setup flow

```
Steps:
  1. First launch → prompt to set passphrase (minimum 8 chars)
  2. Hash with Argon2id (memory: 65536, iterations: 3, parallelism: 4)
  3. Store hash in app_settings table (key: "passphrase_hash")
  4. Derive AES-256 session key from passphrase using PBKDF2
  5. Store session key in memory only (SecurityState.session_key)
  6. On subsequent launch → passphrase prompt → verify → unlock

Dependencies to add to Cargo.toml:
  argon2 = "0.5"
  aes-gcm = "0.10"
```

#### Task 0.5.4 — Lock screen UI (TSX)

```
Components needed:
  - LockScreen.tsx: renders on top of everything when locked
    - BiometricButton (iOS/Android only)
    - PassphraseInput (all platforms)
    - Error message on failed attempt
    - Attempt counter (lock out after 5 failed attempts)
  - useAppLock hook: listens to "app_locked" / "app_unlocked" events
  - App.tsx: conditionally renders <LockScreen /> vs <MainApp />
```

**SMOKE 0.5:**
- [ ] iOS: FaceID / TouchID prompt appears on launch
- [ ] iOS: failed biometrics → passphrase fallback
- [ ] iOS: correct passphrase → app unlocks
- [ ] Android: BiometricPrompt appears on launch
- [ ] Desktop: passphrase input appears on launch
- [ ] Desktop: wrong passphrase → error, attempt counted
- [ ] Desktop: 5 wrong attempts → 30s lockout
- [ ] App goes to background → returns locked
- [ ] is_secure notes → cannot be read unless unlocked

---

### Stage 0.6 — Settings Persistence

**Goal:** User preferences survive app restarts. Settings accessible from both Rust and TSX.

#### Task 0.6.1 — Settings commands

```rust
#[tauri::command]
pub async fn get_setting(
    state: State<'_, DbState>,
    key: String,
    default: Option<String>,
) -> Result<Option<String>, String> {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_settings WHERE key = ?"
    )
    .bind(&key)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(result.or(default))
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value,
                                        updated_at = excluded.updated_at"
    )
    .bind(&key).bind(&value).bind(&now)
    .execute(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
```

#### Task 0.6.2 — Settings keys registry

```typescript
// src/lib/settings.ts
// All settings keys in one place. Never hardcode strings elsewhere.

export const SETTINGS = {
  FIRST_LAUNCH: "first_launch",
  THEME: "theme",                          // "light" | "dark" | "system"
  ACTIVE_PROJECT: "active_project_id",
  VAULT_DIR_OVERRIDE: "vault_dir_override",
  PASSPHRASE_HASH: "passphrase_hash",
  PASSPHRASE_SALT: "passphrase_salt",
  SECURITY_METHOD: "security_method",      // "biometric" | "passphrase" | "none"
  ACTIVE_MODEL: "active_model_id",         // Phase 1
  ACTIVE_MODEL_SOURCE: "model_source",     // Phase 1: "local" | "cloud"
  CLOUD_PROVIDER: "cloud_provider",        // Phase 2
  ONBOARDING_COMPLETE: "onboarding_done",  // Phase 1
} as const
```

---

### AUDIT 0 — Phase Gate

**Completion criteria — all must be ✅ before Phase 1 begins:**

| Check | Criteria | Status |
|---|---|---|
| Compiles clean | Zero warnings on all 5 targets | [ ] |
| Schema migrates | Running migrations twice does not error | [ ] |
| Notes CRUD | All UNIT 0.3 tests pass | [ ] |
| Tasks CRUD | All UNIT 0.4 tests pass | [ ] |
| Task transitions | SMOKE 0.4 complete | [ ] |
| File persistence | MD files survive app restart | [ ] |
| Security lock | SMOKE 0.5 complete on iOS + Android + Desktop | [ ] |
| IPC hygiene | No payload > 50KB crosses IPC bridge | [ ] |
| No hardcoded paths | vault_dir() uses AppHandle on all platforms | [ ] |
| Settings survive | get_setting after restart returns correct value | [ ] |
| No panics | No unwrap() on user-facing paths | [ ] |

**SCRIPT test_phase_0.sh:**
```bash
#!/bin/bash
set -e
echo "=== PHASE 0 SMOKE ==="
cd src-tauri
cargo test -- --nocapture 2>&1 | grep -E "(test .* ok|FAILED|error)"
cargo clippy -- -D warnings
cargo build --target aarch64-apple-ios 2>&1 | tail -5
echo "=== Phase 0 PASS ==="
```

---

## PHASE 1 — Inference Integration

**Goal:** Real on-device inference running. User can download a model, have a basic chat, and see their conversation persist across sessions. The LEAP plugin drives everything.

**Prerequisites:** Phase 0 AUDIT 0 passed.

---

### Stage 1.1 — tauri-plugin-leap-ai Setup

#### Task 1.1.1 — Add plugin

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-leap-ai = "latest"   # verify version on docs.rs at build time
```

```json
// src-tauri/capabilities/default.json — add:
"leap-ai:allow-runtime-info",
"leap-ai:allow-download-model",
"leap-ai:allow-load-model",
"leap-ai:allow-load-cached-model",
"leap-ai:allow-list-cached-models",
"leap-ai:allow-remove-cached-model",
"leap-ai:allow-unload-model",
"leap-ai:allow-create-conversation",
"leap-ai:allow-create-conversation-from-history",
"leap-ai:allow-generate",
"leap-ai:allow-stop-generation",
"leap-ai:allow-export-conversation"
```

#### Task 1.1.2 — Runtime info check

```typescript
// src/lib/commands.ts — add:
import { invoke } from "@tauri-apps/api/core"

export const leapRuntimeInfo = () =>
  invoke<LeapRuntimeInfo>("plugin:leap-ai|runtime_info")

export const leapListCachedModels = () =>
  invoke<CachedModel[]>("plugin:leap-ai|list_cached_models")
```

**SMOKE 1.1:**
- [ ] `runtime_info` returns without error on iOS simulator
- [ ] `runtime_info` returns without error on Android emulator
- [ ] `runtime_info` returns without error on Desktop
- [ ] Plugin registers without panic in lib.rs setup

---

### Stage 1.2 — Model Catalog + Download + Management

#### Task 1.2.1 — Model catalog definition

```typescript
// src/lib/models.ts
// Single source of truth for all supported models.
// New models added here ONLY — never hardcoded elsewhere.

export interface ModelDefinition {
  id: string
  name: string
  description: string
  sizeBytes: number
  sizeMB: number
  quantizations: QuantizationOption[]
  isDefault: boolean
  minRamMB: number
  source: "local"
}

export interface QuantizationOption {
  id: string
  label: string
  recommendedFor: "quality" | "speed" | "balanced"
}

export const MODEL_CATALOG: ModelDefinition[] = [
  {
    id: "LFM2.5-1.2B-Instruct",
    name: "Vibo Resident (1.2B)",
    description: "Primary model. Fast instruction following and tool use.",
    sizeBytes: 900_000_000,
    sizeMB: 900,
    minRamMB: 1500,
    isDefault: true,
    source: "local",
    quantizations: [
      { id: "Q4_K_M", label: "Q4 Balanced (recommended)", recommendedFor: "balanced" },
      { id: "Q8_0",   label: "Q8 Quality",                recommendedFor: "quality" },
    ]
  },
  {
    id: "LFM2.5-350M",
    name: "Vibo Nano (350M)",
    description: "Background worker. Data extraction and enhancement.",
    sizeBytes: 355_000_000,
    sizeMB: 355,
    minRamMB: 400,
    isDefault: false,
    source: "local",
    quantizations: [
      { id: "Q8_0", label: "Q8 (recommended at this size)", recommendedFor: "quality" },
    ]
  }
]
```

#### Task 1.2.2 — Download commands (typed wrappers)

```typescript
// src/lib/commands.ts — add:

export const leapDownloadModel = (model: string, quantization: string) =>
  invoke<void>("plugin:leap-ai|download_model", {
    payload: { model, quantization }
  })

export const leapLoadModel = (model: string, quantization: string) =>
  invoke<void>("plugin:leap-ai|load_model", {
    payload: { model, quantization }
  })

export const leapLoadCachedModel = (model: string) =>
  invoke<void>("plugin:leap-ai|load_cached_model", { payload: { model } })

export const leapUnloadModel = () =>
  invoke<void>("plugin:leap-ai|unload_model")

export const leapRemoveCachedModel = (model: string) =>
  invoke<void>("plugin:leap-ai|remove_cached_model", { payload: { model } })
```

#### Task 1.2.3 — Download progress listener

```typescript
// src/lib/events.ts
import { onLeapEvent } from "tauri-plugin-leap-ai-api"

export function useModelDownloadProgress(
  onProgress: (percent: number) => void,
  onComplete: () => void,
  onError: (err: string) => void
) {
  return onLeapEvent((event) => {
    if (event.type === "download-progress") {
      onProgress(event.percent)
    }
    if (event.type === "download-complete") {
      onComplete()
    }
    if (event.type === "download-error") {
      onError(event.message)
    }
  })
}
```

#### Task 1.2.4 — ModelManager (Rust state)

```rust
// src-tauri/src/model_manager.rs

pub struct ModelManager {
    pub loaded_model: Mutex<Option<LoadedModel>>,
    pub loaded_nano: Mutex<Option<LoadedModel>>,
}

pub struct LoadedModel {
    pub model_id: String,
    pub quantization: String,
    pub loaded_at: std::time::Instant,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            loaded_model: Mutex::new(None),
            loaded_nano: Mutex::new(None),
        }
    }

    pub fn is_resident_loaded(&self) -> bool {
        self.loaded_model.lock().unwrap().is_some()
    }

    pub fn record_loaded(&self, model_id: String, quantization: String) {
        *self.loaded_model.lock().unwrap() = Some(LoadedModel {
            model_id,
            quantization,
            loaded_at: std::time::Instant::now(),
        });
    }

    pub fn clear_resident(&self) {
        *self.loaded_model.lock().unwrap() = None;
    }
}
```

**SMOKE 1.2:**
- [ ] Download LFM2.5-1.2B-Instruct Q4_K_M → progress events fire
- [ ] Download completes → model appears in list_cached_models
- [ ] Load model → no error
- [ ] Unload model → model removed from memory
- [ ] Remove cached model → file deleted from device storage

---

### Stage 1.3 — Onboarding Flow

**Goal:** First-time user can download a model and understand the app in < 3 minutes.

#### Task 1.3.1 — Onboarding detection

```typescript
// src/App.tsx
const onboardingDone = await getSetting(SETTINGS.ONBOARDING_COMPLETE)
if (!onboardingDone) {
  return <OnboardingFlow />
}
```

#### Task 1.3.2 — Onboarding steps

```
Step 1: Welcome screen
  → Name the app, brief description
  → "Let's set up your AI"

Step 2: Security setup
  → Biometric enrollment (iOS/Android)
  → OR Passphrase setup (Desktop)

Step 3: Model download
  → Show MODEL_CATALOG[0] (LFM2.5-1.2B-Instruct Q4_K_M) as default
  → Show estimated download size and time
  → "Download and continue" button
  → Progress bar during download
  → Estimated time remaining

Step 4: First launch
  → Model downloaded and loaded
  → setSetting(ONBOARDING_COMPLETE, "true")
  → Navigate to main app

Notes:
  - Download can be skipped → user can download later from Settings
  - All onboarding state in app_settings table
  - Onboarding can be re-triggered from Settings > Reset
```

**SMOKE 1.3:**
- [ ] Fresh install → onboarding shows on launch
- [ ] Complete onboarding → onboarding never shows again
- [ ] Skip download → app loads with "No model loaded" state
- [ ] Settings > Reset onboarding → onboarding shows on next launch

---

### Stage 1.4 — Inline Chat + Token Streaming

**Goal:** User can send a message and see a response stream in real time. No RAG yet. Simple context window management.

#### Task 1.4.1 — Conversation management commands (Rust)

```rust
// src-tauri/src/commands/chat.rs

#[tauri::command]
pub async fn create_session(
    state: State<'_, DbState>,
    model_manager: State<'_, ModelManager>,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let model = model_manager.loaded_model.lock().unwrap();
    let model_id = model.as_ref()
        .map(|m| m.model_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    sqlx::query(
        "INSERT INTO conversation_sessions
         (id, created_at, last_active, model_id, model_source)
         VALUES (?, ?, ?, ?, 'local')"
    )
    .bind(&session_id).bind(&now).bind(&now).bind(&model_id)
    .execute(&*state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(session_id)
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, DbState>,
    session_id: String,
    message: String,
) -> Result<String, String> {
    // 1. Persist user message
    let msg_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    persist_message(&state.pool, &msg_id, &session_id, "user", &message, &now).await?;

    // 2. Load conversation history for LEAP
    let history = load_session_messages(&state.pool, &session_id).await?;

    // 3. Get or create LEAP conversation_id
    let leap_conv_id = get_or_create_leap_conversation(
        &app, &state.pool, &session_id, &history
    ).await?;

    // 4. Spawn non-blocking generation
    let app_clone = app.clone();
    let pool_clone = state.pool.clone();
    let session_id_clone = session_id.clone();
    tauri::async_runtime::spawn(async move {
        run_generation(app_clone, pool_clone, session_id_clone,
                       leap_conv_id, message).await
    });

    Ok(msg_id)
}

async fn run_generation(
    app: AppHandle,
    pool: Arc<SqlitePool>,
    session_id: String,
    leap_conv_id: String,
    user_message: String,
) {
    // NOTE: leap plugin API surface must be verified in Phase 0 check
    // Pattern depends on whether plugin exposes Rust state or IPC only
    // This is the bridge to tauri-plugin-leap-ai

    // Generation events arrive via onLeapEvent in frontend
    // Rust side just triggers the generation
    // Full response collected here for persistence

    let result = invoke_leap_generate(&app, &leap_conv_id, &user_message).await;

    match result {
        Ok(full_response) => {
            let msg_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let _ = persist_message(
                &pool, &msg_id, &session_id,
                "assistant", &full_response, &now
            ).await;
        }
        Err(e) => {
            app.emit("generation_error", e).ok();
        }
    }
}
```

#### Task 1.4.2 — Chat UI (TSX)

```
Components:
  ChatView.tsx
    - MessageList: renders user + assistant messages
    - MessageInput: text input + send button
    - StreamingMessage: renders streaming assistant output
    - useChat hook: manages session + messages + streaming state

useChat hook responsibilities:
  - Load session on mount (or create new)
  - Subscribe to onLeapEvent for generation-chunk and generation-complete
  - Call send_message command on submit
  - Append streaming tokens to current assistant message in local state
  - Persist final message on generation-complete

Streaming render pattern:
  - Keep assistantBuffer: string in state
  - On generation-chunk: setAssistantBuffer(prev => prev + chunk)
  - On generation-complete: addMessage(finalMessage); clearBuffer()
  - Render assistantBuffer as a "typing" message with cursor
```

#### Task 1.4.3 — Stop generation

```typescript
export const stopGeneration = (generationId: string) =>
  invoke<void>("plugin:leap-ai|stop_generation", {
    payload: { generationId }
  })
```

**SMOKE 1.4:**
- [ ] Send message → response streams in < 2s TTFT on iOS
- [ ] Generation streams token by token with visible cursor
- [ ] Stop button cancels generation mid-stream
- [ ] Send 10 messages → all persist in SQL
- [ ] Close and reopen app → conversation history visible
- [ ] New conversation → starts fresh context

---

### Stage 1.5 — Basic SQL Conversation Memory

**Goal:** Sessions persist across restarts. LEAP conversation restored via `create_conversation_from_history`.

#### Task 1.5.1 — Session resume

```rust
// src-tauri/src/commands/chat.rs — add:

async fn get_or_create_leap_conversation(
    app: &AppHandle,
    pool: &SqlitePool,
    session_id: &str,
    history: &[Message],
) -> Result<String, String> {
    // Check if LEAP conversation_id already stored
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT leap_conversation_id FROM conversation_sessions
         WHERE id = ? AND leap_conversation_id IS NOT NULL"
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(conv_id) = existing {
        // LEAP session still warm — reuse
        return Ok(conv_id);
    }

    // No active LEAP session — restore from history
    let conv_id = if history.is_empty() {
        // New conversation
        invoke_leap_create_conversation(app, &system_prompt()).await?
    } else {
        // Restore from persisted messages
        invoke_leap_create_conversation_from_history(app, history).await?
    };

    // Persist the new LEAP conversation_id
    sqlx::query(
        "UPDATE conversation_sessions SET leap_conversation_id = ? WHERE id = ?"
    )
    .bind(&conv_id)
    .bind(session_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(conv_id)
}
```

**SMOKE 1.5:**
- [ ] Send 5 messages → close app → reopen → messages visible
- [ ] After reopen, send 6th message → model responds in context of previous 5
- [ ] Model unloaded on app suspend → reloaded on resume → conversation continues

---

### Stage 1.6 — App Lifecycle (Suspend / Resume)

#### Task 1.6.1 — Lifecycle hooks

```rust
// src-tauri/src/lib.rs — in run():
.on_event(|app, event| {
    match event {
        RunEvent::WindowEvent { event: WindowEvent::CloseRequested { .. }, .. }
        | RunEvent::ExitRequested { .. } => {
            // Graceful shutdown
            let model_manager = app.state::<ModelManager>();
            if model_manager.is_resident_loaded() {
                // Trigger unload — best effort on shutdown
                let app_clone = app.clone();
                tauri::async_runtime::block_on(async move {
                    let _ = invoke_leap_unload(&app_clone).await;
                });
            }
        }
        _ => {}
    }
})
```

```typescript
// src/hooks/useAppLifecycle.ts
// iOS/Android specific — listen for background/foreground transitions
import { getCurrentWebview } from "@tauri-apps/api/webview"

export function useAppLifecycle(
  onSuspend: () => void,
  onResume: () => void
) {
  useEffect(() => {
    // Tauri 2.0 mobile lifecycle events
    const unlisten = listen("tauri://blur", onSuspend)
    const unlisten2 = listen("tauri://focus", onResume)
    return () => {
      unlisten.then(f => f())
      unlisten2.then(f => f())
    }
  }, [])
}
```

**SMOKE 1.6:**
- [ ] iOS: home button → model unloads (check memory)
- [ ] iOS: return to app → model reloads → chat continues
- [ ] Android: back to home → same as iOS
- [ ] Desktop: close window → graceful shutdown

---

### AUDIT 1 — Phase Gate

| Check | Criteria | Status |
|---|---|---|
| Model downloads | LFM2.5-1.2B Q4_K_M downloads and loads on all targets | [ ] |
| Inference runs | TTFT < 3s on iPhone 12 or newer | [ ] |
| Streaming | Tokens appear progressively in UI | [ ] |
| Memory persists | Conversation survives app restart | [ ] |
| Session restore | LEAP conversation rebuilt from SQL history | [ ] |
| Lifecycle | Model unloads/reloads on suspend/resume | [ ] |
| No crashes | 30-minute chat session without crash on iOS | [ ] |
| Onboarding | First-launch flow completes without errors | [ ] |
| Model switching | Can switch between Q4 and Q8 from Settings | [ ] |
| Nano available | LFM2.5-350M downloadable and loadable separately | [ ] |

**SCRIPT test_phase_1.sh:**
```bash
#!/bin/bash
set -e
echo "=== PHASE 1 INFERENCE TEST ==="
# Verify plugin crate compiles
cd src-tauri
cargo build --target aarch64-apple-ios 2>&1 | tail -10
# Run command unit tests
cargo test chat:: -- --nocapture
# Verify no blocking calls on main thread
grep -r "block_on\|std::thread::sleep" src/commands/ && echo "WARN: blocking call found" || echo "OK: no blocking calls"
echo "=== Phase 1 PASS ==="
```

---

## PHASE 2 — Cloud Model Integration

**Goal:** User can connect an Anthropic, OpenAI, or custom provider API key. The app routes to cloud OR local transparently. Provider registry is extensible — adding a new provider is < 20 lines.

**Prerequisites:** Phase 1 AUDIT 1 passed.

---

### Stage 2.1 — Provider Registry Architecture

**Goal:** Single Rust registry that defines all cloud providers. New providers added here and nowhere else.

#### Task 2.1.1 — Provider definition

```rust
// src-tauri/src/providers/mod.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub auth_method: AuthMethod,
    pub models: &'static [ModelSpec],
    pub openai_compatible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey,
    OAuth { scopes: &'static [&'static str] },
    BearerToken,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub context_window: usize,
    pub supports_tools: bool,
    pub cost_per_1k_input: f32,
    pub cost_per_1k_output: f32,
}

// The registry — add new providers here only
pub const PROVIDERS: &[ProviderDefinition] = &[
    ProviderDefinition {
        id: "anthropic",
        name: "Anthropic",
        base_url: "https://api.anthropic.com/v1",
        auth_method: AuthMethod::ApiKey,
        openai_compatible: false,   // Anthropic has its own format
        models: &[
            ModelSpec { id: "claude-sonnet-4-6", name: "Claude Sonnet 4.6",
                        context_window: 200_000, supports_tools: true,
                        cost_per_1k_input: 0.003, cost_per_1k_output: 0.015 },
            ModelSpec { id: "claude-haiku-4-5-20251001", name: "Claude Haiku 4.5",
                        context_window: 200_000, supports_tools: true,
                        cost_per_1k_input: 0.0008, cost_per_1k_output: 0.004 },
        ],
    },
    ProviderDefinition {
        id: "openai",
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        auth_method: AuthMethod::ApiKey,
        openai_compatible: true,
        models: &[
            ModelSpec { id: "gpt-4o",      name: "GPT-4o", context_window: 128_000,
                        supports_tools: true, cost_per_1k_input: 0.005, cost_per_1k_output: 0.015 },
            ModelSpec { id: "gpt-4o-mini", name: "GPT-4o Mini", context_window: 128_000,
                        supports_tools: true, cost_per_1k_input: 0.00015, cost_per_1k_output: 0.0006 },
        ],
    },
    ProviderDefinition {
        id: "google",
        name: "Google Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        auth_method: AuthMethod::ApiKey,
        openai_compatible: false,
        models: &[
            ModelSpec { id: "gemini-2.0-flash", name: "Gemini 2.0 Flash",
                        context_window: 1_000_000, supports_tools: true,
                        cost_per_1k_input: 0.00010, cost_per_1k_output: 0.00040 },
        ],
    },
    ProviderDefinition {
        id: "custom",
        name: "Custom (OpenAI Compatible)",
        base_url: "",  // User fills this in
        auth_method: AuthMethod::ApiKey,
        openai_compatible: true,
        models: &[],   // User fills in model ID manually
    },
];

#[tauri::command]
pub fn list_providers() -> Vec<ProviderDefinition> {
    PROVIDERS.to_vec()
}
```

---

### Stage 2.2 — API Key Management (Encrypted)

#### Task 2.2.1 — Encrypted key storage

```toml
# src-tauri/Cargo.toml — add:
tauri-plugin-stronghold = "2"
# OR if stronghold is not yet stable for mobile:
keyring = "2"  # OS-native keychain (Keychain on iOS/macOS, Keystore on Android)
```

```rust
// src-tauri/src/commands/providers.rs

#[tauri::command]
pub async fn save_api_key(
    app: AppHandle,
    provider_id: String,
    api_key: String,
) -> Result<(), String> {
    // Validate key format before storing
    if api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    // Store encrypted in OS keychain
    // Key name: "vibo_apikey_{provider_id}"
    keyring::Entry::new("vibo", &format!("apikey_{}", provider_id))
        .map_err(|e| e.to_string())?
        .set_secret(api_key.as_bytes())
        .map_err(|e| e.to_string())?;

    // Store confirmation (NOT the key itself) in app_settings
    set_setting_internal(&app, &format!("provider_{}_configured", provider_id), "true").await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_api_key(provider_id: String) -> Result<(), String> {
    keyring::Entry::new("vibo", &format!("apikey_{}", provider_id))
        .map_err(|e| e.to_string())?
        .delete_credential()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// INTERNAL ONLY — never exposed via Tauri command
pub(crate) fn get_api_key_internal(provider_id: &str) -> Result<String, String> {
    keyring::Entry::new("vibo", &format!("apikey_{}", provider_id))
        .map_err(|e| e.to_string())?
        .get_secret()
        .map_err(|e| e.to_string())
        .and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn provider_is_configured(provider_id: String) -> Result<bool, String> {
    let entry = keyring::Entry::new("vibo", &format!("apikey_{}", provider_id))
        .map_err(|e| e.to_string())?;
    Ok(entry.get_secret().is_ok())
}
```

---

### Stage 2.3 — OAuth Flows (Google)

#### Task 2.3.1 — OAuth implementation

```
Google OAuth is needed for: Gmail, Calendar, Google Maps (service account or OAuth)

Implementation approach:
  1. Register Vibo as OAuth app in Google Console
     - Client ID + Secret stored in Cargo env (not in source)
  2. Open OAuth URL in device browser (not in-app webview)
     - iOS: ASWebAuthenticationSession
     - Android: Custom Tabs
     - Desktop: tauri::shell::open(oauth_url)
  3. Handle redirect via deep link (vibo://oauth/callback)
  4. Exchange code for access + refresh tokens
  5. Store tokens encrypted via keyring
  6. Refresh tokens automatically before expiry

Rust crate: oauth2 = "4"
Deep link config:
  - iOS: Info.plist URL scheme: vibo
  - Android: AndroidManifest.xml intent filter
  - Desktop: tauri deep link plugin or custom handler

Commands needed:
  - start_oauth_flow(provider: String) → opens browser
  - oauth_callback_handler(code: String, state: String) → exchanges tokens
  - check_oauth_status(provider: String) → returns token validity
  - revoke_oauth(provider: String) → deletes tokens
```

---

### Stage 2.4 — Model Routing + Switching UI

#### Task 2.4.1 — Unified inference command

```rust
// src-tauri/src/commands/chat.rs — refactor send_message:

pub enum InferenceTarget {
    Local { model_id: String },
    Cloud { provider_id: String, model_id: String },
}

// Routing logic — deterministic Rust, never LLM
fn resolve_inference_target(
    settings: &Settings,
    model_manager: &ModelManager,
) -> InferenceTarget {
    let source = settings.get("model_source").unwrap_or("local");
    match source {
        "local" => {
            if model_manager.is_resident_loaded() {
                InferenceTarget::Local {
                    model_id: model_manager.get_loaded_id().unwrap()
                }
            } else {
                // Fallback to cloud if configured
                try_cloud_fallback(settings)
                    .unwrap_or_else(|| panic!("No model available"))
            }
        }
        "cloud" => {
            let provider = settings.get("cloud_provider").expect("provider set");
            let model = settings.get("cloud_model").expect("model set");
            InferenceTarget::Cloud { provider_id: provider, model_id: model }
        }
        _ => panic!("Unknown model source")
    }
}
```

#### Task 2.4.2 — Cloud inference via reqwest + async-openai

```rust
// src-tauri/src/providers/inference.rs

use async_openai::Client;
use async_openai::config::OpenAIConfig;

pub async fn stream_cloud(
    app: AppHandle,
    provider: &ProviderDefinition,
    api_key: &str,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolDefinition>>,
) -> Result<String, String> {
    if provider.openai_compatible {
        stream_openai_compatible(app, provider, api_key, messages, tools).await
    } else {
        match provider.id {
            "anthropic" => stream_anthropic(app, api_key, messages, tools).await,
            "google" => stream_google(app, api_key, messages, tools).await,
            _ => Err(format!("Provider {} not implemented", provider.id))
        }
    }
}
```

**SMOKE 2.4:**
- [ ] Enter Anthropic API key → validated → "Connected" shown
- [ ] Switch to Claude Sonnet → send message → cloud response streams
- [ ] Switch back to local → on-device response streams
- [ ] Invalid API key → clear error message, not a crash
- [ ] Cloud selected but no internet → graceful error, offer switch to local

---

### Stage 2.5 — Privacy + Routing Policy

#### Task 2.5.1 — Data routing policy

```
User-configurable, stored in app_settings:

cloud_routing_policy:
  "always_ask"      → prompt user before any cloud inference
  "local_preferred" → use local if loaded, fallback to cloud if not
  "cloud_preferred" → use cloud if configured, fallback to local
  "local_only"      → never send data to cloud, even if configured

Data never sent to cloud:
  - Note content marked is_secure = true
  - Vault file paths
  - Encryption keys
  - Biometric data

Always shown in UI when cloud is used:
  - Small indicator: "Using [Provider] · [Model]"
```

---

### AUDIT 2 — Phase Gate

| Check | Criteria | Status |
|---|---|---|
| Provider registry | list_providers() returns all built-in providers | [ ] |
| API key encrypted | Key stored in OS keychain, never in SQLite | [ ] |
| Key never in logs | grep logs for sk-, AIza-, no API keys appear | [ ] |
| Cloud inference | Anthropic + OpenAI both stream responses | [ ] |
| Model switching | Local → Cloud → Local works in one session | [ ] |
| OAuth Google | Authorization flow completes, token refreshes | [ ] |
| Privacy indicator | Cloud use visible in UI | [ ] |
| Custom provider | Custom base_url + model works with OpenAI-compat API | [ ] |
| Error handling | Network error → clear message, no crash | [ ] |

---

## PHASE 3 — Swiftide RAG Pipeline

**Goal:** Notes and tasks are indexed as vectors. Every chat message retrieves relevant context before inference. The model responds with knowledge of the vault, not just the conversation.

**Prerequisites:** Phase 2 AUDIT 2 passed.

---

### Stage 3.1 — FastEmbed Setup + Model Bundle

#### Task 3.1.1 — Add dependencies

```toml
# src-tauri/Cargo.toml
[dependencies]
swiftide = { version = "latest", features = ["fastembed"] }
swiftide-integrations = { version = "latest", features = ["fastembed"] }
```

#### Task 3.1.2 — Bundle ONNX model

```
Steps:
  1. Download all-MiniLM-L6-v2 ONNX from:
     https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2
     Files needed: model.onnx, tokenizer.json, tokenizer_config.json, vocab.txt
  2. Place in: models/all-minilm-l6-v2/ (project root)
  3. Configure tauri.conf.json resources:
     "resources": { "models/all-minilm-l6-v2/*": "models/all-minilm-l6-v2/" }
  4. In Rust: resolve path via app.path().resource_dir()
  5. CRITICAL: this model always outputs 384-dim vectors. Do NOT truncate.
```

#### Task 3.1.3 — EmbedderState

```rust
// src-tauri/src/embedder.rs

use swiftide_integrations::fastembed::{FastEmbed, EmbeddingModel};

pub struct EmbedderState {
    pub embedder: Arc<FastEmbed>,
}

impl EmbedderState {
    pub fn init(app: &AppHandle) -> Result<Self, String> {
        let model_path = app.path()
            .resource_dir()
            .map_err(|e| e.to_string())?
            .join("models/all-minilm-l6-v2");

        let embedder = FastEmbed::builder()
            .with_user_defined_model(model_path)
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self { embedder: Arc::new(embedder) })
    }

    pub async fn embed_single(&self, text: &str) -> Result<Vec<f32>, String> {
        let result = self.embedder
            .embed(vec![text.to_string()])
            .await
            .map_err(|e| e.to_string())?;
        result.into_iter().next()
            .ok_or_else(|| "Empty embedding result".to_string())
    }

    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        self.embedder.embed(texts).await.map_err(|e| e.to_string())
    }
}
```

**SMOKE 3.1:**
- [ ] App starts → FastEmbed loads without error
- [ ] embed_single("hello world") → Vec<f32> with len == 384
- [ ] Embedding on iOS simulator: < 100ms per text
- [ ] Embedding on older Android (API 26): < 500ms per text
- [ ] 20 texts in batch: total time < single × 20 (confirms batching works)

---

### Stage 3.2 — Swiftide Indexing Pipeline

#### Task 3.2.1 — Add velesdb

```toml
tauri-plugin-velesdb = "latest"  # verify version at build time
```

```json
// capabilities/default.json — add:
"velesdb:default"
```

#### Task 3.2.2 — VaultLoader

```rust
// src-tauri/src/pipeline/vault_loader.rs
use swiftide::indexing::{Loader, Node, NodeStream};

pub struct VaultLoader {
    pub note_id: String,
    pub content: String,
    pub title: String,
    pub is_secure: bool,
    pub updated_at: String,
}

impl Loader for VaultLoader {
    fn into_stream(self) -> NodeStream<'static> {
        let node = Node::new(self.content)
            .with_metadata("note_id",    self.note_id)
            .with_metadata("title",      self.title)
            .with_metadata("is_secure",  self.is_secure.to_string())
            .with_metadata("updated_at", self.updated_at)
            .with_metadata("source_type", "note".to_string());

        Box::pin(futures::stream::once(async move { Ok(node) }))
    }
}
```

#### Task 3.2.3 — VaultStorage (impl Persist)

```rust
// src-tauri/src/pipeline/vault_storage.rs
use swiftide::indexing::{Persist, Node};

pub struct VaultStorage {
    pub app: AppHandle,
}

#[async_trait]
impl Persist for VaultStorage {
    async fn store(&self, node: Node) -> Result<Node, SwiftideError> {
        let note_id = node.metadata("note_id")
            .ok_or_else(|| SwiftideError::custom("missing note_id"))?;
        let chunk_text = &node.chunk;
        let heading = node.metadata("heading").cloned().unwrap_or_default();
        let embedding = node.vectors.get("default")
            .ok_or_else(|| SwiftideError::custom("missing embedding"))?;
        let is_secure: bool = node.metadata("is_secure")
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        // DELETE old chunks for this source first (upsert via delete+insert)
        // then INSERT new chunk
        // Uses tauri-plugin-velesdb invoke under the hood
        store_chunk_in_velesdb(
            &self.app,
            note_id,
            chunk_text,
            &heading,
            embedding,
            is_secure,
        ).await?;

        Ok(node)
    }

    fn batch_size(&self) -> Option<usize> {
        Some(20)  // Mobile RAM budget
    }
}
```

#### Task 3.2.4 — Indexing pipeline runner

```rust
// src-tauri/src/pipeline/indexer.rs
use swiftide::indexing::{self, Pipeline};
use swiftide::transformers::{ChunkMarkdown, Embed};
use swiftide::integrations::fastembed::FastEmbed;

pub async fn index_note(
    app: AppHandle,
    pool: Arc<SqlitePool>,
    embedder: Arc<FastEmbed>,
    note_id: String,
    content: String,
    title: String,
    is_secure: bool,
    updated_at: String,
) -> Result<(), String> {
    // Skip secure notes entirely
    if is_secure {
        return Ok(());
    }

    // Delete old chunks before re-indexing
    delete_note_chunks(&app, &note_id).await?;

    let loader = VaultLoader { note_id, content, title, is_secure, updated_at };
    let storage = VaultStorage { app: app.clone() };

    Pipeline::from_loader(loader)
        .filter_cached(
            swiftide::integrations::redb::Redb::builder()
                .path(get_cache_path(&app)?)
                .build()?
        )
        .then_chunk(
            ChunkMarkdown::builder()
                .max_tokens(500)
                .build()?
        )
        .then_in_batch(
            Embed::new(embedder.clone())
        )
        .then_store_with(storage)
        .run()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

#### Task 3.2.5 — Index trigger on note save

```rust
// src-tauri/src/commands/notes.rs — update create_note and update_note:

// After Ok(id) returned to frontend, THEN spawn background index:
let embedder = state.embedder.clone();
let app_clone = app.clone();
let pool_clone = state.pool.clone();
tauri::async_runtime::spawn(async move {
    if ram_available_for_indexing() {
        if let Err(e) = indexer::index_note(
            app_clone, pool_clone, embedder,
            id, content, title, false, now
        ).await {
            eprintln!("Indexing error: {}", e);
        }
    } else {
        queue_background_task(&pool_clone, "index_note", &id).await.ok();
    }
});
```

**UNIT 3.2:**
```rust
#[tokio::test]
async fn test_note_gets_indexed() {
    let (app, pool, embedder) = test_setup_with_embedder().await;
    let id = create_note(app.clone(), ...).await.unwrap();
    // Allow background task to run
    tokio::time::sleep(Duration::from_millis(500)).await;
    let chunks = query_chunks_for_note(&app, &id).await;
    assert!(!chunks.is_empty(), "Note should be indexed");
    assert_eq!(chunks[0].embedding.len(), 384, "Embedding should be 384-dim");
}

#[tokio::test]
async fn test_secure_note_not_indexed() {
    let (app, pool, embedder) = test_setup_with_embedder().await;
    // Create note with is_secure = true
    // Wait for background indexer
    // Verify zero chunks in velesdb
}
```

---

### Stage 3.3 — velesdb Integration

#### Task 3.3.1 — velesdb schema setup

```
velesdb handles its own storage format. Vibo's responsibilities:
  - Call the appropriate insert/delete/search commands
  - Pass the right metadata fields
  - Handle the typed results

Commands to implement as Tauri wrappers:
  - store_chunk(source_id, source_type, chunk_text, heading, embedding, is_secure)
  - delete_chunks_for_source(source_id)
  - search_chunks(query_embedding, top_k, exclude_secure) → Vec<ChunkResult>
  - search_hybrid(query_embedding, bm25_text, top_k) → Vec<ChunkResult>
```

---

### Stage 3.4 — retrieval.rs (Digestion Path)

#### Task 3.4.1 — Full retrieval implementation

```rust
// src-tauri/src/retrieval.rs

use regex::Regex;

lazy_static::lazy_static! {
    static ref WIKILINK_RE: Regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    static ref HASHTAG_RE:  Regex = Regex::new(r"#([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();
    static ref MENTION_RE:  Regex = Regex::new(r"@([a-zA-Z][a-zA-Z0-9_-]*)").unwrap();
}

pub struct ContextBundle {
    pub exact_refs: Vec<ExactRef>,
    pub semantic_chunks: Vec<SemanticChunk>,
}

pub struct ExactRef {
    pub note_id: String,
    pub title: String,
    pub content_preview: String,
}

pub struct SemanticChunk {
    pub source_id: String,
    pub source_type: String,
    pub chunk_text: String,
    pub heading_context: String,
    pub score: f32,
}

pub async fn build_context_bundle(
    app: &AppHandle,
    pool: &SqlitePool,
    embedder: &FastEmbed,
    message: &str,
    is_unlocked: bool,
) -> Result<ContextBundle, String> {
    // STEP 1: Regex scan — zero cost
    let exact_refs = scan_exact_references(pool, message).await?;

    // STEP 2: Embed full prompt — ONE call, 384 dims
    let query_vec = embedder.embed_single(message).await?;

    // STEP 3: Hybrid search in velesdb
    let semantic_chunks = search_semantic(
        app,
        &query_vec,
        message,  // for BM25
        4,        // top_k
        !is_unlocked,  // exclude_secure if locked
    ).await?;

    Ok(ContextBundle { exact_refs, semantic_chunks })
}

async fn scan_exact_references(
    pool: &SqlitePool,
    message: &str,
) -> Result<Vec<ExactRef>, String> {
    let mut refs = Vec::new();

    // [[Wikilinks]]
    for cap in WIKILINK_RE.captures_iter(message) {
        let title = cap[1].trim().to_string();
        if let Ok(Some(row)) = sqlx::query_as::<_, NoteRow>(
            "SELECT id, title, file_path FROM notes WHERE title = ? COLLATE NOCASE"
        )
        .bind(&title)
        .fetch_optional(pool)
        .await {
            // Read first 300 chars as preview
            let content = std::fs::read_to_string(&row.file_path)
                .unwrap_or_default();
            refs.push(ExactRef {
                note_id: row.id,
                title: row.title,
                content_preview: content.chars().take(300).collect(),
            });
        }
    }

    // TODO: #hashtag → notes tagged with that tag
    // TODO: @mention → notes mentioning that person

    Ok(refs)
}

pub fn format_context_for_prompt(bundle: &ContextBundle) -> String {
    let mut ctx = String::new();

    if !bundle.exact_refs.is_empty() {
        ctx.push_str("## Referenced notes\n");
        for r in &bundle.exact_refs {
            ctx.push_str(&format!("### {}\n{}\n\n", r.title, r.content_preview));
        }
    }

    if !bundle.semantic_chunks.is_empty() {
        ctx.push_str("## Related context\n");
        for c in &bundle.semantic_chunks {
            if !c.heading_context.is_empty() {
                ctx.push_str(&format!("**{}**: {}\n\n", c.heading_context, c.chunk_text));
            } else {
                ctx.push_str(&format!("{}\n\n", c.chunk_text));
            }
        }
    }

    ctx
}
```

**UNIT 3.4:**
```rust
#[tokio::test]
async fn test_wikilink_resolution() {
    let (pool) = db_setup().await;
    create_note_in_db(&pool, "Project Alpha", "...", vec![]).await;
    let refs = scan_exact_references(&pool, "What happened in [[Project Alpha]]?").await.unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].title, "Project Alpha");
}

#[tokio::test]
async fn test_semantic_retrieval_after_index() {
    // Create note about "meeting blockers"
    // Index it
    // Query with "what blocked the team"
    // Assert top result contains meeting note
}
```

---

### Stage 3.5 — Context Injection into Chat

#### Task 3.5.1 — Update send_message to use retrieval

```rust
// In send_message, before calling LEAP:

let security = state.security.lock().unwrap();
let is_unlocked = matches!(*security, LockState::Unlocked);

let bundle = retrieval::build_context_bundle(
    &app, &state.pool, &state.embedder.embedder, &message, is_unlocked
).await?;

let context_str = retrieval::format_context_for_prompt(&bundle);

let enriched_message = if context_str.is_empty() {
    message.clone()
} else {
    format!("{}\n\n---\nUser message:\n{}", context_str, message)
};

// Pass enriched_message to LEAP
```

---

### Stage 3.6 — RAG Skills + Full-Text Search

#### Task 3.6.1 — Full-text search command

```rust
#[tauri::command]
pub async fn search_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<SearchResults, String> {
    let limit = limit.unwrap_or(10);

    // Parallel: SQL full-text + semantic vector
    let (sql_results, semantic_results) = tokio::join!(
        search_sql_fulltext(&state.pool, &query, limit),
        retrieval::build_context_bundle(&app, &state.pool,
            &state.embedder.embedder, &query, true)
    );

    // Merge and deduplicate by note_id
    // SQL results for exact match, semantic for fuzzy
    Ok(merge_search_results(sql_results?, semantic_results?))
}
```

#### Task 3.6.2 — Task embedding on create

```rust
// In create_task: after INSERT, spawn background embed
tauri::async_runtime::spawn(async move {
    let text = format!("{} {}", title, description.unwrap_or_default());
    if let Ok(embedding) = embedder.embed_single(&text).await {
        store_task_chunk(&app, &task_id, &text, &embedding).await.ok();
    }
});
```

**SMOKE 3.6:**
- [ ] Create 10 notes with varied topics
- [ ] Wait for indexing → verify chunks in velesdb
- [ ] Send chat: "Summarize my notes about project planning" → correct notes retrieved
- [ ] [[wikilink]] in chat → correct note pulled
- [ ] Secure note → NOT returned in context while app locked
- [ ] search_vault("meeting blockers") → correct notes in results
- [ ] Edit note → re-indexed → new content appears in next search

---

### AUDIT 3 — Phase Gate

| Check | Criteria | Status |
|---|---|---|
| Embedding dims | All stored vectors are exactly 384-dim | [ ] |
| No truncation | No vec.truncate() calls in codebase | [ ] |
| Index on save | Every new note gets indexed within 2s (background) | [ ] |
| Secure isolation | is_secure notes never appear in context while locked | [ ] |
| Cache works | Edit note → unchanged chunks skipped (redb hash match) | [ ] |
| Context in chat | LLM response references vault content when relevant | [ ] |
| Batch efficiency | 20-note index < 10s on iPhone 12 | [ ] |
| RAM budget | Background index does not OOM on 3GB device | [ ] |
| velesdb mobile | Compiles and runs on arm64-apple-ios | [ ] |

**SCRIPT test_phase_3.sh:**
```bash
#!/bin/bash
set -e
echo "=== PHASE 3 RAG TEST ==="
cd src-tauri
# Verify no truncation
if grep -r "truncate(256)\|truncate(128)" src/; then
  echo "FAIL: Illegal truncation found"
  exit 1
fi
# Verify embedding dim tests
cargo test embedding:: -- --nocapture
cargo test retrieval:: -- --nocapture
# Verify no 384-related hardcodes except in test assertions
echo "=== Phase 3 PASS ==="
```

---

## PHASE 4 — Swiftide Agentic Layer

**Goal:** The LLM can take actions — create notes, search the vault, call external APIs — autonomously within a bounded loop. Nano runs background enhancement. External integrations (Google, GitHub, Notion) are callable as tools.

**Prerequisites:** Phase 3 AUDIT 3 passed. LeapBridge API surface verified (Phase 0 open item resolved).

---

### Stage 4.1 — LeapBridge Trait Implementation

**⚠️ BLOCKER: Must verify tauri-plugin-leap-ai Rust API before writing this.**

```
Two possible patterns — verify before implementing:

Pattern A (preferred): Plugin exposes AppHandle extension trait
  use tauri_plugin_leap_ai::LeapAiExt;
  let result = app.leap_ai().complete(messages).await?;
  → Direct in-process call, no channel overhead

Pattern B (fallback): Plugin only exposes IPC commands
  → Use tokio::sync::oneshot channel
  → Send request via mpsc to dedicated leap executor task
  → Receive response on oneshot rx

Verification step (Phase 0 open item):
  1. Read tauri-plugin-leap-ai source on GitHub
  2. Check if Rust state is exposed via extension trait
  3. Record decision here
  4. Implement accordingly
```

#### Task 4.1.1 — LeapBridge struct

```rust
// src-tauri/src/bridge/leap_bridge.rs

use swiftide_core::chat_completion::{
    ChatCompletion, ChatCompletionRequest, ChatCompletionResponse,
    ChatMessage, Role, ToolCall,
};

pub struct LeapBridge {
    app: AppHandle,
    model_id: String,
}

impl LeapBridge {
    pub fn new_resident(app: AppHandle) -> Self {
        Self { app, model_id: "LFM2.5-1.2B-Instruct".to_string() }
    }

    pub fn new_nano(app: AppHandle) -> Self {
        Self { app, model_id: "LFM2.5-350M".to_string() }
    }
}

#[async_trait]
impl ChatCompletion for LeapBridge {
    async fn complete(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, SwiftideError> {
        // Convert Swiftide messages to LEAP format
        let leap_messages: Vec<LeapMessage> = request.messages
            .iter()
            .map(convert_to_leap_message)
            .collect();

        // Convert tool schemas to LFM2.5 format
        // LFM2.5 uses: <|tool_list_start|>[JSON schemas]<|tool_list_end|>
        let tool_list = if let Some(tools) = &request.tools {
            Some(format_tool_list_for_lfm(tools))
        } else {
            None
        };

        // Invoke LEAP (pattern A or B per verification above)
        let response = invoke_leap_complete(
            &self.app,
            &self.model_id,
            leap_messages,
            tool_list,
        ).await.map_err(SwiftideError::custom)?;

        // Parse response for tool calls (LFM2.5 format)
        // <|tool_call_start|>[create_note(title="...", content="...")]<|tool_call_end|>
        let tool_calls = parse_lfm_tool_calls(&response.content);

        Ok(ChatCompletionResponse {
            message: response.content,
            tool_calls,
            finish_reason: response.finish_reason,
        })
    }
}

fn parse_lfm_tool_calls(content: &str) -> Vec<ToolCall> {
    // Parse LFM2.5 Pythonic tool call format:
    // [create_note(title="Meeting", content="..."), update_task(id="x", status="done")]
    // Returns Vec<ToolCall> with name + parsed args
    // Uses pest or manual parser — NOT regex for nested brackets
    todo!("Implement LFM2.5 tool call parser")
}
```

#### Task 4.1.2 — LFM2.5 tool call parser

```
LFM2.5 format:
  <|tool_call_start|>[
    function_name(arg1="value", arg2="value"),
    other_function(arg="value")
  ]<|tool_call_end|>

Parser requirements:
  - Handle multiple tool calls in one response
  - Handle quoted strings with escapes
  - Handle nested parentheses in values
  - Return structured Vec<ToolCall { name, args: HashMap<String, Value> }>
  - On parse error: log + return empty Vec (don't crash)

Implementation: manual recursive descent or pest grammar
Test cases:
  - Single call: [create_note(title="hello", content="world")]
  - Multiple calls: [read_note(id="abc"), list_tasks()]
  - No call: plain text response
  - Malformed: [broken(arg= → empty result, no panic
```

**UNIT 4.1:**
```rust
#[test]
fn test_tool_call_parser() {
    let input = r#"<|tool_call_start|>[create_note(title="Meeting Notes", content="Discussed Q2")]<|tool_call_end|>"#;
    let calls = parse_lfm_tool_calls(input);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "create_note");
    assert_eq!(calls[0].args["title"], "Meeting Notes");
}

#[test]
fn test_parallel_tool_calls() {
    let input = r#"<|tool_call_start|>[read_note(id="abc"), list_tasks()]<|tool_call_end|>"#;
    let calls = parse_lfm_tool_calls(input);
    assert_eq!(calls.len(), 2);
}

#[test]
fn test_malformed_tool_call_no_panic() {
    let calls = parse_lfm_tool_calls("<|tool_call_start|>[broken(arg=");
    assert!(calls.is_empty());
}
```

---

### Stage 4.2 — swiftide-agents + Tool Definitions

#### Task 4.2.1 — Swiftide tool definitions

```rust
// src-tauri/src/tools/vault_tools.rs

use swiftide_agents::tools::{tool, ToolOutput, ToolError};
use swiftide_agents::AgentContext;

#[tool(description = "Create a new note in the vault with the given title and markdown content")]
pub async fn create_note(
    context: &dyn AgentContext,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
) -> Result<ToolOutput, ToolError> {
    let app = context.get_state::<AppHandle>("app_handle")?;
    let pool = context.get_state::<Arc<SqlitePool>>("pool")?;

    let id = crate::commands::notes::create_note_internal(
        app, pool, title, content, tags.unwrap_or_default()
    ).await.map_err(ToolError::from)?;

    Ok(ToolOutput::text(format!("Created note with id: {}", id)))
}

#[tool(description = "Read the full content of a note by its ID")]
pub async fn read_note(
    context: &dyn AgentContext,
    id: String,
) -> Result<ToolOutput, ToolError> {
    let app = context.get_state::<AppHandle>("app_handle")?;
    let pool = context.get_state::<Arc<SqlitePool>>("pool")?;
    let note = crate::commands::notes::read_note_internal(app, pool, &id)
        .await.map_err(ToolError::from)?;
    Ok(ToolOutput::text(format!("# {}\n{}", note.title, note.content)))
}

#[tool(description = "Search the vault for notes and tasks related to a query")]
pub async fn search_vault(
    context: &dyn AgentContext,
    query: String,
    limit: Option<usize>,
) -> Result<ToolOutput, ToolError> {
    let app = context.get_state::<AppHandle>("app_handle")?;
    let pool = context.get_state::<Arc<SqlitePool>>("pool")?;
    let embedder = context.get_state::<Arc<FastEmbed>>("embedder")?;

    let bundle = retrieval::build_context_bundle(
        &app, &pool, &embedder, &query, true
    ).await.map_err(ToolError::from)?;

    let result = retrieval::format_context_for_prompt(&bundle);
    Ok(ToolOutput::text(result))
}

#[tool(description = "Create a new task in the kanban board")]
pub async fn create_task(
    context: &dyn AgentContext,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<ToolOutput, ToolError> {
    let app = context.get_state::<AppHandle>("app_handle")?;
    let pool = context.get_state::<Arc<SqlitePool>>("pool")?;
    let id = crate::commands::tasks::create_task_internal(
        app, pool, title, description, due_date, None
    ).await.map_err(ToolError::from)?;
    Ok(ToolOutput::text(format!("Created task: {}", id)))
}

#[tool(description = "Update the status of a task (todo, in_progress, done)")]
pub async fn update_task_status(
    context: &dyn AgentContext,
    id: String,
    status: String,
) -> Result<ToolOutput, ToolError> {
    // Validates status is a valid enum variant
    let status: TaskStatus = status.parse().map_err(ToolError::from)?;
    let app = context.get_state::<AppHandle>("app_handle")?;
    let pool = context.get_state::<Arc<SqlitePool>>("pool")?;
    crate::commands::tasks::move_task_internal(app, pool, &id, status)
        .await.map_err(ToolError::from)?;
    Ok(ToolOutput::text("Task updated"))
}

#[tool(description = "List all notes, optionally filtered by tag or project")]
pub async fn list_notes(
    context: &dyn AgentContext,
    tag_filter: Option<String>,
    project_id: Option<String>,
) -> Result<ToolOutput, ToolError> {
    // Returns a compact list of note titles + IDs
    // Agent calls read_note() for full content
}
```

#### Task 4.2.2 — Tool registry + schema formatter

```rust
// src-tauri/src/tools/registry.rs
// The router that selects which tools to inject based on intent signals

pub struct ToolRegistry {
    pub all_tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn full() -> Self {
        Self {
            all_tools: vec![
                Box::new(VaultTools::CreateNote),
                Box::new(VaultTools::ReadNote),
                Box::new(VaultTools::SearchVault),
                Box::new(VaultTools::ListNotes),
                Box::new(TaskTools::CreateTask),
                Box::new(TaskTools::UpdateTaskStatus),
                Box::new(TaskTools::ListTasks),
                // Phase 4.4: external tools added here
            ]
        }
    }

    /// Selects ≤3 tool schemas based on deterministic signal matching
    /// NEVER calls an LLM for this routing decision
    pub fn select_for_message(&self, message: &str) -> Vec<&dyn Tool> {
        let lower = message.to_lowercase();
        let mut selected = Vec::new();

        // Vault signals
        if has_signals(&lower, &["note", "write", "save", "document", "[[", "#"]) {
            selected.push(self.find("create_note"));
            selected.push(self.find("read_note"));
            selected.push(self.find("search_vault"));
        }
        // Task signals
        else if has_signals(&lower, &["task", "todo", "kanban", "ticket", "progress"]) {
            selected.push(self.find("create_task"));
            selected.push(self.find("update_task_status"));
            selected.push(self.find("list_tasks"));
        }
        // Search signals
        else if has_signals(&lower, &["find", "search", "look for", "related"]) {
            selected.push(self.find("search_vault"));
            selected.push(self.find("list_notes"));
            selected.push(self.find("list_tasks"));
        }
        // Default
        else {
            selected.push(self.find("search_vault"));
            selected.push(self.find("create_note"));
            selected.push(self.find("create_task"));
        }

        selected.into_iter().flatten().take(3).collect()
    }

    /// Formats selected tools as LFM2.5 <|tool_list_start|> format
    pub fn format_tool_list(tools: &[&dyn Tool]) -> String {
        let schemas: Vec<serde_json::Value> = tools.iter()
            .map(|t| t.to_mcp_schema())
            .collect();
        format!(
            "<|tool_list_start|>{}<|tool_list_end|>",
            serde_json::to_string(&schemas).unwrap()
        )
    }
}
```

---

### Stage 4.3 — Nano Background Agent + Task Queue

#### Task 4.3.1 — Background task queue

```rust
// src-tauri/src/bg_queue.rs

pub struct BackgroundQueue {
    sender: mpsc::Sender<BackgroundJob>,
}

#[derive(Debug)]
pub enum BackgroundJob {
    IndexNote { note_id: String },
    EnhanceNote { note_id: String },
    ParseTaskDescription { task_id: String },
}

impl BackgroundQueue {
    pub fn start(app: AppHandle, pool: Arc<SqlitePool>) -> Self {
        let (tx, mut rx) = mpsc::channel::<BackgroundJob>(100);

        tauri::async_runtime::spawn(async move {
            while let Some(job) = rx.recv().await {
                // RAM check before every job
                if !ram_available_for_bg_task() {
                    // Put back in DB queue for retry
                    persist_job_to_db(&pool, &job).await.ok();
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }

                match job {
                    BackgroundJob::IndexNote { note_id } => {
                        run_index_job(&app, &pool, &note_id).await.ok();
                    }
                    BackgroundJob::EnhanceNote { note_id } => {
                        run_enhance_job(&app, &pool, &note_id).await.ok();
                    }
                    BackgroundJob::ParseTaskDescription { task_id } => {
                        run_parse_task_job(&app, &pool, &task_id).await.ok();
                    }
                }
            }
        });

        Self { sender: tx }
    }

    pub async fn enqueue(&self, job: BackgroundJob) {
        self.sender.send(job).await.ok();
    }
}
```

#### Task 4.3.2 — Nano agent for note enhancement

```rust
// src-tauri/src/agents/nano_agent.rs

pub async fn run_enhance_note(
    app: AppHandle,
    pool: Arc<SqlitePool>,
    note_id: String,
) -> Result<(), String> {
    // 1. Check note doesn't already have #enhanced tag
    let note = read_note_internal(&app, &pool, &note_id).await?;
    if note.tags.contains(&"enhanced".to_string()) {
        return Ok(());
    }

    // 2. Ensure Nano is loaded (lazy load)
    ensure_nano_loaded(&app).await?;

    // 3. Build Swiftide agent with Nano bridge
    let nano_bridge = LeapBridge::new_nano(app.clone());

    let agent = swiftide_agents::Agent::builder()
        .llm(nano_bridge)
        .system_prompt(NANO_ENHANCEMENT_PROMPT)
        .tools(vec![
            Box::new(ReadNoteTool::new(app.clone(), pool.clone())),
            Box::new(ListVaultTagsTool::new(pool.clone())),
            Box::new(UpdateNoteTool::new(app.clone(), pool.clone())),
        ])
        .max_iterations(3)  // Hard cap: never more than 3 turns
        .context_value("app_handle", app.clone())
        .context_value("pool", pool.clone())
        .build()?;

    agent.query(&format!("Enhance note {}", note_id)).await?;

    // 4. Unload Nano if RAM pressure
    if ram_pressure_detected() {
        unload_nano(&app).await.ok();
    }

    Ok(())
}

const NANO_ENHANCEMENT_PROMPT: &str = r#"
You are a note enhancement assistant. Your job is to:
1. Read the target note
2. List existing vault tags
3. Add 2-3 relevant tags from the existing tag set to the note
4. Never create new tags that don't exist in the vault
5. Append #enhanced to the note tags when done
Use your tools. Stop when the note is updated.
"#;
```

**UNIT 4.3:**
```rust
#[tokio::test]
async fn test_nano_enhancement_adds_tags() {
    // Setup: note with "Project Alpha meeting" content
    // Vault has tags: #project, #meeting, #alpha
    // Run nano enhancement
    // Assert note now has at least one of those tags + #enhanced
}

#[tokio::test]
async fn test_nano_stops_after_3_iterations() {
    // Mock a Nano that keeps calling tools
    // Assert agent loop exits at max_iterations=3
}
```

---

### Stage 4.4 — External Integrations (MCP + REST)

#### Task 4.4.1 — Tool implementations for each integration

```
Integration priority order (build in this sequence):
  1. Google Calendar  (OAuth already done in Phase 2)
  2. Google Maps      (API key, simple REST)
  3. Gmail            (OAuth already done in Phase 2)
  4. GitHub           (rmcp → GitHub MCP server)
  5. Notion           (rmcp → Notion MCP server)
  6. WhatsApp         (direct REST → WhatsApp Business API)
```

```rust
// src-tauri/src/tools/external/calendar.rs

#[tool(description = "List upcoming calendar events in the next N days")]
pub async fn list_calendar_events(
    context: &dyn AgentContext,
    days: Option<u32>,
) -> Result<ToolOutput, ToolError> {
    let days = days.unwrap_or(7);
    let token = get_oauth_token_internal("google")?;

    let response = reqwest::Client::new()
        .get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .bearer_auth(&token)
        .query(&[("maxResults", "10"), ("orderBy", "startTime"),
                 ("singleEvents", "true")])
        .send()
        .await
        .map_err(ToolError::from)?
        .json::<CalendarEventsResponse>()
        .await
        .map_err(ToolError::from)?;

    let summary = format_events_for_llm(&response.items);
    Ok(ToolOutput::text(summary))
}

#[tool(description = "Create a new calendar event")]
pub async fn create_calendar_event(
    context: &dyn AgentContext,
    title: String,
    start_time: String,   // ISO 8601
    end_time: String,     // ISO 8601
    description: Option<String>,
) -> Result<ToolOutput, ToolError> {
    // POST to Google Calendar API
    // Returns event ID
}
```

```rust
// src-tauri/src/tools/external/github.rs
// Uses rmcp as HTTP client to GitHub MCP server

use rmcp::{ClientBuilder, Transport};

#[tool(description = "Search GitHub issues and PRs")]
pub async fn github_search(
    context: &dyn AgentContext,
    query: String,
    repo: Option<String>,
) -> Result<ToolOutput, ToolError> {
    let token = get_api_key_internal("github")?;

    // Connect to GitHub MCP server via HTTPS (not local subprocess)
    let client = ClientBuilder::new()
        .base_url("https://api.githubcopilot.com/mcp/v1")  // verify current URL
        .bearer_auth(&token)
        .build()
        .map_err(ToolError::from)?;

    let result = client.call_tool("search_issues", serde_json::json!({
        "query": query,
        "repo": repo
    })).await.map_err(ToolError::from)?;

    Ok(ToolOutput::text(result.to_string()))
}
```

#### Task 4.4.2 — Tool registry update

```rust
// src-tauri/src/tools/registry.rs — update full() to include external tools:

pub fn full_with_integrations(configured: &[&str]) -> Self {
    let mut tools: Vec<Box<dyn Tool>> = vec![/* vault and task tools */];

    // Only include tools for configured providers
    if configured.contains(&"google") {
        tools.push(Box::new(CalendarTools::ListEvents));
        tools.push(Box::new(CalendarTools::CreateEvent));
        tools.push(Box::new(GmailTools::ReadMessages));
        tools.push(Box::new(GmailTools::SendEmail));
    }
    if configured.contains(&"github") {
        tools.push(Box::new(GitHubTools::Search));
        tools.push(Box::new(GitHubTools::CreateIssue));
    }
    if configured.contains(&"notion") {
        tools.push(Box::new(NotionTools::Query));
        tools.push(Box::new(NotionTools::CreatePage));
    }

    Self { all_tools: tools }
}
```

---

### Stage 4.5 — Agentic Chat Loop + Tool Call Detection

#### Task 4.5.1 — Tool call loop in send_message

```rust
// Full agentic chat loop — replaces simple generation in Phase 1

async fn run_agentic_generation(
    app: AppHandle,
    pool: Arc<SqlitePool>,
    embedder: Arc<FastEmbed>,
    session_id: String,
    leap_conv_id: String,
    enriched_message: String,
    selected_tools: Vec<&dyn Tool>,
    max_iterations: usize,
) {
    let mut iteration = 0;
    let mut current_message = enriched_message;

    loop {
        if iteration >= max_iterations {
            // Force a final answer without tools
            let final_prompt = format!(
                "{}\n\nNote: Please provide your final answer now without calling any more tools.",
                current_message
            );
            invoke_leap_generate_final(&app, &leap_conv_id, &final_prompt).await.ok();
            break;
        }

        // Generate (streams tokens to frontend)
        let response = invoke_leap_generate(&app, &leap_conv_id, &current_message).await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                app.emit("generation_error", &e).ok();
                break;
            }
        };

        // Check for tool calls
        let tool_calls = parse_lfm_tool_calls(&response.content);

        if tool_calls.is_empty() {
            // No tool calls → done, persist response
            persist_assistant_message(&pool, &session_id, &response.content).await.ok();
            break;
        }

        // Execute all tool calls in parallel
        let results = execute_tools_parallel(
            &app, &pool, &embedder, tool_calls
        ).await;

        // Format tool results back to model
        let tool_result_msg = format_tool_results_for_lfm(&results);
        current_message = tool_result_msg;

        iteration += 1;
        app.emit("tool_iteration", serde_json::json!({"iteration": iteration})).ok();
    }
}

async fn execute_tools_parallel(
    app: &AppHandle,
    pool: &Arc<SqlitePool>,
    embedder: &Arc<FastEmbed>,
    calls: Vec<ToolCall>,
) -> Vec<ToolResult> {
    // Run all tool calls concurrently
    let futures: Vec<_> = calls.into_iter()
        .map(|call| execute_single_tool(app.clone(), pool.clone(), embedder.clone(), call))
        .collect();

    futures::future::join_all(futures).await
}
```

**SMOKE 4.5:**
- [ ] "Create a note called Meeting with content hello" → note created in vault
- [ ] "What are my notes about project alpha?" → search_vault called → context returned
- [ ] "Move task X to done" → task status updated → kanban refreshes
- [ ] Complex: "Summarize my meeting notes and create a task for each action item" → multiple tools called in parallel where possible
- [ ] Tool call loop exits after max 3 iterations regardless of model output

---

### Stage 4.6 — Context Window Management + Loop Guards

#### Task 4.6.1 — Context window budget

```rust
// src-tauri/src/context_window.rs

pub struct ContextBudget {
    pub max_tokens: usize,           // 32768 for LFM2.5
    pub system_prompt_tokens: usize,
    pub tool_schema_tokens: usize,
    pub history_budget: usize,
    pub context_bundle_budget: usize,
    pub response_budget: usize,
}

impl ContextBudget {
    pub fn for_lfm_1_2b() -> Self {
        Self {
            max_tokens: 32768,
            system_prompt_tokens: 500,
            tool_schema_tokens: 800,     // 3 tools × ~266 tokens each
            history_budget: 8000,        // last N messages fitting this budget
            context_bundle_budget: 4000, // RAG chunks
            response_budget: 2000,       // expected response length
        }
    }

    /// Trim message history to fit budget
    pub fn trim_history(&self, messages: Vec<Message>) -> Vec<Message> {
        let used = self.system_prompt_tokens + self.tool_schema_tokens
                 + self.context_bundle_budget + self.response_budget;
        let available = self.max_tokens.saturating_sub(used);
        trim_messages_to_tokens(messages, available)
    }
}
```

---

### AUDIT 4 — Phase Gate

| Check | Criteria | Status |
|---|---|---|
| LeapBridge compiles | impl ChatCompletion builds without error | [ ] |
| Tool call parser | All UNIT 4.1 tests pass | [ ] |
| Parallel tools | 3 tool calls execute concurrently (tokio::join!) | [ ] |
| Loop guard | No infinite loops, max_iterations enforced | [ ] |
| Tool routing | Correct ≤3 schemas selected per message type | [ ] |
| Nano runs | Enhancement agent completes on background thread | [ ] |
| External tools | Calendar list/create work with real Google account | [ ] |
| GitHub tools | Search returns results via MCP client | [ ] |
| Context fits | 30-message conversation fits LFM2.5 context window | [ ] |
| No UI block | Agentic loop never freezes main thread | [ ] |
| Tool results emit | Frontend receives tool_iteration events | [ ] |

---

## PHASE 5 — Polish, Hardening & Future Extensibility

**Goal:** The app is production-ready. Export works. Plugins can be added without touching core. Performance is profiled. Edge cases handled.

**Prerequisites:** Phase 4 AUDIT 4 passed.

---

### Stage 5.1 — Export, Backup, Offline Handling

| Task | Description | Priority |
|---|---|---|
| Export note as PDF | Use tauri-plugin-fs + printable HTML | High |
| Export conversation | LEAP `export_conversation` command wrapper | High |
| Vault backup as zip | Zip all MD files + SQLite → share sheet | High |
| Offline detection | tauri-plugin-network, disable cloud buttons when offline | High |
| Offline queue | Cloud inference fails → queue for retry when online | Medium |
| Import markdown | Drag-drop MD files into vault | Medium |

---

### Stage 5.2 — Notifications + Reminders

| Task | Description | Priority |
|---|---|---|
| Task due reminders | tauri-plugin-notification, check due_date on launch | High |
| Background indexing complete | Silent notification after bulk re-index | Low |
| Model download complete | Notification when large model finishes downloading | Medium |
| Daily digest | Opt-in: morning summary of tasks due today | Low |

---

### Stage 5.3 — Plugin Registry Architecture

**Goal:** Third-party tools and MCP servers can be added by config, without recompiling.

```rust
// src-tauri/src/plugins/registry.rs
// Plugin manifest loaded at runtime from ~/vibo/plugins/*.json

#[derive(Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub config: serde_json::Value,
}

#[derive(Deserialize)]
pub enum PluginType {
    McpServer { url: String },        // Remote HTTPS MCP server
    RestTool { base_url: String },    // OpenAPI-described REST tool
    NativeCommand { command_name: String },  // Registered Tauri command
}
```

| Task | Description |
|---|---|
| Plugin manifest format | JSON schema definition + validation |
| Plugin discovery | Load all ~/vibo/plugins/*.json at startup |
| MCP plugin type | Connect to any HTTPS MCP server from manifest |
| REST plugin type | Auto-generate #[tool] from OpenAPI spec |
| Plugin UI | Settings > Plugins list + enable/disable |
| Plugin sandbox | Each plugin gets its own tool namespace |

---

### Stage 5.4 — Performance Profiling + Mobile Hardening

```
Profiling targets:
  - Cold launch to first render: < 1.5s on iPhone 12
  - Note save to indexed: < 2s (background, not blocking)
  - chat TTFT (local, 1.2B Q4): < 3s on iPhone 12
  - chat TTFT (cloud, Anthropic): < 2s on WiFi
  - Kanban board with 100 tasks: < 200ms render
  - Search vault (1000 chunks): < 100ms

Mobile hardening:
  - iOS memory warning handler → unload Nano first, then Resident if critical
  - Android OOM handler → same priority
  - Thermal state observer → throttle background indexing at high temp
  - Battery-aware queue → pause enhancement at < 20% battery

Tools:
  tauri-plugin-tracing → flamegraph for Rust hot paths
  Xcode Instruments → iOS memory + CPU profiling
  Android Studio Profiler → Android memory + CPU
```

---

### Stage 5.5 — Accessibility + Privacy Settings

| Task | Description | Priority |
|---|---|---|
| VoiceOver / TalkBack | All interactive elements have aria-label | High |
| Dynamic type support | Text scales with system font size | High |
| High contrast mode | Respects iOS/Android high contrast setting | Medium |
| Privacy dashboard | Show: what's local, what goes to cloud, key storage | High |
| Data deletion | "Delete all my data" wipes vault + SQL + vectors + keys | High |
| Telemetry opt-in | Anonymous crash reporting, explicitly opt-in only | High |
| Secure note audit | Show list of which notes are secure-flagged | Medium |

---

### AUDIT 5 — Final Phase Gate

| Check | Criteria | Status |
|---|---|---|
| Cold launch | < 1.5s on iPhone 12 (measured) | [ ] |
| Memory profile | No memory leak over 1-hour session | [ ] |
| Offline graceful | No crash when all network disabled | [ ] |
| Export works | Note PDF export on all platforms | [ ] |
| Accessibility | VoiceOver navigates main flows | [ ] |
| Privacy dashboard | All data flows documented and visible | [ ] |
| Data deletion | Delete all → vault empty, SQL clean, vectors gone | [ ] |
| Plugin registry | External MCP server loads from manifest | [ ] |
| No hardcoded secrets | grep for API keys in source returns empty | [ ] |
| App Store ready | iOS build passes Xcode validation | [ ] |

---

## CROSS-CUTTING CONCERNS

### Error handling policy

```
Rule: No unwrap() or expect() on user-facing code paths.
      Panic is acceptable only in:
        - test code
        - truly unrecoverable init failures (DB migration failed on first launch)

Pattern:
  All Tauri commands return Result<T, String>
  All errors logged via tracing::error! before returning Err
  Frontend receives typed error strings, not stack traces
  Critical errors → app.emit("app_error", { code, message, recoverable })
```

### Logging policy

```
tracing levels:
  ERROR: user-visible failures, inference errors, DB errors
  WARN:  degraded operation (cache miss, RAM throttle, retry)
  INFO:  lifecycle events (model loaded, session started, chunk indexed)
  DEBUG: per-message flows, tool calls, embeddings (debug builds only)

Never log:
  - API keys
  - Note content
  - Conversation messages
  - File paths containing personal data
```

### Dependency update policy

```
Review dependencies monthly:
  - tauri-plugin-leap-ai: watch for breaking API changes
  - swiftide: breaking changes announced in changelog
  - LFM2.5 model updates: monitor https://huggingface.co/LiquidAI

Before major version bumps:
  - Run all SCRIPT tests
  - Run all UNIT tests
  - SMOKE test on iOS + Android + Desktop
```

### Adding a new LLM provider

```
Checklist:
  1. Add ProviderDefinition to PROVIDERS array in providers/mod.rs
  2. If OpenAI-compatible: done (stream_openai_compatible handles it)
  3. If not compatible: add match arm in stream_cloud()
  4. Add to settings.ts PROVIDERS constant
  5. Update privacy dashboard
  6. Write UNIT test for token streaming
  7. SMOKE: send one message, verify response
```

### Adding a new Swiftide tool

```
Checklist:
  1. Add #[tool] function in appropriate tools/ file
  2. Add to ToolRegistry::full() and full_with_integrations()
  3. Add signal keywords to select_for_message() if needed
  4. Write parser test for LFM2.5 tool call format
  5. Write UNIT test for tool execution
  6. Update MCP schema format test
```

---

## SUMMARY TABLE

| Phase | Focus | Key deliverable | Gate |
|---|---|---|---|
| 0 | Foundation | Notes + Tasks CRUD, security, SQLite | All UNIT + SMOKE tests passing |
| 1 | Inference | On-device chat, model lifecycle, SQL memory | 30-min session no crash on iOS |
| 2 | Cloud | Provider registry, encrypted keys, routing | Anthropic + OpenAI both streaming |
| 3 | RAG | Indexing pipeline, vector search, context injection | Context visibly improves responses |
| 4 | Agents | LeapBridge, tool dispatch, Nano, external APIs | Agentic task creates note + task autonomously |
| 5 | Polish | Export, plugins, perf, accessibility | App Store validation passes |

---

*This document is the source of truth. Updates require engineering lead sign-off and a note in the git commit message: `docs(roadmap): updated phase N - reason`*

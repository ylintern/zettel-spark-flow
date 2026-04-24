# VIBO — Project Setup
`install this once · source of truth for project init`

Run these instructions exactly once, in order, after `bun create tauri-app vibo --template react-ts`.

---

## 1. RUST DEPENDENCIES
`src-tauri/Cargo.toml`

```toml
[package]
name    = "vibo"
version = "0.1.0"
edition = "2021"

[lib]
name          = "vibo_lib"
crate-type    = ["staticlib", "cdylib", "rlib"]

# ── Tauri core ────────────────────────────────────────────────────
[dependencies]
tauri = { version = "2", features = [] }

# ── Official Tauri plugins ────────────────────────────────────────
tauri-plugin-sql       = { version = "2", features = ["sqlite"] }
tauri-plugin-fs        = "2"
tauri-plugin-notification = "2"
tauri-plugin-tracing   = "2"
tauri-plugin-shell     = "2"     # deep-link OAuth callback + open browser
tauri-plugin-stronghold = "2"    # encrypted key storage (API keys, OAuth tokens)

# ── Community Tauri plugins (verify versions at install time) ─────
tauri-plugin-leap-ai   = "0"     # pin to latest on crates.io at setup time
tauri-plugin-velesdb   = "0"     # pin to latest on crates.io at setup time

# ── Async runtime ─────────────────────────────────────────────────
tokio = { version = "1", features = ["full"] }

# ── Database ──────────────────────────────────────────────────────
sqlx  = { version = "0.7", features = ["sqlite", "runtime-tokio", "macros"] }

# ── Serialization ─────────────────────────────────────────────────
serde      = { version = "1", features = ["derive"] }
serde_json = "1"

# ── Utilities ─────────────────────────────────────────────────────
uuid    = { version = "1", features = ["v4"] }
chrono  = { version = "0.4", features = ["serde"] }
thiserror = "1"        # typed errors — no stringly-typed errors in lib code
anyhow    = "1"        # top-level error composition in commands
tracing   = "1"        # structured logging (pairs with tauri-plugin-tracing)

# ── Security ──────────────────────────────────────────────────────
argon2   = "0.5"       # passphrase hashing (desktop unlock)
aes-gcm  = "0.10"      # session key encryption

# ── HTTP (external integrations Phase 2+) ────────────────────────
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"],
            default-features = false }
oauth2  = "4"          # Google OAuth flow

# ── AI pipeline (Phase 3+) ────────────────────────────────────────
swiftide = { version = "0", features = [
  "fastembed",          # embedding via ONNX
  "redb",               # node cache — embedded, no server
  "tree-sitter-markdown" # ChunkMarkdown
]}
swiftide-integrations = { version = "0", features = [
  "fastembed",
  "redb"
]}

# MCP client — remote HTTPS MCP servers only (Notion, GitHub)
rmcp = { version = "0", features = ["client", "transport-streamable-http"] }

# ── Platform-conditional ──────────────────────────────────────────
[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]
keyring = "2"          # OS keychain on desktop (Keychain/Secret Service/WinCred)

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dev-dependencies]
tempfile = "3"
```

> **Version pinning:** replace every `"0"` version with the exact version from `crates.io` at the moment of install. Run `cargo add <crate>` to get the latest and let cargo fill it in. Never leave `"0"` in a committed Cargo.toml.

---

## 2. BUN / NPM DEPENDENCIES
Run from project root:

```bash
# Tauri JS APIs
bun add @tauri-apps/api
bun add @tauri-apps/plugin-sql
bun add @tauri-apps/plugin-fs
bun add @tauri-apps/plugin-notification
bun add @tauri-apps/plugin-shell
bun add tauri-plugin-leap-ai-api      # JS bindings for LEAP plugin
bun add tauri-plugin-tracing-api      # JS bindings for tracing

# UI
bunx shadcn@latest init               # interactive — choose TypeScript + Tailwind
bunx shadcn@latest add button input textarea card dialog \
  dropdown-menu badge separator toast  \
  scroll-area sheet tabs avatar        \
  progress skeleton tooltip

# State + routing
bun add zustand                       # global state — lightweight, no boilerplate
bun add @tanstack/react-router        # file-based routing

# Dev
bun add -d typescript @types/react @types/react-dom tailwindcss
```

---

## 3. CAPABILITIES
`src-tauri/capabilities/default.json`

```json
{
  "identifier": "default",
  "description": "Vibo default capabilities",
  "platforms": ["linux", "macOS", "windows", "iOS", "android"],
  "permissions": [
    "core:default",
    "core:event:default",
    "sql:default",
    "fs:default",
    "fs:allow-read-text-file",
    "fs:allow-write-text-file",
    "fs:allow-create-dir",
    "fs:allow-remove-file",
    "fs:allow-read-dir",
    "fs:allow-exists",
    "notification:default",
    "shell:allow-open",
    "tracing:default",
    "stronghold:default",
    "leap-ai:default",
    "velesdb:default"
  ]
}
```

---

## 4. FOLDER STRUCTURE

```
vibo/
├── src/                          # TSX frontend — everything the user sees
│   │
│   ├── components/               # UI building blocks
│   │   ├── ui/                   # shadcn generated — never edit manually
│   │   ├── notes/
│   │   │   ├── NoteEditor.tsx    # markdown editor for a single note
│   │   │   ├── NoteList.tsx      # sidebar list of notes
│   │   │   └── NoteCard.tsx      # compact card in list view
│   │   ├── tasks/
│   │   │   ├── KanbanBoard.tsx   # full board — renders 3 columns
│   │   │   ├── KanbanColumn.tsx  # single column (todo / in progress / done)
│   │   │   └── TaskCard.tsx      # draggable task card
│   │   ├── chat/
│   │   │   ├── ChatPanel.tsx     # full chat view
│   │   │   ├── MessageList.tsx   # scrollable message thread
│   │   │   ├── MessageBubble.tsx # single message (user or assistant)
│   │   │   └── ChatInput.tsx     # text input + send button + stop
│   │   ├── security/
│   │   │   ├── LockScreen.tsx    # covers app when locked
│   │   │   └── PassphraseForm.tsx
│   │   ├── settings/
│   │   │   ├── SettingsPanel.tsx
│   │   │   ├── ModelSettings.tsx # download, switch, manage models
│   │   │   └── ProviderSettings.tsx # API keys, OAuth, routing policy
│   │   └── onboarding/
│   │       └── OnboardingFlow.tsx
│   │
│   ├── hooks/                    # custom React hooks — one concern per file
│   │   ├── useChat.ts            # session, messages, streaming state
│   │   ├── useNotes.ts           # note list + CRUD
│   │   ├── useTasks.ts           # kanban board + drag state
│   │   ├── useAppLock.ts         # lock/unlock state + biometrics
│   │   ├── useModel.ts           # loaded model, download progress
│   │   ├── useKanbanEvents.ts    # Tauri event listeners for board refresh
│   │   └── useAppLifecycle.ts    # suspend / resume handlers
│   │
│   ├── lib/                      # pure logic — no React, no UI
│   │   ├── commands.ts           # ALL invoke() calls — typed, nothing else
│   │   ├── events.ts             # ALL listen() calls — typed, nothing else
│   │   ├── models.ts             # MODEL_CATALOG — model definitions
│   │   ├── providers.ts          # PROVIDERS — cloud provider definitions
│   │   ├── settings.ts           # SETTINGS keys registry
│   │   └── types.ts              # shared TypeScript types
│   │
│   ├── store/
│   │   └── app.ts                # Zustand store — global UI state only
│   │                             # (lock state, active session, active project)
│   │
│   ├── routes/                   # TanStack Router pages
│   │   ├── __root.tsx
│   │   ├── index.tsx             # home — note list + kanban
│   │   ├── notes.$id.tsx         # single note view
│   │   ├── chat.tsx              # chat view
│   │   └── settings.tsx          # settings view
│   │
│   ├── App.tsx                   # root — renders LockScreen or main routes
│   ├── main.tsx                  # Tauri + React entry point
│   └── index.css                 # Tailwind base imports only
│
├── src-tauri/
│   ├── src/
│   │   │
│   │   ├── lib.rs                # plugin registration + command registration
│   │   │                         # ONLY wires things together — no logic here
│   │   │
│   │   ├── commands/             # Tauri command handlers (#[tauri::command])
│   │   │   ├── mod.rs            # pub mod declarations only
│   │   │   ├── notes.rs          # create, read, update, delete, list notes
│   │   │   ├── tasks.rs          # create, update, move, delete, list tasks
│   │   │   ├── chat.rs           # session, send_message, stop, history
│   │   │   ├── security.rs       # lock, unlock, biometric, passphrase
│   │   │   ├── settings.rs       # get_setting, set_setting
│   │   │   ├── providers.rs      # list, save_key, delete_key, oauth
│   │   │   └── models.rs         # load, unload, list, download wrappers
│   │   │
│   │   ├── db/                   # database — schema and migrations only
│   │   │   ├── mod.rs
│   │   │   ├── schema.sql        # full schema — single source of truth
│   │   │   └── migrations.rs     # runner that applies schema.sql idempotently
│   │   │
│   │   ├── vault/                # file I/O for .md notes
│   │   │   └── mod.rs            # vault_dir(), note_path(), read, write, delete
│   │   │
│   │   ├── models/               # Rust structs matching DB rows and IPC shapes
│   │   │   ├── mod.rs
│   │   │   ├── note.rs           # Note, NoteRow, NoteListItem
│   │   │   ├── task.rs           # Task, TaskStatus, KanbanBoard
│   │   │   ├── message.rs        # Message, ConversationSession
│   │   │   └── provider.rs       # ProviderDefinition, ModelSpec
│   │   │
│   │   ├── providers/            # cloud inference + provider registry
│   │   │   ├── mod.rs            # PROVIDERS const + list_providers command
│   │   │   └── inference.rs      # stream_cloud(), per-provider adapters
│   │   │
│   │   ├── pipeline/             # Swiftide indexing (Phase 3)
│   │   │   ├── mod.rs
│   │   │   ├── indexer.rs        # run_index_note() — the pipeline itself
│   │   │   ├── loader.rs         # VaultLoader impl Loader
│   │   │   └── storage.rs        # VaultStorage impl Persist → velesdb
│   │   │
│   │   ├── retrieval.rs          # digestion path: regex + embed + velesdb search
│   │   │                         # flat file — not a folder, it is one unit
│   │   │
│   │   ├── embedder.rs           # EmbedderState: FastEmbed init + embed_single/batch
│   │   │
│   │   ├── tools/                # Swiftide #[tool] definitions (Phase 4)
│   │   │   ├── mod.rs
│   │   │   ├── vault.rs          # create_note, read_note, search_vault, list_notes
│   │   │   ├── tasks.rs          # create_task, update_task_status, list_tasks
│   │   │   ├── registry.rs       # ToolRegistry: full(), select_for_message()
│   │   │   └── external/         # one file per integration
│   │   │       ├── calendar.rs
│   │   │       ├── gmail.rs
│   │   │       ├── github.rs
│   │   │       ├── notion.rs
│   │   │       └── whatsapp.rs
│   │   │
│   │   ├── agents/               # Swiftide agent runners (Phase 4)
│   │   │   ├── mod.rs
│   │   │   └── nano.rs           # run_enhance_note(), NANO_ENHANCEMENT_PROMPT
│   │   │
│   │   ├── bridge/               # LeapBridge: Swiftide ↔ LEAP plugin (Phase 4)
│   │   │   └── leap.rs           # impl ChatCompletion for LeapBridge
│   │   │
│   │   ├── bg_queue.rs           # background job queue with RAM check + retry
│   │   │
│   │   ├── model_manager.rs      # load/unload state for resident + nano models
│   │   │
│   │   ├── security.rs           # SecurityState, session key, lock state machine
│   │   │
│   │   └── context_window.rs     # ContextBudget: token budget + history trimming
│   │
│   ├── capabilities/
│   │   └── default.json
│   │
│   ├── gen/                      # generated by Tauri CLI — never edit
│   │   ├── android/
│   │   └── apple/
│   │
│   ├── icons/                    # app icons — all sizes
│   ├── build.rs
│   └── tauri.conf.json
│
├── models/                       # bundled ONNX embedding model (added Phase 3)
│   └── all-minilm-l6-v2/
│       ├── model.onnx
│       ├── tokenizer.json
│       ├── tokenizer_config.json
│       └── vocab.txt
│
├── bunfig.toml
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── components.json               # shadcn config
```

---

## 5. lib.rs REGISTRATION TEMPLATE

```rust
// src-tauri/src/lib.rs
// This file ONLY wires things together.
// Zero business logic lives here.

mod commands;
mod db;
mod vault;
mod models;
mod providers;
mod security;
mod model_manager;
mod bg_queue;

// Phase 3+
// mod embedder;
// mod pipeline;
// mod retrieval;

// Phase 4+
// mod tools;
// mod agents;
// mod bridge;
// mod context_window;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        // ── Plugins ─────────────────────────────────────────────
        .plugin(tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:vibo.db", db::migrations::get())
            .build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_tracing::init())
        .plugin(tauri_plugin_stronghold::Builder::new(|pass| {
            // derive stronghold key from passphrase — filled in Phase 2
            todo!()
        }).build())
        .plugin(tauri_plugin_leap_ai::init())
        // velesdb init — verify API at install time
        // .plugin(tauri_plugin_velesdb::init())

        // ── App state ───────────────────────────────────────────
        .setup(|app| {
            app.manage(security::SecurityState::new());
            app.manage(model_manager::ModelManager::new());
            // Phase 3: app.manage(embedder::EmbedderState::init(app.handle())?);
            // Phase 4: app.manage(bg_queue::BackgroundQueue::start(...));
            Ok(())
        })

        // ── Commands ────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // notes
            commands::notes::create_note,
            commands::notes::read_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::list_notes,
            // tasks
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::move_task,
            commands::tasks::delete_task,
            commands::tasks::list_tasks,
            // chat
            commands::chat::create_session,
            commands::chat::send_message,
            commands::chat::list_sessions,
            commands::chat::load_session,
            // security
            commands::security::check_lock_state,
            commands::security::authenticate_biometric,
            commands::security::authenticate_passphrase,
            commands::security::lock_app,
            commands::security::setup_passphrase,
            // settings
            commands::settings::get_setting,
            commands::settings::set_setting,
            // providers (Phase 2)
            commands::providers::list_providers,
            commands::providers::save_api_key,
            commands::providers::delete_api_key,
            commands::providers::provider_is_configured,
            // models
            commands::models::get_loaded_model,
            commands::models::list_downloaded_models,
        ])

        // ── Lifecycle ───────────────────────────────────────────
        .on_event(|app, event| {
            match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    // graceful shutdown — unload model best-effort
                }
                _ => {}
            }
        })

        .run(tauri::generate_context!())
        .expect("error running vibo");
}
```

---

## 6. NAMING CONVENTIONS

### Files and modules

| What | Convention | Example |
|---|---|---|
| Rust module files | `snake_case.rs` | `model_manager.rs` |
| Rust structs | `PascalCase` | `NoteListItem` |
| Rust commands | `snake_case` (matches JS invoke name) | `create_note` |
| TSX components | `PascalCase.tsx` | `KanbanBoard.tsx` |
| TSX hooks | `use` prefix, camelCase | `useKanbanEvents.ts` |
| TSX lib files | `camelCase.ts` | `commands.ts` |
| SQLite tables | `snake_case`, plural | `conversation_messages` |
| SQLite columns | `snake_case` | `updated_at` |
| Tauri events (Rust→TS) | `snake_case` | `task_moved`, `app_locked` |
| Settings keys | `SCREAMING_SNAKE` in `SETTINGS` const | `SETTINGS.ACTIVE_MODEL` |
| Tool IDs (MCP format) | `snake_case` | `create_note`, `search_vault` |

### What lives where — the hard rules

| Rule | Reason |
|---|---|
| `commands.ts` is the ONLY file that calls `invoke()` | One place to audit all IPC calls |
| `events.ts` is the ONLY file that calls `listen()` | One place to audit all event subscriptions |
| `lib.rs` contains ZERO business logic | Only wires plugins + commands + state |
| Commands in `commands/` are THIN — they delegate to internal functions | Internal functions are testable without Tauri |
| DB types live in `models/` — not duplicated in `commands/` | Single struct definition per entity |
| No `utils.rs` or `helpers.rs` — name by what the code does | `vault.rs`, `security.rs`, not `utils.rs` |
| Phase 3+ modules are commented out in lib.rs until that phase begins | Keeps early builds clean and fast |

### The `internal` pattern

Every command that does real work calls an `_internal` function:

```rust
// The Tauri command — thin wrapper, handles AppHandle + State
#[tauri::command]
pub async fn create_note(
    app: AppHandle,
    state: State<'_, DbState>,
    title: String,
    content: String,
    tags: Vec<String>,
) -> Result<String, String> {
    create_note_internal(&app, &state.pool, title, content, tags)
        .await
        .map_err(|e| e.to_string())
}

// The actual logic — testable, callable from other Rust code (tools, agents)
pub async fn create_note_internal(
    app: &AppHandle,
    pool: &SqlitePool,
    title: String,
    content: String,
    tags: Vec<String>,
) -> Result<String, anyhow::Error> {
    // real implementation
}
```

This matters because Swiftide tools (Phase 4) need to call `create_note_internal` directly — they cannot go through the Tauri command dispatch path.

---

## 7. INSTALL SEQUENCE

Run these commands in order, once, from the project root:

```bash
# 1. Create project
bun create tauri-app vibo --template react-ts
cd vibo

# 2. Mobile init (macOS required for iOS)
bun run tauri android init
bun run tauri ios init

# 3. Rust crates
# Edit src-tauri/Cargo.toml first (paste section 1 above)
cd src-tauri && cargo fetch && cd ..

# 4. Bun packages
bun add @tauri-apps/api \
        @tauri-apps/plugin-sql \
        @tauri-apps/plugin-fs \
        @tauri-apps/plugin-notification \
        @tauri-apps/plugin-shell \
        tauri-plugin-leap-ai-api \
        zustand \
        @tanstack/react-router

bun add -d typescript @types/react @types/react-dom

# 5. shadcn
bunx shadcn@latest init
bunx shadcn@latest add button input textarea card dialog \
  dropdown-menu badge separator toast scroll-area \
  sheet tabs avatar progress skeleton tooltip

# 6. Create folder structure
mkdir -p src/components/{ui,notes,tasks,chat,security,settings,onboarding}
mkdir -p src/{hooks,lib,store,routes}
mkdir -p src-tauri/src/{commands,db,vault,models,providers,pipeline,tools/external,agents,bridge}
mkdir -p models/all-minilm-l6-v2

# 7. Create placeholder mod.rs files (prevents cargo errors on unused modules)
touch src-tauri/src/commands/mod.rs
touch src-tauri/src/db/mod.rs
touch src-tauri/src/vault/mod.rs
touch src-tauri/src/models/mod.rs
touch src-tauri/src/providers/mod.rs

# 8. Verify build
bun run tauri build --debug 2>&1 | tail -20
bun run tauri ios dev       # iOS simulator
bun run tauri android dev   # Android emulator
```

---

## 8. SMOKE CHECK AFTER INSTALL

```bash
# Rust compiles clean
cd src-tauri && cargo check 2>&1 | grep -E "^error" | wc -l
# Expected: 0

# No unused imports warnings
cargo clippy 2>&1 | grep "unused import" | wc -l
# Expected: 0

# iOS target compiles
cargo build --target aarch64-apple-ios 2>&1 | tail -5
# Expected: "Finished dev [unoptimized + debuginfo] target(s)"

# Android target compiles
cargo build --target aarch64-linux-android 2>&1 | tail -5
# Expected: "Finished dev [unoptimized + debuginfo] target(s)"

# Frontend has zero TS errors
cd .. && bun run tsc --noEmit 2>&1 | wc -l
# Expected: 0
```

---

## 9. WHAT GETS ADDED PER PHASE (not at init)

| Phase | What gets added |
|---|---|
| 0 | Nothing new — all Phase 0 deps are in the init list above |
| 1 | No new deps — LEAP plugin already installed |
| 2 | `keyring`, `oauth2` in Cargo.toml. Stronghold init filled in. |
| 3 | Uncomment `swiftide`, `swiftide-integrations`. Download ONNX model files into `models/`. Uncomment `pipeline`, `embedder`, `retrieval` in lib.rs. |
| 4 | Uncomment `rmcp`, `tools`, `agents`, `bridge` in lib.rs. |
| 5 | `zip` or `tar` crate for backup export if needed. |

> Never add Phase 3+ imports to lib.rs before Phase 3 begins. Unused feature flags increase compile time on every build.

# P0 Storage Architecture

## Goal
- make persistence work now
- avoid repainting storage when inference arrives later

## Recommendation
| Data | Canonical store | Why |
| --- | --- | --- |
| note body | markdown file | portable, inspectable, future RAG-ready |
| note metadata | SQLite | fast queries, UI filters, sync state |
| tasks / kanban | SQLite | structured, transactional, sortable |
| settings | SQLite | typed app config, easy migration |
| secrets | Stronghold | passphrase-derived secure storage |
| embeddings / retrieval | later | separate from Phase 0 canonical app data |

## Note Model
- one stable `note_id` UUID
- one markdown file per note
- SQL row stores:
  - `id`
  - `path`
  - `title`
  - `folder`
  - `tags`
  - `created_at`
  - `updated_at`
  - `is_encrypted`
  - `content_hash`
  - `sync_state`

## Write Path
1. frontend calls Rust command
2. Rust writes markdown file
3. Rust updates SQLite metadata in same app operation
4. later phases can trigger indexing off `note_saved`

## Why This Beats SQL-Only Notes
- markdown is future-proof for vault workflows
- Swiftide can index files later without redesign
- users keep portable content, not opaque blobs

## Why This Beats MD-Only Everything
- kanban and settings need structured queries
- sync state, filters, and ordering are cleaner in SQL
- migration path to search, chat memory, and audits is easier

## Cross-Phase Prep Allowed Now
- stable UUIDs
- event names like `note_saved`, `task_updated`
- metadata fields needed later for indexing
- clean service boundaries in Rust

## Not Now
- embeddings tables
- chat memory design for inference loops
- hybrid retrieval persistence
- tool execution logs

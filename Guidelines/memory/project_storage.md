---
name: Storage Architecture — Where Data Lives
description: Which store owns which data type; update contracts; canonical sources
type: project
---

Source: whitepaper rev 1.0. Current Phase 0 paths may differ — verify against lib.rs before assuming.

## Storage Map

| Store | Plugin | Owns |
|-------|--------|------|
| SQLite | tauri-plugin-sql | notes metadata, tasks, projects, conversation sessions, messages |
| velesdb | tauri-plugin-velesdb | RAG chunks with 384-dim embeddings + BM25 text |
| Filesystem | tauri-plugin-fs / std::fs | note .md files (canonical content) |
| Stronghold | tauri-plugin-stronghold | secrets, OAuth tokens, encryption keys |
| redb | embedded (Swiftide NodeCache) | ingestion pipeline dedup cache |

## Critical Rules
- **Notes**: `.md` file is canonical content. SQLite stores metadata only.
- **Tasks**: `database/tasks/{id}.md` is canonical content. SQLite stores metadata only (kind='task'). No separate tasks table.
- **Secure notes**: never chunked, never embedded, never in velesdb.
- **Conversation messages**: stored in SQL; recalled by recency SELECT, not vector search.

## Update Contracts
- Note edited → write `.md` to `database/notes/` + update SQLite `updated_at` → queue reindex (redb skips unchanged chunks)
- Task created/edited → write `.md` to `database/tasks/` + upsert SQLite row with `kind='task'`
- Task status toggled → SQLite only, no re-embed
- Session resumed → SELECT messages WHERE session_id = last_active → leap.create_conversation_from_history()

## Canonical Storage Map (confirmed by Lead — 2026-04-09)

| Data | File path | SQLite |
|------|-----------|--------|
| Note content | `database/notes/{id}.md` | `notes` table, `kind = 'note'` |
| Task content | `database/tasks/{id}.md` | `notes` table, `kind = 'task'` |
| Both | `.md` file is canonical content | SQLite holds metadata only |

- SQLite: `{app_data}/vibo.db`
- Stronghold: `{app_data}/secure-vault.hold`
- macOS dev path: `~/Library/Application Support/com.viboai.app.dev/`
- macOS release path: `~/Library/Application Support/com.viboai.app/`

**Decision locked 2026-04-09**: Path rename `vault/` → `database/` is confirmed. Code update pending Lead approval of diff.

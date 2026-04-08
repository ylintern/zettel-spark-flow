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
- **Tasks**: SQLite ONLY — no .md file. Kanban rendered from SQL, not parsed markdown.
- **Notes**: .md file is canonical content. SQLite stores metadata only.
- **Secure notes**: never chunked, never embedded, never in velesdb.
- **Conversation messages**: stored in SQL; recalled by recency SELECT, not vector search.

## Update Contracts
- Note edited → write .md + update SQLite `updated_at` → queue reindex (redb skips unchanged chunks)
- Task status toggled → SQLite only, no re-embed
- Task description edited → SQLite + queue re-embed
- Session resumed → SELECT messages WHERE session_id = last_active → leap.create_conversation_from_history()

## Current Phase 0 Paths (verify in lib.rs — may differ from whitepaper)
- Target: `{app_data}/database/notes/` and `{app_data}/database/tasks/`
- SQLite: `{app_data}/vibo.db`
- Stronghold: `{app_data}/secure-vault.hold`
- macOS dev path: `~/Library/Application Support/com.viboai.app/`

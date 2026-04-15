# Phase 3: Swiftide Indexing Pipelines
## Strategy Document (Deferred)

**Status:** Design locked, implementation deferred until Phase 2 complete  
**Timeline:** 10-14 days after Phase 2  
**Owner:** Engineering Team  

---

## Overview

Phase 3 indexes all notes for semantic search. Swiftide's async pipelines stream files, generate embeddings, store vectors locally.

**Goal:** "Search across all my notes semantically."

---

## Scope

### ✅ In Phase 3
- File streaming pipeline (FileLoader)
- Vector embeddings (dense + sparse)
- Local vector storage (DuckDB or SQLite)
- Semantic search (top-K retrieval)
- Incremental indexing (on note create/update)
- Index rebuild (periodic or on-demand)

### ❌ Out of Phase 3
- Agent reasoning (Phase 4)
- Tool execution (Phase 4)
- Multi-step RAG queries (Phase 4)
- Cloud indexing

---

## Architecture

### Indexing Pipeline
```
FileLoader → read /notes/*.md
  ↓
Extract frontmatter (title, tags) + body
  ↓
Chunk markdown (sentence-level or paragraph-level)
  ↓
Generate embeddings (fastembed or candle)
  ↓
Store in vector DB (chunk_text, embedding, source_note_id, created)
  ↓
Background process (respects mobile OS limits)
```

### Search Flow
```
User types query: "project management"
  ↓
invoke('search_notes', { query })
  ↓
Rust:
  - Embed query
  - Vector search (top-K similar)
  - Return: Vec<{ note_id, title, score }>
  ↓
TSX: Display search results
```

---

## Technical Implementation

### Commands Added
```rust
invoke('index_vault', {})
  → Scans /notes, generates embeddings, stores in DB
  → Runs in background, respects mobile OS

invoke('search_notes', { query })
  → Returns top-K semantically similar notes

invoke('reindex_note', { note_id })
  → On note update, re-embed + update DB

invoke('delete_note_from_index', { note_id })
  → On note delete, remove from index
```

### Vector Storage
- **DuckDB** with vector extension (preferred, lightweight)
- **OR** custom SQLite trait (if DuckDB cross-compilation fails)
- Stores: `(chunk_id, note_id, chunk_text, embedding, created)`

### Embedding Model
- **fastembed** (fast, local, pure Rust)
- Fallback: **candle** framework (if fastembed has cross-compilation issues)

---

## Swiftide Crates Used

- **swiftide-core** — file streaming, chunking, embedding pipelines
- **duckdb** — vector storage (embedded database)
- **fastembed** (TBD cross-compilation)

---

## Success Criteria

- ✅ Vault can be indexed (all notes → vectors)
- ✅ Search returns semantically similar notes
- ✅ Index updates on note create/update/delete
- ✅ Indexing respects mobile OS limits (no battery drain)
- ✅ Vector search completes in <1 second

---

## What Phase 3 Does NOT Include

- ❌ Agent reasoning (Phase 4)
- ❌ Multi-step queries (Phase 4)
- ❌ Tool execution (Phase 4)
- ❌ Web scraping
- ❌ Cloud indexing
- ❌ Real-time indexing (batch only)

---

## Next: Phase 4

Once Phase 3 locked:
- Phase 4: Swiftide Agents + Tool Calls

See [Roadmap Overview](../README.md) for full vision.

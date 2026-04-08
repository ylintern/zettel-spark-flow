# 06 SQL VelesBD

## Goal
- Describe storage layout for chunks, metadata, and semantic vectors.

## Storage Philosophy
- SQL is the durable index of structured retrieval state.
- Vector storage may live in SQL, extension tables, or a dedicated local vector layer.
- The system must remain inspectable and migration-friendly.

## Core Records
- notes
- chunks
- chunk_links
- chunk_tags
- embeddings
- indexing_jobs
- indexing_versions

## Suggested Tables
- `chunks`
- id
- note_id
- ordinal
- heading_path
- content
- content_type
- token_estimate
- content_hash
- chunk_version

- `embeddings`
- chunk_id
- model_id
- dimensions
- vector_ref or inline vector
- created_at
- embedding_version

- `indexing_state`
- note_id
- indexed_at
- policy_version
- status
- last_error

## Retrieval Patterns
- keyword filter by title/tags/links
- note-local chunk fetch
- recent chunk prioritization
- vector similarity candidate fetch
- hybrid rerank stage

## VelesBD Angle
- Treat “vector DB” as a local subsystem, not a cloud dependency
- Keep adapter boundary so implementation can evolve:
- sqlite-only prototype
- sqlite + extension
- dedicated embedded vector engine later

## Dependencies Needed
- SQLite
- migrations
- optional vector extension or adapter
- metadata indexes
- content hash utility

## Core Interfaces
- `ChunkRepository`
- `EmbeddingRepository`
- `IndexStateRepository`
- `HybridSearchRepository`

## Pseudocode
```text
upsert_chunks(note_id, chunks)
delete stale chunks for note_id
upsert embeddings for current chunk hashes
update indexing_state(note_id, status=ready)
```

## Potential Pitfalls
- Large inline vectors bloating SQLite
- weak indexing strategy causing slow retrieval
- no policy version field leading to migration pain
- coupling vector schema too tightly to one provider

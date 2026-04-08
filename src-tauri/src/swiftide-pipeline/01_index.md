# 01 Index

## Goal
- Define note ingestion flow for local-first RAG.
- Keep Markdown vault canonical.
- Produce stable chunk + metadata records for SQL/vector storage.

## Scope
- Raw note text
- Frontmatter extraction
- Markdown normalization
- Chunk planning
- Embedding enqueue
- Persistence + reindex bookkeeping

## Flow
1. Note saved or discovered in vault.
2. Read file + frontmatter.
3. Normalize content:
- line endings
- heading spacing
- fenced code boundaries
- wiki link extraction
4. Build document model:
- note id
- title
- tags
- wikilinks
- folder/path
- timestamps
- encryption state
5. Choose chunking strategy:
- markdown-aware by default
- AST-aware inside code fences
- fallback plain semantic chunking only when structure is weak
6. Persist chunk rows in SQL.
7. Generate embeddings for eligible chunks.
8. Persist embedding/vector references.
9. Mark note indexed and emit progress event.

## Data Stages
- `RawNote`
- `NormalizedNote`
- `StructuredDocument`
- `ChunkPlan`
- `ChunkRecord`
- `EmbeddingJob`
- `IndexedDocument`

## Suggested Pipeline Shape
```text
vault read
-> parse frontmatter
-> normalize markdown
-> extract note metadata
-> choose chunk strategy
-> build chunks
-> write SQL records
-> enqueue embeddings
-> update index status
```

## Dependencies Needed
- Markdown parser
- frontmatter parser
- file watcher or reindex trigger
- SQL layer
- embedding runtime or job queue
- event emitter for progress

## Core Interfaces
- `IngestionRequest { note_id, path, cause }`
- `StructuredDocument { metadata, sections, links, code_blocks }`
- `ChunkPlanner`
- `ChunkWriter`
- `EmbeddingScheduler`
- `IndexStatusStore`

## Reindex Rules
- Reindex on content change
- Reindex on title/tag/frontmatter change
- Reindex on chunking policy version change
- Skip full rebuild when only non-searchable UI state changes

## Pseudocode
```text
ingest(note):
  raw = read_note(note.path)
  doc = normalize_and_parse(raw)
  plan = planner.build(doc)
  sql.replace_chunks(note.id, plan.chunks)
  embeddings.enqueue(plan.embedding_jobs)
  sql.mark_indexed(note.id, plan.version)
```

## Potential Pitfalls
- Reindex loops triggered by generated metadata writes
- Duplicate chunks after partial updates
- Mixing encrypted note content into indexing while vault is locked
- Losing Markdown structure during normalization
- Blocking UI on large reindex runs

# 05 Semantic

## Goal
- Define embedding strategy for semantic retrieval without overcommitting startup cost.

## Principles
- Embeddings are support infrastructure, not canonical storage.
- Markdown + SQL remain source of truth.
- Embedding generation should be asynchronous and resumable.

## Embedding Role
- represent semantic meaning of chunks
- support nearest-neighbor retrieval
- complement keywords, tags, and wiki links
- improve recall on paraphrased queries

## Candidate Local Models
- all-MiniLM class for lightweight local embeddings
- future mobile-aware smaller variants
- optional cloud embeddings later only if explicitly allowed

## Lifecycle
1. Chunk created or changed
2. Embedding job scheduled
3. Model activated on demand
4. Vector generated
5. Vector stored with chunk version
6. Stale vectors invalidated on re-chunk/version change

## Runtime Policy
- Do not eager-load embeddings model on every startup in v1
- Load on indexing/retrieval demand
- Keep capability to cache activation later if benchmarks justify it

## Vector Metadata
- `chunk_id`
- `embedding_model_id`
- `embedding_version`
- `dimensions`
- `created_at`
- `content_hash`

## Dependencies Needed
- embedding model runtime
- job queue / worker
- vector persistence layer
- model manager integration

## Core Interfaces
- `EmbeddingRequest`
- `EmbeddingProvider`
- `VectorStore`
- `EmbeddingLifecyclePolicy`

## Pseudocode
```text
if chunk.needs_embedding():
  model = embedding_provider.activate_if_needed()
  vector = model.embed(chunk.text)
  vector_store.upsert(chunk.id, vector, metadata)
```

## Potential Pitfalls
- Re-embedding too often after small edits
- Startup slowdown if model is always loaded
- Dimension mismatch after model swaps
- Storing vectors without clear model/version metadata

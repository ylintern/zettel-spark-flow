# 02 Chunks

## Goal
- Define chunk philosophy for retrieval quality, not just storage convenience.

## Principles
- Chunks must preserve meaning boundaries.
- Chunks must be small enough for efficient retrieval.
- Chunks must retain enough context to stand alone in an LLM prompt.
- Chunk IDs must be stable across minor edits where possible.

## Why Naive Chunking Fails
- Fixed character splits break thoughts mid-sentence.
- Token windows without structure lose heading context.
- Arbitrary splits make embeddings noisy.
- Small edits can invalidate every chunk id and force expensive reindexing.

## Chunk Constraints
- Target token size: medium, not maximal
- Hard max token size for embedding/runtime safety
- Soft min token size to avoid tiny fragments
- Preserve title/section ancestry in each chunk
- Preserve note id + heading path + source offsets

## Overlap Strategy
- Use small semantic overlap between adjacent chunks
- Overlap should carry continuity, not duplicate whole sections
- Avoid overlap in code chunks when AST structure already preserves context

## Chunk Metadata
- `chunk_id`
- `note_id`
- `heading_path`
- `ordinal`
- `token_estimate`
- `content_type`
- `source_start`
- `source_end`
- `tags`
- `wikilinks`

## Chunk Types
- prose
- list
- table
- code
- quote
- frontmatter summary

## Stability Strategy
- Derive ids from note id + heading path + ordinal + policy version
- Allow selective re-chunking of affected regions only

## Dependencies Needed
- tokenizer estimator
- markdown structure parser
- stable hashing util
- policy versioning

## Core Interfaces
- `ChunkPolicy`
- `ChunkCandidate`
- `ChunkEnvelope`
- `ChunkMetadata`

## Pseudocode
```text
for each section in document:
  if section too large:
    split by paragraph/list boundary
  if still too large:
    split by sentence boundary
  apply limited overlap
  emit stable chunk envelopes
```

## Potential Pitfalls
- Overlap inflation causing duplicated retrieval hits
- Chunks too large for embeddings on low-end devices
- Chunks too small to answer user prompts usefully
- Unstable ids causing constant vector churn

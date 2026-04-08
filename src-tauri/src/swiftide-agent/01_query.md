# 01 Query

## Goal
- Define user prompt -> retrieval -> bundled context -> LLM flow.

## Query Flow
1. User sends prompt from chat UI.
2. Frontend sends `ContextRequest` hints:
- active note id
- active view
- selected notes
- session id
- prompt
3. Rust resolves workspace-aware context.
4. Retrieval layer performs hybrid search:
- metadata/keyword pass
- semantic vector pass
- note-local boosting
5. Merge and rerank results.
6. Build compact context bundle.
7. Pass prompt + context bundle to LLM runtime.
8. Stream response tokens back to UI.

## Hybrid Search Strategy
- keyword search for exact tags, titles, ids, wiki links
- semantic search for paraphrase/intent similarity
- structural boosts:
- active note
- linked notes
- recent notes
- in-progress tasks

## Context Bundle Rules
- prefer fewer higher-quality chunks
- avoid redundant chunks from same section
- attach provenance:
- note title
- heading path
- note id
- chunk id
- skip encrypted content if vault is locked

## Retrieval Stages
- query understanding
- candidate generation
- reranking
- context assembly
- token budget trimming

## Query Transform Ideas
- extract entities/tags/wiki links
- generate keyword subqueries
- detect whether question is local-note, project, or general
- prefer note-local context when prompt references active note

## Dependencies Needed
- retrieval provider
- SQL metadata search
- vector similarity search
- reranker or scoring heuristic
- event bus for streaming/debug status

## Core Interfaces
- `ContextRequest`
- `RetrievalProvider`
- `SearchCandidate`
- `ContextBundleAssembler`
- `AgentQueryExecutor`

## Pseudocode
```text
handle_query(request):
  lexical = search_keywords(request)
  semantic = search_vectors(request)
  merged = rerank(lexical, semantic, request)
  bundle = assemble_context(merged, budget)
  return bundle
```

## Potential Pitfalls
- returning too many low-value chunks
- semantic search overpowering exact-match tags/wiki links
- leaking encrypted note content while vault is locked
- context bundles exceeding model budget
- retrieval latency stacking with inference latency

---
name: Ingestion vs Digestion — Pipeline Split
description: Write path (background Swiftide) vs Read path (hot retrieval.rs) — never mix them
type: project
---

Source: whitepaper rev 1.0. Core architectural split — stable.

## Ingestion (Write Path — always background)
Trigger: `EVENT_NOTE_SAVED`
Never blocks UI. User never waits.

```
VaultLoader → is_secure filter → redb cache check → ChunkMarkdown (≤500 tokens/heading)
→ then_in_batch(FastEmbed) → 384-dim vectors → VaultStorage (velesdb)
```

- Secure notes: filtered OUT before ONNX runs (never embedded)
- Tasks: no chunking — one embedding per task (title + description)
- RAM check before spawn: if low → push to `background_tasks` DB queue, retry on next save
- `background_tasks` table: DB-backed, survives OS kill (tokio queue does not)

## Digestion (Read Path — hot path, every chat call)
Location: `retrieval.rs` — NOT Swiftide query pipeline (~50 lines)
Triggered before every inference call.

```
1. regex_scan(msg) → exact [[wikilinks]], #tags, @mentions → direct velesdb lookup
2. fastembed.embed([full_prompt]) → ONE 384-dim vector (never per-word)
3. velesdb.search(vector, top_k=4, exclude_secure=true) → hybrid BM25 + cosine
4. ContextBundle { exact_refs, semantic_chunks }
5. format_for_lfm(bundle) → prepended to user message → LEAP inference
```

## Why NOT Swiftide Query Pipeline on hot path
Swiftide query pipeline = multi-step retrieval with sub-question generation = multiple LLM calls.
For pre-inference context assembly that's 2 LLM calls where 1 is needed.
Swiftide query pipeline IS correct inside a `#[tool] fn search_vault(...)` called from agent loop.

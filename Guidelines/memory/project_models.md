---
name: Model Assignments — 1.2B, 350M, Embedding
description: Which model owns which task; benchmarks; what each must NOT do
type: project
---

Source: whitepaper rev 1.0. Verify model catalog availability in LEAP before implementing.

## LFM2.5-1.2B-Instruct (Resident)
- Loaded at app launch, stays resident
- Owns: ALL chat, ALL tool calls, ALL agentic turns
- Context: 32K tokens, <1GB RAM at Q4
- IFEval: 74.89% (strong instruction following)
- NOT for: math, programming tasks

## LFM2.5-350M (Nano)
- Lazy load — only when background queue has work
- Unload when: RAM pressure OR batch complete
- Owns: background chunk enrichment ONLY
  - Hypothetical query generation per chunk
  - Task description → structured fields (NL parsing)
- Speed: 88 tok/s on iPhone 13 Mini, 56MB RAM
- IFEval: 65.12% (beats Llama-3.2-1B at 1/3 params)
- Best fine-tune target for Vibo-specific tasks (v2+)
- NOT for: intent classification (use deterministic Rust), hot path ops

## all-MiniLM-L6-v2 (Embedding — always resident)
- Dims: **384 (fixed, NEVER truncate)**
- Input token limit: 256 tokens MAX (not to be confused with output dims)
- Loaded at launch, never unloaded
- Implementation: FastEmbed-rs via Swiftide
- Mobile path: `try_new_from_user_defined(resource_dir/models/all-minilm)`
- Acceleration: CoreML EP on iOS, CPU elsewhere

**Critical:** all-MiniLM-L6-v2 is NOT a Matryoshka model. Truncating to 256 dims degrades cosine similarity. Always use full 384.

**Why:** velesdb is flagged as deprecation risk (may not be needed if SQLite full-text search sufficient) — verify before Phase 1 build.

# Phase 2: Inline Chat & Inference
## Strategy Document (Deferred)

**Status:** Design locked, implementation deferred until Phase 1 complete  
**Timeline:** 7-10 days after Phase 1  
**Owner:** Engineering Team  

---

## Overview

Phase 2 adds simple AI-powered chat sidebar. User asks questions about current note, gets answers from local model or cloud API.

**Goal:** "Ask a quick question about this note, get an insight."

---

## Scope

### ✅ In Phase 2
- Chat sidebar component (simple Q&A)
- Local inference (small model on device) **OR** cloud API (Groq/OpenAI)
- Context feeding (current note to LLM)
- Simple prompt templates (summarize, explain, generate ideas)
- Basic response UI

### ❌ Out of Phase 2
- Multi-turn conversations (V2)
- Vector search (that's Phase 3)
- Agents (that's Phase 4)
- Image generation
- Web scraping

---

## Inference Strategy (TBD)

### Option A: Local Model (Rust)
- Embed small LLM (e.g., Llama 3 Nano, 8B, ~720MB)
- Bind via Rust crate (candle or llama.cpp)
- Runs entirely on device
- **Pro:** Privacy, offline, no API cost
- **Con:** Bundle size, inference latency

### Option B: Cloud API
- Call Groq API, OpenAI, or AWS Bedrock
- HTTP request from Tauri
- **Pro:** Fast, small bundle
- **Con:** Requires internet, API key, privacy concern

### Option C: Hybrid
- Simple Q&A: local
- Complex reasoning: cloud
- **Pro:** Best of both
- **Con:** Complex logic

**Decision:** Engineering to choose based on performance + user privacy tradeoff

---

## User Flow

```
User opens note about "Project X"
User clicks "Ask AI" button
Chat sidebar opens with input field
User types: "Summarize this note"
ViBo passes: { prompt, context_note_id }
  ↓
Rust backend:
  - Fetch note content
  - Route to local model OR cloud API
  - Generate completion
  - Return to TSX
  ↓
Chat sidebar shows response
User can ask follow-up questions (within same note context)
```

---

## Technical Implementation

### Commands Added
```rust
invoke('chat_local', { prompt, context_note_id })
  → Returns completion string

invoke('chat_cloud', { prompt, context_note_id, provider })
  → Returns completion string

invoke('list_available_models', {})
  → Returns available models

invoke('set_inference_provider', { provider })
  → Switches between local/cloud
```

### UI Components
- Chat sidebar (collapsible)
- Input field + send button
- Message history (in sidebar)
- Response streaming (optional, Phase 2.5)

### Prompt Templates
- "Summarize this note"
- "Explain this concept"
- "Generate ideas related to this"
- "Find tasks related to this"
- Custom prompt (user types freely)

---

## Dependencies (Conditional on Strategy)

### If Local:
- **candle-core, candle-nn** (pure Rust ML)
- OR **llama.cpp** (Rust bindings)
- Model file (720MB for Llama 3 Nano)

### If Cloud:
- **reqwest** (HTTP client)
- API keys in secure storage

### Either Way:
- **tokio** (async)

---

## Success Criteria

- ✅ User can ask question about current note
- ✅ Inference (local or cloud) returns answer
- ✅ Response displayed in chat sidebar
- ✅ Context (note content) passed to LLM
- ✅ Simple responses work (no hallucinations yet)
- ✅ Works on desktop + mobile (respects OS limits)

---

## What Phase 2 Does NOT Include

- ❌ Multi-turn conversations (save history, continue)
- ❌ Vector search (Phase 3)
- ❌ Tool execution (Phase 4)
- ❌ Image generation
- ❌ Web scraping
- ❌ Streaming responses (V2)

---

## Next: Phase 3

Once Phase 2 locked:
- Phase 3: Swiftide Indexing (semantic search)

See [Roadmap Overview](../README.md) for full vision.

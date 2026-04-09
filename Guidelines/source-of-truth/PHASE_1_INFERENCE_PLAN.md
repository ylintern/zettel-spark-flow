# Vibo — Phase 1 Inference Plan
**Status**: Organizational only — no code until pre-flight gates pass
**Date**: 2026-04-09
**Sources**: Research paper (Gemini+Swiftide analysis), VIBO Sprint Brief, whitepaper rev 1.0, confirmed code state

---

## 0 · Current State Snapshot

| Item | Status | Notes |
|------|--------|-------|
| Phase 0 storage/CRUD/events | ✅ CLOSED | database/ path, vault_status_changed, note_indexing_progress wired |
| P1-A Event bus | ✅ DONE | agent_thinking_delta defined, not yet emitting real work |
| P1-B CallerContext on all writes | ✅ DONE | All 5 write commands + TypeScript callers pass { type: 'user' } |
| S1 reset_passphrase | ✅ DONE | Rust + frontend |
| S2 Biometrics UI | ✅ DONE | Reads from is_biometric_available |
| S3 Cloud providers UI | ✅ DONE | 3 visible, 4 hidden, get_provider_status (bool-only) |
| S4 Appearance toggle | ✅ DONE | vibo_theme persists |
| S5 Export notes | ✅ DONE | Rust command + dialog |
| P1-C Context bundle | ⏳ PLANNED | retrieval.rs stub exists |
| P1-D LEAP runtime | ⏳ PLANNED | plugin not yet wired |
| P1-cloud Cloud inference | ⏳ PLANNED | after P1-D + P1-C |
| P1-tools Swiftide agent | ⏳ PLANNED | after P1-cloud |

---

## 1 · Research Reconciliation

> The research is a reference, not a spec. Items below have been validated, adapted, or flagged for verification.

### ✅ Confirmed / Adopted as-is

| Research claim | Decision |
|----------------|----------|
| LEAP called via TypeScript guest bindings (Pattern B) | CONFIRMED — do not wrap in custom Tauri command |
| Context must be assembled by Rust invoke('build_context') BEFORE generate() | ADOPTED — P1-C target |
| Stronghold holds API keys; never in frontend state | ALREADY IMPLEMENTED |
| CallerContext::Agent for all agent mutations | ALREADY IMPLEMENTED |
| Swiftide #[tool] macro for agent actions | ADOPTED for P1-tools |
| on_new_message hook → Galvanized Context | ADOPTED — see Section 4 |
| before_tool validator + after_tool compressor | ADOPTED — see Section 4 |
| after_completion → async queue update | ADOPTED — see Section 4 |
| Max 3 tool schemas injected per turn (Rust selects, not LLM) | ADOPTED — permanent rule |
| Conversation messages stored in SQLite, recalled by recency SELECT | ADOPTED |

### ⚠️ Adapted / Corrected

| Research claim | Correction |
|----------------|-----------|
| Build InferenceProvider Rust trait | **Deferred** — abstract ONLY after LEAP works end-to-end. No premature abstraction. |
| Tor (arti-client) routing | **Deferred** — cloud clear-net must be stable and tested first |
| Swiftide query pipeline on hot path | **Killed** — multi-step retrieval = multiple LLM calls. Use retrieval.rs 50-line hot path only. Swiftide query pipeline valid ONLY inside #[tool] fn search_vault() in agent loop. |
| velesdb for vector storage | **Verify first** — flagged deprecation risk in project_models.md. If SQLite FTS sufficient for v1, skip velesdb entirely. Decide at P1-C gate. |

### ❌ Not adopted

| Research claim | Reason |
|----------------|--------|
| Local MCP server | iOS incompatible. MCP = JSON schema format only in Vibo. |
| Specialist 3rd model | Killed (see project_killed_decisions.md) |
| Redis / sidecar caching | Killed — no sidecar processes |
| sqlite-vec for embeddings | Killed — no mobile path handling |

---

## 2 · Dependency Tree

```
P0 DONE
  └── P1-D: LEAP runtime wired
        ├── Pre-flight: 5 questions (Section 3)
        ├── Step 1: Cargo.toml platform split
        ├── Step 2: capabilities/default.json add leap-ai:default
        ├── Step 3: lib.rs plugin registration
        ├── Step 4: Model download UI (catalog-driven)
        ├── Step 5: loadModel + createConversation
        └── Step 6: generate → onLeapEvent → tokens in UI
              └── P1-C: Context bundle / Galvanizer
                    ├── verify: velesdb vs SQLite FTS decision
                    ├── retrieval.rs hot path (~50 lines)
                    │   └── regex scan → embed → vector search → ContextBundle
                    └── invoke('build_context') wired before generate()
                          └── P1-cloud: Cloud inference
                                ├── tauri-plugin-http (NOT raw reqwest)
                                ├── Anthropic + Ollama + Local endpoints
                                ├── Clear-net stable test gate
                                └── TOR only after this gate
                                      └── P1-tools: Swiftide agent loop
                                            ├── Swiftide hooks wired
                                            ├── #[tool] functions (read_note, search_vault, create_task)
                                            ├── Tool schema injection (≤3/turn, Rust selects)
                                            └── agent_thinking_delta real emission
```

---

## 3 · P1-D Pre-Flight (Answer Before Writing Any Code)

> Gate: all 5 must be answered in writing before P1-D code begins.

| # | Question | Where to check |
|---|----------|----------------|
| 1 | Where is tauri-plugin-leap-ai source? Local path or needs cloning? | Cargo.toml, check if path dep exists |
| 2 | Does capabilities/default.json include leap-ai:default? | src-tauri/capabilities/default.json |
| 3 | Does the frontend chat import anything from tauri-plugin-leap-ai-api? | grep src/ for plugin-leap |
| 4 | Is onLeapEvent or agent_thinking_delta subscribed anywhere in frontend? | grep src/ for LeapEvent |
| 5 | Does tauri-plugin-leap-ai appear in Cargo.toml? Show the block. | src-tauri/Cargo.toml |

---

## 4 · Dual Flow Map — User vs Agent

```
USER FLOW (every message)
  ├── Human types query
  ├── Frontend: invoke('build_context', { query })
  ├── Rust: retrieval.rs → regex + embed → top_k=4 → ContextBundle
  ├── Frontend: generate({ conversationId, prompt: context + query })
  │     via tauri-plugin-leap-ai-api guest bindings
  ├── LEAP streams → onLeapEvent generation-chunk
  ├── Frontend renders tokens progressively
  └── generation-complete → finalize

AGENT FLOW (tool loop — P1-tools phase)
  ├── Swiftide agent receives message
  ├── on_new_message hook → Galvanized Context (500-800 chars)
  │     - regex_scan: exact [[wikilinks]], #tags, @mentions → direct lookup
  │     - fastembed: ONE 384-dim vector → hybrid BM25 + cosine top_k=4
  │     - format: [Original Prompt] + [Galvanized Context] list
  ├── Rust selects ≤3 tool schemas (deterministic, NOT an LLM call)
  ├── LLM decides to call tool
  ├── before_tool: validate — was this in Galvanized Context? throttle repeats
  ├── #[tool] fn executes (read_note / search_vault / create_task)
  │     CallerContext::Agent { agent_id } logged → [AGENT MUTATION]
  ├── after_tool: compress raw .md (3000 words → 300-char summary)
  ├── Tool result appended to history → loops back to LLM
  ├── after_completion: parse LLM output → push to async queue (update .md, move kanban card)
  └── on_stop: clear localized memory → return resources to background indexing
```

---

## 5 · Galvanized Context — Format Spec

Source: data_galvanizer.md. Adapted.

```
[Original Prompt]: "user query here"

[Galvanized Context]:
- ID: [[note-847]] (Match: 94%): "one-line summary ≤80 chars"
- ID: [[task-112]] (Match: 89%): "one-line summary ≤80 chars"
- ID: [[note-842]] (Match: 82%): "one-line summary ≤80 chars"
- ID: [[note-831]] (Match: 78%): "one-line summary ≤80 chars"
```

**Rules:**
- Total ≤ 800 characters (enforced in Rust before inject)
- Max 4 entries (top_k=4 from vector search)
- Secure notes never appear — excluded before embedding
- Exact [[wikilink]] matches take priority over semantic matches
- Assembled by retrieval.rs — NOT by a Swiftide query pipeline call

---

## 6 · Storage — What Goes Where

> Carry-forward from project_storage.md. No changes. Verify velesdb at P1-C gate.

| Data | Store | Plugin |
|------|-------|--------|
| Note content | database/notes/{id}.md | tauri-plugin-fs |
| Task content | database/tasks/{id}.md | tauri-plugin-fs |
| Metadata (notes + tasks) | SQLite vibo.db notes table | tauri-plugin-sql |
| Conversation sessions + messages | SQLite vibo.db | tauri-plugin-sql |
| RAG chunks + 384-dim embeddings | velesdb OR SQLite FTS (decide at P1-C gate) | tauri-plugin-velesdb or sql |
| Ingestion dedup cache | redb (Swiftide NodeCache) | embedded |
| Secrets, OAuth tokens, vault keys | Stronghold | tauri-plugin-stronghold |

**velesdb decision gate (P1-C):**
- If SQLite FTS sufficient for v1 search quality → skip velesdb
- If vector similarity is essential from day 1 → proceed with velesdb
- Do not add the dependency speculatively

---

## 7 · Model Assignments (confirmed, no changes)

| Model | Role | Load behavior |
|-------|------|---------------|
| LFM2.5-1.2B-Instruct | ALL chat, ALL tool calls, ALL agentic turns | Resident — loaded at launch |
| LFM2.5-350M (Nano) | Background chunk enrichment ONLY | Lazy load — unload on RAM pressure |
| all-MiniLM-L6-v2 | Embeddings — 384-dim fixed | Always resident, never truncate dims |

**Do not add a 3rd specialist model.** Killed decision.

---

## 8 · Strict Prohibitions (from research + existing guardrails)

```
✗ Do not build InferenceProvider trait before LEAP works end-to-end
✗ Do not integrate Tor (arti-client) before clear-net providers are stable
✗ Do not wire external MCP before internal #[tool] execution is verified
✗ Do not wire LEAP without completing the 5 pre-flight questions first
✗ Do not run Swiftide query pipeline on the hot chat path (only inside #[tool])
✗ Do not inject more than 3 tool schemas per LLM turn
✗ Do not use an LLM call for tool routing — Rust selects deterministically
✗ Do not let agent flow call Tauri commands — only #[tool] from Swiftide loop
✗ Do not let human clicks call #[tool] — only Tauri commands
✗ Do not truncate all-MiniLM embeddings to 256 dims (INPUT limit ≠ output dims)
✗ Do not embed secure notes
✗ Do not store secrets in frontend state
✗ Do not use raw reqwest for cloud calls — use tauri-plugin-http (mobile compat)
```

---

## 9 · Sprint Gate Order (locked)

| Sprint | Gate | Blocked by |
|--------|------|-----------|
| P1-D step 1-3 | Plugin registered, cargo check clean | Pre-flight 5 questions answered |
| P1-D step 4-6 | Token stream visible in UI | Steps 1-3 pass |
| P1-C | build_context wired, context in every inference | P1-D step 6 pass |
| P1-cloud | Anthropic + Ollama + Local work on clear-net | P1-C pass |
| P1-tools | Swiftide agent + #[tool] functions | P1-cloud pass |
| Tor (arti) | Outbound routed through Tor | P1-cloud stable |
| Phase 2 | Private note encryption, external MCP | P1-tools stable |

---

## 10 · Deferred (explicit — do not start)

| Item | Phase | Reason |
|------|-------|--------|
| Private note AES encryption | Phase 2 | Auth and encryption are separate concerns |
| External MCP (rmcp crate) | Phase 2+ | After internal #[tool] stable |
| Tor / arti-client | After P1-cloud | Clear-net must be proven first |
| Biometric hardware-backed unlock | Phase 2 | Secure enclave path not yet built |
| InferenceProvider abstraction trait | After P1-D | Premature abstraction risk |
| Agent role YAML/Markdown files | P1-tools | No agent loop yet |
| velesdb integration | P1-C gate decision | May not be needed if SQLite FTS sufficient |

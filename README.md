

**Engineering lead pre-write verdict:**



1. **Matryoshka truncation to 256 dims on all-MiniLM** — wrong. all-MiniLM-L6-v2 always outputs 384 dims. It was NOT trained with Matryoshka loss. Truncating it arbitrarily breaks cosine similarity. The "256" in the v4 doc confuses the model's **input token limit** (256 tokens max) with its **output dimension** (384, fixed). Use full 384. Done.

2. **"Specialist" third model** — not established, not in LEAP catalog confirmation, adds complexity with zero v1 benefit. Killed.

3. **sqlite-vec replacing velesdb** — v4 silently swapped to sqlite-vec everywhere. sqlite-vec is a raw SQLite extension requiring manual loading and cross-platform binding work. velesdb is a **Tauri plugin** — it handles mobile storage paths, Tauri lifecycle, and cross-platform compilation already. We confirmed it's real. We keep it.

4. **Swiftide query pipeline "overkill" blanket ruling** — partially right for pre-inference retrieval (use `retrieval.rs` there), but the ruling shouldn't extend inside tool functions. A `#[tool] fn search_vault(...)` that runs a mini retrieval pipeline internally is fine and correct Swiftide usage.

Everything else in v4 is sound or better than what preceded it. Now writing.

---Now I have everything I need. Writing from engineering lead position — disagreeing where I disagree, committing where I commit.---

# VIBO AI — Engineering Whitepaper
`final · rev 1.0 · lead sign-off april 2026`

---

## ENGINEERING LEAD AUDIT — V4 VERDICT

Before the spec, the record. Three sources fed this doc. Here is what each got right and wrong.

```yaml
v4_whitepaper:
  survives:
    - Three primitives distinction (Command / Tool / MCP schema) → correct, keeping it
    - FastEmbed-rs over raw ort → correct decision, confirmed
    - Parallel tool execution via Swiftide → confirmed from changelog
    - retrieval.rs for pre-inference read path (not Swiftide query pipeline) → correct
    - redb as Swiftide NodeCache → confirmed from Swiftide changelog entry #346
    - Ingestion vs Digestion naming → clearest framing yet, keeping it
    - LeapBridge trait pattern → correct architecture
    - Swiftide agents only for Nano background tasks → correct
    - No Swiftide query pipeline for interactive chat → correct

  killed:
    - "Matryoshka truncation to 256 dims"
        reason: >
          all-MiniLM-L6-v2 is NOT a Matryoshka model.
          It outputs 384 dims, always.
          The "256" in the v4 doc is the INPUT TOKEN LIMIT (256 tokens max input),
          confused with OUTPUT EMBEDDING DIMENSIONS (384, fixed).
          Truncating this vector arbitrarily degrades cosine similarity with no gain.
          Decision: use full 384 dims. Done.

    - "Specialist" third model
        reason: >
          Introduced in v4 with no prior establishment in research.
          Not in LEAP catalog confirmation.
          Adds model management complexity for zero v1 benefit.
          Two models only: 1.2B Instruct + 350M.

    - sqlite-vec (silently replaced velesdb throughout v4)
        reason: >
          sqlite-vec is a raw SQLite extension. Needs manual loading,
          cross-platform .so/.dylib management, no Tauri lifecycle integration.
          tauri-plugin-velesdb is purpose-built as a Tauri plugin,
          handles mobile storage paths and Tauri lifecycle,
          confirmed real in official awesome-tauri list.
          Decision: velesdb stays.

gemini_v1_errors:
  already_audited: true
  see: "VIBO Engineering Ground Truth rev 1.0"
  summary: >
    LEAP "URL compatible" → false (native FFI only)
    Graph Tasks → not a Swiftide API
    Redis → sidecar on mobile = instant kill
    MCP as local server → iOS incompatible
    Thinking model for chat → wrong variant
    tauri-plugin-mcp-bridge → hallucinated
```

---

## ARCHITECTURE DECISIONS — FINAL AND COMMITTED

```yaml
inference:
  all_calls: tauri-plugin-leap-ai only
  primary: LFM2.5-1.2B-Instruct
    loaded_at: app launch
    stays_resident: true
    owns: all chat, all tool calls, all agentic turns
  nano: LFM2.5-350M
    loaded_at: lazy (first background queue job)
    unload_when: RAM pressure OR batch complete
    owns: background chunk enrichment only
  desktop_fallback: embedded llama.cpp (desktop-embedded-llama feature)

embedding:
  model: all-MiniLM-L6-v2 ONNX
  dims: 384 (fixed, always — NOT truncated)
  bundled_size: ~22MB in app bundle
  loaded_at: app launch, never unloaded
  implementation: FastEmbed-rs via Swiftide native integration
  mobile_path: try_new_from_user_defined(resource_dir/models/all-minilm)
  acceleration: CoreML EP on iOS, CPU elsewhere

vector_storage:
  plugin: tauri-plugin-velesdb
  why_not_sqlite_vec: >
    velesdb is a Tauri plugin — handles mobile paths,
    Tauri lifecycle, cross-platform compilation.
    sqlite-vec is a raw extension needing manual loading.
  search: hybrid BM25 + vector cosine, 70µs, offline-first

pipeline_cache:
  implementation: redb (Swiftide NodeCache, embedded KV, no sidecar)
  confirmed_from: Swiftide changelog entry #346

sql:
  plugin: tauri-plugin-sql (SQLite via sqlx)
  stores: notes metadata, tasks, projects, conversation sessions, messages

mcp:
  local: NEVER (no subprocess, no socket server, not mobile compatible)
  role_in_vibo: schema convention only
    meaning: >
      LFM2.5 expects tools in <|tool_list_start|>...<|tool_list_end|> format.
      That format matches MCP JSON tool definitions.
      We write tool schemas in MCP format.
      Execution happens via Swiftide #[tool] dispatch.
      rmcp crate: used as HTTP client ONLY for remote HTTPS MCP servers.

models:
  no_specialist: true
  count: 2
  names:
    resident: LFM2.5-1.2B-Instruct
    nano: LFM2.5-350M
```

---

## THE THREE PRIMITIVES — COMMITTED DECISION MATRIX

| Primitive | Caller | Runtime | Use for | Never use for |
|---|---|---|---|---|
| `#[tauri::command]` | TSX (human click) | Tauri IPC | save_note, send_message, open_file, settings, trigger_reindex | LLM-initiated actions |
| `#[tool]` (Swiftide) | LLM (agent loop) | In-process Rust | read_note, search_vault, create_task, update_note, calendar_event, github_issue | Human-triggered UI actions |
| MCP schema format | LFM2.5 token parser | Prompt injection | Describing tools to the model in `<\|tool_list_start\|>` format | Runtime transport — MCP is a description format here, not a protocol |

**The clean mental model:**
- Human clicks → Tauri command
- LLM decides → Swiftide tool
- Tool schema in prompt → MCP format (just JSON, no server)

**Shared implementation underneath:** `save_note` Tauri command and `create_note` Swiftide tool can both call the same internal `vault::write_note()` Rust function. The commands are different, the storage function is shared.

---

## SWIFTIDE BOUNDARY — WHAT IT OWNS, WHAT IT DOESN'T

```yaml
swiftide_owns:
  - Indexing pipeline (write path, always background)
      trigger: EVENT_NOTE_SAVED
      steps: VaultLoader → isSecure filter → redb cache → ChunkMarkdown → FastEmbed → velesdb
  - Nano agent loop (background tasks only)
      runtime: swiftide-agents + LeapBridge → LFM2.5-350M
      parallel_tools: automatic via tokio (Swiftide default)
      persistence: before_all hook reads DB, on_stop hook writes DB
  - Tool dispatch when LLM emits tool_calls
      parser: Swiftide handles <|tool_call_start|> token detection
      execution: #[tool] Rust functions, concurrent when multiple calls emitted

swiftide_does_NOT_own:
  - LLM inference → tauri-plugin-leap-ai only
  - Token streaming → LEAP emits onLeapEvent directly to AppHandle
  - Conversation state → LEAP holds KV cache per conversation_id
  - Interactive chat path → direct retrieval.rs + LEAP, no agent overhead
  - Vector storage implementation → velesdb (Swiftide calls it via impl Persist)
  - External HTTP calls → reqwest in tool functions, not Swiftide concern
```

---

## INGESTION vs DIGESTION — THE SPLIT

### Ingestion (write path)

Background only. User never waits. Triggered by `EVENT_NOTE_SAVED`.

```
Trigger: note or task saved to disk/SQLite
         │
         └─► tokio::spawn(async {
               ram_check() → skip if under threshold, push to retry queue
               │
               VaultLoader (impl Loader)
               Reads file from vault dir, sets metadata:
               { note_id, parent_title, is_secure }
               │
               ▼
               Closure: is_secure == true? → Err(NodeSkipped) → ONNX never runs
               │
               ▼
               filter_cached(redb NodeCache)
               hash(path + content) unchanged? → skip entirely
               │
               ▼
               ChunkMarkdown (notes only)
               Splits at heading boundaries ≤500 tokens
               Metadata propagates automatically to all child nodes
               Tasks: skip — one embedding per task, no chunking
               │
               ▼
               then_in_batch(FastEmbed::try_new_from_user_defined())
               Batch ONNX inference
               Output: each node gets embedding: Vec<f32> (384 dims, full)
               │
               ▼
               VaultStorage (impl Persist)
               DELETE old chunks WHERE source_id = note_id
               INSERT { chunk_id, source_type, source_id, chunk_text,
                        heading_context, embedding_vector, bm25_text }
               batch_size: Some(20) for mobile RAM
             })
```

**What regex does here:** Nothing. Ingestion is structural.
**What embedding does here:** Converts chunks to 384-dim vectors for storage.

---

### Digestion (read path)

Fast. No model loading. Pure Rust. Triggered before every chat inference call.

```
User sends message: "What were the blockers from [[Project Alpha]]?"
         │
         ▼ retrieval.rs (NOT Swiftide query pipeline)
         │
         STEP 1: Regex scan — zero cost, instant
         regex_scan(message) →
           [[Project Alpha]] → direct velesdb lookup by title
           #planning → all chunks tagged #planning
           @team-member → notes mentioning that person
         Returns: exact_matches Vec<SourceRef>
         │
         STEP 2: Semantic embed — one call, full prompt
         fastembed.embed(&[message]) → [384-dim vector]
         ONE embed for the entire prompt, never per-word
         │
         STEP 3: Hybrid search — velesdb
         velesdb.search(vector, top_k=4, exclude_secure=true)
         Returns: Vec<Chunk> by combined BM25 + cosine score
         │
         STEP 4: Build ContextBundle
         ContextBundle {
           exact_refs: Vec<SourceRef>,    // from regex
           semantic_chunks: Vec<Chunk>,   // from vector search
         }
         │
         STEP 5: Format context window
         context_str = format_for_lfm(exact_refs, semantic_chunks)
         enriched_msg = format!("{}\n\nUser: {}", context_str, message)
         │
         ▼ Goes into LEAP inference
```

**Why not Swiftide query pipeline here:**
Swiftide's query pipeline is designed for multi-step retrieval with sub-question generation — every non-retrieval stage requires an LLM call. For pre-inference context assembly, that's two LLM calls (sub-questions + answer) where one is needed. `retrieval.rs` at ~50 lines does exactly what's required. The Swiftide query pipeline is useful *inside* a `#[tool] fn search_vault(...)` when the agent calls it from within a loop — not on the hot chat path.

---

## STORAGE ARCHITECTURE

```
SOURCE OF TRUTH (tauri-plugin-sql → SQLite)
├── notes
│   columns: id, title, file_path, created_at, updated_at, tags, is_secure
│   file: .md file on disk is canonical content
│
├── tasks
│   columns: id, title, description, status, due_date, project_id, created_at
│   note: NO .md file. Tasks are structured records, not documents.
│         Kanban view rendered from SQL, not by parsing .md
│
├── projects
│   columns: id, name, color, created_at
│
├── conversation_sessions
│   columns: id, created_at, leap_conversation_id, model_used, last_active
│
└── conversation_messages
    columns: id, session_id, role, content, timestamp, has_tool_calls

RAG INDEX (tauri-plugin-velesdb)
└── chunks
    {
      chunk_id,
      source_type,      # "note" | "task" | "chat_summary"
      source_id,        # FK → SQLite id
      chunk_text,       # raw text
      heading_context,  # note heading, preserved from ChunkMarkdown
      embedding_vector, # 384-dim f32 (all-MiniLM, full dims)
      bm25_text         # for hybrid search
    }

FILE SYSTEM (tauri-plugin-fs / std::fs)
└── ~/vibo/vault/*.md   # note files only, canonical content
```

**Update contracts:**
- Note edited → write `.md` + update SQLite `updated_at` → reindex via ingestion pipeline (redb skips unchanged chunks)
- Task status toggled → SQLite only, no re-embed
- Task description edited → SQLite + queue re-embed
- Session resumed → `SELECT messages WHERE session_id = last_active` → `leap.create_conversation_from_history(msgs)`

---

## CHUNKING MAP

| Content | Strategy | Chunk unit | Embed unit | Notes |
|---|---|---|---|---|
| Notes (`.md`) | `ChunkMarkdown` | Per heading section ≤500 tokens | Each chunk | Heading travels as metadata |
| Tasks | None — parse to SQLite | N/A | `title + description` as single node | Atomic. One embed per task. |
| Chat messages | Not chunked at write time | N/A | Not embedded | Recalled by SQL recency |
| Chat summaries | Single node | N turns → 350M summary → embed | Summary as note-like entry | Background, queued, v1.1+ |
| Code (GitHub) | `ChunkCode` (tree-sitter AST) | Per function/class | Each chunk | v2 only |

---

## MODEL BENCHMARK REALITY

```yaml
LFM2.5_1.2B_Instruct:
  IFEval: 74.89%  # instruction following — strong
  GSM8K: 58.30%   # math — moderate
  recommended_for: agentic tasks, data extraction, RAG, tool calls, chat
  NOT_recommended_for: knowledge-intensive, programming
  context: 32K tokens, <1GB RAM at Q4

LFM2.5_350M:
  IFEval: 65.12%  # beats Llama-3.2-1B (52.39%) at 1/3 params
  GPQA: 27.46%    # beats Qwen3-0.6B (22.14%)
  speed: 88 tok/s on iPhone 13 Mini, single core, 56MB RAM
  recommended_for: >
    data extraction, structured outputs, tool use — exactly
    background enrichment: hypothetical query gen for chunks
    task description parsing from natural language
  NOT_recommended_for: math, programming, creative writing, open-ended chat
  fine_tune_signal: >
    #1 ranked in tunability across 15 models.
    Best target for Vibo-specific fine-tuning later.
    Data extraction + structured output tasks absorb training signal better
    than any model 20x its size.

assignment_clarity:
  350M_does_NOT_handle:
    - Intent classification → deterministic Rust (regex + keyword, not LLM)
    - Tool calls in chat → 1.2B Instruct
    - Any hot path operation
  350M_handles:
    - Background: hypothetical query generation per chunk (Swiftide step)
    - Background: task description → structured fields (NL parsing, queued)
    - Future: fine-tuned narrow tasks
```

---

## COMPLETE FLOW DIAGRAMS

### Flow 1 — Note save + background index

```
[TSX]                      [Rust Core]                    [Background]
  │                             │
  ├─invoke('create_note',──────▶│
  │  {title, content})          ├─ std::fs::write(vault/id.md)
  │                             ├─ sqlx INSERT notes table
  │◀─Ok(note_id)────────────────┤  ← IPC closes. UI unlocked immediately.
  │  "Saved ✓"                  │
  │                             ├─ tokio::spawn(async move {
  │                             │    if ram_available() {
  │                             │      Pipeline::from(VaultLoader)
  │                             │        .filter(is_secure)
  │                             │        .filter_cached(redb)
  │                             │        .then_chunk(ChunkMarkdown)
  │                             │        .then_in_batch(Embed::new(fastembed))
  │                             │        .then_store_with(velesdb)
  │                             │        .run().await
  │                             │    } else {
  │                             │      index_queue.push(note_id)
  │                             │    }
  │                             │  })
```

---

### Flow 2 — Chat message (full hot path)

```
[TSX]                [Rust]            [retrieval.rs]   [LEAP]     [Frontend]
  │                    │
  ├─invoke('chat',────▶│
  │  {msg, session_id})│
  │                    ├─tokio::spawn───────────────────────────────────────────┐
  │                    │               │                                        │
  │                    │               ├─ regex_scan(msg) → exact refs          │
  │                    │               ├─ fastembed.embed([msg]) → 384-dim vec  │
  │                    │               ├─ velesdb.search(vec, k=4) → chunks     │
  │                    │               └─ ContextBundle assembled               │
  │                    │                                                        │
  │                    ├─ tool_routing(msg)                                     │
  │                    │   → select ≤3 schemas (deterministic Rust)             │
  │                    │   → build system_prompt + tool_schemas                 │
  │                    │                                                        │
  │                    ├─ get/create conversation                                │
  │                    │   session exists? use leap_conversation_id             │
  │                    │   new session? leap.create_conversation(system_prompt) │
  │                    │                                                        │
  │                    ├─ leap.generate(conv_id, enriched_msg)────────▶         │
  │◀─emit(chat_token)◀─┤◀─ onLeapEvent('generation-chunk', {chunk}) ◀─┤        │
  │  renders stream     │                                               │        │
  │                    │◀─ onLeapEvent('generation-complete') ◀─────────┤        │
  │                    │                                                        │
  │                    ├─ tool_call detected? (max 3 iterations)                │
  │                    │   ├─ parse tool_name + args                            │
  │                    │   ├─ dispatch_tool() → pure Rust                       │
  │                    │   │   create_note / search_vault / calendar / github   │
  │                    │   ├─ result → format as tool response message          │
  │                    │   └─ leap.generate(conv_id, tool_result) → loop        │
  │                    │                                                        │
  │                    └─ sqlx INSERT conversation_messages                     │
```

---

### Flow 3 — Nano background agent (parallel tools)

```
EVENT_NOTE_SAVED (no #enhanced tag, RAM available)
         │
         ▼ DB: INSERT background_tasks { type: enhance, target: note_id, status: pending }
           BEFORE spawn (survives OS kill)
         │
         ▼ tokio::spawn(nano_agent)
         │
         ▼ swiftide_agents::Agent
             LeapBridge → LFM2.5-350M (load if not resident)
             before_all hook: check DB for resume state

         ─── LLM TURN 1 ───────────────────────────────
         system: "You enhance notes with vault tags.
                  Tools: read_note, list_vault_tags, update_note"
         user: "Enhance note {id}"
         │
         LFM2.5-350M emits:
         [read_note(id=note_id), list_vault_tags()]
         │
         ▼ Swiftide executes BOTH in parallel (tokio::join!)
         read_note → file content
         list_vault_tags → existing tag set
         │
         ─── LLM TURN 2 ───────────────────────────────
         LFM2.5-350M receives: note content + tag set
         LFM2.5-350M emits:
         [update_note(id=note_id, content="...enhanced...")]
         │
         ▼ Swiftide executes update_note (sequential, write op)
         │
         ─── LLM TURN 3 ───────────────────────────────
         LFM2.5-350M: ToolOutput::Stop
         │
         ▼ on_stop hook: UPDATE background_tasks SET status=done
         ▼ unload 350M if RAM pressure
         │
         ▼ EVENT_NOTE_SAVED fires again
           → ingestion pipeline re-indexes the enhanced note (redb detects change)
```

---

### Flow 4 — App suspend / resume

```
SUSPEND (iOS: applicationWillResignActive):
  Tauri RunEvent::WindowEvent or lifecycle signal
    → CancellationToken.cancel() → halts background indexing streams
    → sqlx flush WAL, close connections
    → leap.unload_model() → frees ~900MB resident
    → velesdb: flush pending writes
    → background_tasks: leave as pending → survives kill

RESUME:
    → leap.load_cached_model(LFM2.5-1.2B-Instruct)
    → SELECT sessions WHERE last_active = most_recent
    → SELECT messages WHERE session_id = ?
    → leap.create_conversation_from_history(messages)
    → drain index_queue (RAM check per item)
    → drain background_tasks WHERE status = pending
```

---

## INTEGRATION MAP

```yaml
internal_tools_always_hot:
  - create_note(title, content, tags) → vault write + sqlx
  - read_note(id) → std::fs read
  - search_vault(query) → retrieval.rs mini (embed + velesdb)
  - list_notes(filter) → sqlx SELECT
  - update_note(id, content) → vault write + sqlx + reindex queue
  - create_task(title, description, due_date) → sqlx
  - update_task(id, status, description) → sqlx
  - list_tasks(filter, project_id) → sqlx

external_integrations:
  claude_ai:
    type: "cloud LLM fallback"
    transport: "reqwest → Anthropic API (OpenAI-compat endpoint)"
    swiftide: "async-openai integration, user opts in explicitly"
    mobile: true

  google_maps:
    type: "location context"
    transport: "reqwest HTTPS → Maps REST API"
    impl: "#[tool] fn get_location(query: String) → ToolOutput"
    mobile: true

  gmail:
    transport: "reqwest + OAuth2 → Gmail REST API"
    impl: "#[tool] fn read_emails / send_email"
    oauth_storage: tauri-plugin-stronghold (encrypted)
    mobile: true

  google_calendar:
    transport: "reqwest + OAuth2 → Calendar REST API"
    impl: "#[tool] fn list_events / create_event"
    mobile: true

  notion:
    transport: "rmcp HTTP client → official Notion MCP server (HTTPS)"
    impl: "#[tool] fn notion_query / notion_create_page (wraps rmcp call)"
    mobile: true  # outbound HTTPS only

  github:
    transport: "rmcp HTTP client → official GitHub MCP server (HTTPS)"
    impl: "#[tool] fn github_search / create_issue / list_prs"
    mobile: true

  whatsapp:
    transport: "reqwest HTTPS → WhatsApp Business REST API"
    note: "NO MCP server exists for WhatsApp — direct REST only"
    impl: "#[tool] fn send_whatsapp(to, body)"
    mobile: true

tool_schema_injection_rule:
  max_per_turn: 3
  selection: "deterministic Rust routing — NOT an LLM call"
  logic: |
    match intent_signals(&message) {
      has_calendar_signal  → inject calendar schema
      has_note_signal      → inject note tools
      has_task_signal      → inject task tools
      has_github_signal    → inject github schema
      has_location_signal  → inject maps schema
      _                    → inject note tools (default)
    }
```

---

## CUSTOM BUILD LIST — HONEST SCOPE

```yaml
must_build:
  LeapBridge:
    what: "impl ChatCompletion for LeapBridge { app: AppHandle }"
    why: "LEAP is not HTTP. No existing Swiftide integration."
    complexity: medium
    blocker: "Phase 0: verify LEAP plugin Rust API surface (state extension trait vs IPC)"
    note: >
      Two patterns possible:
      A) leap_plugin_state.complete(messages) if plugin exposes state
      B) tokio oneshot channel bridge if plugin is IPC-only
      Must check before writing a line of LeapBridge.

  VaultLoader:
    what: "impl Loader for vault file system"
    why: "Swiftide FileLoader reads current dir. Vibo reads ~/vibo/vault/ with custom metadata."
    complexity: low (~30 lines)

  VaultStorage:
    what: "impl Persist for velesdb custom schema"
    why: "Swiftide has no velesdb integration. Schema has is_secure, heading_context."
    complexity: medium
    note: "batch_size() → Some(20) for mobile RAM"

  retrieval_rs:
    what: "regex scan + fastembed.embed + velesdb search → ContextBundle"
    why: "Direct path, ~50 lines, Swiftide query pipeline is overkill here"
    complexity: low

  tool_router:
    what: "Rust function: intent_signals(&str) → Vec<ToolSchema>"
    why: "LLM must not see all 15+ schemas at once — Rust selects ≤3"
    complexity: low

  model_manager:
    what: "load/unload/resume for 1.2B + 350M across lifecycle events"
    why: "LEAP handles per-model state. ModelManager is Vibo's policy layer."
    complexity: medium

  background_task_queue:
    what: "SQLite-backed queue with RAM check, retry, and crash recovery"
    why: "tokio queue doesn't survive OS kill. DB-backed queue does."
    complexity: medium
    schema: "background_tasks { id, type, target_id, status, retry_count, created_at }"

reuses_without_modification:
  - swiftide indexing pipeline (ChunkMarkdown, FastEmbed integration, filter_cached)
  - swiftide-agents (DefaultContext, parallel tool dispatch, lifecycle hooks)
  - fastembed (model loading, batching, tokenization)
  - redb (Swiftide NodeCache, zero config)
  - tokio (runtime, already in Tauri)
  - sqlx (async SQLite)
  - reqwest (HTTP for external integrations)
  - rmcp (MCP HTTP client for Notion, GitHub)
  - tauri-plugin-velesdb
  - tauri-plugin-sql
  - tauri-plugin-fs
  - tauri-plugin-tracing
```

---

## OPEN ITEMS — ENGINEERING HONEST LIST

```yaml
must_verify_in_phase_0:
  - "LeapBridge: does tauri-plugin-leap-ai expose Rust state via extension trait,
     or only via Tauri IPC commands? This determines bridge pattern."
  - "LFM2.5-350M: confirm .bundle in LEAP model catalog (vs requires bundling CLI)"
  - "velesdb: compile test for arm64-apple-ios target"
  - "Android LEAP SDK: not fully validated. iOS-first until confirmed."

must_decide_before_build:
  - "OAuth token storage: tauri-plugin-stronghold or OS keychain plugin?"
  - "Background queue RAM threshold: what number triggers skip vs proceed?
     Suggestion: <400MB available → skip, retry when next NOTE_SAVED fires"
  - "350M load policy: never on devices ≤3GB RAM total?"

hard_deferred_to_v2:
  - Chat summary rolling embeddings (long-term memory)
  - ChunkCode + AST for GitHub integration
  - LFM2.5-Thinking for deep reasoning tasks
  - LFM2.5-350M fine-tuning on Vibo-specific extraction tasks
  - ModernBERT embedding upgrade
```

---

## THE STACK — ONE PLACE, NO AMBIGUITY

```yaml
final_stack:
  runtime: Tauri 2.0
  frontend: TSX + React + shadcn/ui + Bun
  orchestration: Swiftide RS (indexing + Nano agents)
  inference: tauri-plugin-leap-ai (all LLM calls, no exceptions)
  embedding: FastEmbed-rs via Swiftide → all-MiniLM-L6-v2 ONNX (384-dim, bundled)
  vector_store: tauri-plugin-velesdb (hybrid BM25 + cosine)
  pipeline_cache: redb (Swiftide NodeCache, embedded)
  sql: tauri-plugin-sql (SQLite via sqlx)
  file_io: tauri-plugin-fs + std::fs
  http: reqwest (external integrations)
  mcp_client: rmcp (remote HTTPS only, for Notion + GitHub)
  logging: tauri-plugin-tracing
  models:
    resident: LFM2.5-1.2B-Instruct
    nano: LFM2.5-350M (lazy, background only)
    embedding: all-MiniLM-L6-v2 (always resident)
  not_in_stack:
    - Redis, any server process, any sidecar, any subprocess
    - Local MCP servers
    - Swiftide query pipeline on interactive chat path
    - Matryoshka truncation on all-MiniLM
    - "Specialist" third model
    - sqlite-vec
    - Any hardcoded sequential tool batching
```

# Phase 0.7-B Plan — Agent Tool-Calls, Skill Injection, Per-Model Config

**Date:** 2026-04-28
**Status:** ✅ SIGNED OFF 2026-04-28 — Q1–Q7 locked (see §7). Executing Phase A next.
**Scope:** Make the agent (Junior / Specialist) actually *do* things — create notes, search the vault, eventually browse — while injecting role+skill+skill-index at every session start. Plus: per-model config files, vision/thinking modes, and a sandbox so the agent can only touch `myspace/`.

**Operating rules for this plan (per user instructions 2026-04-28):**
- Feature-by-feature, not big push.
- Scout before any change. (✅ done — see §2.)
- Don't *kill* code that may be needed later. Comment-out / archive instead of delete.
- Always present a TODO plan, get approval, then execute.

---

## 1. Context — what this plan resolves

User asked, after the dead-code report + 6 templates landed (commit `40698e8`):

1. **Fix the retrieval.rs test errors surfaced by `cargo check --tests`** without killing the file (it's Phase-1 SQL-RAG scaffolding, not dead).
2. **Inventory what's missing** for the agent to:
   - create notes / tasks
   - perform a web search
3. **Design the injection mechanism** for role + skill + skill-index into the active agent's context, *and* a parallel index for tools / commands / MCP / Tauri plugins.
4. **Sandbox the agent** to `myspace/` only.
5. **Per-model config files** so we don't keep tweaking `temperature`, `top_k`, `n_ctx`, etc. ad-hoc.
6. **Auto-vision mode** for `LFM2.5-VL-450M-Q4_0`. **Auto-thinking mode** for `LFM2.5-1.2B-Thinking-Q4_K_M`. Auto-deactivate when the active model changes.

---

## 2. Scout findings (already done — read-only)

| Area | Reality on disk | Implication |
|---|---|---|
| `src-tauri/src/services/retrieval.rs` (145 LoC) | Header reads *"This is a placeholder. Phase 1 v1: SqlContextProvider (lightweight RAG)"*. Trait-implements `RetrievalProvider` from `services/context.rs`. Body is `// TODO:` with the planned SQLite query shape. No live caller yet. | **NOT dead.** Preserve. The 5 test errors are scaffolding drift, not the file's fault. |
| `src-tauri/src/services/context.rs` | Defines `ContextRequest`, `ContextBundle`, `RetrievalProvider` trait, `EmptyProvider` test stub. | Interface contract for the future RAG. Keep. |
| `src-tauri/src/commands/workspace.rs` | 7 commands: `load_workspace_snapshot`, `save_note`, `delete_note`, `create_folder`, `save_column`, `export_notes`, `delete_column`. Each takes `caller: CallerContext` (User / Agent { agent_id }). | **The agent-call audit hook is already wired into the type system.** No new Tauri commands needed for note creation — we wrap these. |
| `src-tauri/src/commands/inference.rs` | 11 `viboinference_*` commands (list/download/delete/active model + start/stream/stop/end chat session + session_info). | Streaming chat works; tool-call layer **does not exist**. |
| `src/lib/inference.ts` | Frontend wrapper, pure text in/out. | Same — no tool-call layer on FE. |
| `src/components/ChatAssistant.tsx` | Streams text; no `<tool>` parsing, no function-call branch. | Need to add parse-and-display affordance for tool calls. |
| `src-tauri/src/services/llama.rs` (16 451 bytes) | Has full sampler chain (`top_k`, `top_p`, `temp`, `repeat_pen`, `seed`, `n_ctx`). Reads embedded `chat_template` from GGUF. **No mmproj / vision plumbing.** | Per-model sampler swap is a 1-line fetch. Vision is real work (new C++ FFI calls). |
| `src-tauri/src/services/llama_config.rs` | `SamplerConfig` + `SessionConfig` with sensible defaults. `system_reserve = 8 192` is **explicitly documented** as: *"agent persona file (~1 k) + hot context chunk (~1 k) + tool definitions (~1 k) + skills index (~1 k) + headroom"*. | The token budget for our injection design is **already reserved in code**. We just need to fill it. |
| `src-tauri/src/services/model_catalog.rs` | `ModelEntry` already has `alias` and `ModelLimits { supports_vision, mmproj_url: Option<&str> }`. Catalog comment marks `inspector` (VL) as deferred because plugin:leap-ai 0.1.1 only handles single-URL downloads. Thinking model not yet listed. | Vision flag exists; we just haven't shipped a model with it set. Thinking model must be added. |
| `RESERVED_FOLDER_NAMES` in `vault/mod.rs` | `["notes", "tasks", "agents", "skills", "roles", "providers", "tools", "mcp", "plugin"]` | **No `models/` folder reserved.** If we want per-model `.md` config, either add the folder or repurpose `providers/`. |
| Sandboxing | **Zero** path-validation in vault/commands. No `canonicalize`, no `starts_with(myspace)` check. | Whatever path the caller passes is trusted. Adding agent-issued FS calls without a validator = security gap. |
| Internet access | No `http:` permission, no `tauri-plugin-http`, no fetch helper. `reqwest` only used in `providers/mod.rs:172` for cloud LLM streaming. | Zero today. |

---

## 3. Open architecture questions (must answer before §4 code)

These are choices the user owns. Plan stays in DRAFT until each is picked.

### Q1 — Per-model config: where does it live?

**A.** Extend Rust `ModelEntry` with `Option<SamplerConfig>` + `Option<ModelLimits>` overrides. User-invisible. *Pros: simple; one source of truth; no new I/O.* *Cons: requires a rebuild to tweak temperature.*

**B.** Add `myspace/models/<id>.md` (one frontmatter file per model) — bootstrapped from a Rust template, app reads it at startup, falls back to baked-in defaults if absent. User-editable. *Pros: matches the "user can edit skills" pattern; consistent with role/skill design.* *Cons: requires new reserved folder + new bootstrap code.*

**C.** Hybrid: ship Rust defaults inside `model_catalog.rs`; allow `myspace/models/<id>.md` overrides if present. *Pros: best of both.* *Cons: one more code path to maintain.*

> **Recommendation: B**, because it's consistent with the role/skill bootstrap we already shipped templates for. Add `models` to `RESERVED_FOLDER_NAMES`. One new template per shipped model. Same write-once-on-bootstrap lifecycle.

### Q2 — Tool-call protocol: how does the model invoke a tool?

**A.** **Structured-tag parsing** (recommended, ships now). The model emits `<tool name="save_note">{json}</tool>` inside its output stream. Backend `tool_dispatcher.rs` regex-tokenizes the stream, intercepts tags, dispatches to the matching Tauri command, appends `<tool_result>{json}</tool_result>` back to the conversation, resumes generation. The role.md tells the model the protocol; the skill.md gives examples. *Pros: works on any model that follows instructions; no special inference-side support needed.* *Cons: model might mis-format; need a strict parser + retry-on-malformed.*

**B.** **JSON function calling** (model-native). LFM 2.5 instruct may or may not support OpenAI-style function-calling templates. Need experiment. If supported, more reliable; if not, falls back to A.

**C.** **MCP-over-stdin** (overkill for now). The agent talks MCP to a local server in our process. Future-proof but heavy.

> **Recommendation: A**, with a stub interface that lets us swap to B later if LFM supports it natively. Junior runs the most-constrained version (3 tools max in the system prompt).

### Q3 — Index hierarchy: one mega-index or layered?

You said *"index for tool, command, mcp, tauri, tauri plugins or whatever call it needs … we just make the index and then we make the files."*

**A.** **Layered (recommended).** Top-level `myspace/INDEX.md` (the agent's "table of contents") points to four sub-indexes:
- `myspace/skills/INDEX.md` (auto-generated from skill frontmatter)
- `myspace/tools/INDEX.md` (auto-generated from Rust tool registry)
- `myspace/plugin/INDEX.md` (auto-generated from `src-tauri/src/plugins/active/*.md`)
- `myspace/mcp/INDEX.md` (placeholder, empty until MCP support lands)

Each sub-index lists items with one-line descriptions. Per-item `.md` files live alongside.

**B.** One single `myspace/CAPABILITIES.md` with all four categories. Easier to grep. Worse for token budget when injected.

> **Recommendation: A.** Token-cheap for the always-on injection (only the top-level `INDEX.md` goes in the system prompt — it's under 50 lines). Sub-indexes load on demand via the `index` skill / `tools-index` skill we already wrote templates for.

### Q4 — Tool registry: hand-authored or generated?

**A.** **Auto-generate** `myspace/tools/INDEX.md` and per-tool .md files from a Rust-side `pub static TOOLS: &[ToolDef]` registry on every launch. Same lifecycle as plugin docs (mirror-overwrite, app-owned, never user-edited). *Pros: never drifts from code; one source of truth.* *Cons: ~150 LoC for a `tools_registry.rs` module.*

**B.** Hand-author the .md files, validate at startup that every advertised tool actually corresponds to a `#[tauri::command]`. *Pros: more readable, tighter prose.* *Cons: drifts; one more thing to update on every command rename.*

> **Recommendation: A.** With per-tool `.md` files written in plain markdown (Rust just owns the catalog + serializes it).

### Q5 — Sandboxing: how strict for v1?

**A.** **Path validator** in vault FS commands: when called by `CallerContext::Agent`, canonicalize input and require `starts_with(myspace_dir)`. Reject otherwise. *Pros: minimal; one helper function.* *Cons: only covers vault commands — agent could still hit `read_file` if we expose it.*

**B.** **Capability list per role.md** — role frontmatter declares `allowed_tools: [...]`; dispatcher checks before invoking. Belt-and-suspenders with A. *Pros: defense in depth; lets us ship Specialist-only tools.* *Cons: more wiring.*

> **Recommendation: A + B together.** A is the FS-level guarantee; B is the high-level whitelist. The 6 templates already have `equipped_tools:` frontmatter — B is just enforcing that.

### Q6 — Vision / Thinking modes: how do we model them?

**Vision (LFM2.5-VL-450M-Q4_0):**
- Real work. Requires loading the mmproj projector shard alongside the GGUF, plus new `llama-cpp-2` calls (or `mtmd_*` C bindings) to encode an image into image-tokens before generation.
- `tauri-plugin-leap-ai 0.1.1` `download_model` is single-URL only — won't fetch the mmproj. Either: (a) call HF download directly via `reqwest` (small new code path), or (b) wait for the plugin to support paired downloads.
- `ModelLimits.supports_vision` flag already exists. Setting it to `true` is the activation signal: `start_session` checks it, loads mmproj, exposes a new `attach_image` step.

**Thinking (LFM2.5-1.2B-Thinking-Q4_K_M):**
- Easier. The model produces `<think>…</think>` blocks before its visible answer. No special inference-side code — just **parse the stream**, route `<think>…</think>` to a different UI lane (collapsible "reasoning" panel), and pass the rest to the visible chat.
- "Auto-activate" = on `set_active_model`, check `model.modes.thinking`; if true, FE stream parser splits `<think>` blocks; if false, no-op.

**Auto-deactivate:** both modes are model-bound. When `set_active_model` changes the active model, the runtime re-reads `ModelLimits` / `model.modes` from the catalog. No persistent toggle in user state — modes follow the model.

> **Recommendation:**
> - Add `LFM2.5-1.2B-Thinking-Q4_K_M` to the catalog with `modes.thinking = true`. Ship the `<think>` parser. **(~half day)**
> - Add `LFM2.5-VL-450M-Q4_0` to the catalog with `supports_vision = true` + `mmproj_url = Some(...)`. Mark `enabled = false` until §4-Phase-E lands. **(~1 day for the mmproj download path + image-token encoding once we touch it.)**

### Q7 — Web search: which provider?

User confirmed earlier: **not Brave (paid)**. Free Rust-friendly options:
- **DuckDuckGo HTML** (no key, fragile)
- **Tor-routed search** (privacy-first, complex)
- **Firecrawl** (good for scraping after we have URLs)

> **Recommendation:** scope the tool surface now (`web_search(query) → [{title, url, snippet}]`), defer the actual provider choice + implementation to Phase 0.7-C. Listed in §4 as a stub only.

---

## 4. Critical-path plan (assuming the recommended answers above)

Ordered by dependency. Each phase is independently committable. **No phase touches the next phase's files.**

### Phase A — `retrieval.rs` test fix (no behavior change)

**Goal:** unblock `cargo check --tests`. Preserve the SQL-RAG scaffolding.

**Touch:**
- `src-tauri/Cargo.toml` — add `tokio = { version = "1", features = ["macros", "rt"] }` to `[dev-dependencies]`.
- `src-tauri/src/services/retrieval.rs` lines 114–115 — fix field types:
  - `user_prompt: None` → `user_prompt: String::new()`
  - `selected_note_ids: None` → `selected_note_ids: Vec::new()`

**Don't touch:** the `// TODO:` comments. The trait. `context.rs`. The `SqlContextProvider` body.

**Verify:** `cargo check && cargo check --tests` both green.

**Risk:** ~zero. Pre-existing test scaffolding, two-line type fix.

### Phase B — Per-model config files (Q1 = B)

**Goal:** ship per-model sampler/limits as user-editable `myspace/models/<id>.md` files. Defaults live in Rust, files seed from defaults.

**Touch:**
- `src-tauri/src/vault/mod.rs` — append `"models"` to `RESERVED_FOLDER_NAMES`.
- `src-tauri/src/templates/myspace/models/lfm2.5-350m.md` (new — frontmatter + commentary).
- `src-tauri/src/templates/myspace/models/lfm2.5-1.2b-instruct.md` (new).
- `src-tauri/src/templates/myspace/models/lfm2.5-1.2b-thinking.md` (new — adds the new model).
- `src-tauri/src/templates/myspace/models/lfm2.5-vl-450m.md` (new — `enabled: false`).
- `src-tauri/src/services/model_catalog.rs` — add 2 entries for thinking + VL. Set `modes.thinking = true` and `supports_vision = true` respectively. Mark VL `enabled: false` until phase E.
- `src-tauri/src/services/llama_config.rs` — extend `ModelEntry` to optionally hold a `SamplerConfig` override. (Or add a parallel registry.)

**Don't touch:** the seed template lifecycle code (Phase C will handle bootstrap).

**Verify:** entries appear in `viboinference_list_models`. Frontend shows the two new entries.

**Risk:** medium — touches catalog + adds new reserved folder. Need a unit test that all aliases stay unique.

### Phase C — Bootstrap injection: seed user templates + index regeneration

**Goal:** on first onboarding (and on factory-reset), the app writes the role/skill/model templates into `myspace/` and regenerates the indexes.

**Touch:**
- `src-tauri/src/vault/mod.rs` — add `seed_user_templates()`, `mirror_plugin_docs()`, `regenerate_indexes()`. Wire all three after `ensure_vault_dirs()` in `lib.rs:93`.
- `src-tauri/src/vault/seeds.rs` (new) — `pub static SEEDS: &[(&str, &str)]` populated via `include_str!()` of every `templates/myspace/**/*.md`.
- `src-tauri/src/lib.rs:93` — call the new functions.

**Don't touch:** anything outside `vault/`. No frontend change.

**Verify:** delete `~/Library/Application Support/com.viboai.app/myspace/`, relaunch, see the templates appear; check `myspace/INDEX.md`, `myspace/skills/INDEX.md`, etc. were written.

**Risk:** medium. New code paths run on every launch — must be idempotent + cheap.

### Phase D — Agent context assembly: role + skill + index injection

**Goal:** when a chat session starts, the system prompt is assembled from the active alias's role.md + skill.md + the top-level `INDEX.md`. ~3 k tokens, fits inside `system_reserve`.

**Touch:**
- `src-tauri/src/services/agent_context.rs` (new ~150 LoC) — `build_system_prompt(vault_dir, alias) -> String`, `list_skills_for_role(...)`, `list_tools_for_role(...)`.
- `src-tauri/src/commands/inference.rs` — `viboinference_start_chat_session` accepts an optional `alias` parameter (default `"junior"`); passes assembled prompt down.
- `src/lib/inference.ts` — pass `alias` through.
- `src/components/ChatAssistant.tsx` — minimal role-picker chip in the header (default Junior).

**Don't touch:** the actual inference/streaming code. The skill/role markdown bodies (we wrote them already).

**Verify:** start a session as Junior, ask "what skills do you have?" — agent reads the injected INDEX, replies with the list. Switch to Specialist, ask the same — see longer/structured response.

**Risk:** medium. The system prompt is the lever — too long → bad chat; too short → agent forgets the skill index. Test with both alias defaults.

### Phase E — Tool-call dispatch (minimum viable)

**Goal:** Junior can `save_note`, `search_notes`, `read_note`. Tool calls flow through a dispatcher that enforces sandboxing.

**Touch:**
- `src-tauri/src/services/tool_dispatcher.rs` (new ~250 LoC) — protocol:
  - registry of `ToolDef { name, description, schema, handler: fn }`
  - stream parser that recognizes `<tool name="…">{json}</tool>`
  - dispatcher that validates `equipped_tools` against `role.md` frontmatter, runs handler, formats `<tool_result>…</tool_result>`, re-injects, resumes generation.
- `src-tauri/src/services/tools_registry.rs` (new ~200 LoC) — populates `myspace/tools/INDEX.md` + per-tool `.md` files at launch.
- `src-tauri/src/services/path_guard.rs` (new ~50 LoC) — `assert_inside_vault(path, vault_dir, caller)`. All vault commands gain `caller.is_agent() => path_guard::check(...)`.
- `src-tauri/src/commands/workspace.rs` — wrap existing commands with the guard for agent callers (no public-API change for human callers).
- `src-tauri/src/commands/inference.rs::viboinference_stream_chat` — pipe stream through dispatcher.
- `src/components/ChatAssistant.tsx` — render tool calls / results inline (collapsible).

**Don't touch:** model_catalog, llama_config, retrieval.rs.

**Verify:** in chat, ask Junior "save a note titled 'test' with body 'hello'". Watch the `<tool>` call appear, see the file land in `myspace/notes/`, see Junior confirm.

**Risk:** highest of any phase. Stream mid-flight injection is fragile. Ship behind a feature flag (`agent_tool_calls_enabled`). Default off.

### Phase F — Vision + Thinking mode plumbing

**Sub-phase F1: thinking** (cheap)
- FE-only stream parser splits `<think>…</think>` → "Reasoning" lane.
- Already covered by F's catalog entry having `modes.thinking = true`.

**Sub-phase F2: vision** (real C++ work)
- New: `download_mmproj` helper (single-URL via `reqwest`).
- New: `llama.rs::load_model` accepts optional `mmproj_path`.
- New: `llama.rs::encode_image(session, png_bytes) -> ImageTokens`.
- New: `viboinference_attach_image(session_id, png_bytes)` Tauri command.
- New: `<image>…</image>` token in chat stream → image upload UI affordance.

> Defer F2 until A–E ship.

### Phase G — Web search tool (defer until E ships)

Stub interface only. Implementation = Phase 0.7-C.

---

## 5. Files this plan will touch by phase

| Phase | New files | Edited files | Deleted | LoC est. |
|---|---|---|---|---|
| A | — | Cargo.toml, retrieval.rs (test only) | — | ~5 |
| B | 4 templates, model_catalog entries | model_catalog.rs, llama_config.rs, vault/mod.rs | — | ~250 |
| C | seeds.rs | vault/mod.rs, lib.rs | — | ~150 |
| D | agent_context.rs | inference.rs (cmd), inference.ts, ChatAssistant.tsx | — | ~250 |
| E | tool_dispatcher.rs, tools_registry.rs, path_guard.rs, ~7 tool .md | workspace.rs, inference.rs (cmd), ChatAssistant.tsx | — | ~600 |
| F1 | — | ChatAssistant.tsx | — | ~50 |
| F2 | mmproj_download.rs | llama.rs, model_catalog.rs, ChatAssistant.tsx | — | ~500 |
| G | web_search.rs (stub) | tools_registry.rs | — | ~80 |

**Total new/edited LoC if all phases ship: ~1 900.** Spread over 6+ commits.

---

## 6. What this plan deliberately does NOT do

- ❌ No deletion of "looks dead" code without a follow-up dead-code report. Anything that smells dead gets archived (`// LEGACY: kept for Phase X` comment) or moved to `Guidelines/archive/code/`, never `rm`'d.
- ❌ No frontend redesign. Role-picker chip is the only new UI surface in this plan; everything else is invisible.
- ❌ No internet access wired in. Web search tool is *registered* (so the index has a slot) but throws "not implemented" until 0.7-C.
- ❌ No Stronghold change. Encryption stays as-is.
- ❌ No mobile work. `cfg(target_os = "ios" | "android")` paths untouched.

---

## 7. Sign-off — LOCKED 2026-04-28

- [x] **Q1** — Per-model config: **B** (user-editable `myspace/models/<id>.md`, write-once; Rust catalog holds defaults).
- [x] **Q2** — Tool-call protocol: **A + JSON OpenAI-style**. Dispatcher accepts both shapes, normalizes to one internal `ToolCall`.
- [x] **Q3** — Index hierarchy: **A** (layered: top-level `myspace/INDEX.md` always-injected; sub-indexes load on demand).
- [x] **Q4** — Tool registry: **A** (auto-generated from `pub static TOOLS` Rust registry, mirror-overwrite).
- [x] **Q5** — Sandboxing: **both A and B** (FS-level path validator + role frontmatter `equipped_tools` whitelist).
- [x] **Q6** — Vision/thinking: **ship thinking now**; **defer vision implementation** but **spawn a parallel research agent** to investigate Rust crates for mmproj-aware inference consistent with our infra rules. Output → `Guidelines/source-of-truth/VISION_RESEARCH_<date>.md`.
- [x] **Q7** — Web search: **defer to Phase 0.7-C**. Tool surface registered as placeholder so the index has a slot.

Decisions captured for future sessions in memory file `project_phase_0.7-B_decisions.md`.

## 8. Execution begins now

**Phase A** — `retrieval.rs` test fix (small "validate the workflow" commit). After Phase A lands and is verified, re-confirm with user before Phase B.

---
plugin: llama-cpp-2
kind: rust-crate
version: "0.1"
scope: desktop-only
status: active
since: 2026-04-25
phase: 0.7-A
commit: 61447f3
---

# llama-cpp-2 (rust-crate, not a Tauri plugin)

Direct Rust bindings to `llama.cpp`. Vibo links it explicitly so the inference loop lives in our own service layer (`src-tauri/src/services/llama.rs`, ~512 LoC), not behind the leap-ai plugin's API. The plugin still owns *downloads + cache*; the *generation* belongs to us.

Cargo dedupes `llama-cpp-2` against the version pinned by `tauri-plugin-leap-ai` 0.1.x → a single copy of `llama.cpp` links into the binary.

## Why this lives outside the plugin

- The plugin's `generate()` is opinionated about prompts and stop conditions. We need flexibility for agent loops (system prompt assembly from `myspace/roles/*-role.md` + `myspace/skills/*-skill.md`, planned).
- Multiple trigger surfaces (UI chat, cron jobs, event-driven agents) all funnel through one Rust service, not separate TS/Rust paths.
- Architecture decision (2026-04-24, "Option B — Single Rust Inference Service") in `Guidelines/source-of-truth/PHASE_0_COMPLETION.md`.

## Surface used by Vibo (`services/llama.rs`)

The wrapper exposes a small, opinionated API. Internals are subject to change; treat the Tauri command layer (`viboinference_*`) as the stable contract.

| Concern | Wrapper function (informal name) | Notes |
|---|---|---|
| Backend init | `init_backend()` | Once per process, idempotent |
| Model load | `load_model(path, params)` | GGUF path from `model_catalog::cache_key` |
| Session create | `new_session(model, params)` | `default_n_ctx` from catalog (32K) |
| Tokenize | `tokenize(text)` | Used for context budgeting |
| Prompt + stream | `stream_chat(session, prompt, on_token)` | Backbone of `viboinference_stream_chat` |
| Stop | `stop_generation(session)` | Cooperative cancel via flag |
| Drop | `drop_session(session)` | Frees KV cache; called on `viboinference_end_chat_session` |

## Stop conditions (current)

- EOS token from model
- User-invoked stop (sets cancel flag, next token check returns)
- Soft cap by `default_n_ctx` (catalog-driven)
- Hard cap by `n_ctx_max` (catalog-driven, model-documented)

## Templating

`minijinja = "2"` is in the same desktop target. Used for prompt assembly (chat templates, system prompt injection from role/skill .md files — agent integration **planned**, not yet shipped). Not exported as a separate plugin.

## What is NOT here

- Mobile inference (LEAP vendor SDKs handle that path; gated on task M).
- Embeddings (planned `EmbeddingService` runs on `ort`, not llama.cpp).
- Vision / multimodal (Phase 0.7-B `inspector`; needs mmproj companion shard handling).

---
*Last updated: 2026-04-26 (consolidation pass)*

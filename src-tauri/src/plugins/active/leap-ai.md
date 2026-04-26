---
plugin: tauri-plugin-leap-ai
crate: tauri-plugin-leap-ai
version: 0.1.1
features: [desktop-embedded-llama]
scope: desktop-only
status: active
since: 2026-04-24
phase: 0.7-A
---

# tauri-plugin-leap-ai

Local LLM inference plugin. Wraps `llama.cpp` in-process on desktop; mobile (LEAP vendor SDKs) is deferred to **task M**.

## Status — ACTIVE since 2026-04-24

Wired in `src-tauri/src/lib.rs` with feature `desktop-embedded-llama`. Deduplicates `llama-cpp-2` against the version pinned by the plugin so a single copy of llama.cpp links into the binary. Plugin is responsible for downloads + cache; the actual generation loop lives in `src-tauri/src/services/llama.rs` (commit `61447f3`, T10/T12).

**Mobile re-enable** is gated on task M (vendor SDK linking + device-recognition-on-first-launch). Cargo.toml mobile target is currently commented.

## Storage layout

```
<app_data_dir>/leap-ai/
├── downloaded-models.json    # plugin's index of cached entries
└── <model files>             # GGUF blobs, plugin-managed names
```

Dev: `~/Library/Application Support/com.viboai.app.dev/leap-ai/`
Prod: `~/Library/Application Support/com.viboai.app/leap-ai/`

Source: `tauri-plugin-leap-ai-0.1.1/src/desktop.rs::storage_root_for_app` → `app.path().app_data_dir() + "leap-ai"`.

**Caches do NOT cross bundle IDs.** Vibo dev and prod do not share. A clean install always re-downloads.

## Vibo command surface (10 native commands)

All 10 are registered in `src-tauri/src/lib.rs` and prefixed `viboinference_*`. Frontend never sees the plugin's raw `leap-ai://event` shape — events are translated Rust-side to typed `vibo://*` channels.

| Command | Purpose |
|---|---|
| `viboinference_list_models` | Curated catalog from `services/model_catalog.rs` |
| `viboinference_list_downloaded` | Models present in cache |
| `viboinference_download_model` | Fetch GGUF for a slug; emits `vibo://model-download-progress` |
| `viboinference_delete_model` | Remove from cache |
| `viboinference_get_active_model` | Current alias / id |
| `viboinference_set_active_model` | Switch alias / id; emits `vibo://model-state` |
| `viboinference_start_chat_session` | Open llama.cpp session for a model |
| `viboinference_stream_chat` | Stream tokens; emits `vibo://chat-delta` |
| `viboinference_stop_generation` | Cancel an in-flight stream |
| `viboinference_end_chat_session` | Drop session, free context |

## Translated events

| Vibo channel | Source plugin event | Payload (camelCase) |
|---|---|---|
| `vibo://model-download-progress` | `leap-ai://download/*` | `{ modelId, progress, bytesDownloaded, bytesTotal }` |
| `vibo://model-state` | `leap-ai://model/*` | `{ modelId, state: "loading"\|"loaded"\|"failed", error? }` |
| `vibo://chat-delta` | `leap-ai://generate/*` | `{ sessionId, delta }` |
| `vibo://chat-done` | `leap-ai://generate/end` | `{ sessionId, finishReason }` |

## Capabilities (`capabilities/default.json`)

`leap-ai:default` is granted, which expands to the per-method allow-list:

```
leap-ai:allow-runtime-info
leap-ai:allow-download-model
leap-ai:allow-load-model
leap-ai:allow-load-cached-model
leap-ai:allow-list-cached-models
leap-ai:allow-remove-cached-model
leap-ai:allow-unload-model
leap-ai:allow-create-conversation
leap-ai:allow-create-conversation-from-history
leap-ai:allow-generate
leap-ai:allow-stop-generation
leap-ai:allow-export-conversation
```

## Model catalog

`src-tauri/src/services/model_catalog.rs` is the single source of truth. Frontend gets URL-stripped `ModelEntryDto`. As of 2026-04-25:

| id | alias | family | params | size | quantization | recommended |
|---|---|---|---|---|---|---|
| `lfm2.5-350m` | `junior` | LFM 2.5 | 350M | 219 MB | Q4_K_M | ✅ |
| `lfm2.5-1.2b-instruct` | `specialist` | LFM 2.5 | 1.2B | 697 MB | Q4_K_M | ✅ |

**Aliases (`junior`, `specialist`)** are how the agent layer addresses models by role, not by raw id.

## Roadmap (NOT in 0.7-A)

| Phase | Model | Alias | Blocker |
|---|---|---|---|
| 0.7-B | `lfm2.5-vl-450m` (LFM 2.5 VL 450M, Q4_0) | `inspector` | Plugin 0.1.1 `download_model` only handles a single URL; vision needs an `mmproj-*.gguf` companion shard. Workaround: call `download_model` twice and load with `source_path` set, OR wait for plugin multi-shard support. The `ModelEntry.limits.mmproj_url: Option<&str>` field is reserved for this. |
| 0.7-C | `modernbert-base` | `emb` | ONNX-ORT runtime, not llama.cpp — separate `EmbeddingService`. Adds ~30 MB native lib. |
| Out of scope (2026-04-24 decision) | `lfm2.5-1.2b-thinking` | (proposed `thinker`) | Removed from 0.7 roadmap; trivially addable later (text-only, no mmproj). |

## Configuration in `Cargo.toml`

```toml
[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]
tauri-plugin-leap-ai = { version = "0.1.1", features = ["desktop-embedded-llama"] }
llama-cpp-2 = "0.1"
minijinja = "2"

# Mobile: re-enable after task M
# [target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
# tauri-plugin-leap-ai = { version = "0.1.1" }
```

## Next steps (Phase 0.7-A residual)

- **T8** — verify `OnboardingWizard` ModelStep wires real `downloadModel` + `onModelDownloadProgress` + `setActiveModel` (currently uses static catalog list — needs progress wiring trace).
- **T11** — delete `src-tauri/src/models/{manager,manifest,mod}.rs` if confirmed unused (legacy from pre-leap-ai approach).

---
*Last updated: 2026-04-26 (consolidation pass)*

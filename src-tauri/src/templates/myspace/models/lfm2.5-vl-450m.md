---
model_id: lfm2.5-vl-450m
alias: inspector
family: LFM 2.5
params: 450M
quantization: Q4_0
size_mb: 280
enabled: false
modality: [text, vision]
supports_vision: true
supports_thinking: false
template: true
created: 2026-04-28

sampler:
  temperature: 0.70
  top_p: 0.92
  top_k: 50
  repeat_penalty: 1.10
  seed: 1234

session:
  default_n_ctx: 16384
  n_ctx_max: 32768
  max_tokens: 4096
  generation_timeout_secs: 600
---

# LFM 2.5 Vision — `inspector` (DISABLED)

**Vision-language model.** Describes images, answers questions about images, OCRs lightly. Disabled until Phase 0.7-B Phase F2 ships the multimodal pipeline.

## Why it's disabled

The model needs a paired **mmproj** (multimodal projector) shard alongside the main `.gguf`. Today's loader (`src-tauri/src/services/llama.rs`) only loads a single GGUF, and the download plugin (`tauri-plugin-leap-ai 0.1.1`) only fetches one URL per model. Phase F2 lands:

1. A direct mmproj download via `reqwest` (already a dep — no new plugin).
2. A `load_mmproj(path)` call in `services/llama.rs`.
3. An `encode_image(png_bytes) -> ImageTokens` step before generation.
4. A `viboinference_attach_image(session_id, png_bytes)` Tauri command.
5. UI affordance to upload an image into a chat turn.

A research agent is currently producing `Guidelines/source-of-truth/VISION_RESEARCH_2026-04-28.md` to recommend the right Rust crate path consistent with our infra rules (in-process, no new plugin, llama-cpp-2 0.1.x line).

## What it would be for once enabled

- "What's in this screenshot?"
- "Read the text from this photo."
- "Summarize this diagram into a note."

## What it would NOT be for

- Long reasoning chains (use `thinker`).
- High-res photo analysis (Q4_0 quant + 450M params = compact, not deep).

## How to enable (manual override)

**Don't.** Enabling `inspector` before Phase F2 means the chat will start a session, attempt to encode an image with no mmproj loaded, and crash. Wait for the research note + the Phase F2 commit. The catalog ships a `vision_model_is_disabled_until_phase_f2` test that fails CI if `enabled` is flipped to `true` here without the corresponding pipeline.

## GGUF source

`https://huggingface.co/LiquidAI/LFM2.5-VL-450M-GGUF/resolve/main/LFM2.5-VL-450M-Q4_0.gguf`

mmproj URL: TBD pending research (see `VISION_RESEARCH_2026-04-28.md`).

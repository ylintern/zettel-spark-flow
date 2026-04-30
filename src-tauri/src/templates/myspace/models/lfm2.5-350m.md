---
model_id: lfm2.5-350m
alias: junior
family: LFM 2.5
params: 350M
quantization: Q4_K_M
size_mb: 219
enabled: true
modality: [text]
supports_vision: false
supports_thinking: false
template: true
created: 2026-04-28

# Sampler — see src-tauri/src/services/llama_config.rs for defaults.
# Lower temperature than `specialist` for snappier first-responder replies.
sampler:
  temperature: 0.85
  top_p: 0.95
  top_k: 64
  repeat_penalty: 1.15
  seed: 1234

# Session — full LFM 2.5 capacity available, but we open small by default
# for snappy first-token latency on a 350M model.
session:
  default_n_ctx: 32768
  n_ctx_max: 131072
  max_tokens: 8192
  generation_timeout_secs: 300
---

# LFM 2.5 Compact — `junior`

**Fast generalist.** This is the first responder for any chat session. Optimized for low-latency triage; hands off to `specialist` (or `thinker`) for harder reasoning.

## What it's for

- Quick factual answers (≤ 2 sentences default).
- Drafting notes (`save_note` tool).
- Vault search lookups (`search_notes`, `read_note`).
- Routing complex requests to a deeper model.

## What it's NOT for

- Multi-step reasoning or planning (use `specialist`).
- Long-form output > ~100 words (use `specialist` or `thinker`).
- Image input (use `inspector` once Phase F2 ships).

## Edit me

You can change any sampler / session value above. Vibo reads this file at session start; changes take effect on the next chat session (no restart needed). The model id, alias, family, params, quantization, size_mb, and capability flags are **informational** — editing them here does not change which GGUF is downloaded. To switch the underlying weights, edit `src-tauri/src/services/model_catalog.rs` (and rebuild).

## GGUF source

`https://huggingface.co/LiquidAI/LFM2.5-350M-GGUF/resolve/main/LFM2.5-350M-Q4_K_M.gguf`

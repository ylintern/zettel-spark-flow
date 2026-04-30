---
model_id: lfm2.5-1.2b-instruct
alias: specialist
family: LFM 2.5
params: 1.2B
quantization: Q4_K_M
size_mb: 697
enabled: true
modality: [text]
supports_vision: false
supports_thinking: false
template: true
created: 2026-04-28

# Sampler — slightly more deterministic than junior, since specialist
# produces longer structured output where coherence matters more than
# variety.
sampler:
  temperature: 0.75
  top_p: 0.92
  top_k: 50
  repeat_penalty: 1.12
  seed: 1234

session:
  default_n_ctx: 32768
  n_ctx_max: 131072
  max_tokens: 8192
  generation_timeout_secs: 300
---

# LFM 2.5 Instruct — `specialist`

**Balanced reasoning.** The deeper model. Takes hand-offs from `junior` when a request needs careful analysis, planning, or multi-step thought.

## What it's for

- Structured plans (numbered steps, headings, tables).
- Cross-referencing the vault (`search_notes` then `read_note` then synthesize).
- Linking notes (`link_notes`).
- Answering questions where reasoning > speed.

## What it's NOT for

- Trivial one-liners (use `junior`).
- Reasoning that benefits from explicit `<think>` blocks (use `thinker`).
- Image input (use `inspector` once Phase F2 ships).

## Edit me

Same rules as `junior`: sampler/session values are live-editable; structural fields are informational. Per-session reload picks up your changes.

## GGUF source

`https://huggingface.co/LiquidAI/LFM2.5-1.2B-Instruct-GGUF/resolve/main/LFM2.5-1.2B-Instruct-Q4_K_M.gguf`

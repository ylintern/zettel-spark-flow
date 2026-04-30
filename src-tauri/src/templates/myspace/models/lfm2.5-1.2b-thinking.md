---
model_id: lfm2.5-1.2b-thinking
alias: thinker
family: LFM 2.5
params: 1.2B
quantization: Q4_K_M
size_mb: 697
enabled: true
modality: [text]
supports_vision: false
supports_thinking: true
template: true
created: 2026-04-28

# Sampler — lower temperature than instruct because thinking models reason
# inside <think> blocks before the answer; we want that reasoning to stay
# focused, not wander.
sampler:
  temperature: 0.65
  top_p: 0.90
  top_k: 40
  repeat_penalty: 1.10
  seed: 1234

session:
  default_n_ctx: 32768
  n_ctx_max: 131072
  # Higher max_tokens than instruct to leave room for both the <think> block
  # and the visible answer. The FE will strip <think> from the rendered
  # response and route it to a collapsible "Reasoning" lane.
  max_tokens: 12288
  generation_timeout_secs: 600
---

# LFM 2.5 Thinking — `thinker`

**Reasoning-first.** Emits `<think>…</think>` blocks before the visible answer. The FE stream parser splits these into a separate "Reasoning" UI lane; the user reads the polished answer first and can expand the reasoning if curious.

## What it's for

- Math / code / debugging / multi-step problems where the working matters.
- Cross-checking a `specialist` answer ("does the chain of thought hold up?").
- Tasks where you want to **see** how the model arrived at the answer.

## What it's NOT for

- Quick triage (use `junior`).
- Writing tasks where you don't want to read the chain of thought (use `specialist`).
- Image input (use `inspector` once Phase F2 ships).

## How `<think>` blocks are handled

When `supports_thinking: true`, the FE stream parser does:

1. Buffer tokens.
2. On `<think>` token: route subsequent tokens to the "Reasoning" lane.
3. On `</think>` token: route subsequent tokens to the visible answer lane.
4. The visible answer is what gets streamed into the chat bubble; reasoning is collapsed by default.

Set `supports_thinking: false` here if you want the raw stream including `<think>` tags rendered inline (useful for debugging).

## GGUF source

`https://huggingface.co/LiquidAI/LFM2.5-1.2B-Thinking-GGUF/resolve/main/LFM2.5-1.2B-Thinking-Q4_K_M.gguf`

## Phase F1 status

UI parsing of `<think>` blocks is **Phase F1** of the 0.7-B plan. Until that lands, `<think>` tags will render inline as plain text. The model is still useful — it just looks busy in the bubble.

---
name: Open Items — Must Verify Before Building
description: Unresolved questions that block implementation; must check before writing code
type: project
---

Source: whitepaper rev 1.0. These are honest unknowns — do not assume answers.

## Must Verify in Phase 0 (before any Phase 1 build)

**LeapBridge pattern** — BLOCKING for all inference work
Does tauri-plugin-leap-ai expose Rust state via extension trait, or only via Tauri IPC commands?
- Pattern A: `leap_plugin_state.complete(messages)` if plugin exposes state
- Pattern B: tokio oneshot channel bridge if plugin is IPC-only
Action: Read LEAP plugin source / docs before writing a single line of LeapBridge.

**LFM2.5-350M catalog** — BLOCKING for Nano agent
Confirm .bundle is in LEAP model catalog (vs requires separate CLI bundling).

**velesdb iOS compile** — BLOCKING for mobile support
Compile test needed for arm64-apple-ios target. Not yet validated.

**Android LEAP SDK** — Not fully validated. iOS-first until confirmed.

## Must Decide Before Build

**OAuth token storage:** tauri-plugin-stronghold or OS keychain plugin?

**Background queue RAM threshold:** What number triggers skip vs proceed?
Suggestion in whitepaper: <400MB available → skip, retry on next NOTE_SAVED

**350M load policy:** Never load on devices ≤3GB RAM total?

## Hard Deferred to v2
- Chat summary rolling embeddings (long-term memory)
- ChunkCode + AST for GitHub integration
- LFM2.5-Thinking for deep reasoning tasks
- 350M fine-tuning on Vibo-specific extraction tasks
- ModernBERT embedding upgrade

**Why:** v1 ships with 2 models + notes/tasks/chat. Everything else is post-ship.

# tauri-plugin-leap-ai

AI inference plugin with local model support.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-leap-ai |
| Version | 0.1.1 |
| Status | RESEARCH |

## Purpose

Provides local AI inference using embedded LLMs (Llama.cpp). Enables AI-powered features without cloud dependency.

## Current Status

**Status:** RESEARCH

The plugin is commented out in Cargo.toml due to compatibility concerns:

```toml
# Research note (2026-04-11): tauri-plugin-leap-ai@0.1.1 documents
# target-specific setup for cross-device builds. Keep it target-gated:
# - Mobile (iOS/Android): default LEAP SDK backend
# - Desktop (macOS/Windows/Linux): enable `desktop-embedded-llama`
#
# IMPORTANT REVIEW FLAG:
# This workspace is still on Tauri 2.0.0-rc.* while leap-ai@0.1.1
# documentation references Tauri 2.10.x. Validate dependency compatibility
# in a dedicated upgrade branch before enabling leap-ai in production.
```

## Requirements

### Tauri Version Compatibility
- Current: 2.0.0-rc.17
- Required by leap-ai: 2.10.x
- **Action needed:** Validate in upgrade branch

### Target-Specific Features
```toml
# Desktop
tauri-plugin-leap-ai = { version = "0.1.1", features = ["desktop-embedded-llama"] }

# Mobile  
tauri-plugin-leap-ai = { version = "0.1.1" }
```

## Capabilities

From `capabilities/default.json`, permissions are already defined:
```json
"leap-ai:allow-runtime-info",
"leap-ai:allow-download-model",
"leap-ai:allow-load-model",
"leap-ai:allow-load-cached-model",
"leap-ai:allow-list-cached-models",
"leap-ai:allow-remove-cached-model",
"leap-ai:allow-unload-model",
"leap-ai:allow-create-conversation",
"leap-ai:allow-create-conversation-from-history",
"leap-ai:allow-generate",
"leap-ai:allow-stop-generation",
"leap-ai:allow-export-conversation"
```

## Usage Scenarios

- Local LLM inference
- RAG (Retrieval-Augmented Generation)
- Text generation
- Conversation memory

## Next Steps

1. Create Tauri upgrade branch
2. Test compatibility with Tauri 2.10.x
3. Add dependency with correct features
4. Test embedding and inference

---
*Last updated: 2026-04-14*

# Planned Plugins

Plugins not yet linked, with a clear roadmap slot. **Last reviewed 2026-04-26.**

> `tauri-plugin-leap-ai` was previously listed here. It is now ACTIVE (since 2026-04-24) — see [`active/leap-ai.md`](active/leap-ai.md).

## Currently planned

| Plugin / crate | Version | Purpose | Phase | Status |
|---|---|---|---|---|
| `tauri-plugin-velesdb` | 1.12.0 | Vector database for semantic search / RAG | 0.7-C → 3 | Commented in Cargo.toml |
| `swiftide` | 0.32.1 | RAG framework (indexing + retrieval) | 3 | Commented in Cargo.toml |
| `swiftide-agents` | 0.32.1 | Agent layer over Swiftide | 3 | Commented in Cargo.toml |

See [`planned/velesdb.md`](planned/velesdb.md) for vector-DB notes.

## Out-of-scope (intentionally not planned)

| Plugin | Reason |
|---|---|
| `tauri-plugin-biometric` | Custom implementation lives in `src-tauri/src/security/biometric.rs`; OS-specific hardware unlock deferred to Phase 2 (16+ hrs OS integration work). No official Tauri plugin currently meets the iOS Secure Enclave / Android BiometricPrompt bar. |
| `tauri-plugin-http` | The agent's web-search path is intentionally deferred. When implemented (post agent skeleton), preferred route is DDG / Tor / Firecrawl — free, Rust-friendly. Adding `tauri-plugin-http` would only be needed if we go the API-key path. |

## Pre-requisites for the next planned plugin (velesdb)

- Stable inference layer (✅ shipped 2026-04-25, commit `61447f3`)
- Embeddings runtime decision: `ort` (ONNX Runtime) for `modernbert-base` is the current plan — Phase 0.7-C, separate `EmbeddingService` (~30 MB native lib).
- Vector store API contract (TBD when Phase 0.7-C kicks off).

---
*Last updated: 2026-04-26 (consolidation pass)*

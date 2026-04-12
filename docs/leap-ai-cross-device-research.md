# LEAP AI plugin research (Tauri, cross-device)

Date: 2026-04-11
Scope: `src-tauri/Cargo.toml` dependency strategy + command calls for frontend integration.

## What docs.rs (tauri-plugin-leap-ai 0.1.1) shows

### 1) Cross-device dependency layout (recommended by plugin docs)

- iOS/Android: include `tauri-plugin-leap-ai` with default backend (LEAP mobile SDKs).
- Desktop (Linux/macOS/Windows): include `tauri-plugin-leap-ai` with feature:
  - `desktop-embedded-llama`

This is now reflected as commented guidance in `src-tauri/Cargo.toml` so the project can enable it per-target when the version matrix is validated.

### 2) Plugin registration (Rust)

In `src-tauri/src/lib.rs`, plugin init call is expected:

- `.plugin(tauri_plugin_leap_ai::init())`

Current codebase does **not** register it yet. This is intentionally deferred until compatibility validation is complete.

### 3) Command calls (low-level invoke names)

When calling via `@tauri-apps/api/core` `invoke`, command namespace is:

- `plugin:leap-ai|download_model`
- `plugin:leap-ai|load_model`
- `plugin:leap-ai|load_cached_model`
- `plugin:leap-ai|list_cached_models`
- `plugin:leap-ai|remove_cached_model`
- `plugin:leap-ai|unload_model`
- `plugin:leap-ai|create_conversation`
- `plugin:leap-ai|create_conversation_from_history`
- `plugin:leap-ai|generate`
- `plugin:leap-ai|stop_generation`
- `plugin:leap-ai|export_conversation`
- `plugin:leap-ai|runtime_info`

### 4) Capability permissions (Tauri v2)

Plugin docs indicate either:

- include `leap-ai:default`, or
- explicitly add per-command `leap-ai:allow-*` permissions.

Current app capabilities do not include LEAP permissions yet.

## Review/blocker notes (no assumptions)

1. **Version matrix risk**
   - Project currently pins Tauri `2.0.0-rc.*`.
   - `tauri-plugin-leap-ai@0.1.1` docs list dependency on Tauri `2.10.x`.
   - Action: validate cargo resolution in a dedicated branch before enabling.

2. **Mobile-first architecture fit**
   - Keep model operations behind an internal service boundary so frontend calls are identical on mobile and desktop.
   - Prefer plugin guest bindings API (`tauri-plugin-leap-ai-api`) over raw `invoke` strings in app code where possible.

3. **Operational recommendation**
   - Gate desktop-only heavy model workflows in UI and runtime checks.
   - Start with `runtime_info`, `list_cached_models`, `download_model`, `load_model`, and `generate` as MVP command path.


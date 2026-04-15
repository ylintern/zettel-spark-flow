# tauri-plugin-clipboard-manager

Clipboard read/write plugin.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-clipboard-manager |
| Version | 2.0.0-beta.0 |
| Status | ACTIVE |

## Purpose

Provides clipboard read and write operations.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 102)
.plugin(tauri_plugin_clipboard_manager::init())
```

**Initialization:**
- Universal plugin (all platforms)
- Uses default permissions
- Initialized via `.plugin()` chain

## Platform Support

- macOS: Yes
- Windows: Yes
- Linux: Yes
- iOS: Yes
- Android: Yes

## Dependencies

```toml
# Cargo.toml (line 44)
tauri-plugin-clipboard-manager = "2.0.0-beta.0"
```

## Permissions

```json
// capabilities/default.json (line 13)
"clipboard-manager:default"
```

## Usage Notes

- Read text from clipboard
- Write text to clipboard
- Beta version - API may change

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

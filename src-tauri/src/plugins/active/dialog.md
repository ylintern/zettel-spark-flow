# tauri-plugin-dialog

Native system dialogs plugin for file picker, message boxes, and confirmations.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-dialog |
| Version | 2.0.0-rc |
| Status | ACTIVE |

## Purpose

Provides native system dialogs for file open/save, messages, and confirmations.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 104)
.plugin(tauri_plugin_dialog::init())
```

**Initialization:**
- Universal plugin (all platforms)
- Uses default permissions
- Initialized via `.plugin()` chain

## Platform Support

- macOS: Yes
- Windows: Yes
- Linux: Yes
- iOS: Limited (no folder picker)
- Android: Limited (no folder picker)

## Dependencies

```toml
# Cargo.toml (line 42)
tauri-plugin-dialog = "2.0.0-rc"
```

## Permissions

No additional permissions required - uses default dialog permissions.

## Usage Notes

- Message dialogs (info, warning, error)
- File/folder picker
- Save dialog
- Confirm/ask dialogs

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

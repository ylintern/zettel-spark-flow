# tauri-plugin-global-shortcut

Global keyboard shortcuts plugin for desktop platforms.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-global-shortcut |
| Version | 2.0.0-rc |
| Status | ACTIVE |

## Purpose

Registers global keyboard shortcuts that work even when the app is not in focus.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 87)
.plugin(tauri_plugin_global_shortcut::Builder::new().build())
```

**Initialization:**
- Desktop-only (macOS, Windows, Linux)
- Uses Builder pattern
- Initialized in `#[cfg(desktop)]` block

## Platform Support

- macOS: Yes
- Windows: Yes
- Linux: Yes
- iOS: No
- Android: No

## Dependencies

```toml
# Cargo.toml (line 62)
tauri-plugin-global-shortcut = "2.0.0-rc"
```

**Note:** Defined in `[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]`

## Permissions

No additional permissions required.

## Usage Notes

- Register shortcuts with modifiers (Ctrl, Cmd, Alt, etc.)
- Works when app is not focused
- Platform-specific behavior

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

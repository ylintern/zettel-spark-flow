# tauri-plugin-autostart

Auto-start on login plugin for desktop platforms.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-autostart |
| Version | 2 |
| Status | ACTIVE |

## Purpose

Enables the application to start automatically when the user logs in (macOS/Windows/Linux).

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (lines 88-91)
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    Some(vec![]),
))
```

**Initialization:**
- Desktop-only (macOS, Windows, Linux)
- macOS uses LaunchAgent
- Initialized in `#[cfg(desktop)]` block

## Platform Support

- macOS: Yes (LaunchAgent)
- Windows: Yes (Registry)
- Linux: Yes (XDG autostart)
- iOS: No
- Android: No

## Dependencies

```toml
# Cargo.toml (line 61)
tauri-plugin-autostart = "2"
```

**Note:** Defined in `[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]`

## Permissions

No additional permissions required.

## Usage Notes

- macOS: Uses LaunchAgent for login items
- Windows: Registry-based autostart
- Linux: XDG autostart desktop files

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

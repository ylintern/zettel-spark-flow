# tauri-plugin-os

Operating system information plugin.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-os |
| Version | 2 |
| Status | ACTIVE |

## Purpose

Provides OS information: platform, version, arch, locale, etc.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 103)
.plugin(tauri_plugin_os::init())
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
# Cargo.toml (line 43)
tauri-plugin-os = "2"
```

## Permissions

```json
// capabilities/default.json (line 9)
"os:default"
```

## Available Information

- `platform()` - OS platform (linux, macos, ios, windows, etc.)
- `version()` - OS version
- `arch()` - CPU architecture
- `family()` - OS family
- `locale()` - System locale
- `os_type()` - OS type
- `exe_extension()` - Executable extension

## Usage Notes

- Read-only information access
- No sensitive data exposed

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

# tauri-plugin-fs

File system access plugin for Tauri.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-fs |
| Version | 2 |
| Status | ACTIVE |

## Purpose

Provides file system operations: read, write, create, delete files and directories.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 108)
.plugin(tauri_plugin_fs::init())
```

**Initialization:**
- Universal plugin (all platforms)
- Uses default permissions
- Initialized via `.plugin()` chain

## Platform Support

- macOS: Yes
- Windows: Yes
- Linux: Yes
- iOS: Yes (with security-scoped resources)
- Android: Yes

## Dependencies

```toml
# Cargo.toml (line 40)
tauri-plugin-fs = "2"
```

## Permissions

```json
// capabilities/default.json (line 11)
"fs:default"
```

## Usage Notes

- Path scoping for security
- Supports base directories: $APPDATA, $APPCONFIG, etc.
- File handles for read/write operations

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

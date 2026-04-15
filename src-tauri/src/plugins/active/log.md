# tauri-plugin-log

Logging plugin for Tauri applications.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-log |
| Version | 2.0.0-rc |
| Status | ACTIVE |

## Purpose

Provides structured logging capabilities for debugging and diagnostics. Used primarily in debug builds to output application events and errors.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (lines 156-162)
if cfg!(debug_assertions) {
    app.handle().plugin(
        tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
    )?;
}
```

**Initialization:**
- Only enabled in debug mode (`debug_assertions`)
- Log level: Info
- Initialized in `.setup()` callback

## Platform Support

- macOS: Yes
- Windows: Yes  
- Linux: Yes
- iOS: Yes
- Android: Yes

## Dependencies

```toml
# Cargo.toml (line 38)
tauri-plugin-log = "2.0.0-rc"
```

## Permissions

No additional permissions required in `capabilities/default.json` - uses default logging permissions.

## Usage Notes

- Active only in debug builds (development)
- Can be extended to production with custom configuration
- Integrates with Rust's `log` crate

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

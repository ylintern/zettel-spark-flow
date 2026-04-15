# tauri-plugin-haptics

Haptic feedback plugin for mobile platforms.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-haptics |
| Version | 2.0.0-rc |
| Status | ACTIVE |

## Purpose

Provides haptic feedback (vibration) on iOS and Android devices.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 97)
builder = builder.plugin(tauri_plugin_haptics::init());
```

**Initialization:**
- Mobile-only (iOS, Android)
- Initialized in `#[cfg(mobile)]` block

## Platform Support

- macOS: No
- Windows: No
- Linux: No
- iOS: Yes
- Android: Yes

## Dependencies

```toml
# Cargo.toml (line 66)
tauri-plugin-haptics = "2.0.0-rc"
```

**Note:** Defined in `[target.'cfg(any(target_os = "android", target_os = "ios")))'.dependencies]`

## Permissions

No additional permissions required.

## Usage Notes

- Triggers haptic feedback patterns
- iOS: Uses UIImpactFeedbackGenerator
- Android: Uses VibrationEffect API

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition

---
*Last updated: 2026-04-14*

# Active Plugins

All Tauri plugins currently integrated and working in the Zettel Spark Flow project.

## Universal Plugins (All Platforms)

| # | Plugin | Version | Purpose |
|---|--------|--------|---------|
| 1 | tauri-plugin-log | 2.0.0-rc | Logging and diagnostics |
| 2 | tauri-plugin-sql | 2.0.0-rc | SQLite database operations |
| 3 | tauri-plugin-fs | 2 | File system access |
| 4 | tauri-plugin-stronghold | 2.0.0-rc | Secure secret storage |
| 5 | tauri-plugin-dialog | 2.0.0-rc | Native dialogs (file picker, messages) |
| 6 | tauri-plugin-os | 2 | OS information |
| 7 | tauri-plugin-clipboard-manager | 2.0.0-beta.0 | Clipboard read/write |

## Desktop-Only Plugins (macOS, Windows, Linux)

| # | Plugin | Version | Purpose |
|---|--------|--------|---------|
| 8 | tauri-plugin-autostart | 2 | Auto-start on login |
| 9 | tauri-plugin-global-shortcut | 2.0.0-rc | Global keyboard shortcuts |

## Mobile-Only Plugins (iOS, Android)

| # | Plugin | Version | Purpose |
|---|--------|--------|---------|
| 10 | tauri-plugin-haptics | 2.0.0-rc | Haptic feedback |

## Total Active: 10 plugins

## Configuration Notes

All plugins are initialized in `src-tauri/src/lib.rs`:
- Universal plugins: Lines 101-109
- Desktop plugins: Lines 85-92 (via `#[cfg(desktop)]`)
- Mobile plugins: Lines 95-98 (via `#[cfg(mobile)]`)
- Log plugin: Lines 156-162 (debug mode only, via `debug_assertions`)

## Permissions

All plugin permissions are defined in `src-tauri/capabilities/default.json`.

---
*See individual files in `active/` subdirectory for detailed documentation.*

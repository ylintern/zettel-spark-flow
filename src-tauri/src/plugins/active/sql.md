# tauri-plugin-sql

SQL database plugin with SQLite support via sqlx.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-sql |
| Version | 2.0.0-rc |
| Features | sqlite |
| Status | ACTIVE |

## Purpose

Provides SQL database operations using SQLite. Core plugin for application data persistence.

## Configuration (lib.rs)

```rust
// src-tauri/src/lib.rs (line 109)
.plugin(tauri_plugin_sql::Builder::default().build())
```

**Initialization:**
- Universal plugin (all platforms)
- SQLite database
- Initialized via `.plugin()` chain

## Platform Support

- macOS: Yes
- Windows: Yes
- Linux: Yes
- iOS: Yes
- Android: Yes

## Dependencies

```toml
# Cargo.toml (line 39)
tauri-plugin-sql = { version = "2.0.0-rc", features = ["sqlite"] }
```

## Permissions

```json
// capabilities/default.json (line 10)
"sql:default"
```

## Database Path

```rust
// src-tauri/src/lib.rs (line 137)
let db_path = app_data_dir.join("vibo.db");
```

Database is stored in app local data directory as `vibo.db`.

## Usage Notes

- Uses sqlx for async database operations
- SQLite for local storage
- Migrations can be added via builder

## Related Files

- `src-tauri/src/lib.rs` - Plugin initialization
- `src-tauri/Cargo.toml` - Dependency definition  
- `src-tauri/src/db.rs` - Database module

---
*Last updated: 2026-04-14*

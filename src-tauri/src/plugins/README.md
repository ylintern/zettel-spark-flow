# Tauri Plugins Index

This directory contains documentation for all Tauri plugins used in the Zettel Spark Flow project.

## Structure

```
plugins/
├── README.md              # This index file
├── ACTIVE.md              # List of all active plugins
├── PLANNED.md             # List of planned/future plugins
├── active/                # Active plugin documentation
│   ├── log.md            # tauri-plugin-log
│   ├── sql.md            # tauri-plugin-sql  
│   ├── fs.md             # tauri-plugin-fs
│   ├── stronghold.md     # tauri-plugin-stronghold
│   ├── dialog.md         # tauri-plugin-dialog
│   ├── os.md             # tauri-plugin-os
│   ├── clipboard.md      # tauri-plugin-clipboard-manager
│   ├── autostart.md      # tauri-plugin-autostart (desktop)
│   ├── shortcuts.md      # tauri-plugin-global-shortcut (desktop)
│   └── haptics.md         # tauri-plugin-haptics (mobile)
└── planned/               # Planned plugin documentation
    ├── leap_ai.md        # tauri-plugin-leap-ai
    └── velesdb.md        # tauri-plugin-velesdb
```

## Quick Reference

| Plugin | Version | Platform | Status |
|--------|---------|----------|--------|
| tauri-plugin-log | 2.0.0-rc | All (debug) | ACTIVE |
| tauri-plugin-sql | 2.0.0-rc | All | ACTIVE |
| tauri-plugin-fs | 2 | All | ACTIVE |
| tauri-plugin-stronghold | 2.0.0-rc | All | ACTIVE |
| tauri-plugin-dialog | 2.0.0-rc | All | ACTIVE |
| tauri-plugin-os | 2 | All | ACTIVE |
| tauri-plugin-clipboard-manager | 2.0.0-beta.0 | All | ACTIVE |
| tauri-plugin-autostart | 2 | Desktop | ACTIVE |
| tauri-plugin-global-shortcut | 2.0.0-rc | Desktop | ACTIVE |
| tauri-plugin-haptics | 2.0.0-rc | Mobile | ACTIVE |
| tauri-plugin-leap-ai | 0.1.1 | All | PENDING |
| tauri-plugin-velesdb | 1.12.0 | All | PENDING |

## Links

- [Active Plugins](./ACTIVE.md)
- [Planned Plugins](./PLANNED.md)
- [Tauri Official Plugins](https://tauri.app/plugins/)
- [crates.io](https://crates.io/)

---
*Last updated: 2026-04-14*

# tauri-plugin-velesdb

Vector database plugin for semantic search and RAG.

## Plugin Details

| Field | Value |
|-------|-------|
| Name | tauri-plugin-velesdb |
| Version | 1.12.0 |
| Status | PENDING |

## Purpose

Provides vector database operations for semantic search and Retrieval-Augmented Generation (RAG).

## Current Status

**Status:** PENDING

The plugin is commented out in Cargo.toml:

```toml
# tauri-plugin-velesdb = "1.12.0"
# swiftide = "0.32.1"
# swiftide-agents = "0.32.1"
```

## Dependencies

To enable this plugin, the following would be needed:
```toml
tauri-plugin-velesdb = "1.12.0"
swiftide = "0.32.1"
swiftide-agents = "0.32.1"
```

## Requirements

- Requires tauri-plugin-leap-ai to be enabled first
- Local vector storage for embeddings
- Integration with Swiftide for semantic indexing

## Usage Scenarios

- Semantic note search
- RAG-powered AI responses
- Knowledge base indexing
- similarity search

## Next Steps

1. Enable tauri-plugin-leap-ai first
2. Add velesdb and swiftide dependencies
3. Implement semantic search layer

## Notes

- Vector database allows storing embeddings
- Enables semantic (not just keyword) search
- Part of the RAG pipeline for AI features

---
*Last updated: 2026-04-14*

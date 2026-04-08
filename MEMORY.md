# Memory Index — vibo / zettel-spark-flow-main

- [user_role.md](user_role.md) — Dev lead & product owner; senior decision-maker, expects research-first, task-by-task execution
- [feedback_workflow.md](feedback_workflow.md) — Read → Diagnose → Propose → Approve → Execute; never act without verification
- [project_stack.md](project_stack.md) — Final committed tech stack (Tauri 2, LEAP, Swiftide, velesdb, FastEmbed)
- [project_models.md](project_models.md) — Model assignments: 1.2B resident chat, 350M nano background only, all-MiniLM embedding
- [project_primitives.md](project_primitives.md) — Three primitives: Tauri command (human), Swiftide tool (LLM), MCP schema (format only)
- [project_pipelines.md](project_pipelines.md) — Ingestion (write/background) vs Digestion (read/hot path retrieval.rs)
- [project_storage.md](project_storage.md) — Storage: SQLite (sql plugin), velesdb (vectors), vault .md files, Stronghold (secrets)
- [project_killed_decisions.md](project_killed_decisions.md) — Killed: sqlite-vec, Matryoshka truncation, specialist 3rd model, local MCP server
- [project_open_items.md](project_open_items.md) — Must-verify before build: LeapBridge pattern, 350M catalog, velesdb iOS compile
- [project_phase0.md](project_phase0.md) — Phase 0 current tasks: build fix, password persistence, data paths, notes/tasks CRUD

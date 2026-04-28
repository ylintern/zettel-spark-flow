# Dead-Code Report — `src-tauri/src/models/`

**Date:** 2026-04-28
**Scope:** Resolves task **T11** in `PHASE_0_7_BACKLOG.md` ("delete obsolete models manager/manifest after leap-ai migration").
**Verdict:** The original T11 wording is partially wrong. **Do NOT delete `mod.rs`** — only `manager.rs` and `manifest.rs` are dead. Read this file before running any `rm`.

---

## TL;DR

| File | LoC | Status | Action |
|---|---|---|---|
| `src-tauri/src/models/manager.rs` | 197 | 💀 Dead | **Delete** |
| `src-tauri/src/models/manifest.rs` | 167 | 💀 Dead | **Delete** |
| `src-tauri/src/models/mod.rs` | 60 | ✅ **Alive — heavily used** | **Keep**, edit 2 lines |

**Net cleanup: −364 LoC, +0 −2 lines in mod.rs.**

---

## 1. Why `mod.rs` is alive (DO NOT DELETE)

`mod.rs` is the home of the core domain types. Removing it would require rewriting the vault, db, security, and workspace command layers. Verified callers:

| Type | Defined in `mod.rs` | Used by |
|---|---|---|
| `WorkspaceNote` | line 25–38 | `db/mod.rs:12,139,200,514,529`, `vault/mod.rs:9,67,149`, `vault/reconcile.rs:24,295,348,373,399`, `commands/workspace.rs:6,34` |
| `WorkspaceSnapshot` | line 56–60 | `db/mod.rs:12`, `commands/workspace.rs:6` |
| `KanbanColumn` | line 17–21 | `db/mod.rs:12`, `commands/workspace.rs:6` |
| `CallerContext` (User / Agent) | line 10–13 | `security/mod.rs:544,546`, `commands/workspace.rs:6,36,38,57,59,78,80,110,112,130,132,165,167` |

`CallerContext::Agent { agent_id }` is exactly the audit hook the agent skills/roles plumbing will write through — **we want to keep this**.

**Required edit (only):** lines 1–2 of `mod.rs`

```diff
- pub mod manifest;
- pub mod manager;
-
  use serde::{Deserialize, Serialize};
```

That's it. The rest of `mod.rs` stays.

---

## 2. Why `manager.rs` is dead

**File:** `src-tauri/src/models/manager.rs` (197 LoC)
**Defines:** `pub struct ModelManager` — owns model lifecycle (Discovered / Ready / Selected / Active / Failed states).
**External callers:** `0`.

```bash
$ grep -rn 'ModelManager\|models::manager' --include='*.rs' src-tauri/src
src-tauri/src/models/manager.rs:164:    use crate::models::manifest::FileType;   # self-ref inside #[cfg(test)]
```

The only reference is **its own test module** importing from its sibling. Nothing in the running app instantiates `ModelManager`.

**Why it's dead:** This was Vibo's pre-leap-ai model registry, written when we planned to manage GGUF files manually. It was superseded by `tauri-plugin-leap-ai = "0.1.1"` (active since 2026-04-24), whose internal registry now owns: download path, state, active selection, manifest. Re-implementing this struct would conflict with the plugin's own state.

**Confirmed by file age:** untouched since `91d466d` (2026-04-14, pre-leap-ai integration).

---

## 3. Why `manifest.rs` is dead

**File:** `src-tauri/src/models/manifest.rs` (167 LoC)
**Defines:** `pub enum FileType` (Gguf / Safetensors / Other), `pub struct ModelManifest`, `pub enum ModelState`.
**External callers:** `0` outside the dead `manager.rs`.

```bash
$ grep -rn 'ModelManifest\|FileType\|ModelState\|models::manifest' --include='*.rs' src-tauri/src
src-tauri/src/models/manager.rs:164:    use crate::models::manifest::FileType;   # uses the dead manager
```

Replaced by leap-ai's own manifest types, which the 10 `viboinference_*` commands serialize through.

**Confirmed by file age:** untouched since `91d466d` (2026-04-14).

---

## 4. Cross-checks (so you can verify)

Run any of these to reproduce the findings:

```bash
# 1. Confirm mod.rs is heavily used:
grep -rn 'crate::models::' --include='*.rs' src-tauri/src | grep -v '^src-tauri/src/models/'

# 2. Confirm manager.rs has zero external callers:
grep -rn 'ModelManager\|models::manager' --include='*.rs' src-tauri/src

# 3. Confirm manifest.rs has zero external callers:
grep -rn 'ModelManifest\|FileType\|ModelState\|models::manifest' --include='*.rs' src-tauri/src

# 4. Confirm leap-ai is the active replacement:
grep -n 'viboinference_' src-tauri/src/lib.rs

# 5. Build proof — after the deletion these must still compile:
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --tests
```

---

## 5. Recommended deletion procedure

When you're ready (NOT done in this consolidation pass — code-touching is out of scope here):

```bash
# 1. Delete the two dead files
rm src-tauri/src/models/manager.rs
rm src-tauri/src/models/manifest.rs

# 2. Edit mod.rs lines 1–2: remove the `pub mod manifest;` and `pub mod manager;` declarations.
#    Keep everything from line 4 onward.

# 3. Verify the build is still green
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --tests

# 4. Commit
git add src-tauri/src/models/
git commit -m "chore(t11): remove dead ModelManager/ModelManifest superseded by tauri-plugin-leap-ai"
```

---

## 6. Things this report deliberately does NOT recommend deleting

These came up during the audit but are **alive** — listed here so future audits don't trip on them:

| Suspicious path | Why it looked dead | Why it's actually alive |
|---|---|---|
| `src-tauri/src/swiftide-pipeline/*.md` | swiftide is commented out in Cargo.toml | Design docs for future RAG; do NOT mistake for dead code |
| `src-tauri/src/swiftide-agent/*.md` | same | same |
| `src-tauri/src/plugins/active/haptics.md` | only mobile builds use it | `cfg(any(target_os = "android", target_os = "ios"))` — alive on mobile |
| `src-tauri/src/plugins/planned/velesdb.md` | not in active deps | reserved for Phase 1+ RAG; keep |
| `src-tauri/src/models/mod.rs` | sibling files dead | **heavily used** — see §1 |

---

**Sign-off line for whoever runs the deletion:**

```
[ ] I have read §1 and confirmed mod.rs is the source of WorkspaceNote, CallerContext, KanbanColumn, WorkspaceSnapshot.
[ ] I will delete only manager.rs and manifest.rs.
[ ] I will edit only the first 2 lines of mod.rs.
[ ] I will run `cargo check` before committing.
```

# Build Regression Memo — April 8, 2026

**Status**: FIXED (2 changes applied)  
**Root Causes**: 2 independent bugs introduced during the Copilot session

---

## Bug 1: White Screen After Unlock (FIXED)

**File**: `src/lib/store.tsx`  
**Root cause**: `loadAgentNotes` and `saveAgentNotes` called at lines 58 and 172 but not imported.  
**Why build didn't catch it**: `tsconfig.app.json` has `strict: false` + `noImplicitAny: false` — TypeScript silently treats undeclared identifiers as `any`.  
**Runtime effect**: `saveAgentNotes()` throws `ReferenceError` inside a useEffect on every mount → React unmounts tree → white screen.  
**Fix applied**: Changed import on line 3:
```tsx
// Before (broken — verifyPin/isPinSetup were not used anywhere in this file)
import { verifyPin, isPinSetup } from "./crypto";

// After (correct)
import { loadAgentNotes, saveAgentNotes } from "./crypto";
```

---

## Bug 2: Dev Build Failure — swiftide README.docs.md (FIXED)

**File**: `src-tauri/Cargo.toml`  
**Error**: `couldn't read .../target/debug/build/swiftide-17e14b3c323f5bc8/out/README.docs.md`  
**Root cause**: `swiftide`, `swiftide-agents`, `tauri-plugin-velesdb`, and `tauri-plugin-leap-ai` were listed as dependencies but have **zero usage** in any Rust source file. They are Phase 1–3 work added prematurely. Swiftide's build script must generate `README.docs.md` before its lib.rs macro can include it. In debug mode this build artifact was missing.  
**Why release succeeded**: The prior release binary was built when the debug artifact issue didn't yet exist (or was cached).  
**Fix applied**: Commented out the four unused deps with a clear re-activation note:
```toml
# ── IA, RAG e Semântica ── (Phase 1-3 — uncomment when implementing)
# tauri-plugin-velesdb = "1.12.0"
# tauri-plugin-leap-ai = "0.1.1"
# swiftide = "0.32.1"
# swiftide-agents = "0.32.1"
```
**Verification**: `grep -rn "use swiftide\|use tauri_plugin_velesdb\|use tauri_plugin_leap"` returns zero results — no Rust code references these crates.

---

## Commands to Run on Your Machine

Run these in order:

```bash
# 1. Clean only debug build artifacts (safe — no data loss)
cd /Users/cristianovb/Desktop/zettel-spark-flow-main
cargo clean --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target

# 2. Verify cargo compiles
cargo check --manifest-path src-tauri/Cargo.toml

# 3. Run dev build
bun run tauri dev

# OR for a release bundle:
bunx @tauri-apps/cli build
```

If `cargo clean` still feels risky, you can use the narrower command:
```bash
# Alternative: only remove the bad swiftide build artifact directory
# (Find the exact path first, then remove just that directory)
ls src-tauri/target/debug/build/ | grep swiftide
# Then: rm -rf src-tauri/target/debug/build/swiftide-*
# (Only after confirming the directory is the swiftide build cache)
```

---

## What Was NOT Changed

- `security/mod.rs` — key derivation is correct (SHA-256)
- `lib.rs` — plugin registration is correct
- `db/mod.rs` — migration and schema are correct
- `tauri.conf.json` — bundle ID `com.viboai.app` is correct
- All Kanban/folder/note backend logic — untouched

---

## Architecture Note

`swiftide`, `swiftide-agents`, `tauri-plugin-velesdb`, and `tauri-plugin-leap-ai` belong to Phases 1–3. They must not re-enter `Cargo.toml` until the corresponding Rust modules are actually written and wired. Adding them prematurely creates:
1. Build failures from unresolved build scripts
2. Compile-time dependency bloat slowing every build cycle
3. False signals that Phase 1+ work is "started" when it isn't

**Rule**: Cargo dependency added → module must be `use`d → command must be registered in lib.rs.

---

## Sign-Off

- **Author**: Engineering Lead
- **Date**: April 8, 2026
- **Files changed**: `src/lib/store.tsx` (1 line), `src-tauri/Cargo.toml` (4 lines commented out)

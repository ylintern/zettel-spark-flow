# VIBO — Cross-Device Tauri + LEAP Plugin Reference
> Based on: localcowork (Liquid4All/cookbook) · tauri-plugin-leap-ai 0.1.1
> Stack: Tauri 2 · Rust · React/TSX · LEAP SDK · MCP (plugin-based, no sidecars)

---

## 0. Core Constraints (read before touching anything)

| Rule | Why |
|------|-----|
| **No sidecars** | Inference lives inside the Tauri plugin, not a separate process |
| **No `localStorage` in TSX** | Violates Tauri local-first invariants; use Tauri's `store` plugin or in-memory Zustand state |
| **One `invoke_handler`** | `main.rs` must have exactly one `.invoke_handler(tauri::generate_handler![...])` call |
| **OpenAI-compat abstraction** | All LLM calls go through the OpenAI chat-completions shape — swapping models is a config change |
| **Plugin > sidecar** | `tauri-plugin-leap-ai` exposes inference as Tauri commands; no llama.cpp subprocess to manage |

---

## 1. Plugin Setup (`tauri-plugin-leap-ai`)

### 1.1 Add the dependency

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-leap-ai = "=0.1.1"      # pin exact — API is still maturing
tauri         = { version = "^2.10.0", features = ["protocol-asset"] }
serde         = { version = "^1.0", features = ["derive"] }
serde_json    = "^1.0"
tokio         = { version = "1", features = ["full"] }
thiserror     = "^2"
anyhow        = "1"
```

```bash
cargo add tauri-plugin-leap-ai@=0.1.1
```

### 1.2 Register in `main.rs` — the one place, done right

```rust
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent_core;
mod commands;
mod inference;
mod mcp_client;

fn main() {
    tauri::Builder::default()
        // ① Register leap plugin BEFORE invoke_handler
        .plugin(tauri_plugin_leap_ai::init())
        // ② All your Tauri commands in ONE call
        .invoke_handler(tauri::generate_handler![
            commands::chat::send_message,
            commands::models::list_models,
            commands::mcp::call_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> ❌ **Don't do this** — duplicate `invoke_handler` won't compile:
> ```rust
> .invoke_handler(tauri::generate_handler![commands::chat::send_message])
> .invoke_handler(tauri::generate_handler![commands::mcp::call_tool]) // BROKEN
> ```

### 1.3 `tauri.conf.json` capabilities — include plugin permissions

```json
{
  "app": {
    "security": {
      "capabilities": ["default"]
    }
  }
}
```

`capabilities/default.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "leap-ai:default",
    "leap-ai:allow-load-model",
    "leap-ai:allow-generate",
    "leap-ai:allow-download-model",
    "leap-ai:allow-list-cached-models",
    "leap-ai:allow-runtime-info"
  ]
}
```

> ❌ If `capabilities/default.json` is missing the `leap-ai:*` permissions, all plugin commands silently fail with a capability error at runtime.

---

## 2. Plugin API Surface (Key Structs)

From `tauri_plugin_leap_ai` crate:

| Struct | Purpose |
|--------|---------|
| `LoadModelRequest` | Path + options to load a GGUF model into memory |
| `LoadModelResponse` | Model handle / status |
| `GenerateRequest` | Messages array (OpenAI shape) + sampling params |
| `GenerateResponse` | Token stream or full text |
| `DownloadModelRequest` | URL + destination for model download |
| `CreateConversationRequest` | Stateful conversation creation |
| `LeapEvent` | Streaming token event emitted to frontend |
| `StopGenerationRequest` | Cancel in-flight generation |
| `RuntimeInfoResponse` | VRAM, backend (CPU/GPU/NPU), loaded model info |

### 2.1 Load a model (Rust side)

```rust
// src-tauri/src/inference/loader.rs
use tauri_plugin_leap_ai::{LeapAiExt, LoadModelRequest};

pub async fn load_model(app: tauri::AppHandle, model_path: &str) 
    -> Result<(), tauri_plugin_leap_ai::Error> 
{
    app.leap_ai().load_model(LoadModelRequest {
        path: model_path.to_string(),
        ..Default::default()
    }).await?;
    Ok(())
}
```

### 2.2 Generate (streaming, Rust → frontend events)

```rust
// src-tauri/src/inference/generate.rs
use tauri_plugin_leap_ai::{LeapAiExt, GenerateRequest, ChatMessage};

pub async fn stream_generate(
    app: tauri::AppHandle,
    messages: Vec<ChatMessage>,
    window: tauri::Window,
) -> anyhow::Result<()> {
    let mut stream = app.leap_ai().generate(GenerateRequest {
        messages,
        max_tokens: Some(2048),
        temperature: Some(0.7),
        ..Default::default()
    }).await?;

    while let Some(event) = stream.next().await {
        // Emit each token to the React frontend
        window.emit("leap://token", &event)?;
    }
    Ok(())
}
```

### 2.3 Frontend — listen to streaming tokens

```tsx
// src/hooks/useLeapStream.ts
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export function useLeapStream() {
  const [tokens, setTokens] = useState<string[]>([]);

  useEffect(() => {
    const unlisten = listen<{ token: string }>("leap://token", (ev) => {
      setTokens((prev) => [...prev, ev.payload.token]);
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  return tokens.join("");
}
```

---

## 3. Architecture: How the Pieces Connect

```
React/TSX
   │  invoke("send_message", { messages })
   │  listen("leap://token", handler)
   ▼
Tauri IPC Bridge
   │
   ▼
commands/chat.rs          ← thin Tauri command, no business logic
   │  calls
   ▼
agent_core/orchestrator.rs  ← ConversationManager + ToolRouter
   │  calls                     (plan → route → synthesize)
   ├──► inference/generate.rs   ← tauri_plugin_leap_ai::LeapAiExt
   │       (local on-device LFM via LEAP plugin)
   │
   └──► mcp_client/client.rs    ← JSON-RPC over stdio to MCP servers
            (plugin-based tools, no sidecars)
```

**Key invariant:** inference only flows through `tauri_plugin_leap_ai`. There is no `reqwest` call to `localhost:11434` (Ollama) or any other subprocess in production.

---

## 4. Agent Core Pattern (from localcowork)

### 4.1 Dual-session model (Planner + Delegate → VIBO's equivalent)

```
User prompt
    │
    ▼
[Planner session — LFM2-24B]
    │  Decomposes into bracketed steps
    │  No tool definitions sent to planner
    ▼
[Delegate/Router session — LFM2.5-1.2B-Router]
    │  Per step: RAG pre-filters tools (K=15 from full registry)
    │  Selects one tool per step
    ▼
[Synthesizer — LFM2-24B]
    Streams final answer from accumulated results
```

```rust
// src-tauri/src/agent_core/orchestrator.rs  (sketch)
pub struct Orchestrator {
    planner:     InferenceClient,   // wraps tauri_plugin_leap_ai for large model
    router:      InferenceClient,   // wraps tauri_plugin_leap_ai for small router
    tool_filter: ToolPreFilter,     // RAG embedding index (ADR-010)
    mcp:         McpClient,
}

impl Orchestrator {
    pub async fn run(&self, prompt: &str) -> anyhow::Result<String> {
        let plan  = self.planner.plan(prompt).await?;          // step 1
        let tools = self.tool_filter.top_k(&plan, 15).await?;  // step 2
        let results = self.router.execute(&plan, &tools).await?; // step 3
        self.planner.synthesize(&results).await               // step 4
    }
}
```

> ⚠️ **Fallback rule:** If any phase errors, fall through to single-model loop. Never surface orchestrator internals to the user.

### 4.2 MCP client (no sidecars — plugin-managed stdio)

```rust
// src-tauri/src/mcp_client/client.rs
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct McpClient {
    stdin:  tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
}

impl McpClient {
    /// Spawn the MCP server as a child process owned by Tauri — NOT a sidecar
    pub async fn spawn(server_bin: &str, args: &[&str]) -> anyhow::Result<Self> {
        let mut child = Command::new(server_bin)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        Ok(Self {
            stdin:  child.stdin.take().unwrap(),
            stdout: BufReader::new(child.stdout.take().unwrap()),
        })
    }

    pub async fn call(&mut self, method: &str, params: serde_json::Value) 
        -> anyhow::Result<serde_json::Value> 
    {
        let req = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "method": method, "params": params
        });
        self.stdin.write_all(
            format!("{}\n", req).as_bytes()
        ).await?;

        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        Ok(serde_json::from_str(&line)?)
    }
}
```

> ❌ **No Tauri sidecar API** (`tauri::api::process::Command` with `sidecar()`). That's the localcowork pattern we're replacing. Our MCP servers run as regular child processes managed entirely by `agent_core`.

---

## 5. Tauri Commands — Thin Wrappers Only

```rust
// src-tauri/src/commands/chat.rs
use tauri::State;
use crate::agent_core::Orchestrator;

#[tauri::command]
pub async fn send_message(
    orchestrator: State<'_, Orchestrator>,
    messages: Vec<serde_json::Value>,
    window: tauri::Window,
) -> Result<(), String> {
    let prompt = messages.last()
        .and_then(|m| m["content"].as_str())
        .unwrap_or("")
        .to_string();

    orchestrator
        .run_streaming(&prompt, window)
        .await
        .map_err(|e| e.to_string())
}
```

**Rules for commands:**
- No business logic — delegate to `agent_core`
- Return `Result<T, String>` (Tauri serialisation requirement)
- Use `State<'_>` for shared handles, not global statics
- Emit events for streams, don't block the command future

---

## 6. Cross-Device Inference Strategy

The `tauri-plugin-leap-ai` plugin handles local on-device inference. For VIBO's cross-device scenario (e.g. running model on a desktop, controlling from a mobile or secondary device via Tailscale):

```
Device A (model host)          Device B (thin client)
────────────────────           ─────────────────────
tauri-plugin-leap-ai           Tauri app (no model loaded)
        │                              │
        └── exposes OpenAI-compat      └── connects via Tailscale IP
            HTTP endpoint                  to Device A's endpoint
            (localhost:PORT)
```

### 6.1 Config-driven endpoint (no code change to swap local ↔ remote)

```yaml
# _models/config.yaml
inference:
  mode: local                  # local | remote
  local:
    backend: leap              # leap | ollama | llama-cpp
    model: LFM2-1.2B
    quantization: Q5_K_M
  remote:
    base_url: http://100.x.x.x:8080   # Tailscale IP of host device
    api_key_env: VIBO_REMOTE_KEY
```

```rust
// src-tauri/src/inference/client.rs
pub struct InferenceClient {
    base_url: String,           // "http://localhost:PORT" or Tailscale URL
    client:   reqwest::Client,
}

impl InferenceClient {
    pub fn from_config(cfg: &InferenceConfig) -> Self {
        let base_url = match cfg.mode {
            Mode::Local  => format!("http://localhost:{}", cfg.local.port),
            Mode::Remote => cfg.remote.base_url.clone(),
        };
        Self { base_url, client: reqwest::Client::new() }
    }

    // Always uses OpenAI-compat shape regardless of backend
    pub async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        Ok(self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&req)
            .send().await?
            .json().await?)
    }
}
```

> The LEAP plugin exposes its own local OpenAI-compat endpoint when loaded. Point `InferenceClient` at it for local mode, at the Tailscale IP for remote mode. No other code changes needed.

---

## 7. State Management — No `localStorage`

```tsx
// ❌ NEVER — violates Tauri local-first invariants
localStorage.setItem("onboardingComplete", "true");
const val = localStorage.getItem("onboardingComplete");

// ✅ DO — use Tauri store plugin for persistence
import { load } from "@tauri-apps/plugin-store";

const store = await load("vibo.json", { autoSave: true });
await store.set("onboardingComplete", true);
const val = await store.get<boolean>("onboardingComplete");

// ✅ DO — use Zustand for in-session state (no persistence needed)
import { create } from "zustand";

interface SessionState {
  messages: Message[];
  addMessage: (m: Message) => void;
}
export const useSessionStore = create<SessionState>((set) => ({
  messages: [],
  addMessage: (m) => set((s) => ({ messages: [...s.messages, m] })),
}));
```

---

## 8. Smoke Gate — P0 Checklist

Run before every push:

```bash
./scripts/smoke.sh
```

The gate checks in order:

1. **`cargo build`** — `main.rs` must compile (single `invoke_handler`, correct builder chain)
2. **`capabilities/default.json`** — all `leap-ai:*` permissions present
3. **TSX scan** — no `localStorage` usage in `DashboardView.tsx` or `OnboardingWizard.tsx`
4. **MCP server health** — each server responds to JSON-RPC `initialize`
5. **Plugin smoke** — `leap-ai:allow-runtime-info` returns hardware backend info

```bash
# Quick manual checks before the full gate
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
grep -r "localStorage" src/                         # must be empty
grep -c "invoke_handler" src-tauri/src/main.rs      # must print 1
```

---

## 9. What NOT to Do (Anti-Pattern Cheatsheet)

| Anti-pattern | Problem | Fix |
|---|---|---|
| `tauri::api::process::Command::sidecar()` | Sidecars are banned for inference | Use `tauri-plugin-leap-ai` |
| Multiple `.invoke_handler()` in builder chain | Won't compile | One call, all commands listed |
| `localStorage` in any TSX file | Breaks privacy-first model | `@tauri-apps/plugin-store` or Zustand |
| `capabilities/default.json` missing `leap-ai:*` | Silent runtime failure | Add all required permissions |
| Hardcoded `localhost:11434` (Ollama) | Not cross-device, bypasses LEAP | Config-driven `InferenceClient` |
| `console.log()` in MCP servers | No structured audit trail | Use shared `Logger` service |
| Sending tool definitions to Planner | Wastes tokens, confuses model | Send only to Router/Delegate |
| Raw `reqwest` calls inside commands | Business logic in wrong layer | Move to `inference/` module |
| LEAP SDK mock left in production | No real inference | Wire real plugin, smoke gate enforces |

---

## 10. Ordered Fix Sequence for Current Blockers

```
1. Fix src-tauri/src/main.rs
   └─ Remove duplicate invoke_handler
   └─ Correct builder chain: .plugin() → .invoke_handler() → .run()

2. Update capabilities/default.json  
   └─ Add all leap-ai:allow-* permissions

3. Remove localStorage from TSX
   └─ DashboardView.tsx   → @tauri-apps/plugin-store
   └─ OnboardingWizard.tsx → @tauri-apps/plugin-store or Zustand

4. Pass smoke.sh P0 gate
   └─ cargo build clean + all 5 smoke checks green

5. Clear P1/P3 gates
   └─ MCP tool contract tests
   └─ Frontend integration tests

6. Wire real LEAP inference
   └─ Replace mock InferenceClient with tauri_plugin_leap_ai::LeapAiExt calls
   └─ Load LFM2-1.2B via LoadModelRequest on app startup
   └─ Stream tokens via LeapEvent → window.emit("leap://token")
```

---

## 11. Quick Reference

```bash
# Add plugin
cargo add tauri-plugin-leap-ai@=0.1.1

# Dev loop
cargo tauri dev

# Full build
cargo tauri build

# Lint gate (must be clean before PR)
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
npx tsc --noEmit
npx eslint src/
```

```
Plugin docs:  https://docs.rs/tauri-plugin-leap-ai/0.1.1/tauri_plugin_leap_ai/
Crate page:   https://crates.io/crates/tauri-plugin-leap-ai/0.1.1
LEAP SDK:     https://leap.liquid.ai/
Reference app: https://github.com/Liquid4All/cookbook/tree/main/examples/localcowork
```

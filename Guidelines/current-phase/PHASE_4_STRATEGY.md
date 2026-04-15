# Phase 4: Swiftide Agents & Tool Calls
## Strategy Document (Deferred)

**Status:** Design locked, implementation deferred until Phase 3 complete  
**Timeline:** 14-21 days after Phase 3  
**Owner:** Engineering Team  

---

## Overview

Phase 4 adds multi-step agentic reasoning. Agents can read notes, create tasks, reorganize vault, provide insights.

**Goal:** "Ask an agent to refactor my notes. It reads, creates, moves, reorganizes."

---

## Scope

### ✅ In Phase 4
- Manager agent (routes user requests)
- Specialized agents (Content Writer, Analyst, Organizer, Researcher)
- Safe tool implementations (#[tool] macro)
- Multi-step reasoning (internal agent loops)
- Tool result feedback
- Structured JSON outputs (schemars validation)

### ❌ Out of Phase 4
- Web scraping agents
- Docker/shell executors
- Arbitrary code execution
- Cloud-only agents

---

## Agent Types

### Manager Agent
- Routes user requests to specialists
- Decides which agent(s) to invoke
- Aggregates results
- Returns final answer

### Content Writer Agent
- Generates, rewrites, summarizes notes
- Tools: read_note, create_note, update_note, append_to_note

### Analyst Agent
- Queries notes, finds connections
- Extracts insights from multiple notes
- Tools: search_index, read_note, get_note_connections

### Organizer Agent
- Creates tasks, moves notes, suggests structure
- Tools: create_task, move_task, create_folder, list_notes

### Researcher Agent
- Multi-step RAG queries
- Finds evidence, supports claims
- Tools: search_index, read_note, get_note_connections

---

## Safe Tool Implementations

### Tool Macro Pattern
```rust
#[tool]
fn read_note(note_id: String) -> Result<String> {
    // Safe: only read, no write
}

#[tool]
fn create_task(title: String, column: String) -> Result<TaskId> {
    // Safe: restricted to vault files
}

#[tool]
fn append_to_note(note_id: String, text: String) -> Result<()> {
    // Safe: only appends, validates note_id
}

#[tool]
fn search_index(query: String) -> Result<Vec<NoteMetadata>> {
    // Safe: read-only search
}

#[tool]
fn get_note_connections(note_id: String) -> Result<Vec<WikiLink>> {
    // Safe: read-only, returns references
}
```

### Restrictions
- ❌ No shell execution
- ❌ No arbitrary filesystem access
- ❌ No external HTTP calls (except to LLM)
- ✅ Only vault operations (read/write/create/delete notes/tasks)

---

## User Flow

```
User: "Refactor my project notes - separate planning from execution"

Manager Agent processes request:
  1. Analyst reads all project notes
  2. Classifies each note (planning vs execution)
  3. Reports categories
  
Content Writer Agent:
  4. Creates new "Planning" folder
  5. Reads planning notes
  6. Recreates them in new folder with updated title
  
Organizer Agent:
  7. Moves execution notes to "Execution" folder
  8. Creates sub-tasks for action items
  
Researcher Agent:
  9. Finds cross-references between plans and executions
  10. Reports on alignment
  
Result: Vault refactored, user sees agent reasoning + summary
```

---

## Technical Implementation

### Commands Added
```rust
invoke('ask_agent', { question, context })
  → Agent reasons, calls tools, returns answer + reasoning

invoke('get_agent_suggestions', { note_id })
  → Suggestions for connections, refactoring, etc.

invoke('list_available_agents', {})
  → Returns available specialized agents
```

### Swiftide Integration
- **swiftide-agents** — multi-agent graph routing
- **schemars** — JSON schema validation (force structured outputs)

---

## Success Criteria

- ✅ Manager agent routes requests appropriately
- ✅ Specialized agents execute multi-step reasoning
- ✅ Tools execute safely (no arbitrary access)
- ✅ Agent reasoning is transparent (user sees steps)
- ✅ Tools integrate with Phase 0-3 vault operations
- ✅ Works on mobile (respects background limits)

---

## What Phase 4 Does NOT Include

- ❌ Web scraping agents
- ❌ Docker/container execution
- ❌ Arbitrary code execution
- ❌ Shell command agents
- ❌ External API agents (except LLM)
- ❌ Cross-device collaboration

---

## Constraints on Mobile

- Agents run in foreground (no background agent loops)
- Tools operate only on vault files (no external network)
- Multi-step reasoning respects OS timeouts
- Tool results cached to avoid repeated file reads

---

## Full ViBo Vision Complete at Phase 4

Once Phase 4 locked, ViBo MVP is feature-complete:

| Phase | Capability |
|-------|-----------|
| 0 | Create/edit notes + tasks |
| 1 | + Encrypt notes |
| 2 | + Ask AI questions |
| 3 | + Search semantically |
| 4 | + Agents organize & reason |

See [Roadmap Overview](../README.md) for full vision.

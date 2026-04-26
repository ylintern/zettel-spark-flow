# Consolidation Pass — 2026-04-26

This file is the closing record of a docs-only audit done on 2026-04-26. No code was changed in this session — only `.md` files were touched.

The single live source of truth for code state remains `Guidelines/source-of-truth/PHASE_0_COMPLETION.md` (and its "2026-04-24 Update" section).

---

## 1. What changed in this pass

### Edited in place (banners + ticks)

| File | Change |
|---|---|
| `TO_DO.md` | Last-updated → 2026-04-26. Stronghold activation rows + Onboarding-state-migration rows ticked done. Status matrix rows updated. |
| `Guidelines/current-phase/PHASE_0_7_BACKLOG.md` | Added `Last reviewed: 2026-04-26`. Stronghold activation block ticked done with per-line dates. |
| `Guidelines/current-phase/PHASE_0_REMAINING.md` | (untouched in this pass — already shows P0-1..P0-4 fixed) |
| `Guidelines/current-phase/PHASE_0_FINAL_AUDIT.md` | (untouched in this pass — kept; Phase 1 review will banner) |
| `Guidelines/current-phase/PHASE_1_ACTIVATION_CHECKLIST.md` | Top banner: Parts A–E shipped 2026-04-24. Sign-off table all green except F (docs partial), G (deferred), H (deferred). |
| `Guidelines/current-phase/DORMANT_FEATURES_INVENTORY.md` | Top banner. Encryption / vault lifecycle / passphrase rows now ✅ ACTIVE. New row for `0.7-A Local LFM inference`. Plugin status table rewritten to reflect Cargo.toml current state. |
| `Guidelines/current-phase/PHASE_0_CONFLICTS.md` | Top banner: all 7 conflicts resolved with one-line per-conflict resolution summary. |
| `Guidelines/current-phase/PHASE_0_MANDATE_AUDIT.md` | Top banner: audit completed and acted on. |
| `Guidelines/agents/AGENT_A_RUST_AUDIT.md` | Top banner: historical, points to PHASE_0_COMPLETION.md. |
| `Guidelines/agents/AGENT_B_FRONTEND_AUDIT.md` | Top banner: historical. |
| `Guidelines/agents/AGENT_C_YAML_TEST.md` | Top banner: historical. |
| `Guidelines/agents/READY_FOR_HANDOFF.md` | Top banner: historical handoff note. |
| `src-tauri/src/plugins/active/stronghold.md` | Frontmatter added. Activation note 2026-04-24 inserted. Status: `ACTIVE (since 2026-04-24)`. |

### Files moved to `Guidelines/archive/`

Done via `git mv` (history preserved):

| From | To |
|---|---|
| `Guidelines/current-phase/PHASE_0_RESTART_PLAN.md` | `Guidelines/archive/PHASE_0_RESTART_PLAN.md` |
| `Guidelines/current-phase/PHASE_0_TASKS.md` | `Guidelines/archive/PHASE_0_TASKS.md` |
| `Guidelines/current-phase/PHASE_0_EXECUTION_BOARD.md` | `Guidelines/archive/PHASE_0_EXECUTION_BOARD.md` |
| `Guidelines/current-phase/PHASE_0_STRATEGY.md` | `Guidelines/archive/PHASE_0_STRATEGY.md` |
| `Guidelines/current-phase/PHASE_0_MANDATE_AUDIT.md` | `Guidelines/archive/PHASE_0_MANDATE_AUDIT.md` |
| `Guidelines/PHASE 0 MANDATE AUDIT.md` (root dupe) | `Guidelines/archive/PHASE_0_MANDATE_AUDIT_root_dupe.md` |
| `Guidelines/CRITICAL CONFLICTS: User Memories vs. Phase 0 Mandate.md` | `Guidelines/archive/CRITICAL_CONFLICTS_user_memories_vs_phase_0.md` |
| `HANDOFF_SUMMARY_2026-04-19.md` (repo root) | `Guidelines/archive/HANDOFF_SUMMARY_2026-04-19.md` |
| `src-tauri/src/plugins/planned/leap_ai.md` | `src-tauri/src/plugins/active/leap-ai.md` |

### New files

| File | Purpose |
|---|---|
| `Guidelines/archive/INDEX.md` | Inventory of archived files with reasons + supersession links. |
| `src-tauri/src/plugins/active/llama-cpp.md` | Documents the `llama-cpp-2` direct Rust crate (not a Tauri plugin). |
| `src-tauri/src/plugins/active/leap-ai.md` (rewrite of moved file) | Reflects ACTIVE state, lists the 10 `viboinference_*` commands + 4 translated `vibo://*` events, model catalog, roadmap. |
| `src-tauri/src/plugins/ACTIVE.md` (rewrite) | Single 12-row index of every plugin / native crate. |
| `src-tauri/src/plugins/PLANNED.md` (rewrite) | Single index of planned items. `tauri-plugin-leap-ai` removed (now active). |

---

## 2. Phase 0 / 0.7-A residual

Verified against code. These are the **only** items still open from the Phase 0 / 0.7-A scope:

| Item | Status | Where |
|---|---|---|
| **T8** — `OnboardingWizard` ModelStep real `downloadModel` + `onModelDownloadProgress` + `setActiveModel` | Needs verification (LocalModelsSection has it; OnboardingWizard might piggyback or might still be static) | `src/components/OnboardingWizard.tsx` |
| **T11** — Delete legacy model files | Files still on disk; confirm unused, then delete | `src-tauri/src/models/{manager,manifest,mod}.rs` |
| **Reset-UI Step 2** — Reset Onboarding card | Backend command exists; UI card missing | `src/components/settings/` |
| **Reset-UI Step 3** — Delete All Notes card | Neither command nor UI | `src/components/settings/` + `src-tauri/src/commands/workspace.rs` |
| **Reset-UI Step 4** — Delete Vault keep-files checkbox | Refactor needed | `src/components/settings/SafeVaultResetSection.tsx` |
| **Reset-UI Step 5** — Zone layout (Account & Security / Danger Zone) | Pure UI grouping | `src/components/SettingsView.tsx` |
| **Reset-UI Step 6** — "Coming in Phase 0.8" placeholder | Stub component | `src/components/settings/` |

Everything else from `TO_DO.md` Phase 0.7 list is **done**. Phase 0 itself is done.

---

## 3. Two new model proposals — decision

User asked to add:

1. `LiquidAI/LFM2.5-VL-450M-GGUF:Q4_0`
2. `LiquidAI/LFM2.5-1.2B-Thinking-GGUF:Q4_K_M`

| Model | Existing plan | Add now? |
|---|---|---|
| `LFM2.5-VL-450M:Q4_0` | Already on Phase 0.7-B as `inspector` alias. Not in `MODELS` const yet. | **No, not yet.** Real blocker: `tauri-plugin-leap-ai` 0.1.1 `download_model` accepts a single URL; vision models need an `mmproj-*.gguf` companion shard. The `ModelEntry.limits.mmproj_url: Option<&str>` field is already reserved. Workaround (call `download_model` twice + load with `source_path`) is a 0.7-B task, not a config tweak. |
| `LFM2.5-1.2B-Thinking:Q4_K_M` | Was on a 2026-04-24 draft, then explicitly removed as out of 0.7 scope. | **Trivially addable.** Text-only, no mmproj. Same family/size as `specialist` (~697 MB) but tuned for chain-of-thought. Natural alias: `thinker`. Two-line addition to `src-tauri/src/services/model_catalog.rs::MODELS`. Decision is scope, not technical. |

**Recommendation:** Add `thinker` (LFM2.5-1.2B-Thinking) as a third entry in `MODELS` when convenient — counts as a tiny scope creep. Hold `inspector` (VL) for the 0.7-B mmproj work.

---

## 4. Decisions locked during this session (grill-me)

For the agent / skills / roles design ahead. These are settled — write code against them.

| Topic | Decision |
|---|---|
| **Three-layer separation** | `alias` = model nickname (e.g. `junior`); `role` = identity + warm-up; `skill` = behavior contract. Each alias auto-equips its same-name `roles/<alias>-role.md` and `skills/<alias>-skill.md`. |
| **role.md schema** | Identity, mission, default-equipped skills. Read once at session start (becomes the system prompt). |
| **skill.md schema** | Behavior rules + voice/style. Reread on every turn (per-turn behavior contract). |
| **Index-skill** | Always-equipped on every role. Lists *available skills*, not raw tool schemas (token budget). On-disk: `myspace/skills/index-skill.md`. |
| **Tools-Index-skill** | Equipped on demand. Returns the raw tool registry. On-disk: `myspace/skills/tools-index-skill.md`. |
| **Doc placement** | `myspace/` = agent-readable + Obsidian-visible (roles, skills, plugin docs, INDEX). `Guidelines/` = repo documentation only. |
| **Vision / Sandwich / Task tree** | `Guidelines/source-of-truth/{VISION,SANDWICH,TASK_TREE}.md` (planned, not yet written). VISION = scope + non-goals; SANDWICH = layered architecture; TASK_TREE = live to-do referencing the layers + actual file paths. |
| **Lifecycle per artifact** | (a) `roles/*.md` + `skills/*.md` → write-if-absent at bootstrap; user owns after. (b) `myspace/plugin/*.md` → mirror-overwrite from `src-tauri/src/plugins/active/*.md` on every launch (factual, app-owned). (c) `myspace/skills/INDEX.md` → regenerated every launch from `myspace/skills/*.md` frontmatter. |
| **Agent runtime injection** | Backend builds the system prompt. Frontend sends `start_chat_session({ alias: "..." })`. Rust reads `myspace/roles/<alias>-role.md` + `myspace/skills/<alias>-skill.md` + INDEX summary, concatenates, hands to llama.cpp. New module: `src-tauri/src/services/agent_context.rs`. Frontend never reads vault files for prompt assembly. |

---

## 5. Parked items

| Item | Status |
|---|---|
| **Web search for the agent** | Deferred until after the skill/role/Index/Tools-Index plumbing exists. Provider when we ship: free Rust path — DuckDuckGo / Tor-routed / Firecrawl for scraping. **NOT** Brave (paid). NOT a goal for the next pass. |
| **Copy/paste in chat input** | Not yet decided (chat input only vs. app-wide). Pending grill. |
| **File upload from device** | Not yet decided (agent context attachment vs. note attachment vs. both). Pending grill. |
| **Sub-agent strategy for execution** | Tried haiku in this session; their sandbox blocks Edit/Write — different permission boundary than the orchestrator. For docs-only tasks, doing it inline is faster. Investigate before next batch. |

---

## 6. Next-stage expectations — what unblocks "agents start using tool calls / creating notes / tasks"

Ordered by what each step depends on. Steps marked **(D)** are docs-only and can land in any session; **(C)** require code changes.

1. **(D) Author the four user-facing template `.md` files** in repo (then later inject at bootstrap):
   - `src-tauri/src/templates/myspace/roles/junior-role.md`
   - `src-tauri/src/templates/myspace/roles/specialist-role.md`
   - `src-tauri/src/templates/myspace/skills/junior-skill.md`
   - `src-tauri/src/templates/myspace/skills/specialist-skill.md`
   - `src-tauri/src/templates/myspace/skills/index-skill.md`
   - `src-tauri/src/templates/myspace/skills/tools-index-skill.md`
   Decided schemas: role = identity + objectives + default skills; skill = behavior + voice + tool-trigger rules.
2. **(C) Bootstrap injection** — three small functions added to `vault/mod.rs`:
   - `seed_user_templates(vault_dir)` — write-if-absent walk of an embedded const list (use `include_str!` per file; no new crate needed).
   - `mirror_plugin_docs(vault_dir)` — copy `src-tauri/src/plugins/active/*.md` → `myspace/plugin/*.md` every launch.
   - `regenerate_skills_index(vault_dir)` — walk `myspace/skills/*.md`, write `myspace/skills/INDEX.md`.
   All three called from the existing `ensure_vault_dirs` site in `lib.rs:93`. `reset_vault_dir` inherits free.
3. **(C) Tools registry** — `src-tauri/src/services/tools_registry.rs`. One Rust module enumerating every Tauri command exposed to the agent (with JSON-schema in/out). Auto-rendered on launch into `myspace/tools/REGISTRY.md` (mirror-overwrite). The Tools-Index skill reads from REGISTRY.md, not from raw Rust.
4. **(C) Agent context assembly** — `src-tauri/src/services/agent_context.rs` (new). `build_system_prompt(vault_dir, alias)` reads role+skill+INDEX, concatenates, returns string. Hook into `start_chat_session` (5-line edit).
5. **(C) Frontend wiring** — `ChatAssistant.tsx` gains optional `alias` param (default `"junior"`). Later: a Role picker chip in the chat header.
6. **(C) Tool-call loop** — once the agent can be told "you have tools", we add the actual call/response cycle: model emits a tool call in a structured envelope, Rust dispatches to the registered command, response returns to the next turn. This is the moment "agent uses tool calls" becomes real. Earlier steps just give the agent the *catalog*; this step gives it the *invocation*.
7. **(C) First three concrete tools** wired through registry: `create_note`, `read_note`, `list_folders`. These already exist as Tauri commands — the work is the registry entry + JSON schema + dispatch, not new business logic. Once these land, the agent can create notes and tasks on the user's behalf.
8. **(C → D)** Web search slot — once steps 1–7 land, swap in a `web_search` tool backed by DDG/Tor/Firecrawl per the parked decision. Author `web-search-skill.md`. Equip by default on `specialist` only (token budget on `junior`).

**Critical-path order:** (1) → (2) → (3) → (4) → (5) → (6) → (7). Anything else is parallelizable.

---

## 7. Open grill questions (for the next session)

The grill stopped mid-tree. Open branches, in priority order:

1. **Copy/paste scope** — chat input only vs. app-wide.
2. **File upload from device** — agent context attachment vs. note attachment vs. both. Where files land on disk. Whether `dragDropEnabled: false` in `tauri.conf.json` should flip back on.
3. **Tools registry shape** — auto-generated from a `#[tool]` macro vs. hand-written enum vs. Rust+macro hybrid.
4. **Doc consolidation cadence** — when to archive the next batch (`PHASE_0_REMAINING`, `PHASE_0_FINAL_AUDIT`, `PHASE_0_CONFLICTS`, `PHASE_1_ACTIVATION_CHECKLIST`, `DORMANT_FEATURES_INVENTORY` — currently bannered but not yet moved).
5. **Add `thinker` model now or wait** — see §3.

---

*Generated by the 2026-04-26 consolidation pass. No code touched. All changes are in `.md` files only.*

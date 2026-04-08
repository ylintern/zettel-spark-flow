# VIBO Repo Onboarding Audit & Action Plan

**Date**: April 8, 2026  
**Author**: Engineering Session  
**Scope**: Full repo read-through, folder-by-folder starting with Guidelines

---

## 1. What VIBO Is (Confirmed Understanding)

VIBO is a local-first Zettelkasten notebook + Kanban board, powered by on-device AI, targeting smartphones ≥8GB RAM and cross-device (iOS, Android, desktop). It has two distinct flows — user and agent — both passing through Tauri commands into Rust services. No sidecars. Privacy-first with optional Tor-encrypted cloud model calls. Swiftide pipelines handle data ingestion (regex + semantic + embedding + contextually enriched chunks). WikiLinks connect notes into a knowledge graph.

---

## 2. Repo Health Summary

### What's Solid
- **Guidelines system** is well-structured: source-of-truth, current-phase, audits, archive — all working as intended
- **Architecture rules** are clearly codified and internally consistent across codex.md, ROADMAP, and ARCHITECTURE_RULES.md
- **Storage architecture** is sound: markdown files for note bodies, SQLite for metadata/tasks/settings, Stronghold for secrets
- **Phase governance** is disciplined: sequential phases, audit gates, no skipping
- **Event bus skeleton** exists (vault_status_changed and note_indexing_progress are wired)
- **Device-aware builds** working (macOS .app bundle built, device capability detection in Rust)
- **Frontend UX** is advanced enough to serve as target for backend integration

### What's Blocked or Incomplete
- **Stronghold passphrase derivation is broken** — raw UTF-8 bytes instead of 32-byte derived key (GATE_0_AUDIT.md documents this thoroughly). This blocks ALL secure vault operations in packaged builds
- **Core app QA gate is open** — only folder bootstrap passed; reset routing, unlock regression, note/kanban persistence, and private-note encrypt/decrypt still need manual QA
- **Biometric path is placeholder** — acknowledged and deferred, but no hardware-backed implementation exists
- **agent_thinking_delta** event defined but not producing real payloads
- **.dmg packaging fails** (though .app bundle succeeds)

---

## 3. Architecture Stack (from SVG + code)

```
┌─ Frontend ─────────────────────────────────────────────────┐
│  TSX + React + shadcn/ui · Bun · invoke() + onLeapEvent() │
├─ Tauri 2.0 Shell ─────────────────────────────────────────┤
│  WKWebView (iOS) · WebView (Android) · WebView2 (Desktop) │
├─ Rust Services ───────────────────────────────────────────┤
│  commands/  security/  vault/  db/  events/  models/       │
│  services/context.rs  services/retrieval.rs                │
├─ Swiftide RS ─────────────────────────────────────────────┤
│  Indexing pipeline · Agent loop (Nano) · Tool dispatch     │
│  retrieval.rs (read) · redb NodeCache · FastEmbed 384-dim  │
├─ Inference ───────────────────────────────────────────────┤
│  tauri-plugin-leap-ai (load/unload/generate/stream)        │
│  LFM2.5-1.2B-Instruct (resident) · LFM2.5-350M (lazy)    │
├─ Storage ─────────────────────────────────────────────────┤
│  SQLite: notes·tasks·projects·conversations                │
│  tauri-plugin-velesdb: vector+BM25 hybrid · 70µs           │
├─ Embedding ───────────────────────────────────────────────┤
│  FastEmbed-rs · all-MiniLM-L6-v2 ONNX · 384-dim · 22MB   │
│  CoreML EP on iOS · CPU fallback                           │
├─ Platform ────────────────────────────────────────────────┤
│  iOS: Swift LEAP SDK ✅ · Android: Kotlin LEAP SDK ⚠️ WIP │
│  Desktop: embedded llama.cpp ✅                            │
└───────────────────────────────────────────────────────────┘
```

---

## 4. Phase Status

| Phase | Focus | Status |
|-------|-------|--------|
| **0** | Foundation, storage, CRUD, security | `[~]` Hardening — gate passed for sequencing, sign-off pending |
| **1** | Local inference integration | `[~]` Event bus started, app-core QA gate active |
| 2 | Cloud model integration | Not started |
| 3 | RAG pipeline (Swiftide) | Not started |
| 4 | Agentic layer | Not started |
| 5 | Polish and hardening | Not started |

---

## 5. Critical Questions That Need Answers

### Architecture & Security
1. **Stronghold key derivation**: Which algorithm? SHA-256 (fast, simple) vs Argon2 (GPU-resistant, Tauri-recommended) vs Tauri built-in `with_argon2()`. This is the #1 blocker — nothing secure works until this is fixed.
2. **Biometric timeline**: Is hardware-backed biometric deferred to a specific phase, or is it permanently "passphrase + PIN only" until a Tauri plugin matures?
3. **Private note encryption UX contract**: What should the user see when they toggle a note to private? What happens on decrypt? This is undefined at the UI layer.

### Inference & LEAP SDK
4. **LEAP SDK fork strategy**: The community plugin (`tauri-plugin-leap-ai`) is a dependency. When do we fork it? Do we fork just the Tauri plugin wrapper or also the underlying LEAP SDK? What's the maintenance burden?
5. **LEAP SDK docs**: Need to research current API surface, model loading lifecycle, streaming protocol, and memory management for mobile devices. Is there an official doc site?
6. **Model asset management**: Where do LFM2.5-1.2B and LFM2.5-350M models live on disk? Are they bundled or downloaded on first launch? What's the size impact on mobile?

### Swiftide & RAG
7. **Swiftide pipeline scope**: The architecture SVG shows indexing pipeline + agent loop + tool dispatch + retrieval. How much of this is implemented vs. planned? The `swiftide-pipeline/` and `swiftide-agent/` folders exist — are they populated?
8. **VelesDB**: Listed as vector + BM25 hybrid search at 70µs. Is this proven in the current codebase or aspirational from docs?
9. **Embedding model bundling**: all-MiniLM-L6-v2 at 22MB is reasonable for mobile. Is this already bundled or still planned?

### External Integrations
10. **Google/Gmail/Calendar/Notion/GitHub integrations**: Mentioned in the vision. Which phase? Through MCP servers, Tauri plugins, or direct API calls? How does this interact with the "no sidecars" rule?
11. **Tor plugin for cloud API encryption**: Which crate? Is this a Tauri plugin or a Rust-native approach? What phase?

### Mobile
12. **Android LEAP SDK**: Marked as ⚠️ WIP in the architecture SVG. What's the actual status? Is there a tracking issue?
13. **iOS build pipeline**: Has anyone done a real device build? The `.app` bundle is macOS — what about `.ipa`?

---

## 6. To-Do Items (Immediate)

### Must Do Before Any New Feature Work
- [ ] **Fix Stronghold key derivation** — implement `derive_vault_key()` with chosen algorithm, update lib.rs plugin builder and security/mod.rs (GATE_0_AUDIT.md has the full plan)
- [ ] **Complete app-core QA gate**:
  - [ ] Factory reset → onboarding routing
  - [ ] Normal unlock regression
  - [ ] Simple note create/edit/relaunch persistence
  - [ ] Kanban task create/edit/relaunch persistence
  - [ ] Private-note encrypt/decrypt UX contract definition + QA
- [ ] **Clean up duplicate reset UX entrypoints** in settings (flagged in P1_CRITICALS_QA_RESULT)
- [ ] **Fix semantic copy drift** ("Reset Encryption & PIN" label mismatch)

### Research Tasks
- [ ] Research LEAP SDK docs — API surface, streaming protocol, model lifecycle
- [ ] Research `tauri-plugin-leap-ai` source — assess fork viability, identify maintenance surface
- [ ] Audit `swiftide-pipeline/` and `swiftide-agent/` folders — are they scaffolds or real code?
- [ ] Research VelesDB — is it integrated or just a Cargo.toml dependency?
- [ ] Research Tor crates for Rust — `arti` vs others for encrypted cloud API calls
- [ ] Research Tauri mobile build pipeline — iOS (.ipa) and Android (.apk) packaging status

### Architecture Decisions Needed
- [ ] Algorithm choice for `derive_vault_key()` — present options to CTO
- [ ] LEAP SDK fork timing and scope
- [ ] External integration approach (Gmail/Calendar/Notion/GH) — MCP vs plugins vs direct
- [ ] Private-note UX contract — define the full user journey
- [ ] Model download strategy — bundled vs first-launch download vs user-triggered

---

## 7. Observations & Recommendations

### The Good
The governance model here is unusually disciplined for a project at this stage. The Guidelines system, phase gates, and audit trail mean that context is preserved across sessions and contributors. The architecture rules (no sidecars, ping-pong flow, dual user/agent flows, mobile-first) are clear and consistently enforced.

### Watch Out For
- **Doc drift risk**: There are ~25 markdown files across Guidelines. Some audit docs from the same day (2026-04-07) have slight variations. The "one actively maintained doc per concern" rule in Guidelines/README.md is the right antidote — enforce it.
- **Frontend ahead of backend**: The React UI has views for agents, knowledge graph, chat, model management — all of which have no real backend yet. This is acknowledged and fine as long as nobody mistakes the UI for working features.
- **Stronghold is the single point of failure for security**: If the key derivation fix doesn't work cleanly, the entire security model (passphrase, secrets, private notes) is non-functional. This should be item #1.

### Strategic Notes
- The decision to use markdown files for note bodies + SQLite for metadata is excellent for the RAG pipeline later — Swiftide can index files directly without schema migration.
- The event bus architecture (Rust → UI via Tauri events) is the right foundation for streaming inference. Getting `agent_thinking_delta` working end-to-end should be an early Phase 1 milestone.
- Forking `tauri-plugin-leap-ai` sooner rather than later reduces supply chain risk, especially if the community maintainer goes inactive.

---

## Sign-Off

- **Auditor**: Cowork Session
- **Date**: April 8, 2026
- **Status**: Onboarding audit complete. Awaiting CTO decisions on key derivation algorithm, LEAP fork timing, and QA prioritization.

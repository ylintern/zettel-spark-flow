# Planned Plugins

Plugins that are not yet integrated but are planned for future versions.

## Research Phase

| # | Plugin | Version | Purpose | Status |
|---|--------|--------|---------|--------|
| 1 | tauri-plugin-leap-ai | 0.1.1 | AI inference with local models | RESEARCH |
| 2 | tauri-plugin-velesdb | 1.12.0 | Vector database for RAG | PENDING |

## Not Planned (Using Custom Implementation)

| Plugin | Purpose | Notes |
|--------|---------|-------|
| tauri-plugin-biometric | Biometric authentication | Custom implementation in `security/biometric.rs` |

## Details

### tauri-plugin-leap-ai (0.1.1)

**Status:** RESEARCH

**Purpose:** Local AI inference using embedded LLMs (Llama.cpp)

**Requirements:**
- Review compatibility with Tauri 2.0.0-rc.17
- Target-specific features needed:
  - Desktop: `desktop-embedded-llama` feature
  - Mobile: Default LEAP SDK backend

**Notes from Cargo.toml:**
```
# Research note (2026-04-11): tauri-plugin-leap-ai@0.1.1 documents
# target-specific setup for cross-device builds. Keep it target-gated:
# - Mobile (iOS/Android): default LEAP SDK backend
# - Desktop (macOS/Windows/Linux): enable `desktop-embedded-llama`
#
# IMPORTANT REVIEW FLAG:
# This workspace is still on Tauri 2.0.0-rc.* while leap-ai@0.1.1
# documentation references Tauri 2.10.x. Validate dependency compatibility
# in a dedicated upgrade branch before enabling leap-ai in production.
```

### tauri-plugin-velesdb (1.12.0)

**Status:** PENDING

**Purpose:** Vector database for semantic search and RAG (Retrieval-Augmented Generation)

**Requirements:**
- Swiftide integration for semantic search
- Depends on tauri-plugin-leap-ai being enabled

**Notes:**
```
# tauri-plugin-velesdb = "1.12.0"
# swiftide = "0.32.1"
# swiftide-agents = "0.32.1"
```

### tauri-plugin-biometric

**Status:** NOT NEEDED

**Purpose:** Fingerprint/Face ID authentication

**Notes:** Custom implementation exists in `src-tauri/src/security/biometric.rs` - no official Tauri plugin required.

---
*See individual files in `planned/` subdirectory for detailed documentation.*

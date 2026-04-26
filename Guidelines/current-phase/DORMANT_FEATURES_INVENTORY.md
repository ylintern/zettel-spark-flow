> **STATUS (2026-04-26 review):** ⚠️ Historical — superseded by `Guidelines/source-of-truth/PHASE_0_COMPLETION.md` (2026-04-24 Update section). Stronghold + encryption + lock/unlock are NO LONGER DORMANT (activated 2026-04-24). Marked for archive after Phase 1 review.

# Dormant Features & Phase Roadmap Inventory

**Last Updated:** 2026-04-19 (status table fixed-up 2026-04-26)
**Status:** Comprehensive audit of compiled-but-inactive code
**Goal:** Visibility into what's built and waiting for activation

---

## Summary by Phase

| Phase | Feature | Frontend | Backend | Status | Activation Cost |
|-------|---------|----------|---------|--------|-----------------|
| **1** | **Encryption (vault + AES-256)** | ✅ 100% | ✅ 100% | ✅ ACTIVE 2026-04-24 | — (shipped) |
| **1** | **Vault lifecycle (setup/unlock/lock)** | ✅ 100% | ✅ 100% | ✅ ACTIVE 2026-04-24 | — (shipped + reload-lock) |
| **1** | **Passphrase mgmt (reset, change)** | ✅ 100% | ✅ 100% | ✅ ACTIVE 2026-04-24 | — (shipped) |
| **1.5** | **Secret storage (API keys, tokens)** | ⚠️ Stub | ✅ 100% | Dormant (UI) | 4 hrs (UI) |
| **0.7-A** | **Local LFM inference (llama-cpp-2)** | ✅ 100% | ✅ 100% | ✅ ACTIVE 2026-04-25 | — (commit `61447f3`, T10/T12) |
| **2** | **Biometric unlock (iOS/Android)** | ✅ 100% | ⚠️ Stubbed | Dormant (plugin) | 16 hrs (OS integration) |
| **2** | **Encryption key rotation** | ❌ None | ❌ None | Not designed | 8 hrs (design + impl) |
| **3** | **Cross-device vault sync** | ❌ None | ❌ None | Not designed | 12 hrs (design + impl) |
| **0.7-C** | **Semantic search / embeddings (ModernBERT-ONNX)** | ❌ None | ❌ None | Planned — separate `EmbeddingService` | 30+ hrs |
| **3** | **RAG & context retrieval** | ❌ Stub | ⚠️ Stub | Design-time only | 40 hrs (full design) |
| **10** | **iOS native app** | ❌ None | ❌ None | Not started | 80+ hrs (platform) |
| **10** | **iCloud Drive vault sync** | ❌ None | ❌ None | Not designed | 20 hrs (design + impl) |

---

## Phase 0.5/0.6 (COMPLETE)

### ✅ Shipped

**Vault Reconciliation (FS-first, SQL-follows)**
- `vault/reconcile.rs` (600 lines) — scan disk, classify files, adopt foreign .md, delete stale SQL
- Per-file error tolerance — one corrupt file doesn't break hydration
- Idempotent on every launch
- Live verified: foreign Obsidian files auto-indexed, stale SQL rows cleaned

**Files Modified:**
- `Cargo.toml` — added `walkdir = "2"`
- `vault/frontmatter.rs` — added `PartialFrontmatter` for soft parsing
- `vault/mod.rs` — registered `pub mod reconcile`
- `db/mod.rs` — Layer 1 tolerant hydration (per-note error skipping)

---

## Phase 1 (READY, NEEDS FLAG)

### Encryption Vault (100% Complete)

**What's Built:**
- Stronghold integration (`security/mod.rs` lines 44–63)
  - SecurityState struct (vault session management)
  - `setup(passphrase)` — initialize vault with SHA-256 key derivation
  - `unlock(passphrase)` — open vault, store session in Mutex
  - `lock()` — save & close vault
  - `reset_passphrase(current, new)` — change PIN (re-derives key, no note re-encryption needed)

- AES-256-GCM encryption (`security/mod.rs` lines 237–282)
  - `encrypt_note_content(plaintext)` → `!vibo-encrypted:v1:BASE64(nonce):BASE64(ciphertext)`
  - `decrypt_note_content(ciphertext)` → plaintext (format validation, error handling)
  - 12-byte random nonce per encryption (prevents IV reuse)
  - Authenticated encryption (tampering detected)

- Vault Lifecycle Events (`events/mod.rs` lines 28–58)
  - `vault_status_changed` event: `{ configured, unlocked, reason }`
  - Fired on setup/unlock/lock/reset
  - Frontend listens conditionally (only if `encryption_enabled`)

- Note Encryption Flow
  - Save path: `save_note()` → `render_markdown()` checks `FLAGS.encryption_enabled && note.is_private()`
  - Load path: `load_workspace_snapshot()` → `read_note()` checks `FLAGS.encryption_enabled && is_encrypted`
  - SQL `is_encrypted` column tracks metadata

**Frontend (LockScreen & Settings):**
- `LockScreen.tsx` (236 lines) — PIN/passphrase entry UI, device capability checks
- `SettingsView.tsx` encryption card — reset passphrase, vault reset buttons
- `OnboardingWizard.tsx` — auth method selection (PIN/Passphrase/Biometrics)
- Event listener wiring in `Index.tsx` (conditional on `encryption_enabled`)

**Activation:**
```rust
// File: src-tauri/src/config/features.rs, line 59
// Change:
pub const FLAGS: FeatureFlags = FeatureFlags::phase_0();
// To:
pub const FLAGS: FeatureFlags = FeatureFlags {
    encryption_enabled: true,
    biometric_enabled: false,
    ios_enabled: false,
};
// Then: cargo build
```

**Verification:**
- ✅ Unit tests for encrypt/decrypt roundtrip exist
- ✅ Feature flag consumed in all code paths
- ✅ No compilation barriers
- ✅ E2E tested in isolation

---

### Secret Storage (100% Backend, Stub UI)

**Backend Commands (All Active):**
- `store_secret(key, value)` — stores in Stronghold vault (`security/mod.rs:137–157`)
- `get_secret(key)` — retrieves decrypted value
- `delete_secret(key)` — removes from vault
- Secrets are encrypted by Stronghold (vault-level encryption, not note-level)

**Frontend (Stub):**
- `BiometricsSection.tsx` shows how to UI a secret-like input
- No "Secrets" settings page yet
- No use case in Phase 1 (reserved for inference API keys in Phase 1.5+)

**To Activate UI (4 hours):**
1. Create `SettingsView/SecretsSection.tsx` — list secrets, add/delete UI
2. Wire `store_secret()`, `get_secret()` commands to buttons
3. Mask secret values in display (show only first 4 chars + asterisks)
4. Add confirmation dialog for secret deletion

---

## Phase 2 (DESIGNED, NOT STARTED)

### Biometric Unlock (Stubbed)

**What's Built:**
- Device capability detection (`security/biometric.rs` lines 124–218)
  - `get_device_capabilities()` — returns `{ isMobile, canOfferBiometrics, ... }`
  - **Actual platform detection** (not mocked) via Tauri
  
- Enable/disable infrastructure (`security/biometric.rs` lines 230–289)
  - `enable_biometric_unlock(encryptedPassphrase)` — stores passphrase in biometric-locked storage
  - `disable_biometric_unlock()` — removes biometric unlock
  - Biometric config stored in Stronghold secrets

- Frontend UI (100% complete)
  - `LockScreen.tsx` line 42–46 — biometric button conditionally rendered
  - `BiometricsSection.tsx` (229 lines) — enable/disable toggle + passphrase modal
  - `OnboardingWizard.tsx` — biometric method selectable (mock 1.5s delay in UI)

**What's Stubbed:**
- `verify_biometric_and_unlock()` (line 290) — **always returns Ok(false)**
- No actual fingerprint/face verification (awaits platform plugin)

**To Complete (16 hours):**
1. **Install tauri-plugin-biometric** (or equivalent)
2. **iOS integration (8 hours):**
   - Use `LocalAuthentication` framework
   - Release vault key from Secure Enclave on successful biometric
   - Fallback to passphrase if biometric fails
3. **Android integration (8 hours):**
   - Use `androidx.biometric.BiometricPrompt`
   - Store passphrase in Android Keystore (hardware-backed if available)
   - Biometric → auto-fill passphrase → unlock

**Design Decision Deferred:**
- Should plaintext notes be readable without biometric unlock? (UX: security vs. convenience)

---

### Encryption Key Rotation (Design Phase)

**Problem:** User changes passphrase, but notes are encrypted with old key. Options:

1. **Passphrase change does NOT re-encrypt notes**
   - Pro: Instant operation
   - Con: Old passphrase can still decrypt all notes (security concern)
   
2. **Passphrase change DOES re-encrypt all notes**
   - Pro: Cryptographically clean
   - Con: Latency (1K notes = 5s+ lock-up time)
   - Con: Concurrent edit hazard (note edited while re-encrypting)

3. **Hybrid: Lazy key rotation**
   - New notes use new key; old notes re-encrypted on first read
   - Smooth UX but adds complexity

**Currently Implemented:** Option 1 (fastest)

**Recommended for Phase 2:** Implement option 2 with background queue (re-encrypt 100 notes/sec in background, UI stays responsive)

---

### Cross-Device Vault Sync (Design Phase)

**Problem:** `secure-vault.hold` is device-specific (Stronghold file format). Can't sync to another device without breaking vault.

**Options:**

1. **No vault sync (each device has separate vault)**
   - User enters passphrase on each device
   - Notes sync via Obsidian (plaintext only)
   - Encrypted notes don't sync

2. **Export/import encrypted backup**
   - User exports vault on Device A → encrypted backup
   - User imports on Device B → re-lock vault with same passphrase
   - Manual workflow

3. **Cloud-backed vault key (Phase 3+)**
   - Master passphrase stored in iCloud Keychain / Google account
   - Device A derives note encryption key from master
   - Device B retrieves master key → derives same encryption key
   - Encrypted notes sync transparently
   - **High security risk**: master key in cloud

**Currently Implemented:** None (deferred to Phase 3 at earliest)

---

## Phase 3+ (FUTURE ROADMAP)

### Semantic Search (Blocked on Dependency)

**Status:** Tauri v2.0.0-rc.* compatibility issue

**What's Ready:**
- LEAP AI plugin designed for inference (Cargo.toml lines 48–64)
- `ModelManifest`, `ModelManager` stubs in `models/manager.rs` (incomplete)
- Context retrieval framework skeleton in `services/context.rs` (unused)

**Blocker:**
- `tauri-plugin-leap-ai@0.1.1` documents Tauri 2.10.x
- Current project uses Tauri 2.0.0-rc.*
- Incompatible plugin versions; upgrade required

**To Enable:**
1. Upgrade Tauri to 2.10+ (non-trivial, test all Tauri plugins)
2. Re-integrate LEAP AI plugin
3. Implement semantic search UI (query builder, results view)
4. Wire context window to note retriever

**Effort:** 24+ hours (mostly testing plugin compatibility)

---

### RAG & Context Retrieval (Design Phase)

**Vision:** LLM context window populated from vault (notes, tasks, configs)

**What's Stubbed:**
- `services/context.rs` (173 lines)
  - `ContextRequest` struct (user query)
  - `ContextBundle` struct (retrieved notes + metadata)
  - `RetrievalProvider` trait (pluggable backends)
  - `SqlContextProvider` stub (not implemented)

- `models/manager.rs` (292 lines)
  - Model manifest parsing
  - Model state machine (downloading, loaded, failed)
  - Inference NOT wired

**Missing:**
- How to embed notes (sentence-transformers? OpenAI embeddings?)
- Vector database (Swiftide? Veles DB? PostgreSQL pgvector?)
- Context ranking (BM25? Semantic similarity?)
- UI for "Ask assistant about my notes"

**Design Effort:** 8 hours (architecture + API design)  
**Implementation Effort:** 32 hours (embedding, search, UI)

---

### iOS Native App (Not Started)

**Scope:** React Native or Swift UI for iOS, sharing Rust backend

**Current State:** Not designed

**Effort:** 80+ hours (platform work outside Tauri scope)

**Decision Needed:** Continue with Tauri (desktop-focused) or parallel iOS native app?

---

### iCloud Drive Vault Sync (Not Started)

**Vision:** Vault syncs to iCloud Drive, accessible from any iOS/macOS device

**Current State:** Not designed

**Blocker:** Vault is `secure-vault.hold` (device-specific format)

**Path Forward:**
1. Export vault as encrypted backup (Phase 2 prerequisite)
2. Store in iCloud Drive
3. Import on other device with same passphrase

**Alternative:** Cloud-backed master key (high risk, see Cross-Device Sync above)

---

## Dormant Code Inventory

### Comments Indicating Phases

**Phase 0 Comments (Informational):**
- `src-tauri/src/config/features.rs:9-12` — Phase schedule
- `src-tauri/src/vault/mod.rs:19, 225` — Reserved folder names for Phase 1+
- `src-tauri/src/security/mod.rs:182` — "Safe in Phase 1: no encrypted notes yet"

**Phase 1+ Design Comments:**
- `src-tauri/src/vault/frontmatter.rs:35-36` — `is_encrypted` field behavior by phase
- `src-tauri/src/lib.rs:76-78` — Two-tree layout versioning

**Phase 2/3 References:**
- `src-tauri/src/security/biometric.rs:271` — "TODO: integrate with platform plugin"
- `src-tauri/src/models/manager.rs:50, 122, 232, 289` — Orphan sweep, FS scan TODOs (moved to Phase 0.6 reconciliation)

**Phase 3+ Stubs:**
- `src-tauri/src/services/context.rs:33` — "TODO: implement SQL context provider"
- `src-tauri/src/models/manager.rs:156` — "TODO: download model"

### Dead Code (Safe)

- `src-tauri/src/vault/mod.rs:226-228` — `yaml_string_legacy()` (marked `#[allow(dead_code)]`, kept for rollback)
  - **Why:** If serde_yml has a regression, fallback to raw YAML escaping

- `src/components/DashboardView.tsx` (mock data only, no live functionality)
  - **Why:** Placeholder for dashboard (Phase 1+ feature)

### Cargo.toml Commented Dependencies

- **Line 57:** `# tauri-plugin-velesdb = "1.12.0"` — Vector database (Phase 3+ RAG)
- **Line 58:** `# swiftide = "0.32.1"` — RAG framework (Phase 3+)
- **Line 64:** `# tauri-plugin-leap-ai = { ... }` — Inference (Phase 3+, blocked on Tauri version)

---

## Plugins Status

| Plugin | Cargo.toml | Version | Status | Used In |
|--------|----------|---------|--------|---------|
| `tauri-plugin-stronghold` | active | 2.0.0-rc | ✅ Active (USED 2026-04-24) | Vault encryption + secret storage |
| `tauri-plugin-fs` | active | 2 | ✅ Active | File I/O (vault reads) |
| `tauri-plugin-sql` | active | 2.0.0-rc | ✅ Active | SQLite metadata |
| `tauri-plugin-dialog` | active | 2.0.0-rc | ✅ Active | Passphrase / file UI |
| `tauri-plugin-clipboard-manager` | active | 2.0.0-beta.0 | ✅ Active | Copy/paste |
| `tauri-plugin-os` | active | 2 | ✅ Active | Device capability checks |
| `tauri-plugin-log` | active | 2.0.0-rc | ✅ Active | Logging |
| `tauri-plugin-autostart` | desktop-only | 2 | ✅ Active | Desktop auto-launch |
| `tauri-plugin-global-shortcut` | desktop-only | 2.0.0-rc | ✅ Active | Keyboard shortcuts |
| `tauri-plugin-leap-ai` | desktop-only | 0.1.1 + `desktop-embedded-llama` | ✅ ACTIVE 2026-04-24 | Local LFM inference (10 `viboinference_*` cmds) |
| `llama-cpp-2` (rust-crate) | desktop-only | 0.1 | ✅ Active 2026-04-25 | In-process GGUF inference (commit `61447f3`) |
| `tauri-plugin-haptics` | mobile-only | 2.0.0-rc | ✅ Active | iOS/Android haptics |
| `tauri-plugin-biometric` | — | — | ❌ Not integrated | Biometric unlock (Phase 2) |
| `tauri-plugin-velesdb` | commented | — | Planned | Vector DB (Phase 0.7-C / Phase 3 RAG) |

---

## Feature Flag Deep Dive

### PHASE_0_FEATURE_FLAGS (src/lib/commands.ts:70-81)

```typescript
export interface FeatureFlags {
  encryption_enabled: boolean;      // Phase 1+
  biometric_enabled: boolean;        // Phase 2+
  ios_enabled: boolean;              // Phase 10
}

export const PHASE_0_FEATURE_FLAGS: FeatureFlags = {
  encryption_enabled: false,
  biometric_enabled: false,
  ios_enabled: false,
};
```

### Flag Consumption Points

**Frontend:**
- `Index.tsx:195-198` — Skip LockScreen if `!encryption_enabled`
- `Index.tsx:272-300` — Disable vault event listener if `!encryption_enabled`
- `SettingsView.tsx:26, 173, 283` — Load flags, gate encryption UI
- `NoteEditor.tsx:23-54, 157` — Load flags, gate lock button
- `store.tsx:384` — Guard `toggleNoteEncryption()` function

**Backend:**
- `vault/mod.rs:152` — Gate encryption on save (`render_markdown`)
- `vault/mod.rs:102` — Gate decryption on load (`read_note`)
- `db/mod.rs:115` — Determine effective encryption state
- `db/mod.rs:212, 253` — Store is_encrypted metadata

### How to Transition Between Phases

| Transition | Action | Impact |
|-----------|--------|--------|
| Phase 0 → 1 | Flip `encryption_enabled: false → true` in Rust | Vault lifecycle active, encryption mandatory for is_private notes |
| Phase 1 → 2 | Integrate biometric plugin, flip `biometric_enabled: false → true` | Biometric unlock available (mobile only) |
| Phase 2 → 3 | Upgrade Tauri to 2.10+, integrate LEAP AI, design RAG context | Semantic search + inference enabled |
| Phase 3 → 10 | Build iOS native app alongside Tauri desktop | Dual platform support |

---

## Recommended Phase 1+ Sequence

### Immediate (This Sprint)

1. ✅ **Phase 0.5/0.6** — Vault reconciliation (COMPLETE)
2. **Phase 1 Encryption** — Flag flip + testing (8 hours)
   - Activate encryption
   - E2E test vault setup, encrypt/decrypt, passphrase change
   - Document user-facing security model (passphrase is non-recoverable)

### Near-term (1-2 Sprints)

3. **Phase 1.5 Secrets UI** — Secret storage UI (4 hours)
   - Settings page for API keys, tokens
   - Use case: OpenRouter API key for inference

4. **Phase 2 Research** — Biometric plugin evaluation
   - Check Tauri 2.0.0-rc.* biometric plugin landscape
   - Plan iOS/Android integration
   - Estimate effort

### Mid-term (Roadmap)

5. **Phase 2 Biometric** — Hardware-backed unlock (16 hours)
   - Implement native OS integration
   - Test on real devices
   - Phase 1 notes (plaintext) remain readable without biometric

6. **Phase 2 Key Rotation** — Seamless passphrase change (8 hours)
   - Background re-encryption of notes
   - Queue-based approach (non-blocking)

### Long-term (Vision)

7. **Phase 3 LEAP AI** — Semantic search + inference
   - Upgrade Tauri to 2.10+
   - Integrate LEAP AI plugin
   - Design context retrieval from vault

8. **Phase 3+ Sync** — Cross-device vault (design-heavy)
   - Cloud-backed master key (or export/import flow)
   - Device-to-device note sync

---

## Security Model Summary

### Phase 0 (Current)

- **All plaintext** — no encryption
- **Single-device only** — no sync
- **Obsidian-compatible** — users can edit in Obsidian desktop

### Phase 1 (Post-Activation)

- **Per-note opt-in encryption** — `is_private: true` → AES-256-GCM
- **Vault-level passphrase** — single PIN/passphrase unlocks all encrypted notes
- **Single-device vault** — `secure-vault.hold` not synced
- **Plaintext notes still readable after unlock** — TBD: should plaintext require unlock?
- **No key recovery** — passphrase lost → vault lost (intentional)

### Phase 2 (Biometric)

- **Hardware-backed unlock** — fingerprint/face instead of passphrase
- **Fallback to passphrase** — biometric fails → user enters passphrase
- **Mobile priority** — iOS/Android first, desktop later

### Phase 3+ (Vision)

- **Cross-device vault** — master key in cloud or device-to-device pairing
- **Semantic search** — query encrypted notes via inference (requires full decryption in memory)
- **Conflict resolution** — notes edited in Obsidian (can't decrypt) trigger warnings

---

## Known Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|-----------|
| **Passphrase lost** | CRITICAL | User gets vault reset dialog, no recovery option. Document clearly. |
| **Vault file corrupted** | CRITICAL | No recovery. Recommend plaintext backup workflow. |
| **Biometric hardware fails** | HIGH | Fallback to passphrase entry. Test both paths. |
| **Concurrent encryption during key rotation** | MEDIUM | Queue-based re-encryption (Phase 2). Lock notes during bulk op. |
| **Cross-device sync complexity** | HIGH | Design Phase 3+. Start with device-local vaults. |
| **Inference context includes encrypted notes** | MEDIUM | Require explicit user consent. Decrypt in-memory only. |
| **Mobile biometric interception** | MEDIUM | Hardware-backed key release (Phase 2). Avoid passphrase in memory. |

---

## Decision Points for User

1. **Should plaintext notes require vault unlock?**
   - Option A: Plaintext always readable (Phase 1 current design)
   - Option B: Plaintext hidden until vault unlocked (Phase 2 security improvement)
   - TBD: Affects UX and security model

2. **Passphrase strength requirements?**
   - Option A: No minimum (user convenience)
   - Option B: 12+ chars, entropy check (security)
   - TBD: Recommend Option B with grace period

3. **Biometric on desktop (Phase 2)?**
   - Option A: Biometric only on mobile, passphrase on desktop
   - Option B: macOS Touch ID support (more complex)
   - TBD: Start with Option A, revisit for macOS

---

**End of Inventory**  
**Next Review:** Post-Phase 1 activation

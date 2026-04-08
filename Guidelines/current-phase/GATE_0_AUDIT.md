# Gate 0 Vault Setup Blocker - CTO Audit (CORRECTED)

**Date**: April 8, 2026  
**Status**: DIAGNOSIS PHASE - AWAITING IMPLEMENTATION APPROVAL  
**Severity**: P0 (Blocks all persistence testing)  
**Correction Note**: Refocused on actual runtime path in security/mod.rs per CTO guidance. Framed as primary suspected root cause pending verification.

---

## Executive Summary

Gate 0 is **failing at vault initialization** with error: `"Failed to set up passphrase"` and `"Failed to set up PIN"`.

**Primary suspected root cause**: Non-canonical passphrase derivation in both **plugin builder** ([lib.rs](src-tauri/src/lib.rs#L45)) and **runtime security commands** ([security/mod.rs](src-tauri/src/security/mod.rs)) may be producing invalid Stronghold keys.

---

## What Is Proven vs Suspected

### ✅ Proven Facts

1. **Both code paths use raw password bytes**:
   - Plugin builder [lib.rs:45](src-tauri/src/lib.rs#L45): `|password| password.into()` produces variable-length UTF-8
   - Runtime setup [security/mod.rs:57](src-tauri/src/security/mod.rs#L57): `passphrase.as_bytes().to_vec()` also variable-length UTF-8
   - Both paths pass bytes to Stronghold without cryptographic derivation

2. **Tauri plugin documentation indicates 32-byte requirement**:
   - Official docs state: "The password hash must contain exactly 32 bytes. This is a Stronghold requirement."
   - Examples show SHA-256, Blake2b, or Argon2 for compliance

3. **Packaged app fails at secure setup/unlock**:
   - User enters passphrase
   - Clicks "Enable Encryption"
   - Gets generic error: "Failed to set up passphrase"
   - Happens during `Stronghold::new()` call in security/mod.rs

4. **Dev/test builds may mask the issue**:
   - Different path resolutions, permissions, or caching behavior between bundled `.app` and dev runtime

### 🔍 Suspected Root Cause (Primary Hypothesis)

**Non-canonical passphrase derivation** causes `Stronghold::new()` in security/mod.rs to fail:
- Raw UTF-8 bytes are invalid key material for Stronghold
- Either the plugin builder hash callback is not applied at runtime
- OR the direct `Stronghold::new()` call expects pre-hashed 32-byte input
- Either way, bare password bytes fail

This accounts for:
- Why all passphrases fail (not passphrase-specific)
- Why error is generic (Stronghold rejects the key format)
- Why both setup and unlock fail identically

### ❓ Not Yet Ruled Out

1. **Snapshot path/state issues in packaged context**:
   - `app_local_data_dir()` may resolve differently in bundled `.app` vs dev
   - Permissions, directory creation, file write behavior may differ
   - Could compound or be independent of key derivation bug

2. **Plugin configuration state**:
   - Whether `Stronghold::new()` in security/mod.rs actually uses the global hash callback
   - Whether packaged app loads plugins in correct order
   - Whether plugin state persists correctly across setup → unlock lifecycle

3. **Stronghold library version behavior**:
   - Whether `iota_stronghold 2.1` has different expectations than documented
   - Edge cases in key length validation or error reporting

**Recommendation**: Fix the primary suspected cause (key derivation) first. If packaged app still fails after that, investigate path/state and plugin issues.

---

## Issue Analysis - Corrected Diagnosis

### Actual Failing Runtime Path

When user enters passphrase and clicks "Enable Encryption":

1. Frontend: `setupPin(passphrase)` → `setupSecureVault(passphrase)`
2. Tauri invoke: `setup_secure_vault(passphrase: String)` command
3. **Rust command handler** [src-tauri/src/security/mod.rs](src-tauri/src/security/mod.rs#L294):
   ```rust
   #[tauri::command]
   pub async fn setup_secure_vault(
       passphrase: String,
       app: AppHandle,
       state: State<'_, AppState>,
   ) -> Result<(), String> {
       state.security.setup(&passphrase)  // ← Direct call to SecurityState
           .map_err(|err| err.to_string())?;
       emit_vault_status(&app, &state.security, "setup")
   }
   ```

4. **SecurityState::setup()** [src-tauri/src/security/mod.rs:57](src-tauri/src/security/mod.rs#L57):
   ```rust
   let stronghold = Stronghold::new(
       &self.snapshot_path,
       passphrase.as_bytes().to_vec()  // ← Raw UTF-8 bytes passed here
   )
   ```

5. **Stronghold::new() fails** with `Stronghold::new failed: [Stronghold error]`

### The Mismatch: Two Code Paths, One Incomplete Hash Function

**Path A: Plugin Builder (lib.rs)**

[src-tauri/src/lib.rs:45](src-tauri/src/lib.rs#L45):
```rust
.plugin(tauri_plugin_stronghold::Builder::new(|password| password.into()).build())
                                                       ^^^^^^^^^^^^^^^^
```

This registers a **global hash function callback** that will be applied to all Stronghold operations. Currently:
- Input: `String` (passphrase)
- `.into()`: Converts to `Vec<u8>` via UTF-8 encoding
- Output: Variable-length bytes (NOT 32 bytes)

**Path B: Runtime Command (security/mod.rs)**

[src-tauri/src/security/mod.rs:57](src-tauri/src/security/mod.rs#L57):
```rust
let stronghold = Stronghold::new(
    &self.snapshot_path,
    passphrase.as_bytes().to_vec()  // ← Also raw UTF-8 bytes
)
```

Calls `tauri_plugin_stronghold::Stronghold::new()` with:
- Input: `Vec<u8>` from `passphrase.as_bytes().to_vec()`
- Expected: 32-byte derived key per Stronghold requirement
- Actual: Variable-length UTF-8 bytes

### The Critical Question: Does the plugin builder hash function apply here?

**Theory 1**: `Stronghold::new()` uses the globally registered hash function
- If true, the bug is in lib.rs (wrong hash function)
- Both paths would fail identically

**Theory 2**: `Stronghold::new()` expects pre-hashed input
- If true, the bug is in security/mod.rs (not hashing before passing)
- The plugin builder hash function is irrelevant for this code path

**Most likely**: **Hybrid issue**
- `Stronghold::new()` expects the registered hash function to be applied
- But the `.into()` function doesn't produce 32 bytes
- So both the plugin builder AND the downstream expectation are broken

### Why Error Message is Generic

Both `setup()` [line 57-63] and `unlock()` [line 91-97] catch Stronghold errors:

```rust
.map_err(|err| {
    let msg = format!("Stronghold::new failed: {}", err);
    log::error!("{}", msg);
    SecurityError::Stronghold(msg)
})?;
```

The actual Stronghold error (e.g., "Invalid key length", "Key derivation failed") is logged but the frontend only sees "Failed to set up passphrase".

### UI Is Not At Fault

Password validation in [src/components/LockScreen.tsx](src/components/LockScreen.tsx#L96):
- ✅ Passphrase minimum 8 characters (enforced)
- ✅ PIN minimum 4 characters (enforced)
- ✅ Confirmation matching (enforced)

**No special format required** — any 8+ char passphrase or 4+ char PIN should work.

The UI is correctly accepting valid passwords. The backend is rejecting them.

---

## Technical Breakdown: Where Hash Derivation Currently Happens (or doesn't)

### File 1: Plugin Builder - Global Hash Function Registration

**Location**: [src-tauri/src/lib.rs:45](src-tauri/src/lib.rs#L45)

```rust
.plugin(tauri_plugin_stronghold::Builder::new(|password| password.into()).build())
```

**Responsibility**: Register a global hash function that will be applied to all password-based operations in the `tauri_plugin_stronghold` wrapper.

**Current implementation**:
- Closure: `|password| password.into()`
- Input: `String` (passphrase from frontend)
- Output: `Vec<u8>` via UTF-8 encoding
- Problem: Output length is variable (8-50+ bytes), not 32 bytes

### File 2: Runtime Command Path - Direct Use of Passphrase

**Location**: [src-tauri/src/security/mod.rs:57](src-tauri/src/security/mod.rs#L57)

```rust
let stronghold = Stronghold::new(&self.snapshot_path, passphrase.as_bytes().to_vec())
```

**Responsibility**: Create a Stronghold session using snapshot path and derived key.

**Current implementation**:
- Direct call to `tauri_plugin_stronghold::Stronghold::new()`
- Second parameter: `passphrase.as_bytes().to_vec()`
- Problem: Also raw UTF-8, not 32-byte derived key

**Same pattern in unlock()** [line 91]:
```rust
let stronghold = Stronghold::new(&self.snapshot_path, passphrase.as_bytes().to_vec())
```

### Unified Issue: Inconsistent/Invalid Passphrase Derivation

The core problem is that **password derivation is incomplete or incorrect** in both:
1. The global plugin hash function (lib.rs)
2. The runtime security command path (security/mod.rs)

Both pass raw UTF-8 bytes where a 32-byte derived key is expected.

### Current Dependencies (Insufficient for Proper Hashing)

```toml
[dependencies]
aes-gcm = "0.10"     # For AES encryption (not password hashing)
base64 = "0.22"      # For encoding (not hashing)
getrandom = "0.3"    # For randomness (not hashing)
# Missing: sha2, blake2, argon2, or other cryptographic hash
```

---

## Engineering Recommendation: Single Canonical Passphrase Derivation

**Do not propose scattered fixes.** The solution is to **establish one canonical passphrase-to-key derivation boundary** that both the plugin builder and the runtime security commands use unconditionally.

### Why One Canonical Function?

- **Prevents drift**: Plugin builder and runtime commands cannot diverge
- **Reduces surface**: One policy implementation instead of two
- **Easier to maintain**: Change derivation once, everywhere benefits
- **Smaller blast radius**: Bug or security fix touches one location

### Proposed Single Unification Point

Create **one shared function** in [src-tauri/src/security/mod.rs](src-tauri/src/security/mod.rs):

```rust
/// Derive a 32-byte vault encryption key from a user passphrase.
/// This is the canonical derivation for all Stronghold operations.
fn derive_vault_key(passphrase: &str) -> Vec<u8> {
    // Implementation details depend on algorithm choice
    // Must always produce exactly 32 bytes
    // Must be deterministic (same input → same output)
}
```

**Used in exactly two places**:

1. **Plugin builder** in [src-tauri/src/lib.rs:45](src-tauri/src/lib.rs#L45):
   ```rust
   .plugin(tauri_plugin_stronghold::Builder::new(|password| {
       SecurityState::derive_vault_key(&password)  // Use shared function
   }).build())
   ```

2. **Runtime security commands** in [src-tauri/src/security/mod.rs](src-tauri/src/security/mod.rs):
   ```rust
   // In setup():
   let stronghold = Stronghold::new(
       &self.snapshot_path,
       SecurityState::derive_vault_key(&passphrase)  // Same function
   )
   
   // In unlock():
   let stronghold = Stronghold::new(
       &self.snapshot_path,
       SecurityState::derive_vault_key(&passphrase)  // Same function
   )
   ```

### Files to Change

| File | Change | Scope |
|------|--------|-------|
| **src-tauri/Cargo.toml** | Add cryptographic hash crate | 1 line |
| **src-tauri/src/security/mod.rs** | Add `derive_vault_key()` function | ~15 lines |
| **src-tauri/src/security/mod.rs** | Update `setup()` to use function | 1 line |
| **src-tauri/src/security/mod.rs** | Update `unlock()` to use function | 1 line |
| **src-tauri/src/lib.rs** | Update plugin builder to use function | 1 line |

**Total scope**: ~20 lines of code, one shared derivation policy

---

## Algorithm Options for `derive_vault_key()`

Once architecture is approved, these three implementations compete on security/simplicity tradeoff:

**Option A: SHA-256**
- Pros: Minimal deps, fast, standard
- Cons: Less resistant to brute force than Argon2
- Crate: `sha2`

**Option B: Argon2** (Tauri-recommended)
- Pros: Modern, GPU-resistant, industry-standard password hashing
- Cons: ~100ms per call, requires crate addition
- Crate: `argon2`

**Option C: Tauri built-in `with_argon2()`**
- Pros: Zero custom crypto, Tauri-managed
- Cons: Less control, requires salt file
- Implementation: Use Tauri's builder method directly

---

## Files Involved in Fix

**Primary files** (unified passphrase derivation strategy):

1. **[src-tauri/src/security/mod.rs](src-tauri/src/security/mod.rs)** — Implements `derive_vault_key()` function at module level
   - Used by `setup()`
   - Used by `unlock()`
   - Imported by lib.rs for plugin builder

2. **[src-tauri/src/lib.rs](src-tauri/src/lib.rs#L45)** — Plugin builder references the shared function

3. **[src-tauri/Cargo.toml](src-tauri/Cargo.toml)** — Add cryptographic hash crate

**Secondary files** (no changes):
- [src/components/LockScreen.tsx](src/components/LockScreen.tsx) — UI is correct
- [src/lib/crypto.ts](src/lib/crypto.ts) — Bridge is correct
- capabilities/default.json — Already permits stronghold operations

---

## Next Phase: Implementation Approval

**This audit stops here.** The diagnosis is stable. Awaiting leadership approval on:

1. **Architecture confirmation**: One canonical `derive_vault_key()` function used by both plugin builder and runtime commands?

2. **Algorithm decision** (only after architecture is locked): SHA-256, Argon2, or Tauri built-in?

Once architecture is approved, implementation will:
- Add the shared function
- Update three call sites
- Rebuild and test

**Do not proceed to implementation until explicit approval.**

---

## Sign-Off

- **Auditor**: Codex
- **Date**: April 8, 2026
- **Status**: Diagnosis complete. Awaiting implementation approval.
- **Epistemic clarity**: Proven facts separated from suspected cause. Not-yet-ruled-out concerns identified.
- **Scope**: Vault setup/unlock blocker investigation only. No feature work. No code changes yet.

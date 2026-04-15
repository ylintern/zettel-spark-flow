# Phase 1: Encryption & Security
## Strategy Document (Deferred)

**Status:** Design locked, implementation deferred until Phase 0 complete  
**Timeline:** 5-7 days after Phase 0  
**Owner:** Engineering Team  

---

## Overview

Phase 1 adds vault-level encryption. Users set a passphrase, notes are encrypted before disk write, decrypted on read.

**Goal:** Phase 0 vault data becomes secure. User can protect notes with AES-256 encryption.

---

## Scope

### ✅ In Phase 1
- Vault unlock ceremony (passphrase entry on app startup)
- Biometric unlock (iOS Face ID, Android fingerprint)
- Stronghold vault integration
- Note encryption/decryption (AES-256-GCM)
- Encrypted notes prefixed `!vibo-encrypted:v1:` in file
- Key management (passphrase → encryption key)
- Safe key erasure from memory
- Passphrase change UI

### ❌ Out of Phase 1
- Encryption for individual notes (not whole vault)
- Cloud backup of encrypted vault
- Encryption of metadata (SQLite if added)

---

## User Flow

```
Phase 1 Unlocks:
1. User opens ViBo (first time after Phase 1)
2. Prompt: "Set a vault passphrase"
3. User enters passphrase + confirms
4. Vault locked with Stronghold
5. On subsequent opens: passphrase or biometric prompt
6. Once unlocked: can create notes, some can be marked "private"
7. Private notes encrypted before disk write
8. Encrypted notes show lock icon in UI
```

---

## Technical Implementation

### Commands Added
```rust
invoke('setup_vault_passphrase', { passphrase })
  → Result<()>

invoke('unlock_vault', { passphrase })
  → Result<()>

invoke('unlock_with_biometric', {})
  → Result<()>

invoke('toggle_note_encryption', { note_id })
  → Result<()>

invoke('change_vault_passphrase', { old, new })
  → Result<()>
```

### UI Changes
- Lock/unlock icon on note (or vault indicator)
- Passphrase setup wizard
- Biometric permission prompt
- Lock screen (if vault is locked)

### Backward Compatibility
- Phase 0 notes remain unencrypted by default
- Migration path: on Phase 1 launch, ask user to encrypt existing vault
- OR: automatic migration (encrypt all notes in background)

---

## Dependencies

- **aes-gcm** — Encryption algorithm
- **iota_stronghold** — Vault key storage
- **tauri-plugin-stronghold** — Tauri integration
- **sha2** — Key derivation (passphrase → key)

---

## Success Criteria

- ✅ User can set vault passphrase on first launch
- ✅ Vault unlocks with passphrase or biometric
- ✅ Private notes encrypted before disk write
- ✅ Encrypted notes readable only after unlock
- ✅ Files portable but protected (encrypted .md is unreadable without passphrase)
- ✅ Backward compatible with Phase 0 unencrypted notes

---

## What Phase 1 Does NOT Include

- ❌ Per-note encryption (just vault-level for now)
- ❌ Cloud backup of encrypted vault
- ❌ Key recovery / passphrase reset
- ❌ Hardware security keys (future)

---

## Next: Phase 2

Once Phase 1 locked:
- Phase 2: Inline Chat (local or cloud inference)

See [Roadmap Overview](../README.md) for full vision.

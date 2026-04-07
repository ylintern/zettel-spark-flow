pub mod biometric;

use std::{path::PathBuf, sync::Mutex};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use iota_stronghold::Client;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_stronghold::stronghold::Stronghold;
use thiserror::Error;

use crate::{
    db,
    events::{self, VaultStatusChangedEvent},
    state::AppState,
    vault,
};

const CLIENT_NAME: &[u8] = b"vibo-app";
const NOTE_KEY_SECRET: &str = "note_encryption_key";
const PRIVATE_NOTE_PREFIX: &str = "!vibo-encrypted:v1:";

pub struct SecurityState {
    snapshot_path: PathBuf,
    session: Mutex<Option<Stronghold>>,
}

impl SecurityState {
    pub fn new(snapshot_path: PathBuf) -> Self {
        Self {
            snapshot_path,
            session: Mutex::new(None),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.snapshot_path.exists()
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    pub fn setup(&self, passphrase: &str) -> Result<(), SecurityError> {
        let stronghold = Stronghold::new(&self.snapshot_path, passphrase.as_bytes().to_vec())
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;

        let client = ensure_client(&stronghold)?;
        ensure_note_key(&stronghold, &client)?;
        stronghold
            .save()
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;

        *self.session.lock().unwrap() = Some(stronghold);
        Ok(())
    }

    pub fn unlock(&self, passphrase: &str) -> Result<(), SecurityError> {
        if !self.is_configured() {
            return Err(SecurityError::VaultNotConfigured);
        }

        let stronghold = Stronghold::new(&self.snapshot_path, passphrase.as_bytes().to_vec())
            .map_err(|_| SecurityError::InvalidPassphrase)?;
        let client = ensure_client(&stronghold)?;
        ensure_note_key(&stronghold, &client)?;

        *self.session.lock().unwrap() = Some(stronghold);
        Ok(())
    }

    pub fn lock(&self) -> Result<(), SecurityError> {
        let mut session = self.session.lock().unwrap();
        if let Some(stronghold) = session.take() {
            stronghold
                .save()
                .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
        }
        Ok(())
    }

    pub fn store_secret(&self, key: &str, value: &str) -> Result<(), SecurityError> {
        let session = self.session.lock().unwrap();
        let stronghold = session.as_ref().ok_or(SecurityError::VaultLocked)?;
        let client = ensure_client(stronghold)?;
        client
            .store()
            .insert(key.as_bytes().to_vec(), value.as_bytes().to_vec(), None)
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
        stronghold
            .save()
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
        Ok(())
    }

    pub fn get_secret(&self, key: &str) -> Result<Option<String>, SecurityError> {
        let session = self.session.lock().unwrap();
        let stronghold = session.as_ref().ok_or(SecurityError::VaultLocked)?;
        let client = ensure_client(stronghold)?;
        let value = client
            .store()
            .get(key.as_bytes())
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;

        value
            .map(|bytes| String::from_utf8(bytes).map_err(|_| SecurityError::SecretEncoding))
            .transpose()
    }

    pub fn delete_secret(&self, key: &str) -> Result<(), SecurityError> {
        let session = self.session.lock().unwrap();
        let stronghold = session.as_ref().ok_or(SecurityError::VaultLocked)?;
        let client = ensure_client(stronghold)?;
        client
            .store()
            .delete(key.as_bytes())
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
        stronghold
            .save()
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
        Ok(())
    }

    /// Completely destroy the vault and all its contents
    /// This is irreversible and used when user forgot passphrase
    pub fn reset_vault(&self) -> Result<(), SecurityError> {
        let mut session = self.session.lock().unwrap();

        // Close current session
        if let Some(stronghold) = session.take() {
            stronghold
                .save()
                .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
        }

        // Delete the snapshot file
        if self.snapshot_path.exists() {
            std::fs::remove_file(&self.snapshot_path).map_err(|_| {
                SecurityError::Stronghold("Failed to delete vault file".to_string())
            })?;
        }

        Ok(())
    }

    pub fn encrypt_note_content(&self, content: &str) -> Result<String, SecurityError> {
        let note_key = self.get_note_key()?;
        let cipher = Aes256Gcm::new_from_slice(&note_key).map_err(|_| SecurityError::CipherInit)?;
        let mut nonce = [0_u8; 12];
        getrandom::fill(&mut nonce).map_err(|_| SecurityError::Randomness)?;

        let encrypted = cipher
            .encrypt(Nonce::from_slice(&nonce), content.as_bytes())
            .map_err(|_| SecurityError::EncryptionFailed)?;

        Ok(format!(
            "{PRIVATE_NOTE_PREFIX}{}:{}",
            BASE64.encode(nonce),
            BASE64.encode(encrypted)
        ))
    }

    pub fn decrypt_note_content(&self, content: &str) -> Result<String, SecurityError> {
        if !content.starts_with(PRIVATE_NOTE_PREFIX) {
            return Ok(content.to_string());
        }

        let payload = &content[PRIVATE_NOTE_PREFIX.len()..];
        let mut parts = payload.splitn(2, ':');
        let nonce = parts
            .next()
            .ok_or(SecurityError::EncryptedPayloadMalformed)?;
        let ciphertext = parts
            .next()
            .ok_or(SecurityError::EncryptedPayloadMalformed)?;

        let note_key = self.get_note_key()?;
        let cipher = Aes256Gcm::new_from_slice(&note_key).map_err(|_| SecurityError::CipherInit)?;
        let nonce = BASE64
            .decode(nonce)
            .map_err(|_| SecurityError::EncryptedPayloadMalformed)?;
        let ciphertext = BASE64
            .decode(ciphertext)
            .map_err(|_| SecurityError::EncryptedPayloadMalformed)?;

        let decrypted = cipher
            .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| SecurityError::EncryptionFailed)?;

        String::from_utf8(decrypted).map_err(|_| SecurityError::SecretEncoding)
    }

    fn get_note_key(&self) -> Result<Vec<u8>, SecurityError> {
        let session = self.session.lock().unwrap();
        let stronghold = session.as_ref().ok_or(SecurityError::VaultLocked)?;
        let client = ensure_client(stronghold)?;
        let key = client
            .store()
            .get(NOTE_KEY_SECRET.as_bytes())
            .map_err(|err| SecurityError::Stronghold(err.to_string()))?
            .ok_or(SecurityError::VaultLocked)?;
        Ok(key)
    }
}

fn ensure_client(stronghold: &Stronghold) -> Result<Client, SecurityError> {
    stronghold
        .get_client(CLIENT_NAME)
        .or_else(|_| stronghold.create_client(CLIENT_NAME))
        .map_err(|err| SecurityError::Stronghold(err.to_string()))
}

fn ensure_note_key(stronghold: &Stronghold, client: &Client) -> Result<(), SecurityError> {
    let existing = client
        .store()
        .get(NOTE_KEY_SECRET.as_bytes())
        .map_err(|err| SecurityError::Stronghold(err.to_string()))?;

    if existing.is_some() {
        return Ok(());
    }

    let mut key = [0_u8; 32];
    getrandom::fill(&mut key).map_err(|_| SecurityError::Randomness)?;
    client
        .store()
        .insert(NOTE_KEY_SECRET.as_bytes().to_vec(), key.to_vec(), None)
        .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
    stronghold
        .save()
        .map_err(|err| SecurityError::Stronghold(err.to_string()))?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Vault locked")]
    VaultLocked,
    #[error("Vault not configured")]
    VaultNotConfigured,
    #[error("Invalid passphrase")]
    InvalidPassphrase,
    #[error("Stronghold error: {0}")]
    Stronghold(String),
    #[error("Secret encoding error")]
    SecretEncoding,
    #[error("Encrypted note payload is malformed")]
    EncryptedPayloadMalformed,
    #[error("Failed to initialize cipher")]
    CipherInit,
    #[error("Failed to encrypt or decrypt private note")]
    EncryptionFailed,
    #[error("Secure randomness unavailable")]
    Randomness,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretValue {
    pub value: Option<String>,
}

fn emit_vault_status(
    app: &AppHandle,
    security: &SecurityState,
    reason: &str,
) -> Result<(), String> {
    events::emit_vault_status_changed(
        app,
        &VaultStatusChangedEvent {
            configured: security.is_configured(),
            unlocked: security.is_unlocked(),
            reason: reason.to_string(),
        },
    )
}

#[tauri::command]
pub async fn setup_secure_vault(
    passphrase: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .security
        .setup(&passphrase)
        .map_err(|err| err.to_string())?;
    emit_vault_status(&app, &state.security, "setup")
}

#[tauri::command]
pub async fn unlock_vault(
    passphrase: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .security
        .unlock(&passphrase)
        .map_err(|err| err.to_string())?;
    emit_vault_status(&app, &state.security, "unlock")
}

#[tauri::command]
pub async fn lock_vault(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.security.lock().map_err(|err| err.to_string())?;
    emit_vault_status(&app, &state.security, "lock")
}

#[tauri::command]
pub async fn is_vault_unlocked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.security.is_unlocked())
}

#[tauri::command]
pub async fn is_vault_configured(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.security.is_configured())
}

#[tauri::command]
pub async fn store_secret(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .security
        .store_secret(&key, &value)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_secret(key: String, state: State<'_, AppState>) -> Result<SecretValue, String> {
    state
        .security
        .get_secret(&key)
        .map(|value| SecretValue { value })
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_secret(key: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .security
        .delete_secret(&key)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn factory_reset(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.biometric.reset();

    if let Some(db) = state.take_db() {
        db.close().await;
    }

    vault::reset_vault_dir(&state.vault_dir).map_err(|err| err.to_string())?;
    db::delete_database_files(&state.db_path).map_err(|err| err.to_string())?;
    state
        .security
        .reset_vault()
        .map_err(|err| err.to_string())?;

    let new_db = db::init_pool(&state.db_path)
        .await
        .map_err(|err| err.to_string())?;
    state.replace_db(new_db);

    emit_vault_status(&app, &state.security, "factory-reset")
}

/// Command: Safely reset the vault and all stored secrets
///
/// WARNING: This is completely irreversible!
/// Use only when user has forgotten the passphrase.
///
/// This will:
/// 1. Close the vault session
/// 2. Delete the stronghold snapshot file
/// 3. User can then set up a new vault with a new passphrase
#[tauri::command]
pub async fn reset_vault_and_secrets(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    factory_reset(app, state).await
}

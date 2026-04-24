use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{security, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingState {
    pub user_name: String,
    pub tone: String,
    pub local_model: String,
    pub cloud_fallback: String,
    pub integrations: Vec<String>,
    pub auth_method: String,
    pub cloud_providers: Vec<String>,
    pub tor_enabled: bool,
}

fn onboarding_path(state: &AppState) -> PathBuf {
    // myspace_dir = .../viboai/myspace → parent = .../viboai/
    state
        .myspace_dir
        .parent()
        .map(|p| p.join("onboarding.json"))
        .unwrap_or_else(|| PathBuf::from("onboarding.json"))
}

#[tauri::command]
pub async fn read_onboarding(
    state: State<'_, AppState>,
) -> Result<Option<OnboardingState>, String> {
    let path = onboarding_path(&state);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| format!("read onboarding: {}", e))?;
    let parsed: OnboardingState =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse onboarding: {}", e))?;
    Ok(Some(parsed))
}

#[tauri::command]
pub async fn write_onboarding(
    config: OnboardingState,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = onboarding_path(&state);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir parent: {}", e))?;
    }
    let data = serde_json::to_vec_pretty(&config).map_err(|e| format!("serialize: {}", e))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &data).map_err(|e| format!("write tmp: {}", e))?;
    fs::rename(&tmp, &path).map_err(|e| format!("atomic rename: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn reset_onboarding(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Nuke the vault snapshot FIRST (atomic ordering: if vault reset fails, we
    // don't delete onboarding.json — user can retry). This prevents the
    // BadFileKey scenario where re-onboarding calls setup(new_pin) but the old
    // snapshot is still on disk keyed to the previous passphrase.
    state
        .security
        .reset_vault()
        .map_err(|e| e.to_string())?;

    // Only delete onboarding preferences after vault is confirmed gone.
    let path = onboarding_path(&state);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("remove onboarding: {}", e))?;
    }

    // Emit vault status so the frontend transitions to onboarding phase immediately.
    security::emit_vault_status(&app, &state.security, "reset-onboarding")
}

#[tauri::command]
pub async fn is_onboarding_complete(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(onboarding_path(&state).exists())
}

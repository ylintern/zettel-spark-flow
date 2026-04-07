use tauri::State;

use crate::{db, models::WorkspaceNote, models::WorkspaceSnapshot, state::AppState};

#[tauri::command]
pub async fn load_workspace_snapshot(
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let db = state.db().map_err(|err| err.to_string())?;

    db::load_workspace_snapshot(&db, &state.vault_dir, &state.security)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_note(note: WorkspaceNote, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db().map_err(|err| err.to_string())?;

    db::save_note(&db, &state.vault_dir, &note, &state.security)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_note(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db().map_err(|err| err.to_string())?;

    db::delete_note(&db, &state.vault_dir, &id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_folder(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db().map_err(|err| err.to_string())?;

    db::create_folder(&db, &name)
        .await
        .map_err(|err| err.to_string())
}

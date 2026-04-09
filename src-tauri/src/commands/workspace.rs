use serde::Serialize;
use tauri::{AppHandle, State};

use crate::{
    db,
    models::{CallerContext, KanbanColumn, WorkspaceNote, WorkspaceSnapshot},
    state::AppState,
};

#[derive(Serialize)]
pub struct ExportedNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn load_workspace_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let db = state.db().map_err(|err| err.to_string())?;

    db::load_workspace_snapshot(&db, &state.database_dir, &state.security, Some(&app))
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_note(
    note: WorkspaceNote,
    state: State<'_, AppState>,
    caller: CallerContext,
) -> Result<(), String> {
    if let CallerContext::Agent { ref agent_id } = caller {
        log::info!(
            "[AGENT MUTATION] agent_id={} command=save_note target_id={}",
            agent_id,
            note.id
        );
    }

    let db = state.db().map_err(|err| err.to_string())?;

    db::save_note(&db, &state.database_dir, &note, &state.security)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_note(
    id: String,
    state: State<'_, AppState>,
    caller: CallerContext,
) -> Result<(), String> {
    if let CallerContext::Agent { ref agent_id } = caller {
        log::info!(
            "[AGENT MUTATION] agent_id={} command=delete_note target_id={}",
            agent_id,
            id
        );
    }

    let db = state.db().map_err(|err| err.to_string())?;

    db::delete_note(&db, &state.database_dir, &id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_folder(
    name: String,
    state: State<'_, AppState>,
    caller: CallerContext,
) -> Result<(), String> {
    if let CallerContext::Agent { ref agent_id } = caller {
        log::info!(
            "[AGENT MUTATION] agent_id={} command=create_folder name={}",
            agent_id,
            name
        );
    }

    let db = state.db().map_err(|err| err.to_string())?;

    db::create_folder(&db, &name)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn save_column(
    column: KanbanColumn,
    state: State<'_, AppState>,
    caller: CallerContext,
) -> Result<(), String> {
    if let CallerContext::Agent { ref agent_id } = caller {
        log::info!(
            "[AGENT MUTATION] agent_id={} command=save_column target_id={}",
            agent_id,
            column.id
        );
    }

    let db = state.db().map_err(|err| err.to_string())?;

    db::save_column(&db, &column)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn export_notes(
    state: State<'_, AppState>,
    caller: CallerContext,
) -> Result<String, String> {
    if let CallerContext::Agent { ref agent_id } = caller {
        log::info!(
            "[AGENT MUTATION] agent_id={} command=export_notes",
            agent_id
        );
    }

    let db = state.db().map_err(|err| err.to_string())?;

    let snapshot = db::load_workspace_snapshot(&db, &state.database_dir, &state.security, None)
        .await
        .map_err(|err| err.to_string())?;

    let exported: Vec<ExportedNote> = snapshot
        .notes
        .into_iter()
        .filter(|n| !n.is_kanban)
        .map(|n| ExportedNote {
            id: n.id,
            title: n.title,
            content: n.content,
            created_at: n.created_at,
            updated_at: n.updated_at,
        })
        .collect();

    serde_json::to_string_pretty(&exported).map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delete_column(
    id: String,
    state: State<'_, AppState>,
    caller: CallerContext,
) -> Result<(), String> {
    if let CallerContext::Agent { ref agent_id } = caller {
        log::info!(
            "[AGENT MUTATION] agent_id={} command=delete_column target_id={}",
            agent_id,
            id
        );
    }

    let db = state.db().map_err(|err| err.to_string())?;

    db::delete_column(&db, &id)
        .await
        .map_err(|err| err.to_string())
}

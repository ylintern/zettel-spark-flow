//! `viboinference_*` — thin command handlers wrapping [`crate::services::inference::InferenceService`].
//!
//! Every command is a one-liner that forwards to the service. Keep business
//! logic out of this file — it lives in the service so Rust-side triggers
//! (cron, events, tasks) share the same code path.

use tauri::{AppHandle, Runtime, State};

use crate::services::{
    inference::InferenceService,
    llama::SessionInfo,
    model_catalog::ModelEntryDto,
};

#[tauri::command]
pub fn viboinference_list_models(
    svc: State<'_, InferenceService>,
) -> Vec<ModelEntryDto> {
    svc.list_models()
}

#[tauri::command]
pub fn viboinference_list_downloaded<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
) -> Result<Vec<String>, String> {
    svc.list_downloaded(&app)
}

#[tauri::command]
pub async fn viboinference_download_model<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
    id: String,
) -> Result<(), String> {
    svc.download_model(&app, &id).await
}

#[tauri::command]
pub fn viboinference_delete_model<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
    id: String,
) -> Result<(), String> {
    svc.delete_model(&app, &id)
}

#[tauri::command]
pub fn viboinference_get_active_model(
    svc: State<'_, InferenceService>,
) -> Option<String> {
    svc.get_active_model()
}

#[tauri::command]
pub fn viboinference_set_active_model<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
    id: String,
) -> Result<(), String> {
    svc.set_active_model(&app, &id)
}

#[tauri::command]
pub fn viboinference_start_chat_session<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
    system_prompt: Option<String>,
) -> Result<String, String> {
    svc.start_chat_session(&app, system_prompt)
}

#[tauri::command]
pub fn viboinference_stream_chat<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
    session_id: String,
    prompt: String,
) -> Result<String, String> {
    svc.stream_chat(&app, &session_id, &prompt)
}

#[tauri::command]
pub fn viboinference_stop_generation<R: Runtime>(
    app: AppHandle<R>,
    svc: State<'_, InferenceService>,
    generation_id: String,
) -> Result<(), String> {
    svc.stop_generation(&app, &generation_id)
}

#[tauri::command]
pub fn viboinference_end_chat_session(
    svc: State<'_, InferenceService>,
    session_id: String,
) -> Result<(), String> {
    svc.end_chat_session(&session_id);
    Ok(())
}

#[tauri::command]
pub fn viboinference_session_info(
    svc: State<'_, InferenceService>,
    session_id: String,
) -> Option<SessionInfo> {
    svc.session_info(&session_id)
}

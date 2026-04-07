#![allow(dead_code)]

use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const NOTE_INDEXING_PROGRESS_EVENT: &str = "note_indexing_progress";
pub const AGENT_THINKING_DELTA_EVENT: &str = "agent_thinking_delta";
pub const VAULT_STATUS_CHANGED_EVENT: &str = "vault_status_changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteIndexingProgressEvent {
    pub note_id: Option<String>,
    pub stage: String,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentThinkingDeltaEvent {
    pub session_id: Option<String>,
    pub delta: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatusChangedEvent {
    pub configured: bool,
    pub unlocked: bool,
    pub reason: String,
}

pub fn emit_note_indexing_progress(
    app: &AppHandle,
    payload: &NoteIndexingProgressEvent,
) -> Result<(), String> {
    app.emit(NOTE_INDEXING_PROGRESS_EVENT, payload)
        .map_err(|err| err.to_string())
}

pub fn emit_agent_thinking_delta(
    app: &AppHandle,
    payload: &AgentThinkingDeltaEvent,
) -> Result<(), String> {
    app.emit(AGENT_THINKING_DELTA_EVENT, payload)
        .map_err(|err| err.to_string())
}

pub fn emit_vault_status_changed(
    app: &AppHandle,
    payload: &VaultStatusChangedEvent,
) -> Result<(), String> {
    app.emit(VAULT_STATUS_CHANGED_EVENT, payload)
        .map_err(|err| err.to_string())
}

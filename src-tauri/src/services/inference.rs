//! InferenceService — single Rust choke point for all local-LLM inference.
//!
//! ## Design (T12 / Path A)
//!
//! Chat is now served by [`super::llama::LlamaService`] running directly on
//! `llama-cpp-2`. The leap-ai plugin is retained ONLY for model downloads
//! and on-disk cache management:
//!
//!   - `download_model` / progress events
//!   - `list_cached_models` (used to resolve GGUF paths)
//!   - `remove_cached_model`
//!
//! All chat-path plugin calls (`load_model`, `unload_model`,
//! `create_conversation`, `generate`, `stop_generation`, `end_conversation`)
//! have been removed. Two paths to llama.cpp would fight over the Metal
//! backend.
//!
//! ## Event translation
//!
//!   - `vibo://model-download-progress`  ← plugin `download-progress`
//!   - `vibo://model-state`              ← plugin `model-loaded` / `model-unloaded` / `download-removed`
//!   - `vibo://chat-delta`               ← emitted directly by LlamaService
//!   - `vibo://chat-done`                ← emitted directly by LlamaService
//!   - `vibo://session-handover-needed`  ← emitted directly by LlamaService

use std::{
    collections::HashMap,
    path::{PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager, Runtime};
use tauri_plugin_leap_ai::{
    DownloadModelRequest, LeapAiExt, LeapEvent, RemoveCachedModelRequest,
};

use super::llama::LlamaService;
use super::llama_config::SessionConfig;
use super::model_catalog;

const LEAP_EVENT_CHANNEL: &str = "leap-ai://event";
const PREFS_FILENAME: &str = "preferences.json";

// ── Event payloads re-emitted to the frontend ────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgressEvent {
    model_id: String,
    progress: f64,
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelStateEvent {
    model_id: String,
    state: String,
}

// ── Persisted preferences ────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Preferences {
    active_model_id: Option<String>,
}

fn prefs_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_local_data_dir()
        .map(|d| d.join("viboai").join(PREFS_FILENAME))
        .unwrap_or_else(|_| PathBuf::from(PREFS_FILENAME))
}

fn load_prefs<R: Runtime>(app: &AppHandle<R>) -> Preferences {
    let path = prefs_path(app);
    std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice::<Preferences>(&b).ok())
        .unwrap_or_default()
}

fn save_prefs<R: Runtime>(app: &AppHandle<R>, prefs: &Preferences) -> Result<(), String> {
    let path = prefs_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("prefs mkdir: {}", e))?;
    }
    let data = serde_json::to_vec_pretty(prefs).map_err(|e| format!("prefs serialize: {}", e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &data).map_err(|e| format!("prefs tmp: {}", e))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("prefs rename: {}", e))?;
    Ok(())
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct InferenceState {
    /// Model label (download `model` field) → our internal slug id.
    /// Populated up-front so download-progress events route to the right card.
    label_to_id: HashMap<String, String>,
    /// Plugin-internal "model_N" id → our slug. Discovered lazily from
    /// the start chunk URL (see `parse_start_chunk_url`).
    download_id_map: HashMap<String, String>,
    /// User-active model id (persisted to `preferences.json`).
    active_model_id: Option<String>,
}

pub struct InferenceService {
    state: Mutex<InferenceState>,
    llama: Arc<LlamaService>,
}

impl InferenceService {
    /// Construct, init the llama-cpp-2 backend, and attach the leap-ai
    /// event listener. Call once at app setup.
    pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<Self, String> {
        let prefs = load_prefs(app);

        let mut label_to_id = HashMap::new();
        for m in model_catalog::MODELS {
            label_to_id.insert(m.desktop.label.to_string(), m.id.to_string());
        }

        let llama = Arc::new(LlamaService::init()?);

        let service = Self {
            state: Mutex::new(InferenceState {
                active_model_id: prefs.active_model_id,
                label_to_id,
                ..Default::default()
            }),
            llama,
        };

        let handle = app.clone();
        app.listen_any(LEAP_EVENT_CHANNEL, move |event| {
            match serde_json::from_str::<LeapEvent>(event.payload()) {
                Ok(leap_event) => forward_leap_event(&handle, &leap_event),
                Err(e) => log::warn!("leap event decode failed: {}", e),
            }
        });

        Ok(service)
    }

    // ── Model catalog / download ──────────────────────────────────────

    pub fn list_models(&self) -> Vec<model_catalog::ModelEntryDto> {
        model_catalog::dto_list()
    }

    /// Cross-reference the plugin's cached models with our catalog and
    /// return the set of internal ids currently on disk.
    pub fn list_downloaded<R: Runtime>(&self, app: &AppHandle<R>) -> Result<Vec<String>, String> {
        let cached = app
            .leap_ai()
            .list_cached_models()
            .map_err(|e| e.to_string())?;

        let ids: Vec<String> = model_catalog::MODELS
            .iter()
            .filter(|m| cached.iter().any(|c| c.model == m.desktop.label))
            .map(|m| m.id.to_string())
            .collect();

        Ok(ids)
    }

    pub async fn download_model<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        id: &str,
    ) -> Result<(), String> {
        let entry = model_catalog::get(id)
            .ok_or_else(|| format!("unknown model id: {}", id))?;

        let req = DownloadModelRequest {
            model: entry.desktop.label.to_string(),
            quantization: Some(entry.desktop.quantization.to_string()),
            url: Some(entry.desktop.gguf_url.to_string()),
        };
        app.leap_ai()
            .download_model(req)
            .await
            .map_err(|e| format!("download failed: {}", e))?;
        Ok(())
    }

    pub fn delete_model<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        id: &str,
    ) -> Result<(), String> {
        let entry = model_catalog::get(id)
            .ok_or_else(|| format!("unknown model id: {}", id))?;

        let cached = app
            .leap_ai()
            .list_cached_models()
            .map_err(|e| e.to_string())?;
        let cache_key = cached
            .iter()
            .find(|c| c.model == entry.desktop.label)
            .map(|c| c.cache_key.clone())
            .ok_or_else(|| "model not cached".to_string())?;

        // Drop from the llama backend first (closes any sessions).
        self.llama.unload_model(id);

        app.leap_ai()
            .remove_cached_model(RemoveCachedModelRequest { cache_key })
            .map_err(|e| format!("delete failed: {}", e))?;

        let cleared = {
            let mut st = self.state.lock().unwrap();
            if st.active_model_id.as_deref() == Some(id) {
                st.active_model_id = None;
                true
            } else {
                false
            }
        };
        if cleared {
            let _ = save_prefs(app, &Preferences { active_model_id: None });
        }
        Ok(())
    }

    // ── Active model / load / unload ──────────────────────────────────

    pub fn get_active_model(&self) -> Option<String> {
        self.state.lock().unwrap().active_model_id.clone()
    }

    /// Set the user-selected active model. Loads it via LlamaService and
    /// unloads any previous. GGUF path is resolved through the leap-ai
    /// plugin's cache (`list_cached_models().local_path`).
    pub fn set_active_model<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        id: &str,
    ) -> Result<(), String> {
        let entry = model_catalog::get(id)
            .ok_or_else(|| format!("unknown model id: {}", id))?;

        // Locate the GGUF file on disk via the plugin's cache.
        let cached = app
            .leap_ai()
            .list_cached_models()
            .map_err(|e| format!("list_cached_models: {}", e))?;
        let path = cached
            .iter()
            .find(|c| c.model == entry.desktop.label)
            .map(|c| PathBuf::from(&c.local_path))
            .ok_or_else(|| format!("model {} not on disk — download first", id))?;

        // Unload the previous active model (if any and different).
        let prev = {
            let st = self.state.lock().unwrap();
            st.active_model_id.clone()
        };
        if let Some(prev_id) = prev {
            if prev_id != id {
                self.llama.unload_model(&prev_id);
            }
        }

        // Load the new one.
        self.llama.load_model(id, entry.alias, &path)?;

        // Emit a synthetic model-state so the frontend's existing listeners
        // (LocalModelsSection's `onModelState`) refresh without a poll.
        let _ = app.emit(
            "vibo://model-state",
            ModelStateEvent {
                model_id: id.to_string(),
                state: "model-loaded".to_string(),
            },
        );

        {
            let mut st = self.state.lock().unwrap();
            st.active_model_id = Some(id.to_string());
        }
        save_prefs(
            app,
            &Preferences {
                active_model_id: Some(id.to_string()),
            },
        )?;
        Ok(())
    }

    // ── Chat sessions ─────────────────────────────────────────────────

    pub fn start_chat_session<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        system_prompt: Option<String>,
    ) -> Result<String, String> {
        let active = self
            .state
            .lock()
            .unwrap()
            .active_model_id
            .clone()
            .ok_or_else(|| "no active model set".to_string())?;

        // Lazy-load: if this is the first call after a relaunch, the model
        // is in prefs but not yet in VRAM. set_active_model handles both.
        if !self.llama.loaded_ids().iter().any(|i| i == &active) {
            self.set_active_model(app, &active)?;
        }

        self.llama
            .start_session(&active, system_prompt, SessionConfig::default())
    }

    pub fn stream_chat<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        session_id: &str,
        prompt: &str,
    ) -> Result<String, String> {
        self.llama.stream(app, session_id, prompt)
    }

    pub fn stop_generation<R: Runtime>(
        &self,
        _app: &AppHandle<R>,
        generation_id: &str,
    ) -> Result<(), String> {
        self.llama.stop(generation_id);
        Ok(())
    }

    pub fn end_chat_session(&self, session_id: &str) {
        self.llama.end_session(session_id);
    }

    pub fn session_info(&self, session_id: &str) -> Option<super::llama::SessionInfo> {
        self.llama.session_info(session_id)
    }

    // ── Internal helpers used by the event forwarder ──────────────────

    fn id_for_label(&self, label: &str) -> Option<String> {
        self.state.lock().unwrap().label_to_id.get(label).cloned()
    }

    fn id_for_download_label(&self, plugin_label: &str) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .download_id_map
            .get(plugin_label)
            .cloned()
    }

    fn register_download_label(&self, plugin_label: String, our_id: String) {
        self.state
            .lock()
            .unwrap()
            .download_id_map
            .insert(plugin_label, our_id);
    }
}

/// Find our internal slug for a Hugging Face URL by exact catalog match.
fn id_by_url(url: &str) -> Option<String> {
    model_catalog::MODELS
        .iter()
        .find(|m| m.desktop.gguf_url == url)
        .map(|m| m.id.to_string())
}

/// Parse `"starting download from <url>"` start-chunks emitted by the
/// leap-ai plugin's downloader.
fn parse_start_chunk_url(chunk: &str) -> Option<String> {
    chunk
        .strip_prefix("starting download from ")
        .map(|s| s.trim().to_string())
}

// ── Event forwarder ──────────────────────────────────────────────────────
//
// Chat events (`generation-chunk`, `generation-complete`) are NO LONGER
// handled here — LlamaService emits `vibo://chat-delta` / `vibo://chat-done`
// directly. We only translate download / model-lifecycle events.

fn forward_leap_event<R: Runtime>(app: &AppHandle<R>, ev: &LeapEvent) {
    let Some(svc) = app.try_state::<InferenceService>() else {
        return;
    };

    match ev.kind.as_str() {
        "download-progress" => {
            let Some(plugin_label) = ev.model_id.as_ref() else {
                return;
            };

            let our_id = svc
                .id_for_label(plugin_label)
                .or_else(|| svc.id_for_download_label(plugin_label))
                .or_else(|| {
                    let chunk = ev.chunk.as_deref()?;
                    let url = parse_start_chunk_url(chunk)?;
                    let id = id_by_url(&url)?;
                    svc.register_download_label(plugin_label.clone(), id.clone());
                    Some(id)
                });

            let Some(our_id) = our_id else {
                log::debug!(
                    "download-progress: no route for plugin id '{}' (chunk: {:?})",
                    plugin_label,
                    ev.chunk
                );
                return;
            };

            let _ = app.emit(
                "vibo://model-download-progress",
                DownloadProgressEvent {
                    model_id: our_id,
                    progress: ev.progress.unwrap_or(0.0),
                    status: ev.chunk.clone(),
                },
            );
        }
        "model-loaded" | "model-unloaded" | "download-removed" => {
            // Plugin emits these for its OWN load/unload — under T12 we no
            // longer call plugin load_model, so these come from downloads
            // (`download-removed`) and from any plugin-side auto-load that
            // still fires on download completion. Translate by best effort.
            let plugin_model_id = ev.model_id.as_deref().unwrap_or("");
            let our_id = svc
                .id_for_label(plugin_model_id)
                .or_else(|| svc.id_for_download_label(plugin_model_id))
                .unwrap_or_else(|| plugin_model_id.to_string());
            let _ = app.emit(
                "vibo://model-state",
                ModelStateEvent {
                    model_id: our_id,
                    state: ev.kind.clone(),
                },
            );
        }
        "generation-chunk" | "generation-complete" => {
            // Ignored — LlamaService emits chat events directly. If the
            // plugin emits these, that means we accidentally still have a
            // plugin-side conversation alive somewhere; log to surface bugs.
            log::debug!(
                "leap event: ignoring '{}' (chat is on LlamaService now)",
                ev.kind
            );
        }
        other => {
            log::debug!("leap event: unhandled type '{}'", other);
        }
    }
}

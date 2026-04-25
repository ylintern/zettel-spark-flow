//! LlamaService — in-process chat inference on `llama-cpp-2`.
//!
//! Replaces the chat path of `tauri-plugin-leap-ai` (T12 / Path A). The
//! plugin's `build_generation_prompt` ignored the GGUF embedded
//! `tokenizer.chat_template` and serialized history as plain
//! `system: ...\nuser: ...` text; LFM 2.5 (chatml-trained) responded with
//! multilingual hallucinations. This service:
//!
//!   1. Reads the GGUF's embedded jinja chat_template via `LlamaModel::chat_template`.
//!   2. Renders history with [`minijinja`] each turn.
//!   3. Runs its own decode loop with a `LlamaSampler` chain we control.
//!   4. Streams tokens via `vibo://chat-delta` / `vibo://chat-done`.
//!   5. Watches the KV-cache fill and emits `vibo://session-handover-needed`
//!      when usage crosses [`SessionConfig::soft_handover_threshold`] so a
//!      future Swiftide layer can compact + spawn a continuation session.
//!
//! Convergence note: the leap-ai plugin transitively depends on the same
//! `llama-cpp-2 = "0.1"` crate, so Cargo dedupes to a single C++ static
//! lib. We must NOT also call the plugin's `load_model`/`generate` once
//! this service is wired in — that would fight over the same backend.

use std::{
    collections::HashMap,
    num::NonZeroU32,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};
use minijinja::{context, Environment};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use uuid::Uuid;

use super::llama_config::{SessionConfig, MAX_ACTIVE_SESSIONS};

/// Singleton `LlamaBackend` — `llama_backend_init` may only be called once
/// per process.
static BACKEND: OnceLock<Arc<LlamaBackend>> = OnceLock::new();

#[derive(Clone, Serialize)]
struct ChatTurn {
    role: String,
    content: String,
}

/// Tagged-union error reported to the frontend via the `error` field of
/// `vibo://chat-done` (JSON-encoded into the existing string slot so the
/// TS contract stays unchanged).
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DoneError {
    MaxTokens,
    ContextOverflow { used: u32, max: u32 },
    Timeout,
    Aborted,
    Tokenize { detail: String },
    TemplateRender { detail: String },
    Internal { detail: String },
}

struct LoadedModel {
    model: Arc<LlamaModel>,
    chat_template: String,
    alias: String,
}

struct GenHandle {
    abort: Arc<AtomicBool>,
    #[allow(dead_code)]
    started: Instant,
}

struct Session {
    model_id: String,
    history: Vec<ChatTurn>,
    cfg: SessionConfig,
    used_tokens: u32,
    generation: Option<(String, GenHandle)>,
    last_used: Instant,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub model_alias: String,
    pub n_ctx: u32,
    pub used_tokens: u32,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub struct LlamaService {
    loaded: Mutex<HashMap<String, Arc<LoadedModel>>>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl LlamaService {
    pub fn init() -> Result<Self, String> {
        BACKEND.get_or_init(|| {
            Arc::new(
                LlamaBackend::init()
                    .expect("LlamaBackend::init() must succeed once per process"),
            )
        });
        Ok(Self {
            loaded: Mutex::new(HashMap::new()),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn backend() -> Arc<LlamaBackend> {
        BACKEND
            .get()
            .expect("LlamaBackend not initialized — call LlamaService::init first")
            .clone()
    }

    /// Load a GGUF off disk into VRAM. `our_id` is the catalog id (e.g.
    /// `lfm2-1.2b`); `alias` is a short tag for UI (`junior` / `specialist`);
    /// `path` is the absolute path returned by the leap-ai plugin's
    /// `list_cached_models()` (`local_path` field).
    pub fn load_model(&self, our_id: &str, alias: &str, path: &Path) -> Result<(), String> {
        let backend = Self::backend();
        let params = LlamaModelParams::default().with_n_gpu_layers(999);
        let model = LlamaModel::load_from_file(&backend, path, &params)
            .map_err(|e| format!("load_from_file: {e}"))?;
        let template = model
            .chat_template(None)
            .ok()
            .and_then(|t| t.to_string().ok())
            .unwrap_or_default();
        if template.is_empty() {
            log::warn!(
                "[llama] model {our_id} has no embedded chat_template — falling back to chatml"
            );
        }
        let lm = LoadedModel {
            model: Arc::new(model),
            chat_template: template,
            alias: alias.to_string(),
        };
        self.loaded
            .lock()
            .unwrap()
            .insert(our_id.to_string(), Arc::new(lm));
        log::info!("[llama] loaded model id={our_id} alias={alias} path={path:?}");
        Ok(())
    }

    pub fn unload_model(&self, our_id: &str) {
        let to_end: Vec<String> = self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, s)| s.model_id == our_id)
            .map(|(k, _)| k.clone())
            .collect();
        for sid in to_end {
            self.end_session(&sid);
        }
        self.loaded.lock().unwrap().remove(our_id);
        log::info!("[llama] unloaded model id={our_id}");
    }

    pub fn loaded_ids(&self) -> Vec<String> {
        self.loaded.lock().unwrap().keys().cloned().collect()
    }

    pub fn start_session(
        &self,
        our_id: &str,
        system: Option<String>,
        cfg: SessionConfig,
    ) -> Result<String, String> {
        if !self.loaded.lock().unwrap().contains_key(our_id) {
            return Err(format!("model not loaded: {our_id}"));
        }

        // LRU eviction.
        {
            let mut sessions = self.sessions.lock().unwrap();
            if sessions.len() >= MAX_ACTIVE_SESSIONS {
                if let Some(oldest) = sessions
                    .iter()
                    .min_by_key(|(_, s)| s.last_used)
                    .map(|(k, _)| k.clone())
                {
                    sessions.remove(&oldest);
                    log::info!("[llama] evicted LRU session {oldest}");
                }
            }
        }

        let mut history = Vec::new();
        if let Some(sys) = system {
            history.push(ChatTurn {
                role: "system".into(),
                content: sys,
            });
        }
        let sid = Uuid::new_v4().to_string();
        self.sessions.lock().unwrap().insert(
            sid.clone(),
            Session {
                model_id: our_id.to_string(),
                history,
                cfg,
                used_tokens: 0,
                generation: None,
                last_used: Instant::now(),
            },
        );
        Ok(sid)
    }

    pub fn end_session(&self, sid: &str) {
        if let Some(s) = self.sessions.lock().unwrap().remove(sid) {
            if let Some((_, h)) = &s.generation {
                h.abort.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn stop(&self, gen_id: &str) {
        let sessions = self.sessions.lock().unwrap();
        for s in sessions.values() {
            if let Some((g, h)) = &s.generation {
                if g == gen_id {
                    h.abort.store(true, Ordering::SeqCst);
                    return;
                }
            }
        }
    }

    pub fn session_info(&self, sid: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        let s = sessions.get(sid)?;
        let loaded = self.loaded.lock().unwrap();
        let lm = loaded.get(&s.model_id)?;
        Some(SessionInfo {
            model_alias: lm.alias.clone(),
            n_ctx: s.cfg.n_ctx,
            used_tokens: s.used_tokens,
            max_tokens: s.cfg.max_tokens,
            temperature: s.cfg.sampler.temperature,
        })
    }

    /// Append the user turn to history, render the chat template, and
    /// kick off decode on a dedicated OS thread. Returns the generation id
    /// immediately; tokens flow asynchronously via `vibo://chat-delta`,
    /// terminating with `vibo://chat-done`.
    pub fn stream<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        sid: &str,
        user: &str,
    ) -> Result<String, String> {
        let (lm, cfg, history_for_render, gen_id, abort_flag) = {
            let mut sessions = self.sessions.lock().unwrap();
            let s = sessions
                .get_mut(sid)
                .ok_or_else(|| format!("unknown session: {sid}"))?;
            s.history.push(ChatTurn {
                role: "user".into(),
                content: user.to_string(),
            });
            s.last_used = Instant::now();
            let loaded = self.loaded.lock().unwrap();
            let lm = loaded.get(&s.model_id).ok_or("model not loaded")?.clone();
            let gen_id = Uuid::new_v4().to_string();
            let abort = Arc::new(AtomicBool::new(false));
            s.generation = Some((
                gen_id.clone(),
                GenHandle {
                    abort: abort.clone(),
                    started: Instant::now(),
                },
            ));
            (lm, s.cfg, s.history.clone(), gen_id, abort)
        };

        let prompt = render_template(&lm.chat_template, &history_for_render)
            .map_err(|e| format!("template render: {e}"))?;

        let app_handle = app.clone();
        let sid_owned = sid.to_string();
        let gen_id_owned = gen_id.clone();
        let sessions_arc = self.sessions.clone();

        std::thread::spawn(move || {
            let result = run_decode(
                &app_handle,
                &sid_owned,
                &gen_id_owned,
                &lm,
                &cfg,
                &prompt,
                abort_flag,
                &sessions_arc,
            );
            let error_str = result
                .err()
                .map(|e| serde_json::to_string(&e).unwrap_or_default());

            // Soft-handover signal — UI displays it; Swiftide hooks in later.
            if let Some(s) = sessions_arc.lock().unwrap().get(&sid_owned) {
                if s.used_tokens > cfg.soft_handover_threshold {
                    let _ = app_handle.emit(
                        "vibo://session-handover-needed",
                        serde_json::json!({
                            "sessionId": sid_owned,
                            "usedTokens": s.used_tokens,
                        }),
                    );
                }
            }

            let _ = app_handle.emit(
                "vibo://chat-done",
                serde_json::json!({
                    "sessionId": sid_owned,
                    "generationId": gen_id_owned,
                    "error": error_str,
                }),
            );

            if let Some(s) = sessions_arc.lock().unwrap().get_mut(&sid_owned) {
                s.generation = None;
            }
        });

        Ok(gen_id)
    }
}

fn render_template(template: &str, history: &[ChatTurn]) -> Result<String, String> {
    if template.is_empty() {
        // Fallback to chatml when the GGUF didn't embed one.
        let mut out = String::new();
        for t in history {
            out.push_str(&format!(
                "<|im_start|>{}\n{}<|im_end|>\n",
                t.role, t.content
            ));
        }
        out.push_str("<|im_start|>assistant\n");
        return Ok(out);
    }
    let mut env = Environment::new();
    env.add_template("chat", template)
        .map_err(|e| e.to_string())?;
    let tmpl = env.get_template("chat").map_err(|e| e.to_string())?;
    tmpl.render(context! {
        messages => history,
        add_generation_prompt => true,
        bos_token => "<|begin_of_text|>",
        eos_token => "<|end_of_text|>",
    })
    .map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
fn run_decode<R: Runtime>(
    app: &AppHandle<R>,
    sid: &str,
    gen_id: &str,
    lm: &Arc<LoadedModel>,
    cfg: &SessionConfig,
    prompt: &str,
    abort: Arc<AtomicBool>,
    sessions: &Arc<Mutex<HashMap<String, Session>>>,
) -> Result<(), DoneError> {
    let backend = LlamaService::backend();
    let ctx_params =
        LlamaContextParams::default().with_n_ctx(NonZeroU32::new(cfg.n_ctx));
    let mut ctx = lm
        .model
        .new_context(&backend, ctx_params)
        .map_err(|e| DoneError::Internal {
            detail: format!("ctx new: {e}"),
        })?;

    let tokens = lm
        .model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| DoneError::Tokenize {
            detail: e.to_string(),
        })?;

    let n_input = tokens.len() as u32;
    let n_ctx = cfg.n_ctx;
    if n_input + 64 > n_ctx {
        return Err(DoneError::ContextOverflow {
            used: n_input,
            max: n_ctx,
        });
    }

    if let Some(s) = sessions.lock().unwrap().get_mut(sid) {
        s.used_tokens = s.used_tokens.saturating_add(n_input);
    }

    // Decode prompt.
    let mut batch = LlamaBatch::new(tokens.len(), 1);
    let last_idx = (tokens.len() - 1) as i32;
    for (i, tok) in tokens.iter().enumerate() {
        batch
            .add(*tok, i as i32, &[0], i as i32 == last_idx)
            .map_err(|e| DoneError::Internal {
                detail: format!("batch add: {e}"),
            })?;
    }
    ctx.decode(&mut batch).map_err(|e| DoneError::Internal {
        detail: format!("decode: {e}"),
    })?;

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::penalties(64, cfg.sampler.repeat_penalty, 0.0, 0.0),
        LlamaSampler::top_k(cfg.sampler.top_k),
        LlamaSampler::top_p(cfg.sampler.top_p, 1),
        LlamaSampler::temp(cfg.sampler.temperature),
        LlamaSampler::dist(cfg.sampler.seed),
    ]);

    let mut kv_pos = tokens.len() as i32;
    let mut produced: u32 = 0;
    let timeout = Duration::from_secs(cfg.generation_timeout_secs);
    let started = Instant::now();
    let mut accum = String::new();

    loop {
        if abort.load(Ordering::SeqCst) {
            return Err(DoneError::Aborted);
        }
        if started.elapsed() > timeout {
            return Err(DoneError::Timeout);
        }
        if produced >= cfg.max_tokens {
            return Err(DoneError::MaxTokens);
        }
        if (kv_pos as u32) + 1 >= n_ctx {
            return Err(DoneError::ContextOverflow {
                used: kv_pos as u32,
                max: n_ctx,
            });
        }

        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if lm.model.is_eog_token(token) {
            break;
        }

        // `token_to_str` is marked deprecated but is still the simplest
        // single-call detokenizer in 0.1.x; the canonical replacement
        // requires holding a stateful `encoding_rs::Decoder` across the
        // loop. Acceptable until we hit multi-byte UTF-8 split issues.
        #[allow(deprecated)]
        let piece = lm
            .model
            .token_to_str(token, Special::Tokenize)
            .unwrap_or_default();
        accum.push_str(&piece);

        let _ = app.emit(
            "vibo://chat-delta",
            serde_json::json!({
                "sessionId": sid,
                "generationId": gen_id,
                "delta": piece,
            }),
        );

        produced += 1;

        batch.clear();
        batch
            .add(token, kv_pos, &[0], true)
            .map_err(|e| DoneError::Internal {
                detail: format!("batch add cont: {e}"),
            })?;
        kv_pos += 1;
        ctx.decode(&mut batch).map_err(|e| DoneError::Internal {
            detail: format!("decode cont: {e}"),
        })?;
    }

    if let Some(s) = sessions.lock().unwrap().get_mut(sid) {
        s.history.push(ChatTurn {
            role: "assistant".into(),
            content: accum,
        });
        s.used_tokens = s.used_tokens.saturating_add(produced);
        s.last_used = Instant::now();
    }
    Ok(())
}

//! Configuration for [`super::llama::LlamaService`] — sampling, session
//! limits, token budget.
//!
//! Defaults are tuned for LFM 2.5 GGUF models (both junior 350M and
//! specialist 1.2B share the same ceiling):
//!   - 131 072 tokens session context (`n_ctx` default = full LFM capacity)
//!   - 131 072 tokens session ceiling (`N_CTX_HARD_MAX`)
//!   - 8 192 tokens generation cap (`max_tokens`)
//!
//! LFM 2.5 uses a hybrid SSM/recurrent architecture — KV cache scales far
//! better than pure transformers, so the full 128 k context is practical
//! on-device for both model sizes.
//!
//! `system_reserve` is the budget set aside at every session start for
//! framework injection: agent persona file (~1 k) + hot context chunk
//! (~1 k) + tool definitions (~1 k) + skills index (~1 k) + headroom.
//!
//! `soft_handover_threshold` (120 k) is the watermark that emits
//! `vibo://session-handover-needed`. Today the UI only displays it; later
//! Swiftide will hook in to compact + spawn a continuation session.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SamplerConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub seed: u32,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            temperature: 0.85,  // up from 0.7 — warmer, more natural tone
            top_p: 0.95,        // up from 0.9 — wider vocabulary range
            top_k: 64,          // up from 40 — more word diversity
            repeat_penalty: 1.15, // up from 1.1 — breaks loops without clamping personality
            seed: 1234,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SessionConfig {
    pub n_ctx: u32,
    pub max_tokens: u32,
    pub generation_timeout_secs: u64,
    pub system_reserve: u32,
    pub soft_handover_threshold: u32,
    pub sampler: SamplerConfig,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            n_ctx: 131_072,           // full LFM 2.5 capacity — same for both models
            max_tokens: 8_192,        // up from 2 048 — lets responses breathe
            generation_timeout_secs: 300, // up from 120 — 350M needs time on long replies
            system_reserve: 8_192,
            soft_handover_threshold: 122_880, // ~120 k — earlier warning before hard ceiling
            sampler: SamplerConfig::default(),
        }
    }
}

/// LFM 2.5 session ceiling. Refuse to allocate larger contexts even if asked.
pub const N_CTX_HARD_MAX: u32 = 131_072;

/// Maximum concurrent chat sessions before LRU eviction kicks in.
pub const MAX_ACTIVE_SESSIONS: usize = 4;

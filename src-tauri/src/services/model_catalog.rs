//! Curated registry of local LFM models.
//!
//! This is the **single source of truth** for which models Vibo ships.
//! - Rust-side: static catalog used by InferenceService to resolve an
//!   internal slug (e.g. `"lfm2.5-350m"`) into the `.gguf` URL that
//!   `plugin:leap-ai|download_model` requires on desktop.
//! - Frontend-side: sees only `ModelEntryDto` (no URL). Defense in depth —
//!   UI cannot inject arbitrary URLs.
//!
//! ## Aliases / roles
//!
//! Each shipped model has a **canonical alias** (a short nickname) that the
//! agent layer uses to address it by role rather than by raw model id:
//!   - `junior`     → fast generalist (LFM 2.5 350M)
//!   - `specialist` → balanced reasoning (LFM 2.5 1.2B Instruct)
//!   - `thinker`    → reasoning model with `<think>…</think>` blocks
//!     (LFM 2.5 1.2B Thinking) — `enabled: true`. UI parsing of think blocks
//!     is Phase 0.7-B Phase F1.
//!   - `inspector`  → vision (LFM 2.5 VL 450M) — registered with
//!     `enabled: false` until Phase 0.7-B Phase F2 lands the mmproj download
//!     + multimodal inference path. Crate-landscape research completed; see
//!     `Guidelines/source-of-truth/VISION_RESEARCH_2026-04-30.md`
//!     (recommendation: keep `llama-cpp-2 = 0.1.x`, wrap in `services/llama.rs`).
//!
//! Roadmap aliases (not yet shippable, listed for reference only):
//!   - `emb` → embeddings (ModernBERT-base ONNX-ORT) — different runtime
//!     entirely (`ort` crate, not llama.cpp); will live in a separate
//!     `EmbeddingService` rather than this catalog.
//!
//! Platform notes:
//! - **Desktop** (current): llama.cpp in-process, GGUF from Hugging Face.
//! - **Mobile** (task M, deferred): LEAP vendor SDK resolves slugs internally.
//!   `MobileArtifact` placeholder is reserved for that phase.
//!
//! Catalog URLs verified 2026-04-25 against huggingface.co/LiquidAI for the
//! Q4_K_M instruct/350M shards. Thinking + VL shards added 2026-04-28; URL
//! patterns follow the LiquidAI HF convention. The VL `mmproj_url` is left
//! `None` until vision research completes; no UI path can reach it while
//! `enabled: false`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct DesktopArtifact {
    /// Human label passed to `download_model.model` (cache-key input).
    pub label: &'static str,
    /// Quantization string passed to `download_model.quantization`.
    pub quantization: &'static str,
    /// Raw `.gguf` URL fetched on desktop.
    pub gguf_url: &'static str,
}

/// Per-model context-window and modality limits.
///
/// `n_ctx_max` is the model's documented session ceiling (LFM 2.5 → 128 k);
/// `default_n_ctx` is the budget we open chat sessions with by default.
/// `mmproj_url` is the optional companion projector shard for vision models;
/// when `Some`, a vision-aware load path must download + load it before the
/// model can encode images.
/// `supports_thinking` flags models that emit `<think>…</think>` blocks
/// before their visible answer; the FE stream parser routes those into a
/// separate UI lane (Phase F1).
#[derive(Debug, Clone, Copy)]
pub struct ModelLimits {
    pub n_ctx_max: u32,
    pub default_n_ctx: u32,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub mmproj_url: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelEntry {
    /// Internal slug — stable, used everywhere the app refers to this model.
    pub id: &'static str,
    pub name: &'static str,
    /// Short role-name used by the agent layer (e.g. `"junior"`, `"specialist"`).
    /// MUST be unique across MODELS.
    pub alias: &'static str,
    pub family: &'static str,
    pub params: &'static str,
    pub size_mb: u32,
    pub modality: &'static [&'static str],
    pub description: &'static str,
    pub recommended: bool,
    /// `true` = selectable in the UI / downloadable / loadable.
    /// `false` = registered for forward compat (slot in INDEX, alias reserved)
    /// but no UI path may select it. Used for VL until Phase F2 ships.
    pub enabled: bool,
    pub desktop: DesktopArtifact,
    pub limits: ModelLimits,
}

/// Frontend-safe DTO (no URL). Serde rename_all camelCase to match TS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntryDto {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub family: String,
    pub params: String,
    pub size_mb: u32,
    pub modality: Vec<String>,
    pub description: String,
    pub recommended: bool,
    pub enabled: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub quantization: String,
}

impl From<&ModelEntry> for ModelEntryDto {
    fn from(e: &ModelEntry) -> Self {
        Self {
            id: e.id.to_string(),
            name: e.name.to_string(),
            alias: e.alias.to_string(),
            family: e.family.to_string(),
            params: e.params.to_string(),
            size_mb: e.size_mb,
            modality: e.modality.iter().map(|s| s.to_string()).collect(),
            description: e.description.to_string(),
            recommended: e.recommended,
            enabled: e.enabled,
            supports_vision: e.limits.supports_vision,
            supports_thinking: e.limits.supports_thinking,
            quantization: e.desktop.quantization.to_string(),
        }
    }
}

/// Phase 0.7-B catalog: four GGUF models. Two shippable today (junior,
/// specialist), one shippable text-side now and refined later (thinker —
/// `<think>` UI is Phase F1), one registered-but-disabled for forward compat
/// (inspector / VL — vision pipeline lands in Phase F2).
///
/// Equivalent llama.cpp invocations (for reference / parity check):
///   llama-cli   -hf LiquidAI/LFM2.5-350M-GGUF:Q4_K_M
///   llama-cli   -hf LiquidAI/LFM2.5-1.2B-Instruct-GGUF:Q4_K_M
///   llama-server -hf LiquidAI/LFM2.5-1.2B-Thinking-GGUF:Q4_K_M
///   llama-server -hf LiquidAI/LFM2.5-VL-450M-GGUF:Q4_0
pub const MODELS: &[ModelEntry] = &[
    ModelEntry {
        id: "lfm2.5-350m",
        name: "LFM 2.5 Compact",
        alias: "junior",
        family: "LFM 2.5",
        params: "350M",
        size_mb: 219,
        modality: &["text"],
        description: "Fast generalist. Drafting, quick replies, low-latency tasks.",
        recommended: true,
        enabled: true,
        desktop: DesktopArtifact {
            label: "LFM2.5-350M",
            quantization: "Q4_K_M",
            gguf_url: "https://huggingface.co/LiquidAI/LFM2.5-350M-GGUF/resolve/main/LFM2.5-350M-Q4_K_M.gguf",
        },
        limits: ModelLimits {
            n_ctx_max: 131_072,
            default_n_ctx: 32_768,
            supports_vision: false,
            supports_thinking: false,
            mmproj_url: None,
        },
    },
    ModelEntry {
        id: "lfm2.5-1.2b-instruct",
        name: "LFM 2.5 Instruct",
        alias: "specialist",
        family: "LFM 2.5",
        params: "1.2B",
        size_mb: 697,
        modality: &["text"],
        description: "Balanced reasoning. General-purpose text — speed and quality.",
        recommended: true,
        enabled: true,
        desktop: DesktopArtifact {
            label: "LFM2.5-1.2B-Instruct",
            quantization: "Q4_K_M",
            gguf_url: "https://huggingface.co/LiquidAI/LFM2.5-1.2B-Instruct-GGUF/resolve/main/LFM2.5-1.2B-Instruct-Q4_K_M.gguf",
        },
        limits: ModelLimits {
            n_ctx_max: 131_072,
            default_n_ctx: 32_768,
            supports_vision: false,
            supports_thinking: false,
            mmproj_url: None,
        },
    },
    ModelEntry {
        id: "lfm2.5-1.2b-thinking",
        name: "LFM 2.5 Thinking",
        alias: "thinker",
        family: "LFM 2.5",
        params: "1.2B",
        size_mb: 697,
        modality: &["text"],
        description: "Reasoning-first. Emits <think>…</think> blocks before the answer.",
        recommended: false,
        enabled: true,
        desktop: DesktopArtifact {
            label: "LFM2.5-1.2B-Thinking",
            quantization: "Q4_K_M",
            gguf_url: "https://huggingface.co/LiquidAI/LFM2.5-1.2B-Thinking-GGUF/resolve/main/LFM2.5-1.2B-Thinking-Q4_K_M.gguf",
        },
        limits: ModelLimits {
            n_ctx_max: 131_072,
            default_n_ctx: 32_768,
            supports_vision: false,
            supports_thinking: true,
            mmproj_url: None,
        },
    },
    ModelEntry {
        id: "lfm2.5-vl-450m",
        name: "LFM 2.5 Vision",
        alias: "inspector",
        family: "LFM 2.5",
        params: "450M",
        size_mb: 280,
        modality: &["text", "vision"],
        description: "Vision-language. Describes / answers about images. (Disabled until Phase F2 — multimodal pipeline pending research.)",
        recommended: false,
        enabled: false,
        desktop: DesktopArtifact {
            label: "LFM2.5-VL-450M",
            quantization: "Q4_0",
            gguf_url: "https://huggingface.co/LiquidAI/LFM2.5-VL-450M-GGUF/resolve/main/LFM2.5-VL-450M-Q4_0.gguf",
        },
        limits: ModelLimits {
            n_ctx_max: 32_768,
            default_n_ctx: 16_384,
            supports_vision: true,
            supports_thinking: false,
            // mmproj_url stays None until Phase F2 plan-approval picks up
            // the multimodal pipeline. Crate-landscape research is complete:
            // see Guidelines/source-of-truth/VISION_RESEARCH_2026-04-30.md.
            mmproj_url: None,
        },
    },
];

pub fn get(id: &str) -> Option<&'static ModelEntry> {
    MODELS.iter().find(|m| m.id == id)
}

pub fn get_by_alias(alias: &str) -> Option<&'static ModelEntry> {
    MODELS.iter().find(|m| m.alias == alias)
}

/// All catalog entries — including `enabled: false` rows. Used by tools that
/// need the full catalog (e.g. tools-index, model-research scripts).
pub fn dto_list() -> Vec<ModelEntryDto> {
    MODELS.iter().map(ModelEntryDto::from).collect()
}

/// Only entries the user can actually select today (`enabled: true`).
/// Frontend pickers should use this to avoid presenting non-functional rows.
pub fn dto_list_enabled() -> Vec<ModelEntryDto> {
    MODELS
        .iter()
        .filter(|m| m.enabled)
        .map(ModelEntryDto::from)
        .collect()
}

/// Build the deterministic cache-key string the Leap plugin derives internally.
/// Documented shape: `model_cache_key(model, quantization, "download-only-desktop")`.
/// We mirror the format the plugin uses so our list_downloaded() can match entries
/// returned by `list_cached_models`.
pub fn cache_key(entry: &ModelEntry) -> String {
    format!("{}::{}::download-only-desktop", entry.desktop.label, entry.desktop.quantization)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ids_unique() {
        let mut seen = std::collections::HashSet::new();
        for m in MODELS {
            assert!(seen.insert(m.id), "duplicate id: {}", m.id);
        }
    }

    #[test]
    fn all_aliases_unique() {
        let mut seen = std::collections::HashSet::new();
        for m in MODELS {
            assert!(seen.insert(m.alias), "duplicate alias: {}", m.alias);
        }
    }

    #[test]
    fn all_urls_huggingface_https() {
        for m in MODELS {
            assert!(
                m.desktop.gguf_url.starts_with("https://huggingface.co/"),
                "non-HF URL for {}: {}",
                m.id,
                m.desktop.gguf_url
            );
            assert!(m.desktop.gguf_url.ends_with(".gguf"));
        }
    }

    #[test]
    fn at_least_one_recommended() {
        assert!(MODELS.iter().any(|m| m.recommended));
    }

    #[test]
    fn vision_model_is_disabled_until_phase_f2() {
        // VL is registered for forward compat but cannot be selected in UI
        // until the multimodal pipeline lands. If/when this assertion fails,
        // make sure VISION_RESEARCH_*.md has graduated to a phase plan and
        // the mmproj_url + load path are live.
        let vl = get("lfm2.5-vl-450m").expect("VL entry must exist");
        assert!(!vl.enabled, "VL entry must stay enabled=false until Phase F2 ships");
        assert!(vl.limits.supports_vision, "VL must declare supports_vision=true");
    }

    #[test]
    fn thinking_model_capability_set() {
        let th = get("lfm2.5-1.2b-thinking").expect("Thinking entry must exist");
        assert!(th.enabled);
        assert!(th.limits.supports_thinking);
        assert!(!th.limits.supports_vision);
    }

    #[test]
    fn enabled_filter_excludes_vl() {
        let enabled_ids: Vec<_> = dto_list_enabled().iter().map(|d| d.id.clone()).collect();
        assert!(!enabled_ids.iter().any(|id| id == "lfm2.5-vl-450m"));
        assert!(enabled_ids.iter().any(|id| id == "lfm2.5-350m"));
    }
}

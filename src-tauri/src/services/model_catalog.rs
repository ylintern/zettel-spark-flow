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
//!
//! Roadmap aliases (not yet shippable, listed for reference only):
//!   - `inspector` → vision (LFM 2.5 VL 450M) — needs mmproj companion shard;
//!     `plugin:leap-ai|download_model` 0.1.1 only handles a single URL.
//!   - `emb`       → embeddings (ModernBERT-base ONNX-ORT) — different runtime
//!     entirely (`ort` crate, not llama.cpp); will live in a separate
//!     `EmbeddingService` rather than this catalog.
//!
//! Platform notes:
//! - **Desktop** (current): llama.cpp in-process, GGUF from Hugging Face.
//! - **Mobile** (task M, deferred): LEAP vendor SDK resolves slugs internally.
//!   `MobileArtifact` placeholder is reserved for that phase.
//!
//! URLs verified 2026-04-25 against huggingface.co/LiquidAI.

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
/// `mmproj_url` is reserved for Phase 0.7-B (`inspector` / VL).
#[derive(Debug, Clone, Copy)]
pub struct ModelLimits {
    pub n_ctx_max: u32,
    pub default_n_ctx: u32,
    pub supports_vision: bool,
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
            quantization: e.desktop.quantization.to_string(),
        }
    }
}

/// Phase 0.7-A catalog: two GGUF models shippable through `tauri-plugin-leap-ai`.
///
/// Equivalent llama.cpp invocations (for reference / parity check):
///   llama-cli -hf LiquidAI/LFM2.5-350M-GGUF:Q4_K_M
///   llama-cli -hf LiquidAI/LFM2.5-1.2B-Instruct-GGUF:Q4_K_M
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
        desktop: DesktopArtifact {
            label: "LFM2.5-350M",
            quantization: "Q4_K_M",
            gguf_url: "https://huggingface.co/LiquidAI/LFM2.5-350M-GGUF/resolve/main/LFM2.5-350M-Q4_K_M.gguf",
        },
        limits: ModelLimits {
            n_ctx_max: 131_072,
            default_n_ctx: 32_768,
            supports_vision: false,
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
        desktop: DesktopArtifact {
            label: "LFM2.5-1.2B-Instruct",
            quantization: "Q4_K_M",
            gguf_url: "https://huggingface.co/LiquidAI/LFM2.5-1.2B-Instruct-GGUF/resolve/main/LFM2.5-1.2B-Instruct-Q4_K_M.gguf",
        },
        limits: ModelLimits {
            n_ctx_max: 131_072,
            default_n_ctx: 32_768,
            supports_vision: false,
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

pub fn dto_list() -> Vec<ModelEntryDto> {
    MODELS.iter().map(ModelEntryDto::from).collect()
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
}

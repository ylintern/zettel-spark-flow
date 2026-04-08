use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// File type of the model artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Gguf,
    Safetensors,
    Other,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Gguf => write!(f, "gguf"),
            FileType::Safetensors => write!(f, "safetensors"),
            FileType::Other => write!(f, "other"),
        }
    }
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "gguf" => FileType::Gguf,
            "safetensors" => FileType::Safetensors,
            _ => FileType::Other,
        }
    }
}

/// Model lifecycle state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelState {
    /// Found on disk, not yet verified
    Discovered,
    /// Verified and ready to be loaded
    Ready,
    /// User has selected this model (intent)
    Selected,
    /// Actively loaded in runtime (state)
    Active,
    /// Failed to load or initialize
    Failed(String),
}

/// Complete model metadata and manifest
/// 
/// This struct represents a model artifact with full lifecycle tracking.
/// It is designed to accommodate future expansion (downloader, versioning, integrity).
/// 
/// Fields marked `Option<_>` are intentionally deferred (not MVP priority).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    /// Unique identifier for this model (e.g. "phi-2", "mistral-7b")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Path on disk (absolute or relative to app-managed models dir)
    pub file_path: PathBuf,

    /// Type of model file (gguf, safetensors, etc.)
    pub file_type: FileType,

    /// Size in bytes (populated after discovery)
    pub size_bytes: u64,

    /// Cryptographic hash of the file (deferred to downloader work)
    pub hash: Option<String>,

    /// Algorithm used for hashing (e.g., "sha256")
    pub hash_algorithm: Option<String>,

    /// Current lifecycle state
    pub state: ModelState,

    /// When the model was first installed/discovered
    pub installed_at: DateTime<Utc>,

    /// When the model was last accessed (used for cache/cleanup policies)
    pub last_accessed: Option<DateTime<Utc>>,

    /// Semantic version of the model (deferred to upgrade/pinning work)
    pub version: Option<String>,
}

impl ModelManifest {
    /// Create a new manifest from discovery
    pub fn new(id: String, name: String, file_path: PathBuf, file_type: FileType, size_bytes: u64) -> Self {
        Self {
            id,
            name,
            file_path,
            file_type,
            size_bytes,
            hash: None,
            hash_algorithm: None,
            state: ModelState::Discovered,
            installed_at: Utc::now(),
            last_accessed: None,
            version: None,
        }
    }

    /// Transition state to Ready
    pub fn mark_ready(mut self) -> Self {
        self.state = ModelState::Ready;
        self
    }

    /// Transition state to Selected (user intent)
    pub fn select(&mut self) {
        if !matches!(self.state, ModelState::Failed(..)) {
            self.state = ModelState::Selected;
        }
    }

    /// Transition state to Active (runtime state)
    pub fn activate(&mut self) {
        if !matches!(self.state, ModelState::Failed(..)) {
            self.state = ModelState::Active;
            self.last_accessed = Some(Utc::now());
        }
    }

    /// Transition state to Failed
    pub fn fail(&mut self, reason: String) {
        self.state = ModelState::Failed(reason);
    }

    /// Check if model is available for use
    pub fn is_available(&self) -> bool {
        matches!(self.state, ModelState::Ready | ModelState::Selected | ModelState::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension("gguf"), FileType::Gguf);
        assert_eq!(FileType::from_extension("safetensors"), FileType::Safetensors);
        assert_eq!(FileType::from_extension("unknown"), FileType::Other);
    }

    #[test]
    fn test_manifest_state_transitions() {
        let mut manifest = ModelManifest::new(
            "test-model".to_string(),
            "Test".to_string(),
            PathBuf::from("/models/test.gguf"),
            FileType::Gguf,
            1024,
        );

        assert_eq!(manifest.state, ModelState::Discovered);
        manifest.select();
        assert_eq!(manifest.state, ModelState::Selected);
        manifest.activate();
        assert_eq!(manifest.state, ModelState::Active);
    }
}

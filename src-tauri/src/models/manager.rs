use super::manifest::{ModelManifest, ModelState};
use std::path::PathBuf;

/// Model Manager: Owns the lifecycle of all discovered/installed models
/// 
/// Responsibilities:
/// - Discover models from app-managed storage
/// - Track state (Discovered, Ready, Selected, Active, Failed)
/// - Distinguish between user *intent* (selection) and runtime *state* (activation)
/// 
/// NOTE: This struct will become shared app state (Arc<RwLock<ModelManager>>)
/// because multiple commands may query/mutate it concurrently.
/// Design anticipates Mutex/RwLock wrapping.
#[derive(Debug, Clone)]
pub struct ModelManager {
    /// Root directory where models are stored (app-owned)
    models_dir: PathBuf,

    /// List of discovered/installed models
    manifests: Vec<ModelManifest>,

    /// ID of currently selected model (user intent)
    /// This is independent of what's actually loaded in memory.
    selected_model_id: Option<String>,

    /// ID of currently active model (runtime state)
    /// This is what LEAP/inference engine is actually using.
    active_model_id: Option<String>,
}

impl ModelManager {
    /// Create a new manager for the given models directory
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            manifests: Vec::new(),
            selected_model_id: None,
            active_model_id: None,
        }
    }

    /// Scan the models directory and discover available models
    /// 
    /// This is a placeholder. Implementation will:
    /// 1. List files in models_dir
    /// 2. Filter for .gguf, .safetensors
    /// 3. Create ModelManifest for each
    /// 4. Mark as Discovered
    pub async fn discover_models(&mut self) -> Result<(), String> {
        // TODO: Implement filesystem scan
        // for file in models_dir {
        //     if file.ends_with(".gguf") || file.ends_with(".safetensors") {
        //         let manifest = ModelManifest::new(...);
        //         self.manifests.push(manifest);
        //     }
        // }
        Ok(())
    }

    /// Get list of all discovered/installed models
    pub fn get_installed_models(&self) -> Vec<ModelManifest> {
        self.manifests.clone()
    }

    /// Get a specific model by ID
    pub fn get_model(&self, id: &str) -> Option<ModelManifest> {
        self.manifests.iter().find(|m| m.id == id).cloned()
    }

    /// User selects a model (intent, not necessarily loaded)
    /// 
    /// This is distinct from activation. A user can select a model
    /// in the UI, and it will be loaded later when needed.
    pub fn select_model(&mut self, id: &str) -> Result<(), String> {
        if !self.manifests.iter().any(|m| m.id.as_str() == id) {
            return Err(format!("Model {} not found", id));
        }

        // TODO: Mark model as Selected in manifest
        self.selected_model_id = Some(id.to_string());
        Ok(())
    }

    /// Get the currently selected model (user intent)
    pub fn get_selected_model(&self) -> Option<String> {
        self.selected_model_id.clone()
    }

    /// Activate a model in the runtime
    /// 
    /// This is distinct from selection. Activation means the model
    /// is loaded into memory and ready for inference.
    /// 
    /// Precondition: Model must be Ready or better.
    /// 
    /// TODO: This will eventually call LEAP plugin to actually load the model.
    pub async fn activate_model(&mut self, id: &str) -> Result<(), String> {
        let manifest = self
            .manifests
            .iter_mut()
            .find(|m| m.id.as_str() == id)
            .ok_or_else(|| format!("Model {} not found", id))?;

        if !manifest.is_available() {
            return Err(format!("Model {} is not available (state: {:?})", id, manifest.state));
        }

        // TODO: Call LEAP plugin to load the model
        // leap::load_model(&manifest.file_path).await?;

        manifest.activate();
        self.active_model_id = Some(id.to_string());
        Ok(())
    }

    /// Get the currently active model (runtime state)
    pub fn get_active_model(&self) -> Option<String> {
        self.active_model_id.clone()
    }

    /// Deactivate the current model
    /// 
    /// Unloads the model from memory.
    /// 
    /// TODO: This will eventually call LEAP plugin to unload.
    pub async fn deactivate_model(&mut self) -> Result<(), String> {
        if let Some(id) = &self.active_model_id.clone() {
            if let Some(manifest) = self.manifests.iter_mut().find(|m| m.id.as_str() == id) {
                // TODO: Call LEAP plugin to unload the model
                // leap::unload_model().await?;

                manifest.state = ModelState::Selected; // Back to selected, not active
            }
        }

        self.active_model_id = None;
        Ok(())
    }

    /// Get the state of a specific model
    pub fn get_model_state(&self, id: &str) -> Option<ModelState> {
        self.manifests.iter().find(|m| m.id.as_str() == id).map(|m| m.state.clone())
    }

    /// Mark a model as failed
    pub fn mark_failed(&mut self, id: &str, reason: String) -> Result<(), String> {
        self.manifests
            .iter_mut()
            .find(|m| m.id.as_str() == id)
            .ok_or_else(|| format!("Model {} not found", id))?
            .fail(reason);
        Ok(())
    }

    /// Get all manifests (for debug/inspection)
    pub fn get_manifests(&self) -> &[ModelManifest] {
        &self.manifests
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::manifest::FileType;

    #[test]
    fn test_model_manager_creation() {
        let manager = ModelManager::new(PathBuf::from("/models"));
        assert!(manager.get_installed_models().is_empty());
        assert_eq!(manager.get_selected_model(), None);
        assert_eq!(manager.get_active_model(), None);
    }

    #[test]
    fn test_select_nonexistent_model() {
        let mut manager = ModelManager::new(PathBuf::from("/models"));
        let result = manager.select_model("nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_activate_model_not_available() {
        let mut manager = ModelManager::new(PathBuf::from("/models"));
        let manifest = ModelManifest::new(
            "test".to_string(),
            "Test".to_string(),
            PathBuf::from("/models/test.gguf"),
            FileType::Gguf,
            1024,
        );
        manager.manifests.push(manifest);

        // Should fail because model is only Discovered, not Ready
        let result = manager.activate_model("test").await;
        assert!(result.is_err());
    }
}

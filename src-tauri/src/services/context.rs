use serde::{Deserialize, Serialize};

/// Request for context generation
/// 
/// Sent from the frontend UI to Rust backend to request a ContextBundle.
/// Contains UI state hints that Rust can use to enrich the context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequest {
    /// ID of the currently open note (if any)
    pub active_note_id: Option<String>,

    /// Current UI view (e.g., "editor", "kanban", "graph")
    pub active_view: Option<String>,

    /// Chat session identifier (for conversation continuity)
    pub chat_session_id: Option<String>,

    /// The user's prompt/query
    pub user_prompt: String,

    /// Notes currently selected by the user
    pub selected_note_ids: Vec<String>,
}

/// A single note in the context bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNote {
    /// Note ID
    pub id: String,

    /// Note title
    pub title: String,

    /// Note content (decrypted if it was encrypted)
    pub content: String,

    /// Associated tags
    pub tags: Vec<String>,

    /// Whether the original note is encrypted
    pub is_encrypted: bool,
}

/// Complete context bundle for inference
/// 
/// This is what gets passed to LEAP/agent for decision-making.
/// Contains notes, metadata, and enriched state from the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBundle {
    /// Retrieved notes (ranked by relevance, if applicable)
    pub notes: Vec<ContextNote>,

    /// The primary/active note, if available
    pub active_note: Option<ContextNote>,

    /// Additional structured metadata (tasks, folder context, etc.)
    pub metadata: serde_json::Value,
}

impl ContextBundle {
    /// Create a minimal/empty bundle
    pub fn empty() -> Self {
        Self {
            notes: vec![],
            active_note: None,
            metadata: serde_json::json!({}),
        }
    }
}

/// Trait for context retrieval
/// 
/// Implemented by different strategies (SQL, Swiftide, etc.).
/// Must be async to support I/O and complex queries.
/// 
/// The separation of `RetrievalProvider` from the context request/response
/// allows for easy swapping of implementations:
/// - Phase 1 v1: `SqlContextProvider` (lightweight RAG)
/// - Phase 2: `SwiftideContextProvider` (semantic embeddings + re-ranking)
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait RetrievalProvider: Send + Sync {
    /// Retrieve a context bundle based on the request
    /// 
    /// Implementations may query databases, compute embeddings,
    /// rank results, etc. All async operations are permitted.
    async fn get_context(&self, request: ContextRequest) -> Result<ContextBundle, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_bundle_empty() {
        let bundle = ContextBundle::empty();
        assert!(bundle.notes.is_empty());
        assert!(bundle.active_note.is_none());
    }

    #[test]
    fn test_context_request_serializable() {
        let req = ContextRequest {
            active_note_id: Some("note-123".to_string()),
            active_view: Some("editor".to_string()),
            chat_session_id: None,
            user_prompt: "Tell me about X".to_string(),
            selected_note_ids: vec!["note-1".to_string()],
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ContextRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_prompt, "Tell me about X");
    }
}

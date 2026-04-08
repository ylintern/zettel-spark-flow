use super::context::{ContextBundle, ContextRequest, RetrievalProvider};
use async_trait::async_trait;

/// SQL-based context provider
///
/// Implements RetrievalProvider using SQLite queries.
/// Retrieves context from:
/// 1. Active note (metadata + encrypted content)
/// 2. Recently modified notes (limited window)
/// 3. Workspace snapshot (structure/hierarchy)
/// 4. Search results (if query provided)
///
/// This is a placeholder. Implementation will:
/// - Query SQLite for notes matching request filters
/// - Handle encryption/decryption of private notes
/// - Rank results by relevance (last accessed, frequency, query match)
/// - Assemble into ContextBundle with metadata
#[derive(Debug, Clone)]
pub struct SqlContextProvider {
    // TODO: Add database connection pool reference
    // db: SqlitePool,
}

impl SqlContextProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RetrievalProvider for SqlContextProvider {
    async fn get_context(&self, request: ContextRequest) -> Result<ContextBundle, String> {
        // TODO: Implementation steps:
        //
        // 1. If active_note_id is Some:
        //    - SELECT note_content, created_at, modified_at FROM notes WHERE id = active_note_id
        //    - Decrypt if note.is_private
        //    - Add to bundle.active_note
        //
        // 2. Fetch recently modified notes:
        //    - SELECT id, title, modified_at FROM notes
        //      WHERE workspace_id = ? AND modified_at > (NOW() - 7 days)
        //      ORDER BY modified_at DESC LIMIT 5
        //    - Decrypt if is_private
        //    - Add to bundle.notes
        //
        // 3. If user_prompt length > 3:
        //    - Placeholder for semantic search (Phase 2: Swiftide)
        //    - For now: simple substring match in titles
        //    - SELECT id, title, snippet FROM notes WHERE title LIKE ? LIMIT 10
        //    - Append to bundle.notes
        //
        // 4. Build metadata:
        //    - SELECT COUNT(*) as note_count, SUM(word_count) as total_words
        //      FROM notes WHERE workspace_id = ?
        //    - Add to bundle.metadata
        //
        // 5. For selected_note_ids:
        //    - SELECT id, title, content FROM notes WHERE id IN (...)
        //    - Decrypt as needed
        //    - Append to bundle.notes
        //
        // 6. Return Ok(ContextBundle) with filled fields

        let context = ContextBundle {
            active_note: None,
            notes: Vec::new(),
            metadata: serde_json::json!({}),
        };

        // Placeholder: Log the request
        eprintln!("SqlContextProvider: Received request: {:?}", request);

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_provider_creation() {
        let provider = SqlContextProvider::new();
        assert_eq!(format!("{:?}", provider), "SqlContextProvider { }");
    }

    #[tokio::test]
    async fn test_get_context_empty_request() {
        let provider = SqlContextProvider::new();
        let request = ContextRequest {
            active_note_id: None,
            active_view: None,
            chat_session_id: None,
            user_prompt: String::new(),
            selected_note_ids: Vec::new(),
        };

        let context = provider.get_context(request).await;
        assert!(context.is_ok());

        let bundle = context.unwrap();
        assert!(bundle.active_note.is_none());
        assert!(bundle.notes.is_empty());
    }

    #[tokio::test]
    async fn test_get_context_with_active_note() {
        let provider = SqlContextProvider::new();
        let request = ContextRequest {
            active_note_id: Some("note-1".to_string()),
            active_view: None,
            chat_session_id: None,
            user_prompt: None,
            selected_note_ids: None,
        };

        let context = provider.get_context(request).await;
        assert!(context.is_ok());

        // Once database is connected:
        // - Should query for note-1
        // - Should decrypt if necessary
        // - Should populate bundle.active_note
    }

    #[tokio::test]
    async fn test_get_context_with_search_prompt() {
        let provider = SqlContextProvider::new();
        let request = ContextRequest {
            active_note_id: None,
            active_view: None,
            chat_session_id: None,
            user_prompt: "architecture design".to_string(),
            selected_note_ids: Vec::new(),
        };

        let context = provider.get_context(request).await;
        assert!(context.is_ok());

        // Once database is connected:
        // - Should query for notes matching "architecture design"
        // - Should populate bundle.notes (limited window)
    }
}

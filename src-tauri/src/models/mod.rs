use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub column: String,
    pub position: i64,
    pub is_kanban: bool,
    pub created_at: String,
    pub updated_at: String,
    pub folder: Option<String>,
    pub is_encrypted: Option<bool>,
}

impl WorkspaceNote {
    pub fn kind(&self) -> &'static str {
        if self.is_kanban {
            "task"
        } else {
            "note"
        }
    }

    pub fn is_private(&self) -> bool {
        self.is_encrypted.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub notes: Vec<WorkspaceNote>,
    pub folders: Vec<String>,
}

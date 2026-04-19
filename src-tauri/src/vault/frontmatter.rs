//! Type-safe YAML frontmatter serialization for notes and tasks.
//!
//! Uses serde_yml for robust YAML handling with proper escaping of special
//! characters (colons, quotes, emojis, multiline text). All fields serialize
//! correctly for Obsidian cross-compatibility.

use serde::{Deserialize, Serialize};

/// YAML frontmatter for notes and tasks.
///
/// Serializes to clean YAML with proper escaping. Fields map directly to
/// the markdown frontmatter block (between `---` delimiters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFrontmatter {
    /// Unique identifier (UUID). Used as filename: `{id}.md`
    pub id: String,

    /// Note type: "note" | "task"
    #[serde(rename = "type")]
    pub note_type: String,

    /// Display title. Supports special characters (colons, emojis, etc.)
    pub title: String,

    /// For tasks: kanban column name (e.g., "To Do", "In Progress")
    /// For notes: empty or used for categorical grouping.
    ///
    /// Serialized as YAML `status:` for Obsidian cross-compatibility
    /// (matches the on-disk schema per Section 1 Decision Lock).
    pub status: String,

    /// For tasks: vertical position within column (0-indexed)
    pub position: i64,

    /// Whether the note body is encrypted. Phase 0: always false.
    /// Phase 2/3: true for private notes.
    pub is_encrypted: bool,

    /// ISO 8601 creation timestamp
    pub created: String,

    /// ISO 8601 last modification timestamp
    pub modified: String,

    /// Parent folder name (empty string if no folder)
    #[serde(skip_serializing_if = "is_empty_folder")]
    pub folder: Option<String>,

    /// Tags array for filtering and search
    pub tags: Vec<String>,
}

fn is_empty_folder(folder: &Option<String>) -> bool {
    folder.as_ref().map(|f| f.is_empty()).unwrap_or(true)
}

impl NoteFrontmatter {
    /// Serialize frontmatter to YAML string.
    ///
    /// # Example
    /// ```text
    /// ---
    /// id: 550e8400-e29b-41d4-a716-446655440000
    /// type: note
    /// title: "Let's go: test"
    /// column: ""
    /// position: 0
    /// is_encrypted: false
    /// created: 2026-04-16T12:00:00Z
    /// modified: 2026-04-16T12:00:00Z
    /// tags: []
    /// ---
    /// ```
    pub fn to_yaml_block(&self) -> anyhow::Result<String> {
        let yaml = serde_yml::to_string(&self)?;
        Ok(format!("---\n{yaml}---", yaml = yaml))
    }

    /// Deserialize frontmatter from YAML string (without delimiters).
    pub fn from_yaml_block(yaml: &str) -> anyhow::Result<Self> {
        Ok(serde_yml::from_str(yaml)?)
    }
}

/// Partial frontmatter for reconciliation scans — every field optional.
///
/// Used by `vault::reconcile` to inspect foreign .md files (Obsidian-created,
/// manually dropped, partially migrated) without erroring on missing fields.
/// Classification logic decides whether to adopt / relocate / insert based
/// on which fields are present.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialFrontmatter {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type", default)]
    pub note_type: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub is_encrypted: Option<bool>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub modified: Option<String>,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl PartialFrontmatter {
    /// Returns `None` if the YAML is undeserializable — caller can treat as
    /// "no frontmatter" and fall through to full adoption.
    pub fn try_parse(yaml: &str) -> Option<Self> {
        serde_yml::from_str::<Self>(yaml).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_note_with_special_title() {
        let fm = NoteFrontmatter {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            note_type: "note".to_string(),
            title: "Let's go: \"test\" — 🚀 / café".to_string(),
            status: "".to_string(),
            position: 0i64,
            is_encrypted: false,
            created: "2026-04-16T12:00:00Z".to_string(),
            modified: "2026-04-16T12:00:00Z".to_string(),
            folder: None,
            tags: vec!["test".to_string()],
        };

        let yaml = fm.to_yaml_block().unwrap();
        assert!(yaml.contains("title:"));
        assert!(yaml.contains("Let's go")); // Title properly escaped
        assert!(yaml.contains("🚀"));       // Emoji preserved
        assert!(yaml.contains("café"));     // Accents preserved
    }

    #[test]
    fn roundtrip_frontmatter() {
        let original = NoteFrontmatter {
            id: "test-id".to_string(),
            note_type: "task".to_string(),
            title: "Test task".to_string(),
            status: "In Progress".to_string(),
            position: 1i64,
            is_encrypted: false,
            created: "2026-04-16T10:00:00Z".to_string(),
            modified: "2026-04-16T11:00:00Z".to_string(),
            folder: Some("work".to_string()),
            tags: vec!["urgent".to_string(), "review".to_string()],
        };

        let yaml = serde_yml::to_string(&original).unwrap();
        let deserialized: NoteFrontmatter = serde_yml::from_str(&yaml).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.title, deserialized.title);
        assert_eq!(original.status, deserialized.status);
        assert_eq!(original.tags.len(), deserialized.tags.len());
    }

    #[test]
    fn empty_folder_not_serialized() {
        let fm = NoteFrontmatter {
            id: "test".to_string(),
            note_type: "note".to_string(),
            title: "No folder".to_string(),
            status: "".to_string(),
            position: 0i64,
            is_encrypted: false,
            created: "2026-04-16T12:00:00Z".to_string(),
            modified: "2026-04-16T12:00:00Z".to_string(),
            folder: None,
            tags: vec![],
        };

        let yaml = serde_yml::to_string(&fm).unwrap();
        // folder should not appear in YAML when None
        assert!(!yaml.contains("folder:"));
    }

    #[test]
    fn column_serializes_as_status_for_obsidian_compat() {
        // Per Fix 5 Decision Lock: on-disk YAML key is `status:` (Obsidian-native
        // convention), while the Rust struct field remains `column` in Step 5.1.
        let fm = NoteFrontmatter {
            id: "task-1".to_string(),
            note_type: "task".to_string(),
            title: "Kanban card".to_string(),
            status: "In Progress".to_string(),
            position: 2i64,
            is_encrypted: false,
            created: "2026-04-16T12:00:00Z".to_string(),
            modified: "2026-04-16T12:00:00Z".to_string(),
            folder: None,
            tags: vec![],
        };

        let yaml = serde_yml::to_string(&fm).unwrap();
        assert!(
            yaml.contains("status:"),
            "YAML should emit `status:` for Obsidian compat, got:\n{yaml}"
        );
        assert!(
            !yaml.contains("column:"),
            "YAML should NOT emit `column:` (legacy key), got:\n{yaml}"
        );
        assert!(yaml.contains("In Progress"), "status value preserved");

        // Deserialization must read `status:` back.
        let back: NoteFrontmatter = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(back.status, "In Progress");
    }

    #[test]
    fn tags_array_preserved() {
        let fm = NoteFrontmatter {
            id: "test".to_string(),
            note_type: "note".to_string(),
            title: "Tagged".to_string(),
            status: "".to_string(),
            position: 0i64,
            is_encrypted: false,
            created: "2026-04-16T12:00:00Z".to_string(),
            modified: "2026-04-16T12:00:00Z".to_string(),
            folder: None,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let yaml = serde_yml::to_string(&fm).unwrap();
        assert!(yaml.contains("tags:"));
        assert!(yaml.contains("- tag1"));
        assert!(yaml.contains("- tag2"));
    }
}

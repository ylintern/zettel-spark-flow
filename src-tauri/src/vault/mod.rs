use std::{fs, path::Path};

use anyhow::Context;
use tauri::AppHandle;

use crate::{
    config::FLAGS,
    events::{self, NoteIndexingProgressEvent},
    models::WorkspaceNote,
    security::SecurityState,
};

pub mod frontmatter;
pub mod reconcile;
use frontmatter::NoteFrontmatter;

/// Reserved folder names. Never allowed as user-created folders.
/// - Type defaults: `notes`, `tasks`
/// - Infrastructure (Phase 1+): `agents`, `skills`, `roles`, `providers`, `tools`, `mcp`, `plugin`
pub const RESERVED_FOLDER_NAMES: &[&str] = &[
    "notes", "tasks", "agents", "skills", "roles", "providers", "tools", "mcp", "plugin",
];

pub fn is_reserved_folder(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    RESERVED_FOLDER_NAMES.iter().any(|r| *r == n)
}

pub fn ensure_vault_dirs(vault_dir: &Path) -> anyhow::Result<()> {
    for name in RESERVED_FOLDER_NAMES {
        fs::create_dir_all(vault_dir.join(name)).with_context(|| {
            format!("failed to create reserved dir {}/{}", vault_dir.display(), name)
        })?;
    }
    Ok(())
}

pub fn ensure_folder_dir(vault_dir: &Path, folder: &str) -> anyhow::Result<()> {
    let trimmed = folder.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(vault_dir.join(trimmed))
        .with_context(|| format!("failed to create folder dir {}/{}", vault_dir.display(), trimmed))?;
    Ok(())
}

pub fn reset_vault_dir(vault_dir: &Path) -> anyhow::Result<()> {
    if vault_dir.exists() {
        fs::remove_dir_all(vault_dir)
            .with_context(|| format!("failed to clear vault dir {}", vault_dir.display()))?;
    }

    ensure_vault_dirs(vault_dir)
}

pub fn note_relative_path(note_id: &str) -> String {
    format!("notes/{note_id}.md")
}

pub fn task_relative_path(note_id: &str) -> String {
    format!("tasks/{note_id}.md")
}

pub fn write_note(
    vault_dir: &Path,
    note: &WorkspaceNote,
    security: &SecurityState,
) -> anyhow::Result<String> {
    ensure_vault_dirs(vault_dir)?;

    let default_subdir = if note.is_kanban { "tasks" } else { "notes" };
    let subdir = note
        .folder
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_subdir);
    ensure_folder_dir(vault_dir, subdir)?;

    let relative_path = format!("{subdir}/{}.md", note.id);
    let absolute_path = vault_dir.join(&relative_path);
    let markdown = render_markdown(note, security)?;

    fs::write(&absolute_path, markdown)
        .with_context(|| format!("failed to write note file {}", absolute_path.display()))?;

    Ok(relative_path)
}

pub fn read_note(
    vault_dir: &Path,
    relative_path: &str,
    is_encrypted: bool,
    security: &SecurityState,
) -> anyhow::Result<String> {
    let absolute_path = vault_dir.join(relative_path);
    let markdown = fs::read_to_string(&absolute_path)
        .with_context(|| format!("failed to read note file {}", absolute_path.display()))?;

    let body = extract_body(&markdown);
    if FLAGS.encryption_enabled && is_encrypted {
        return security
            .decrypt_note_content(&body)
            .map_err(|err| anyhow::anyhow!(err.to_string()));
    }

    Ok(body)
}

pub fn emit_indexing_progress(
    app: &AppHandle,
    note_id: Option<&str>,
    processed_notes: usize,
    total_notes: usize,
) -> anyhow::Result<()> {
    let progress = if total_notes == 0 {
        100.0
    } else {
        ((processed_notes as f32 / total_notes as f32) * 100.0).clamp(0.0, 100.0)
    };

    events::emit_note_indexing_progress(
        app,
        &NoteIndexingProgressEvent {
            note_id: note_id.map(ToOwned::to_owned),
            stage: if progress >= 100.0 {
                "complete".to_string()
            } else {
                "loading".to_string()
            },
            progress,
            processed_notes,
            total_notes,
        },
    )
    .map_err(|err| anyhow::anyhow!(err))
}

pub fn delete_note(vault_dir: &Path, relative_path: &str) -> anyhow::Result<()> {
    let absolute_path = vault_dir.join(relative_path);
    if absolute_path.exists() {
        fs::remove_file(&absolute_path)
            .with_context(|| format!("failed to delete note file {}", absolute_path.display()))?;
    }
    Ok(())
}

fn render_markdown(note: &WorkspaceNote, security: &SecurityState) -> anyhow::Result<String> {
    // Use Obsidian-native "type:" field instead of "kind:"
    let note_type = note.kind();
    let is_effectively_encrypted = FLAGS.encryption_enabled && note.is_private();

    // Build frontmatter using type-safe serde_yml serialization.
    // This ensures special characters (colons, quotes, emojis) are properly escaped.
    let frontmatter = NoteFrontmatter {
        id: note.id.clone(),
        note_type: note_type.to_string(),
        title: note.title.clone(),
        status: note.status.clone(),
        position: note.position,
        is_encrypted: is_effectively_encrypted,
        created: note.created_at.clone(),
        modified: note.updated_at.clone(),
        folder: note.folder.clone(),
        tags: note.tags.clone(),
    };

    let yaml_block = serde_yml::to_string(&frontmatter)?;
    let mut lines = vec![
        "---".to_string(),
        yaml_block.trim_end().to_string(),
        "---".to_string(),
        String::new(),
    ];

    // For tasks, prepend checkbox if not already present
    if note.is_kanban {
        let content = &note.content;
        // Check if content already starts with a checkbox
        let has_checkbox =
            content.trim_start().starts_with("- [") || content.trim_start().starts_with("- [x]");
        if !has_checkbox {
            // Prepend unchecked checkbox
            lines.push("- [ ]".to_string());
            if !content.is_empty() {
                lines.push(String::new());
            }
        }
        if is_effectively_encrypted {
            lines.push(
                security
                    .encrypt_note_content(content)
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?,
            );
        } else {
            lines.push(content.clone());
        }
    } else {
        if is_effectively_encrypted {
            lines.push(
                security
                    .encrypt_note_content(&note.content)
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?,
            );
        } else {
            lines.push(note.content.clone());
        }
    }

    Ok(lines.join("\n"))
}

fn extract_body(markdown: &str) -> String {
    if let Some(stripped) = markdown.strip_prefix("---\n") {
        if let Some(idx) = stripped.find("\n---\n") {
            return stripped[(idx + 5)..].trim_start_matches('\n').to_string();
        }
    }

    markdown.to_string()
}

/// Legacy YAML string escaper using Rust Debug format.
/// Kept for reference and rollback capability. Phase 0+ uses serde_yml instead.
#[allow(dead_code)]
fn yaml_string_legacy(value: &str) -> String {
    format!("{value:?}")
}

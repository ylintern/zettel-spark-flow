use std::{fs, path::Path};

use anyhow::Context;
use tauri::AppHandle;

use crate::{
    events::{self, NoteIndexingProgressEvent},
    models::WorkspaceNote,
    security::SecurityState,
};

pub fn ensure_vault_dirs(vault_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(vault_dir.join("notes"))
        .with_context(|| format!("failed to create notes dir {}", vault_dir.display()))?;
    fs::create_dir_all(vault_dir.join("tasks"))
        .with_context(|| format!("failed to create tasks dir {}", vault_dir.display()))?;
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

    let relative_path = if note.is_kanban {
        task_relative_path(&note.id)
    } else {
        note_relative_path(&note.id)
    };
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
    if is_encrypted {
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

    let mut lines = vec![
        "---".to_string(),
        format!("id: {}", yaml_string(&note.id)),
        format!("type: {}", yaml_string(note_type)),
        format!("title: {}", yaml_string(&note.title)),
        format!("column: {}", yaml_string(&note.column)),
        format!("position: {}", note.position),
        format!("is_encrypted: {}", note.is_private()),
        format!("created: {}", yaml_string(&note.created_at)),
        format!("modified: {}", yaml_string(&note.updated_at)),
    ];

    match &note.folder {
        Some(folder) if !folder.is_empty() => {
            lines.push(format!("folder: {}", yaml_string(folder)))
        }
        _ => lines.push("folder: null".to_string()),
    }

    lines.push("tags:".to_string());
    if note.tags.is_empty() {
        lines.push("  []".to_string());
    } else {
        for tag in &note.tags {
            lines.push(format!("  - {}", yaml_string(tag)));
        }
    }

    lines.push("---".to_string());
    lines.push(String::new());

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
        if note.is_private() {
            lines.push(
                security
                    .encrypt_note_content(content)
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?,
            );
        } else {
            lines.push(content.clone());
        }
    } else {
        if note.is_private() {
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

fn yaml_string(value: &str) -> String {
    format!("{value:?}")
}

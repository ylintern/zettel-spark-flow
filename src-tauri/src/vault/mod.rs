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
pub mod seeds;
use frontmatter::NoteFrontmatter;

/// Reserved folder names. Never allowed as user-created folders.
/// - Type defaults: `notes`, `tasks`
/// - Infrastructure (Phase 1+): `agents`, `skills`, `roles`, `providers`, `tools`, `mcp`, `plugin`
/// - Per-model config (Phase 0.7-B / Q1=B): `models` — user-editable .md per
///   shipped model (sampler, limits, modes). Bootstrap seeds from
///   `src-tauri/src/templates/myspace/models/*.md`; user owns after.
pub const RESERVED_FOLDER_NAMES: &[&str] = &[
    "notes", "tasks", "agents", "skills", "roles", "providers", "tools", "mcp", "plugin", "models",
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

/// Write user-template `.md` files into `myspace/` if absent (write-once).
///
/// Lifecycle (Q1 = B): seeded at first onboarding + after factory reset.
/// **User owns these files after the first write** — re-running this fn is
/// a no-op for any path that already exists, even if the user has edited
/// the body. To restore the original template, the user can delete the
/// file and relaunch (or factory-reset).
///
/// Drives off the static [`seeds::SEEDS`] array.
pub fn seed_user_templates(vault_dir: &Path) -> anyhow::Result<()> {
    for (rel, content) in seeds::SEEDS {
        let target = vault_dir.join(rel);
        if target.exists() {
            continue; // write-once — user owns this path now
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create parent for seed {}", rel))?;
        }
        fs::write(&target, content)
            .with_context(|| format!("failed to write seed {}", rel))?;
    }
    Ok(())
}

/// Mirror per-plugin reference docs from `src-tauri/src/plugins/active/*.md`
/// into `myspace/plugin/`, **overwriting** on every launch.
///
/// App-owned. User edits here are intentionally lost — these files describe
/// the compiled-in plugin set, not user preferences. (This is how the agent
/// can grep `myspace/plugin/<name>.md` and trust it reflects the real build.)
///
/// Drives off the static [`seeds::PLUGIN_DOCS`] array.
pub fn mirror_plugin_docs(vault_dir: &Path) -> anyhow::Result<()> {
    let plugin_dir = vault_dir.join("plugin");
    fs::create_dir_all(&plugin_dir)
        .with_context(|| format!("failed to ensure plugin dir {}", plugin_dir.display()))?;
    for (name, content) in seeds::PLUGIN_DOCS {
        let target = plugin_dir.join(name);
        fs::write(&target, content)
            .with_context(|| format!("failed to mirror plugin doc {}", name))?;
    }
    Ok(())
}

/// Walk every subdir in [`seeds::INDEXED_DIRS`] and (re)write its
/// `INDEX.md` from the current contents. Also writes the top-level
/// `myspace/INDEX.md` pointing at the sub-indexes.
///
/// App-owned. Regenerated every launch. The agent reads the top-level
/// INDEX (cheap) and drills down via the `index` / `tools-index` skills.
pub fn regenerate_indexes(vault_dir: &Path) -> anyhow::Result<()> {
    for (subdir, title) in seeds::INDEXED_DIRS {
        write_subdir_index(vault_dir, subdir, title)?;
    }
    write_toplevel_index(vault_dir)?;
    Ok(())
}

fn write_subdir_index(vault_dir: &Path, subdir: &str, title: &str) -> anyhow::Result<()> {
    let dir = vault_dir.join(subdir);
    if !dir.exists() {
        return Ok(()); // ensure_vault_dirs creates it next launch; nothing to index this turn
    }
    let mut entries: Vec<(String, String)> = Vec::new();
    if let Ok(read) = fs::read_dir(&dir) {
        for ent in read.flatten() {
            let p = ent.path();
            let fname = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if !fname.ends_with(".md") || fname == "INDEX.md" {
                continue;
            }
            let body = fs::read_to_string(&p).unwrap_or_default();
            let display = first_h1(&body)
                .unwrap_or_else(|| fname.trim_end_matches(".md").to_string());
            entries.push((fname, display));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::new();
    out.push_str(&format!("# {} — Index\n\n", title));
    out.push_str("> Auto-generated on app launch. Do not hand-edit; this file is regenerated every time the app starts.\n\n");
    if entries.is_empty() {
        out.push_str("_(empty — no entries yet.)_\n");
    } else {
        for (fname, display) in &entries {
            let stem = fname.trim_end_matches(".md");
            out.push_str(&format!("- [{}](./{}) — {}\n", stem, fname, display));
        }
    }
    fs::write(dir.join("INDEX.md"), out)
        .with_context(|| format!("failed to write {}/INDEX.md", subdir))?;
    Ok(())
}

fn write_toplevel_index(vault_dir: &Path) -> anyhow::Result<()> {
    let mut out = String::from("# Vibo myspace — Index\n\n");
    out.push_str("> Auto-generated on app launch. The agent reads this file at session start to discover what's available in this vault.\n\n");
    out.push_str("## Sub-indexes\n\n");
    for (subdir, title) in seeds::INDEXED_DIRS {
        out.push_str(&format!("- [{}](./{}/INDEX.md)\n", title, subdir));
    }
    out.push_str("\n## How the agent uses this\n\n");
    out.push_str("- The `index` skill (always equipped) reads `skills/INDEX.md` to list skills available to the active role.\n");
    out.push_str("- The `tools-index` skill (on-demand) reads `tools/INDEX.md`.\n");
    out.push_str("- Per-plugin reference docs live under `plugin/`. Per-model config under `models/`.\n");
    out.push_str("- See `Guidelines/source-of-truth/PHASE_0.7-B_PLAN_2026-04-28.md` for the full design.\n");
    fs::write(vault_dir.join("INDEX.md"), out)
        .context("failed to write top-level INDEX.md")?;
    Ok(())
}

/// Pull the first level-1 heading (`# Foo`) from a markdown body, skipping
/// any leading YAML frontmatter. Returns `None` if no H1 found.
fn first_h1(body: &str) -> Option<String> {
    let no_fm: &str = if let Some(rest) = body.strip_prefix("---\n") {
        // Find closing "\n---" of the frontmatter block; take everything after.
        rest.find("\n---")
            .map(|i| &rest[i + 4..])
            .unwrap_or(rest)
    } else {
        body
    };
    no_fm
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("# ") && !t.starts_with("## ")
        })
        .map(|l| l.trim_start().trim_start_matches('#').trim().to_string())
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

    ensure_vault_dirs(vault_dir)?;
    // Phase 0.7-B/C: re-seed the user-editable templates and refresh the
    // app-owned mirrors after a factory reset, so the next launch isn't
    // required to repopulate myspace/. The seeds + plugin doc + index calls
    // are best-effort: if any single one fails, the vault is still usable —
    // bootstrap on next launch will retry.
    if let Err(e) = seed_user_templates(vault_dir) {
        log::warn!("reset_vault_dir: seed_user_templates failed (non-fatal): {}", e);
    }
    if let Err(e) = mirror_plugin_docs(vault_dir) {
        log::warn!("reset_vault_dir: mirror_plugin_docs failed (non-fatal): {}", e);
    }
    if let Err(e) = regenerate_indexes(vault_dir) {
        log::warn!("reset_vault_dir: regenerate_indexes failed (non-fatal): {}", e);
    }
    Ok(())
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

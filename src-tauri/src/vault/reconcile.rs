//! Vault ↔ SQL reconciliation (Phase 0.6).
//!
//! On launch we scan `myspace/**/*.md` and make SQL match what is physically
//! on disk. Three outcomes per file (Known / Relocated / Adopted) plus a
//! sweep that deletes SQL rows whose files no longer exist. Filesystem is
//! the source of truth — SQL is a cache we rebuild from disk.
//!
//! This replaces the brittle "SQL drives hydration" path where one missing
//! file aborted the whole snapshot load.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    models::WorkspaceNote,
    security::SecurityState,
    vault::{self, frontmatter::PartialFrontmatter},
};

#[derive(Debug, Default)]
pub struct ReconcileStats {
    pub scanned: usize,
    pub known: usize,
    pub relocated: usize,
    pub adopted: usize,
    pub deleted: usize,
    pub errors: usize,
}

struct ScannedFile {
    absolute: PathBuf,
    relative: String,
    parent_folder: String,
    frontmatter: Option<PartialFrontmatter>,
    body: String,
    mtime_iso: String,
    filename_stem: String,
}

enum Outcome {
    Known(String),
    Relocated(String),
    Adopted(String),
}

pub async fn reconcile_vault(
    pool: &SqlitePool,
    vault_dir: &Path,
    security: &SecurityState,
) -> anyhow::Result<ReconcileStats> {
    let scanned = scan_vault(vault_dir);
    let mut stats = ReconcileStats::default();
    stats.scanned = scanned.len();

    let sql_rows = sqlx::query("SELECT id, file_path FROM notes")
        .fetch_all(pool)
        .await?;
    let sql_by_id: HashMap<String, String> = sql_rows
        .into_iter()
        .filter_map(|row| {
            let id: String = row.try_get("id").ok()?;
            let path: String = row.try_get("file_path").ok()?;
            Some((id, path))
        })
        .collect();

    let mut seen_ids: HashSet<String> = HashSet::new();

    for file in &scanned {
        match apply_one(pool, vault_dir, security, file, &sql_by_id).await {
            Ok(Outcome::Known(id)) => {
                seen_ids.insert(id);
                stats.known += 1;
            }
            Ok(Outcome::Relocated(id)) => {
                seen_ids.insert(id);
                stats.relocated += 1;
            }
            Ok(Outcome::Adopted(id)) => {
                seen_ids.insert(id);
                stats.adopted += 1;
            }
            Err(err) => {
                stats.errors += 1;
                log::warn!(
                    "[vibo] reconcile: skipping {}: {:#}",
                    file.relative,
                    err
                );
            }
        }
    }

    // Reconcile-delete: any SQL note whose id was never seen on disk is stale.
    for (id, path) in sql_by_id.iter() {
        if !seen_ids.contains(id) {
            log::info!(
                "[vibo] reconcile: deleting stale SQL row {} (file {} missing)",
                id,
                path
            );
            match delete_note_row(pool, id).await {
                Ok(()) => stats.deleted += 1,
                Err(err) => {
                    stats.errors += 1;
                    log::warn!(
                        "[vibo] reconcile: failed to delete stale row {}: {:#}",
                        id,
                        err
                    );
                }
            }
        }
    }

    log::info!(
        "[vibo] reconcile: scanned={} known={} relocated={} adopted={} deleted={} errors={}",
        stats.scanned,
        stats.known,
        stats.relocated,
        stats.adopted,
        stats.deleted,
        stats.errors
    );

    Ok(stats)
}

fn scan_vault(vault_dir: &Path) -> Vec<ScannedFile> {
    let mut out = Vec::new();
    if !vault_dir.exists() {
        return out;
    }

    for entry in WalkDir::new(vault_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let absolute = path.to_path_buf();
        let relative = absolute
            .strip_prefix(vault_dir)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let parent_folder = absolute
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let raw = match fs::read_to_string(&absolute) {
            Ok(s) => s,
            Err(err) => {
                log::warn!(
                    "[vibo] scan_vault: cannot read {}: {}",
                    absolute.display(),
                    err
                );
                continue;
            }
        };

        let (fm_yaml, body) = split_frontmatter(&raw);
        let frontmatter = fm_yaml.and_then(PartialFrontmatter::try_parse);

        let mtime_iso = fs::metadata(&absolute)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| DateTime::<Utc>::from(t).to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let filename_stem = absolute
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        out.push(ScannedFile {
            absolute,
            relative,
            parent_folder,
            frontmatter,
            body: body.to_string(),
            mtime_iso,
            filename_stem,
        });
    }

    out
}

fn split_frontmatter(markdown: &str) -> (Option<&str>, &str) {
    if let Some(stripped) = markdown.strip_prefix("---\n") {
        if let Some(idx) = stripped.find("\n---\n") {
            let yaml = &stripped[..idx];
            let body = stripped[idx + 5..].trim_start_matches('\n');
            return (Some(yaml), body);
        }
        if let Some(idx) = stripped.find("\n---") {
            let yaml = &stripped[..idx];
            let body = stripped[idx + 4..].trim_start_matches('\n');
            return (Some(yaml), body);
        }
    }
    (None, markdown)
}

async fn apply_one(
    pool: &SqlitePool,
    vault_dir: &Path,
    security: &SecurityState,
    file: &ScannedFile,
    sql_by_id: &HashMap<String, String>,
) -> anyhow::Result<Outcome> {
    let id_in_yaml = file
        .frontmatter
        .as_ref()
        .and_then(|fm| fm.id.clone())
        .filter(|s| !s.is_empty());

    if let Some(id) = id_in_yaml.as_ref() {
        if let Some(existing_path) = sql_by_id.get(id) {
            if existing_path == &file.relative {
                return Ok(Outcome::Known(id.clone()));
            }
            // Relocated: same id, different physical location.
            sqlx::query("UPDATE notes SET file_path = ? WHERE id = ?")
                .bind(&file.relative)
                .bind(id)
                .execute(pool)
                .await?;
            return Ok(Outcome::Relocated(id.clone()));
        }
        // Foreign file with an id not in SQL — if YAML is complete enough,
        // insert as-is; otherwise fall through to full adoption below.
        if let Some(fm) = file.frontmatter.as_ref() {
            if let Some(note) = build_note_from_full_yaml(fm, file) {
                ensure_folder_row(pool, note.folder.as_deref()).await?;
                save_note_row(pool, &note, &file.relative).await?;
                return Ok(Outcome::Adopted(note.id));
            }
        }
    }

    // Adoption: generate UUID, fill defaults, rewrite file with clean YAML,
    // then insert SQL row. Idempotent — next launch sees it as Known.
    let new_id = Uuid::new_v4().to_string();
    let parent_type_hint = file.parent_folder.as_str();
    let is_kanban = parent_type_hint == "tasks"
        || file
            .frontmatter
            .as_ref()
            .and_then(|fm| fm.note_type.as_deref())
            == Some("task");

    let title = extract_first_heading(&file.body)
        .or_else(|| {
            file.frontmatter
                .as_ref()
                .and_then(|fm| fm.title.clone())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| file.filename_stem.clone());

    let folder = if file.parent_folder.is_empty() {
        None
    } else {
        Some(file.parent_folder.clone())
    };

    let now_millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let fm = file.frontmatter.as_ref();
    let note = WorkspaceNote {
        id: new_id.clone(),
        title,
        content: file.body.clone(),
        tags: fm.map(|f| f.tags.clone()).unwrap_or_default(),
        status: fm
            .and_then(|f| f.status.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                if is_kanban {
                    "inbox".into()
                } else {
                    String::new()
                }
            }),
        position: fm.and_then(|f| f.position).unwrap_or(now_millis),
        is_kanban,
        created_at: fm
            .and_then(|f| f.created.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| file.mtime_iso.clone()),
        updated_at: fm
            .and_then(|f| f.modified.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| file.mtime_iso.clone()),
        folder,
        is_encrypted: Some(false),
    };

    ensure_folder_row(pool, note.folder.as_deref()).await?;

    // Rewrite at canonical `{folder}/{uuid}.md` with full YAML.
    let new_relative = vault::write_note(vault_dir, &note, security)?;

    // If the original filename differed (foreign name like "Obsidian test note.md"),
    // remove it so we don't end up with two copies.
    if new_relative != file.relative && file.absolute.exists() {
        if let Err(err) = fs::remove_file(&file.absolute) {
            log::warn!(
                "[vibo] reconcile: failed to remove original foreign file {}: {}",
                file.absolute.display(),
                err
            );
        }
    }

    save_note_row(pool, &note, &new_relative).await?;
    Ok(Outcome::Adopted(new_id))
}

fn build_note_from_full_yaml(
    fm: &PartialFrontmatter,
    file: &ScannedFile,
) -> Option<WorkspaceNote> {
    let id = fm.id.clone().filter(|s| !s.is_empty())?;
    let note_type = fm.note_type.clone().filter(|s| !s.is_empty())?;
    let title = fm.title.clone().filter(|s| !s.is_empty())?;
    let created = fm
        .created
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| file.mtime_iso.clone());
    let modified = fm
        .modified
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| file.mtime_iso.clone());
    let folder = fm
        .folder
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            if file.parent_folder.is_empty() {
                None
            } else {
                Some(file.parent_folder.clone())
            }
        });
    Some(WorkspaceNote {
        id,
        title,
        content: file.body.clone(),
        tags: fm.tags.clone(),
        status: fm.status.clone().unwrap_or_default(),
        position: fm.position.unwrap_or(0),
        is_kanban: note_type == "task",
        created_at: created,
        updated_at: modified,
        folder,
        is_encrypted: fm.is_encrypted,
    })
}

async fn ensure_folder_row(pool: &SqlitePool, folder: Option<&str>) -> anyhow::Result<()> {
    if let Some(name) = folder.filter(|s| !s.is_empty()) {
        if !vault::is_reserved_folder(name) {
            crate::db::create_folder(pool, name).await?;
        }
    }
    Ok(())
}

async fn save_note_row(
    pool: &SqlitePool,
    note: &WorkspaceNote,
    relative_path: &str,
) -> anyhow::Result<()> {
    let is_enc = note.is_encrypted.unwrap_or(false);
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO notes (id, title, file_path, folder, column_id, position, kind, created_at, updated_at, is_encrypted)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          title = excluded.title,
          file_path = excluded.file_path,
          folder = excluded.folder,
          column_id = excluded.column_id,
          position = excluded.position,
          kind = excluded.kind,
          created_at = excluded.created_at,
          updated_at = excluded.updated_at,
          is_encrypted = excluded.is_encrypted
        "#,
    )
    .bind(&note.id)
    .bind(&note.title)
    .bind(relative_path)
    .bind(note.folder.as_deref())
    .bind(&note.status)
    .bind(note.position)
    .bind(note.kind())
    .bind(&note.created_at)
    .bind(&note.updated_at)
    .bind(if is_enc { 1_i64 } else { 0_i64 })
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
        .bind(&note.id)
        .execute(&mut *tx)
        .await?;
    for tag in note.tags.iter().filter(|t| !t.trim().is_empty()) {
        sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?, ?)")
            .bind(&note.id)
            .bind(tag.trim())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn delete_note_row(pool: &SqlitePool, id: &str) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

fn extract_first_heading(body: &str) -> Option<String> {
    for line in body.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("# ") {
            let s = rest.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

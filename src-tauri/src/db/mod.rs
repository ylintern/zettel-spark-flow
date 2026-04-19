use std::{fs, path::Path};

use anyhow::Context;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Row, SqlitePool,
};
use tauri::AppHandle;

use crate::{
    config::FLAGS,
    models::{KanbanColumn, WorkspaceNote, WorkspaceSnapshot},
    security::SecurityState,
    vault,
};

pub async fn init_pool(db_path: &Path) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .with_context(|| format!("failed to open sqlite db {}", db_path.display()))?;

    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    migrate(&pool).await?;

    Ok(pool)
}

pub fn delete_database_files(db_path: &Path) -> anyhow::Result<()> {
    for suffix in ["", "-wal", "-shm"] {
        let path = if suffix.is_empty() {
            db_path.to_path_buf()
        } else {
            db_path.with_file_name(format!(
                "{}{suffix}",
                db_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("vibo.db")
            ))
        };

        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("failed to delete sqlite file {}", path.display()))?;
        }
    }

    Ok(())
}

pub async fn load_workspace_snapshot(
    pool: &SqlitePool,
    vault_dir: &Path,
    security: &SecurityState,
    app: Option<&AppHandle>,
) -> anyhow::Result<WorkspaceSnapshot> {
    // Phase 0.6: reconcile SQL to match filesystem before reading. Skips missing
    // files, adopts foreign .md files, deletes stale SQL rows. Non-fatal on error.
    if let Err(err) = vault::reconcile::reconcile_vault(pool, vault_dir, security).await {
        log::warn!("[vibo] reconcile_vault failed (continuing with existing SQL): {err:#}");
    }

    let note_rows = sqlx::query(
        r#"
        SELECT id, title, file_path, folder, column_id, position, kind, created_at, updated_at, is_encrypted
        FROM notes
        ORDER BY updated_at DESC, created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let total_notes = note_rows.len();
    let mut notes = Vec::with_capacity(total_notes);

    if let Some(app) = app {
        vault::emit_indexing_progress(app, None, 0, total_notes)?;
    }

    for (index, row) in note_rows.into_iter().enumerate() {
        // Per-note error tolerance (Phase 0.6 Layer 1): one corrupt / missing
        // file must never abort the whole snapshot. Log + skip and continue.
        let id: String = match row.try_get("id") {
            Ok(v) => v,
            Err(err) => {
                log::warn!("[vibo] snapshot: skipping row with unreadable id: {err}");
                continue;
            }
        };
        let relative_path: String = match row.try_get("file_path") {
            Ok(v) => v,
            Err(err) => {
                log::warn!("[vibo] snapshot: skipping note {id}: unreadable file_path: {err}");
                continue;
            }
        };
        let tags = match load_tags(pool, &id).await {
            Ok(v) => v,
            Err(err) => {
                log::warn!("[vibo] snapshot: skipping note {id}: failed to load tags: {err}");
                continue;
            }
        };
        let stored_is_encrypted = row.try_get::<i64, _>("is_encrypted").unwrap_or(0) != 0;
        let effective_is_encrypted = FLAGS.encryption_enabled && stored_is_encrypted;
        let content = match vault::read_note(vault_dir, &relative_path, stored_is_encrypted, security) {
            Ok(v) => v,
            Err(err) => {
                log::warn!(
                    "[vibo] snapshot: skipping note {id} ({relative_path}): {err:#}"
                );
                if let Some(app) = app {
                    let _ = vault::emit_indexing_progress(app, Some(&id), index + 1, total_notes);
                }
                continue;
            }
        };

        let title: String = row.try_get("title").unwrap_or_default();
        let status: String = row.try_get("column_id").unwrap_or_default();
        let position: i64 = row.try_get("position").unwrap_or(0);
        let kind: String = row.try_get("kind").unwrap_or_else(|_| "note".into());
        let created_at: String = row.try_get("created_at").unwrap_or_default();
        let updated_at: String = row.try_get("updated_at").unwrap_or_default();
        let folder: Option<String> = row.try_get("folder").ok();
        let event_note_id = id.clone();

        notes.push(WorkspaceNote {
            id,
            title,
            content,
            tags,
            status,
            position,
            is_kanban: kind == "task",
            created_at,
            updated_at,
            folder,
            is_encrypted: Some(effective_is_encrypted),
        });

        if let Some(app) = app {
            vault::emit_indexing_progress(app, Some(&event_note_id), index + 1, total_notes)?;
        }
    }

    let folder_rows = sqlx::query("SELECT name FROM folders ORDER BY name ASC")
        .fetch_all(pool)
        .await?;

    let folders = folder_rows
        .into_iter()
        .map(|row| row.try_get("name"))
        .collect::<Result<Vec<String>, _>>()?;

    // Phase 0.5 physical-first recovery: ensure every SQL-known folder has a
    // physical subdir under myspace/. Also ensure any folder referenced by a
    // note row exists on disk (covers drift / restored DB / pre-0.5 data).
    for folder in &folders {
        if let Err(err) = vault::ensure_folder_dir(vault_dir, folder) {
            log::warn!("snapshot: failed to ensure folder dir '{folder}': {err}");
        }
    }
    for note in &notes {
        if let Some(folder) = note.folder.as_deref().filter(|s| !s.is_empty()) {
            if let Err(err) = vault::ensure_folder_dir(vault_dir, folder) {
                log::warn!(
                    "snapshot: failed to ensure folder dir '{folder}' for note {}: {err}",
                    note.id
                );
            }
        }
    }

    let columns = load_columns(pool).await?;

    if total_notes == 0 {
        if let Some(app) = app {
            vault::emit_indexing_progress(app, None, 0, 0)?;
        }
    }

    Ok(WorkspaceSnapshot { notes, folders, columns })
}

pub async fn save_note(
    pool: &SqlitePool,
    vault_dir: &Path,
    note: &WorkspaceNote,
    security: &SecurityState,
) -> anyhow::Result<()> {
    // 0. Fetch the old file path BEFORE we update it (for move/folder-change cleanup).
    let old_file_path = sqlx::query("SELECT file_path FROM notes WHERE id = ?")
        .bind(&note.id)
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_get::<String, _>("file_path"))
        .transpose()?;

    // 1. FS write first — capture absolute path for compensating rollback.
    let relative_path = vault::write_note(vault_dir, note, security)?;
    let absolute_path = vault_dir.join(&relative_path);
    let effective_is_encrypted = FLAGS.encryption_enabled && note.is_private();

    // 2. Ensure folder row exists (idempotent; outside tx to avoid nested-pool issues).
    if let Some(folder) = note.folder.as_ref().filter(|folder| !folder.is_empty()) {
        create_folder(pool, folder).await?;
    }

    // 3. All SQL mutations for this note inside a single transaction.
    //    On any error → rollback SQL + delete the MD we just wrote (compensating).
    let tx_result = async {
        let mut tx = pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO notes (
              id, title, file_path, folder, column_id, position, kind, created_at, updated_at, is_encrypted
            )
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
        .bind(&relative_path)
        .bind(note.folder.as_deref())
        .bind(&note.status)
        .bind(note.position)
        .bind(note.kind())
        .bind(&note.created_at)
        .bind(&note.updated_at)
        .bind(if effective_is_encrypted { 1_i64 } else { 0_i64 })
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
            .bind(&note.id)
            .execute(&mut *tx)
            .await?;

        for tag in note.tags.iter().filter(|tag| !tag.trim().is_empty()) {
            sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?, ?)")
                .bind(&note.id)
                .bind(tag.trim())
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    if let Err(err) = tx_result {
        // Compensating FS cleanup — best effort. If this fails, log and surface
        // original error; orphan sweep (Phase 0.5) will reconcile later.
        if let Err(cleanup_err) = fs::remove_file(&absolute_path) {
            log::warn!(
                "[vibo] save_note SQL failed and FS cleanup also failed for {}: {} (original: {})",
                absolute_path.display(),
                cleanup_err,
                err
            );
        } else {
            log::warn!(
                "[vibo] save_note SQL failed; rolled back MD at {} (original: {})",
                absolute_path.display(),
                err
            );
        }
        return Err(err);
    }

    // Post-transaction cleanup: if the note moved folders, delete the old file.
    if let Some(old_path) = old_file_path {
        if old_path != relative_path {
            let old_absolute_path = vault_dir.join(&old_path);
            if let Err(err) = fs::remove_file(&old_absolute_path) {
                log::warn!(
                    "[vibo] save_note: failed to clean up old file {} after move: {}",
                    old_absolute_path.display(),
                    err
                );
            }
        }
    }

    Ok(())
}

pub async fn delete_note(pool: &SqlitePool, vault_dir: &Path, note_id: &str) -> anyhow::Result<()> {
    // Resolve file path before mutating — needed for post-commit FS delete.
    let file_path = sqlx::query("SELECT file_path FROM notes WHERE id = ?")
        .bind(note_id)
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_get::<String, _>("file_path"))
        .transpose()?;

    // SQL is the source of truth — delete rows atomically first.
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
        .bind(note_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    // FS delete happens after SQL commit. If it fails, log and proceed —
    // DB already reflects deletion; orphan sweep (Phase 0.5) reconciles.
    if let Some(path) = file_path {
        if let Err(err) = vault::delete_note(vault_dir, &path) {
            log::warn!(
                "[vibo] delete_note: SQL committed but FS delete failed for {} (orphan): {}",
                path,
                err
            );
        }
    }

    Ok(())
}

pub async fn create_folder(pool: &SqlitePool, folder_name: &str) -> anyhow::Result<()> {
    let trimmed = folder_name.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    // Reserved names (notes, tasks, agents, ...) live only on disk. Never seed them
    // into the folders table — any caller (including save_note) is protected here.
    if vault::is_reserved_folder(trimmed) {
        return Ok(());
    }

    sqlx::query("INSERT OR IGNORE INTO folders (name, created_at) VALUES (?, CURRENT_TIMESTAMP)")
        .bind(trimmed)
        .execute(pool)
        .await?;

    Ok(())
}

/// Phase 0.5: default folders (`notes`, `tasks`) are NOT seeded into the
/// `folders` SQL table. They are reserved names created physically on disk
/// by `vault::ensure_vault_dirs`. The `folders` table is reserved for
/// user-created folders only, so the UI can iterate it without filtering.
pub async fn init_default_folders(_pool: &SqlitePool) -> anyhow::Result<()> {
    Ok(())
}

async fn load_tags(pool: &SqlitePool, note_id: &str) -> anyhow::Result<Vec<String>> {
    let tag_rows = sqlx::query("SELECT tag FROM note_tags WHERE note_id = ? ORDER BY tag ASC")
        .bind(note_id)
        .fetch_all(pool)
        .await?;

    let tags = tag_rows
        .into_iter()
        .map(|row| row.try_get("tag"))
        .collect::<Result<Vec<String>, _>>()?;

    Ok(tags)
}

pub async fn load_columns(pool: &SqlitePool) -> anyhow::Result<Vec<KanbanColumn>> {
    let rows = sqlx::query("SELECT id, title, \"order\" FROM columns ORDER BY \"order\" ASC")
        .fetch_all(pool)
        .await?;

    let columns = rows
        .into_iter()
        .map(|row| -> anyhow::Result<KanbanColumn> {
            Ok(KanbanColumn {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                order: row.try_get("order")?,
            })
        })
        .collect::<Result<Vec<KanbanColumn>, _>>()?;

    Ok(columns)
}

pub async fn save_column(pool: &SqlitePool, column: &KanbanColumn) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO columns (id, title, "order", created_at)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          title = excluded.title,
          "order" = excluded."order"
        "#,
    )
    .bind(&column.id)
    .bind(&column.title)
    .bind(column.order)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_column(pool: &SqlitePool, id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM columns WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
          id TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          file_path TEXT NOT NULL UNIQUE,
          folder TEXT,
          column_id TEXT NOT NULL DEFAULT 'inbox',
          position INTEGER NOT NULL DEFAULT 0,
          kind TEXT NOT NULL CHECK (kind IN ('note', 'task')),
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          is_encrypted INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS note_tags (
          note_id TEXT NOT NULL,
          tag TEXT NOT NULL,
          PRIMARY KEY (note_id, tag),
          FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS folders (
          name TEXT PRIMARY KEY,
          created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Phase 0.5 cleanup: evict reserved names that may have leaked into folders
    // via older save_note paths. Idempotent — safe on every launch.
    for reserved in vault::RESERVED_FOLDER_NAMES {
        sqlx::query("DELETE FROM folders WHERE name = ?")
            .bind(reserved)
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS columns (
          id TEXT PRIMARY KEY,
          title TEXT NOT NULL,
          "order" INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::*;
    use crate::{models::WorkspaceNote, security::SecurityState, vault};

    #[test]
    fn cold_start_persists_markdown_and_sqlite_metadata() {
        tauri::async_runtime::block_on(async {
            let root = std::env::temp_dir().join(format!("vibo-cold-start-{}", Uuid::new_v4()));
            let database_dir = root.join("database");
            let db_path = root.join("vibo.db");
            let secure_vault_path = root.join("secure-vault.hold");

            fs::create_dir_all(&root).expect("failed to create temp root");

            let security = SecurityState::new(secure_vault_path);
            let pool = init_pool(&db_path).await.expect("failed to init db");

            let note = WorkspaceNote {
                id: "note-cold-start".to_string(),
                title: "Cold Start Note".to_string(),
                content: "Persistence must survive app restart.".to_string(),
                tags: vec!["zettel".to_string(), "phase-0".to_string()],
                status: "inbox".to_string(),
                position: 10,
                is_kanban: false,
                created_at: "2026-04-07T00:00:00.000Z".to_string(),
                updated_at: "2026-04-07T00:05:00.000Z".to_string(),
                folder: Some("Engineering".to_string()),
                is_encrypted: Some(false),
            };

            save_note(&pool, &database_dir, &note, &security)
                .await
                .expect("failed to save note");

            let markdown_path = database_dir.join(vault::note_relative_path(&note.id));
            let markdown = fs::read_to_string(&markdown_path).expect("failed to read markdown");
            assert!(markdown.contains("title: \"Cold Start Note\""));
            assert!(markdown.contains("Persistence must survive app restart."));

            pool.close().await;

            let reopened_pool = init_pool(&db_path).await.expect("failed to reopen db");
            let snapshot = load_workspace_snapshot(&reopened_pool, &database_dir, &security, None)
                .await
                .expect("failed to load snapshot after restart");

            assert_eq!(snapshot.notes.len(), 1);
            assert_eq!(snapshot.folders, vec!["Engineering".to_string()]);

            let restored = &snapshot.notes[0];
            assert_eq!(restored.id, note.id);
            assert_eq!(restored.title, note.title);
            assert_eq!(restored.content, note.content);
            assert_eq!(
                restored.tags,
                vec!["phase-0".to_string(), "zettel".to_string()]
            );
            assert_eq!(restored.folder, note.folder);
            assert_eq!(restored.updated_at, note.updated_at);

            reopened_pool.close().await;
            fs::remove_dir_all(&root).expect("failed to remove temp root");
        });
    }
}

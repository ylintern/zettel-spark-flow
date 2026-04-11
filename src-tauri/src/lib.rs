mod commands;
mod db;
mod events;
mod models;
mod providers;
mod security;
mod services;
mod state;
mod vault;

use std::fs;
use std::path::Path;

use tauri::Manager;

use crate::{
    security::{biometric::BiometricState, derive_vault_key, SecurityState},
    state::AppState,
};

/// One-time silent migration: vault/ → database/
/// Safe to call on every launch. Does nothing if already migrated or no vault/ present.
fn migrate_vault_to_database(app_data: &Path) {
    let old_root = app_data.join("vault");
    let new_root = app_data.join("database");

    if !old_root.exists() || new_root.exists() {
        return;
    }

    if let Err(e) = fs::create_dir_all(&new_root) {
        eprintln!("[vibo] migration: failed to create database/: {e}");
        return;
    }

    // vault/notes → database/notes
    let old_notes = old_root.join("notes");
    let new_notes = new_root.join("notes");
    if old_notes.exists() {
        if let Err(e) = fs::rename(&old_notes, &new_notes) {
            eprintln!("[vibo] migration: failed to move notes: {e}");
            return;
        }
        eprintln!("[vibo] migration: vault/notes → database/notes complete");
    }

    // vault/kanban → database/tasks
    let old_kanban = old_root.join("kanban");
    let new_tasks = new_root.join("tasks");
    if old_kanban.exists() {
        if let Err(e) = fs::rename(&old_kanban, &new_tasks) {
            eprintln!("[vibo] migration: failed to move kanban→tasks: {e}");
            return;
        }
        eprintln!("[vibo] migration: vault/kanban → database/tasks complete");
    }

    // Remove now-empty vault/ dir (best effort — ignore if not empty)
    let _ = fs::remove_dir(&old_root);
    eprintln!("[vibo] migration: vault/ → database/ complete");
}

/// Bootstrap app directory structure on every launch
/// Creates: base/, base/notes/, base/tasks/
/// Self-healing: missing directories are created silently
/// Error handling: logs failure but does not panic
fn bootstrap_app_directories(base: &Path) -> Result<(), String> {
    let dirs = [base.to_path_buf(), base.join("notes"), base.join("tasks")];
    for dir in &dirs {
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| format!("bootstrap failed {:?}: {}", dir, e))?;
            eprintln!("[vibo] created missing directory: {:?}", dir);
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Iniciamos o motor
    let mut builder = tauri::Builder::default();

    // 2. Plugins exclusivos de PC (Mac/Windows/Linux)
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_global_shortcut::Builder::new().build())
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec![]),
            ));
    }

    // 3. Plugins exclusivos de Telemóvel (iOS/Android)
    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_haptics::init());
    }

    // 4. Plugins Universais e arranque da App
    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_stronghold::Builder::new(|password| derive_vault_key(&password)).build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            // One-time migration: move data from old bundle path to new bundle path
            let old_data_dir = app
                .path()
                .app_local_data_dir()
                .map(|p| p.parent().unwrap_or(&p).join("com.vibo.zettel-spark-flow"))
                .ok();
            let new_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("failed to resolve app local data dir");

            if let Some(old) = old_data_dir {
                if old.exists() && !new_data_dir.exists() {
                    fs::rename(&old, &new_data_dir)
                        .unwrap_or_else(|e| eprintln!("[vibo] migration failed: {}", e));
                }
            }

            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("failed to resolve app local data dir");
            // One-time migration: vault/ → database/ (silent, idempotent)
            migrate_vault_to_database(&app_data_dir);

            let database_dir = app_data_dir.join("database");
            let db_path = app_data_dir.join("vibo.db");
            let secure_vault_path = app_data_dir.join("secure-vault.hold");

            // Bootstrap app directory structure on every launch
            if let Err(e) = bootstrap_app_directories(&database_dir) {
                eprintln!("[vibo] CRITICAL: directory bootstrap failed: {}", e);
            }

            let db = tauri::async_runtime::block_on(db::init_pool(&db_path))
                .expect("failed to initialize app database");

            app.manage(AppState::new(
                db,
                db_path,
                database_dir,
                SecurityState::new(secure_vault_path),
                BiometricState::new(),
            ));

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // A nossa lógica futura vai arrancar aqui!
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::load_workspace_snapshot,
            commands::workspace::save_note,
            commands::workspace::delete_note,
            commands::workspace::create_folder,
            commands::workspace::save_column,
            commands::workspace::delete_column,
            commands::workspace::export_notes,
            security::setup_secure_vault,
            security::unlock_vault,
            security::lock_vault,
            security::is_vault_unlocked,
            security::is_vault_configured,
            security::store_secret,
            security::get_secret,
            security::delete_secret,
            security::get_provider_status,
            security::factory_reset,
            security::reset_vault_and_secrets,
            security::reset_passphrase,
            security::biometric::is_biometric_available,
            security::biometric::get_device_capabilities,
            security::biometric::is_biometric_enabled,
            security::biometric::enable_biometric_unlock,
            security::biometric::disable_biometric_unlock,
            security::biometric::verify_biometric_and_unlock,
            security::biometric::fallback_passphrase_unlock,
            providers::stream_cloud_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

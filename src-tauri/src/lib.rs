mod commands;
mod db;
mod events;
mod models;
mod services;
mod security;
mod state;
mod vault;

use std::fs;
use std::path::Path;

use tauri::Manager;

use crate::{
    security::{biometric::BiometricState, derive_vault_key, SecurityState},
    state::AppState,
};

/// Bootstrap app directory structure on every launch
/// Creates: base/, base/notes/, base/kanban/
/// Self-healing: missing directories are created silently
/// Error handling: logs failure but does not panic
fn bootstrap_app_directories(base: &Path) -> Result<(), String> {
    let dirs = [
        base.to_path_buf(),
        base.join("notes"),
        base.join("kanban"),
    ];
    for dir in &dirs {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("bootstrap failed {:?}: {}", dir, e))?;
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
        .plugin(tauri_plugin_stronghold::Builder::new(|password| {
            derive_vault_key(&password)
        }).build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            // One-time migration: move data from old bundle path to new bundle path
            let old_data_dir = app.path().app_local_data_dir()
                .map(|p| p.parent().unwrap_or(&p).join("com.vibo.zettel-spark-flow"))
                .ok();
            let new_data_dir = app.path().app_local_data_dir()
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
            let vault_dir = app_data_dir.join("vault");
            let db_path = app_data_dir.join("vibo.db");
            let secure_vault_path = app_data_dir.join("secure-vault.hold");

            // Bootstrap app directory structure on every launch
            if let Err(e) = bootstrap_app_directories(&vault_dir) {
                eprintln!("[vibo] CRITICAL: directory bootstrap failed: {}", e);
            }

            let db = tauri::async_runtime::block_on(db::init_pool(&db_path))
                .expect("failed to initialize app database");

            app.manage(AppState::new(
                db,
                db_path,
                vault_dir,
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
            security::setup_secure_vault,
            security::unlock_vault,
            security::lock_vault,
            security::is_vault_unlocked,
            security::is_vault_configured,
            security::store_secret,
            security::get_secret,
            security::delete_secret,
            security::factory_reset,
            security::reset_vault_and_secrets,
            security::biometric::is_biometric_available,
            security::biometric::get_device_capabilities,
            security::biometric::is_biometric_enabled,
            security::biometric::enable_biometric_unlock,
            security::biometric::disable_biometric_unlock,
            security::biometric::verify_biometric_and_unlock,
            security::biometric::fallback_passphrase_unlock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod commands;
mod config;
mod db;
mod events;
mod models;
mod providers;
mod security;
mod services;
mod state;
mod vault;

use std::fs;

use tauri::{Manager, webview::PageLoadEvent};

use crate::{
    security::{biometric::BiometricState, derive_vault_key, SecurityState},
    services::inference::InferenceService,
    state::AppState,
};

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
            ))
            // Leap AI (llama.cpp in-process, desktop-embedded-llama feature).
            // Mobile registration deferred to task M (see Cargo.toml note).
            .plugin(tauri_plugin_leap_ai::init());
    }

    // 3. Plugins exclusivos de Telemóvel (iOS/Android)
    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_haptics::init());
    }

    // 4. Plugins Universais e arranque da App
    builder
        .on_page_load(|webview, payload| {
            // Reload-lock: on every WebView navigation/reload (⌘R), drop the
            // Stronghold session so the app boots into "configured + locked" →
            // LockScreen. The Rust process survives reloads. First load fires
            // too but lock() is a no-op when the session is already None.
            if matches!(payload.event(), PageLoadEvent::Started) {
                let handle = webview.app_handle().clone();
                let state = handle.state::<AppState>();
                if let Err(e) = state.security.lock() {
                    log::warn!("on_page_load: lock failed (non-fatal): {}", e);
                }
                let _ = security::emit_vault_status(
                    &handle,
                    &state.security,
                    "page-load-reload",
                );
            }
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_stronghold::Builder::new(|password| derive_vault_key(&password)).build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("failed to resolve app local data dir");
            // Phase 0.5 two-tree layout:
            //   viboai/myspace/   — user-visible vault (Obsidian-compatible)
            //   viboai/database/  — internal SQL + stronghold
            let viboai_dir = app_data_dir.join("viboai");
            let myspace_dir = viboai_dir.join("myspace");
            let database_dir = viboai_dir.join("database");
            let db_path = database_dir.join("vibo.db");
            let secure_vault_path = database_dir.join("secure-vault.hold");

            // Ensure both trees exist (idempotent via create_dir_all).
            if let Err(e) = fs::create_dir_all(&database_dir) {
                eprintln!("[vibo] CRITICAL: database dir bootstrap failed: {}", e);
            }
            if let Err(e) = vault::ensure_vault_dirs(&myspace_dir) {
                eprintln!("[vibo] CRITICAL: myspace bootstrap failed: {}", e);
            }

            // Phase 0.7-B/C: seed → mirror → index, in order.
            // - Seeds (roles/skills/models): write-once, user owns after first write.
            // - Plugin docs: mirror-overwrite from src-tauri/src/plugins/active/*.md.
            // - Indexes: regenerated every launch by walking the live tree.
            // All three are best-effort — failures are logged but don't block startup.
            if let Err(e) = vault::seed_user_templates(&myspace_dir) {
                eprintln!("[vibo] WARN: myspace seed failed (non-fatal): {}", e);
            }
            if let Err(e) = vault::mirror_plugin_docs(&myspace_dir) {
                eprintln!("[vibo] WARN: plugin doc mirror failed (non-fatal): {}", e);
            }
            if let Err(e) = vault::regenerate_indexes(&myspace_dir) {
                eprintln!("[vibo] WARN: index regeneration failed (non-fatal): {}", e);
            }

            // One-time migration: move legacy DB files (bundle root) into database/.
            for name in ["vibo.db", "vibo.db-shm", "vibo.db-wal", "secure-vault.hold"] {
                let legacy = app_data_dir.join(name);
                let target = database_dir.join(name);
                if legacy.exists() && !target.exists() {
                    if let Err(e) = fs::rename(&legacy, &target) {
                        eprintln!("[vibo] legacy {} migration failed: {}", name, e);
                    } else {
                        eprintln!("[vibo] migrated {} into database/", name);
                    }
                }
            }

            let db = tauri::async_runtime::block_on(db::init_pool(&db_path))
                .expect("failed to initialize app database");

            // Initialize default folders on first launch
            tauri::async_runtime::block_on(db::init_default_folders(&db))
                .unwrap_or_else(|e| eprintln!("[vibo] failed to initialize default folders: {}", e));

            app.manage(AppState::new(
                db,
                db_path,
                myspace_dir,
                SecurityState::new(secure_vault_path),
                BiometricState::new(),
            ));

            // InferenceService: attaches the leap-ai event listener and
            // loads persisted active model preference. Managed as its own
            // state rather than inside AppState so commands can take
            // State<'_, InferenceService> directly.
            app.manage(
                InferenceService::init(app.handle())
                    .expect("InferenceService init must succeed (LlamaBackend)"),
            );

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
            commands::onboarding::read_onboarding,
            commands::onboarding::write_onboarding,
            commands::onboarding::reset_onboarding,
            commands::onboarding::is_onboarding_complete,
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
            security::verify_vault_passphrase,
            security::biometric::is_biometric_available,
            security::biometric::get_device_capabilities,
            security::biometric::is_biometric_enabled,
            security::biometric::enable_biometric_unlock,
            security::biometric::disable_biometric_unlock,
            security::biometric::verify_biometric_and_unlock,
            security::biometric::fallback_passphrase_unlock,
            config::features::get_feature_flags,
            providers::stream_cloud_message,
            commands::inference::viboinference_list_models,
            commands::inference::viboinference_list_downloaded,
            commands::inference::viboinference_download_model,
            commands::inference::viboinference_delete_model,
            commands::inference::viboinference_get_active_model,
            commands::inference::viboinference_set_active_model,
            commands::inference::viboinference_start_chat_session,
            commands::inference::viboinference_stream_chat,
            commands::inference::viboinference_stop_generation,
            commands::inference::viboinference_end_chat_session,
            commands::inference::viboinference_session_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

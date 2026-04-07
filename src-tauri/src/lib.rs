#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Iniciamos o motor
    let mut builder = tauri::Builder::default();

    // 2. Plugins exclusivos de PC (Mac/Windows/Linux)
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_global_shortcut::Builder::new().build())
            .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])));
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
        // AQUI ESTÁ A CORREÇÃO (o .into() converte o texto em bytes para o cofre)
        .plugin(tauri_plugin_stronghold::Builder::new(|password| password.into()).build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
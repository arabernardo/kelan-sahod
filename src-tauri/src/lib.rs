// Pure modules (no Tauri, no UI): all payday, date and layout logic lives here.
mod holidays;
mod layout;
mod payday;

// The only module that touches the network (optional holiday sync).
mod remote;

// Thin Tauri command layer.
mod commands;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_payday_info,
            commands::refresh_holidays,
            commands::set_layout_mode
        ])
        .setup(|app| {
            // The window starts hidden (see tauri.conf.json). Size it for the saved layout
            // first, then show it, so a compact window never flashes at full size.
            // Any problem reading the settings simply means the Full layout.
            let mode = app
                .path()
                .app_data_dir()
                .ok()
                .and_then(|dir| std::fs::read_to_string(dir.join("settings.json")).ok())
                .map(|text| layout::layout_from_settings_json(&text))
                .unwrap_or_default();

            if let Some(window) = app.get_webview_window("main") {
                let _ = commands::apply_layout(&window, mode);
                let _ = window.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kelan Sahod");
}

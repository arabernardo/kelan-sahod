// Pure modules (no Tauri, no UI): all payday and date logic lives here.
mod holidays;
mod payday;

// Thin Tauri command layer.
mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![commands::get_payday_info])
        .run(tauri::generate_context!())
        .expect("error while running Kelan Sahod");
}

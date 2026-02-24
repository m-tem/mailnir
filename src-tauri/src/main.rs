#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::parse_template_cmd,
            commands::preview_csv,
            commands::get_smtp_profiles,
            commands::save_smtp_profiles,
            commands::store_smtp_credential,
            commands::delete_smtp_credential,
            commands::test_smtp_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

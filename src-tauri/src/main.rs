#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::SendState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_version_info,
            commands::parse_template_cmd,
            commands::preview_csv,
            commands::get_smtp_profiles,
            commands::save_smtp_profiles,
            commands::store_smtp_credential,
            commands::delete_smtp_credential,
            commands::test_smtp_connection,
            commands::get_data_fields,
            commands::get_form_fields,
            commands::save_template,
            commands::create_template,
            commands::preview_validate,
            commands::preview_render_entry,
            commands::send_batch,
            commands::cancel_send,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

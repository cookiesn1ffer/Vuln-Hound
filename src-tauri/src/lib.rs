pub mod agent;
mod commands;
pub mod findings;
pub mod llm;
pub mod playbook;
mod secrets;
pub mod security;
mod settings;
pub mod tools;

use agent::session::SessionRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(SessionRegistry::default())
        .invoke_handler(tauri::generate_handler![
            commands::fs_dialog::pick_target_directory,
            commands::playbook::get_playbook,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::get_provider_presets,
            commands::settings::save_api_key,
            commands::settings::get_api_key_status,
            commands::settings::delete_api_key,
            commands::settings::test_provider_connection,
            commands::scan::start_scan_session,
            commands::scan::cancel_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

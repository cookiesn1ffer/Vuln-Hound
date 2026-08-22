use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn pick_target_directory(app: tauri::AppHandle) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .blocking_pick_folder()
            .map(|p| p.to_string())
    })
    .await
    .unwrap_or(None)
}

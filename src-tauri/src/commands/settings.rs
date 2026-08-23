use crate::llm::client::{fetch_provider_models, OpenAiCompatClient};
use crate::llm::providers::{ProviderPreset, PROVIDER_PRESETS};
use crate::llm::types::ChatMessage;
use crate::secrets;
use crate::settings::{load_settings, save_settings, Settings};

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    load_settings(&app)
}

#[tauri::command]
pub fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn get_provider_presets() -> Vec<ProviderPreset> {
    PROVIDER_PRESETS.to_vec()
}

#[tauri::command]
pub fn save_api_key(provider_id: String, key: String) -> Result<(), String> {
    secrets::save_api_key(&provider_id, &key)
}

#[tauri::command]
pub fn get_api_key_status(provider_id: String) -> bool {
    secrets::has_api_key(&provider_id)
}

#[tauri::command]
pub fn delete_api_key(provider_id: String) -> Result<(), String> {
    secrets::delete_api_key(&provider_id)
}

/// Fetches the live model list for a provider directly from its `/models`
/// endpoint — never from a hardcoded list. `base_url` is passed explicitly
/// (rather than re-reading settings) so the UI can list models for whatever
/// base URL is currently in the form, including an unsaved edit to a custom
/// endpoint.
#[tauri::command]
pub async fn fetch_models(provider_id: String, base_url: String) -> Result<Vec<String>, String> {
    let api_key = secrets::get_api_key(&provider_id).ok();
    fetch_provider_models(&base_url, api_key.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_provider_connection(app: tauri::AppHandle) -> Result<String, String> {
    let settings = load_settings(&app)?;
    let api_key = secrets::get_api_key(&settings.provider_id).ok();

    let client = OpenAiCompatClient::new(settings.base_url, api_key, settings.model);
    let messages = vec![ChatMessage::user(
        "Reply with exactly the single word: OK",
    )];

    let response = client
        .chat_completion(messages, vec![])
        .await
        .map_err(|e| e.to_string())?;

    let reply = response
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();

    Ok(reply.trim().to_string())
}

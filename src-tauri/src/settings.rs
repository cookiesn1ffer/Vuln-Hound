use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;

use crate::llm::providers::find_preset;

const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "providerId")]
    pub provider_id: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub model: String,
}

impl Default for Settings {
    fn default() -> Self {
        // LM Studio is the default: works fully offline with no API key to set up.
        let preset = find_preset("lmstudio").expect("lmstudio preset must exist");
        Self {
            provider_id: preset.id.to_string(),
            base_url: preset.base_url.to_string(),
            model: preset.default_model.unwrap_or("local-model").to_string(),
        }
    }
}

pub fn load_settings(app: &tauri::AppHandle) -> Result<Settings, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("couldn't open settings store: {e}"))?;

    match store.get(SETTINGS_KEY) {
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|e| format!("couldn't parse saved settings: {e}")),
        None => Ok(Settings::default()),
    }
}

pub fn save_settings(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("couldn't open settings store: {e}"))?;
    let value = serde_json::to_value(settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, value);
    store
        .save()
        .map_err(|e| format!("couldn't persist settings: {e}"))
}

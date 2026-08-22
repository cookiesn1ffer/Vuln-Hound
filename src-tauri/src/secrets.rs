const SERVICE_NAME: &str = "vuln-hound";

fn entry(provider_id: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| format!("couldn't access the system keychain: {e}"))
}

/// Stores `key` for `provider_id` in the OS-native credential store (Keychain on
/// macOS, Credential Manager on Windows, Secret Service on Linux). The raw key
/// value never leaves this module — callers only ever see success/failure.
pub fn save_api_key(provider_id: &str, key: &str) -> Result<(), String> {
    entry(provider_id)?
        .set_password(key)
        .map_err(|e| format!("couldn't save the key: {e}"))
}

/// Reads the key back out. Only ever called from within the LLM client when
/// building a request — never returned across the Tauri IPC boundary to JS.
pub fn get_api_key(provider_id: &str) -> Result<String, String> {
    entry(provider_id)?
        .get_password()
        .map_err(|e| format!("couldn't read the key: {e}"))
}

pub fn has_api_key(provider_id: &str) -> bool {
    entry(provider_id)
        .and_then(|e| e.get_password().map_err(|e| e.to_string()))
        .is_ok()
}

pub fn delete_api_key(provider_id: &str) -> Result<(), String> {
    match entry(provider_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("couldn't delete the key: {e}")),
    }
}

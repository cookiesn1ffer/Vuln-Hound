use crate::playbook::{load_playbook, Playbook};

#[tauri::command]
pub fn get_playbook() -> Result<Playbook, String> {
    load_playbook().map_err(|e| format!("failed to parse bundled playbook: {e}"))
}

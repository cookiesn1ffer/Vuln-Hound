use std::path::PathBuf;

use tauri::{Emitter, State};

use crate::agent::events::SCAN_EVENT;
use crate::agent::loop_::run_scan_session;
use crate::agent::session::SessionRegistry;
use crate::llm::client::OpenAiCompatClient;
use crate::playbook::load_playbook;
use crate::secrets;
use crate::security::mode::ScanMode;
use crate::settings::load_settings;

#[tauri::command]
pub async fn start_scan_session(
    app: tauri::AppHandle,
    sessions: State<'_, SessionRegistry>,
    target_dir: String,
    mode: ScanMode,
    group_id: String,
) -> Result<String, String> {
    let playbook = load_playbook().map_err(|e| e.to_string())?;
    let group = playbook
        .group(&group_id)
        .ok_or_else(|| format!("unknown checklist group: {group_id}"))?
        .clone();

    let settings = load_settings(&app)?;
    let api_key = secrets::get_api_key(&settings.provider_id).ok();
    let client = OpenAiCompatClient::new(settings.base_url, api_key, settings.model);

    let session_id = uuid::Uuid::new_v4().to_string();
    let cancel = sessions.register(session_id.clone());

    let app_handle = app.clone();
    let sid = session_id.clone();
    tauri::async_runtime::spawn(async move {
        let emit = move |event| {
            let _ = app_handle.emit(SCAN_EVENT, event);
        };
        run_scan_session(
            emit,
            sid,
            PathBuf::from(target_dir),
            mode,
            group,
            client,
            cancel,
        )
        .await;
    });

    Ok(session_id)
}

#[tauri::command]
pub fn cancel_session(sessions: State<'_, SessionRegistry>, session_id: String) {
    sessions.cancel(&session_id);
    sessions.remove(&session_id);
}

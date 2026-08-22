use serde::Serialize;

use crate::findings::Finding;

pub const SCAN_EVENT: &str = "scan_event";

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ScanEvent {
    #[serde(rename = "status")]
    Status {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "groupId")]
        group_id: String,
        status: String,
    },
    #[serde(rename = "finding")]
    Finding {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "groupId")]
        group_id: String,
        finding: Finding,
    },
    #[serde(rename = "log")]
    Log {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "groupId")]
        group_id: String,
        message: String,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "groupId")]
        group_id: String,
        message: String,
    },
}

use serde::{Deserialize, Serialize};

use crate::playbook::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Pending,
    Scanning,
    Found,
    NotFound,
    Fixing,
    Fixed,
    FixFailed,
    Reverifying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingEvidence {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(default, rename = "lineStart", skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(default, rename = "lineEnd", skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub title: String,
    pub severity: Severity,
    pub status: FindingStatus,
    pub description: String,
    #[serde(default)]
    pub evidence: Vec<FindingEvidence>,
    #[serde(rename = "fixGuidance")]
    pub fix_guidance: String,
    #[serde(default, rename = "fixDiff", skip_serializing_if = "Option::is_none")]
    pub fix_diff: Option<String>,
    #[serde(
        default,
        rename = "fixSummary",
        skip_serializing_if = "Option::is_none"
    )]
    pub fix_summary: Option<String>,
    #[serde(
        default,
        rename = "reverifyResult",
        skip_serializing_if = "Option::is_none"
    )]
    pub reverify_result: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitFindingsArgs {
    #[serde(default)]
    pub findings: Vec<SubmittedFinding>,
}

/// Shape the model fills in via the `submit_findings` tool call — everything
/// except the fields Rust itself assigns (`id`, `status`, timestamps).
#[derive(Debug, Clone, Deserialize)]
pub struct SubmittedFinding {
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub title: String,
    pub severity: Severity,
    pub description: String,
    #[serde(default)]
    pub evidence: Vec<FindingEvidence>,
    #[serde(rename = "fixGuidance")]
    pub fix_guidance: String,
}

impl Finding {
    pub fn from_submitted(group_id: &str, submitted: SubmittedFinding) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            group_id: group_id.to_string(),
            category_id: submitted.category_id,
            title: submitted.title,
            severity: submitted.severity,
            status: FindingStatus::Found,
            description: submitted.description,
            evidence: submitted.evidence,
            fix_guidance: submitted.fix_guidance,
            fix_diff: None,
            fix_summary: None,
            reverify_result: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

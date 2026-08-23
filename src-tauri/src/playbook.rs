use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "baseSeverity")]
    pub base_severity: Severity,
    #[serde(rename = "whatItIs")]
    pub what_it_is: String,
    #[serde(rename = "howItHurtsTemplate")]
    pub how_it_hurts_template: String,
    #[serde(rename = "fixGuidanceTemplate")]
    pub fix_guidance_template: String,
    #[serde(rename = "requiresDynamicTest")]
    pub requires_dynamic_test: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookGroup {
    pub id: String,
    pub order: u32,
    pub title: String,
    pub description: String,
    pub entries: Vec<PlaybookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub version: u32,
    pub groups: Vec<PlaybookGroup>,
}

const PLAYBOOK_JSON: &str = include_str!("../resources/playbook.json");

pub fn load_playbook() -> Result<Playbook, serde_json::Error> {
    serde_json::from_str(PLAYBOOK_JSON)
}

impl Playbook {
    pub fn group(&self, group_id: &str) -> Option<&PlaybookGroup> {
        self.groups.iter().find(|g| g.id == group_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playbook_parses_and_is_nonempty() {
        let playbook = load_playbook().expect("bundled playbook.json must parse");
        assert!(!playbook.groups.is_empty());
        let total_entries: usize = playbook.groups.iter().map(|g| g.entries.len()).sum();
        assert!(
            total_entries >= 40,
            "expected a broad checklist, got {total_entries} entries"
        );
    }

    #[test]
    fn every_entry_has_unique_id() {
        let playbook = load_playbook().unwrap();
        let mut ids = std::collections::HashSet::new();
        for group in &playbook.groups {
            for entry in &group.entries {
                assert!(
                    ids.insert(entry.id.clone()),
                    "duplicate entry id: {}",
                    entry.id
                );
            }
        }
    }
}

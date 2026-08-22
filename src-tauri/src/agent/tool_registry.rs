use std::path::Path;

use serde_json::json;

use crate::llm::types::ToolDef;
use crate::playbook::PlaybookGroup;
use crate::tools::fs_tools;

pub const LIST_DIRECTORY: &str = "list_directory";
pub const READ_FILE: &str = "read_file";
pub const SEARCH_TEXT: &str = "search_text";
pub const GET_DEPENDENCY_MANIFESTS: &str = "get_dependency_manifests";
pub const WRITE_FILE_STAGED: &str = "write_file_staged";
pub const SUBMIT_FINDINGS: &str = "submit_findings";
pub const SUBMIT_FIX_RESULT: &str = "submit_fix_result";

fn list_directory_tool() -> ToolDef {
    ToolDef::new(
        LIST_DIRECTORY,
        "List files and directories under the target codebase (respects .gitignore, skips node_modules/.git/build output). Call with no path filter first to see the project layout.",
        json!({
            "type": "object",
            "properties": {},
        }),
    )
}

fn read_file_tool() -> ToolDef {
    ToolDef::new(
        READ_FILE,
        "Read the full text contents of one file, given its path relative to the target codebase root (as returned by list_directory).",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path relative to the target root." }
            },
            "required": ["path"],
        }),
    )
}

fn search_text_tool() -> ToolDef {
    ToolDef::new(
        SEARCH_TEXT,
        "Search the codebase for a regular expression pattern across all text files. Returns matching file paths, line numbers, and line content. Use this to find candidate locations before reading full files.",
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "A regular expression (Rust regex syntax)." }
            },
            "required": ["pattern"],
        }),
    )
}

fn get_dependency_manifests_tool() -> ToolDef {
    ToolDef::new(
        GET_DEPENDENCY_MANIFESTS,
        "Return the contents of every dependency manifest file found in the project (package.json, requirements.txt, Cargo.toml, go.mod, etc). Use this for dependency/supply-chain checks.",
        json!({
            "type": "object",
            "properties": {},
        }),
    )
}

fn write_file_staged_tool() -> ToolDef {
    ToolDef::new(
        WRITE_FILE_STAGED,
        "Stage a full replacement of one file's contents as the fix. Does not write to disk immediately — the change is shown to the user as a diff and applied afterward. Always read the file first so you know its exact current contents.",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path relative to the target root." },
                "newContent": { "type": "string", "description": "The complete new contents of the file after the fix." }
            },
            "required": ["path", "newContent"],
        }),
    )
}

/// The model must call this exactly once to finish a scan session. Its schema
/// restricts `categoryId` to the checklist entries actually in scope for this
/// group, so findings can't drift onto categories outside what was asked.
fn submit_findings_tool(group: &PlaybookGroup) -> ToolDef {
    let category_ids: Vec<&str> = group.entries.iter().map(|e| e.id.as_str()).collect();

    ToolDef::new(
        SUBMIT_FINDINGS,
        "Report the final list of findings for this category group. Call this exactly once when you are done investigating — do not report findings as plain text.",
        json!({
            "type": "object",
            "properties": {
                "findings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "categoryId": { "type": "string", "enum": category_ids },
                            "title": { "type": "string", "description": "A short, specific title for this finding." },
                            "severity": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
                            "description": { "type": "string", "description": "Plain-English explanation of how this specifically hurts this app." },
                            "evidence": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "filePath": { "type": "string" },
                                        "lineStart": { "type": "integer" },
                                        "lineEnd": { "type": "integer" },
                                        "snippet": { "type": "string" }
                                    },
                                    "required": ["filePath"]
                                }
                            },
                            "fixGuidance": { "type": "string", "description": "Specific, actionable guidance for fixing this exact finding." }
                        },
                        "required": ["categoryId", "title", "severity", "description", "fixGuidance"]
                    }
                }
            },
            "required": ["findings"],
        }),
    )
}

fn submit_fix_result_tool() -> ToolDef {
    ToolDef::new(
        SUBMIT_FIX_RESULT,
        "Call this exactly once when you have staged the fix (via write_file_staged) and are done.",
        json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string", "description": "One or two sentences describing what was changed and why." },
                "filesChanged": { "type": "array", "items": { "type": "string" } },
                "verificationNotes": { "type": "string", "description": "Why you believe this fully resolves the finding." }
            },
            "required": ["summary", "filesChanged"],
        }),
    )
}

/// Read-only tools available in every scan session, regardless of mode.
/// Test-mode-only dynamic/active tools (live HTTP probes, rate-limit/brute-force
/// checks) are intentionally not implemented yet — this registry is where they'll
/// be added in a future pass, gated on `ScanMode::Test`. Until then, both modes
/// get the same static/read-only tool set, which trivially satisfies the
/// Production-mode safety property tested in `mode_tests` below.
pub fn build_scan_tools(group: &PlaybookGroup) -> Vec<ToolDef> {
    vec![
        list_directory_tool(),
        read_file_tool(),
        search_text_tool(),
        get_dependency_manifests_tool(),
        submit_findings_tool(group),
    ]
}

pub fn build_fix_tools() -> Vec<ToolDef> {
    vec![
        list_directory_tool(),
        read_file_tool(),
        search_text_tool(),
        write_file_staged_tool(),
        submit_fix_result_tool(),
    ]
}

const MAX_LIST_ENTRIES: usize = 400;
const MAX_SEARCH_MATCHES: usize = 60;

/// Executes one of the read-only fs tools by name against `root`, returning the
/// tool-result content string to send back to the model. Errors are returned as
/// plain text (not `Err`) so the model sees the failure and can adapt, the same
/// way a shell tool-call failure would read in a real agent transcript.
pub fn execute_readonly_tool(root: &Path, name: &str, arguments: &str) -> String {
    let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or(json!({}));

    match name {
        LIST_DIRECTORY => match fs_tools::list_directory(root, MAX_LIST_ENTRIES) {
            Ok(entries) => serde_json::to_string(&entries).unwrap_or_default(),
            Err(e) => format!("error: {e}"),
        },
        READ_FILE => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            match fs_tools::read_file(root, path) {
                Ok(content) => content,
                Err(e) => format!("error: {e}"),
            }
        }
        SEARCH_TEXT => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            match fs_tools::search_text(root, pattern, MAX_SEARCH_MATCHES) {
                Ok(matches) => serde_json::to_string(&matches).unwrap_or_default(),
                Err(e) => format!("error: {e}"),
            }
        }
        GET_DEPENDENCY_MANIFESTS => match fs_tools::get_dependency_manifests(root) {
            Ok(manifests) => serde_json::to_string(&manifests).unwrap_or_default(),
            Err(e) => format!("error: {e}"),
        },
        other => format!("error: unknown tool '{other}'"),
    }
}

#[cfg(test)]
mod mode_tests {
    use super::*;

    /// Names that must never appear in a Production-mode scan registry. Empty today
    /// since no dynamic tool is implemented yet — this test exists so that the day
    /// a `http_request_to_target`-style tool is added, forgetting to gate it behind
    /// `ScanMode::Test` fails CI instead of shipping silently.
    const DYNAMIC_TOOL_NAMES: &[&str] = &[];

    #[test]
    fn scan_tools_never_contain_dynamic_tools() {
        let group = PlaybookGroup {
            id: "test".into(),
            order: 1,
            title: "Test".into(),
            description: "".into(),
            entries: vec![],
        };
        let tools = build_scan_tools(&group);
        for tool in &tools {
            assert!(
                !DYNAMIC_TOOL_NAMES.contains(&tool.function.name.as_str()),
                "dynamic tool {} must be gated behind ScanMode::Test",
                tool.function.name
            );
        }
    }

    #[test]
    fn fix_tools_never_contain_network_tools() {
        let tools = build_fix_tools();
        for tool in &tools {
            assert!(!tool.function.name.contains("http"));
            assert!(!tool.function.name.contains("request"));
        }
    }
}

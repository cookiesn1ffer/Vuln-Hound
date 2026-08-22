use std::path::PathBuf;

use tokio_util::sync::CancellationToken;

use crate::findings::{Finding, SubmitFindingsArgs};
use crate::llm::client::OpenAiCompatClient;
use crate::llm::types::ChatMessage;
use crate::playbook::PlaybookGroup;
use crate::security::mode::ScanMode;

use super::events::ScanEvent;
use super::prompt::build_scan_system_prompt;
use super::tool_registry::{self, build_scan_tools};

const MAX_ITERATIONS: u32 = 16;

/// Drives one scan session against a real (or test) LLM client. Emission is
/// decoupled from Tauri via a plain callback so this can run — and be tested —
/// with no `AppHandle` at all; the Tauri command wires `emit` up to a real
/// `app.emit(SCAN_EVENT, ..)` call, and tests can just push into a `Vec`.
pub async fn run_scan_session(
    emit: impl Fn(ScanEvent),
    session_id: String,
    target_dir: PathBuf,
    mode: ScanMode,
    group: PlaybookGroup,
    client: OpenAiCompatClient,
    cancel: CancellationToken,
) {
    let group_id = group.id.clone();

    emit(ScanEvent::Status {
        session_id: session_id.clone(),
        group_id: group_id.clone(),
        status: "scanning".into(),
    });

    let tools = build_scan_tools(&group);
    let mut messages = vec![
        ChatMessage::system(build_scan_system_prompt(&group, mode)),
        ChatMessage::user(format!(
            "Begin investigating the \"{}\" category group in this codebase.",
            group.title
        )),
    ];

    let mut submitted = false;
    let mut had_error = false;
    // Tracks the last read-only tool call (name, arguments) so an exact repeat
    // can be caught and redirected — local models sometimes get stuck retrying
    // the same search instead of exploring further.
    let mut last_call: Option<(String, String)> = None;

    for iteration in 0..MAX_ITERATIONS {
        if cancel.is_cancelled() {
            break;
        }

        let response = match client.chat_completion(messages.clone(), tools.clone()).await {
            Ok(r) => r,
            Err(e) => {
                emit(ScanEvent::Error {
                    session_id: session_id.clone(),
                    group_id: group_id.clone(),
                    message: e.to_string(),
                });
                had_error = true;
                break;
            }
        };

        let Some(choice) = response.choices.into_iter().next() else {
            break;
        };
        let assistant_msg = choice.message;
        messages.push(assistant_msg.clone());

        let tool_calls = assistant_msg.tool_calls.clone().unwrap_or_default();
        if tool_calls.is_empty() {
            if let Some(text) = assistant_msg.content.as_deref().filter(|t| !t.trim().is_empty()) {
                let preview: String = text.chars().take(240).collect();
                emit(ScanEvent::Log {
                    session_id: session_id.clone(),
                    group_id: group_id.clone(),
                    message: format!("(no tool call) {preview}"),
                });
            }
            if iteration + 1 >= MAX_ITERATIONS {
                break;
            }
            messages.push(ChatMessage::user(
                "Continue investigating by calling a tool, or call submit_findings when you're done.",
            ));
            continue;
        }

        for call in tool_calls {
            if cancel.is_cancelled() {
                break;
            }

            match call.function.name.as_str() {
                tool_registry::SUBMIT_FINDINGS => {
                    let args: SubmitFindingsArgs =
                        serde_json::from_str(&call.function.arguments).unwrap_or(SubmitFindingsArgs {
                            findings: vec![],
                        });
                    for submitted_finding in args.findings {
                        let finding = Finding::from_submitted(&group_id, submitted_finding);
                        emit(ScanEvent::Finding {
                            session_id: session_id.clone(),
                            group_id: group_id.clone(),
                            finding,
                        });
                    }
                    submitted = true;
                    messages.push(ChatMessage::tool_result(
                        call.id.clone(),
                        "Findings recorded.",
                    ));
                }
                other => {
                    emit(ScanEvent::Log {
                        session_id: session_id.clone(),
                        group_id: group_id.clone(),
                        message: format!("{other}({})", call.function.arguments),
                    });

                    let this_call = (other.to_string(), call.function.arguments.clone());
                    let is_repeat = last_call.as_ref() == Some(&this_call);
                    last_call = Some(this_call);

                    let mut result = tool_registry::execute_readonly_tool(
                        &target_dir,
                        other,
                        &call.function.arguments,
                    );
                    if is_repeat {
                        result.push_str(
                            "\n\n[Vuln-Hound note: you already called this exact tool with these exact arguments. \
                             Try a different search pattern, read a specific file, or call submit_findings if you're done.]",
                        );
                    }
                    messages.push(ChatMessage::tool_result(call.id.clone(), result));
                }
            }
        }

        if submitted || cancel.is_cancelled() {
            break;
        }
    }

    let final_status = if cancel.is_cancelled() {
        "idle"
    } else if had_error {
        "error"
    } else {
        // Even if the model never called submit_findings (e.g. hit the iteration
        // cap), treat the group as scanned rather than leaving it stuck spinning.
        "scanned"
    };

    emit(ScanEvent::Status {
        session_id,
        group_id,
        status: final_status.into(),
    });
}

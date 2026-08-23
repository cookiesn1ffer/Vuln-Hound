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

const MAX_ITERATIONS: u32 = 20;
/// How many of the most recent tool results are kept at full size; anything
/// older is collapsed to a placeholder. Without this, the full conversation
/// (every prior tool result) is re-sent on every request, so token cost grows
/// with each tool call — observed in practice: by call ~19 a single request
/// needed more tokens than Groq's entire free-tier per-minute budget (8000),
/// a request no amount of retrying can ever succeed at.
const KEEP_RECENT_TOOL_RESULTS: usize = 6;
const TRIMMED_PLACEHOLDER: &str =
    "[older result trimmed to save context — call this tool again if you need it]";
/// When this many iterations remain, stop suggesting further exploration and
/// force a conclusion instead — observed in practice: a thorough model can
/// keep investigating (or get slowed by rate-limit retries eating the ambient
/// clock) right up to the cap without ever calling submit_findings, silently
/// discarding everything it already confirmed.
const FORCE_CONCLUDE_WITHIN: u32 = 3;

/// Collapses all but the most recent `KEEP_RECENT_TOOL_RESULTS` tool-result
/// messages down to a short placeholder, bounding how much the conversation
/// grows per request regardless of how many tool calls the session makes.
fn trim_old_tool_results(messages: &mut [ChatMessage]) {
    let tool_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == "tool")
        .map(|(i, _)| i)
        .collect();

    if tool_indices.len() <= KEEP_RECENT_TOOL_RESULTS {
        return;
    }

    let cutoff = tool_indices.len() - KEEP_RECENT_TOOL_RESULTS;
    for &i in &tool_indices[..cutoff] {
        let already_trimmed = messages[i].content.as_deref() == Some(TRIMMED_PLACEHOLDER);
        if !already_trimmed {
            messages[i].content = Some(TRIMMED_PLACEHOLDER.to_string());
        }
    }
}

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

        trim_old_tool_results(&mut messages);

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

        let remaining = MAX_ITERATIONS.saturating_sub(iteration + 1);
        if remaining <= FORCE_CONCLUDE_WITHIN && remaining > 0 {
            messages.push(ChatMessage::user(format!(
                "You have {remaining} tool call(s) left in this session. Stop investigating further and call \
                 submit_findings now with everything you've confirmed so far — an empty array if genuinely nothing \
                 was found, but do not let confirmed findings go unreported."
            )));
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

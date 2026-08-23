use std::path::PathBuf;

use tokio_util::sync::CancellationToken;

use crate::findings::{Finding, SubmitFindingsArgs};
use crate::llm::client::OpenAiCompatClient;
use crate::llm::types::{ChatMessage, ToolCall, ToolCallFunction};
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

/// Recovers `(name, arguments_json)` from assistant text when a provider
/// wrote the intended tool call as plain JSON instead of using the
/// structured `tool_calls` field — e.g. `{"name": "search_text",
/// "parameters": {"pattern": "..."}}` somewhere in a longer message. Accepts
/// either `parameters` or `arguments` as the args key, and either an object
/// or an already-JSON-encoded string for its value. Returns `None` (falling
/// through to the normal "no tool call" handling) if nothing JSON-shaped
/// with a `name` field is found.
fn extract_fallback_tool_call(text: &str) -> Option<(String, String)> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end < start {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(&text[start..=end]).ok()?;

    let name = value.get("name").and_then(|v| v.as_str())?.to_string();
    let args_value = value.get("parameters").or_else(|| value.get("arguments"))?;
    let arguments = match args_value {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).ok()?,
    };
    Some((name, arguments))
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

        let response = match client
            .chat_completion(messages.clone(), tools.clone())
            .await
        {
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

        let mut tool_calls = assistant_msg.tool_calls.clone().unwrap_or_default();

        // Some providers never populate the structured tool_calls field at all
        // for certain models — observed live: Ollama with an otherwise-capable
        // model instead echoes the intended call as JSON text in `content`
        // (e.g. `{"name": "search_text", "parameters": {"pattern": "..."}}`),
        // every single iteration. Without recovering it, the session can never
        // make real progress: it silently burns the whole iteration budget on
        // the same dead-end nudge and ends as "scanned" with zero findings,
        // which looks in the UI exactly like a clean scan rather than a
        // provider-side tool-calling failure.
        if tool_calls.is_empty() {
            if let Some((name, arguments)) = assistant_msg
                .content
                .as_deref()
                .and_then(extract_fallback_tool_call)
            {
                emit(ScanEvent::Log {
                    session_id: session_id.clone(),
                    group_id: group_id.clone(),
                    message: format!(
                        "(recovered tool call from text — this provider/model didn't use structured tool calling) {name}({arguments})"
                    ),
                });
                tool_calls.push(ToolCall {
                    id: format!("fallback-{iteration}"),
                    kind: "function".into(),
                    function: ToolCallFunction { name, arguments },
                });
            }
        }

        if tool_calls.is_empty() {
            if let Some(text) = assistant_msg
                .content
                .as_deref()
                .filter(|t| !t.trim().is_empty())
            {
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
                    let args: SubmitFindingsArgs = serde_json::from_str(&call.function.arguments)
                        .unwrap_or(SubmitFindingsArgs { findings: vec![] });
                    for submitted_finding in args.findings {
                        let finding = Finding::from_submitted(&group_id, submitted_finding);
                        emit(ScanEvent::Finding {
                            session_id: session_id.clone(),
                            group_id: group_id.clone(),
                            finding: Box::new(finding),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_fallback_tool_call_parses_parameters_key() {
        // The exact text observed live from Ollama (huihui_ai/dolphin3-abliterated:8b),
        // which never populated the structured tool_calls field for this model.
        let text = "I will continue investigating by calling a tool. Here is the JSON for \
                     the function call with its proper arguments that best answers the given \
                     prompt:\n\n{\"name\": \"search_text\", \"parameters\": {\"pattern\": \"eval\\\\(.*\\\\)\"}}";
        let (name, arguments) = extract_fallback_tool_call(text).unwrap();
        assert_eq!(name, "search_text");
        let parsed: serde_json::Value = serde_json::from_str(&arguments).unwrap();
        assert_eq!(parsed["pattern"], "eval\\(.*\\)");
    }

    #[test]
    fn extract_fallback_tool_call_accepts_arguments_key() {
        let text = "{\"name\": \"read_file\", \"arguments\": {\"path\": \"server.js\"}}";
        let (name, arguments) = extract_fallback_tool_call(text).unwrap();
        assert_eq!(name, "read_file");
        assert_eq!(arguments, "{\"path\":\"server.js\"}");
    }

    #[test]
    fn extract_fallback_tool_call_returns_none_for_plain_prose() {
        let text = "I've reviewed the files and found no injection issues so far.";
        assert!(extract_fallback_tool_call(text).is_none());
    }

    #[test]
    fn extract_fallback_tool_call_returns_none_without_name_field() {
        let text = "{\"tool\": \"search_text\", \"parameters\": {\"pattern\": \"eval\"}}";
        assert!(extract_fallback_tool_call(text).is_none());
    }
}

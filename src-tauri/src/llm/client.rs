use std::time::Duration;

use thiserror::Error;

use super::types::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ToolDef};

/// Generous enough for slow local CPU inference on a long prompt, but bounded
/// so a stuck request surfaces as a clear error instead of hanging the session
/// indefinitely (observed in practice: an 8B local model over a slow backend
/// can occasionally hang far longer than any reasonable UI should wait).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

/// Caps how many tokens a single response may generate. Nothing in this
/// app's tool-calling flow ever legitimately needs more than a short
/// tool-call JSON blob plus a brief reasoning preamble — observed live: a
/// weak local model without this cap rambled to 2000+ tokens (at ~12 tok/s
/// on this hardware) without ever concluding, and got cut off mid-generation
/// by REQUEST_TIMEOUT instead. Bounding response length directly fixes the
/// actual problem instead of just widening the timeout to let it ramble
/// longer.
const MAX_RESPONSE_TOKENS: u32 = 1024;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("provider returned an error ({status}): {body}")]
    ProviderError { status: u16, body: String },
    #[error("provider returned no choices")]
    EmptyResponse,
    /// Distinct from a transient rate limit: a daily quota doesn't refill on
    /// any timescale a retry loop should ever wait for, so this is reported
    /// as an immediate hard failure instead of being retried.
    #[error("{0}")]
    DailyQuotaExceeded(String),
}

pub struct OpenAiCompatClient {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl OpenAiCompatClient {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        model: impl Into<String>,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            model: model.into(),
        }
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<ToolDef>,
    ) -> Result<ChatCompletionResponse, LlmError> {
        // Low temperature: agentic tool-use needs consistent, decisive tool calls
        // far more than it needs creative variation. Smaller local models in
        // particular get noticeably flakier about calling tools at all with the
        // provider's (often ~0.7-1.0) sampling default.
        self.chat_completion_with_temperature(messages, tools, 0.2)
            .await
    }

    pub async fn chat_completion_with_temperature(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<ToolDef>,
        temperature: f32,
    ) -> Result<ChatCompletionResponse, LlmError> {
        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            tools,
            max_tokens: Some(MAX_RESPONSE_TOKENS),
            temperature: Some(temperature),
        };

        // Free-tier rate limits (e.g. Groq's tokens-per-minute cap) are routine,
        // not exceptional — a single scan session can easily cross them once the
        // conversation accumulates a few tool results. Retry with backoff instead
        // of surfacing every 429 as a hard failure (observed in practice: this
        // happened on the very first real multi-tool-call scan against Groq's
        // free tier).
        const MAX_RETRIES: u32 = 3;
        // A provider's suggested wait (header or body) is often only valid for
        // the *next* token to free up, not for the full size of this specific
        // request — so a retry that hits 429 again on the same request needs to
        // wait longer than what was suggested last time, not the same amount.
        const BACKOFF_FLOORS: [Duration; 3] = [
            Duration::from_secs(1),
            Duration::from_secs(4),
            Duration::from_secs(10),
        ];
        let mut attempt = 0;

        loop {
            let mut req = self
                .http
                .post(format!("{}/chat/completions", self.base_url))
                .json(&body);
            if let Some(key) = &self.api_key {
                req = req.bearer_auth(key);
            }

            let resp = req.send().await?;
            let status = resp.status();

            if status.as_u16() == 429 {
                let header_wait = retry_after_header(&resp);
                let text = resp.text().await.unwrap_or_default();

                // Observed live: a daily-quota (TPD) 429 retried with the usual
                // short backoff just burns retries and an iteration hitting the
                // same 429 again — a TPD cap doesn't refill within any window a
                // retry loop should wait for, unlike a per-minute (TPM) cap.
                if is_daily_quota_exceeded(&text) {
                    return Err(LlmError::DailyQuotaExceeded(extract_error_message(&text)));
                }

                if attempt < MAX_RETRIES {
                    let suggested = header_wait
                        .or_else(|| retry_after_from_body(&text))
                        .unwrap_or(Duration::from_millis(2500));
                    let wait = suggested.max(BACKOFF_FLOORS[attempt as usize]);
                    attempt += 1;
                    tokio::time::sleep(wait).await;
                    continue;
                }

                return Err(LlmError::ProviderError {
                    status: 429,
                    body: text,
                });
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(LlmError::ProviderError {
                    status: status.as_u16(),
                    body,
                });
            }

            let parsed: ChatCompletionResponse = resp.json().await?;
            if parsed.choices.is_empty() {
                return Err(LlmError::EmptyResponse);
            }
            return Ok(parsed);
        }
    }
}

/// Reads a `Retry-After` header (seconds form) if the provider sent one, so we
/// wait exactly as long as asked rather than guessing.
fn retry_after_header(resp: &reqwest::Response) -> Option<Duration> {
    let value = resp.headers().get(reqwest::header::RETRY_AFTER)?;
    let seconds: u64 = value.to_str().ok()?.parse().ok()?;
    Some(Duration::from_secs(seconds.clamp(1, 30)))
}

/// Some providers (observed: Groq) only communicate the suggested wait inside
/// the JSON error body's message text — either plain seconds ("...try again
/// in 2.7225s...") or minutes+seconds ("...try again in 19m59.232s...") —
/// not a header. Best-effort scrape rather than a hard requirement — falls
/// back to `None` if the format doesn't match.
fn retry_after_from_body(body: &str) -> Option<Duration> {
    let idx = body.find("try again in")?;
    let rest = body[idx + "try again in".len()..].trim_start();

    // Capture the duration token (digits/'.'/'m', ending at the first 's') so
    // a trailing sentence period right after it (".") isn't swept in too.
    let mut end = 0;
    let mut seen_seconds_unit = false;
    for (i, c) in rest.char_indices() {
        if seen_seconds_unit {
            break;
        }
        if c.is_ascii_digit() || c == '.' || c == 'm' {
            end = i + c.len_utf8();
        } else if c == 's' {
            end = i + c.len_utf8();
            seen_seconds_unit = true;
        } else {
            break;
        }
    }
    let token = &rest[..end];

    let seconds: f64 = if let Some((min_part, sec_part)) = token.split_once('m') {
        let minutes: f64 = min_part.parse().ok()?;
        let sec_part = sec_part.strip_suffix('s')?;
        let secs: f64 = if sec_part.is_empty() {
            0.0
        } else {
            sec_part.parse().ok()?
        };
        minutes * 60.0 + secs
    } else {
        token.strip_suffix('s')?.parse().ok()?
    };

    Some(Duration::from_millis(
        ((seconds * 1000.0) as u64).clamp(250, 30_000),
    ))
}

/// Groq's tokens-per-day message includes "(TPD)"/"tokens per day"; distinct
/// from a tokens-per-minute (TPM) message, which is worth a short retry.
fn is_daily_quota_exceeded(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("tpd") || lower.contains("tokens per day")
}

/// Providers wrap their error text in `{"error":{"message": "..."}}`; surface
/// just that human-readable sentence when present instead of the raw JSON.
fn extract_error_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("error")?.get("message")?.as_str().map(str::to_string))
        .unwrap_or_else(|| body.to_string())
}

/// A listing endpoint should respond quickly or not at all — no reason to make
/// the UI wait as long as a slow local model's chat completion might take.
const MODELS_LIST_TIMEOUT: Duration = Duration::from_secs(15);

/// Fetches the live model list from any OpenAI-compatible provider's `GET
/// /models` endpoint. Never hardcode a model list in this app — provider
/// lineups drift (confirmed in practice: a hardcoded Groq model id went stale
/// within the same day it was set), so every provider's models are always
/// fetched fresh from the provider itself.
pub async fn fetch_provider_models(
    base_url: &str,
    api_key: Option<&str>,
) -> Result<Vec<String>, LlmError> {
    let http = reqwest::Client::builder()
        .timeout(MODELS_LIST_TIMEOUT)
        .build()
        .unwrap_or_default();

    let mut req = http.get(format!("{}/models", base_url.trim_end_matches('/')));
    if let Some(key) = api_key {
        req = req.bearer_auth(key);
    }

    let resp = req.send().await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(LlmError::ProviderError {
            status: status.as_u16(),
            body,
        });
    }

    let parsed: ModelsListResponse = resp.json().await?;
    let mut ids: Vec<String> = parsed.data.into_iter().map(|m| m.id).collect();
    ids.sort();
    Ok(ids)
}

#[derive(serde::Deserialize)]
struct ModelsListResponse {
    data: Vec<ModelListEntry>,
}

#[derive(serde::Deserialize)]
struct ModelListEntry {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_from_body_parses_plain_seconds() {
        let body = r#"{"error":{"message":"try again in 2.7225s.","type":"tokens"}}"#;
        assert_eq!(
            retry_after_from_body(body),
            Some(Duration::from_millis(2722))
        );
    }

    #[test]
    fn retry_after_from_body_parses_minutes_and_seconds() {
        // The exact format that triggered a live bug: this used to parse as
        // "19" seconds (truncated at the 'm') instead of ~1199 seconds.
        let body = r#"{"error":{"message":"Please try again in 19m59.232s. Need more tokens?"}}"#;
        let parsed = retry_after_from_body(body).unwrap();
        // Clamped to the 30s ceiling — real value (~1199s) is far above it,
        // but the parse itself must land in the right ballpark pre-clamp.
        assert_eq!(parsed, Duration::from_millis(30_000));
    }

    #[test]
    fn retry_after_from_body_missing_phrase_returns_none() {
        let body = r#"{"error":{"message":"rate limited"}}"#;
        assert_eq!(retry_after_from_body(body), None);
    }

    #[test]
    fn is_daily_quota_exceeded_detects_tpd() {
        let body =
            r#"{"error":{"message":"...on tokens per day (TPD): Limit 200000, Used 197747..."}}"#;
        assert!(is_daily_quota_exceeded(body));
    }

    #[test]
    fn is_daily_quota_exceeded_ignores_per_minute_limits() {
        let body = r#"{"error":{"message":"Rate limit reached on tokens per minute (TPM)"}}"#;
        assert!(!is_daily_quota_exceeded(body));
    }

    #[test]
    fn extract_error_message_pulls_message_field() {
        let body = r#"{"error":{"message":"quota exceeded","type":"tokens"}}"#;
        assert_eq!(extract_error_message(body), "quota exceeded");
    }

    #[test]
    fn extract_error_message_falls_back_to_raw_body() {
        let body = "not json";
        assert_eq!(extract_error_message(body), "not json");
    }
}

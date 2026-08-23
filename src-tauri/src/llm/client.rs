use std::time::Duration;

use thiserror::Error;

use super::types::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ToolDef};

/// Generous enough for slow local CPU inference on a long prompt, but bounded
/// so a stuck request surfaces as a clear error instead of hanging the session
/// indefinitely (observed in practice: an 8B local model over a slow backend
/// can occasionally hang far longer than any reasonable UI should wait).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("provider returned an error ({status}): {body}")]
    ProviderError { status: u16, body: String },
    #[error("provider returned no choices")]
    EmptyResponse,
}

pub struct OpenAiCompatClient {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl OpenAiCompatClient {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>, model: impl Into<String>) -> Self {
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
        self.chat_completion_with_temperature(messages, tools, 0.2).await
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
            max_tokens: None,
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
        const BACKOFF_FLOORS: [Duration; 3] =
            [Duration::from_secs(1), Duration::from_secs(4), Duration::from_secs(10)];
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

            if status.as_u16() == 429 && attempt < MAX_RETRIES {
                let header_wait = retry_after_header(&resp);
                let suggested = match header_wait {
                    Some(w) => w,
                    None => {
                        let text = resp.text().await.unwrap_or_default();
                        retry_after_from_body(&text).unwrap_or(Duration::from_millis(2500))
                    }
                };
                let wait = suggested.max(BACKOFF_FLOORS[attempt as usize]);
                attempt += 1;
                tokio::time::sleep(wait).await;
                continue;
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
/// the JSON error body's message text (e.g. "...try again in 2.7225s..."),
/// not a header. Best-effort scrape rather than a hard requirement — falls
/// back to a short fixed wait if the format doesn't match.
fn retry_after_from_body(body: &str) -> Option<Duration> {
    let idx = body.find("try again in")?;
    let rest = &body[idx + "try again in".len()..];
    let numeric: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let seconds: f64 = numeric.parse().ok()?;
    Some(Duration::from_millis(((seconds * 1000.0) as u64).clamp(250, 30_000)))
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

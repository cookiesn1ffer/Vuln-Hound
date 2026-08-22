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

        let mut req = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body);

        if let Some(key) = &self.api_key {
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

        let parsed: ChatCompletionResponse = resp.json().await?;
        if parsed.choices.is_empty() {
            return Err(LlmError::EmptyResponse);
        }
        Ok(parsed)
    }
}

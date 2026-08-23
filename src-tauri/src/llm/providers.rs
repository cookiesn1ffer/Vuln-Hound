use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub label: &'static str,
    /// Groups related presets under one heading in the UI (e.g. "OpenCode" for
    /// both OpenCode Go and OpenCode Zen). `None` renders as a standalone entry.
    pub family: Option<&'static str>,
    #[serde(rename = "baseUrl")]
    pub base_url: &'static str,
    #[serde(rename = "requiresKey")]
    pub requires_key: bool,
}

/// LLM backends Vuln-Hound ships presets for. All speak the same OpenAI-compatible
/// chat-completions + tool-calling schema, so one client implementation covers
/// every entry here — only base_url/key differ. Models are never hardcoded here:
/// the actual model list is fetched live from each provider's `/models` endpoint
/// (see `fetch_provider_models`), since provider lineups drift over time.
pub const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "lmstudio",
        label: "LM Studio (Local, Offline)",
        family: None,
        base_url: "http://localhost:1234/v1",
        requires_key: false,
    },
    ProviderPreset {
        id: "ollama",
        label: "Ollama (Local, Offline)",
        family: None,
        base_url: "http://localhost:11434/v1",
        requires_key: false,
    },
    ProviderPreset {
        id: "groq",
        label: "Groq (Free Cloud, Recommended)",
        family: None,
        base_url: "https://api.groq.com/openai/v1",
        requires_key: true,
    },
    ProviderPreset {
        id: "openrouter",
        label: "OpenRouter (Free Models)",
        family: None,
        base_url: "https://openrouter.ai/api/v1",
        requires_key: true,
    },
    ProviderPreset {
        id: "opencode-zen",
        label: "OpenCode Zen",
        family: Some("OpenCode"),
        base_url: "https://opencode.ai/zen/v1",
        requires_key: true,
    },
    ProviderPreset {
        id: "opencode-go",
        label: "OpenCode Go",
        family: Some("OpenCode"),
        base_url: "https://opencode.ai/zen/go/v1",
        requires_key: true,
    },
    ProviderPreset {
        id: "gemini",
        label: "Google Gemini (OpenAI-Compatible)",
        family: None,
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        requires_key: true,
    },
    ProviderPreset {
        id: "custom",
        label: "Custom OpenAI-Compatible Endpoint",
        family: None,
        base_url: "",
        requires_key: true,
    },
];

pub fn find_preset(id: &str) -> Option<&'static ProviderPreset> {
    PROVIDER_PRESETS.iter().find(|p| p.id == id)
}

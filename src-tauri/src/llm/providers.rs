use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub label: &'static str,
    #[serde(rename = "baseUrl")]
    pub base_url: &'static str,
    #[serde(rename = "requiresKey")]
    pub requires_key: bool,
    #[serde(rename = "defaultModel")]
    pub default_model: Option<&'static str>,
}

/// Free-to-use LLM backends Vuln-Hound ships presets for. All speak the same
/// OpenAI-compatible chat-completions + tool-calling schema, so one client
/// implementation covers every entry here — only base_url/key/model differ.
pub const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "lmstudio",
        label: "LM Studio (Local, Offline)",
        base_url: "http://localhost:1234/v1",
        requires_key: false,
        default_model: None,
    },
    ProviderPreset {
        id: "ollama",
        label: "Ollama (Local, Offline)",
        base_url: "http://localhost:11434/v1",
        requires_key: false,
        default_model: None,
    },
    ProviderPreset {
        id: "groq",
        label: "Groq (Free Cloud, Recommended)",
        base_url: "https://api.groq.com/openai/v1",
        requires_key: true,
        default_model: Some("openai/gpt-oss-120b"),
    },
    ProviderPreset {
        id: "openrouter",
        label: "OpenRouter (Free Models)",
        base_url: "https://openrouter.ai/api/v1",
        requires_key: true,
        default_model: Some("meta-llama/llama-3.3-70b-instruct:free"),
    },
    ProviderPreset {
        id: "gemini",
        label: "Google Gemini (OpenAI-Compatible)",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        requires_key: true,
        default_model: Some("gemini-2.0-flash"),
    },
    ProviderPreset {
        id: "custom",
        label: "Custom OpenAI-Compatible Endpoint",
        base_url: "",
        requires_key: true,
        default_model: None,
    },
];

pub fn find_preset(id: &str) -> Option<&'static ProviderPreset> {
    PROVIDER_PRESETS.iter().find(|p| p.id == id)
}

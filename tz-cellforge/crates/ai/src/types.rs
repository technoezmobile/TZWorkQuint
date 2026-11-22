//! Shared types for AI providers, connections, models, and chat streams.

use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The provider backend protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderType {
    /// Standard OpenAI-compatible API (`/v1/chat/completions`, `/v1/models`),
    /// used by local TZ AI Gateway, Ollama, vLLM, and OpenAI.
    #[default]
    OpenAiCompatible,
    /// Official Google Gemini API (`generativelanguage.googleapis.com/v1beta`).
    Gemini,
    /// Deterministic mock provider for automated tests and offline verification.
    Mock,
}

impl ProviderType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "TZ AI Gateway / OpenAI Compatible",
            Self::Gemini => "Google Gemini (Official API)",
            Self::Mock => "Deterministic Test Mock",
        }
    }
}

/// A stored AI connection configuration.
#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub enabled: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            id: "tz-gateway-local".to_string(),
            name: "TZ AI Gateway (Local)".to_string(),
            provider_type: ProviderType::OpenAiCompatible,
            base_url: "http://localhost:3001/v1".to_string(),
            api_key: None,
            model: String::new(),
            temperature: Some(0.2),
            max_tokens: Some(4096),
            enabled: true,
        }
    }
}

// Redact API key in debug and display to avoid accidental leakage in logs.
impl fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("provider_type", &self.provider_type)
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "••••••••"))
            .field("model", &self.model)
            .field("temperature", &self.temperature)
            .field("max_tokens", &self.max_tokens)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl ConnectionConfig {
    /// Returns a masked representation of the API key for UI display.
    pub fn masked_key(&self) -> String {
        match &self.api_key {
            None => "(none)".to_string(),
            Some(k) if k.is_empty() => "(empty)".to_string(),
            Some(k) if k.len() <= 8 => "••••••••".to_string(),
            Some(k) => format!("••••••••{}", &k[k.len() - 4..]),
        }
    }
}

/// Tool support status for a provider model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolSupport {
    Supported,
    Unsupported,
    Unknown,
}

impl ToolSupport {
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Supported => "Tools Supported",
            Self::Unsupported => "No Tool Support",
            Self::Unknown => "Tool Support Unknown",
        }
    }
}

/// Metadata about a discovered model from the provider.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tool_support: ToolSupport,
    pub supports_streaming: bool,
    #[serde(default)]
    pub modalities: Vec<String>,
    #[serde(default)]
    pub is_preview: bool,
    #[serde(default)]
    pub is_deprecated: bool,
}

impl ModelInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>, tool_support: ToolSupport) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            tool_support,
            supports_streaming: true,
            modalities: vec!["chat".to_string()],
            is_preview: false,
            is_deprecated: false,
        }
    }

    pub fn supports_tools(&self) -> bool {
        self.tool_support == ToolSupport::Supported
    }

    pub fn is_chat_capable(&self) -> bool {
        self.modalities.is_empty() || self.modalities.iter().any(|m| m == "chat" || m == "generateContent")
    }
}

/// Status and latency from a connection health check.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub models: Vec<ModelInfo>,
    pub message: String,
    pub error_code: Option<String>,
}

/// The role of a chat message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A structured tool definition provided to the LLM.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// A tool invocation request generated by the LLM.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// A single message in the conversation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// An incremental chunk yielded during streaming.
#[derive(Clone, Debug)]
pub enum ChatDelta {
    /// Text fragment from the assistant.
    Text(String),
    /// Incremental tool call fragment.
    ToolCallChunk {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        args_chunk: String,
    },
    /// Stream ended with finish reason.
    Done { finish_reason: Option<String> },
    /// Stream error.
    Error(String),
}

/// Shared cancellation handle.
pub type CancelHandle = Arc<AtomicBool>;

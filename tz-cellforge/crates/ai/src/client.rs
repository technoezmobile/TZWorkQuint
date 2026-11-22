//! LLM client trait, error definitions, and client instantiation.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use thiserror::Error;

use crate::security::SecurityError;
use crate::types::{ChatMessage, ChatDelta, ConnectionConfig, ConnectionTestResult, ModelInfo, ProviderType, ToolDefinition};

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Authentication failed: {0}. Please verify your API key.")]
    Authentication(String),

    #[error("Quota exceeded or rate limited (HTTP 429): {0}")]
    RateLimit(String),

    #[error("Model not found or not permitted: {0}")]
    ModelNotFound(String),

    #[error("Network connection error: {0}")]
    Connection(String),

    #[error("HTTP status {code}: {message}")]
    HttpStatus { code: u16, message: String },

    #[error("Protocol parsing error: {0}")]
    Protocol(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Security policy violation: {0}")]
    Security(#[from] SecurityError),
}

/// Abstract provider interface implemented by OpenAI-compatible, Gemini, and Mock clients.
pub trait LlmClient: Send + Sync {
    /// Performs a health check against the provider, measuring roundtrip latency.
    fn test_connection(&self) -> Result<ConnectionTestResult, AiError>;

    /// Queries the provider for available models.
    fn list_models(&self) -> Result<Vec<ModelInfo>, AiError>;

    /// Streams completion chunks, tool call requests, and completion signals.
    fn stream_chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
        cancel: &Arc<AtomicBool>,
        callback: &mut dyn FnMut(ChatDelta),
    ) -> Result<(), AiError>;

    /// Checks whether a specific model has established tool/function-calling capability.
    fn check_model_tool_support(&self, _model: &str) -> crate::types::ToolSupport {
        crate::types::ToolSupport::Unknown
    }
}

/// Creates an instantiated client for the given connection configuration.
pub fn create_client(config: &ConnectionConfig) -> Result<Box<dyn LlmClient>, AiError> {
    match config.provider_type {
        ProviderType::OpenAiCompatible => {
            let client = crate::openai::OpenAiClient::new(config)?;
            Ok(Box::new(client))
        }
        ProviderType::Gemini => {
            let client = crate::gemini::GeminiClient::new(config)?;
            Ok(Box::new(client))
        }
        ProviderType::Mock => {
            let client = crate::mock::MockClient::new(config);
            Ok(Box::new(client))
        }
    }
}

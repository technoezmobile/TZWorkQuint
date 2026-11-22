//! Deterministic mock LLM client for automated fixture tests, edge-case verification, and CI.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde_json::json;

use crate::client::{AiError, LlmClient};
use crate::types::{
    ChatDelta, ChatMessage, ConnectionConfig, ConnectionTestResult, MessageRole, ModelInfo,
    ToolDefinition, ToolSupport,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MockScenario {
    AddTotals,
    AuthFailure,
    RateLimit,
    InvalidRange,
    OutdatedRevision,
    SlowCancellation,
}

pub struct MockClient {
    pub scenario: MockScenario,
    pub model_id: String,
}

impl MockClient {
    pub fn new(config: &ConnectionConfig) -> Self {
        let scenario = match config.model.as_str() {
            "mock-auth-fail" => MockScenario::AuthFailure,
            "mock-rate-limit" => MockScenario::RateLimit,
            "mock-invalid-range" => MockScenario::InvalidRange,
            "mock-outdated-revision" => MockScenario::OutdatedRevision,
            "mock-cancellation" => MockScenario::SlowCancellation,
            _ => MockScenario::AddTotals,
        };

        Self {
            scenario,
            model_id: config.model.clone(),
        }
    }

    pub fn with_scenario(scenario: MockScenario) -> Self {
        Self {
            scenario,
            model_id: "mock-model".to_string(),
        }
    }
}

impl LlmClient for MockClient {
    fn test_connection(&self) -> Result<ConnectionTestResult, AiError> {
        match self.scenario {
            MockScenario::AuthFailure => Ok(ConnectionTestResult {
                success: false,
                latency_ms: 12,
                models: Vec::new(),
                message: "Authentication failed: Invalid mock API token (HTTP 401)".to_string(),
                error_code: Some("AUTH_FAILED".to_string()),
            }),
            MockScenario::RateLimit => Ok(ConnectionTestResult {
                success: false,
                latency_ms: 15,
                models: Vec::new(),
                message: "Rate limit / quota exceeded (HTTP 429)".to_string(),
                error_code: Some("RATE_LIMIT".to_string()),
            }),
            _ => Ok(ConnectionTestResult {
                success: true,
                latency_ms: 5,
                models: vec![
                    ModelInfo {
                        id: "mock-totals".to_string(),
                        name: "Mock Totals Assistant".to_string(),
                        description: Some("Deterministic mock that suggests table totals".to_string()),
                        tool_support: ToolSupport::Supported,
                        supports_streaming: true,
                        modalities: vec!["chat".to_string(), "tools".to_string()],
                        is_preview: false,
                        is_deprecated: false,
                    },
                    ModelInfo {
                        id: "mock-auth-fail".to_string(),
                        name: "Mock Auth Failure".to_string(),
                        description: Some("Simulates HTTP 401 response".to_string()),
                        tool_support: ToolSupport::Supported,
                        supports_streaming: true,
                        modalities: vec!["chat".to_string()],
                        is_preview: false,
                        is_deprecated: false,
                    },
                    ModelInfo {
                        id: "mock-rate-limit".to_string(),
                        name: "Mock Rate Limit".to_string(),
                        description: Some("Simulates HTTP 429 response".to_string()),
                        tool_support: ToolSupport::Supported,
                        supports_streaming: true,
                        modalities: vec!["chat".to_string()],
                        is_preview: false,
                        is_deprecated: false,
                    },
                ],
                message: "Mock provider connected (deterministic fixture mode)".to_string(),
                error_code: None,
            }),
        }
    }

    fn list_models(&self) -> Result<Vec<ModelInfo>, AiError> {
        let test = self.test_connection()?;
        if !test.success {
            return Err(AiError::Connection(test.message));
        }
        Ok(test.models)
    }

    fn stream_chat(
        &self,
        messages: &[ChatMessage],
        _tools: &[ToolDefinition],
        cancel: &Arc<AtomicBool>,
        callback: &mut dyn FnMut(ChatDelta),
    ) -> Result<(), AiError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(AiError::Cancelled);
        }

        match self.scenario {
            MockScenario::AuthFailure => {
                return Err(AiError::Authentication("Mock HTTP 401: Invalid API Key".to_string()));
            }
            MockScenario::RateLimit => {
                return Err(AiError::RateLimit("Mock HTTP 429: Daily request quota exceeded".to_string()));
            }
            MockScenario::SlowCancellation => {
                // Yield initial text, then simulate cancellation
                callback(ChatDelta::Text("Starting generation... ".to_string()));
                if cancel.load(Ordering::Relaxed) {
                    return Err(AiError::Cancelled);
                }
                return Err(AiError::Cancelled);
            }
            MockScenario::InvalidRange => {
                // Generates an invalid tool proposal that attempts an out-of-bounds write
                callback(ChatDelta::Text("Attempting calculation... ".to_string()));
                callback(ChatDelta::ToolCallChunk {
                    index: 0,
                    id: Some("call_invalid_range".to_string()),
                    name: Some("propose_edit".to_string()),
                    args_chunk: json!({
                        "sheet": "Sheet1",
                        "range": "INVALID_COLUMN_REF_9999999",
                        "values": [["=BAD()"]],
                        "explanation": "Out of bounds proposal"
                    }).to_string(),
                });
                callback(ChatDelta::Done { finish_reason: Some("tool_calls".to_string()) });
                return Ok(());
            }
            MockScenario::OutdatedRevision => {
                // Proposes edit bound to a stale revision 0
                callback(ChatDelta::Text("Here is the requested total:".to_string()));
                callback(ChatDelta::ToolCallChunk {
                    index: 0,
                    id: Some("call_stale".to_string()),
                    name: Some("propose_edit".to_string()),
                    args_chunk: json!({
                        "sheet": "Sheet1",
                        "range": "B6",
                        "values": [["=SUM(B1:B5)"]],
                        "explanation": "Stale revision total"
                    }).to_string(),
                });
                callback(ChatDelta::Done { finish_reason: Some("tool_calls".to_string()) });
                return Ok(());
            }
            MockScenario::AddTotals => {
                // Inspect conversation history to determine step
                let has_tool_response = messages.iter().any(|m| m.role == MessageRole::Tool);

                if !has_tool_response {
                    // Turn 1: Call `read_range` to inspect the table data
                    callback(ChatDelta::Text("I will inspect the selected table data to compute the totals.\n".to_string()));
                    callback(ChatDelta::ToolCallChunk {
                        index: 0,
                        id: Some("call_read_range_1".to_string()),
                        name: Some("read_range".to_string()),
                        args_chunk: json!({
                            "range": "A1:B5",
                            "formulas": true
                        }).to_string(),
                    });
                    callback(ChatDelta::Done { finish_reason: Some("tool_calls".to_string()) });
                } else {
                    // Turn 2: We received the cell values; now propose the formula edit at A6:B6!
                    callback(ChatDelta::Text(
                        "I analyzed the table data in A1:B5. I propose adding column totals at row 6 using SUM formulas.\n\n\
                         Please review the proposed formula changes below:".to_string()
                    ));
                    callback(ChatDelta::ToolCallChunk {
                        index: 0,
                        id: Some("call_propose_totals".to_string()),
                        name: Some("propose_edit".to_string()),
                        args_chunk: json!({
                            "sheet": "Sheet1",
                            "range": "A6:B6",
                            "values": [["=SUM(A1:A5)", "=SUM(B1:B5)"]],
                            "explanation": "Add column totals for columns A and B"
                        }).to_string(),
                    });
                    callback(ChatDelta::Done { finish_reason: Some("tool_calls".to_string()) });
                }
                return Ok(());
            }
        }
    }

    fn check_model_tool_support(&self, _model: &str) -> ToolSupport {
        ToolSupport::Supported
    }
}

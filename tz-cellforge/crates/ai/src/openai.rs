//! OpenAI-compatible client implementation (FreeLLMAPI, Ollama, vLLM, OpenAI).

use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::client::{AiError, LlmClient};
use crate::security::validate_transport_url;
use crate::types::{
    ChatDelta, ChatMessage, ConnectionConfig, ConnectionTestResult, MessageRole, ModelInfo,
    ToolDefinition, ToolSupport,
};

pub struct OpenAiClient {
    config: ConnectionConfig,
    base_url: String,
    agent: ureq::Agent,
}

impl OpenAiClient {
    pub fn new(config: &ConnectionConfig) -> Result<Self, AiError> {
        let validated_url = validate_transport_url(&config.base_url)?;

        let agent_config = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .timeout_global(Some(Duration::from_secs(300)))
            .max_redirects(0)
            .build();

        Ok(Self {
            config: config.clone(),
            base_url: validated_url,
            agent: ureq::Agent::new_with_config(agent_config),
        })
    }

    fn apply_auth<B>(&self, mut req: ureq::RequestBuilder<B>) -> ureq::RequestBuilder<B> {
        if let Some(key) = &self.config.api_key {
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                req = req.header("Authorization", &format!("Bearer {trimmed}"));
            }
        }
        req
    }

    fn map_ureq_error(e: ureq::Error) -> AiError {
        match e {
            ureq::Error::StatusCode(code) => match code {
                301 | 302 | 303 | 307 | 308 => AiError::Security(
                    crate::security::SecurityError::InsecureRedirect(
                        format!("HTTP {code} redirect blocked: automatic redirects are disabled to ensure credentials and workbook context stay strictly within configured transport and destination policy")
                    )
                ),
                401 | 403 => AiError::Authentication(format!("Provider returned HTTP {code}")),
                404 => AiError::ModelNotFound(format!("Endpoint or model returned HTTP 404")),
                429 => AiError::RateLimit(format!("Rate limit / quota exceeded (HTTP 429)")),
                other => AiError::HttpStatus {
                    code: other,
                    message: format!("HTTP error {other}"),
                },
            },
            ureq::Error::Timeout(_) => AiError::Connection("Request timed out".to_string()),
            other => AiError::Connection(other.to_string()),
        }
    }

    /// Evaluates tool/function calling support for an OpenAI-compatible model.
    pub fn eval_tool_support(id: &str) -> ToolSupport {
        let lower = id.to_ascii_lowercase();
        if lower.contains("embedding") || lower.contains("whisper") || lower.contains("dall-e") || lower.contains("tts") || lower.contains("moderation") {
            ToolSupport::Unsupported
        } else if lower.starts_with("gpt-4o")
            || lower.starts_with("gpt-4-turbo")
            || lower == "gpt-4"
            || lower.starts_with("gpt-4-0")
            || lower.starts_with("gpt-4-1")
            || lower == "gpt-3.5-turbo-0125"
            || lower == "gpt-3.5-turbo-1106"
            || lower.starts_with("claude-3-5")
            || lower.starts_with("claude-3-opus")
            || lower.starts_with("claude-3-sonnet")
            || lower.starts_with("gemini-2.5")
            || lower.starts_with("gemini-2.0")
            || lower.starts_with("gemini-1.5")
            || lower.starts_with("mistral-large")
            || lower.starts_with("command-r-plus")
            || lower.starts_with("qwen2.5-72b-instruct")
        {
            ToolSupport::Supported
        } else if lower.contains("instruct") || lower.contains("chat") || lower.contains("turbo") || lower.contains("gpt") || lower.contains("llama") || lower.contains("qwen") {
            ToolSupport::Unknown
        } else {
            ToolSupport::Unsupported
        }
    }
}

impl LlmClient for OpenAiClient {
    fn test_connection(&self) -> Result<ConnectionTestResult, AiError> {
        let start = Instant::now();
        let models_url = format!("{}/models", self.base_url);

        let req = self.agent.get(&models_url);
        let req = self.apply_auth(req);

        let mut resp = match req.call() {
            Ok(r) => r,
            Err(e) => {
                let err = Self::map_ureq_error(e);
                return Ok(ConnectionTestResult {
                    success: false,
                    latency_ms: start.elapsed().as_millis() as u64,
                    models: Vec::new(),
                    message: err.to_string(),
                    error_code: Some("CONNECT_ERROR".to_string()),
                });
            }
        };

        let latency = start.elapsed().as_millis() as u64;
        let text = match resp.body_mut().read_to_string() {
            Ok(s) => s,
            Err(e) => {
                return Ok(ConnectionTestResult {
                    success: false,
                    latency_ms: latency,
                    models: Vec::new(),
                    message: format!("Failed to read models response: {e}"),
                    error_code: Some("READ_ERROR".to_string()),
                });
            }
        };

        let body: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                return Ok(ConnectionTestResult {
                    success: false,
                    latency_ms: latency,
                    models: Vec::new(),
                    message: format!("Failed to parse models JSON: {e}"),
                    error_code: Some("PARSE_ERROR".to_string()),
                });
            }
        };

        let mut models = Vec::new();
        if let Some(arr) = body.get("data").and_then(Value::as_array) {
            for item in arr {
                if let Some(id) = item.get("id").and_then(Value::as_str) {
                    let tool_support = Self::eval_tool_support(id);
                    let lower = id.to_ascii_lowercase();
                    let is_unsupported_modality = lower.contains("embedding") || lower.contains("whisper") || lower.contains("dall-e") || lower.contains("tts");
                    let mut modalities = Vec::new();
                    if lower.contains("embedding") {
                        modalities.push("embedding".to_string());
                    } else if lower.contains("whisper") || lower.contains("tts") {
                        modalities.push("audio".to_string());
                    } else if lower.contains("dall-e") {
                        modalities.push("image".to_string());
                    } else {
                        modalities.push("chat".to_string());
                    }
                    if tool_support == ToolSupport::Supported {
                        modalities.push("tools".to_string());
                    }

                    let is_preview = lower.contains("preview") || lower.contains("exp") || lower.contains("beta");
                    let is_deprecated = lower.contains("deprecated");

                    models.push(ModelInfo {
                        id: id.to_string(),
                        name: id.to_string(),
                        description: None,
                        tool_support,
                        supports_streaming: !is_unsupported_modality,
                        modalities,
                        is_preview,
                        is_deprecated,
                    });
                }
            }
        }

        Ok(ConnectionTestResult {
            success: true,
            latency_ms: latency,
            models,
            message: format!("Connection healthy (roundtrip {}ms)", latency),
            error_code: None,
        })
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
        tools: &[ToolDefinition],
        cancel: &Arc<AtomicBool>,
        callback: &mut dyn FnMut(ChatDelta),
    ) -> Result<(), AiError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(AiError::Cancelled);
        }

        let chat_url = format!("{}/chat/completions", self.base_url);

        let mut messages_json = Vec::new();
        for m in messages {
            let role_str = match m.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            };
            let mut obj = json!({
                "role": role_str,
                "content": m.content,
            });
            if let Some(tool_calls) = &m.tool_calls {
                let tc_json: Vec<Value> = tool_calls
                    .iter()
                    .map(|tc| {
                        json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments.to_string(),
                            }
                        })
                    })
                    .collect();
                obj["tool_calls"] = json!(tc_json);
            }
            if let Some(call_id) = &m.tool_call_id {
                obj["tool_call_id"] = json!(call_id);
            }
            messages_json.push(obj);
        }

        let mut payload = json!({
            "model": self.config.model,
            "messages": messages_json,
            "stream": true,
        });

        if let Some(temp) = self.config.temperature {
            payload["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            payload["max_tokens"] = json!(max_tokens);
        }

        if !tools.is_empty() {
            let tools_json: Vec<Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            payload["tools"] = json!(tools_json);
            payload["tool_choice"] = json!("auto");
        }

        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| AiError::Protocol(format!("Failed to serialize chat payload: {e}")))?;

        let req = self.agent.post(&chat_url).header("Content-Type", "application/json");
        let req = self.apply_auth(req);

        let mut resp = req.send(&payload_bytes[..]).map_err(Self::map_ureq_error)?;

        // Stream the SSE body line by line
        let reader = resp.body_mut().as_reader();
        let buf_reader = BufReader::new(reader);

        for line_result in buf_reader.lines() {
            if cancel.load(Ordering::Relaxed) {
                return Err(AiError::Cancelled);
            }

            let line = line_result.map_err(|e| AiError::Connection(format!("Error reading stream: {e}")))?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with(':') {
                continue;
            }

            if let Some(data) = trimmed.strip_prefix("data:") {
                let payload = data.trim();
                if payload == "[DONE]" {
                    callback(ChatDelta::Done { finish_reason: None });
                    break;
                }

                let chunk_val: Value = match serde_json::from_str(payload) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(choice) = chunk_val.get("choices").and_then(|c| c.get(0)) {
                    // Check finish reason
                    let finish_reason = choice.get("finish_reason").and_then(Value::as_str).map(String::from);

                    // Check delta content
                    if let Some(delta) = choice.get("delta") {
                        if let Some(content) = delta.get("content").and_then(Value::as_str) {
                            if !content.is_empty() {
                                callback(ChatDelta::Text(content.to_string()));
                            }
                        }

                        // Check tool calls
                        if let Some(tcs) = delta.get("tool_calls").and_then(Value::as_array) {
                            for tc in tcs {
                                let idx = tc.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                                let id = tc.get("id").and_then(Value::as_str).map(String::from);
                                let name = tc
                                    .get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(Value::as_str)
                                    .map(String::from);
                                let args = tc
                                    .get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string();

                                callback(ChatDelta::ToolCallChunk {
                                    index: idx,
                                    id,
                                    name,
                                    args_chunk: args,
                                });
                            }
                        }
                    }

                    if let Some(reason) = finish_reason {
                        callback(ChatDelta::Done {
                            finish_reason: Some(reason),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn check_model_tool_support(&self, model: &str) -> ToolSupport {
        Self::eval_tool_support(model)
    }
}

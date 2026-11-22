//! Official Google Gemini API client implementation (v1beta).

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

pub struct GeminiClient {
    config: ConnectionConfig,
    base_url: String,
    agent: ureq::Agent,
}

impl GeminiClient {
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

    fn apply_auth<B>(&self, mut req: ureq::RequestBuilder<B>) -> Result<ureq::RequestBuilder<B>, AiError> {
        let key = self
            .config
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .ok_or_else(|| {
                AiError::Authentication("Gemini API requires an API key (e.g. AIzaSy...)".to_string())
            })?;

        // Use the official Google API key header
        req = req.header("x-goog-api-key", key);
        Ok(req)
    }

    fn map_ureq_error(e: ureq::Error) -> AiError {
        match e {
            ureq::Error::StatusCode(code) => match code {
                301 | 302 | 303 | 307 | 308 => AiError::Security(
                    crate::security::SecurityError::InsecureRedirect(
                        format!("HTTP {code} redirect blocked: automatic redirects are disabled to ensure credentials and workbook context stay strictly within configured transport and destination policy")
                    )
                ),
                400 => AiError::Protocol("Bad request to Gemini API (check model arguments)".to_string()),
                401 | 403 => AiError::Authentication("Gemini API key invalid or unauthorized".to_string()),
                404 => AiError::ModelNotFound("Requested Gemini model not found".to_string()),
                429 => AiError::RateLimit("Gemini quota exceeded or rate limited (HTTP 429)".to_string()),
                other => AiError::HttpStatus {
                    code: other,
                    message: format!("Google Gemini HTTP error {other}"),
                },
            },
            ureq::Error::Timeout(_) => AiError::Connection("Gemini request timed out".to_string()),
            other => AiError::Connection(other.to_string()),
        }
    }

    /// Evaluates tool/function calling support for a Gemini model.
    pub fn eval_tool_support(id: &str, methods: Option<&Vec<Value>>) -> ToolSupport {
        let lower = id.to_ascii_lowercase();
        // 1. Must support generateContent
        if let Some(meths) = methods {
            if !meths.iter().any(|v| v.as_str() == Some("generateContent")) {
                return ToolSupport::Unsupported;
            }
        }
        if lower.contains("embedding") || lower.contains("aqa") || lower.contains("imagen") || lower.contains("text-embedding") {
            return ToolSupport::Unsupported;
        }

        // 2. Verified specific models supporting function/tool calling
        const VERIFIED_GEMINI_TOOL_MODELS: &[&str] = &[
            "gemini-2.5-flash",
            "gemini-2.5-pro",
            "gemini-2.0-flash",
            "gemini-2.0-flash-exp",
            "gemini-1.5-pro",
            "gemini-1.5-pro-latest",
            "gemini-1.5-flash",
            "gemini-1.5-flash-latest",
            "gemini-1.5-flash-8b",
            "gemini-pro",
        ];

        let base_id = lower.strip_prefix("models/").unwrap_or(&lower);
        if VERIFIED_GEMINI_TOOL_MODELS.iter().any(|m| base_id.starts_with(m)) {
            ToolSupport::Supported
        } else if lower.contains("gemini") {
            ToolSupport::Unknown
        } else {
            ToolSupport::Unsupported
        }
    }
}

impl LlmClient for GeminiClient {
    fn test_connection(&self) -> Result<ConnectionTestResult, AiError> {
        let start = Instant::now();
        let models_url = format!("{}/models", self.base_url);

        let req = self.agent.get(&models_url);
        let req = match self.apply_auth(req) {
            Ok(r) => r,
            Err(e) => {
                return Ok(ConnectionTestResult {
                    success: false,
                    latency_ms: 0,
                    models: Vec::new(),
                    message: e.to_string(),
                    error_code: Some("AUTH_MISSING".to_string()),
                });
            }
        };

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
                    message: format!("Failed to read Gemini models response: {e}"),
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
                    message: format!("Failed to parse Gemini models JSON: {e}"),
                    error_code: Some("PARSE_ERROR".to_string()),
                });
            }
        };

        let mut models = Vec::new();
        if let Some(arr) = body.get("models").and_then(Value::as_array) {
            for m in arr {
                let raw_name = m.get("name").and_then(Value::as_str).unwrap_or("");
                let id = raw_name.strip_prefix("models/").unwrap_or(raw_name).to_string();
                let display = m.get("displayName").and_then(Value::as_str).unwrap_or(&id).to_string();
                let desc = m.get("description").and_then(Value::as_str).map(String::from);

                // Only include models supporting content generation
                let methods = m.get("supportedGenerationMethods").and_then(Value::as_array);
                let can_generate = methods.map_or(true, |arr| {
                    arr.iter().any(|v| v.as_str() == Some("generateContent"))
                });

                if can_generate {
                    let tool_support = Self::eval_tool_support(&id, methods);
                    let mut modalities = vec!["chat".to_string()];
                    if tool_support == ToolSupport::Supported {
                        modalities.push("tools".to_string());
                    }
                    let is_preview = id.contains("exp") || id.contains("preview");
                    let is_deprecated = id.contains("deprecated") || desc.as_deref().unwrap_or("").to_ascii_lowercase().contains("deprecated");

                    models.push(ModelInfo {
                        id,
                        name: display,
                        description: desc,
                        tool_support,
                        supports_streaming: true,
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
            message: format!("Google Gemini connection healthy ({}ms)", latency),
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

        let model_id = if self.config.model.starts_with("models/") {
            self.config.model.clone()
        } else {
            format!("models/{}", self.config.model)
        };

        let stream_url = format!("{}/{}:streamGenerateContent?alt=sse", self.base_url, model_id);

        let mut system_instruction: Option<Value> = None;
        let mut contents: Vec<Value> = Vec::new();

        for m in messages {
            match m.role {
                MessageRole::System => {
                    system_instruction = Some(json!({
                        "parts": [{ "text": m.content }]
                    }));
                }
                MessageRole::User => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [{ "text": m.content }]
                    }));
                }
                MessageRole::Assistant => {
                    let mut parts = Vec::new();
                    if !m.content.is_empty() {
                        parts.push(json!({ "text": m.content }));
                    }
                    if let Some(tool_calls) = &m.tool_calls {
                        for tc in tool_calls {
                            parts.push(json!({
                                "functionCall": {
                                    "name": tc.name,
                                    "args": tc.arguments,
                                }
                            }));
                        }
                    }
                    contents.push(json!({
                        "role": "model",
                        "parts": parts
                    }));
                }
                MessageRole::Tool => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [{
                            "functionResponse": {
                                "name": m.tool_call_id.clone().unwrap_or_else(|| "tool_result".to_string()),
                                "response": {
                                    "output": m.content,
                                }
                            }
                        }]
                    }));
                }
            }
        }

        let mut payload = json!({
            "contents": contents,
        });

        if let Some(sys) = system_instruction {
            payload["systemInstruction"] = sys;
        }

        if !tools.is_empty() {
            let decls: Vec<Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    })
                })
                .collect();
            payload["tools"] = json!([{
                "functionDeclarations": decls
            }]);
        }

        let mut gen_config = json!({});
        if let Some(temp) = self.config.temperature {
            gen_config["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            gen_config["maxOutputTokens"] = json!(max_tokens);
        }
        if gen_config.as_object().map_or(false, |o| !o.is_empty()) {
            payload["generationConfig"] = gen_config;
        }

        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| AiError::Protocol(format!("Failed to serialize Gemini payload: {e}")))?;

        let req = self.agent.post(&stream_url).header("Content-Type", "application/json");
        let req = self.apply_auth(req)?;

        let mut resp = req.send(&payload_bytes[..]).map_err(Self::map_ureq_error)?;

        let reader = resp.body_mut().as_reader();
        let buf_reader = BufReader::new(reader);

        let mut call_counter = 0;

        for line_result in buf_reader.lines() {
            if cancel.load(Ordering::Relaxed) {
                return Err(AiError::Cancelled);
            }

            let line = line_result.map_err(|e| AiError::Connection(format!("Error reading Gemini stream: {e}")))?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with(':') {
                continue;
            }

            if let Some(data) = trimmed.strip_prefix("data:") {
                let payload = data.trim();
                let chunk_val: Value = match serde_json::from_str(payload) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(candidates) = chunk_val.get("candidates").and_then(Value::as_array) {
                    for cand in candidates {
                        let finish_reason = cand.get("finishReason").and_then(Value::as_str).map(String::from);

                        if let Some(parts) = cand.get("content").and_then(|c| c.get("parts")).and_then(Value::as_array) {
                            for part in parts {
                                if let Some(text) = part.get("text").and_then(Value::as_str) {
                                    if !text.is_empty() {
                                        callback(ChatDelta::Text(text.to_string()));
                                    }
                                }

                                if let Some(fc) = part.get("functionCall") {
                                    let name = fc.get("name").and_then(Value::as_str).unwrap_or("").to_string();
                                    let args = fc.get("args").cloned().unwrap_or(json!({}));
                                    let call_id = format!("call_gemini_{call_counter}");
                                    call_counter += 1;

                                    callback(ChatDelta::ToolCallChunk {
                                        index: call_counter - 1,
                                        id: Some(call_id),
                                        name: Some(name),
                                        args_chunk: args.to_string(),
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
        }

        Ok(())
    }

    fn check_model_tool_support(&self, model: &str) -> ToolSupport {
        Self::eval_tool_support(model, None)
    }
}

use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{header, Client, Response, StatusCode};
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::config::{ProviderConfig, ProviderKind};
use crate::provider::{CompletionStream, ModelProvider};
use crate::streaming::{ChunkDelta, StreamChunk};
use crate::tool_schema::{ToolCall, ToolDefinition};
use crate::types::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentBlock, EmbeddingResponse,
    ModelCapabilities, StopReason, Usage,
};
use crate::LlmError;

/// OpenAI Responses API provider backed by a standard API key.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    config: ProviderConfig,
    client: Client,
}

impl OpenAiProvider {
    /// Construct a new OpenAI API-key provider.
    pub fn new(config: ProviderConfig) -> Result<Self, LlmError> {
        config.validate()?;
        if config.provider_kind != ProviderKind::OpenAi {
            return Err(LlmError::InvalidRequest(format!(
                "OpenAiProvider requires provider_kind 'openai', got '{}'",
                config.provider_kind
            )));
        }

        Ok(Self {
            config,
            client: Client::new(),
        })
    }

    fn api_key(&self) -> Result<String, LlmError> {
        let env_name = self
            .config
            .api_key_env
            .as_deref()
            .ok_or_else(|| LlmError::InvalidRequest("missing api_key_env".to_string()))?;
        let api_key = std::env::var(env_name).map_err(|_| {
            LlmError::Authentication(format!(
                "environment variable '{env_name}' is not set for OpenAI authentication"
            ))
        })?;
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            return Err(LlmError::Authentication(format!(
                "environment variable '{env_name}' is empty for OpenAI authentication"
            )));
        }
        Ok(trimmed.to_string())
    }

    fn base_url(&self) -> &str {
        self.config
            .api_base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
    }

    fn responses_url(&self) -> String {
        format!("{}/responses", self.base_url().trim_end_matches('/'))
    }

    fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.base_url().trim_end_matches('/'))
    }

    fn embedding_model_id(&self) -> &str {
        self.config
            .metadata
            .get("embedding_model_id")
            .and_then(Value::as_str)
            .unwrap_or(&self.config.model_id)
    }

    fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.config.timeout_ms)
    }

    fn request_body(&self, request: &CompletionRequest, stream: bool) -> Value {
        let mut body = json!({
            "model": self.config.model_id,
            "input": build_input_items(request),
            "stream": stream
        });

        if let Some(system) = request.system.as_ref() {
            body["instructions"] = Value::String(system.clone());
        }
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_output_tokens"] = json!(max_tokens);
        }
        if let Some(reasoning_effort) = reasoning_effort_override(&self.config.model_id, request) {
            body["reasoning"] = json!({ "effort": reasoning_effort });
        }
        if let Some(stop_sequences) = request.stop_sequences.as_ref() {
            body["stop"] = json!(stop_sequences);
        }
        if let Some(tools) = request.tools.as_ref() {
            body["tools"] = Value::Array(tools.iter().map(tool_to_openai).collect());
            body["tool_choice"] = Value::String("auto".to_string());
            body["parallel_tool_calls"] = Value::Bool(false);
        }

        body
    }

    async fn execute_json(&self, url: String, body: Value) -> Result<Value, LlmError> {
        for attempt in 0..=self.config.max_retries {
            match self.send_request(&url, &body).await {
                Ok(response) if response.status().is_success() => {
                    return parse_json_response(response).await;
                }
                Ok(response) => {
                    let error = parse_error_response(response).await;
                    if self.should_retry(&error, attempt) {
                        tokio::time::sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    return Err(error);
                }
                Err(error) => {
                    if self.should_retry(&error, attempt) {
                        tokio::time::sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    return Err(error);
                }
            }
        }

        Err(LlmError::ProviderError {
            status: 500,
            message: "OpenAI request exhausted retry budget".to_string(),
            retryable: false,
        })
    }

    async fn execute_streaming(
        &self,
        request: CompletionRequest,
        stream_tx: mpsc::Sender<Result<StreamChunk, LlmError>>,
    ) -> Result<(), LlmError> {
        let body = self.request_body(&request, true);
        let responses_url = self.responses_url();
        let mut response = None;
        for attempt in 0..=self.config.max_retries {
            match self.send_request(&responses_url, &body).await {
                Ok(candidate) if candidate.status().is_success() => {
                    response = Some(candidate);
                    break;
                }
                Ok(candidate) => {
                    let error = parse_error_response(candidate).await;
                    if self.should_retry(&error, attempt) {
                        tokio::time::sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    return Err(error);
                }
                Err(error) => {
                    if self.should_retry(&error, attempt) {
                        tokio::time::sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    return Err(error);
                }
            }
        }

        let response = response.ok_or_else(|| LlmError::ProviderError {
            status: 500,
            message: "OpenAI streaming request exhausted retry budget".to_string(),
            retryable: false,
        })?;

        let mut bytes_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut event_name: Option<String> = None;
        let mut data_lines: Vec<String> = Vec::new();
        let mut saw_tool_call = false;
        let mut index = 0usize;

        while let Some(chunk) = bytes_stream.next().await {
            let chunk = chunk.map_err(|error| LlmError::Network(error.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline_index) = buffer.find('\n') {
                let mut line = buffer[..newline_index].to_string();
                buffer = buffer[newline_index + 1..].to_string();

                if line.ends_with('\r') {
                    line.pop();
                }

                if line.is_empty() {
                    process_sse_event(
                        event_name.take(),
                        std::mem::take(&mut data_lines),
                        &stream_tx,
                        &mut index,
                        &mut saw_tool_call,
                    )
                    .await?;
                    continue;
                }

                if let Some(rest) = line.strip_prefix("event:") {
                    event_name = Some(rest.trim().to_string());
                    continue;
                }

                if let Some(rest) = line.strip_prefix("data:") {
                    data_lines.push(rest.trim_start().to_string());
                }
            }
        }

        if !data_lines.is_empty() {
            process_sse_event(
                event_name.take(),
                std::mem::take(&mut data_lines),
                &stream_tx,
                &mut index,
                &mut saw_tool_call,
            )
            .await?;
        }

        Ok(())
    }

    async fn send_request(&self, url: &str, body: &Value) -> Result<Response, LlmError> {
        self.client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key()?))
            .header(header::CONTENT_TYPE, "application/json")
            .timeout(self.request_timeout())
            .json(body)
            .send()
            .await
            .map_err(|error| normalize_request_error(self.config.timeout_ms, error))
    }

    fn should_retry(&self, error: &LlmError, attempt: u32) -> bool {
        if attempt >= self.config.max_retries {
            return false;
        }

        matches!(
            error,
            LlmError::Network(_)
                | LlmError::ProviderError {
                    retryable: true,
                    ..
                }
        )
    }
}

#[async_trait]
impl ModelProvider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let payload = self
            .execute_json(self.responses_url(), self.request_body(&request, false))
            .await?;
        normalize_completion_response(&self.config.model_id, payload)
    }

    fn stream(&self, request: CompletionRequest) -> CompletionStream {
        let provider = self.clone();
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            if let Err(error) = provider.execute_streaming(request, tx.clone()).await {
                let _ = tx.send(Err(error)).await;
            }
        });

        Box::pin(futures::stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|item| (item, rx))
        }))
    }

    async fn embed(&self, input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        let payload = self
            .execute_json(
                self.embeddings_url(),
                json!({
                    "model": self.embedding_model_id(),
                    "input": input,
                    "encoding_format": "float"
                }),
            )
            .await?;

        let embeddings = payload
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                LlmError::Serialization("OpenAI embeddings response missing data array".to_string())
            })?
            .iter()
            .map(|entry| {
                entry
                    .get("embedding")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        LlmError::Serialization(
                            "OpenAI embeddings response entry missing embedding array".to_string(),
                        )
                    })?
                    .iter()
                    .map(|value| {
                        value.as_f64().map(|number| number as f32).ok_or_else(|| {
                            LlmError::Serialization(
                                "OpenAI embedding vector contained a non-numeric value".to_string(),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let usage = payload
            .get("usage")
            .map(normalize_usage)
            .transpose()?
            .unwrap_or_default();

        Ok(EmbeddingResponse {
            embeddings,
            model_id: payload
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or(self.embedding_model_id())
                .to_string(),
            usage,
        })
    }

    fn model_id(&self) -> &str {
        &self.config.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::all()
    }
}

fn build_input_items(request: &CompletionRequest) -> Vec<Value> {
    request
        .messages
        .iter()
        .map(|message| match message {
            ChatMessage::System { content } => json!({
                "type": "message",
                "role": "system",
                "content": [{ "type": "input_text", "text": content }]
            }),
            ChatMessage::User { content } => json!({
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": render_json_value(content) }]
            }),
            ChatMessage::Assistant { content } => json!({
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "input_text", "text": render_json_value(content) }]
            }),
            ChatMessage::Tool { result } => json!({
                "type": "function_call_output",
                "call_id": result.call_id,
                "output": render_tool_result(result)
            }),
        })
        .collect()
}

fn tool_to_openai(tool: &ToolDefinition) -> Value {
    json!({
        "type": "function",
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.input_schema
    })
}

fn render_json_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn render_tool_result(result: &crate::tool_schema::ToolResult) -> String {
    match (&result.output, &result.error) {
        (Some(output), None) => render_json_value(output),
        (None, Some(error)) => error.clone(),
        (Some(output), Some(error)) => {
            format!("output={} error={error}", render_json_value(output))
        }
        (None, None) => String::new(),
    }
}

async fn parse_json_response(response: Response) -> Result<Value, LlmError> {
    if !response.status().is_success() {
        return Err(parse_error_response(response).await);
    }

    response
        .json::<Value>()
        .await
        .map_err(|error| LlmError::Serialization(error.to_string()))
}

async fn parse_error_response(response: Response) -> LlmError {
    let status = response.status();
    let retry_after_secs = response
        .headers()
        .get(header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let body = response.text().await.unwrap_or_default();
    let message = serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| {
            if body.is_empty() {
                format!("OpenAI request failed with status {status}")
            } else {
                body
            }
        });

    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => LlmError::Authentication(message),
        StatusCode::TOO_MANY_REQUESTS => LlmError::RateLimited { retry_after_secs },
        _ => LlmError::ProviderError {
            status: status.as_u16(),
            message,
            retryable: status.is_server_error(),
        },
    }
}

fn normalize_completion_response(
    fallback_model_id: &str,
    payload: Value,
) -> Result<CompletionResponse, LlmError> {
    let usage = payload
        .get("usage")
        .map(normalize_usage)
        .transpose()?
        .unwrap_or_default();
    let mut content = Vec::new();
    let mut tool_calls = Vec::new();

    if let Some(output) = payload.get("output").and_then(Value::as_array) {
        for item in output {
            match item.get("type").and_then(Value::as_str) {
                Some("message") | Some("output_message") => {
                    if let Some(parts) = item.get("content").and_then(Value::as_array) {
                        for part in parts {
                            match part.get("type").and_then(Value::as_str) {
                                Some("output_text") | Some("text") => {
                                    if let Some(text) = part
                                        .get("text")
                                        .and_then(Value::as_str)
                                        .or_else(|| part.get("value").and_then(Value::as_str))
                                    {
                                        content.push(ContentBlock::Text {
                                            text: text.to_string(),
                                        });
                                    }
                                }
                                Some("refusal") => {
                                    if let Some(text) = part.get("refusal").and_then(Value::as_str)
                                    {
                                        content.push(ContentBlock::Text {
                                            text: text.to_string(),
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Some("function_call") => {
                    let call = normalize_tool_call(item)?;
                    content.push(ContentBlock::ToolUse {
                        call_id: call.call_id.clone(),
                        name: call.name.clone(),
                        input: call.input.clone(),
                    });
                    tool_calls.push(call);
                }
                _ => {}
            }
        }
    }

    if content.is_empty() {
        if let Some(text) = payload.get("output_text").and_then(Value::as_str) {
            content.push(ContentBlock::Text {
                text: text.to_string(),
            });
        }
    }

    let stop_reason = if !tool_calls.is_empty() {
        StopReason::ToolCall
    } else {
        normalize_stop_reason(
            payload.get("status").and_then(Value::as_str),
            payload
                .get("incomplete_details")
                .and_then(|details| details.get("reason"))
                .and_then(Value::as_str),
        )
    };

    if content.is_empty()
        && tool_calls.is_empty()
        && !matches!(stop_reason, StopReason::ContentFilter)
    {
        let status = payload
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let detail = payload
            .get("incomplete_details")
            .and_then(|details| details.get("reason"))
            .and_then(Value::as_str);
        let detail_suffix = detail
            .map(|reason| format!(" and reason '{reason}'"))
            .unwrap_or_default();

        return Err(LlmError::InvalidRequest(format!(
            "OpenAI response contained no visible output or tool calls (status '{status}'{detail_suffix}). Increase max_tokens or lower reasoning effort."
        )));
    }

    Ok(CompletionResponse {
        content,
        model_id: payload
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(fallback_model_id)
            .to_string(),
        usage,
        stop_reason,
        tool_calls,
    })
}

fn normalize_tool_call(item: &Value) -> Result<ToolCall, LlmError> {
    let call_id = item
        .get("call_id")
        .and_then(Value::as_str)
        .or_else(|| item.get("id").and_then(Value::as_str))
        .ok_or_else(|| {
            LlmError::Serialization("OpenAI function_call item missing call identifier".to_string())
        })?;
    let name = item.get("name").and_then(Value::as_str).ok_or_else(|| {
        LlmError::Serialization("OpenAI function_call item missing name".to_string())
    })?;
    let arguments = item.get("arguments").ok_or_else(|| {
        LlmError::Serialization("OpenAI function_call item missing arguments".to_string())
    })?;

    Ok(ToolCall {
        call_id: call_id.to_string(),
        name: name.to_string(),
        input: parse_json_string(arguments)?,
    })
}

fn parse_json_string(value: &Value) -> Result<Value, LlmError> {
    match value {
        Value::String(text) => serde_json::from_str(text).map_err(|error| {
            LlmError::Serialization(format!("failed to decode JSON arguments: {error}"))
        }),
        other => Ok(other.clone()),
    }
}

fn normalize_usage(payload: &Value) -> Result<Usage, LlmError> {
    let input_tokens = payload
        .get("input_tokens")
        .and_then(Value::as_u64)
        .or_else(|| payload.get("prompt_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let output_tokens = payload
        .get("output_tokens")
        .and_then(Value::as_u64)
        .or_else(|| payload.get("completion_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let total_tokens = payload
        .get("total_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(input_tokens + output_tokens);

    Ok(Usage {
        input_tokens,
        output_tokens,
        total_tokens,
    })
}

fn retry_delay(attempt: u32) -> Duration {
    let factor = 1u64 << attempt.min(3);
    Duration::from_millis(100 * factor)
}

fn normalize_request_error(timeout_ms: u64, error: reqwest::Error) -> LlmError {
    if error.is_timeout() {
        LlmError::Network(format!("OpenAI request timed out after {timeout_ms}ms"))
    } else {
        LlmError::Network(error.to_string())
    }
}

fn reasoning_effort_override<'a>(
    model_id: &'a str,
    request: &CompletionRequest,
) -> Option<&'static str> {
    if request.max_tokens.is_none() {
        return None;
    }

    if model_id.starts_with("gpt-5.1") || model_id.starts_with("gpt-5-pro") {
        return None;
    }

    if model_id.starts_with("gpt-5") || model_id.starts_with('o') {
        return Some("minimal");
    }

    None
}

fn normalize_stop_reason(status: Option<&str>, incomplete_reason: Option<&str>) -> StopReason {
    match (status, incomplete_reason) {
        (_, Some("content_filter")) => StopReason::ContentFilter,
        (Some("cancelled"), _) => StopReason::Cancelled,
        (Some("incomplete"), _) => StopReason::MaxTokens,
        (Some("failed"), _) => StopReason::ProviderSpecificFallback,
        _ => StopReason::Completed,
    }
}

async fn process_sse_event(
    event_name: Option<String>,
    data_lines: Vec<String>,
    stream_tx: &mpsc::Sender<Result<StreamChunk, LlmError>>,
    index: &mut usize,
    saw_tool_call: &mut bool,
) -> Result<(), LlmError> {
    if data_lines.is_empty() {
        return Ok(());
    }

    let data = data_lines.join("\n");
    if data == "[DONE]" {
        let _ = stream_tx
            .send(Ok(StreamChunk::stop(
                *index,
                if *saw_tool_call {
                    StopReason::ToolCall
                } else {
                    StopReason::Completed
                },
            )))
            .await;
        *index += 1;
        return Ok(());
    }

    let payload: Value =
        serde_json::from_str(&data).map_err(|error| LlmError::Serialization(error.to_string()))?;
    let event_type = event_name
        .as_deref()
        .or_else(|| payload.get("type").and_then(Value::as_str))
        .unwrap_or_default();

    match event_type {
        "response.output_text.delta" => {
            if let Some(delta) = payload.get("delta").and_then(Value::as_str) {
                let _ = stream_tx
                    .send(Ok(StreamChunk {
                        index: *index,
                        delta: ChunkDelta::Text {
                            text: delta.to_string(),
                        },
                    }))
                    .await;
                *index += 1;
            }
        }
        "response.function_call_arguments.done" => {
            let item = payload.get("item").ok_or_else(|| {
                LlmError::Serialization(
                    "OpenAI streaming event missing function_call item".to_string(),
                )
            })?;
            let call = normalize_tool_call(item)?;
            let _ = stream_tx
                .send(Ok(StreamChunk {
                    index: *index,
                    delta: ChunkDelta::ToolUseStart {
                        call_id: call.call_id.clone(),
                        name: call.name.clone(),
                    },
                }))
                .await;
            *index += 1;
            let _ = stream_tx
                .send(Ok(StreamChunk {
                    index: *index,
                    delta: ChunkDelta::ToolUseInput {
                        call_id: call.call_id,
                        input: call.input,
                    },
                }))
                .await;
            *index += 1;
            *saw_tool_call = true;
        }
        "response.completed" => {
            let _ = stream_tx
                .send(Ok(StreamChunk::stop(
                    *index,
                    if *saw_tool_call {
                        StopReason::ToolCall
                    } else {
                        StopReason::Completed
                    },
                )))
                .await;
            *index += 1;
        }
        "response.failed" | "error" => {
            return Err(LlmError::ProviderError {
                status: 500,
                message: payload
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("OpenAI streaming request failed")
                    .to_string(),
                retryable: false,
            });
        }
        _ => {}
    }

    Ok(())
}

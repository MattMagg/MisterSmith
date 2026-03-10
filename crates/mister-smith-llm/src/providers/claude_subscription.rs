//! Claude subscription provider — direct HTTP to Anthropic Messages API.
//!
//! Authenticates with OAuth Bearer tokens from Claude Code CLI credential stores
//! (Keychain, `~/.claude/.credentials.json`, or `CLAUDE_CODE_OAUTH_TOKEN` env var).
//! Tokens are auto-refreshed when expired.
//!
//! Capabilities: completion, streaming, tool calling. No embeddings (Anthropic
//! does not expose an embeddings endpoint for subscription accounts).

use std::sync::RwLock;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{header, Client, Response, StatusCode};
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::claude_credentials::{self, ClaudeOAuthCredentials};
use crate::config::{ProviderConfig, ProviderKind};
use crate::provider::{CompletionStream, ModelProvider};
use crate::streaming::{ChunkDelta, StreamChunk};
use crate::tool_schema::{ToolCall, ToolDefinition};
use crate::types::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentBlock, EmbeddingResponse,
    ModelCapabilities, StopReason, Usage,
};
use crate::LlmError;

const DEFAULT_API_BASE: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Claude subscription provider using direct HTTP to the Anthropic Messages API.
///
/// Authentication uses OAuth Bearer tokens read from Claude Code CLI credential
/// stores. Tokens are cached and auto-refreshed when expired.
#[derive(Debug)]
pub struct ClaudeSubscriptionProvider {
    config: ProviderConfig,
    client: Client,
    credentials: RwLock<Option<ClaudeOAuthCredentials>>,
}

impl ClaudeSubscriptionProvider {
    /// Construct a new Claude subscription provider.
    pub fn new(config: ProviderConfig) -> Result<Self, LlmError> {
        config.validate()?;
        if config.provider_kind != ProviderKind::ClaudeSubscription {
            return Err(LlmError::InvalidRequest(format!(
                "ClaudeSubscriptionProvider requires provider_kind 'claude_subscription', got '{}'",
                config.provider_kind
            )));
        }

        Ok(Self {
            config,
            client: Client::new(),
            credentials: RwLock::new(None),
        })
    }

    fn base_url(&self) -> &str {
        self.config
            .api_base_url
            .as_deref()
            .unwrap_or(DEFAULT_API_BASE)
    }

    fn messages_url(&self) -> String {
        format!("{}/v1/messages", self.base_url().trim_end_matches('/'))
    }

    fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.config.timeout_ms)
    }

    /// Get a valid access token, refreshing if expired.
    async fn access_token(&self) -> Result<String, LlmError> {
        // Check cached credentials first.
        {
            let guard = self
                .credentials
                .read()
                .map_err(|_| LlmError::ProviderError {
                    status: 500,
                    message: "credential lock poisoned".to_string(),
                    retryable: false,
                })?;
            if let Some(creds) = guard.as_ref() {
                if !creds.is_expired() {
                    return Ok(creds.access_token.clone());
                }
            }
        }

        // Load fresh credentials.
        let mut creds = claude_credentials::read_credentials()?;

        // If expired and we have a refresh token, refresh.
        if creds.is_expired() {
            if let Some(refresh_token) = creds.refresh_token.as_deref() {
                creds =
                    claude_credentials::refresh_access_token(&self.client, refresh_token).await?;
            } else {
                return Err(LlmError::Authentication(
                    "Claude subscription token expired and no refresh token available. Re-authenticate with `claude setup-token` or Claude Code CLI.".to_string(),
                ));
            }
        }

        let token = creds.access_token.clone();

        // Cache the credentials.
        if let Ok(mut guard) = self.credentials.write() {
            *guard = Some(creds);
        }

        Ok(token)
    }

    fn build_request_body(&self, request: &CompletionRequest, stream: bool) -> Value {
        let mut body = json!({
            "model": self.config.model_id,
            "max_tokens": request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            "stream": stream,
        });

        // System prompt goes in a top-level field, not in messages.
        if let Some(system) = request.system.as_ref() {
            body["system"] = Value::String(system.clone());
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(stop_sequences) = request.stop_sequences.as_ref() {
            if !stop_sequences.is_empty() {
                body["stop_sequences"] = json!(stop_sequences);
            }
        }

        if let Some(tools) = request.tools.as_ref() {
            if !tools.is_empty() {
                body["tools"] = Value::Array(tools.iter().map(tool_to_anthropic).collect());
            }
        }

        body["messages"] = Value::Array(build_messages(&request.messages));

        body
    }

    async fn send_request(&self, url: &str, body: Vec<u8>) -> Result<Response, LlmError> {
        let token = self.access_token().await?;

        self.client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, "application/json")
            .header("anthropic-version", ANTHROPIC_VERSION)
            .timeout(self.request_timeout())
            .body(body)
            .send()
            .await
            .map_err(|error| normalize_request_error(self.config.timeout_ms, error))
    }

    async fn execute_json(&self, request: &CompletionRequest) -> Result<Value, LlmError> {
        let body = self.build_request_body(request, false);
        let body_bytes = serde_json::to_vec(&body).map_err(|error| {
            LlmError::Serialization(format!("Failed to serialize request: {error}"))
        })?;
        let url = self.messages_url();

        for attempt in 0..=self.config.max_retries {
            match self.send_request(&url, body_bytes.clone()).await {
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
            message: "Claude subscription request exhausted retry budget".to_string(),
            retryable: false,
        })
    }

    async fn execute_streaming(
        &self,
        request: CompletionRequest,
        stream_tx: mpsc::Sender<Result<StreamChunk, LlmError>>,
    ) -> Result<(), LlmError> {
        let body = self.build_request_body(&request, true);
        let body_bytes = serde_json::to_vec(&body).map_err(|error| {
            LlmError::Serialization(format!("Failed to serialize request: {error}"))
        })?;
        let url = self.messages_url();

        let mut response = None;
        for attempt in 0..=self.config.max_retries {
            match self.send_request(&url, body_bytes.clone()).await {
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
            message: "Claude subscription streaming request exhausted retry budget".to_string(),
            retryable: false,
        })?;

        let mut bytes_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut event_name: Option<String> = None;
        let mut data_lines: Vec<String> = Vec::new();
        let mut index = 0usize;
        let mut saw_tool_call = false;
        // Accumulate partial JSON input for tool calls across deltas.
        let mut tool_input_buffers: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        // Track tool call_id → name for ToolUseStart events.
        let mut tool_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

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
                    // End of SSE event — process it.
                    process_anthropic_sse_event(
                        event_name.take(),
                        std::mem::take(&mut data_lines),
                        &stream_tx,
                        &mut index,
                        &mut saw_tool_call,
                        &mut tool_input_buffers,
                        &mut tool_names,
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

        // Flush any trailing event.
        if !data_lines.is_empty() {
            process_anthropic_sse_event(
                event_name.take(),
                std::mem::take(&mut data_lines),
                &stream_tx,
                &mut index,
                &mut saw_tool_call,
                &mut tool_input_buffers,
                &mut tool_names,
            )
            .await?;
        }

        Ok(())
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

// Allow cloning via Arc<RwLock<_>> semantics — the provider is shared across tasks.
impl Clone for ClaudeSubscriptionProvider {
    fn clone(&self) -> Self {
        let cached = self.credentials.read().ok().and_then(|g| g.clone());
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            credentials: RwLock::new(cached),
        }
    }
}

#[async_trait]
impl ModelProvider for ClaudeSubscriptionProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let payload = self.execute_json(&request).await?;
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

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".to_string(),
            model: self.config.model_id.clone(),
        })
    }

    fn model_id(&self) -> &str {
        &self.config.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            completion: true,
            streaming: true,
            embeddings: false,
            tool_calling: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Message building — Anthropic Messages API format
// ---------------------------------------------------------------------------

/// Build the `messages` array for the Anthropic Messages API.
///
/// Key differences from OpenAI:
/// - System messages are a top-level field, not in messages.
/// - Roles are `user` and `assistant` only.
/// - Tool results must be in a `user` message with `tool_result` content blocks.
/// - Consecutive same-role messages are not allowed — merge tool results into
///   a single user message.
fn build_messages(messages: &[ChatMessage]) -> Vec<Value> {
    let mut result: Vec<Value> = Vec::new();

    for message in messages {
        match message {
            ChatMessage::System { .. } => {
                // System messages are handled at the top level, not here.
            }
            ChatMessage::User { content } => {
                let text = render_json_value(content);
                result.push(json!({
                    "role": "user",
                    "content": [{ "type": "text", "text": text }]
                }));
            }
            ChatMessage::Assistant { content } => {
                let text = render_json_value(content);
                result.push(json!({
                    "role": "assistant",
                    "content": [{ "type": "text", "text": text }]
                }));
            }
            ChatMessage::Tool {
                result: tool_result,
            } => {
                let block = json!({
                    "type": "tool_result",
                    "tool_use_id": tool_result.call_id,
                    "content": render_tool_result_content(tool_result),
                    "is_error": tool_result.error.is_some()
                });

                // Anthropic requires tool_result blocks in a user message.
                // If the last message is already a user message, append to it.
                if let Some(last) = result.last_mut() {
                    if last.get("role").and_then(Value::as_str) == Some("user") {
                        if let Some(content) = last.get_mut("content").and_then(Value::as_array_mut)
                        {
                            content.push(block);
                            continue;
                        }
                    }
                }
                result.push(json!({
                    "role": "user",
                    "content": [block]
                }));
            }
        }
    }

    result
}

fn tool_to_anthropic(tool: &ToolDefinition) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.input_schema
    })
}

fn render_json_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn render_tool_result_content(result: &crate::tool_schema::ToolResult) -> String {
    match (&result.output, &result.error) {
        (Some(output), None) => render_json_value(output),
        (None, Some(error)) => error.clone(),
        (Some(output), Some(error)) => {
            format!("output={} error={error}", render_json_value(output))
        }
        (None, None) => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Response parsing — Anthropic Messages API format
// ---------------------------------------------------------------------------

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

    if let Some(content_blocks) = payload.get("content").and_then(Value::as_array) {
        for block in content_blocks {
            match block.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        content.push(ContentBlock::Text {
                            text: text.to_string(),
                        });
                    }
                }
                Some("tool_use") => {
                    let call = normalize_tool_call(block)?;
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

    let stop_reason = if !tool_calls.is_empty() {
        StopReason::ToolCall
    } else {
        normalize_stop_reason(payload.get("stop_reason").and_then(Value::as_str))
    };

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

fn normalize_tool_call(block: &Value) -> Result<ToolCall, LlmError> {
    let call_id = block.get("id").and_then(Value::as_str).ok_or_else(|| {
        LlmError::Serialization("Anthropic tool_use block missing id".to_string())
    })?;
    let name = block.get("name").and_then(Value::as_str).ok_or_else(|| {
        LlmError::Serialization("Anthropic tool_use block missing name".to_string())
    })?;
    let input = block.get("input").cloned().unwrap_or(json!({}));

    Ok(ToolCall {
        call_id: call_id.to_string(),
        name: name.to_string(),
        input,
    })
}

fn normalize_usage(payload: &Value) -> Result<Usage, LlmError> {
    let input_tokens = payload
        .get("input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = payload
        .get("output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    Ok(Usage::new(input_tokens, output_tokens))
}

fn normalize_stop_reason(reason: Option<&str>) -> StopReason {
    match reason {
        Some("end_turn") => StopReason::Completed,
        Some("max_tokens") => StopReason::MaxTokens,
        Some("tool_use") => StopReason::ToolCall,
        Some("stop_sequence") => StopReason::Completed,
        _ => StopReason::Completed,
    }
}

// ---------------------------------------------------------------------------
// SSE streaming — Anthropic event format
// ---------------------------------------------------------------------------

/// Process a single Anthropic SSE event.
///
/// Anthropic SSE event types:
/// - `message_start` — contains the message metadata
/// - `content_block_start` — new content block (text or tool_use)
/// - `content_block_delta` — incremental delta (text_delta or input_json_delta)
/// - `content_block_stop` — content block finished
/// - `message_delta` — message-level updates (stop_reason, usage)
/// - `message_stop` — stream complete
/// - `error` — stream-level error
#[allow(clippy::too_many_arguments)]
async fn process_anthropic_sse_event(
    event_name: Option<String>,
    data_lines: Vec<String>,
    stream_tx: &mpsc::Sender<Result<StreamChunk, LlmError>>,
    index: &mut usize,
    saw_tool_call: &mut bool,
    tool_input_buffers: &mut std::collections::HashMap<String, String>,
    tool_names: &mut std::collections::HashMap<String, String>,
) -> Result<(), LlmError> {
    if data_lines.is_empty() {
        return Ok(());
    }

    let data = data_lines.join("\n");
    let payload: Value =
        serde_json::from_str(&data).map_err(|error| LlmError::Serialization(error.to_string()))?;

    let event_type = event_name.as_deref().unwrap_or_default();

    match event_type {
        "content_block_start" => {
            if let Some(content_block) = payload.get("content_block") {
                match content_block.get("type").and_then(Value::as_str) {
                    Some("tool_use") => {
                        let call_id = content_block
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let name = content_block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();

                        let _ = stream_tx
                            .send(Ok(StreamChunk {
                                index: *index,
                                delta: ChunkDelta::ToolUseStart {
                                    call_id: call_id.clone(),
                                    name: name.clone(),
                                },
                            }))
                            .await;
                        *index += 1;
                        *saw_tool_call = true;

                        tool_input_buffers.insert(call_id.clone(), String::new());
                        tool_names.insert(call_id, name);
                    }
                    _ => {
                        // text blocks start emitting via content_block_delta
                    }
                }
            }
        }
        "content_block_delta" => {
            if let Some(delta) = payload.get("delta") {
                match delta.get("type").and_then(Value::as_str) {
                    Some("text_delta") => {
                        if let Some(text) = delta.get("text").and_then(Value::as_str) {
                            let _ = stream_tx
                                .send(Ok(StreamChunk {
                                    index: *index,
                                    delta: ChunkDelta::Text {
                                        text: text.to_string(),
                                    },
                                }))
                                .await;
                            *index += 1;
                        }
                    }
                    Some("input_json_delta") => {
                        if let Some(partial) = delta.get("partial_json").and_then(Value::as_str) {
                            // Find which tool call this delta belongs to via the
                            // content block index in the payload.
                            let block_index =
                                payload.get("index").and_then(Value::as_u64).unwrap_or(0);

                            // Find the call_id by block index. We track names in
                            // insertion order, so the Nth tool_use block maps to the
                            // Nth entry. Use the last-inserted call_id as fallback.
                            let call_id = tool_input_buffers
                                .keys()
                                .nth(block_index as usize)
                                .cloned()
                                .or_else(|| tool_input_buffers.keys().last().cloned())
                                .unwrap_or_default();

                            if let Some(buf) = tool_input_buffers.get_mut(&call_id) {
                                buf.push_str(partial);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "content_block_stop" => {
            // If a tool_use block just finished, emit the complete input.
            let block_index = payload.get("index").and_then(Value::as_u64).unwrap_or(0);
            let call_id = tool_input_buffers.keys().nth(block_index as usize).cloned();

            if let Some(call_id) = call_id {
                if let Some(json_buf) = tool_input_buffers.remove(&call_id) {
                    let input: Value = if json_buf.is_empty() {
                        json!({})
                    } else {
                        serde_json::from_str(&json_buf).unwrap_or(json!({}))
                    };

                    let _ = stream_tx
                        .send(Ok(StreamChunk {
                            index: *index,
                            delta: ChunkDelta::ToolUseInput { call_id, input },
                        }))
                        .await;
                    *index += 1;
                }
            }
        }
        "message_delta" => {
            // message_delta carries stop_reason and final usage.
            if let Some(delta) = payload.get("delta") {
                let stop = delta.get("stop_reason").and_then(Value::as_str);
                if stop == Some("tool_use") {
                    *saw_tool_call = true;
                }
            }
        }
        "message_stop" => {
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
        "error" => {
            let message = payload
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Anthropic streaming error");
            return Err(LlmError::ProviderError {
                status: 500,
                message: message.to_string(),
                retryable: false,
            });
        }
        _ => {
            // message_start, ping, etc. — ignored.
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

async fn parse_json_response(response: Response) -> Result<Value, LlmError> {
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
                format!("Anthropic request failed with status {status}")
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

fn retry_delay(attempt: u32) -> Duration {
    let factor = 1u64 << attempt.min(3);
    Duration::from_millis(100 * factor)
}

fn normalize_request_error(timeout_ms: u64, error: reqwest::Error) -> LlmError {
    if error.is_timeout() {
        LlmError::Network(format!("Anthropic request timed out after {timeout_ms}ms"))
    } else {
        LlmError::Network(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_schema::ToolResult;

    #[test]
    fn build_messages_extracts_system_to_top_level() {
        let messages = vec![
            ChatMessage::System {
                content: "You are helpful.".to_string(),
            },
            ChatMessage::User {
                content: Value::String("Hello".to_string()),
            },
        ];
        let result = build_messages(&messages);
        // System messages should be excluded from the messages array.
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("role").and_then(Value::as_str), Some("user"));
    }

    #[test]
    fn build_messages_merges_consecutive_tool_results() {
        let messages = vec![
            ChatMessage::User {
                content: Value::String("Use tools".to_string()),
            },
            ChatMessage::Assistant {
                content: Value::String("I'll use both tools.".to_string()),
            },
            ChatMessage::Tool {
                result: ToolResult::success("call_1", json!("result 1")),
            },
            ChatMessage::Tool {
                result: ToolResult::success("call_2", json!("result 2")),
            },
        ];
        let result = build_messages(&messages);

        // user, assistant, then ONE user message containing both tool_results
        assert_eq!(result.len(), 3);

        let tool_msg = &result[2];
        assert_eq!(tool_msg.get("role").and_then(Value::as_str), Some("user"));
        let content = tool_msg.get("content").and_then(Value::as_array).unwrap();
        assert_eq!(content.len(), 2);
        assert_eq!(
            content[0].get("tool_use_id").and_then(Value::as_str),
            Some("call_1")
        );
        assert_eq!(
            content[1].get("tool_use_id").and_then(Value::as_str),
            Some("call_2")
        );
    }

    #[test]
    fn build_request_body_includes_required_fields() {
        let config = ProviderConfig {
            provider_kind: ProviderKind::ClaudeSubscription,
            model_id: "claude-sonnet-4-6".to_string(),
            ..Default::default()
        };
        let provider = ClaudeSubscriptionProvider::new(config).unwrap();
        let request = CompletionRequest {
            messages: vec![ChatMessage::User {
                content: Value::String("Hello".to_string()),
            }],
            system: Some("Be helpful".to_string()),
            max_tokens: Some(1024),
            ..Default::default()
        };

        let body = provider.build_request_body(&request, false);

        assert_eq!(
            body.get("model").and_then(Value::as_str),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(body.get("max_tokens").and_then(Value::as_u64), Some(1024));
        assert_eq!(body.get("stream").and_then(Value::as_bool), Some(false));
        assert_eq!(
            body.get("system").and_then(Value::as_str),
            Some("Be helpful")
        );
        assert!(body.get("messages").and_then(Value::as_array).is_some());
    }

    #[test]
    fn build_request_body_defaults_max_tokens() {
        let config = ProviderConfig {
            provider_kind: ProviderKind::ClaudeSubscription,
            model_id: "claude-sonnet-4-6".to_string(),
            ..Default::default()
        };
        let provider = ClaudeSubscriptionProvider::new(config).unwrap();
        let request = CompletionRequest {
            messages: vec![ChatMessage::User {
                content: Value::String("Hi".to_string()),
            }],
            ..Default::default()
        };

        let body = provider.build_request_body(&request, false);
        assert_eq!(
            body.get("max_tokens").and_then(Value::as_u64),
            Some(DEFAULT_MAX_TOKENS as u64)
        );
    }

    #[test]
    fn build_request_body_includes_tools() {
        let config = ProviderConfig {
            provider_kind: ProviderKind::ClaudeSubscription,
            model_id: "claude-sonnet-4-6".to_string(),
            ..Default::default()
        };
        let provider = ClaudeSubscriptionProvider::new(config).unwrap();
        let request = CompletionRequest {
            messages: vec![ChatMessage::User {
                content: Value::String("weather?".to_string()),
            }],
            tools: Some(vec![ToolDefinition {
                name: "get_weather".to_string(),
                description: "Get weather".to_string(),
                input_schema: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
            }]),
            ..Default::default()
        };

        let body = provider.build_request_body(&request, false);
        let tools = body.get("tools").and_then(Value::as_array).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(
            tools[0].get("name").and_then(Value::as_str),
            Some("get_weather")
        );
    }

    #[test]
    fn normalize_response_parses_text_content() {
        let payload = json!({
            "id": "msg_test",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-6",
            "content": [
                {"type": "text", "text": "Hello, world!"}
            ],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        });

        let response = normalize_completion_response("fallback", payload).unwrap();
        assert_eq!(response.content.len(), 1);
        assert!(
            matches!(&response.content[0], ContentBlock::Text { text } if text == "Hello, world!")
        );
        assert_eq!(response.model_id, "claude-sonnet-4-6");
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 5);
        assert_eq!(response.stop_reason, StopReason::Completed);
    }

    #[test]
    fn normalize_response_parses_tool_use() {
        let payload = json!({
            "id": "msg_test",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-6",
            "content": [
                {"type": "text", "text": "Let me check the weather."},
                {
                    "type": "tool_use",
                    "id": "toolu_abc123",
                    "name": "get_weather",
                    "input": {"city": "Paris"}
                }
            ],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 20, "output_tokens": 15}
        });

        let response = normalize_completion_response("fallback", payload).unwrap();
        assert_eq!(response.content.len(), 2);
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].call_id, "toolu_abc123");
        assert_eq!(response.tool_calls[0].name, "get_weather");
        assert_eq!(response.tool_calls[0].input, json!({"city": "Paris"}));
        assert_eq!(response.stop_reason, StopReason::ToolCall);
    }

    #[test]
    fn normalize_stop_reasons() {
        assert_eq!(
            normalize_stop_reason(Some("end_turn")),
            StopReason::Completed
        );
        assert_eq!(
            normalize_stop_reason(Some("max_tokens")),
            StopReason::MaxTokens
        );
        assert_eq!(
            normalize_stop_reason(Some("tool_use")),
            StopReason::ToolCall
        );
        assert_eq!(
            normalize_stop_reason(Some("stop_sequence")),
            StopReason::Completed
        );
        assert_eq!(normalize_stop_reason(None), StopReason::Completed);
    }

    #[test]
    fn provider_rejects_wrong_kind() {
        let config = ProviderConfig {
            provider_kind: ProviderKind::OpenAi,
            model_id: "gpt-4".to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            ..Default::default()
        };
        let result = ClaudeSubscriptionProvider::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn capabilities_reflect_no_embeddings() {
        let config = ProviderConfig {
            provider_kind: ProviderKind::ClaudeSubscription,
            model_id: "claude-sonnet-4-6".to_string(),
            ..Default::default()
        };
        let provider = ClaudeSubscriptionProvider::new(config).unwrap();
        let caps = provider.capabilities();
        assert!(caps.completion);
        assert!(caps.streaming);
        assert!(caps.tool_calling);
        assert!(!caps.embeddings);
    }

    #[test]
    fn tool_to_anthropic_format() {
        let tool = ToolDefinition {
            name: "search".to_string(),
            description: "Search the web".to_string(),
            input_schema: json!({"type": "object", "properties": {"query": {"type": "string"}}}),
        };
        let result = tool_to_anthropic(&tool);
        assert_eq!(result.get("name").and_then(Value::as_str), Some("search"));
        assert_eq!(
            result.get("description").and_then(Value::as_str),
            Some("Search the web")
        );
        assert!(result.get("input_schema").is_some());
    }
}

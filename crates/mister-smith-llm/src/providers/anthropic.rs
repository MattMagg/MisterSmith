use async_trait::async_trait;
use futures::stream;
use mister_smith_core::LlmError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

use crate::config::{ProviderConfig, ProviderKind};
use crate::provider::{CompletionStream, ModelProvider};
use crate::streaming::{ChunkDelta, StreamChunk};
use crate::tool_schema::ToolCall;
use crate::types::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentBlock, EmbeddingResponse,
    ModelCapabilities, StopReason, Usage,
};

const DEFAULT_ANTHROPIC_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Default)]
struct AnthropicStreamState {
    chunk_index: usize,
    tool_call_ids_by_content_index: HashMap<u64, String>,
    terminal_stop_emitted: bool,
}

fn parse_stream_event(
    event: &serde_json::Value,
    state: &mut AnthropicStreamState,
) -> Vec<StreamChunk> {
    let mut chunks = Vec::new();
    let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match event_type {
        "content_block_delta" => {
            if let Some(delta) = event.get("delta") {
                let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match delta_type {
                    "text_delta" => {
                        if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                            chunks.push(StreamChunk {
                                index: state.chunk_index,
                                delta: ChunkDelta::Text {
                                    text: text.to_string(),
                                },
                            });
                            state.chunk_index += 1;
                        }
                    }
                    "input_json_delta" => {
                        if let Some(partial) = delta.get("partial_json").and_then(|v| v.as_str()) {
                            let block_index =
                                event.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                            let call_id = state
                                .tool_call_ids_by_content_index
                                .get(&block_index)
                                .cloned()
                                .unwrap_or_else(|| format!("tool-{block_index}"));
                            chunks.push(StreamChunk {
                                index: state.chunk_index,
                                delta: ChunkDelta::ToolUseInput {
                                    call_id,
                                    input: serde_json::json!(partial),
                                },
                            });
                            state.chunk_index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        "content_block_start" => {
            if let Some(content_block) = event.get("content_block") {
                let block_type = content_block
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if block_type == "tool_use" {
                    let block_index = event.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                    let call_id = content_block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    state
                        .tool_call_ids_by_content_index
                        .insert(block_index, call_id.clone());

                    let name = content_block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    chunks.push(StreamChunk {
                        index: state.chunk_index,
                        delta: ChunkDelta::ToolUseStart { call_id, name },
                    });
                    state.chunk_index += 1;
                }
            }
        }
        "message_delta" => {
            if let Some(stop_reason_str) = event
                .get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|v| v.as_str())
            {
                let stop_reason = match stop_reason_str {
                    "end_turn" | "stop_sequence" => StopReason::Completed,
                    "max_tokens" => StopReason::MaxTokens,
                    "tool_use" => StopReason::ToolCall,
                    _ => StopReason::Completed,
                };
                state.terminal_stop_emitted = true;
                chunks.push(StreamChunk::stop(state.chunk_index, stop_reason));
                state.chunk_index += 1;
            }
        }
        "message_stop" => {
            // Terminal event — emit a fallback stop if message_delta didn't.
            if !state.terminal_stop_emitted {
                state.terminal_stop_emitted = true;
                chunks.push(StreamChunk::stop(state.chunk_index, StopReason::Completed));
                state.chunk_index += 1;
            }
        }
        _ => {}
    }

    chunks
}

/// Anthropic provider using API-key authentication via the Messages API.
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider from config.
    pub fn new(config: ProviderConfig) -> Result<Self, LlmError> {
        config.validate()?;
        if config.provider_kind != ProviderKind::Anthropic {
            return Err(LlmError::InvalidRequest(format!(
                "AnthropicProvider requires provider_kind 'anthropic', got '{}'",
                config.provider_kind
            )));
        }

        let api_key_env = config
            .api_key_env
            .as_deref()
            .ok_or_else(|| LlmError::Authentication("api_key_env required for Anthropic".into()))?;

        let api_key = std::env::var(api_key_env).map_err(|_| {
            LlmError::Authentication(format!("Environment variable '{}' not set", api_key_env))
        })?;

        let base_url = config
            .api_base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_ANTHROPIC_URL.to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| LlmError::Network(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            config,
            client,
            api_key,
            base_url,
        })
    }

    fn messages_url(&self) -> String {
        format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
    }

    fn build_messages_body(
        &self,
        request: &CompletionRequest,
    ) -> Result<serde_json::Value, LlmError> {
        let mut body = serde_json::json!({
            "model": self.config.model_id,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        // Convert messages — System messages are handled via the top-level system param.
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .filter_map(|msg| match msg {
                ChatMessage::System { .. } => None,
                ChatMessage::User { content } => Some(serde_json::json!({
                    "role": "user",
                    "content": render_content_value(content),
                })),
                ChatMessage::Assistant { content } => Some(serde_json::json!({
                    "role": "assistant",
                    "content": render_content_value(content),
                })),
                ChatMessage::Tool { result } => Some(serde_json::json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": result.call_id,
                        "content": result.output.clone().unwrap_or(serde_json::json!("")),
                    }],
                })),
            })
            .collect();

        body["messages"] = serde_json::json!(messages);

        // System prompt: combine the explicit system param with any System messages.
        let system_parts: Vec<&str> = std::iter::once(request.system.as_deref())
            .flatten()
            .chain(request.messages.iter().filter_map(|msg| {
                if let ChatMessage::System { content } = msg {
                    Some(content.as_str())
                } else {
                    None
                }
            }))
            .collect();

        if !system_parts.is_empty() {
            body["system"] = serde_json::json!(system_parts.join("\n"));
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(stops) = &request.stop_sequences {
            body["stop_sequences"] = serde_json::json!(stops);
        }

        // Tool definitions.
        if let Some(tools) = &request.tools {
            let anthropic_tools: Vec<serde_json::Value> = tools
                .iter()
                .map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "input_schema": tool.input_schema,
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(anthropic_tools);
        }

        Ok(body)
    }

    fn normalize_response(
        &self,
        response: AnthropicMessagesResponse,
    ) -> Result<CompletionResponse, LlmError> {
        let mut content = Vec::new();
        let mut tool_calls = Vec::new();

        for block in &response.content {
            match block.block_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        content.push(ContentBlock::Text { text: text.clone() });
                    }
                }
                "tool_use" => {
                    let call_id = block.id.clone().unwrap_or_default();
                    let name = block.name.clone().unwrap_or_default();
                    let input = block.input.clone().unwrap_or(serde_json::json!({}));
                    content.push(ContentBlock::ToolUse {
                        call_id: call_id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    });
                    tool_calls.push(ToolCall {
                        call_id,
                        name,
                        input,
                    });
                }
                _ => {}
            }
        }

        let stop_reason = match response.stop_reason.as_deref() {
            Some("end_turn") | Some("stop_sequence") => StopReason::Completed,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("tool_use") => StopReason::ToolCall,
            _ => StopReason::ProviderSpecificFallback,
        };

        let usage = Usage::new(response.usage.input_tokens, response.usage.output_tokens);

        Ok(CompletionResponse {
            content,
            model_id: response.model,
            usage,
            stop_reason,
            tool_calls,
        })
    }
}

/// Render a `serde_json::Value` content field to a string for the Anthropic API.
fn render_content_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Internal types for Anthropic API response deserialization
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AnthropicMessagesResponse {
    model: String,
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u64,
    output_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicErrorDetail,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    message: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    error_type: String,
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = self.build_messages_body(&request)?;

        let response = self
            .client
            .post(self.messages_url())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("Anthropic request failed: {e}")))?;

        let status = response.status();

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            return Err(LlmError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            let error_text = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<AnthropicErrorResponse>(&error_text)
                .map(|e| e.error.message)
                .unwrap_or(error_text);
            return Err(LlmError::Authentication(message));
        }

        if !status.is_success() {
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<AnthropicErrorResponse>(&error_text)
                .map(|e| e.error.message)
                .unwrap_or(error_text);
            return Err(LlmError::ProviderError {
                status: status_code,
                message,
                retryable: status.is_server_error(),
            });
        }

        let anthropic_response: AnthropicMessagesResponse = response.json().await.map_err(|e| {
            LlmError::Serialization(format!("Failed to parse Anthropic response: {e}"))
        })?;

        self.normalize_response(anthropic_response)
    }

    fn stream(&self, request: CompletionRequest) -> CompletionStream {
        let body = match self.build_messages_body(&request) {
            Ok(mut body) => {
                body["stream"] = serde_json::json!(true);
                body
            }
            Err(e) => return Box::pin(stream::once(async move { Err(e) })),
        };

        let client = self.client.clone();
        let url = self.messages_url();
        let api_key = self.api_key.clone();

        Box::pin(async_stream::try_stream! {
            let response = client
                .post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| LlmError::Network(format!("Anthropic stream failed: {e}")))?;

            let status_code = response.status();
            if !status_code.is_success() {
                let code = status_code.as_u16();
                let error_text = response.text().await.unwrap_or_default();
                Err(LlmError::ProviderError {
                    status: code,
                    message: error_text,
                    retryable: code >= 500,
                })?;
                // unreachable — the ? above always diverges on Err
                unreachable!();
            }

            let mut stream_state = AnthropicStreamState::default();
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();

            use futures::StreamExt;
            while let Some(chunk_result) = bytes_stream.next().await {
                let chunk = chunk_result.map_err(|e| LlmError::Network(format!("Stream read error: {e}")))?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                // Process complete SSE lines.
                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }

                        if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                            for parsed_chunk in parse_stream_event(&event, &mut stream_state) {
                                yield parsed_chunk;
                            }
                        }
                    }
                }
            }
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stream_event_maps_input_deltas_to_real_tool_use_ids() {
        let mut state = AnthropicStreamState::default();

        let start_event = serde_json::json!({
            "type": "content_block_start",
            "index": 3,
            "content_block": {
                "type": "tool_use",
                "id": "call_123",
                "name": "search_docs"
            }
        });

        let start_chunks = parse_stream_event(&start_event, &mut state);
        assert_eq!(start_chunks.len(), 1);
        assert_eq!(
            start_chunks[0].delta,
            ChunkDelta::ToolUseStart {
                call_id: "call_123".to_string(),
                name: "search_docs".to_string(),
            }
        );

        let delta_event = serde_json::json!({
            "type": "content_block_delta",
            "index": 3,
            "delta": {
                "type": "input_json_delta",
                "partial_json": "{\"query\":\"nats\"}"
            }
        });

        let delta_chunks = parse_stream_event(&delta_event, &mut state);
        assert_eq!(delta_chunks.len(), 1);
        assert_eq!(
            delta_chunks[0].delta,
            ChunkDelta::ToolUseInput {
                call_id: "call_123".to_string(),
                input: serde_json::json!("{\"query\":\"nats\"}"),
            }
        );
    }

    #[test]
    fn parse_stream_event_handles_multi_tool_interleaved_input_deltas() {
        let mut state = AnthropicStreamState::default();

        let events = vec![
            serde_json::json!({
                "type": "content_block_start",
                "index": 1,
                "content_block": {
                    "type": "tool_use",
                    "id": "call_weather",
                    "name": "get_weather"
                }
            }),
            serde_json::json!({
                "type": "content_block_start",
                "index": 2,
                "content_block": {
                    "type": "tool_use",
                    "id": "call_news",
                    "name": "get_news"
                }
            }),
            serde_json::json!({
                "type": "content_block_delta",
                "index": 2,
                "delta": {"type": "input_json_delta", "partial_json": "{\"topic\":"}
            }),
            serde_json::json!({
                "type": "content_block_delta",
                "index": 1,
                "delta": {"type": "input_json_delta", "partial_json": "{\"city\":"}
            }),
            serde_json::json!({
                "type": "content_block_delta",
                "index": 2,
                "delta": {"type": "input_json_delta", "partial_json": "\"rust\"}"}
            }),
            serde_json::json!({
                "type": "content_block_delta",
                "index": 1,
                "delta": {"type": "input_json_delta", "partial_json": "\"nyc\"}"}
            }),
        ];

        let all_chunks: Vec<StreamChunk> = events
            .iter()
            .flat_map(|event| parse_stream_event(event, &mut state))
            .collect();

        let input_chunks: Vec<&StreamChunk> = all_chunks
            .iter()
            .filter(|chunk| matches!(chunk.delta, ChunkDelta::ToolUseInput { .. }))
            .collect();
        assert_eq!(input_chunks.len(), 4);

        assert_eq!(
            input_chunks[0].delta,
            ChunkDelta::ToolUseInput {
                call_id: "call_news".to_string(),
                input: serde_json::json!("{\"topic\":"),
            }
        );
        assert_eq!(
            input_chunks[1].delta,
            ChunkDelta::ToolUseInput {
                call_id: "call_weather".to_string(),
                input: serde_json::json!("{\"city\":"),
            }
        );
        assert_eq!(
            input_chunks[2].delta,
            ChunkDelta::ToolUseInput {
                call_id: "call_news".to_string(),
                input: serde_json::json!("\"rust\"}"),
            }
        );
        assert_eq!(
            input_chunks[3].delta,
            ChunkDelta::ToolUseInput {
                call_id: "call_weather".to_string(),
                input: serde_json::json!("\"nyc\"}"),
            }
        );
    }

    #[test]
    fn message_delta_sets_terminal_stop_emitted() {
        let mut state = AnthropicStreamState::default();
        let event = serde_json::json!({
            "type": "message_delta",
            "delta": { "stop_reason": "max_tokens" }
        });

        let chunks = parse_stream_event(&event, &mut state);
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0].delta,
            ChunkDelta::Stop {
                reason: StopReason::MaxTokens
            }
        );
        assert!(state.terminal_stop_emitted);
    }

    #[test]
    fn message_stop_emits_fallback_when_no_prior_stop() {
        let mut state = AnthropicStreamState::default();

        // message_delta without stop_reason
        let delta = serde_json::json!({ "type": "message_delta", "delta": {} });
        let chunks = parse_stream_event(&delta, &mut state);
        assert!(chunks.is_empty());
        assert!(!state.terminal_stop_emitted);

        // message_stop should emit fallback
        let stop = serde_json::json!({ "type": "message_stop" });
        let chunks = parse_stream_event(&stop, &mut state);
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0].delta,
            ChunkDelta::Stop {
                reason: StopReason::Completed
            }
        );
        assert!(state.terminal_stop_emitted);
    }

    #[test]
    fn message_stop_does_not_duplicate_after_message_delta_stop() {
        let mut state = AnthropicStreamState::default();

        let delta = serde_json::json!({
            "type": "message_delta",
            "delta": { "stop_reason": "end_turn" }
        });
        parse_stream_event(&delta, &mut state);
        assert!(state.terminal_stop_emitted);

        // message_stop should NOT emit another stop
        let stop = serde_json::json!({ "type": "message_stop" });
        let chunks = parse_stream_event(&stop, &mut state);
        assert!(chunks.is_empty());
    }
}

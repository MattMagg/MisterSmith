use async_trait::async_trait;
use futures::stream;
use mister_smith_core::LlmError;

use crate::provider::{CompletionStream, ModelProvider};
use crate::streaming::{ChunkDelta, StreamChunk};
use crate::tool_schema::ToolCall;
use crate::types::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentBlock, EmbeddingResponse,
    ModelCapabilities, StopReason, Usage,
};

/// Deterministic no-network provider used for contract tests and local flows.
#[derive(Debug, Clone)]
pub struct MockProvider {
    model_id: String,
    capabilities: ModelCapabilities,
    embedding_dimensions: usize,
}

impl MockProvider {
    /// Create a mock provider with the given model identifier.
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            capabilities: ModelCapabilities::all(),
            embedding_dimensions: 8,
        }
    }

    /// Override the mock provider capability set.
    pub fn with_capabilities(mut self, capabilities: ModelCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Override the deterministic embedding dimensionality.
    pub fn with_embedding_dimensions(mut self, embedding_dimensions: usize) -> Self {
        self.embedding_dimensions = embedding_dimensions.max(1);
        self
    }

    fn ensure_capability(&self, enabled: bool, capability: &str) -> Result<(), LlmError> {
        if enabled {
            Ok(())
        } else {
            Err(LlmError::UnsupportedCapability {
                capability: capability.to_string(),
                model: self.model_id.clone(),
            })
        }
    }

    fn build_completion(
        &self,
        request: &CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        self.ensure_capability(self.capabilities.completion, "completion")?;

        if request.tools.is_some() {
            self.ensure_capability(self.capabilities.tool_calling, "tool_calling")?;
        }

        if let Some(call) = Self::tool_call_from_metadata(&request.metadata)? {
            let usage = Self::usage_for_request(request, 0);
            return Ok(CompletionResponse {
                content: vec![ContentBlock::ToolUse {
                    call_id: call.call_id.clone(),
                    name: call.name.clone(),
                    input: call.input.clone(),
                }],
                model_id: self.model_id.clone(),
                usage,
                stop_reason: StopReason::ToolCall,
                tool_calls: vec![call],
            });
        }

        let text = Self::text_from_request(request);
        let output_tokens = Self::estimate_tokens(&text);

        Ok(CompletionResponse {
            content: vec![ContentBlock::Text { text: text.clone() }],
            model_id: self.model_id.clone(),
            usage: Self::usage_for_request(request, output_tokens),
            stop_reason: if request
                .max_tokens
                .map(|limit| output_tokens as u32 >= limit)
                .unwrap_or(false)
            {
                StopReason::MaxTokens
            } else {
                StopReason::Completed
            },
            tool_calls: Vec::new(),
        })
    }

    fn build_stream(&self, request: CompletionRequest) -> Vec<Result<StreamChunk, LlmError>> {
        if let Err(error) = self.ensure_capability(self.capabilities.streaming, "streaming") {
            return vec![Err(error)];
        }

        if request.tools.is_some() {
            if let Err(error) =
                self.ensure_capability(self.capabilities.tool_calling, "tool_calling")
            {
                return vec![Err(error)];
            }
        }

        match Self::tool_call_from_metadata(&request.metadata) {
            Ok(Some(call)) => vec![
                Ok(StreamChunk {
                    index: 0,
                    delta: ChunkDelta::ToolUseStart {
                        call_id: call.call_id.clone(),
                        name: call.name,
                    },
                }),
                Ok(StreamChunk {
                    index: 1,
                    delta: ChunkDelta::ToolUseInput {
                        call_id: call.call_id,
                        input: call.input,
                    },
                }),
                Ok(StreamChunk::stop(2, StopReason::ToolCall)),
            ],
            Ok(None) => {
                let text = Self::text_from_request(&request);
                vec![
                    Ok(StreamChunk {
                        index: 0,
                        delta: ChunkDelta::Text { text },
                    }),
                    Ok(StreamChunk::stop(1, StopReason::Completed)),
                ]
            }
            Err(error) => vec![Err(error)],
        }
    }

    fn tool_call_from_metadata(metadata: &serde_json::Value) -> Result<Option<ToolCall>, LlmError> {
        let Some(tool_call) = metadata.get("mock_tool_call") else {
            return Ok(None);
        };

        let name = tool_call
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                LlmError::InvalidRequest("mock_tool_call.name must be a string".into())
            })?;

        let call_id = tool_call
            .get("call_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("mock-call-1");

        let input = tool_call
            .get("input")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        Ok(Some(ToolCall {
            call_id: call_id.to_string(),
            name: name.to_string(),
            input,
        }))
    }

    fn text_from_request(request: &CompletionRequest) -> String {
        if let Some(text) = request
            .metadata
            .get("mock_response_text")
            .and_then(serde_json::Value::as_str)
        {
            return text.to_string();
        }

        if let Some(user_message) = request.messages.iter().rev().find_map(Self::user_content) {
            return format!("mock-response:{user_message}");
        }

        if let Some(system) = request.system.as_deref() {
            return format!("mock-system:{system}");
        }

        "mock-response:empty".to_string()
    }

    fn user_content(message: &ChatMessage) -> Option<String> {
        match message {
            ChatMessage::User { content } => Some(Self::render_json_value(content)),
            _ => None,
        }
    }

    fn render_json_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(text) => text.clone(),
            other => other.to_string(),
        }
    }

    fn usage_for_request(request: &CompletionRequest, output_tokens: u64) -> Usage {
        let mut input = 0;

        if let Some(system) = request.system.as_deref() {
            input += Self::estimate_tokens(system);
        }

        for message in &request.messages {
            input += match message {
                ChatMessage::System { content } => Self::estimate_tokens(content),
                ChatMessage::User { content } | ChatMessage::Assistant { content } => {
                    Self::estimate_tokens(&Self::render_json_value(content))
                }
                ChatMessage::Tool { result } => {
                    let output = result
                        .output
                        .as_ref()
                        .map(Self::render_json_value)
                        .unwrap_or_default();
                    let error = result.error.clone().unwrap_or_default();
                    Self::estimate_tokens(&format!("{}{}{}", result.call_id, output, error))
                }
            };
        }

        Usage::new(input, output_tokens)
    }

    fn estimate_tokens(text: &str) -> u64 {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return 0;
        }

        let word_estimate = trimmed.split_whitespace().count() as u64;
        let char_estimate = (trimmed.chars().count() as u64).div_ceil(4);
        word_estimate.max(char_estimate).max(1)
    }

    fn embed_one(&self, input: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; self.embedding_dimensions];
        if input.is_empty() {
            return embedding;
        }

        for (index, byte) in input.bytes().enumerate() {
            let slot = index % self.embedding_dimensions;
            embedding[slot] += f32::from(byte) / 255.0;
        }

        let normalizer = input.len() as f32;
        for value in &mut embedding {
            *value /= normalizer;
        }

        embedding
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new("mock-default")
    }
}

#[async_trait]
impl ModelProvider for MockProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.build_completion(&request)
    }

    fn stream(&self, request: CompletionRequest) -> CompletionStream {
        Box::pin(stream::iter(self.build_stream(request)))
    }

    async fn embed(&self, input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        self.ensure_capability(self.capabilities.embeddings, "embeddings")?;

        let embeddings = input.iter().map(|item| self.embed_one(item)).collect();
        let usage = Usage::new(
            input.iter().map(|item| Self::estimate_tokens(item)).sum(),
            0,
        );

        Ok(EmbeddingResponse {
            embeddings,
            model_id: self.model_id.clone(),
            usage,
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities
    }
}

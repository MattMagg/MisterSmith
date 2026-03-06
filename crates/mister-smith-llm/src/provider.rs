use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use mister_smith_core::LlmError;

use crate::streaming::StreamChunk;
use crate::types::{CompletionRequest, CompletionResponse, EmbeddingResponse, ModelCapabilities};

/// Boxed completion stream used by all model providers.
pub type CompletionStream =
    Pin<Box<dyn Stream<Item = Result<StreamChunk, LlmError>> + Send + 'static>>;

/// Provider-neutral interface for completion, streaming, embeddings, and tool use.
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Execute a full completion request and return a normalized response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// Start a streaming completion using the normalized streaming surface.
    fn stream(&self, request: CompletionRequest) -> CompletionStream;

    /// Produce embeddings for the given input strings.
    async fn embed(&self, input: Vec<String>) -> Result<EmbeddingResponse, LlmError>;

    /// Return the concrete provider/model identifier used by this adapter.
    fn model_id(&self) -> &str;

    /// Describe the capabilities supported by this provider/model.
    fn capabilities(&self) -> ModelCapabilities;
}

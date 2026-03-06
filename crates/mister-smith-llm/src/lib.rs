//! Provider-neutral LLM contracts and shared types for Mister Smith.
//!
//! Phase 9 introduces a new crate boundary so provider integrations can evolve
//! behind a stable interface. Public call sites consume [`ModelProvider`],
//! provider-neutral request and response types, and the deterministic
//! [`MockProvider`] without depending on vendor-specific payload shapes.

mod app_server;
#[cfg(feature = "claude-subscription")]
pub mod claude_credentials;
mod config;
mod mock;
mod provider;
mod providers;
mod streaming;
mod tool_schema;
mod types;

pub use app_server::{AppServerAccountStatus, ChatGptLoginHandle, CodexAppServerClient};
#[cfg(feature = "claude-subscription")]
pub use claude_credentials::{ClaudeOAuthCredentials, CredentialSource};
pub use config::{ProviderConfig, ProviderKind};
pub use mister_smith_core::LlmError;
pub use mock::MockProvider;
pub use provider::{CompletionStream, ModelProvider};
pub use providers::*;
pub use streaming::{ChunkDelta, StreamChunk};
pub use tool_schema::{ToolCall, ToolDefinition, ToolResult};
pub use types::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentBlock, EmbeddingResponse,
    ModelCapabilities, StopReason, Usage,
};

//! Provider-neutral LLM contracts and shared types for Mister Smith.
//!
//! Phase 9 introduces a new crate boundary so provider integrations can evolve
//! behind a stable interface. Public call sites consume [`ModelProvider`],
//! provider-neutral request and response types, and the deterministic
//! [`MockProvider`] without depending on vendor-specific payload shapes.

mod app_server;
pub mod budget;
#[cfg(feature = "claude-subscription")]
pub mod claude_credentials;
mod config;
pub mod dual_stream;
pub mod health;
mod mock;
pub mod model_event;
mod provider;
mod providers;
pub mod router;
mod streaming;
mod tool_schema;
mod types;

pub use app_server::{AppServerAccountStatus, ChatGptLoginHandle, CodexAppServerClient};
pub use budget::{BudgetEnforcer, BudgetNode, BudgetPolicy, BudgetStore, InMemoryBudgetStore};
#[cfg(feature = "claude-subscription")]
pub use claude_credentials::{ClaudeOAuthCredentials, CredentialSource};
pub use config::{ProviderConfig, ProviderKind};
pub use dual_stream::{DualStreamActor, DualStreamConfig, DualStreamHandle};
pub use health::{CircuitBreaker, CircuitBreakerConfig, CircuitState, HealthStatus};
pub use mister_smith_core::LlmError;
pub use mock::MockProvider;
pub use model_event::{BackpressurePolicy, ModelEvent};
pub use provider::{CompletionStream, ModelProvider};
#[cfg(any(
    feature = "anthropic",
    feature = "openai",
    feature = "openai-chatgpt",
    feature = "claude-subscription"
))]
pub use providers::*;
pub use router::{
    CascadePolicy, CascadeTier, ConfidenceSignal, ModelRouter, RoutingDecision, RoutingHint,
    RoutingPolicy,
};
pub use streaming::{ChunkDelta, StreamChunk};
pub use tool_schema::{ToolCall, ToolDefinition, ToolResult};
pub use types::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentBlock, EmbeddingResponse,
    ModelCapabilities, StopReason, Usage,
};

use serde::{Deserialize, Serialize};

use crate::tool_schema::{ToolCall, ToolDefinition, ToolResult};

/// Caller-provided routing preferences consumed and stripped at the routing boundary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingHint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
}

/// Provider-neutral completion request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Ordered conversation history.
    pub messages: Vec<ChatMessage>,
    /// Optional provider-neutral system prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Optional ToolBus-derived tool definitions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// Optional temperature hint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional output-token limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional provider-neutral stop sequences.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Extra provider-neutral request metadata.
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
    /// Optional routing hints consumed by ModelRouter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_hint: Option<RoutingHint>,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            system: None,
            tools: None,
            temperature: None,
            max_tokens: None,
            stop_sequences: None,
            metadata: default_metadata(),
            routing_hint: None,
        }
    }
}

/// Unified chat message roles consumed by completion requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum ChatMessage {
    /// System-authored instruction content.
    System {
        /// Instruction payload.
        content: String,
    },
    /// Human-authored request content.
    User {
        /// Structured user content.
        content: serde_json::Value,
    },
    /// Prior assistant/model content.
    Assistant {
        /// Structured assistant content.
        content: serde_json::Value,
    },
    /// Tool result re-entering the conversation.
    Tool {
        /// Structured tool execution result.
        result: ToolResult,
    },
}

/// Normalized result of a completion request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Ordered normalized content blocks.
    pub content: Vec<ContentBlock>,
    /// Concrete provider/model identifier used to satisfy the request.
    pub model_id: String,
    /// Provider-normalized usage accounting.
    pub usage: Usage,
    /// Provider-neutral completion reason.
    pub stop_reason: StopReason,
    /// Structured tool calls emitted by the model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

/// Provider-neutral content units returned from completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text model output.
    Text {
        /// Text payload.
        text: String,
    },
    /// Tool-use request embedded in the content stream.
    ToolUse {
        /// Stable tool call identifier.
        call_id: String,
        /// Requested tool name.
        name: String,
        /// Structured tool arguments.
        input: serde_json::Value,
    },
}

/// Embedding response using provider-neutral accounting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Embeddings returned in request order.
    pub embeddings: Vec<Vec<f32>>,
    /// Concrete provider/model identifier used to generate the embeddings.
    pub model_id: String,
    /// Provider-normalized usage accounting.
    pub usage: Usage,
}

/// Provider-normalized usage metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Prompt or input token estimate.
    pub input_tokens: u64,
    /// Completion or output token estimate.
    pub output_tokens: u64,
    /// Aggregate token estimate.
    pub total_tokens: u64,
}

impl Usage {
    /// Construct usage and compute the aggregate total.
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

/// Provider-neutral stop reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The response completed normally.
    #[default]
    Completed,
    /// The provider hit the output-token limit.
    MaxTokens,
    /// The response yielded one or more tool calls.
    ToolCall,
    /// The provider filtered or suppressed content.
    ContentFilter,
    /// The request was cancelled before normal completion.
    Cancelled,
    /// Provider-specific terminal condition that does not fit the normalized set.
    ProviderSpecificFallback,
}

/// Capability flags for a selected provider/model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    /// Supports full completion calls.
    pub completion: bool,
    /// Supports incremental streaming.
    pub streaming: bool,
    /// Supports embeddings.
    pub embeddings: bool,
    /// Supports tool definitions and tool calls.
    pub tool_calling: bool,
}

impl ModelCapabilities {
    /// Return a capability set with all Phase 9 behaviors enabled.
    pub const fn all() -> Self {
        Self {
            completion: true,
            streaming: true,
            embeddings: true,
            tool_calling: true,
        }
    }
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

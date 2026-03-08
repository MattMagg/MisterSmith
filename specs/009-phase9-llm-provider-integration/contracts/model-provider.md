# Contract: Model Provider

## Overview

The `ModelProvider` contract defines the provider-neutral API for completion, streaming, embeddings,
and tool-calling support. Anthropic, OpenAI, and the deterministic `MockProvider` all implement this
contract behind the same public types.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | Keeps the provider contract aligned with existing LLM coordination without importing parser-specific logic. |
| `spec/data-management/message-schemas.md` §5 | Confirms hook-event schemas stay deferred; this contract covers direct provider APIs only. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | Keeps Neural/AI Operations adapters and workflows out of scope. |
| `spec/core-architecture/type-definitions.md` | Anchors unified public types, canonical identifiers, and shared error conventions. |
| `spec/core-architecture/async-patterns.md` | Grounds stream ordering and tool-call types in existing agent-as-tool and ToolBus patterns. |
| `spec/core-architecture/coding-standards.md` | Requires typed errors, explicit unsupported-capability behavior, and test coverage. |

## Public API

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError>;

    fn stream(
        &self,
        request: CompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, LlmError>> + Send>>;

    async fn embed(
        &self,
        input: Vec<String>,
    ) -> Result<EmbeddingResponse, LlmError>;

    fn model_id(&self) -> &str;

    fn capabilities(&self) -> ModelCapabilities;
}
```

## Shared Types

### Request Types

```rust
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub system: Option<String>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub metadata: serde_json::Value,
}
```

### Response Types

```rust
pub struct CompletionResponse {
    pub content: Vec<ContentBlock>,
    pub model_id: String,
    pub usage: Usage,
    pub stop_reason: StopReason,
    pub tool_calls: Vec<ToolCall>,
}

pub struct StreamChunk {
    pub index: usize,
    pub delta: ChunkDelta,
}

pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model_id: String,
    pub usage: Usage,
}
```

### Tool Types

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub input: serde_json::Value,
}

pub struct ToolResult {
    pub call_id: String,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}
```

## Error Contract

All implementations must return the canonical shared error hierarchy:

```rust
pub enum LlmError {
    ProviderError {
        status: u16,
        message: String,
        retryable: bool,
    },
    RateLimited {
        retry_after_secs: Option<u64>,
    },
    Serialization(String),
    Network(String),
    UnsupportedCapability {
        capability: String,
        model: String,
    },
    InvalidRequest(String),
    Authentication(String),
}
```

## Behavioral Requirements

1. Provider-specific request and response payloads remain internal to provider modules.
2. `model_id()` returns the concrete provider/model identifier used to satisfy the request.
3. `capabilities()` describes supported behavior before or alongside execution.
4. `stream()` preserves chunk ordering and emits a terminal stop chunk when supported.
5. Unsupported capabilities return `LlmError::UnsupportedCapability` rather than silently no-oping.
6. `MockProvider` implements the full contract without network access or credentials.

## Router-Provider Relationship

The `ModelRouter` sits above the `ModelProvider` layer. It selects a provider per-request based on
routing policy, provider health, and budget constraints. The `ModelProvider` trait is unchanged —
routing logic lives in the router, not in providers.

```text
Caller -> ModelRouter -> [routing decision] -> ModelProvider -> Provider API
                      -> [health check]     -> HealthStatus / CircuitState
                      -> [budget check]     -> BudgetNode (JetStream KV CAS)
```

### Routing Hints on CompletionRequest

`CompletionRequest` gains an optional `routing_hint: Option<RoutingHint>` field that callers can
use to express preferences (model tier, cost constraint, required capabilities). The `ModelRouter`
consumes and strips this field before forwarding to the selected provider. Providers never see
routing hints.

```rust
pub struct RoutingHint {
    pub preferred_tier: Option<String>,
    pub max_cost_tokens: Option<u64>,
    pub required_capabilities: Vec<String>,
}
```

### Two-Layer Streaming Architecture

Providers emit raw `StreamChunk`/`ChunkDelta` (4 variants) at the provider-to-framework boundary.
The framework's stream actors convert these to canonical `ModelEvent` items (28 variants) for
internal consumption. This is a two-layer design:

- **Layer 1 (Provider boundary)**: `StreamChunk` with `ChunkDelta::Text`, `ToolUseStart`,
  `ToolUseInput`, `Stop` — raw, provider-normalized deltas
- **Layer 2 (Framework internal)**: `ModelEvent` with lifecycle, text, tool-call, observability,
  error, heartbeat, and unknown variants — canonical orchestration events

The `ModelProvider::stream()` method continues to return `Stream<Item = Result<StreamChunk, LlmError>>`.
The conversion to `ModelEvent` happens in the stream actor, not in the provider.

## Validation Requirements

- Unit tests must exercise `complete`, `stream`, `embed`, and tool-calling paths through
  `MockProvider`.
- Anthropic, OpenAI, and Claude subscription real-provider tests must be env-gated.
- Public call sites outside `mister-smith-llm` must not depend on provider-specific structs or
  enums.
- Router tests must verify sub-millisecond routing overhead (excluding provider latency).
- Budget enforcement tests must verify <1% overrun rate under concurrent CAS operations.
- Circuit breaker tests must verify correct state transitions (Closed -> Open -> HalfOpen -> Closed).

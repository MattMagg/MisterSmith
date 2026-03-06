# Design: LLM Provider Integration (Phase 9)

**Date**: 2026-03-05
**Status**: Approved
**Author**: Claude Code + Matt Maggio

---

## Problem

Phases 1-7 built a complete multi-agent orchestration framework: actors, supervision trees, NATS messaging, security, persistence, and agent roles. Phase 8 adds operations and a binary entry point. But no phase connects agents to actual LLMs. Every competitor (OpenAI Agents SDK, Google ADK, CrewAI, LangGraph, Claude Agent SDK) ships something runnable. Mister Smith cannot call a model.

The architecture specs already describe LLM integration patterns extensively:

- `spec/data-management/agent-orchestration.md` §10.4 — `LlmTaskOutputParser`, parallel LLM task coordination, NATS subject routing
- `spec/data-management/message-schemas.md` §5 — Hook event/response JSON schemas for LLM backend communication
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 — Neural/AI Operations domain with 5 specialized agents

This phase implements the foundational layer: the ability to call various models through a unified, model-agnostic interface.

## Approach

**Single new crate: `mister-smith-llm`**

One crate with a `ModelProvider` trait and feature-gated provider implementations. This follows the existing workspace pattern where `mister-smith-transport` defines a `Transport` trait and concrete implementations live behind feature flags.

### Why Not Multiple Crates

Considered and rejected:
- **Separate provider crates** (`mister-smith-llm-claude`, `mister-smith-llm-openai`, etc.) — premature splitting. Provider implementations are ~200-400 lines each. The compile-time cost of having them in one crate behind feature flags is negligible vs. the workspace complexity of 4+ additional crates.
- **Integrate into `mister-smith-agents`** — violates separation of concerns. Not all agent users need LLM deps. The LLM layer is an optional capability, not a core agent requirement.

### Why Not Implement the Full Architecture Spec

The architecture specs describe advanced patterns: hook event systems, `LlmTaskOutputParser` regex routing, Neural/AI Operations domain with 5 specialized agents. Phase 9 deliberately limits scope to the provider abstraction. Advanced patterns (parallel LLM coordination, neural ops agents, hook systems) are deferred to a future phase once the basic provider layer proves out.

## Design

### Core Trait

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Send a completion request and get a full response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;

    /// Stream a completion response token-by-token.
    fn stream(
        &self,
        request: CompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, LlmError>> + Send>>;

    /// Generate embeddings for the given inputs.
    async fn embed(&self, input: Vec<String>) -> Result<EmbeddingResponse, LlmError>;

    /// Provider and model identifier (e.g., "anthropic/claude-sonnet-4-20250514").
    fn model_id(&self) -> &str;

    /// What this model supports (chat, tools, vision, embeddings, streaming).
    fn capabilities(&self) -> ModelCapabilities;
}
```

### Message Types

```rust
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub tools: Option<Vec<ToolDefinition>>,   // JSON Schema tool definitions
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub system: Option<String>,
    pub metadata: HashMap<String, Value>,      // Provider-specific options
}

pub enum ChatMessage {
    System(String),
    User(UserContent),
    Assistant(AssistantContent),
    Tool(ToolResult),
}

pub struct CompletionResponse {
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub usage: Usage,
    pub stop_reason: StopReason,
    pub tool_calls: Vec<ToolCall>,
}

pub enum ContentBlock {
    Text(String),
    ToolUse { id: String, name: String, input: Value },
}

pub struct StreamChunk {
    pub delta: ChunkDelta,
    pub index: usize,
}

pub enum ChunkDelta {
    Text(String),
    ToolUseStart { id: String, name: String },
    ToolUseInput(String),  // Partial JSON
    Stop(StopReason),
}

pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub usage: Usage,
}
```

### Tool Calling Bridge

The `ToolDefinition` type maps bidirectionally to the `ToolBus` in `mister-smith-agents`:

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,  // JSON Schema
}

// Bridge functions (in mister-smith-agents, behind `llm` feature flag)
impl ToolBus {
    pub fn to_tool_definitions(&self) -> Vec<ToolDefinition>;
    pub async fn execute_tool_call(&self, call: &ToolCall) -> Result<ToolResult, AgentSystemError>;
}
```

### Feature Flags

```toml
[features]
default = []
anthropic = ["dep:reqwest"]
openai = ["dep:reqwest"]
google = ["dep:reqwest"]
ollama = ["dep:reqwest"]
all-providers = ["anthropic", "openai", "google", "ollama"]
```

No provider enabled by default. Users opt in to exactly what they need.

### Provider Implementations

Each provider is a struct implementing `ModelProvider`:

| Provider | Struct | HTTP Client | Auth |
|----------|--------|-------------|------|
| Anthropic | `AnthropicProvider` | reqwest | `ANTHROPIC_API_KEY` header |
| OpenAI | `OpenAiProvider` | reqwest | `Authorization: Bearer` |
| Google | `GoogleProvider` | reqwest | `GOOGLE_API_KEY` query param |
| Ollama | `OllamaProvider` | reqwest | None (local) |

Providers handle:
- Request serialization to provider-specific JSON formats
- Response deserialization back to unified types
- Streaming via SSE parsing (Anthropic/OpenAI) or chunked transfer (Ollama)
- Rate limit headers → backoff signals
- Error mapping to `LlmError` variants

### Error Types

```rust
pub enum LlmError {
    /// Provider returned an error response (status code, message).
    ProviderError { status: u16, message: String, retryable: bool },
    /// Rate limited — includes retry-after hint.
    RateLimited { retry_after: Option<Duration> },
    /// Request/response serialization failure.
    Serialization(String),
    /// Network-level failure.
    Network(reqwest::Error),
    /// Model doesn't support requested capability.
    UnsupportedCapability { capability: String, model: String },
    /// Invalid request (bad parameters, too many tokens, etc.).
    InvalidRequest(String),
    /// Authentication failure.
    AuthenticationError(String),
}
```

`LlmError` is defined in `mister-smith-core` (following the SecurityError and PersistenceError pattern) and re-exported from the llm crate.

### Crate Structure

```
crates/mister-smith-llm/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Re-exports, crate docs
│   ├── provider.rs         # ModelProvider trait
│   ├── types.rs            # CompletionRequest, CompletionResponse, ChatMessage, etc.
│   ├── tool_schema.rs      # ToolDefinition, ToolCall, ToolResult
│   ├── streaming.rs        # StreamChunk, ChunkDelta, SSE parser utilities
│   ├── config.rs           # ProviderConfig, model defaults
│   ├── mock.rs             # MockProvider for testing (always available, no feature flag)
│   └── providers/
│       ├── mod.rs
│       ├── anthropic.rs    # #[cfg(feature = "anthropic")]
│       ├── openai.rs       # #[cfg(feature = "openai")]
│       ├── google.rs       # #[cfg(feature = "google")]
│       └── ollama.rs       # #[cfg(feature = "ollama")]
└── tests/
    ├── mock_tests.rs       # Unit tests against MockProvider
    ├── types_tests.rs      # Serialization roundtrip tests
    └── integration/        # Env-gated tests requiring real API keys
        ├── anthropic_tests.rs
        └── openai_tests.rs
```

### Agent Integration (Phase 9.4)

The `mister-smith-agents` crate gains an optional `llm` feature flag:

```toml
# crates/mister-smith-agents/Cargo.toml
[features]
llm = ["dep:mister-smith-llm"]
```

This enables:
1. `AgentRuntime` can hold an `Arc<dyn ModelProvider>`
2. Agent roles (Planner, Critic, Executor) gain `with_model()` constructors
3. `ToolBus` gains `to_tool_definitions()` and `execute_tool_call()` methods
4. The orchestrator can wire model calls into the decompose → assign → aggregate flow

### Testing Strategy

- **Unit tests**: MockProvider validates the full trait contract (complete, stream, embed, tool calling). Always runs, no API keys needed.
- **Serialization tests**: Round-trip CompletionRequest/Response through JSON for each provider format.
- **Integration tests**: `#[ignore]` by default, gated on `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` env vars. Validates real API calls work end-to-end.
- **CI**: Unit + serialization tests run in CI. Integration tests run manually or on a schedule.

## Subphases

### 9.1: Core Types + MockProvider

- `LlmError` in mister-smith-core
- `ModelProvider` trait, all message types, `MockProvider`
- Crate skeleton with tests

**Depends on**: Phase 1 (core types)
**Produces**: Compilable crate, MockProvider passing all trait tests

### 9.2: Anthropic Provider

- `AnthropicProvider` implementing `ModelProvider`
- Messages API (completions + streaming)
- Tool use support (beta header)
- Embeddings (via Voyage or native)

**Depends on**: 9.1
**Produces**: Working Claude integration

### 9.3: OpenAI Provider

- `OpenAiProvider` implementing `ModelProvider`
- Chat Completions API (completions + streaming)
- Function calling / tool use
- Embeddings API

**Depends on**: 9.1
**Produces**: Working GPT integration

### 9.4: Agent-LLM Bridge

- `llm` feature flag in mister-smith-agents
- `AgentRuntime::with_model()` constructor
- Planner, Critic, Executor roles gain LLM-powered implementations
- `ToolBus` ↔ `ToolDefinition` bridge

**Depends on**: 9.1, Phase 7 (agents)
**Produces**: Agent roles that can call real models

### 9.5: Tool Calling Bridge

- `ToolBus::to_tool_definitions()` exports registered tools as JSON Schema
- `ToolBus::execute_tool_call()` dispatches LLM tool calls to registered handlers
- Round-trip test: model requests tool → ToolBus executes → result returns to model

**Depends on**: 9.2 or 9.3 (needs a real provider), 9.4
**Produces**: End-to-end tool calling

## Gate 9

An agent role (Planner) receives a task description, calls a real LLM via `ModelProvider`, gets a structured response containing subtask decomposition, and the Orchestrator assigns those subtasks to Worker agents. The same flow works with at least 2 different providers (Anthropic + OpenAI). Tool calling round-trips through the ToolBus.

**References**:
- [agent-orchestration.md](spec/data-management/agent-orchestration.md) §10.4 — LLM task coordination patterns
- [message-schemas.md](spec/data-management/message-schemas.md) §5 — Hook event schemas (deferred, not implemented in Phase 9)
- [SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) §15 — Neural/AI Ops domain (deferred)

## Out of Scope

- **Hook event system** (NATS `llm.hooks.*` subjects) — architecture spec pattern, deferred
- **LlmTaskOutputParser** regex routing — deferred
- **Neural/AI Operations domain** (5 specialized agents) — deferred
- **Prompt engineering framework** — not a framework responsibility
- **RAG pipeline** — just the embedding primitive, not retrieval
- **Guardrails/safety layer** — potential future phase
- **Ollama/Google providers** — can be added later; Anthropic + OpenAI are the MVP

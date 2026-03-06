# Data Model: Phase 9 — LLM Provider Integration

**Date**: 2026-03-06
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | Grounds `AgentLlmBinding`, Planner decomposition flow, and the ToolBus round-trip states in existing orchestration boundaries. |
| `spec/data-management/message-schemas.md` §5 | Confirms hook-event payloads stay deferred; this model covers provider and ToolBus surfaces only. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | Keeps Neural/AI Operations entities out of scope for this phase. |
| `spec/core-architecture/type-definitions.md` | Constrains `AgentType`, canonical IDs, and shared error/result conventions used by the model. |
| `spec/core-architecture/async-patterns.md` | Anchors `ToolDefinition`, `ToolCall`, `ToolResult`, and `StreamChunk` to agent-as-tool and ToolBus patterns. |
| `spec/core-architecture/coding-standards.md` | Keeps validation rules tied to typed errors, timeout enforcement, permission checks, and explicit testability. |

## Entities

### ProviderConfig

Runtime configuration for a selected model provider.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| provider_kind | `String` | Required, enum-like (`anthropic`, `openai`, `mock`) | Selects the backend adapter |
| model_id | `String` | Required | Canonical provider/model identifier surfaced through the shared contract |
| api_base_url | `Option<String>` | Optional | Override for provider endpoint or local proxy |
| api_key_env | `Option<String>` | Optional for `mock`, required for real providers | Environment variable name holding credentials |
| timeout_ms | `u64` | Default 30000 | Upper bound for provider calls |
| max_retries | `u32` | Default 0 | Retry budget for retryable provider failures |
| metadata | `serde_json::Value` | Optional object | Provider-specific configuration that must stay out of the public request types |

**Invariant**: Provider-specific configuration is allowed here, but public call sites outside
provider modules must continue using unified request and response types.

---

### CompletionRequest

Provider-neutral request for completion, streaming, and tool-use workflows.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| messages | `Vec<ChatMessage>` | Non-empty | Ordered conversation history |
| system | `Option<String>` | Optional | Provider-neutral system prompt |
| tools | `Option<Vec<ToolDefinition>>` | Optional | Tool definitions exported from the ToolBus |
| temperature | `Option<f32>` | Optional | Generation control |
| max_tokens | `Option<u32>` | Optional | Output token limit |
| stop_sequences | `Option<Vec<String>>` | Optional | Provider-neutral stop conditions |
| metadata | `serde_json::Value` | Optional object | Extra request hints that remain provider-neutral at the call site |

**Invariant**: `messages` preserve caller ordering. Provider adapters may reshape the payload
internally, but the public request structure stays stable.

---

### ChatMessage

Conversation item consumed by `CompletionRequest`.

| Variant | Payload | Description |
| --------- | --------- | ------------- |
| `System` | `String` | System-level instruction |
| `User` | `serde_json::Value` or typed user content | Human request content |
| `Assistant` | `serde_json::Value` or typed assistant content | Prior assistant/model content |
| `Tool` | `ToolResult` | Structured result returned from ToolBus execution |

**Invariant**: Tool results re-enter the conversation through this unified type rather than through
provider-specific message formats.

---

### CompletionResponse

Normalized result of a provider completion call.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| content | `Vec<ContentBlock>` | Required | Ordered normalized output blocks |
| model_id | `String` | Required | Provider/model that produced the response |
| usage | `Usage` | Required | Token or request accounting |
| stop_reason | `StopReason` | Required | Provider-neutral completion reason |
| tool_calls | `Vec<ToolCall>` | Optional | Structured model-emitted tool requests |

**Invariant**: Provider-native stop reasons and usage formats must normalize here before leaving the
provider module.

---

### StreamChunk

Incremental output item emitted by `ModelProvider::stream()`.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| index | `usize` | Required, monotonic | Stable ordering for chunk assembly |
| delta | `ChunkDelta` | Required | Partial text, tool-call, or stop signal |

**Invariant**: Chunks must preserve ordering exactly as emitted by the provider adapter.

---

### ToolDefinition

Provider-neutral description of a callable tool exported from the ToolBus.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| name | `String` | Required, stable | Tool identifier presented to the model |
| description | `String` | Required | Human-readable purpose |
| input_schema | `serde_json::Value` | Required, JSON Schema object | Structured arguments contract |

**Invariant**: Definitions are derived from the ToolBus registry rather than reauthored in provider
modules.

---

### ToolCall

Model-emitted request to invoke a tool.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| call_id | `String` | Required | Stable identifier for matching results |
| name | `String` | Required | Requested tool name |
| input | `serde_json::Value` | Required object | Arguments to validate and dispatch through ToolBus |

---

### ToolResult

Structured result returned from ToolBus execution back into the LLM flow.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| call_id | `String` | Required | Matches the originating `ToolCall` |
| output | `serde_json::Value` | Optional on failure | Structured tool output payload |
| error | `Option<String>` | Optional | Typed failure description surfaced through bridge logic |

**Invariant**: Permission, timeout, and audit failures remain ToolBus-governed errors even when the
call originates from a model.

---

### ModelCapabilities

Describes which provider or model features are supported.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| completion | `bool` | Required | Supports full completion calls |
| streaming | `bool` | Required | Supports incremental streaming |
| embeddings | `bool` | Required | Supports embeddings |
| tool_calling | `bool` | Required | Supports tool definitions and tool calls |

**Invariant**: Unsupported capabilities must surface as typed errors rather than implicit no-ops.

---

### AgentLlmBinding

Feature-gated binding between an agent role and a selected provider.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| agent_type | `AgentType` | Required, limited to Planner/Critic/Executor in Phase 9 | Identifies the role gaining provider-backed behavior |
| provider_kind | `String` | Required | Selected provider implementation |
| model_id | `String` | Required | Model used by the role |
| tool_access | `bool` | Default `false` | Whether tool definitions and tool execution are enabled |
| feature_flag | `String` | Const `llm` | Guards direct dependency on `mister-smith-llm` |

## Supporting Value Types

### Usage

| Field | Type | Description |
| ------- | ------ | ------------- |
| input_tokens | `u64` | Provider-normalized prompt or input usage |
| output_tokens | `u64` | Provider-normalized completion usage |
| total_tokens | `u64` | Aggregate usage when exposed by the provider |

### StopReason

Canonical reasons for response completion:

```text
Completed
MaxTokens
ToolCall
ContentFilter
Cancelled
ProviderSpecificFallback
```

### ContentBlock

Canonical response content units:

```text
Text
ToolUse
```

### ChunkDelta

Canonical streaming deltas:

```text
Text
ToolUseStart
ToolUseInput
Stop
```

## Relationships

```text
ProviderConfig 1──1 ModelCapabilities
ProviderConfig 1──* CompletionRequest
CompletionRequest 1──* ChatMessage
CompletionRequest 0──* ToolDefinition
CompletionResponse 1──* ContentBlock
CompletionResponse 0──* ToolCall
ToolCall 0──1 ToolResult
AgentLlmBinding *──1 ProviderConfig
AgentLlmBinding *──1 ModelCapabilities
```

## State Transitions

### Tool Call Round-Trip

```text
Declared -> Requested -> Authorized -> Executed -> Returned
                  \-> Rejected -> Error
                  \-> TimedOut -> Error
```

### Planner Decomposition Flow

```text
TaskInput -> CompletionRequest -> CompletionResponse -> StructuredSubtasks -> OrchestratorAssignment
```

## Validation Rules

1. Provider-specific request and response formats stay internal to provider modules.
2. `ToolDefinition.input_schema` must remain valid JSON Schema after ToolBus export.
3. `ToolResult.call_id` must match the originating `ToolCall.call_id`.
4. `AgentLlmBinding.agent_type` is limited to Planner, Critic, and Executor for Phase 9.
5. Missing or unsupported provider capabilities must return typed `LlmError` values.

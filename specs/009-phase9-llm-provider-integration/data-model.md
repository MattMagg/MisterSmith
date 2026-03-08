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
| provider_kind | `ProviderKind` | Required (`anthropic`, `openai`, `openai_chatgpt`, `claude_subscription`, `mock`) | Selects the backend adapter |
| model_id | `String` | Required | Canonical provider/model identifier surfaced through the shared contract |
| api_base_url | `Option<String>` | Optional | Override for provider endpoint or local proxy |
| api_key_env | `Option<String>` | Optional for `mock` and `openai_chatgpt`, required for API-key providers | Environment variable name holding credentials |
| timeout_ms | `u64` | Default 30000 | Upper bound for provider calls |
| max_retries | `u32` | Default 0 | Retry budget for retryable provider failures |
| metadata | `serde_json::Value` | Optional object | Provider-specific configuration that stays out of public request types; `openai_chatgpt` may use it for app-server hints |

**Invariant**: Provider-specific configuration is allowed here, but public call sites outside
provider modules must continue using unified request and response types.

**Invariant**: `openai_chatgpt` auth state is owned by Codex app-server rather than persisted by
Mister Smith.

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
| routing_hint | `Option<RoutingHint>` | Optional | Caller-provided routing preferences (model tier, budget constraint) consumed by `ModelRouter` |

**Invariant**: `messages` preserve caller ordering. Provider adapters may reshape the payload
internally, but the public request structure stays stable.

**Invariant**: `routing_hint` is consumed by the `ModelRouter` before the request reaches a
provider. Providers never see `routing_hint` — it is stripped at the routing boundary.

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

**Invariant**: `OpenAiChatGptProvider` reports `embeddings = false` and `tool_calling = false` in
this phase and surfaces those requests through `LlmError::UnsupportedCapability`.

---

### AppServerAccountStatus

Normalized view of the Codex app-server account state used by `auth openai-chatgpt status` and by
`OpenAiChatGptProvider` before attempting a turn.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| backend | `String` | Const `openai_chatgpt` | Identifies the ChatGPT-backed provider path |
| account_type | `Option<String>` | Optional (`chatgpt`, `apiKey`, or future Codex account types) | Raw account mode surfaced by Codex `account/read` |
| authenticated | `bool` | Required | Whether Codex app-server currently has any active OpenAI-family account configured |
| email | `Option<String>` | Optional | ChatGPT account email when available from app-server |
| plan_type | `Option<String>` | Optional | Normalized subscription plan reported by app-server |
| requires_openai_auth | `bool` | Required | Mirrors app-server `account/read`; indicates whether the active provider requires OpenAI auth, not whether login succeeded |

**Invariant**: This is operational status for login and readiness checks. It is not a second source
of truth for ChatGPT credentials. `OpenAiChatGptProvider` treats `account_type = "chatgpt"` as the
readiness signal for the ChatGPT-subscription backend and must not misread
`requires_openai_auth = true` as a failed login.

**Invariant**: `account_type = None` with `requires_openai_auth = false` is a distinct operational
state meaning the active Codex provider does not currently require OpenAI authentication. The app
status command must report that state explicitly instead of collapsing it into "login required."

---

### AgentLlmBinding

Feature-gated binding between an agent role and a selected provider.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| agent_type | `AgentType` | Required, limited to Planner/Critic/Executor in Phase 9 | Identifies the role gaining provider-backed behavior |
| provider_kind | `ProviderKind` | Required | Selected provider implementation |
| model_id | `String` | Required | Model used by the role |
| tool_access | `bool` | Default `false` | Whether tool definitions and tool execution are enabled |
| feature_flag | `String` | Const `llm` | Guards direct dependency on `mister-smith-llm` |

---

### MessagePlane

Classifies whether a `MessageEnvelope` carries data-plane or control-plane traffic. Added to
`MessageEnvelope` in `mister-smith-transport` as `Option<MessagePlane>` with `#[serde(default)]`.

```text
#[non_exhaustive]
Data      — Request-reply, streaming, tool calls (microsecond latency budget)
Control   — Configuration updates, health telemetry, budget state (JetStream KV watches)
```

**Invariant**: `None` is treated as `Data` for backward compatibility with pre-Phase-9 messages.

**Invariant**: Data-plane messages use NATS request-reply or Core pub/sub. Control-plane messages
use JetStream KV watches and durable consumers.

---

### StreamClass

Classifies whether a stream event requires lossless or best-effort delivery. Added to
`MessageEnvelope` in `mister-smith-transport` as `Option<StreamClass>` with `#[serde(default)]`.

```text
#[non_exhaustive]
Semantic  — Lossless delivery via JetStream (tool calls, lifecycle, errors, finalization)
Ui        — Best-effort delivery via NATS Core (text deltas, heartbeats, progress indicators)
```

**Invariant**: `None` is treated as `Semantic` (safe default — lossless until explicitly opted out).

---

### ModelEvent

Canonical internal event type emitted by stream actors after converting raw `StreamChunk` items
from providers. This is the framework's event contract — consumers receive `ModelEvent`, not
`StreamChunk`.

```text
#[non_exhaustive]
// Lifecycle (5)
StreamStarted { model_id: String, request_id: String }
StreamCompleted { usage: Usage, stop_reason: StopReason }
StreamFailed { error: String, recoverable: bool }
StreamCancelled { reason: String }
StreamResumed { from_checkpoint: String }

// Text (3)
TextDelta { text: String }
TextCompleted { full_text: String }
TextAnnotation { annotation: serde_json::Value }

// Tool Call (4)
ToolCallStart { call_id: String, name: String }
ToolCallDelta { call_id: String, input_chunk: String }
ToolCallCompleted { call_id: String, name: String, input: serde_json::Value }
ToolResult { call_id: String, result: serde_json::Value, error: Option<String> }

// Observability (3)
UsageUpdate { usage: Usage }
LatencyMarker { checkpoint: String, elapsed_ms: u64 }
RoutingDecision { model_id: String, tier: String, reason: String }

// Error (1)
Error { code: String, message: String, recoverable: bool }

// Heartbeat (1)
Heartbeat { sequence: u64 }

// Forward compatibility (1)
#[serde(other)]
Unknown
```

**Invariant**: `StreamChunk`/`ChunkDelta` (4 variants) remain the raw provider-to-framework
boundary. `ModelEvent` (28 variants) is the canonical internal event type. These are two layers,
not a replacement. Providers emit `StreamChunk`; the framework's stream actors convert them to
`ModelEvent`.

**Invariant**: The `Unknown` variant with `#[serde(other)]` ensures forward compatibility when
providers emit event types not yet in the enum.

---

### ModelRouter

Data-plane router that selects a provider per-request based on routing policy, health, and budget.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| providers | `Vec<ProviderConfig>` | Non-empty | Available provider configurations |
| routing_policy | `RoutingPolicy` | Required | How to select among providers |
| health_table | `HashMap<String, HealthStatus>` | In-memory, refreshed by KV watch | Per-provider health snapshot |
| budget_root | `Option<String>` | Optional | JetStream KV key prefix for budget hierarchy |

**Invariant**: The `ModelRouter` wraps one or more `ModelProvider` instances. It implements the
routing decision in the data plane using local in-memory state. It does not call JetStream for
per-request routing — only for budget CAS operations and health updates.

---

### RoutingPolicy

Configuration for how the `ModelRouter` selects providers.

```text
#[non_exhaustive]
RoundRobin          — Rotate across healthy providers
CostOptimized       — Route to cheapest healthy provider meeting capability requirements
CapabilityMatched   — Route based on model capability matching
Cascade(CascadePolicy) — Multi-tier escalation (SLM-default, LLM-fallback)
```

---

### CascadePolicy

Multi-tier routing configuration for SLM-default / LLM-fallback economics.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| tiers | `Vec<CascadeTier>` | Non-empty, ordered cheapest-first | Model tiers to attempt in order |
| escalation_threshold | `f32` | 0.0-1.0 | Minimum confidence score to accept a tier's response |
| max_escalations | `u32` | Default 1 | Maximum number of tier escalations per request |

**Invariant**: Tiers are attempted in order. The first tier whose response meets the
`escalation_threshold` is accepted. If all tiers are exhausted, the final tier's response is
returned regardless of confidence.

---

### CascadeTier

A single tier within a `CascadePolicy`.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| provider_config | `ProviderConfig` | Required | Provider for this tier |
| label | `String` | Required | Human-readable tier name (e.g., "slm-7b", "llm-gpt4o") |

---

### ConfidenceSignal

Structured signal indicating routing confidence, used by cascade policies.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| score | `f32` | 0.0-1.0 | Overall confidence in the response quality |
| source | `String` | Required | What produced the signal (e.g., "heuristic", "prm", "logprob") |
| metadata | `serde_json::Value` | Optional | Additional signal-specific data |

**Invariant**: In Phase 9, confidence signals are heuristic-based (response length, stop reason,
capability match). PRM-based signals are Phase 10.

---

### BudgetNode

Hierarchical budget entry stored in JetStream KV.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| key | `String` | Required, unique, slash-delimited hierarchy | KV key (e.g., "budget/org1/team-alpha/user-42") |
| limit_tokens | `u64` | Required | Total token budget for this period |
| used_tokens | `u64` | Required, CAS-updated | Tokens consumed so far |
| period | `String` | Required | Budget period identifier (e.g., "2026-03-daily", "2026-03-monthly") |
| policy | `BudgetPolicy` | Required | Enforcement behavior when budget is approached or exhausted |

**Invariant**: Budget updates use JetStream KV CAS (compare-and-swap) to prevent concurrent
overruns. The reserve-before-send pattern estimates token cost, reserves via CAS, then reconciles
actual usage after completion.

---

### BudgetPolicy

Budget enforcement behavior.

```text
#[non_exhaustive]
HardCap         — Reject requests when budget is exhausted
SoftCap         — Downgrade to cheaper model when budget is low
Conditioned     — Route to progressively cheaper models as budget depletes
```

---

### HealthStatus

Per-provider health snapshot maintained in the data-plane routing table.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| provider_id | `String` | Required | Provider identifier |
| circuit_state | `CircuitState` | Required | Current circuit breaker state |
| consecutive_failures | `u32` | Required | Sequential failure count |
| rolling_error_rate | `f64` | 0.0-1.0 | Error rate over sliding window |
| p95_latency_ms | `u64` | Required | 95th percentile response latency |
| last_success | `Option<u64>` | Epoch ms | Timestamp of last successful response |
| rate_limit_until | `Option<u64>` | Epoch ms | Retry-After deadline from 429 responses |

**Invariant**: Health status is updated passively from proxied traffic (circuit breaker pattern).
No active health check probes in Phase 9 — passive monitoring only.

---

### CircuitState

Circuit breaker state machine for provider health.

```text
#[non_exhaustive]
Closed    — Provider is healthy, requests flow normally
Open      — Provider is unhealthy, requests are rejected or routed elsewhere
HalfOpen  — Testing recovery with a single probe request
```

---

### BackpressurePolicy

Per-event-class backpressure behavior for the dual-stream architecture.

```text
#[non_exhaustive]
Lossless    — Must deliver; apply backpressure to sender (JetStream ack)
Coalescible — May merge consecutive events of same type under pressure
Droppable   — May drop under extreme pressure (heartbeats, progress indicators)
```

**Backpressure policy matrix**:

| Event Class | Policy | Stream | Rationale |
| ----------- | ------ | ------ | --------- |
| `ToolCallStart`, `ToolCallDelta`, `ToolCallCompleted` | Lossless | Semantic | Tool calls must never be lost |
| `StreamStarted`, `StreamCompleted`, `StreamFailed` | Lossless | Semantic | Lifecycle events are critical |
| `Error` | Lossless | Semantic | Errors must be delivered |
| `TextDelta` | Coalescible | UI | Consecutive text deltas can merge |
| `UsageUpdate`, `LatencyMarker`, `RoutingDecision` | Coalescible | Semantic | Observability events can merge |
| `Heartbeat` | Droppable | UI | Missing heartbeats are tolerable |
| `Unknown` | Coalescible | UI | Unknown events get best-effort |

---

### RoutingHint

Caller-provided routing preferences attached to `CompletionRequest`.

| Field | Type | Constraints | Description |
| ------- | ------ | ------------- | ------------- |
| preferred_tier | `Option<String>` | Optional | Preferred model tier label (e.g., "slm", "llm") |
| max_cost_tokens | `Option<u64>` | Optional | Maximum token budget for this request |
| required_capabilities | `Vec<String>` | Optional | Capabilities the selected model must support |

**Invariant**: `RoutingHint` is consumed and stripped by the `ModelRouter` before the request
reaches a provider. Providers never see routing hints.

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
CompletionRequest 0──1 RoutingHint
CompletionResponse 1──* ContentBlock
CompletionResponse 0──* ToolCall
ToolCall 0──1 ToolResult
AgentLlmBinding *──1 ProviderConfig
AgentLlmBinding *──1 ModelCapabilities
ProviderConfig 0──1 AppServerAccountStatus
ModelRouter 1──* ProviderConfig
ModelRouter 1──1 RoutingPolicy
ModelRouter 1──* HealthStatus
RoutingPolicy 0──1 CascadePolicy
CascadePolicy 1──* CascadeTier
CascadeTier 1──1 ProviderConfig
HealthStatus 1──1 CircuitState
BudgetNode 1──1 BudgetPolicy
StreamChunk ──converts──> ModelEvent
ModelEvent ──tagged──> BackpressurePolicy
ModelEvent ──tagged──> StreamClass
MessageEnvelope 0──1 MessagePlane
MessageEnvelope 0──1 StreamClass
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

### Circuit Breaker Lifecycle

```text
Closed -> (consecutive failures exceed threshold) -> Open
Open -> (timeout expires) -> HalfOpen
HalfOpen -> (probe succeeds) -> Closed
HalfOpen -> (probe fails) -> Open
```

### Budget Lifecycle

```text
Available -> Reserved (CAS: used_tokens += estimated) -> Consumed (CAS: reconcile actual)
Available -> Reserved -> Overrun (actual > estimated; reconcile negative remaining)
Available -> Exhausted (used_tokens >= limit_tokens) -> Rejected | Downgraded (per BudgetPolicy)
```

### Cascade Routing Flow

```text
Request -> Tier1 (SLM) -> [confidence >= threshold] -> Accept
                        -> [confidence < threshold] -> Tier2 (LLM) -> Accept
                        -> [all tiers exhausted] -> Return final tier response
```

### Stream Event Flow (Two-Layer Architecture)

```text
Provider SSE -> StreamChunk (4 variants) -> Stream Actor -> ModelEvent (28 variants)
                                                         -> Semantic stream (JetStream, lossless)
                                                         -> UI stream (NATS Core, best-effort)
```

## Validation Rules

1. Provider-specific request and response formats stay internal to provider modules.
2. `ToolDefinition.input_schema` must remain valid JSON Schema after ToolBus export.
3. `ToolResult.call_id` must match the originating `ToolCall.call_id`.
4. `AgentLlmBinding.agent_type` is limited to Planner, Critic, and Executor for Phase 9.
5. Missing or unsupported provider capabilities must return typed `LlmError` values.
6. `MessageEnvelope.plane` defaults to `Data` when `None`. `MessageEnvelope.stream_class` defaults
   to `Semantic` when `None`. Both use `Option<T>` with `#[serde(default)]` for backward
   compatibility.
7. `BudgetNode.used_tokens` must only be updated via JetStream KV CAS operations.
8. `ModelEvent` must use `#[non_exhaustive]` and include `#[serde(other)]` on the `Unknown`
   variant for forward compatibility.
9. `HealthStatus.circuit_state` transitions must follow the circuit breaker state machine
   (Closed -> Open -> HalfOpen -> Closed/Open).
10. `RoutingHint` must be stripped from `CompletionRequest` before the request reaches a provider.
11. `CascadePolicy.tiers` must be ordered cheapest-first; the router attempts them in order.
12. `ProviderKind` must include `ClaudeSubscription` alongside `Anthropic` to distinguish
    OAuth-based Claude subscription access from planned API-key Anthropic access.

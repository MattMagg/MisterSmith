# Feature Specification: Phase 9 — LLM Provider Integration

**Feature Branch**: `009-phase9-llm-provider-integration`  
**Created**: 2026-03-06  
**Status**: Draft  
**Input**: `ROADMAP.md` Phase 9, `docs/plans/2026-03-05-llm-provider-integration-design.md`,
`docs/2026-03-05-architectural-grounding-audit.md`,
`docs/2026-03-05-implementation-deviation-report.md`, and the active Phase 7 baseline in
`specs/007-phase7-agent-system/`.

## Scope & Traceability

### Governing Sources

This Phase 9 specification is constrained by the following precedence order:

1. `ROADMAP.md` Phase 9 (`LLM Provider Integration`)
2. `docs/plans/2026-03-05-llm-provider-integration-design.md`
3. Canonical architecture sources in `spec/`
4. Supporting repo context in `README.md`, `VALIDATION_REPORT.md`, `CLAUDE.md`, and `AGENTS.md`

The stale `specs/008-agent-system/` path is not a valid source for this phase. The active agent
baseline is `specs/007-phase7-agent-system/spec.md`, `plan.md`, and `tasks.md`.

### Canonical Architecture Citations

Phase 9 artifacts MUST trace back to these architecture sources:

- `spec/data-management/agent-orchestration.md` §10.4 and ToolBus sections for LLM coordination
  context, agent-role boundaries, and tool dispatch patterns
- `spec/data-management/message-schemas.md` §5 for deferred hook-event schemas and subject naming
  context
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 for deferred Neural/AI
  Operations scope
- `spec/core-architecture/type-definitions.md` for canonical core types and shared error patterns
- `spec/core-architecture/async-patterns.md` for agent-as-tool and ToolBus patterns
- `spec/core-architecture/coding-standards.md` for tool permissions, testing requirements, and
  error-handling expectations

### In Scope

- New `mister-smith-llm` crate
- `ModelProvider` trait
- Unified completion, streaming, embedding, and tool-calling types
- `MockProvider`
- `AnthropicProvider`
- `OpenAiProvider`
- `OpenAiChatGptProvider`
- Codex app-server client integration for ChatGPT subscription auth and turn execution
- `mister-smith-agents` `llm` feature
- `mister-smith-app` auth subcommands for `openai-chatgpt` login and status
- Agent-LLM bridge for Planner, Critic, and Executor roles
- `ToolBus::to_tool_definitions()`
- `ToolBus::execute_tool_call()`
- Gate 9 behavior from `ROADMAP.md` and the approved design document

### Explicitly Deferred

The following items are architecture-adjacent but are not Phase 9 acceptance scope:

- Hook event system and `llm.hooks.*` subjects
- `LlmTaskOutputParser` regex routing
- Neural/AI Operations domain work
- Prompt-engineering framework
- RAG pipeline
- Guardrails or safety layer
- Non-MVP providers beyond Anthropic and OpenAI
- Custom OAuth, browser callback handling, or local ChatGPT token persistence outside Codex
  app-server
- ChatGPT-backed embeddings
- Codex app-server approval, file-change, or shell-execution workflows outside the LLM tool bridge

### Prerequisites & Blockers

Phase 9 depends on the existing Phase 7 agent-system baseline and must keep the following
Phase 7.5 work visible as prerequisites, dependencies, or blockers rather than treating them as
Phase 9 implementation scope:

- Security integration for agent messaging, tool permissions, and audit logging
- Router balancing strategies (`round-robin`, `least-loaded`)
- Memory metadata, timestamps, versions, and access counts
- Heartbeat receiver and failure detection
- Supervisor delegation to the Phase 3 supervision system
- Priority mailbox wiring

If Phase 9 subphases `9.4` or `9.5` require any unresolved item above, that dependency must be
reported as a blocker during analysis instead of being merged into this feature scope.

### Subphase Boundary Summary

| Subphase | Required Phase 9 scope | Must remain out of scope |
| -------- | ---------------------- | ------------------------ |
| `9.1` | Shared `mister-smith-llm` crate, `ModelProvider`, unified types, `MockProvider` | Provider-specific public APIs, role hardening, hook events |
| `9.2`-`9.3` | Anthropic, OpenAI API-key, and OpenAI ChatGPT-backed adapters behind the shared contract | Additional providers, provider-specific orchestration logic, non-neutral public types |
| `9.4` | `mister-smith-agents` `llm` feature and provider-backed Planner/Critic/Executor integration | Router/Memory/Supervisor rewrites, heartbeat receiver, priority mailbox, security hardening |
| `9.5` | `ToolBus::to_tool_definitions()`, `ToolBus::execute_tool_call()`, and Gate 9 tool round-trips | Parallel tool invocation paths, hook-event workflows, prompt-framework features |

## Clarifications

### Session 2026-03-06

- Q: Do the citations to `agent-orchestration.md` §10.4, `message-schemas.md` §5, and
  `SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 mean Phase 9 must implement those sections in full?
  → A: No. They are binding architecture references for traceability and boundary-setting. Phase 9
  implements the provider abstraction, agent bridge, and ToolBus bridge only; hook subjects,
  `LlmTaskOutputParser`, and Neural/AI Operations stay deferred.
- Q: Does Phase 9.4 include broad Phase 7 role hardening because Planner, Critic, and Executor gain
  real LLM calls?
  → A: No. Phase 9.4 is limited to provider-backed behavior for Planner, Critic, and Executor. The
  listed Phase 7.5 items remain prerequisites, dependencies, or blockers and must not be recast as
  Phase 9 deliverables.
- Q: May Phase 9.5 introduce a new tool-execution path tailored to provider APIs?
  → A: No. Model-initiated tool calls must stay inside existing ToolBus permission, timeout, audit,
  and error boundaries. Missing or unverified hardening in those boundaries is a blocker, not a
  scope-expansion trigger.
- Q: Does Anthropic/OpenAI parity require every configured model to support every unified
  capability?
  → A: No. The unified contract must normalize supported capabilities and surface unsupported ones
  via typed errors. Gate 9 parity is satisfied by the same Planner-to-Orchestrator-to-Worker flow
  and tool-call round-trip working with supported Anthropic and OpenAI configurations.
- Q: Should Mister Smith implement its own browser callback, OAuth exchange, and token persistence
  for ChatGPT-subscription auth?
  → A: No. The ChatGPT-subscription path must stay a thin client of the documented Codex
  app-server protocol. `mister-smith-app` owns the explicit login command, while the provider uses
  Codex app-server account and turn methods instead of introducing a second OAuth or session stack.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Provider-Neutral LLM Core Contract (Priority: P1)

A framework developer adds the `mister-smith-llm` crate to the workspace and gets a stable,
provider-neutral contract for completions, streaming, embeddings, and tool calling. The same
contract is available through a deterministic `MockProvider` so consumers can test LLM-dependent
logic without live credentials.

**Why this priority**: Phase 9 cannot safely add providers or agent integration until the shared
types and trait boundaries are fixed. This is the foundation for every downstream subphase.

**Independent Test**: Build the crate with no real provider feature flags enabled, run the
`MockProvider` contract tests, and verify that completion, streaming, embedding, and tool-calling
flows succeed through the unified public API.

**Acceptance Scenarios**:

1. **Given** a consumer of `mister-smith-llm`, **When** it constructs a completion request,
   **Then** it uses provider-neutral message, response, usage, and tool-call types with no
   provider-specific structs required outside provider modules.
2. **Given** the `MockProvider`, **When** deterministic requests are executed through complete,
   stream, and embed paths, **Then** tests receive predictable outputs without network access or
   API keys.
3. **Given** a request for a capability a model does not support, **When** the request is
   executed through the common interface, **Then** the failure is surfaced through a typed LLM
   error rather than provider-specific error leakage.

---

### User Story 2 — Anthropic and OpenAI Provider Parity (Priority: P1)

A framework operator configures Anthropic, OpenAI API-key access, or OpenAI
ChatGPT-subscription access and can execute the same provider-neutral request shape through those
backends for supported capabilities without changing call sites.

**Why this priority**: The roadmap requires at least two real providers. Without parity across the
Anthropic backend and the OpenAI-family backends, Mister Smith remains a single-provider
experiment instead of a model-agnostic framework capability.

**Independent Test**: Run env-gated integration tests once with Anthropic credentials and once
with OpenAI API-key credentials, then run a manual ChatGPT-backed Codex app-server validation
using the same unified request or response contract and asserting equivalent behavioral outcomes
for supported capabilities.

**Acceptance Scenarios**:

1. **Given** a unified completion request, **When** it is executed through `AnthropicProvider`,
   **Then** the result is returned as a unified completion response with normalized content, usage,
   stop reason, and tool-call structures.
2. **Given** the same unified completion request, **When** it is executed through `OpenAiProvider`,
   **Then** the result is returned through the same public response types without call-site changes.
3. **Given** the same unified completion request, **When** it is executed through
   `OpenAiChatGptProvider`, **Then** the result is returned through the same public response types
   while ChatGPT-specific authentication remains outside Mister Smith's public contract.
4. **Given** a streaming request, **When** any supported real provider emits partial output,
   **Then** the stream is surfaced through unified stream-chunk semantics that preserve ordering
   and stop-state information.
5. **Given** provider authentication failure, rate limiting, invalid parameters, or a missing
   ChatGPT login session, **When** the request fails, **Then** the provider maps the failure into
   the shared LLM error hierarchy with retryability information when applicable and the
   ChatGPT-backed path directs the operator to `mister-smith auth openai-chatgpt login` instead of
   attempting hidden interactive login during request execution.
6. **Given** a capability that a selected backend does not support, **When** a caller requests it,
   **Then** the provider returns a typed `LlmError::UnsupportedCapability` instead of pretending
   parity where the backend contract does not exist.

---

### User Story 3 — Planner-Led Task Decomposition via Real LLMs (Priority: P2)

A Planner role receives a high-level task, calls a real model through `ModelProvider`, returns a
structured decomposition, and the Orchestrator assigns subtasks to Workers without provider-specific
logic leaking into the agent system.

**Why this priority**: This is the roadmap Gate 9 proof point. It turns the provider abstraction
into an actual multi-agent workflow instead of leaving it as an isolated utility crate.

**Independent Test**: Enable the `llm` feature in `mister-smith-agents`, wire either provider into
Planner, Critic, and Executor role implementations, and verify that a Planner-generated decomposition
drives Worker assignment through the existing orchestration flow.

**Acceptance Scenarios**:

1. **Given** `mister-smith-agents` built with the `llm` feature, **When** a Planner receives a
   decomposable task, **Then** it can call a configured `ModelProvider` and return a structured
   subtask plan suitable for the Orchestrator.
2. **Given** a structured Planner response, **When** the Orchestrator processes it, **Then** the
   Orchestrator assigns subtasks to Workers through existing Phase 7 task-orchestration paths
   instead of introducing provider-specific assignment logic.
3. **Given** the same orchestration flow executed once with Anthropic and once with OpenAI,
   **When** the providers are swapped, **Then** the Planner-to-Orchestrator-to-Worker flow still
   succeeds with no provider-specific changes outside provider configuration or selection.

---

### User Story 4 — Tool Calling Through the ToolBus (Priority: P2)

A model requests a tool call during execution, the framework exports available tools as JSON Schema,
dispatches the requested tool through the existing `ToolBus`, and returns the result to the model
through the unified LLM response flow.

**Why this priority**: Tool calling is required for provider parity and for the roadmap's Gate 9
round-trip requirement. It must preserve the existing tool, permission, timeout, and audit
boundaries rather than bypassing them.

**Independent Test**: Register representative tools in the existing `ToolBus`, export them as
unified tool definitions, execute a model-initiated tool call, and verify the result round-trips
through the common tool and LLM interfaces.

**Acceptance Scenarios**:

1. **Given** a populated `ToolBus`, **When** an LLM-enabled agent requests tool definitions,
   **Then** the framework exports registered tools as unified definitions with JSON Schema inputs
   and stable names/descriptions.
2. **Given** a model-emitted tool call, **When** `ToolBus::execute_tool_call()` is invoked,
   **Then** the framework dispatches through the existing tool invocation path and returns a
   structured tool result to the caller.
3. **Given** a tool call that violates permission or timeout requirements, **When** execution is
   attempted, **Then** the failure is enforced through the existing tool boundary and reported
   through typed errors instead of silent fallback behavior.

## Edge Cases

- A provider supports completion and streaming but not embeddings or tool calling.
- A ChatGPT-backed provider requires browser login and the active Codex app-server session is
  missing, expired, or logged out.
- Anthropic and OpenAI produce different native stop-reason or tool-call formats for the same
  request.
- Codex app-server completion and turn streams emit agent-message deltas without a final structured
  assistant content block until `turn/completed`.
- A streaming response emits partial tool-call payloads that must be reassembled safely.
- A Planner receives malformed or semantically incomplete model output for subtask decomposition.
- `ToolBus::to_tool_definitions()` sees tools whose schemas are valid for the bus but incompatible
  with provider-side JSON Schema restrictions.
- Codex app-server is used only for ChatGPT-backed completion and streaming in this phase, so
  ChatGPT tool-calling requests must surface as typed unsupported-capability errors instead of
  falling back silently.
- `ToolBus::execute_tool_call()` is invoked while Phase 7.5 permission wiring or audit integration
  is still unresolved.
- The `llm` feature is disabled in `mister-smith-agents` but downstream code attempts to construct
  LLM-enabled role behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The workspace MUST add a new `mister-smith-llm` crate for Phase 9 and keep provider
  integration isolated from the existing agent-system crate by default.
- **FR-002**: `mister-smith-llm` MUST expose a `ModelProvider` trait that supports completion,
  streaming, embeddings, model identification, and capability reporting through a provider-neutral
  API.
- **FR-003**: Phase 9 MUST define unified public types for completion requests/responses, chat
  messages, stream chunks, embeddings, tool definitions, tool calls, tool results, usage, and stop
  reasons.
- **FR-004**: Shared LLM failures MUST map into a typed LLM error hierarchy consistent with the
  core type and error conventions documented in `spec/core-architecture/type-definitions.md` and
  `spec/core-architecture/coding-standards.md`.
- **FR-005**: A deterministic `MockProvider` MUST be available without feature flags and MUST cover
  completion, streaming, embeddings, and tool-calling test paths.
- **FR-006**: `AnthropicProvider` MUST implement the shared contract for completions, streaming,
  embeddings, and tool use using the unified public types.
- **FR-007**: `OpenAiProvider` MUST implement the shared contract for completions, streaming,
  embeddings, and tool use using the unified public types.
- **FR-007A**: `OpenAiChatGptProvider` MUST implement the shared contract for completions and
  streaming by acting as a thin client of the documented Codex app-server protocol. It MUST surface
  embeddings and tool calling as `LlmError::UnsupportedCapability` rather than emulating them.
- **FR-008**: Provider-specific request or response types MUST remain internal to provider modules.
  Public call sites outside `mister-smith-llm` MUST NOT require provider-specific structs or enums.
- **FR-008A**: `mister-smith-app` MUST expose explicit `auth openai-chatgpt login` and
  `auth openai-chatgpt status` commands for the ChatGPT-subscription path. Provider execution MUST
  NOT auto-open the browser or silently start an interactive login flow.
- **FR-009**: `mister-smith-agents` MUST add an optional `llm` feature that gates all direct
  dependencies on `mister-smith-llm`.
- **FR-010**: The `llm` feature MUST add provider-backed behavior only to Planner, Critic, and
  Executor role paths required by the approved Phase 9 design.
- **FR-011**: The Planner integration MUST produce a structured subtask decomposition that the
  existing Orchestrator can consume without introducing provider-specific orchestration logic.
- **FR-012**: The agent bridge MUST preserve the Phase 7 model-agnostic extension boundary by
  keeping planning, evaluation, and execution logic provider-neutral outside the selected
  `ModelProvider`.
- **FR-013**: `ToolBus::to_tool_definitions()` MUST export currently registered tools as unified
  tool definitions backed by JSON Schema.
- **FR-014**: `ToolBus::execute_tool_call()` MUST dispatch model-requested tool calls through the
  existing ToolBus execution path instead of introducing a parallel tool invocation mechanism.
- **FR-015**: Tool-calling integration MUST preserve existing permission, timeout, and error
  semantics defined by the ToolBus patterns in `spec/core-architecture/async-patterns.md`,
  `spec/data-management/agent-orchestration.md`, and `spec/core-architecture/coding-standards.md`.
- **FR-016**: Phase 9 MUST include deterministic unit tests around the shared types and
  `MockProvider`, plus env-gated real-provider integration tests for Anthropic and OpenAI API-key
  access, plus stubbed and manual validation coverage for the ChatGPT-subscription path.
- **FR-017**: The Gate 9 workflow MUST succeed with Anthropic and at least one OpenAI-family
  backend: Planner calls a real model, receives structured subtasks, Orchestrator assigns subtasks
  to Workers, and tool calls round-trip through the ToolBus when requested by the model.
- **FR-018**: Phase 9 MUST treat hook events, `LlmTaskOutputParser`, Neural/AI Operations work,
  prompt frameworks, RAG, guardrails, and non-MVP providers as deferred scope rather than silent
  acceptance criteria.
- **FR-019**: Phase 9 planning artifacts and downstream tasks MUST keep the listed Phase 7.5
  hardening items visible as prerequisites, dependencies, or blockers rather than redefining them
  as Phase 9 feature deliverables.

### Key Entities *(include if feature involves data)*

- **ModelProvider**: Provider-neutral trait boundary for completion, streaming, embedding, and
  tool-calling interactions with external LLM vendors.
- **CompletionRequest**: Unified input structure containing chat messages, optional system prompt,
  generation controls, tool definitions, and provider-neutral metadata.
- **CompletionResponse**: Unified output structure containing normalized content blocks, tool calls,
  usage, model identity, and stop reason.
- **ToolDefinition**: Provider-neutral JSON Schema description of a callable tool exported from the
  existing ToolBus.
- **ToolCall**: Model-emitted request to invoke a named tool with structured input.
- **ToolResult**: Structured result returned from ToolBus execution back into the provider-neutral
  LLM flow.
- **ModelCapabilities**: Description of which unified behaviors a configured provider/model
  supports, including completion, streaming, embeddings, and tool use.
- **ChatGptAuthSession**: Managed authentication state owned by Codex app-server and surfaced to
  Mister Smith through `account/read` and login notifications rather than a Mister Smith-managed
  token store.
- **AgentLlmBridge**: Feature-gated integration boundary inside `mister-smith-agents` that wires
  Planner, Critic, and Executor role behavior to a selected `ModelProvider`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `specs/009-phase9-llm-provider-integration/` defines Phase 9 as a provider-neutral
  feature with explicit architecture citations to all required canonical `spec/` sources.
- **SC-002**: The Phase 9 scope in this spec matches `ROADMAP.md` and the approved design document,
  and the deferred items remain explicitly out of scope.
- **SC-003**: Gate 9 is expressible as an independently testable requirement: a Planner calls a
  real LLM, receives structured subtask decomposition, the Orchestrator assigns subtasks to
  Workers, and the same flow works with Anthropic and an OpenAI-family backend selected through the
  shared provider contract.
- **SC-004**: Tool calling is expressible as an independently testable requirement: registered
  tools export through `ToolBus::to_tool_definitions()`, model tool calls execute through
  `ToolBus::execute_tool_call()`, and results round-trip back into the LLM flow.
- **SC-005**: Phase 7.5 hardening remains visible in the Phase 9 prep artifacts as prerequisite or
  blocker work rather than being absorbed into the main Phase 9 implementation scope.

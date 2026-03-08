# Implementation Plan: Phase 9 — LLM Provider Integration

**Branch**: `009-phase9-llm-provider-integration` | **Date**: 2026-03-07 (revised) | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from `/specs/009-phase9-llm-provider-integration/spec.md`

## Summary

Phase 9 adds provider-neutral LLM connectivity and three research-driven architectural
capabilities: a two-plane router with budget enforcement (Finding #8), SLM-default/LLM-fallback
routing (Finding #9), and dual-stream formalization with the `ModelEvent` enum (Finding #13).

The implementation centers on a new `mister-smith-llm` crate that owns the `ModelProvider` trait,
unified completion and streaming types, deterministic mock behavior, and feature-gated provider
adapters. The `ModelRouter` layer sits above providers to handle routing decisions, circuit
breaking, and budget enforcement. Stream actors convert raw `StreamChunk` items into canonical
`ModelEvent` items for dual-stream delivery.

Existing agent orchestration remains the system boundary: the `mister-smith-agents` crate gains an
optional `llm` feature for Planner, Critic, and Executor, while tool calling continues to flow
through the current `ToolBus`.

**Partial implementation status**: Core types, `MockProvider`, `OpenAiProvider`, and
`ClaudeSubscriptionProvider` are already implemented (tasks T001-T008, T012-T014A complete).
The spec originally planned `AnthropicProvider` (API-key auth) but the codebase implemented
`ClaudeSubscriptionProvider` (OAuth Bearer auth) instead. Both are valid — the revision
acknowledges `ClaudeSubscriptionProvider` as implemented and retains `AnthropicProvider` as
planned future work.

## Technical Context

- **Language/Version**: Rust, MSRV 1.88.0
- **Primary Dependencies**: existing workspace crates plus `reqwest` 0.12+ for provider APIs,
  `tokio`/`futures` for async streaming and process I/O, `serde`/`serde_json`, `async-trait`,
  `webbrowser` for app-driven ChatGPT login UX
- **Storage**: JetStream KV for budget enforcement (CAS), health state, and control-plane
  configuration; no new PostgreSQL layer inside `mister-smith-llm`
- **Testing**: `cargo test`, deterministic mock-provider unit tests, env-gated provider
  integration tests, router/budget/circuit-breaker tests, dual-stream tests, Gate 9 validation
- **Target Platform**: Linux server runtime, macOS development parity
- **Performance Goals**: sub-millisecond data-plane routing overhead, <1% budget overrun rate
  via CAS, lossless tool-call delivery on semantic stream
- **Constraints**: no learned routing (RouteLLM), no step-level PRMs, no guided decoding, no
  local model inference, no disaggregated serving

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
| ----------- | -------- | ---------- |
| I. Canonical Single Source | PASS | Core keeps canonical IDs and shared errors, ToolBus stays in agents, router lives in llm crate. |
| II. Spec-First Design | PASS | Spec defines scope, research grounding, FRs, SCs, and deferred work. |
| III. Phase-Gated Build Order | PASS | Phase 9 follows Phases 1-8, research findings are phased appropriately. |
| IV. Model-Agnostic Architecture | PASS | Providers sit behind `ModelProvider`, router is provider-neutral, cascade policy is model-agnostic. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Stream actors use OTP supervision, circuit breakers handle provider failure, cascade provides escalation. |
| VI. Evidence-Based Validation | PASS | Three-tier validation plus router/budget/dual-stream specific tests. |
| VII. Explicit Dependency Management | PASS | New entities and dependencies enumerated, MessageEnvelope changes are backward-compatible. |

## Project Structure

### Documentation (this feature)

```text
specs/009-phase9-llm-provider-integration/
+-- spec.md                     # Feature specification (revised 2026-03-07)
+-- plan.md                     # This file (revised 2026-03-07)
+-- research.md                 # Research grounding (revised 2026-03-07)
+-- data-model.md               # Entity model (revised 2026-03-07)
+-- quickstart.md               # Validation and usage flow
+-- contracts/                  # Public contracts
|   +-- agent-llm-bridge.md     # Agents feature-gated LLM integration contract
|   +-- model-provider.md       # Provider-neutral LLM interface contract
|   +-- tool-calling-bridge.md  # ToolBus <-> LLM tool-calling contract
+-- tasks.md                    # Task breakdown
+-- analyze.md                  # Cross-artifact analysis
```

### Source Code (repository root)

```text
Cargo.toml                                  # Workspace member + dependencies

crates/mister-smith-core/
+-- src/error.rs                            # Canonical LlmError hierarchy (done)
+-- src/lib.rs                              # Re-export LlmError (done)

crates/mister-smith-llm/
+-- Cargo.toml                              # Crate manifest and feature flags (done)
+-- src/
|   +-- lib.rs                              # Crate docs and re-exports (done)
|   +-- app_server.rs                       # Codex app-server client (done)
|   +-- config.rs                           # Provider/model configuration (done)
|   +-- provider.rs                         # ModelProvider trait (done)
|   +-- streaming.rs                        # StreamChunk and parser helpers (done)
|   +-- tool_schema.rs                      # ToolDefinition, ToolCall, ToolResult (done)
|   +-- types.rs                            # Requests, responses, messages, capabilities (done)
|   +-- mock.rs                             # Deterministic MockProvider (done)
|   +-- router.rs                           # ModelRouter, RoutingPolicy, CascadePolicy (NEW)
|   +-- budget.rs                           # BudgetNode, BudgetPolicy, JetStream KV CAS (NEW)
|   +-- health.rs                           # HealthStatus, CircuitState, circuit breaker (NEW)
|   +-- model_event.rs                      # ModelEvent enum (28 variants) (NEW)
|   +-- dual_stream.rs                      # Dual-stream actor, StreamClass, backpressure (NEW)
|   +-- providers/
|       +-- mod.rs                          # (done)
|       +-- anthropic.rs                    # #[cfg(feature = "anthropic")] (planned)
|       +-- openai.rs                       # #[cfg(feature = "openai")] (done)
|       +-- openai_chatgpt.rs              # #[cfg(feature = "openai-chatgpt")] (done)
|       +-- claude_subscription.rs         # #[cfg(feature = "claude-subscription")] (done)
+-- tests/
    +-- mock_tests.rs                       # Contract tests (done)
    +-- types_tests.rs                      # Serialization tests (done)
    +-- router_tests.rs                     # Router, cascade, health tests (NEW)
    +-- budget_tests.rs                     # Budget CAS, overrun tests (NEW)
    +-- model_event_tests.rs               # ModelEvent serde, forward compat tests (NEW)
    +-- dual_stream_tests.rs               # Dual-stream backpressure tests (NEW)
    +-- integration/
        +-- anthropic_tests.rs              # Env-gated (planned)
        +-- openai_tests.rs                 # Env-gated (done)

crates/mister-smith-transport/
+-- src/envelope.rs                         # MessageEnvelope: add plane, stream_class (NEW)

crates/mister-smith-app/
+-- src/main.rs                             # Auth subcommands (done)

crates/mister-smith-agents/
+-- Cargo.toml                              # Optional llm feature
+-- src/agent.rs                            # Model attachment boundary
+-- src/orchestrator.rs                     # Consume structured Planner output
+-- src/tool_bus.rs                         # to_tool_definitions() + execute_tool_call()
+-- src/roles/
    +-- planner.rs                          # Provider-backed decomposition
    +-- critic.rs                           # Provider-backed evaluation
    +-- executor.rs                         # Provider-backed execution/tool loop
```

## Design Decisions

### D1: Single `mister-smith-llm` Crate With Feature-Gated Providers

**Decision**: One crate owns the provider-neutral contract, with feature flags for real providers.
**Status**: Implemented and confirmed.

### D1a: ChatGPT Subscription Access Uses Codex App-Server

**Decision**: `OpenAiChatGptProvider` is a thin client of Codex app-server's JSON-RPC protocol.
**Status**: Implemented and confirmed.

### D2: `LlmError` Lives In `mister-smith-core`

**Decision**: `LlmError` in `crates/mister-smith-core/src/error.rs`, re-exported from llm crate.
**Status**: Implemented and confirmed.

### D3: Capability Normalization, Not Lowest-Common-Denominator

**Decision**: Unified types plus `ModelCapabilities`; unsupported behavior returns typed errors.
**Status**: Implemented and confirmed.

### D4: Agent Bridge Stops At Planner, Critic, Executor

**Decision**: Feature-gated LLM bridge extends existing role seams.
**Status**: Planned, not yet implemented.

### D5: Tool Calling Through ToolBus Only

**Decision**: `ToolBus::to_tool_definitions()` and `execute_tool_call()` are the only bridge.
**Status**: Planned, not yet implemented.

### D5a: ChatGPT Provider Scope Stops At Completion And Streaming

**Decision**: Tool calling and embeddings surface as `UnsupportedCapability` for ChatGPT.
**Status**: Implemented and confirmed.

### D6: Phase 7.5 Hardening Visible As Blockers

**Decision**: Security, router, memory, heartbeat, supervisor, mailbox hardening are blockers.
**Status**: Active. Security items now have dedicated Phase 9.1 spec.

### D7: Two-Plane Router Architecture (NEW — Finding #8)

**Decision**: Separate microsecond data plane (NATS request-reply, ~50us) from control plane
(JetStream KV watches). The `ModelRouter` executes routing decisions in the data plane using
local in-memory state refreshed by KV watches.

**Rationale**: Converged across all three R3 industry reports, validated by R4 academic surveys,
and reinforced by production gateways. No competing framework separates data plane from control
plane. Source: `consolidated/01-model-routing-and-cost-optimization.md`.

### D8: SLM-Default / LLM-Fallback Routing (NEW — Finding #9)

**Decision**: Default routing policy starts with the cheapest capable model and escalates based
on configurable confidence thresholds. `CascadePolicy` with ordered tiers.

**Rationale**: 10-100x cost reduction for structured tasks. Liu (2025, 106 citations) showed
0.5B outperforms GPT-4o. Guided decoding and local inference are Phase 10+ scope — Phase 9
implements the cascade policy and escalation logic only.

### D9: Dual-Stream with ModelEvent (NEW — Finding #13)

**Decision**: Emit two parallel streams from the same canonical event log — lossless semantic
(JetStream) and best-effort UI (NATS Core). `ModelEvent` enum (28 variants, `#[non_exhaustive]`,
`#[serde(other)]`) is the canonical internal event type.

**Rationale**: All three R3 source reports independently conclude streaming must be a typed event
pipeline. The dual-stream design decouples correctness from presentation. No competing framework
distinguishes lossless orchestration events from lossy UI events.

**Critical design**: `StreamChunk`/`ChunkDelta` (4 variants) = raw provider boundary.
`ModelEvent` (28 variants) = canonical internal event type. Two layers, not a replacement.
Providers emit `StreamChunk`; stream actors convert to `ModelEvent`.

### D10: Budget Enforcement via JetStream KV CAS (NEW — Finding #8)

**Decision**: Hierarchical budget tracking using JetStream KV CAS with reserve-before-send /
reconcile-after-completion pattern.

**Rationale**: All three R3 reports converge on budget enforcement in the router. CAS-based
enforcement demonstrates <1% overrun rate. Budget checks execute in the data plane as
constant-time in-memory lookups, refreshed by control-plane updates.

## Dependency Changes

### Workspace Manifest

- `"crates/mister-smith-llm"` is already in `[workspace].members`
- `reqwest` 0.12+ already in `[workspace.dependencies]`
- No new workspace-level dependencies for router/budget/dual-stream (uses existing `async-nats`)

### Existing Crates Touched

- `crates/mister-smith-core`: `LlmError` already added (T003 complete)
- `crates/mister-smith-transport`: add `MessagePlane`, `StreamClass` to `MessageEnvelope` (NEW)
- `crates/mister-smith-agents`: add optional `llm` feature and `mister-smith-llm` dependency
- `crates/mister-smith-app`: auth subcommands (T014 complete)

## Subphase Execution Plan

### 9.1 Core Types and MockProvider (DONE)

**Status**: Complete. Tasks T001-T008 implemented.

**Outputs**: Compilable `mister-smith-llm` crate with `ModelProvider` trait, unified types,
`MockProvider`, contract tests.

### 9.2a Two-Plane Router + Health + Budget (NEW)

**Scope**:
- `ModelRouter` with data-plane routing using local in-memory state
- `RoutingPolicy` enum (RoundRobin, CostOptimized, CapabilityMatched, Cascade)
- `HealthStatus` and `CircuitState` for health-aware circuit breakers
- `BudgetNode` and `BudgetPolicy` with JetStream KV CAS enforcement
- Control-plane KV watch subscription for configuration updates

**Outputs**:
- Router tests (sub-millisecond overhead)
- Circuit breaker state transition tests
- Budget CAS tests (<1% overrun rate)

- **Depends on**: 9.1 (core types)
- **Must not absorb**: learned routing (RouteLLM), step-level PRMs

### 9.2b Dual-Stream + ModelEvent + MessageEnvelope (NEW)

**Scope**:
- `ModelEvent` enum (28 variants, `#[non_exhaustive]`, `#[serde(other)]`)
- Stream actor converting `StreamChunk` to `ModelEvent`
- Dual-stream delivery (semantic via JetStream, UI via NATS Core)
- `BackpressurePolicy` per event class
- `MessageEnvelope` additions: `plane: Option<MessagePlane>`,
  `stream_class: Option<StreamClass>` with `#[serde(default)]`

**Outputs**:
- `ModelEvent` serde and forward compatibility tests
- Dual-stream backpressure tests
- `MessageEnvelope` backward compatibility tests

- **Depends on**: 9.1 (core types), `mister-smith-transport` (MessageEnvelope)
- **Must not absorb**: streaming content monitors, disaggregated serving

### 9.3 Providers (PARTIALLY DONE)

**Status**: `OpenAiProvider` (T012-T013) and `ClaudeSubscriptionProvider` (T014-T014A) are
implemented. `AnthropicProvider` (API-key auth via Anthropic Messages API) is planned.

**Scope**:
- Feature-gated `AnthropicProvider` for API-key access
- Feature-gated `OpenAiChatGptProvider` for ChatGPT subscription access through Codex app-server
- `mister-smith-app auth openai-chatgpt login` and `status` (done)

**Outputs**:
- Env-gated Anthropic integration tests
- Stubbed Codex app-server client tests

- **Depends on**: 9.1
- **Must not absorb**: additional providers, provider-specific orchestration

### 9.4 Agent-LLM Bridge

**Scope**:
- Add the `llm` feature to `mister-smith-agents`
- Attach `ModelProvider` (via `ModelRouter`) to Planner, Critic, and Executor
- Orchestrator consumes structured model output
- Dual-stream handling in bridge (semantic stream for orchestration, UI stream optional)
- Budget enforcement interface (router handles it, bridge observes routing decisions)

**Outputs**:
- Provider-backed Planner decomposition
- Provider-backed Critic evaluation and Executor action flow
- Same orchestration surface for all providers

- **Depends on**: 9.1, 9.2a, 9.2b, and Phase 7 baseline
- **Blocker sensitivity**: unresolved Phase 7.5 security items now addressed by Phase 9.1 spec

### 9.5 Tool Calling Bridge

**Scope**:
- `ToolBus::to_tool_definitions()` — export as unified definitions
- `ToolBus::execute_tool_call()` — dispatch through existing ToolBus
- Tool-call events are lossless in dual-stream backpressure matrix
- Tool calls route through `ModelRouter` data plane

**Outputs**:
- Tool export and execution tests
- Gate 9 tool-calling round-trip coverage
- Lossless tool-call delivery under backpressure

- **Depends on**: 9.2a (router), 9.2b (dual-stream), 9.4 (bridge)
- **Blocker sensitivity**: unresolved permission or audit hardening is a blocker

### 9.6 SLM-Default Routing Policy Integration (NEW)

**Scope**:
- `CascadePolicy` configuration with ordered tiers
- Confidence-based escalation logic (`ConfidenceSignal`)
- Routing decision logging for observability
- Integration with `ModelRouter` cascade routing mode

**Outputs**:
- Cascade routing tests (SLM attempt -> LLM escalation)
- Confidence threshold tests
- Routing decision observability

- **Depends on**: 9.2a (router)
- **Must not absorb**: guided decoding (XGrammar/Outlines), local model inference

## Blockers and Deferred Work

### Visible Phase 7.5 Dependencies

- Security integration for agent messaging, tool permissions, and audit logging — now addressed
  by Phase 9.1 spec at `specs/011-phase9.1-security-hardening/`
- Router balancing strategies (`round-robin`, `least-loaded`) — partially addressed by
  `ModelRouter` in 9.2a
- Memory metadata, timestamps, versions, and access counts — remains deferred
- Heartbeat receiver and failure detection — remains deferred
- Supervisor delegation to Phase 3 `SupervisedSystem` — remains deferred
- Priority mailbox wiring — remains deferred

### Explicit Deferred Scope

- Learned routing via RouteLLM / kNN / ONNX embeddings (Phase 10+)
- Step-level intelligence / Process Reward Models (Phase 10)
- Guided decoding via XGrammar / Outlines (Phase 10)
- Local model inference / `LocalModelProvider` (Phase 10+)
- Disaggregated serving / shared KV cache / PrefillShare (Phase 10+)
- Dynamic topology / MaAS / MAS^2 (Phase 11)
- Inter-agent message authentication / AgentSandbox (Phase 9.1)
- CRDT coordination (Phase 13)
- MPST session types (Phase 13)

## Complexity Tracking

No constitution violations. The three new capabilities (router, dual-stream, budget) are
additive to the existing plan and do not require re-architecturing any prior phase. The
`MessageEnvelope` changes are backward-compatible via `Option<T>` with `#[serde(default)]`.

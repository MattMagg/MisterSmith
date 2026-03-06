# Implementation Plan: Phase 9 — LLM Provider Integration

**Branch**: `009-phase9-llm-provider-integration` | **Date**: 2026-03-06 | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from `/specs/009-phase9-llm-provider-integration/spec.md`

## Summary

Phase 9 adds provider-neutral LLM connectivity without breaking Mister Smith's model-agnostic
architecture. The implementation centers on a new `mister-smith-llm` crate that owns the
`ModelProvider` trait, unified completion and streaming types, deterministic mock behavior, and
feature-gated Anthropic/OpenAI adapters. Existing agent orchestration remains the system boundary:
the `mister-smith-agents` crate gains an optional `llm` feature for Planner, Critic, and Executor,
while tool calling continues to flow through the current `ToolBus` rather than a provider-specific
execution path.

This plan fixes the traceability gap called out in the 2026-03-05 architectural grounding audit by
anchoring every major decision to canonical `spec/` sources. It also preserves the approved Phase 9
subphases `9.1` through `9.5`, keeps Phase 7.5 hardening visible as prerequisite or blocker work,
and encodes Gate 9 as the real-provider Planner -> Orchestrator -> Worker proof point required by
`ROADMAP.md`.

## Technical Context

- **Language/Version**: Rust, MSRV 1.88.0
- **Primary Dependencies**: existing workspace crates plus `reqwest` 0.12+ for provider APIs,
  `tokio`/`futures` for async streaming, `serde`/`serde_json`, `async-trait`
- **Storage**: N/A inside `mister-smith-llm`; existing PostgreSQL and JetStream integrations remain
  indirect dependencies for Gate 9 orchestration and ToolBus audit boundaries
- **Testing**: `cargo test`, deterministic mock-provider unit tests, env-gated Anthropic/OpenAI
  integration tests, Gate 9 orchestration validation
- **Target Platform**: Linux server runtime, macOS development parity
- **Project Type**: new workspace library crate plus feature-gated integration into existing library
  crate
- **Performance Goals**: preserve streaming order, keep provider-specific serialization outside
  public call sites, and keep model tool-calling inside existing timeout and audit boundaries
- **Constraints**: no hook-event system, no `LlmTaskOutputParser`, no Neural/AI Operations
  implementation, no provider-specific public types, and no new tool-execution path outside
  `ToolBus`
- **Scale/Scope**: 1 new crate, 2 real providers, 1 deterministic mock provider, 1 optional
  agents feature, Planner/Critic/Executor bridge, and ToolBus JSON Schema export and execution
  bridge

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
| ----------- | -------- | ---------- |
| I. Canonical Single Source | PASS | Core keeps canonical IDs and shared errors, ToolBus stays in agents, and the plan cites the required core architecture docs directly. |
| II. Spec-First Design | PASS | `specs/009-phase9-llm-provider-integration/spec.md` defines scope, clarifications, FRs, SCs, and deferred work before any implementation task breakdown. |
| III. Phase-Gated Build Order | PASS | The roadmap places Phase 9 after Phases 1-7, and the deviation report keeps Phase 7.5 hardening ahead of risky 9.4 and 9.5 work. |
| IV. Model-Agnostic Architecture | PASS | Anthropic and OpenAI sit behind `ModelProvider`, and unsupported capabilities return typed `LlmError` values instead of provider-specific leakage. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The plan extends `AgentRuntime`, `Orchestrator`, and `ToolBus` rather than replacing supervision or mailbox boundaries defined by prior phases. |
| VI. Evidence-Based Validation | PASS | Validation uses mock contract tests, env-gated provider integrations, ToolBus round-trip checks, and Gate 9 orchestration tests with blocker reporting. |
| VII. Explicit Dependency Management | PASS | Workspace and crate changes are enumerated below: new crate member, provider HTTP dependency, feature flags, and touched core and agent files. |

**Constitution posture**: No amendment is justified. Phase 9 fits the existing constitution, and the
main correction is better traceability, not new governance.

## Project Structure

### Documentation (this feature)

```text
specs/009-phase9-llm-provider-integration/
├── spec.md                     # Feature specification
├── plan.md                     # This file
├── research.md                 # Phase 0 decisions and rationale
├── data-model.md               # Phase 1 entity model
├── quickstart.md               # Validation and usage flow
├── contracts/                  # Phase 1 public contracts
│   ├── agent-llm-bridge.md     # Agents feature-gated LLM integration contract
│   ├── model-provider.md       # Provider-neutral LLM interface contract
│   └── tool-calling-bridge.md  # ToolBus <-> LLM tool-calling contract
└── tasks.md                    # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
Cargo.toml                                  # Add workspace member + provider HTTP dependency

crates/mister-smith-core/
├── src/error.rs                            # Add canonical LlmError hierarchy
└── src/lib.rs                              # Re-export LlmError for workspace consumers

crates/mister-smith-llm/
├── Cargo.toml                              # New crate manifest and feature flags
├── src/
│   ├── lib.rs                              # Crate docs and re-exports
│   ├── config.rs                           # Provider/model configuration
│   ├── provider.rs                         # ModelProvider trait
│   ├── streaming.rs                        # StreamChunk and parser helpers
│   ├── tool_schema.rs                      # ToolDefinition, ToolCall, ToolResult
│   ├── types.rs                            # Requests, responses, messages, capabilities
│   ├── mock.rs                             # Deterministic MockProvider
│   └── providers/
│       ├── mod.rs
│       ├── anthropic.rs                    # #[cfg(feature = "anthropic")]
│       └── openai.rs                       # #[cfg(feature = "openai")]
└── tests/
    ├── mock_tests.rs                       # Contract tests for MockProvider
    ├── types_tests.rs                      # Unified type serialization and invariants
    └── integration/
        ├── anthropic_tests.rs              # Env-gated real-provider tests
        └── openai_tests.rs                 # Env-gated real-provider tests

crates/mister-smith-agents/
├── Cargo.toml                              # Optional llm feature and new dependency
├── src/agent.rs                            # Runtime-level model attachment boundary
├── src/errors.rs                           # Bridge/provider/tool-call integration errors
├── src/lib.rs                              # Feature-gated re-exports
├── src/orchestrator.rs                     # Consume structured decomposition without provider leakage
├── src/tool_bus.rs                         # to_tool_definitions() + execute_tool_call()
└── src/roles/
    ├── planner.rs                          # Provider-backed decomposition path
    ├── critic.rs                           # Provider-backed evaluation path
    └── executor.rs                         # Provider-backed execution/tool loop path
```

**Structure Decision**: Use one new crate, `mister-smith-llm`, instead of splitting providers into
multiple crates or placing provider logic directly into `mister-smith-agents`. This matches the
workspace's one-domain-per-crate pattern, keeps provider dependencies optional, and preserves the
existing agent crate as an orchestration boundary rather than an API-integration bucket.

## Design Decisions

### D1: Single `mister-smith-llm` Crate With Feature-Gated Providers

**Decision**: Add one new crate that owns the provider-neutral contract, with `anthropic` and
`openai` feature flags for real providers and an always-on `MockProvider`.

**Rationale**: `docs/plans/2026-03-05-llm-provider-integration-design.md:23-35` already approves
this shape, and it mirrors how transport abstractions and implementations are separated elsewhere in
the workspace.

### D2: `LlmError` Lives In `mister-smith-core`

**Decision**: Add `LlmError` to `crates/mister-smith-core/src/error.rs` and re-export it from
`crates/mister-smith-core/src/lib.rs`; `mister-smith-llm` re-exports the canonical type.

**Rationale**: `crates/mister-smith-core/src/error.rs` is already the canonical home for
domain-level errors like `ToolError`, `SecurityError`, and `PersistenceError`. Keeping `LlmError`
there avoids a second top-level error taxonomy.

### D3: Capability Parity Means Normalization, Not Lowest-Common-Denominator

**Decision**: Model parity is expressed through unified request and response types plus
`ModelCapabilities`. Unsupported provider or model behaviors surface as typed errors instead of
forcing the public API down to the least-capable backend.

**Rationale**: The clarified spec explicitly rejects the idea that every configured model must
support every feature. Gate 9 only requires the same provider-neutral workflow to succeed on
supported Anthropic and OpenAI configurations.

### D4: Agent Bridge Scope Stops At Planner, Critic, Executor, and Existing Orchestrator Seams

**Decision**: Add a feature-gated LLM bridge to `mister-smith-agents` that extends Planner, Critic,
and Executor behavior and lets the Orchestrator consume structured decomposition results through its
current scheduler-driven flow.

**Rationale**: `crates/mister-smith-agents/src/orchestrator.rs` already owns decomposition,
assignment, and aggregation flow, while the current role files are intentionally thin. Phase 9 makes
those roles provider-backed without rewriting Router, Memory, Supervisor, heartbeat handling, or
mailbox architecture.

### D5: Tool Calling Must Stay Inside The Existing ToolBus Boundary

**Decision**: Implement `ToolBus::to_tool_definitions()` and `ToolBus::execute_tool_call()` as the
only bridge between model tool calls and framework tools.

**Rationale**: `crates/mister-smith-agents/src/tool_bus.rs` is already the central registry. Both
`spec/core-architecture/async-patterns.md:2164-2315` and
`specs/007-phase7-agent-system/contracts/tool-bus.md:50-89` require permission checks, timeout
enforcement, and metrics or audit behavior at that boundary.

### D6: Phase 7.5 Hardening Remains Visible As Blockers, Not Scope

**Decision**: Security integration, router balancing, memory metadata, heartbeat receiving,
supervisor delegation, and priority mailbox wiring remain prerequisite or blocker work for Phase 9.4
and 9.5 when unresolved.

**Rationale**: `docs/2026-03-05-implementation-deviation-report.md:308-318` and
`spec.md:66-113` are explicit that these items must not be silently absorbed into the feature.

## Dependency Changes

### Workspace Manifest

- Add `"crates/mister-smith-llm"` to the `[workspace].members` list in `Cargo.toml`
- Add `reqwest` 0.12+ with JSON, streaming, and rustls TLS support to `[workspace.dependencies]`

### Existing Crates Touched

- `crates/mister-smith-core`: add and re-export `LlmError`
- `crates/mister-smith-agents`: add optional `llm` feature and `mister-smith-llm` dependency

### New Crate Features

```toml
[features]
default = []
anthropic = ["dep:reqwest"]
openai = ["dep:reqwest"]
all-providers = ["anthropic", "openai"]
```

The Phase 9 spec deliberately excludes non-MVP providers, so the plan omits Google and Ollama even
though the original design doc discussed them as future extensions.

## Integration Points

### Canonical Architecture Anchors

- `spec/data-management/agent-orchestration.md:2467-2665`
  - Boundary reference for LLM coordination concepts and ToolBus patterns
- `spec/data-management/message-schemas.md:1069-1265`
  - Deferred hook-event schema reference that must remain out of scope
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md:453-500`
  - Deferred Neural/AI Operations scope reference
- `spec/core-architecture/type-definitions.md:48-153`
  - Canonical IDs, agent enums, priority levels, and top-level error pattern
- `spec/core-architecture/type-definitions.md:503-523`
  - Agent context includes transport and tool-registry access
- `spec/core-architecture/async-patterns.md:1939-2315`
  - Agent-as-tool and ToolBus behavior that Phase 9 must extend instead of replace
- `spec/core-architecture/coding-standards.md:492-620`
  - Typed error hierarchy and propagation expectations
- `spec/core-architecture/coding-standards.md:1596-1799`
  - Tool timeout, permission, audit, and testing expectations for agent integrations

### Current Code Seams

- `crates/mister-smith-core/src/traits.rs:106-149`
  - Canonical `Tool` and `Agent` traits already define JSON-schema and JSON-value boundaries
- `crates/mister-smith-agents/src/agent.rs:41-208`
  - Runtime wrapper where optional model attachment can remain feature-gated
- `crates/mister-smith-agents/src/orchestrator.rs:30-141`
  - Existing decomposition and aggregation seam for structured Planner output
- `crates/mister-smith-agents/src/tool_bus.rs:31-159`
  - Current registry, discovery, and metrics boundary that Phase 9 extends
- `crates/mister-smith-agents/src/roles/planner.rs:10-109`
  - Current thin Planner role to enrich via provider-backed decomposition
- `crates/mister-smith-agents/src/roles/critic.rs:10-102`
  - Current thin Critic role to enrich via provider-backed evaluation
- `crates/mister-smith-agents/src/roles/executor.rs:10-118`
  - Current thin Executor role to enrich via provider-backed execution/tool flow

## Subphase Execution Plan

### 9.1 Core Types and MockProvider

**Scope**:

- create `mister-smith-llm`
- define `ModelProvider`, unified types, `ModelCapabilities`, and `LlmError` integration
- implement deterministic `MockProvider`

**Outputs**:

- compilable crate with public re-exports
- contract tests for completion, streaming, embeddings, and tool-calling via mock behavior

- **Depends on**: Phase 1 core types only
- **Must not absorb**: provider-specific public types, hook events, role hardening

### 9.2 Anthropic Provider

**Scope**:

- feature-gated `AnthropicProvider`
- completion, streaming, embeddings, and tool use through unified types
- map provider failures into `LlmError`

**Outputs**:

- env-gated Anthropic integration tests
- no call-site changes outside provider selection

- **Depends on**: 9.1
- **Must not absorb**: Anthropic-specific orchestration logic or public response types

### 9.3 OpenAI Provider

**Scope**:

- feature-gated `OpenAiProvider`
- completion, streaming, embeddings, and tool use through unified types
- parity with 9.2 at the shared contract level

**Outputs**:

- env-gated OpenAI integration tests
- shared request and response semantics for supported capabilities

- **Depends on**: 9.1
- **Must not absorb**: OpenAI-specific orchestration logic or alternate public request types

### 9.4 Agent-LLM Bridge

**Scope**:

- add the `llm` feature to `mister-smith-agents`
- attach a selected `ModelProvider` to Planner, Critic, and Executor paths
- keep Orchestrator provider-neutral while consuming structured model output

**Outputs**:

- provider-backed Planner decomposition
- provider-backed Critic evaluation and Executor action flow
- same orchestration surface for Anthropic and OpenAI

- **Depends on**: 9.1 and Phase 7 baseline
- **Blocker sensitivity**: unresolved Phase 7.5 security, supervisor, heartbeat, or mailbox
  hardening must stay visible as blockers

### 9.5 Tool Calling Bridge

**Scope**:

- export registered tools as unified tool definitions
- execute model-emitted tool calls through the existing ToolBus
- round-trip tool results back into provider-neutral completion flow

**Outputs**:

- `ToolBus::to_tool_definitions()`
- `ToolBus::execute_tool_call()`
- model -> ToolBus -> model round-trip tests

- **Depends on**: 9.2 or 9.3, plus 9.4
- **Blocker sensitivity**: unresolved permission or audit hardening remains a blocker, not feature
  scope

## Blockers and Deferred Work

### Visible Phase 7.5 Dependencies

- Security integration for agent messaging, tool permissions, and audit logging
- Router balancing strategies (`round-robin`, `least-loaded`)
- Memory metadata, timestamps, versions, and access counts
- Heartbeat receiver and failure detection
- Supervisor delegation to Phase 3 `SupervisedSystem`
- Priority mailbox wiring

### Explicit Deferred Scope

- Hook event system and `llm.hooks.*` subjects
- `LlmTaskOutputParser` regex routing
- Neural/AI Operations domain work
- Prompt framework or prompt-template DSL
- RAG retrieval pipeline
- Guardrails or safety enforcement layer
- Non-MVP providers beyond Anthropic and OpenAI

## Complexity Tracking

No constitution violations. No complexity justification required.

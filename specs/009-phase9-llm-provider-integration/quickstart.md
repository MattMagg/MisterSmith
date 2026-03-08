# Quickstart: Phase 9 — LLM Provider Integration

## Prerequisites

- Phase 1-7 workspace crates present and building
- Active feature directory: `specs/009-phase9-llm-provider-integration/`
- Branch: `009-phase9-llm-provider-integration`
- For future real-provider validation:
  - `ANTHROPIC_API_KEY`
  - `OPENAI_API_KEY`
- For ChatGPT-subscription validation:
  - `codex` available on `PATH` with `app-server` support
- For router/budget validation:
  - NATS server running with JetStream enabled (for KV CAS budget enforcement)
- For Gate 9 orchestration validation: local NATS and PostgreSQL services available if the
  implementation exercises existing ToolBus security or audit boundaries

## Source Map

| Source | Quickstart impact |
| ------ | ----------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | Grounds the Gate 9 Planner-to-Orchestrator-to-Worker scenario and existing LLM coordination seams. |
| `spec/data-management/message-schemas.md` §5 | Confirms prep-mode validation does not include hook-event subjects or `llm.hooks.*` workflows. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | Keeps Neural/AI Operations flows out of this validation path. |
| `spec/core-architecture/type-definitions.md` | Anchors the provider-neutral types and feature-gated role boundaries shown in the usage sketch. |
| `spec/core-architecture/async-patterns.md` | Keeps `ToolBus::to_tool_definitions()` and `execute_tool_call()` inside the existing tool patterns. |
| `spec/core-architecture/coding-standards.md` | Keeps the validation tiers tied to testing, permission, timeout, audit, and typed-error expectations. |
| `docs/research-output/consolidated/00-MASTER-FINDINGS.md` | Authoritative ranked findings informing router, dual-stream, and budget architecture. |

## Validate The Planning Artifacts

These commands should pass immediately in prep mode:

```bash
./.specify/scripts/bash/check-prerequisites.sh --json
npx markdownlint-cli2 "specs/009-phase9-llm-provider-integration/**/*.md" \
  "specs/009-phase9-llm-provider-integration/*.md" \
  --config .markdownlint.json
```

Expected result:

- `AVAILABLE_DOCS` includes `research.md`, `data-model.md`, `contracts/`, and `quickstart.md`
- markdownlint returns `0 error(s)`

## Planned Build Flow After Implementation

Phase 9 implementation is expected to validate in this order:

```bash
# 1. Core provider-neutral contract and mock behavior (DONE)
cargo test -p mister-smith-llm

# 2. Router, health, and circuit breaker (NEW)
cargo test -p mister-smith-llm -- router health circuit

# 3. Budget enforcement with JetStream KV CAS (NEW, env-gated)
NATS_URL=nats://localhost:4222 cargo test -p mister-smith-llm -- budget --ignored

# 4. ModelEvent serde and forward compatibility (NEW)
cargo test -p mister-smith-llm -- model_event

# 5. Dual-stream backpressure and event classification (NEW, env-gated)
NATS_URL=nats://localhost:4222 cargo test -p mister-smith-llm -- dual_stream --ignored

# 6. MessageEnvelope backward compatibility
cargo test -p mister-smith-transport -- envelope plane stream_class

# 7. Anthropic adapter when credentials are available
ANTHROPIC_API_KEY=... cargo test -p mister-smith-llm --features anthropic -- --ignored

# 8. OpenAI adapter when credentials are available (DONE)
OPENAI_API_KEY=... cargo test -p mister-smith-llm --features openai -- --ignored

# 9. ChatGPT-backed OpenAI path through Codex app-server (DONE)
cargo test -p mister-smith-llm --features openai-chatgpt
cargo test -p mister-smith-app

# 10. Cascade routing (SLM-default / LLM-fallback) (NEW)
cargo test -p mister-smith-llm -- cascade confidence

# 11. Agent bridge and Gate 9 orchestration path
cargo test -p mister-smith-agents --features llm gate9 -- --ignored --nocapture
```

## Usage Sketch

### Completion Through The Shared Interface

```rust
use mister_smith_llm::{CompletionRequest, ModelProvider, MockProvider};

let provider = MockProvider::default();

let response = provider
    .complete(CompletionRequest {
        messages: vec![],
        system: Some("decompose work into subtasks".into()),
        tools: None,
        temperature: None,
        max_tokens: Some(512),
        stop_sequences: None,
        metadata: serde_json::json!({}),
        routing_hint: None,
    })
    .await?;
```

### Routing Through ModelRouter

```rust
use mister_smith_llm::{ModelRouter, RoutingPolicy, ProviderConfig};

let router = ModelRouter::new(
    vec![cheap_provider_config, expensive_provider_config],
    RoutingPolicy::CostOptimized,
);

// Router selects cheapest healthy provider meeting capability requirements
let response = router.complete(request).await?;
```

### Cascade Routing (SLM-Default / LLM-Fallback)

```rust
use mister_smith_llm::{ModelRouter, RoutingPolicy, CascadePolicy, CascadeTier};

let cascade = CascadePolicy {
    tiers: vec![
        CascadeTier { provider_config: slm_7b_config, label: "slm-7b".into() },
        CascadeTier { provider_config: gpt4o_config, label: "llm-gpt4o".into() },
    ],
    escalation_threshold: 0.7,
    max_escalations: 1,
};

let router = ModelRouter::new(
    vec![slm_7b_config, gpt4o_config],
    RoutingPolicy::Cascade(cascade),
);
```

### Consuming ModelEvent (Dual-Stream)

```rust
use mister_smith_llm::ModelEvent;

// Stream actor converts StreamChunk -> ModelEvent
let mut event_stream = stream_actor.subscribe_semantic(); // lossless
while let Some(event) = event_stream.next().await {
    match event {
        ModelEvent::ToolCallCompleted { call_id, name, input } => {
            // Tool calls are lossless — always delivered
            let result = tool_bus.execute_tool_call(&call).await?;
        }
        ModelEvent::TextDelta { text } => {
            // Text deltas may be coalesced on UI stream under backpressure
        }
        ModelEvent::Unknown => {
            // Forward compatibility — unknown events from future provider updates
        }
        _ => {}
    }
}
```

### Tool Definitions From ToolBus

```rust
use mister_smith_agents::ToolBus;

let tool_definitions = tool_bus.to_tool_definitions();
```

### Tool Call Round-Trip

```rust
let tool_result = tool_bus.execute_tool_call(/* caller context */, &tool_call).await?;
```

### ChatGPT Subscription Login And Status

```bash
cargo run -p mister-smith-app -- auth openai-chatgpt login
cargo run -p mister-smith-app -- auth openai-chatgpt status
```

## Gate 9 Scenario

The roadmap Gate 9 validation for Phase 9 is:

1. A Planner receives a high-level task.
2. The Planner calls a real LLM through `ModelProvider` via `ModelRouter`.
3. The `ModelRouter` selects the provider based on routing policy, health, and budget.
4. The response contains a structured subtask decomposition.
5. The Orchestrator assigns subtasks to Worker agents through the existing agent-system flow.
6. Tool calls, when requested by the model, round-trip through the ToolBus.
7. Tool-call events are delivered losslessly via the semantic stream (JetStream).
8. The same flow succeeds with Anthropic/Claude and the OpenAI API-key backend.
9. Budget enforcement prevents budget overruns under concurrent requests.
10. The ChatGPT-backed OpenAI path validates completion and streaming through Codex app-server
    and returns typed unsupported-capability errors for embeddings and tool calling.
11. Routing decisions are recorded for observability.

## Blocker Review

Before implementation, confirm these remain visible as blockers or prerequisites rather than
Phase 9 scope:

- ToolBus permission and audit hardening (Phase 9.1 addresses security aspects)
- Memory metadata and versioning
- Heartbeat receiver and failure detection
- Supervisor delegation to Phase 3 supervision
- Priority mailbox wiring

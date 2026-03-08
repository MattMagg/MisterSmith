# Phase 9 Implementation Handoff — SpecKit Workflow

## Mission

Implement Phase 9 (LLM Provider Integration) for the Mister Smith multi-agent orchestration framework using the SpecKit `/implement` workflow against the completed spec set at `specs/009-phase9-llm-provider-integration/`.

## Orientation

Before writing any code, review these files in order:

1. **CLAUDE.md** — Project overview, workspace structure, technology stack, conventions
2. **ROADMAP.md** — 9-phase build roadmap; Phase 9 is next
3. **specs/009-phase9-llm-provider-integration/spec.md** — The governing specification
4. **specs/009-phase9-llm-provider-integration/plan.md** — Subphase execution plan with design decisions D1-D10
5. **specs/009-phase9-llm-provider-integration/tasks.md** — 44 tasks, dependency-ordered
6. **specs/009-phase9-llm-provider-integration/data-model.md** — All entity definitions and type contracts
7. **specs/009-phase9-llm-provider-integration/contracts/** — ModelProvider, Agent-LLM Bridge, Tool-Calling Bridge contracts

## What Already Exists

Substantial implementation is in place. **Do not rewrite or contradict existing code.**

| Subphase | Status | Tasks | What's Built |
|----------|--------|-------|-------------|
| 9.1 Core Types | DONE | T001-T008 | `mister-smith-llm` crate, `ModelProvider` trait, unified types, `MockProvider`, contract tests |
| 9.3 Providers | PARTIAL | T012-T014A done | `OpenAiProvider` (API-key), `ClaudeSubscriptionProvider` (OAuth Bearer), `OpenAiChatGptProvider` (Codex app-server), app auth CLI |

**Key divergence**: The spec planned `AnthropicProvider` (API-key auth via Anthropic Messages API). The codebase instead has `ClaudeSubscriptionProvider` (OAuth Bearer auth via Claude subscription). These are different providers for different auth flows. Both are valid `ModelProvider` implementations. The remaining `AnthropicProvider` work is tasks T009-T011.

## What Needs Building

### Parallel Track A: Subphase 9.2a — Router + Health + Budget (T031-T037)

New files in `crates/mister-smith-llm/src/`:
- `health.rs` — `HealthStatus`, `CircuitState` enum, circuit breaker state machine
- `budget.rs` — `BudgetNode`, `BudgetPolicy`, JetStream KV CAS reserve/reconcile
- `router.rs` — `ModelRouter`, `RoutingPolicy`, data-plane routing with control-plane KV watch

Add `LlmError::BudgetExhausted` and `LlmError::NoHealthyProvider` to `crates/mister-smith-core/src/error.rs`.

Tests: `router_tests.rs`, `budget_tests.rs` (env-gated with `NATS_URL`)

### Parallel Track B: Subphase 9.2b — Dual-Stream + ModelEvent + Envelope (T042-T047)

New files in `crates/mister-smith-llm/src/`:
- `model_event.rs` — `ModelEvent` enum (28 variants, `#[non_exhaustive]`, `#[serde(other)]`)
- `dual_stream.rs` — `StreamClass`, `BackpressurePolicy`, stream actor converting `StreamChunk` to `ModelEvent`

Transport changes in `crates/mister-smith-transport/src/envelope.rs`:
- Add `MessagePlane` enum (`Data`, `Control`) with `#[non_exhaustive]`
- Add `StreamClass` enum (`Semantic`, `Ui`) with `#[non_exhaustive]`
- Add `plane: Option<MessagePlane>` and `stream_class: Option<StreamClass>` to `MessageEnvelope` with `#[serde(default)]`

Tests: `model_event_tests.rs`, `dual_stream_tests.rs`, transport backward compat tests

### Parallel Track C: Subphase 9.3 Remaining — AnthropicProvider (T009-T011)

New file: `crates/mister-smith-llm/src/providers/anthropic.rs` behind `#[cfg(feature = "anthropic")]`

Tests: `integration/anthropic_tests.rs` (env-gated with `ANTHROPIC_API_KEY`)

### Sequential: Subphase 9.4 — Agent-LLM Bridge (T015-T021)

Depends on 9.2a + 9.2b. Add optional `llm` feature to `mister-smith-agents`. Extend Planner, Critic, Executor roles with provider-backed behavior via `ModelRouter`. Orchestrator consumes structured output unchanged.

### Sequential: Subphase 9.5 — Tool Calling Bridge (T022-T025)

Depends on 9.4. Add `ToolBus::to_tool_definitions()` and `ToolBus::execute_tool_call()`. Gate 9 validation: Planner -> model -> ToolBus -> model -> Orchestrator round-trip.

### Parallel with 9.4: Subphase 9.6 — SLM-Default Routing (T051-T053)

Depends only on 9.2a. `CascadePolicy`, `CascadeTier`, `ConfidenceSignal` in router.rs. Cascade tests.

### Final: Verification (T026-T030)

Workspace-wide clippy, doc generation, CLAUDE.md update.

## Execution Order

```
          9.2a (Router/Health/Budget)  ----+
         /                                 |
9.1 DONE ---  9.2b (Dual-Stream/Event) ---+---> 9.4 (Agent Bridge) ---> 9.5 (Tool Bridge) ---> Verify
         \                                 |
          9.3 (AnthropicProvider)  --------+
          9.6 (SLM-Default) --------------/
              (depends only on 9.2a)
```

Tracks A, B, and C are independent — use parallel agents where possible.

## Critical Design Constraints

1. **Two-layer streaming**: Providers emit `StreamChunk` (4 variants). Stream actors convert to `ModelEvent` (28 variants). These are two layers, not a replacement. Never pass raw `StreamChunk` to agents.

2. **Budget CAS pattern**: Reserve estimated tokens before send via JetStream KV CAS. Reconcile actual usage after completion. Target <1% overrun rate. Handle negative balance gracefully.

3. **Backward compatibility**: All new `MessageEnvelope` fields use `Option<T>` with `#[serde(default)]`. Treat `None` as `Data` plane / `Semantic` stream. Pre-Phase-9 messages must deserialize without error.

4. **Forward compatibility**: `ModelEvent` uses `#[non_exhaustive]` and `#[serde(other)]` on `Unknown` variant. New provider event types must not break existing consumers.

5. **Router architecture**: Data-plane routing uses local in-memory state (sub-millisecond). Control-plane updates arrive via JetStream KV watches. Data plane never blocks on control plane.

6. **Feature gating**: `anthropic`, `openai`, `openai-chatgpt`, `claude-subscription` features in `mister-smith-llm`. `llm` feature in `mister-smith-agents`. Framework builds cleanly without any provider features.

7. **Error taxonomy**: `LlmError` lives in `mister-smith-core/src/error.rs` (like `SecurityError`, `PersistenceError`). Re-exported from `mister-smith-llm`.

## Established Patterns to Follow

| Pattern | Example | Apply To |
|---------|---------|----------|
| Error in core, re-export | `SecurityError` in `core/error.rs` | `LlmError::BudgetExhausted`, `LlmError::NoHealthyProvider` |
| Config in config crate | `PersistenceConfig` | `LlmConfig` if needed |
| Feature flags | `jwt`/`rbac`/`tls`/`audit` in security | `anthropic`/`openai` providers |
| Orphan rule workaround | `from_jwt_error()` free fn | Foreign type conversions |
| `#[non_exhaustive]` enums | Throughout codebase | `MessagePlane`, `StreamClass`, `ModelEvent`, `CircuitState` |
| Env-gated integration tests | `#[ignore]` + `DATABASE_URL`/`NATS_URL` | `ANTHROPIC_API_KEY`, `NATS_URL` for budget |
| Cross-phase bridges | `HeartbeatBridge`, `SecurityBridge` | Agent-LLM bridge |

## Scope Boundaries — Do NOT Build

- Learned routing (RouteLLM, kNN, ONNX embeddings) — Phase 10+
- Step-level intelligence / Process Reward Models — Phase 10
- Guided decoding (XGrammar, Outlines) — Phase 10
- Local model inference / `LocalModelProvider` — Phase 10+
- Disaggregated serving / shared KV cache — Phase 10+
- Streaming content monitors for mid-stream abort — Phase 10
- Inter-agent message auth / AgentSandbox — Phase 9.1 (separate spec)
- Dynamic topology / MaAS — Phase 11
- CRDT coordination — Phase 13

## Validation

After implementation, all of these must pass:

```bash
cargo test -p mister-smith-llm                                    # All LLM tests
cargo test -p mister-smith-transport -- envelope                  # Backward compat
cargo test -p mister-smith-agents --features llm                  # Agent bridge
cargo clippy --workspace -- -D warnings                           # Clean lint
```

Gate 9 acceptance: Planner receives task -> calls ModelProvider via ModelRouter -> model returns structured decomposition -> Orchestrator assigns subtasks -> tool calls round-trip through ToolBus -> same flow works for multiple providers.

## SpecKit Invocation

```
/implement
```

This will read `specs/009-phase9-llm-provider-integration/tasks.md` and execute each task in dependency order, creating files, writing tests, and validating at each checkpoint.

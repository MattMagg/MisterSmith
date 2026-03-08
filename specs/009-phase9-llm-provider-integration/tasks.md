# Tasks: Phase 9 — LLM Provider Integration

**Input**: Design documents from `/specs/009-phase9-llm-provider-integration/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`

**Tests**: Included. Phase 9 requires deterministic contract tests, env-gated provider
integration tests, router/budget/circuit-breaker tests, dual-stream/ModelEvent tests,
ToolBus bridge tests, and Gate 9 orchestration validation.

**Organization**: Tasks are grouped by subphase `9.1` through `9.6` and mapped to
user stories. Phase 7.5 hardening remains visible as blocker context. Security items are
addressed by Phase 9.1 spec at `specs/011-phase9.1-security-hardening/`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel when tasks touch different files and have no dependency edge
- **[Story]**: Which Phase 9 user story the task advances (`US1` through `US7`)
- Include exact file paths in every task description

## Path Conventions

- **Workspace root**: `Cargo.toml`
- **Core shared errors**: `crates/mister-smith-core/src/error.rs`, `crates/mister-smith-core/src/lib.rs`
- **New crate**: `crates/mister-smith-llm/`
- **LLM source**: `crates/mister-smith-llm/src/`
- **LLM tests**: `crates/mister-smith-llm/tests/`
- **Transport**: `crates/mister-smith-transport/src/`
- **App auth surface**: `crates/mister-smith-app/src/`, `crates/mister-smith-app/tests/`
- **Agent integration**: `crates/mister-smith-agents/src/`
- **Agent tests**: `crates/mister-smith-agents/tests/`

## Canonical Architecture Traceability

| Source | Task ranges | Why it matters |
| ------ | ----------- | -------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | `T015`-`T025`, `T028` | Keeps Planner-to-Orchestrator flow and LLM/ToolBus coordination inside existing agent seams. |
| `spec/core-architecture/type-definitions.md` | `T003`-`T008`, `T015`-`T021` | Anchors unified IDs, agent-role typing, and shared error conventions. |
| `spec/core-architecture/async-patterns.md` | `T006`, `T015`-`T025` | Preserves agent-as-tool and ToolBus patterns. |
| `docs/research-output/consolidated/01-model-routing-and-cost-optimization.md` | `T031`-`T041` | Two-plane router, budget enforcement, SLM-default routing. |
| `docs/research-output/consolidated/06-streaming-architecture.md` | `T042`-`T050` | Dual-stream, ModelEvent, backpressure policy. |

## Visible Prerequisites & Blockers (Do Not Absorb Into Phase 9 Scope)

- **Security integration**: Now addressed by Phase 9.1 spec at
  `specs/011-phase9.1-security-hardening/`
- **Memory metadata, timestamps, versions**: remains deferred
- **Heartbeat receiver and failure detection**: remains deferred
- **Supervisor delegation**: remains deferred
- **Priority mailbox wiring**: remains deferred

## Status Reconciliation (2026-03-08)

- This checklist was reconciled against repository paths to avoid false gate-signoff signals.
- Any task that references a missing implementation/test path is now unchecked until the
  deliverable exists (or task scope is rewritten).
- Gate readiness should be assessed from this reconciled checklist plus fresh command outputs in
  `T026`-`T029`, not from prior historical checkmarks.

---

## Subphase 9.1 — Core Types & `MockProvider` (User Story 1, Priority: P1) — DONE

**Status**: Complete. All tasks implemented and tested.

- [x] T001 [US1] Add `crates/mister-smith-llm` to workspace members and add shared provider
  dependencies and feature plumbing in root `Cargo.toml`.
- [x] T002 [US1] Create `crates/mister-smith-llm/Cargo.toml` with feature-gated provider flags.
- [x] T003 [P] [US1] Expand shared error hierarchy in
  `crates/mister-smith-core/src/error.rs` with canonical `LlmError` variants.
- [x] T004 [P] [US1] Create `crates/mister-smith-llm/src/lib.rs` and
  `crates/mister-smith-llm/src/provider.rs` with `ModelProvider` trait.
- [x] T005 [P] [US1] Implement provider-neutral request, response, usage, stop-reason,
  capability, and content types in `crates/mister-smith-llm/src/types.rs` and
  `crates/mister-smith-llm/src/config.rs`.
- [x] T006 [P] [US1] Implement unified tool-calling and streaming surface types in
  `crates/mister-smith-llm/src/tool_schema.rs` and
  `crates/mister-smith-llm/src/streaming.rs`.
- [x] T007 [US1] Implement deterministic mock behavior in
  `crates/mister-smith-llm/src/mock.rs`.
- [x] T008 [US1] Add contract and serialization coverage in
  `crates/mister-smith-llm/tests/mock_tests.rs` and
  `crates/mister-smith-llm/tests/types_tests.rs`.

**Checkpoint**: Complete.

---

## Subphase 9.2a — Two-Plane Router + Health + Budget (User Story 5, Priority: P1) — NEW

**Goal**: Implement the `ModelRouter` with data-plane routing, health-aware circuit breakers,
and hierarchical budget enforcement via JetStream KV CAS.

**Independent Test**: `cargo test -p mister-smith-llm -- router health circuit budget`

### Implementation for User Story 5

- [x] T031 [US5] Add `LlmError::BudgetExhausted` and `LlmError::NoHealthyProvider` variants to
  `crates/mister-smith-core/src/error.rs` and re-export from
  `crates/mister-smith-core/src/lib.rs`.
- [x] T032 [P] [US5] Create `crates/mister-smith-llm/src/health.rs` with `HealthStatus`,
  `CircuitState` enum (`Closed`, `Open`, `HalfOpen`), circuit breaker state machine with
  configurable thresholds (consecutive failures, error rate window, Retry-After honoring).
- [x] T033 [P] [US5] Create `crates/mister-smith-llm/src/budget.rs` with `BudgetNode`,
  `BudgetPolicy` enum (`HardCap`, `SoftCap`, `Conditioned`), JetStream KV CAS
  reserve-before-send and reconcile-after-completion operations.
- [x] T034 [US5] Create `crates/mister-smith-llm/src/router.rs` with `ModelRouter`,
  `RoutingPolicy` enum (`RoundRobin`, `CostOptimized`, `CapabilityMatched`,
  `Cascade(CascadePolicy)`), `RoutingHint`, data-plane routing with local in-memory state,
  and control-plane JetStream KV watch subscription for configuration updates.
- [x] T035 [US5] Add router tests in `crates/mister-smith-llm/tests/router_tests.rs`:
  sub-millisecond routing overhead, round-robin distribution, cost-optimized selection,
  unhealthy provider removal, provider recovery after circuit half-open.
- [x] T036 [US5] Add circuit breaker tests in `crates/mister-smith-llm/tests/router_tests.rs`:
  Closed -> Open on threshold, Open -> HalfOpen on timeout, HalfOpen -> Closed on probe
  success, HalfOpen -> Open on probe failure, Retry-After honoring from 429 responses.
- [x] T037 [US5] Add budget CAS tests in `crates/mister-smith-llm/tests/budget_tests.rs`
  (env-gated with `NATS_URL`): reserve-and-reconcile round-trip, concurrent CAS with <1%
  overrun rate, budget exhaustion behavior per policy (reject vs downgrade), hierarchical
  budget key resolution (org/team/user).

**Checkpoint**: `ModelRouter` routes requests with sub-millisecond overhead, circuit breakers
correctly manage provider health, budget CAS prevents overruns.

---

## Subphase 9.2b — Dual-Stream + ModelEvent + MessageEnvelope (User Story 7, Priority: P1) — PARTIALLY DONE

**Goal**: Implement the `ModelEvent` enum, dual-stream delivery, and `MessageEnvelope` additions.

**Independent Test**: `cargo test -p mister-smith-llm -- model_event dual_stream` and
`cargo test -p mister-smith-transport -- envelope`

### Implementation for User Story 7

- [x] T042 [US7] Create `crates/mister-smith-llm/src/model_event.rs` with `ModelEvent` enum
  (28 variants: 5 lifecycle, 3 text, 4 tool-call, 3 observability, 1 error, 1 heartbeat,
  1 unknown) with `#[non_exhaustive]` and `#[serde(other)]` on `Unknown`.
- [x] T043 [P] [US7] Create `crates/mister-smith-llm/src/dual_stream.rs` with `StreamClass`
  enum (`Semantic`, `Ui`), `BackpressurePolicy` enum (`Lossless`, `Coalescible`, `Droppable`),
  backpressure policy matrix mapping `ModelEvent` variants to policies, and stream actor logic
  converting `StreamChunk` to `ModelEvent`.
- [x] T044 [P] [US7] Add `MessagePlane` enum (`Data`, `Control`) and `StreamClass` enum to
  `crates/mister-smith-transport/src/envelope.rs` (or appropriate module). Add
  `plane: Option<MessagePlane>` and `stream_class: Option<StreamClass>` to `MessageEnvelope`
  with `#[serde(default)]`. Both enums use `#[non_exhaustive]`.
- [ ] T045 [US7] Add `ModelEvent` serde and forward compatibility tests in
  `crates/mister-smith-llm/tests/model_event_tests.rs`: round-trip serialization for all 28
  variants, `Unknown` variant via `#[serde(other)]` for unrecognized input, `#[non_exhaustive]`
  forward compatibility.
- [ ] T046 [US7] Add dual-stream tests in `crates/mister-smith-llm/tests/dual_stream_tests.rs`:
  tool-call events delivered losslessly, text deltas coalesced under backpressure, heartbeats
  dropped under extreme backpressure, backpressure policy matrix enforcement.
- [ ] T047 [US7] Add `MessageEnvelope` backward compatibility tests in
  `crates/mister-smith-transport/tests/`: deserialize pre-Phase-9 envelopes without `plane` or
  `stream_class` fields (both default to `None`), `None` treated as `Data`/`Semantic`
  respectively.

**Checkpoint**: `ModelEvent` covers all event classes with forward compatibility, dual-stream
delivers events per backpressure policy, `MessageEnvelope` changes are backward-compatible.

---

## Subphase 9.3 — Providers (User Story 2, Priority: P1) — PARTIALLY DONE

**Status**: `OpenAiProvider` and `ClaudeSubscriptionProvider` are implemented.
`AnthropicProvider` (API-key) is planned.

### Implementation for User Story 2

- [x] T009 [US2] Add Anthropic provider module wiring in
  `crates/mister-smith-llm/src/providers/mod.rs` and create
  `crates/mister-smith-llm/src/providers/anthropic.rs` behind
  `#[cfg(feature = "anthropic")]`.
- [x] T010 [US2] Implement request serialization, response normalization, streaming, embeddings,
  tool-calling support, and typed error mapping in
  `crates/mister-smith-llm/src/providers/anthropic.rs`.
- [ ] T011 [US2] Add env-gated real-provider coverage in
  `crates/mister-smith-llm/tests/integration/anthropic_tests.rs`.
- [x] T012 [US2] Add OpenAI provider module wiring and create
  `crates/mister-smith-llm/src/providers/openai.rs`.
- [x] T013 [US2] Implement API-key `OpenAiProvider`.
- [x] T014 [US2] Implement `OpenAiChatGptProvider` plus `mister-smith-app` auth subcommands.
- [x] T014A [US2] Add env-gated OpenAI API coverage and app CLI coverage.

**Checkpoint**: All provider adapters conform to the shared contract. `ClaudeSubscriptionProvider`
(OAuth Bearer) coexists with planned `AnthropicProvider` (API-key) under
`ProviderKind::ClaudeSubscription` and `ProviderKind::Anthropic` respectively.

---

## Subphase 9.4 — Agent-LLM Bridge (User Story 3, Priority: P2) — PARTIALLY DONE

**Goal**: Feature-gated bridge from `mister-smith-agents` to `ModelProvider` via `ModelRouter`
for Planner, Critic, and Executor.

### Implementation for User Story 3

- [x] T015 [US3] Add the optional `llm` feature and `mister-smith-llm` dependency wiring in
  `crates/mister-smith-agents/Cargo.toml` and update re-exports.
- [x] T016 [P] [US3] Extend `crates/mister-smith-agents/src/agent.rs` and
  `crates/mister-smith-agents/src/errors.rs` with feature-gated model attachment boundary
  and provider-aware error conversions.
- [x] T017 [P] [US3] Implement Planner role integration in
  `crates/mister-smith-agents/src/roles/planner.rs` — provider-backed subtask decomposition
  via `ModelRouter`, dual-stream consumption of `ModelEvent` items.
- [x] T018 [P] [US3] Implement Critic role integration in
  `crates/mister-smith-agents/src/roles/critic.rs`.
- [x] T019 [P] [US3] Implement Executor role integration in
  `crates/mister-smith-agents/src/roles/executor.rs`.
- [ ] T020 [US3] Update `crates/mister-smith-agents/src/orchestrator.rs` to consume structured
  Planner output through existing scheduler and team paths.
- [ ] T021 [US3] Add feature-gated bridge coverage in
  `crates/mister-smith-agents/tests/role_tests.rs` and
  `crates/mister-smith-agents/tests/team_tests.rs`.

**Checkpoint**: Planner, Critic, and Executor use `ModelProvider` via `ModelRouter` behind the
`llm` feature.

---

## Subphase 9.5 — Tool Calling Bridge (User Story 4, Priority: P2)

**Goal**: Export registered tools as unified definitions and execute model-emitted tool calls
through the ToolBus, with lossless delivery on the semantic stream.

### Implementation for User Story 4

- [x] T022 [US4] Extend `crates/mister-smith-agents/src/tool_bus.rs` with
  `ToolBus::to_tool_definitions()`.
- [x] T023 [US4] Extend `crates/mister-smith-agents/src/tool_bus.rs` with
  `ToolBus::execute_tool_call()` preserving permission, timeout, metrics, and audit boundaries.
- [x] T024 [US4] Add ToolBus bridge coverage in
  `crates/mister-smith-agents/tests/tool_bus_tests.rs`.
- [x] T025 [US4] Add Gate 9 tool-calling coverage in
  `crates/mister-smith-agents/tests/gate9_tests.rs` exercising Planner -> model -> ToolBus ->
  model -> Orchestrator flow.

**Checkpoint**: Tool calls route through ToolBus with lossless delivery on semantic stream.

---

## Subphase 9.6 — SLM-Default Routing Policy (User Story 6, Priority: P2) — NEW

**Goal**: Implement cascade routing with SLM-default / LLM-fallback economics.

### Implementation for User Story 6

- [x] T051 [US6] Implement `CascadePolicy`, `CascadeTier`, and `ConfidenceSignal` types in
  `crates/mister-smith-llm/src/router.rs` with cascade routing logic: attempt tiers in order,
  evaluate confidence, escalate on rejection.
- [x] T052 [US6] Add routing decision logging: `ModelEvent::RoutingDecision` emission for each
  cascade attempt with tier label, confidence score, and escalation reason.
- [x] T053 [US6] Add cascade routing tests in `crates/mister-smith-llm/tests/router_tests.rs`:
  SLM attempt accepted when confidence >= threshold, LLM escalation when confidence < threshold,
  final tier response returned when all tiers exhausted, max escalation limit honored.

**Checkpoint**: Cascade routing attempts cheapest tier first, escalates based on confidence.

---

## Verification & Readiness

- [x] T026 [P] [US2] Run shared crate verification: `cargo test -p mister-smith-llm` and
  `cargo doc -p mister-smith-llm --no-deps`.
- [x] T027 [P] [US2] Run env-gated provider verification for Anthropic and OpenAI integration
  tests, run `cargo test -p mister-smith-app`, and complete manual ChatGPT validation.
- [x] T028 [US3] Run feature-gated agent verification:
  `cargo test -p mister-smith-agents --features llm`.
- [x] T029 [US4] Run workspace hygiene checks: `cargo clippy --workspace -- -D warnings`.
- [x] T030 [US4] Update `CLAUDE.md` implementation status and workspace crate tree to reflect
  completed `mister-smith-llm` phase once Gate 9 verification passes.

---

## Dependencies & Execution Order

### Subphase Dependencies

- **9.1 (Core)**: No Phase 9 dependencies. DONE.
- **9.2a (Router/Health/Budget)**: Depends on `9.1`.
- **9.2b (Dual-Stream/ModelEvent/Envelope)**: Depends on `9.1`.
- **9.3 (Providers)**: Depends on `9.1`. Partially DONE.
- **9.4 (Agent Bridge)**: Depends on `9.1`, `9.2a`, `9.2b`, and Phase 7 baseline.
- **9.5 (Tool Bridge)**: Depends on `9.2a`, `9.2b`, `9.4`.
- **9.6 (SLM-Default)**: Depends on `9.2a`.
- **Verification**: Depends on all subphases.

### Parallel Opportunities

- `T031`, `T032`, `T033` can proceed in parallel after 9.1 (different files).
- `T042`, `T043`, `T044` can proceed in parallel after 9.1 (different files).
- `9.2a` and `9.2b` can proceed in parallel (no dependency between them).
- `9.3` (remaining Anthropic work) can proceed in parallel with `9.2a`/`9.2b`.
- `9.6` can proceed in parallel with `9.3` and `9.4` (only depends on `9.2a`).
- `T017`, `T018`, `T019` can run in parallel once `T015`/`T016` establish the bridge.
- `T026` and `T027` can run in parallel once implementation is complete.

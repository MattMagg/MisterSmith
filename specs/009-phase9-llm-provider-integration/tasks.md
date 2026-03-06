# Tasks: Phase 9 — LLM Provider Integration

**Input**: Design documents from `/specs/009-phase9-llm-provider-integration/`  
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`

**Tests**: Included. Phase 9 requires deterministic contract tests, env-gated Anthropic/OpenAI API
integration tests, stubbed Codex app-server client tests plus manual ChatGPT validation, ToolBus
bridge tests, and Gate 9 orchestration validation.

**Organization**: Tasks are grouped by approved subphase `9.1` through `9.5` and mapped to
the Phase 9 user stories. Phase 7.5 hardening remains visible as blocker context and must
not be redefined as Phase 9 implementation scope.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel when tasks touch different files and have no dependency edge
- **[Story]**: Which Phase 9 user story the task advances (`US1` through `US4`)
- Include exact file paths in every task description

## Path Conventions

- **Workspace root**: `Cargo.toml`
- **Core shared errors**: `crates/mister-smith-core/src/error.rs`, `crates/mister-smith-core/src/lib.rs`
- **New crate**: `crates/mister-smith-llm/`
- **LLM source**: `crates/mister-smith-llm/src/`
- **LLM tests**: `crates/mister-smith-llm/tests/`
- **App auth surface**: `crates/mister-smith-app/src/`, `crates/mister-smith-app/tests/`
- **Agent integration**: `crates/mister-smith-agents/src/`
- **Agent tests**: `crates/mister-smith-agents/tests/`
- **Phase docs**: `CLAUDE.md`

## Canonical Architecture Traceability

This task list inherits the Phase 9 source map from `spec.md` and `plan.md`. When tasks are added,
split, or reordered, preserve the mapping below instead of introducing uncited scope.

| Source | Task ranges | Why it matters |
| ------ | ----------- | -------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | `T015`-`T025`, `T028` | Keeps Planner-to-Orchestrator flow and LLM/ToolBus coordination inside existing agent seams. |
| `spec/data-management/message-schemas.md` §5 | `T015`-`T025` scope notes | Confirms hook-event schemas and `llm.hooks.*` subjects stay deferred. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | `T015`-`T030` scope checks | Keeps Neural/AI Operations work out of the Phase 9 backlog. |
| `spec/core-architecture/type-definitions.md` | `T003`-`T008`, `T015`-`T021` | Anchors unified IDs, agent-role typing, and shared error conventions. |
| `spec/core-architecture/async-patterns.md` | `T006`, `T015`-`T025` | Preserves agent-as-tool and ToolBus patterns for tool export and execution. |
| `spec/core-architecture/coding-standards.md` | `T003`, `T008`, `T011`, `T014`, `T021`, `T024`-`T029` | Requires typed errors, timeout and permission handling, audit posture, and explicit tests. |

## Visible Prerequisites & Blockers (Do Not Absorb Into Phase 9 Scope)

These items stay visible because `9.4` and `9.5` may depend on them, but they are not Phase 9 feature deliverables:

- **Security integration for agent messaging, tool permissions, and audit logging**:
  if `ToolBus` permission or audit boundaries are still incomplete in
  `crates/mister-smith-agents/src/tool_bus.rs`, `crates/mister-smith-security/`, or
  related agent messaging seams, treat that as a blocker for `9.4` and `9.5`
  validation rather than expanding Phase 9 scope.
- **Router balancing strategies (`round-robin`, `least-loaded`)**: do not add router hardening work to Phase 9 tasks; only consume the existing router boundary.
- **Memory metadata, timestamps, versions, and access counts**: do not fold Memory-agent hardening into LLM integration tasks.
- **Heartbeat receiver and failure detection**: missing receiver-side liveness handling remains prerequisite work, not part of provider integration.
- **Supervisor delegation to the Phase 3 supervision system**: do not turn provider-backed role work into a supervisor refactor.
- **Priority mailbox wiring**: do not absorb mailbox-ordering hardening into LLM bridge tasks.

Track the current blocker state in
[`checklists/phase-7-5-readiness.md`](checklists/phase-7-5-readiness.md) instead of opening
hidden Phase 9 implementation tasks.

If any unresolved item above prevents reliable `9.4` or `9.5` validation, report it as a blocker during `/speckit.analyze`.

---

## Phase 1: Subphase 9.1 — Core Types & `MockProvider` (User Story 1, Priority: P1)

**Goal**: Create the provider-neutral `mister-smith-llm` crate, the canonical `ModelProvider` contract, unified LLM types, and a deterministic `MockProvider`.

**Independent Test**: `cargo test -p mister-smith-llm` passes contract and serialization tests with no real-provider feature flags enabled.

### Implementation for User Story 1

- [x] T001 [US1] Add `crates/mister-smith-llm` to workspace members and add shared provider dependencies and feature plumbing in root `Cargo.toml`.
- [x] T002 [US1] Create `crates/mister-smith-llm/Cargo.toml` with feature-gated
  `anthropic`, `openai`, and `openai-chatgpt` provider flags, always-on mock support, and
  dependencies aligned with `specs/009-phase9-llm-provider-integration/plan.md`.
- [x] T003 [P] [US1] Expand the shared error hierarchy in
  `crates/mister-smith-core/src/error.rs` and re-export it from
  `crates/mister-smith-core/src/lib.rs` by adding canonical `LlmError`
  variants consistent with the Phase 9 contracts and
  `spec/core-architecture/type-definitions.md`.
- [x] T004 [P] [US1] Create `crates/mister-smith-llm/src/lib.rs` and
  `crates/mister-smith-llm/src/provider.rs` with crate-level docs, public
  re-exports, and the `ModelProvider` trait for complete, stream, embed,
  `model_id`, and `capabilities`.
- [x] T005 [P] [US1] Implement provider-neutral request, response, usage,
  stop-reason, capability, and content types in
  `crates/mister-smith-llm/src/types.rs` and provider configuration in
  `crates/mister-smith-llm/src/config.rs`.
- [x] T006 [P] [US1] Implement unified tool-calling and streaming surface
  types in `crates/mister-smith-llm/src/tool_schema.rs` and
  `crates/mister-smith-llm/src/streaming.rs`, including JSON Schema-backed
  `ToolDefinition`, `ToolCall`, `ToolResult`, and ordered `StreamChunk`
  handling.
- [x] T007 [US1] Implement deterministic mock behavior in
  `crates/mister-smith-llm/src/mock.rs` for completion, streaming,
  embeddings, and tool-calling flows without network access.
- [x] T008 [US1] Add contract and serialization coverage in
  `crates/mister-smith-llm/tests/mock_tests.rs` and
  `crates/mister-smith-llm/tests/types_tests.rs`, including
  unsupported-capability and typed-error assertions.

**Checkpoint**: `mister-smith-llm` builds with no real-provider features, `MockProvider` exercises the full shared contract, and no provider-specific public types leak outside provider modules.

---

## Phase 2: Subphase 9.2 — Anthropic Provider (User Story 2, Priority: P1)

**Goal**: Add a feature-gated `AnthropicProvider` implementing the shared Phase 9 contract.

**Independent Test**: `ANTHROPIC_API_KEY=... cargo test -p mister-smith-llm --features anthropic -- --ignored` passes Anthropic integration coverage using the shared request and response types.

### Implementation for User Story 2 (Anthropic)

- [ ] T009 [US2] Add Anthropic provider module wiring in
  `crates/mister-smith-llm/src/providers/mod.rs` and create
  `crates/mister-smith-llm/src/providers/anthropic.rs` behind
  `#[cfg(feature = "anthropic")]`.
- [ ] T010 [US2] Implement request serialization, response normalization,
  streaming, embeddings, tool-calling support, and typed error mapping in
  `crates/mister-smith-llm/src/providers/anthropic.rs` without exposing
  Anthropic-native payload types outside the provider module.
- [ ] T011 [US2] Add env-gated real-provider coverage in
  `crates/mister-smith-llm/tests/integration/anthropic_tests.rs` for
  completions, streaming, embeddings, tool use, and retryable authentication
  or rate-limit failures.

**Checkpoint**: Anthropic behavior conforms to the shared contract and remains feature-gated.

---

## Phase 3: Subphase 9.3 — OpenAI Providers (User Story 2, Priority: P1)

**Goal**: Add feature-gated OpenAI-family backends for API-key and ChatGPT-subscription access
while keeping the same public contract as `AnthropicProvider` for supported capabilities.

**Independent Test**: `OPENAI_API_KEY=... cargo test -p mister-smith-llm --features openai -- --ignored`
passes OpenAI integration coverage using the same public request and response
types used for Anthropic, and manual ChatGPT validation succeeds through
`cargo test -p mister-smith-llm --features openai-chatgpt` plus
`cargo test -p mister-smith-app`.

### Implementation for User Story 2 (OpenAI)

- [X] T012 [US2] Add OpenAI provider module wiring in
  `crates/mister-smith-llm/src/providers/mod.rs`, create
  `crates/mister-smith-llm/src/providers/openai.rs` behind
  `#[cfg(feature = "openai")]`, create
  `crates/mister-smith-llm/src/providers/openai_chatgpt.rs` behind
  `#[cfg(feature = "openai-chatgpt")]`, and add the shared Codex app-server
  client module in `crates/mister-smith-llm/src/app_server.rs`.
- [X] T013 [US2] Implement API-key `OpenAiProvider` request serialization,
  response normalization, streaming, embeddings, tool-calling support, and
  shared error mapping in `crates/mister-smith-llm/src/providers/openai.rs`
  without introducing OpenAI-specific public types.
- [X] T014 [US2] Implement `OpenAiChatGptProvider` plus `mister-smith-app`
  `auth openai-chatgpt login` and `status` support in
  `crates/mister-smith-llm/src/providers/openai_chatgpt.rs`,
  `crates/mister-smith-llm/src/app_server.rs`,
  `crates/mister-smith-app/src/main.rs`, and
  `crates/mister-smith-app/Cargo.toml`, including typed authentication
  failures, completion, streaming, and `UnsupportedCapability` behavior for
  embeddings and tool calling.
- [X] T014A [US2] Add env-gated OpenAI API coverage in
  `crates/mister-smith-llm/tests/integration/openai_tests.rs`, add stubbed
  Codex app-server coverage in `crates/mister-smith-llm/tests/`, and add app
  CLI coverage in `crates/mister-smith-app/tests/` for login, status, missing
  auth, and unsupported-capability paths.

**Checkpoint**: OpenAI API-key behavior matches the shared contract, the ChatGPT-backed path works
through Codex app-server for completion and streaming with typed unsupported-capability behavior for
embeddings and tool calling, and both can be swapped with Anthropic without call-site changes
outside provider selection or app auth commands.

---

## Phase 4: Subphase 9.4 — Agent-LLM Bridge (User Story 3, Priority: P2)

**Goal**: Add a feature-gated bridge from `mister-smith-agents` to `ModelProvider` for Planner, Critic, and Executor while keeping the Orchestrator provider-neutral.

**Independent Test**: `cargo test -p mister-smith-agents --features llm planner` validates structured Planner decomposition through the shared orchestration flow with provider-backed role logic.

### Implementation for User Story 3

- [ ] T015 [US3] Add the optional `llm` feature and `mister-smith-llm`
  dependency wiring in `crates/mister-smith-agents/Cargo.toml` and update
  `crates/mister-smith-agents/src/lib.rs` re-exports to compile cleanly with
  and without the feature.
- [ ] T016 [P] [US3] Extend `crates/mister-smith-agents/src/agent.rs` and
  `crates/mister-smith-agents/src/errors.rs` with the feature-gated model
  attachment boundary and provider-aware error conversions required by the
  Phase 9 contracts.
- [ ] T017 [P] [US3] Implement Planner role integration in
  `crates/mister-smith-agents/src/roles/planner.rs` so a configured
  `ModelProvider` produces structured subtask decomposition without leaking
  provider-specific types.
- [ ] T018 [P] [US3] Implement Critic role integration in
  `crates/mister-smith-agents/src/roles/critic.rs` so provider-backed
  evaluation returns structured feedback through existing agent error
  handling.
- [ ] T019 [P] [US3] Implement Executor role integration in
  `crates/mister-smith-agents/src/roles/executor.rs` so model-backed
  execution can participate in tool-calling loops without bypassing current
  timeout and error semantics.
- [ ] T020 [US3] Update `crates/mister-smith-agents/src/orchestrator.rs` to
  consume structured Planner output through existing scheduler and team paths
  while keeping provider selection confined to the LLM boundary.
- [ ] T021 [US3] Add feature-gated bridge coverage in
  `crates/mister-smith-agents/tests/role_tests.rs` and
  `crates/mister-smith-agents/tests/team_tests.rs` for Planner-driven
  decomposition, Orchestrator assignment, and provider swapping between
  Anthropic and OpenAI-compatible test doubles.

**Checkpoint**: Planner, Critic, and Executor can use `ModelProvider` behind the `llm` feature while the Orchestrator and broader agent system stay provider-neutral.

**Blocker Reminder**: If unresolved security integration, heartbeat reception,
supervisor delegation, or priority-mailbox wiring prevents reliable role
behavior, stop and report a blocker instead of expanding Phase 9 scope.

---

## Phase 5: Subphase 9.5 — Tool Calling Bridge (User Story 4, Priority: P2)

**Goal**: Export registered tools as unified tool definitions and execute model-emitted tool calls through the existing ToolBus boundary.

**Independent Test**: `cargo test -p mister-smith-agents --features llm tool_bus gate9 -- --ignored` validates model -> ToolBus -> model round-trips plus negative permission and timeout paths.

### Implementation for User Story 4

- [ ] T022 [US4] Extend `crates/mister-smith-agents/src/tool_bus.rs` with `ToolBus::to_tool_definitions()` that exports registered tools as stable, provider-neutral JSON Schema definitions.
- [ ] T023 [US4] Extend `crates/mister-smith-agents/src/tool_bus.rs` with
  `ToolBus::execute_tool_call()` that resolves model-emitted tool requests
  through the existing ToolBus invocation path, preserving current
  permission, timeout, metrics, and audit boundaries.
- [ ] T024 [US4] Add ToolBus bridge coverage in
  `crates/mister-smith-agents/tests/tool_bus_tests.rs` for export shape,
  successful execution, permission denial, timeout behavior, and error
  mapping.
- [ ] T025 [US4] Add Gate 9 tool-calling coverage in
  `crates/mister-smith-agents/tests/gate9_tests.rs` that exercises Planner ->
  model -> ToolBus -> model -> Orchestrator flow with feature-gated
  Anthropic and OpenAI provider selection.

**Checkpoint**: Tool export and execution stay inside the existing ToolBus boundary, and model-initiated tool use does not create a provider-specific execution path.

**Blocker Reminder**: If permission or audit hardening is still unverified, `9.5` is blocked. Do not absorb those hardening items into Phase 9 implementation.

---

## Phase 6: Verification & Readiness

**Purpose**: Prove the full Phase 9 implementation without drifting into deferred scope.

- [ ] T026 [P] [US2] Run shared crate verification:
  `cargo test -p mister-smith-llm` and
  `cargo doc -p mister-smith-llm --no-deps`; fix any public API or rustdoc
  issues in `crates/mister-smith-llm/src/`.
- [ ] T027 [P] [US2] Run env-gated provider verification for
  `crates/mister-smith-llm/tests/integration/anthropic_tests.rs` and
  `crates/mister-smith-llm/tests/integration/openai_tests.rs`, run
  `cargo test -p mister-smith-app`, and complete manual ChatGPT login or
  status validation through Codex app-server, confirming identical public
  contract usage across supported providers.
- [ ] T028 [US3] Run feature-gated agent verification:
  `cargo test -p mister-smith-agents --features llm`, including
  `crates/mister-smith-agents/tests/role_tests.rs`, `team_tests.rs`,
  `tool_bus_tests.rs`, and `gate9_tests.rs`.
- [ ] T029 [US4] Run workspace hygiene checks:
  `cargo clippy --workspace -- -D warnings`; ensure Phase 9 changes preserve
  tool permission, testing, and explicit error-handling expectations from
  `spec/core-architecture/coding-standards.md`.
- [ ] T030 [US4] Update `CLAUDE.md` implementation status and workspace crate tree to reflect the completed `mister-smith-llm` phase once Gate 9 verification passes.

---

## Dependencies & Execution Order

### Subphase Dependencies

- **9.1 (Phase 1)**: No Phase 9 dependencies. This is the blocking foundation.
- **9.2 (Phase 2)**: Depends on `9.1`.
- **9.3 (Phase 3)**: Depends on `9.1`.
- **9.4 (Phase 4)**: Depends on `9.1` and the existing Phase 7 baseline in `mister-smith-agents`.
- **9.5 (Phase 5)**: Depends on `9.2` or `9.3`, plus `9.4`.
- **Verification (Phase 6)**: Depends on `9.1` through `9.5`.

### Blocker-Sensitive Dependencies

- `9.4` and `9.5` depend on existing Phase 7 seams remaining valid without silently pulling in Phase 7.5 hardening work.
- Unresolved tool permission or audit wiring blocks `T023` through `T025`.
- Unresolved heartbeat reception, supervisor delegation, or priority mailbox wiring can block `T020`, `T021`, or `T025` if the Gate 9 workflow cannot be validated honestly.

### Parallel Opportunities

- `T003`, `T004`, `T005`, and `T006` can proceed in parallel after workspace scaffolding begins.
- `T009` through `T014` can run as two parallel provider tracks after `9.1` stabilizes.
- `T017`, `T018`, and `T019` can run in parallel once `T015` and `T016` establish the shared agent bridge boundary.
- `T022` and `T024` can run in parallel after the ToolBus bridge API is defined.
- `T026` and `T027` can run in parallel once the `mister-smith-llm` crate is implementation-complete.

---

## Implementation Strategy

### MVP First

1. Complete `9.1` so the shared contract and deterministic mock behavior are stable.
2. Land Anthropic and OpenAI behind the same public surface in `9.2` and `9.3`.
3. Wire Planner-first provider-backed orchestration in `9.4`, then extend Critic and Executor.
4. Finish ToolBus export and execution in `9.5`.
5. Run Gate 9 verification only after blocker-sensitive Phase 7.5 dependencies have been checked and either cleared or reported.

### Scope Discipline

- Do not implement hook events or `llm.hooks.*` subjects.
- Do not implement `LlmTaskOutputParser` routing.
- Do not add Neural/AI Operations agents.
- Do not add prompt-framework, RAG, guardrail, or non-MVP provider work.
- Do not turn Phase 7.5 hardening into hidden Phase 9 implementation tasks.

# Tasks: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

**Input**: Design documents from `/specs/012-phase10-frontier-autonomy/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`

**Tests**: Included. Phase 10 requires deterministic execution-graph/topology tests, branch
checkpoint and resume tests, managed memory/context budget tests, Guard/Advisor classification
tests, autonomy status view tests, delegation/provenance enforcement tests, and workspace build
verification.

**Organization**: Tasks are grouped by subphase `10.0` through `10.6` and mapped to user stories
`US1` through `US4`. Phase 9 and Phase 9.1 remain explicit prerequisites rather than hidden scope.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel when tasks touch different files and have no dependency edge
- **[Story]**: Which Phase 10 user story the task advances (`US1` through `US4`)
- Include exact file paths in every task description

## Path Conventions

- **Workspace root**: `Cargo.toml`
- **Core shared types**: `crates/mister-smith-core/src/`, `crates/mister-smith-core/tests/`
- **Events**: `crates/mister-smith-events/src/`, `crates/mister-smith-events/tests/`
- **Agents source**: `crates/mister-smith-agents/src/`
- **Agents tests**: `crates/mister-smith-agents/tests/`
- **LLM source**: `crates/mister-smith-llm/src/`
- **LLM tests**: `crates/mister-smith-llm/tests/`
- **Persistence source**: `crates/mister-smith-persistence/src/`
- **Persistence tests**: `crates/mister-smith-persistence/tests/`
- **Security source**: `crates/mister-smith-security/src/`
- **Security tests**: `crates/mister-smith-security/tests/`
- **App source**: `crates/mister-smith-app/src/`
- **App tests**: `crates/mister-smith-app/tests/`
- **Deploy assets**: `deploy/`

## Canonical Architecture Traceability

| Source | Task ranges | Why it matters |
| ------ | ----------- | -------------- |
| `spec/data-management/agent-orchestration.md` | `T005`-`T021`, `T024`, `T034`, `T037` | Keeps planner, router, role, and context-management behavior inside existing agent seams. |
| `spec/core-architecture/supervision-trees.md` | `T009`-`T015`, `T022`-`T031` | Preserves OTP-style failure isolation and targeted intervention boundaries. |
| `spec/operations/observability-monitoring-framework.md` | `T003`, `T012`, `T025`, `T027`-`T031` | Grounds typed autonomy state, operator views, and deployment alerts. |
| `docs/research-output/consolidated/03-supervision-and-resilience.md` | `T009`-`T015`, `T022`-`T031` | Grounds branch-local recovery, Guard/Advisor behavior, and checkpoint-aware remediation. |
| `docs/research-output/consolidated/07-memory-and-context.md` | `T016`-`T021` | Grounds role-aware context routing, summaries, consolidation, and checkpoint-ready snapshots. |
| `docs/research-output/consolidated/04-security-and-trust.md` | `T032`-`T037` | Grounds capability attenuation, provenance, and operator-visible autonomy limits. |

## Visible Prerequisites & Deferred Scope (Do Not Absorb Into Phase 10)

- **Phase 9 substrate**: `ModelEvent`, dual-stream delivery, provider-neutral routing, and budget
  primitives remain authoritative inputs to Phase 10 rather than being redesigned here.
- **Phase 9.1 substrate**: message signing, Auth Callout, state validation, sandboxing, and
  delegation-chain groundwork remain the security boundary Phase 10 extends.
- **Deferred work**: learned routing, guided/speculative decoding, local inference, consensus/CRDT
  suites, auction-based orchestration, and ML/eBPF anomaly detection stay out of Phase 10 tasks.

---

## Subphase 10.0 — Foundational Autonomy Types (Blocking Prerequisites)

**Goal**: Introduce shared autonomy identifiers, typed events, and dependency tooling before user
story work begins.

- [ ] T001 Add `petgraph` workspace dependency plumbing in `Cargo.toml` and `crates/mister-smith-agents/Cargo.toml`.
- [ ] T002 [P] Create shared autonomy IDs, enums, and errors in
  `crates/mister-smith-core/src/autonomy.rs`, extend
  `crates/mister-smith-core/src/ids.rs`, `crates/mister-smith-core/src/enums.rs`,
  `crates/mister-smith-core/src/error.rs`, and re-export from
  `crates/mister-smith-core/src/lib.rs`.
- [ ] T003 [P] Create typed autonomy events and summaries in `crates/mister-smith-events/src/autonomy.rs`, update `crates/mister-smith-events/src/types.rs`, and re-export from `crates/mister-smith-events/src/lib.rs`.
- [ ] T004 Add foundational compile and serialization coverage in `crates/mister-smith-core/tests/trait_compilation_tests.rs` and create `crates/mister-smith-events/tests/autonomy_event_tests.rs`.

**Checkpoint**: Shared autonomy types and event envelopes are stable enough for all user stories.

---

## Subphase 10.1 — Execution Graph + Topology Compiler (User Story 1, Priority: P1)

**Goal**: Normalize planner output into an explicit `ExecutionGraph`, reject invalid graphs, and
select topology deterministically before dispatch.

**Independent Test**: Submit parallel, sequential, and mixed-dependency workflow plans and verify
the compiler selects a compatible topology, rejects cycles, and preserves dependency order.

### US1 Graph Compilation Tasks

- [ ] T005 [US1] Create `crates/mister-smith-agents/src/execution_graph.rs` with
  `ExecutionGraph`, `ExecutionNode`, `ExecutionEdge`, graph-state validation, and
  re-export from `crates/mister-smith-agents/src/lib.rs`.
- [ ] T006 [P] [US1] Create `crates/mister-smith-agents/src/topology.rs` with `TopologyCompiler`, `TopologySignals`, deterministic topology-selection policy, and rationale recording.
- [ ] T007 [P] [US1] Extend `crates/mister-smith-agents/src/roles/planner.rs`,
  `crates/mister-smith-agents/src/roles/coordinator.rs`, and
  `crates/mister-smith-agents/src/orchestrator.rs` so planner output is normalized into a
  validated `ExecutionGraph` before dispatch.
- [ ] T008 [US1] Add graph validation and topology selection coverage in `crates/mister-smith-agents/tests/execution_graph_tests.rs` and `crates/mister-smith-agents/tests/topology_tests.rs`.

**Checkpoint**: Planner output is compiled into a valid `ExecutionGraph` with explicit topology
selection and rationale.

---

## Subphase 10.2 — Branch Checkpointing + Resilience-Aware Routing (User Story 1, Priority: P1)

**Goal**: Persist branch progress, attach health/budget/profile signals to dispatch decisions, and
resume or reassign failed work without replaying completed branches.

**Independent Test**: Fail one branch inside a mixed-dependency workflow with a saved checkpoint.
Verify only the affected branch resumes or reassigns while completed branches remain intact.

### US1 Recovery & Routing Tasks

- [ ] T009 [US1] Create `crates/mister-smith-agents/src/branch_checkpoint.rs` with
  branch-local checkpoint capture, resume, and reassignment helpers, then export it from
  `crates/mister-smith-agents/src/lib.rs`.
- [ ] T010 [P] [US1] Extend `crates/mister-smith-persistence/src/kv/state.rs`,
  `crates/mister-smith-persistence/src/repository/task.rs`, and
  `crates/mister-smith-persistence/src/hybrid/manager.rs` to persist
  `BranchCheckpoint` state and resume metadata.
- [ ] T011 [P] [US1] Create `crates/mister-smith-agents/src/profile.rs` and update
  `crates/mister-smith-agents/src/scheduler.rs`,
  `crates/mister-smith-agents/src/roles/router.rs`, and
  `crates/mister-smith-agents/src/orchestrator.rs` to use health, budget,
  dependency-depth, and profile signals for branch dispatch decisions.
- [ ] T012 [US1] Emit checkpoint and routing-rationale events in `crates/mister-smith-events/src/autonomy.rs` and integrate publishers in `crates/mister-smith-agents/src/orchestrator.rs`.
- [ ] T013 [US1] Add branch checkpoint and reassignment coverage in `crates/mister-smith-agents/tests/checkpoint_tests.rs` and `crates/mister-smith-persistence/tests/repository_tests.rs`.
- [ ] T014 [US1] Add resilience-aware routing coverage in `crates/mister-smith-agents/tests/topology_tests.rs` and `crates/mister-smith-agents/tests/team_tests.rs`.
- [ ] T015 [US1] Add Gate 10 workflow coverage in `crates/mister-smith-agents/tests/gate10_tests.rs` exercising mixed-dependency execution, checkpoint resume, and invalid-graph rejection.

**Checkpoint**: Branch-local recovery and resilience-aware dispatch work without re-running
completed branches.

---

## Subphase 10.3 — Managed Memory / Context Manager (User Story 2, Priority: P1)

**Goal**: Add managed memory fragments, role-aware context assembly, async consolidation, and
checkpoint-ready snapshots over the existing Phase 6 storage substrate.

**Independent Test**: Run a workflow that exceeds role budgets and verify summarization, paging,
metadata-preserving consolidation, and snapshot-based resume without replaying full history.

### US2 Memory Management Tasks

- [ ] T016 [US2] Create `crates/mister-smith-persistence/src/memory/mod.rs`,
  `crates/mister-smith-persistence/src/memory/fragment.rs`,
  `crates/mister-smith-persistence/src/memory/manager.rs`,
  `crates/mister-smith-persistence/src/memory/snapshot.rs`, and export the new module from
  `crates/mister-smith-persistence/src/lib.rs`.
- [ ] T017 [P] [US2] Create `crates/mister-smith-persistence/src/memory/consolidation.rs`
  and extend `crates/mister-smith-persistence/src/repository/agent.rs` and
  `crates/mister-smith-persistence/src/repository/task.rs` with fragment metadata,
  paging, and snapshot persistence APIs.
- [ ] T018 [P] [US2] Create `crates/mister-smith-agents/src/context_manager.rs` and
  integrate role-aware context assembly into
  `crates/mister-smith-agents/src/roles/planner.rs`,
  `crates/mister-smith-agents/src/roles/executor.rs`,
  `crates/mister-smith-agents/src/roles/critic.rs`, and
  `crates/mister-smith-agents/src/roles/memory.rs`.
- [ ] T019 [US2] Update `crates/mister-smith-agents/src/branch_checkpoint.rs` and
  `crates/mister-smith-persistence/src/memory/snapshot.rs` so branch resume reconstructs
  checkpoint-ready context snapshots instead of replaying full history.
- [ ] T020 [US2] Add managed-memory and context-budget tests in `crates/mister-smith-persistence/tests/memory_manager_tests.rs` and `crates/mister-smith-agents/tests/context_manager_tests.rs`.
- [ ] T021 [US2] Extend `crates/mister-smith-persistence/tests/performance_tests.rs` with context-reduction and async-consolidation assertions that prove `SC-202` and `SC-203`.

**Checkpoint**: Context delivery is bounded, role-aware, metadata-rich, and checkpoint-ready.

---

## Subphase 10.4 — Guard / Advisor Supervision (User Story 3, Priority: P1)

**Goal**: Add predictive supervision that classifies failures, consumes step-level degradation
signals, and applies targeted interventions before graph-wide restart.

**Independent Test**: Induce transient, structural, streaming, and semantic degradation and verify
the Guard layer chooses targeted interventions and emits operator-visible records.

### US3 Guard Tasks

- [ ] T022 [US3] Create `crates/mister-smith-agents/src/guard.rs` and
  `crates/mister-smith-agents/src/intervention.rs` with failure taxonomy, decision
  policies, intervention application, and re-export them from
  `crates/mister-smith-agents/src/lib.rs`.
- [ ] T023 [P] [US3] Extend `crates/mister-smith-llm/src/model_event.rs` and create `crates/mister-smith-llm/src/stream_monitor.rs` to emit step-boundary and degradation signals usable by the Guard layer.
- [ ] T024 [P] [US3] Integrate Guard decisions into
  `crates/mister-smith-agents/src/orchestrator.rs`,
  `crates/mister-smith-agents/src/roles/monitor.rs`,
  `crates/mister-smith-agents/src/roles/supervisor.rs`, and
  `crates/mister-smith-agents/src/scheduler.rs`.
- [ ] T025 [US3] Extend `crates/mister-smith-agents/src/profile.rs` and `crates/mister-smith-events/src/autonomy.rs` with supervisory profile snapshots, failure evidence, and intervention summaries.
- [ ] T026 [US3] Add Guard and stream-monitor tests in `crates/mister-smith-agents/tests/guard_tests.rs` and `crates/mister-smith-llm/tests/stream_monitor_tests.rs`.

**Checkpoint**: Predictive supervision exists as a typed, branch-local layer above OTP restart
semantics.

---

## Subphase 10.5 — Operator Autonomy View (User Story 3, Priority: P1)

**Goal**: Surface topology state, checkpoint lineage, context pressure, routing rationale, and
intervention history through typed operator-facing autonomy views.

**Independent Test**: Inspect a running workflow and verify topology rationale, branch state,
checkpoint lineage, memory pressure, and intervention history are visible without raw log access.

### US3 Operator View Tasks

- [ ] T027 [US3] Create `crates/mister-smith-app/src/autonomy.rs` and wire
  autonomy-status inspection commands in `crates/mister-smith-app/src/main.rs` and
  `crates/mister-smith-app/src/bootstrap.rs`.
- [ ] T028 [P] [US3] Extend `crates/mister-smith-app/src/observability.rs`,
  `crates/mister-smith-events/src/bus.rs`, and
  `crates/mister-smith-events/src/types.rs` to assemble `AutonomyStatusView` from typed
  topology, checkpoint, memory-pressure, and intervention events.
- [ ] T029 [P] [US3] Add autonomy dashboards and alerts in `deploy/grafana/` and `deploy/prometheus/` for topology choice, branch health, checkpoint staleness, context pressure, and intervention spikes.
- [ ] T030 [US3] Create `crates/mister-smith-app/tests/autonomy_status_tests.rs` and
  extend `crates/mister-smith-events/tests/autonomy_event_tests.rs` to verify
  operator-visible rationale and intervention history.
- [ ] T031 [US3] Extend `crates/mister-smith-agents/tests/gate10_tests.rs` with transient, streaming, and semantic degradation scenarios that remain operator-visible without raw log inspection.

**Checkpoint**: Operators can inspect the autonomy control plane directly from typed state.

---

## Subphase 10.6 — Bounded Delegation + Provenance (User Story 4, Priority: P2)

**Goal**: Enforce bounded delegation, expiry, revocation, and provenance tracing for privileged
autonomous actions without replacing the Phase 9.1 security substrate.

**Independent Test**: Execute a privileged multi-agent workflow and verify valid capability chains
allow execution while expired, revoked, or broken chains are blocked and surfaced to operators.

### US4 Delegation Tasks

- [ ] T032 [US4] Create `crates/mister-smith-security/src/delegation.rs` and re-export
  it from `crates/mister-smith-security/src/lib.rs` with issue, validate, and revoke
  capability services.
- [ ] T033 [P] [US4] Extend `crates/mister-smith-security/src/jwt/claims.rs`,
  `crates/mister-smith-security/src/auth_callout.rs`, and
  `crates/mister-smith-security/src/audit/events.rs` with bounded delegation scope,
  expiry, revocation, and invalid-chain rejection.
- [ ] T034 [P] [US4] Integrate capability checks into
  `crates/mister-smith-agents/src/agent.rs`,
  `crates/mister-smith-agents/src/tool_bus.rs`,
  `crates/mister-smith-agents/src/orchestrator.rs`, and
  `crates/mister-smith-agents/src/branch_checkpoint.rs` so privileged actions carry
  provenance lineage.
- [ ] T035 [P] [US4] Extend `crates/mister-smith-app/src/autonomy.rs`,
  `crates/mister-smith-app/src/observability.rs`, and
  `crates/mister-smith-events/src/autonomy.rs` to surface delegation chains and
  operator-visible rejection reasons.
- [ ] T036 [US4] Create `crates/mister-smith-security/tests/delegation_tests.rs` and
  extend `crates/mister-smith-security/tests/jwt_tests.rs` with valid, expired, revoked,
  and cyclic-chain coverage.
- [ ] T037 [US4] Extend `crates/mister-smith-agents/tests/gate10_tests.rs` and
  `crates/mister-smith-app/tests/autonomy_status_tests.rs` with privileged workflow
  audit scenarios proving full provenance reconstruction.

**Checkpoint**: Delegated privileged execution is enforceable, revocable, and operator-auditable.

---

## Verification & Readiness

- [ ] T038 [P] Run `cargo test -p mister-smith-agents` after `T005`-`T015`, `T022`-`T031`, and `T034`-`T037`.
- [ ] T039 [P] Run `cargo test -p mister-smith-persistence` and `cargo test -p mister-smith-security` after `T010`, `T016`-`T021`, and `T032`-`T036`.
- [ ] T040 [P] Run `cargo test -p mister-smith-llm` and `cargo test -p mister-smith-core` after `T002`-`T004` and `T023`.
- [ ] T041 [P] Run `cargo test -p mister-smith-app` after `T027`-`T035` and verify deploy artifact syntax for `deploy/grafana/` and `deploy/prometheus/`.
- [ ] T042 Update `ROADMAP.md`, `CLAUDE.md`, and `README.md` with Phase 10 implementation status, then run `cargo build --workspace` and the scenarios in `specs/012-phase10-frontier-autonomy/quickstart.md`.

---

## Dependencies & Execution Order

### Subphase Dependencies

- **10.0 Foundational**: No implementation dependencies; blocks all downstream work.
- **10.1 Execution Graph + Topology Compiler**: Depends on 10.0.
- **10.2 Branch Checkpointing + Resilience-Aware Routing**: Depends on 10.1.
- **10.3 Managed Memory / Context Manager**: Depends on 10.0 and the branch identity introduced in 10.1.
- **10.4 Guard / Advisor Supervision**: Depends on 10.1 and stable Phase 9 stream surfaces.
- **10.5 Operator Autonomy View**: Depends on 10.2, 10.3, and 10.4 because it aggregates their typed state.
- **10.6 Bounded Delegation + Provenance**: Depends on 10.0 and the Phase 9.1 delegation-chain substrate; its operator surfacing benefits from 10.5 but execution-time enforcement can start earlier.
- **Verification**: Depends on all implemented subphases and documentation updates.

### User Story Dependencies

- **US1 (P1)**: Starts after 10.0; defines the execution control plane required by later stories.
- **US2 (P1)**: Starts after 10.0 and integrates with US1 branch identity, but remains independently testable once memory surfaces exist.
- **US3 (P1)**: Starts after US1 graph/topology work and Phase 9 streaming stability; operator views depend on US1/US2/US3 event emission, not on US4.
- **US4 (P2)**: Starts after 10.0 and Phase 9.1 foundations; it integrates with US3 operator views but can enforce capabilities before full operator surfacing is complete.

### Parallel Opportunities

- `T002` and `T003` can proceed in parallel after `T001`.
- `T005` and `T006` can proceed in parallel after 10.0 stabilizes.
- `T017` and `T018` can proceed in parallel once `T016` defines the memory module boundaries.
- `T023` and `T024` can proceed in parallel after `T022` defines the Guard API.
- `T028` and `T029` can proceed in parallel after `T027` defines the autonomy status surface.
- `T033`, `T034`, and `T035` can proceed in parallel after `T032` defines the delegation service contract.

## Implementation Strategy

### MVP First

1. Complete 10.0 foundational autonomy types.
2. Complete 10.1 and 10.2 for `US1`.
3. Validate mixed-dependency execution, topology selection, and branch-local recovery via `T015`.
4. Stop and assess whether the control plane is stable before layering memory, supervision, or delegation.

### Incremental Delivery

1. Deliver `US1` to establish execution graph correctness and branch-local recovery.
2. Add `US2` to bound context growth and stabilize resume behavior.
3. Add `US3` to make autonomy decisions observable and operator-coupled.
4. Add `US4` to enforce bounded delegation once operator visibility is in place.

### Parallel Team Strategy

1. One engineer completes 10.0 and 10.1 to lock graph and event contracts.
2. A second engineer can begin 10.3 once graph/branch identity is stable.
3. A third engineer can begin 10.4, then hand off 10.5 once Guard events are emitting.
4. Delegation work in 10.6 can run alongside 10.5 after shared autonomy types are fixed.

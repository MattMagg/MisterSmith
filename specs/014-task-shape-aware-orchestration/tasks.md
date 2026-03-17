# Tasks: Task-Shape-Aware Orchestration and Dynamic Team Sizing

**Input**: Design documents from `/specs/014-task-shape-aware-orchestration/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. This packet requires targeted adaptive-team tests in `mister-smith-agents`,
typed autonomy status tests in `mister-smith-events` and `mister-smith-app`, a deterministic
evaluation harness, and workspace compile verification.

**Organization**: Tasks are grouped by bounded implementation checkpoints so the shared adaptive
contract lands first, then disjoint Symphony-safe lanes can proceed without overlapping write
ownership.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already landed and the write set is disjoint from every other active lane
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in every task description

## Path Conventions

- **Core shared types**: `crates/mister-smith-core/src/`, `crates/mister-smith-core/tests/`
- **Events**: `crates/mister-smith-events/src/`, `crates/mister-smith-events/tests/`
- **Agents source**: `crates/mister-smith-agents/src/`
- **Agents tests**: `crates/mister-smith-agents/tests/`
- **App source**: `crates/mister-smith-app/src/`
- **App tests**: `crates/mister-smith-app/tests/`
- **Evidence artifacts**: `docs/plans/`

## Status Reconciliation (2026-03-16)

- `MS-60` is already landed on `main` via `aa9301e` / PR `#195`.
- This task pack must preserve that current truth instead of reopening it.
- `MS-61` and `MS-62` remain the unfinished backlog slices represented by the unchecked tasks
  below.

---

## Subphase 14.0 — Task-Shape Classification And Topology Rationale (`MS-60`) — DONE

**Goal**: Preserve the landed baseline that compiles planner output into a typed execution graph
and selects topology before dispatch.

- [x] T001 [US1] Maintain task-shape classification and topology selection in
  `crates/mister-smith-agents/src/topology.rs` and
  `crates/mister-smith-agents/src/execution_graph.rs`.
- [x] T002 [P] [US1] Preserve topology rationale visibility in
  `crates/mister-smith-events/src/autonomy.rs`,
  `crates/mister-smith-events/src/bus.rs`, and
  `crates/mister-smith-agents/src/orchestrator.rs`.
- [x] T003 [US1] Keep execution-graph and topology coverage current in
  `crates/mister-smith-agents/tests/execution_graph_tests.rs` and
  `crates/mister-smith-agents/tests/topology_tests.rs`.

**Checkpoint**: Task-shape classification and topology rationale remain the stable baseline for
the rest of the packet.

---

## Subphase 14.1 — Adaptive-Team Contract Freeze (Blocking Prerequisites)

**Goal**: define one stable adaptive-team contract in shared types before any parallel lane starts.

**⚠️ CRITICAL**: No `[P]` lane may begin until this checkpoint is complete.

- [ ] T004 Add shared adaptive-team value objects in
  `crates/mister-smith-core/src/autonomy.rs` and re-export them from
  `crates/mister-smith-core/src/lib.rs`.
- [ ] T005 Extend typed operator summaries with a frozen team-sizing block in
  `crates/mister-smith-events/src/autonomy.rs`.
- [ ] T006 Extend orchestration status assembly to emit the frozen adaptive-team contract in
  `crates/mister-smith-agents/src/orchestrator.rs`.
- [ ] T007 Add shared contract coverage in
  `crates/mister-smith-core/tests/trait_compilation_tests.rs` and
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`.

**Checkpoint**: adaptive-team fields are frozen once in `core`, `events`, and `orchestrator`.

---

## User Story 2 — Dynamic Team Sizing And Lifecycle Control (`MS-61`) (Priority: P1)

**Goal**: size active workers from task structure while keeping scheduler and supervision behavior
coherent.

**Independent Test**: run representative wide and narrow workload fixtures and verify the runtime
chooses different active worker counts without reopening completed branches or breaking scheduler
invariants.

### Tests For User Story 2

- [ ] T008 [P] [US2] Add adaptive sizing fixture coverage in
  `crates/mister-smith-agents/tests/team_sizing_tests.rs`.
- [ ] T009 [P] [US2] Extend regression coverage for lifecycle and recovery invariants in
  `crates/mister-smith-agents/tests/gate10_tests.rs`.

### Implementation For User Story 2

- [ ] T010 [P] [US2] Extend adaptive team membership planning in
  `crates/mister-smith-agents/src/team.rs`.
- [ ] T011 [P] [US2] Extend lifecycle handling for selected worker sets in
  `crates/mister-smith-agents/src/scheduler.rs`.
- [ ] T012 [US2] Integrate adaptive team-sizing decisions into routing and lifecycle control in
  `crates/mister-smith-agents/src/orchestrator.rs`.
- [ ] T013 [US2] Preserve one-shot runtime compatibility while surfacing selected worker sets from
  the runtime entry point in `crates/mister-smith-app/src/execution.rs`.

**Checkpoint**: the runtime selects active team width from task structure instead of fixed fan-out.

---

## User Story 3 — Operator Status And Evaluation Harness (`MS-62`) (Priority: P2)

**Goal**: make adaptive-team decisions operator-visible and prove them with a repeatable evidence
artifact.

**Independent Test**: inspect one workflow status that includes desired versus selected worker
count, then run the deterministic comparison harness and capture a durable evidence note under
`docs/plans/`.

### Tests For User Story 3

- [ ] T014 [P] [US3] Extend status-view rendering coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`.
- [ ] T015 [P] [US3] Extend typed event/view coverage for adaptive-team summaries in
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`.

### Implementation For User Story 3

- [ ] T016 [P] [US3] Extend autonomy event aggregation with the frozen adaptive-team contract in
  `crates/mister-smith-events/src/bus.rs`.
- [ ] T017 [P] [US3] Render adaptive-team decisions in
  `crates/mister-smith-app/src/autonomy.rs`.
- [ ] T018 [P] [US3] Create a deterministic evaluation harness in
  `crates/mister-smith-agents/tests/team_sizing_benchmark_tests.rs`.
- [ ] T019 [US3] Capture one durable comparison artifact in
  `docs/plans/2026-03-16-ms-45-adaptive-orchestration-evaluation.md`.

**Checkpoint**: operators can inspect adaptive-team decisions and the repo contains durable proof.

---

## Final Validation And Evidence

- [ ] T020 Run `cargo test -p mister-smith-agents`
- [ ] T021 Run `cargo test -p mister-smith-events`
- [ ] T022 Run `cargo test -p mister-smith-app`
- [ ] T023 Run `cargo build --workspace`
- [ ] T024 Verify `MS-45`, `MS-61`, and `MS-62` all reference `specs/014-task-shape-aware-orchestration/`
  in Linear descriptions or workpads

## Parallel Symphony Directive

`[P]` means the task may run in parallel only when its write set is disjoint from every other
active lane and all blocking checkpoint tasks in the current section are already landed.

Shared-write choke points for this packet:

- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- the active `docs/plans/...` evaluation artifact
- the active `## Codex Workpad` comment for `MS-45`

Only one Symphony run may own a choke-point file at a time.

Allowed concurrent lanes after `T004` through `T007`:

- runtime sizing lane:
  `T008`, `T010`, `T011`
- operator-status lane:
  `T014`, `T015`, `T016`, `T017`
- evaluation lane:
  `T018`

Serial merge points:

- `T012` must remain serial because it reopens `crates/mister-smith-agents/src/orchestrator.rs`
- `T019` must remain serial because the evidence note under `docs/plans/` is single-owner
- `T024` must remain serial because the parent workpad and issue descriptions are single-owner

If a task needs to reopen a choke-point file, it is no longer `[P]` and must return to the
parent-owned serial lane.

## Implementation Strategy

### MVP For Remaining Work

1. Complete `T004` through `T007` to freeze the adaptive-team contract
2. Complete `T008` through `T013` for dynamic team sizing
3. Validate adaptive worker counts independently before widening into operator rendering

### Incremental Delivery

1. Preserve landed `MS-60` truth
2. Freeze the shared contract
3. Run the `MS-61` runtime lane
4. Run the `MS-62` operator-status and evaluation lanes
5. Capture durable evidence and sync Linear

## Explicitly Out Of Scope For This Packet

- reopening `MS-60`
- widening into `MS-46`, `MS-47`, or `MS-48`
- introducing a new runtime path or new orchestration crate
- treating issue labels alone as proof that tasks are safe to run in parallel

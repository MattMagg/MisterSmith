# Tasks: Runtime Truth And Run Trace

**Input**: Design documents from `specs/023-runtime-truth-and-run-trace/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. Future implementation should use targeted truth-surface and smoke-harness
checks rather than broad workspace validation first.

**Organization**: Tasks are grouped by bounded checkpoints so packet `023` is revalidated before
any later code work starts.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  complete and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in every task description

## Phase 1: Revalidation Gate (Blocking)

**Purpose**: No implementation may start until this scaffold is revised against current repo truth.

**CRITICAL**: No later task may begin until this phase is complete.

- [ ] T001 Reread `docs/direction.md`, `docs/current-state.md`,
      `docs/research-output/analysis/2026-03-28-dynamic-orchestration-transfer-brief.md`, and
      `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`, then record any
      truth drift in `specs/023-runtime-truth-and-run-trace/spec.md`
- [ ] T002 Confirm packet `022` and any reused upstream packet work are complete enough to rely on
      before code work begins, then update `specs/023-runtime-truth-and-run-trace/spec.md`,
      `specs/023-runtime-truth-and-run-trace/plan.md`, and
      `specs/023-runtime-truth-and-run-trace/quickstart.md`
- [ ] T003 Rerun scaffold refinement for `specs/023-runtime-truth-and-run-trace/spec.md`,
      `specs/023-runtime-truth-and-run-trace/plan.md`, and
      `specs/023-runtime-truth-and-run-trace/tasks.md` if repo truth moved
- [ ] T004 Refresh `specs/023-runtime-truth-and-run-trace/analyze.md` from a fresh
      `/speckit.analyze`-style pass before any implementation begins

**Checkpoint**: Packet `023` is revalidated and safe to implement against.

---

## Phase 2: Foundational Contract Freeze (Blocking Prerequisites)

**Purpose**: Freeze the shared truth and taxonomy contract before any surface-specific work.

**CRITICAL**: No user story work may begin until this phase is complete.

- [ ] T005 Freeze the shared run-trace and proof-boundary contract in
      `specs/023-runtime-truth-and-run-trace/contracts/run-trace-proof-boundary-contract.md`
- [ ] T006 Add or update shared value objects for packet `023` truth and trace taxonomy in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T007 [P] Extend transport-level trace record inputs in
      `crates/mister-smith-transport/src/envelope.rs` only as needed to support the packet `023`
      contract
- [ ] T008 Extend event-bus truth synthesis to project the frozen contract in
      `crates/mister-smith-events/src/bus.rs`

**Checkpoint**: One shared packet `023` contract exists before projection or surface work starts.

---

## Phase 3: User Story 1 - Honest Proof Boundaries (Priority: P1) 🎯 MVP

**Goal**: Make current placeholder-boundary runs clearly non-grounded on result surfaces.

**Independent Test**: A run that completes through the current `workflow.execute_step` path still
states graph success without claiming grounded task proof.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add truth-boundary regression coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T010 [P] [US1] Extend smoke-harness proof-boundary assertions in
      `scripts/tests/test_live_runtime_proof_smoke.py`

### Implementation for User Story 1

- [ ] T011 [US1] Freeze the placeholder-step boundary wording and projection rules in
      `crates/mister-smith-app/src/execution.rs` and
      `crates/mister-smith-app/src/autonomy.rs`
- [ ] T012 [US1] Ensure task and autonomy projections distinguish substrate completion from
      grounded task proof in `crates/mister-smith-core/src/autonomy.rs` and
      `crates/mister-smith-events/src/bus.rs`
- [ ] T013 [US1] Sync packet-owned proof-boundary wording with durable packet notes under `docs/plans/`
      if implementation changes current rendered truth

**Checkpoint**: Placeholder-boundary runs are clearly non-grounded across supported result views.

---

## Phase 4: User Story 2 - Canonical Run-Trace Taxonomy (Priority: P1)

**Goal**: Freeze one shared taxonomy for run-trace relationships without widening packet scope.

**Independent Test**: Graph, branch, node, tool, repair, retry, fan-out, join, and supervision
relationships can all be represented through one packet-owned taxonomy.

### Tests for User Story 2

- [ ] T014 [P] [US2] Add typed contract coverage for new run-trace entities in
      `crates/mister-smith-core/tests/trait_compilation_tests.rs`
- [ ] T015 [P] [US2] Add event projection coverage for trace relationships in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`

### Implementation for User Story 2

- [ ] T016 [P] [US2] Add or refine trace-taxonomy value objects in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T017 [P] [US2] Extend event-bus synthesis for fan-out, join, repair, retry, and supervision
      trace relationships in `crates/mister-smith-events/src/bus.rs`
- [ ] T018 [US2] Reconcile transport trace-root inputs with the packet `023` taxonomy in
      `crates/mister-smith-transport/src/envelope.rs`

**Checkpoint**: Packet `023` taxonomy is shared, bounded, and ready for projection use.

---

## Phase 5: User Story 3 - Consistent Surface Projection (Priority: P2)

**Goal**: Keep task, session, autonomy, and operator run-detail surfaces aligned without turning
the packet into a UI redesign.

**Independent Test**: The same run tells the same bounded truth story across task, session,
autonomy, and operator surfaces.

### Tests for User Story 3

- [ ] T019 [P] [US3] Extend task, session, and autonomy projection tests in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs` and create
      `crates/mister-smith-app/tests/conversation_status_tests.rs` if session-specific coverage
      needs a separate file
- [ ] T020 [P] [US3] Add operator-console view coverage in
      `apps/operator-console/src/views/`

### Implementation for User Story 3

- [ ] T021 [P] [US3] Project the packet `023` truth contract through task, session, and autonomy
      surfaces in `crates/mister-smith-app/src/autonomy.rs`,
      `crates/mister-smith-app/src/conversation.rs`, and
      `crates/mister-smith-events/src/bus.rs`
- [ ] T022 [P] [US3] Render the same bounded truth story in
      `apps/operator-console/src/views/` without widening into new dashboard scope
- [ ] T023 [US3] Update any state-bearing packet notes or router docs only as needed to reflect
      implemented packet `023` truth, without presenting the packet as broader than it is

**Checkpoint**: Supported surfaces tell one bounded truth story for the same run.

---

## Final Validation And Evidence

- [ ] T024 Run `cargo test -p mister-smith-core`
- [ ] T025 Run `cargo test -p mister-smith-events --test autonomy_event_tests`
- [ ] T026 Run `cargo test -p mister-smith-app --test autonomy_status_tests`
- [ ] T027 Run `cargo test -p mister-smith-app workflow_step_tool_marks_payload_as_tool_bus_completed`
- [ ] T028 Run `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- [ ] T029 Run `git diff --check`
- [ ] T030 Capture or refresh the packet-owned analysis and proof-boundary note in
      `specs/023-runtime-truth-and-run-trace/analyze.md` and any required `docs/plans/` note

## Parallel Staging Directive

`[P]` means the task may run in parallel only when its write set is disjoint from every other
active lane and all blocking checkpoint tasks in the current section are already complete.

Shared-write choke points for packet `023` implementation:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- active truth-state notes under `docs/plans/`

Only one active lane may own a choke-point file at a time.

## Explicitly Out Of Scope For This Packet

- packet `022` durable lifecycle, event-history, compaction, or effect-boundary implementation
- generic observability-platform work or export-pipeline work
- UI polish or dashboard redesign
- coordinator-runtime or real subagent-runtime implementation
- pretending that placeholder step completion is grounded task proof

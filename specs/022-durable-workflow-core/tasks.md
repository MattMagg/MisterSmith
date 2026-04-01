# Tasks: Durable Workflow Core

**Input**: Design documents from `/specs/022-durable-workflow-core/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. This packet will need targeted Rust tests for replay, lifecycle projections,
effect boundaries, and compaction posture once implementation begins.

**Organization**: Tasks are grouped by user story so later implementation can stay bounded and
independently testable. This file is provisional scaffolding only and must be refreshed before any
implementation starts.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only after the refresh gate and only when the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US4`)
- Refresh tasks and shared blocking tasks intentionally have no story label

## Status Reconciliation (2026-04-01)

- packet `022` is scaffolded only
- no packet `022` implementation work has started yet
- earlier packet work is still in flight, so the refresh gate below is mandatory
- these tasks are for planning speed and future execution, not for immediate `/speckit.implement`

---

## Phase 1: Pre-Implementation Refresh (Blocking)

**Purpose**: Reconfirm repo truth and refresh the packet before any coding starts

**⚠️ CRITICAL**: No implementation task may begin until this phase is complete

- [ ] T001 Refresh `specs/022-durable-workflow-core/spec.md` and
      `specs/022-durable-workflow-core/plan.md` against the latest repo truth
- [ ] T002 [P] Refresh `specs/022-durable-workflow-core/research.md` and
      `specs/022-durable-workflow-core/data-model.md` if earlier packet work changed touched seams
- [ ] T003 [P] Refresh `specs/022-durable-workflow-core/contracts/durable-workflow-contract.md`
      and `specs/022-durable-workflow-core/quickstart.md` before coding begins
- [ ] T004 Reconcile `specs/022-durable-workflow-core/tasks.md` with the refreshed packet scope
      and mark any changed assumptions before implementation starts

**Checkpoint**: Packet `022` is refreshed and safe to execute as bounded work

---

## Phase 2: Foundational Contract Freeze (Blocking Prerequisites)

**Purpose**: Freeze the shared durable history, lifecycle, and persistence contract before story
work begins

- [ ] T005 [P] Add shared durable-history contract coverage in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [ ] T006 Add durable lifecycle enums and shared value mappings in
      `crates/mister-smith-core/src/enums.rs` and `crates/mister-smith-core/src/lib.rs`
- [ ] T007 Add durable history and effect-boundary persistence records in
      `crates/mister-smith-persistence/src/repository/task.rs`
- [ ] T008 Add durable history and compaction keys in
      `crates/mister-smith-persistence/src/kv/state.rs`

**Checkpoint**: The shared contract is frozen before story-specific runtime work begins

---

## Phase 3: User Story 1 - Recover From Durable History (Priority: P1) 🎯 MVP

**Goal**: Rebuild workflow state from durable history while preserving current session continuity

**Independent Test**: An interrupted workflow can replay more than once from the same durable
history and preserve current restart-resume lineage

### Tests for User Story 1

- [ ] T009 [P] [US1] Add deterministic replay coverage in
      `crates/mister-smith-agents/tests/durable_workflow_replay_tests.rs`
- [ ] T010 [P] [US1] Extend restart-resume projection coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`

### Implementation for User Story 1

- [ ] T011 [P] [US1] Extend checkpoint-backed replay planning in
      `crates/mister-smith-agents/src/branch_checkpoint.rs`
- [ ] T012 [US1] Integrate durable-history reconstruction into
      `crates/mister-smith-agents/src/orchestrator.rs` and
      `crates/mister-smith-app/src/execution.rs`
- [ ] T013 [US1] Preserve session continuity projections in
      `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-http/src/handlers.rs`

**Checkpoint**: Durable history replay works without breaking current restart-resume behavior

---

## Phase 4: User Story 2 - Protect External Effects During Replay (Priority: P1)

**Goal**: Keep external side effects behind an explicit durable boundary during replay and retry

**Independent Test**: Replaying or retrying a completed effect boundary does not create a second
operator-visible external outcome

### Tests for User Story 2

- [ ] T014 [P] [US2] Add effect-boundary persistence coverage in
      `crates/mister-smith-persistence/tests/effect_boundary_tests.rs`
- [ ] T015 [P] [US2] Add replay-versus-effect projection coverage in
      `crates/mister-smith-app/tests/effect_boundary_projection_tests.rs`

### Implementation for User Story 2

- [ ] T016 [P] [US2] Add durable effect intent and completion helpers in
      `crates/mister-smith-persistence/src/repository/task.rs`
- [ ] T017 [P] [US2] Extend hybrid durability handling for effect-boundary recovery in
      `crates/mister-smith-persistence/src/hybrid/manager.rs`
- [ ] T018 [US2] Integrate idempotent effect-boundary handling on the runtime path in
      `crates/mister-smith-app/src/execution.rs`

**Checkpoint**: Replay-safe effect boundaries are explicit and do not overclaim exactly-once
outcomes

---

## Phase 5: User Story 3 - Control Lifecycle With Clear Verbs (Priority: P2)

**Goal**: Keep lifecycle meanings consistent across task, session, and autonomy surfaces

**Independent Test**: One lifecycle scenario produces the same meaning on task, session, and
autonomy views

### Tests for User Story 3

- [ ] T019 [P] [US3] Add lifecycle projection coverage in
      `crates/mister-smith-app/tests/lifecycle_control_tests.rs`
- [ ] T020 [P] [US3] Add task and session lifecycle handler tests in
      `crates/mister-smith-http/tests/lifecycle_handler_tests.rs`

### Implementation for User Story 3

- [ ] T021 [P] [US3] Freeze durable lifecycle mappings in
      `crates/mister-smith-core/src/enums.rs` and `crates/mister-smith-core/src/lib.rs`
- [ ] T022 [P] [US3] Apply lifecycle-command handling in
      `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-app/src/execution.rs`
- [ ] T023 [US3] Surface consistent lifecycle meanings in
      `crates/mister-smith-http/src/handlers.rs` and
      `crates/mister-smith-events/src/autonomy.rs`

**Checkpoint**: Lifecycle verbs have one durable meaning across the operator-facing surfaces

---

## Phase 6: User Story 4 - Keep Replay Bounded Over Time (Priority: P3)

**Goal**: Add the first bounded compaction and replay-governance slice

**Independent Test**: A long-running workflow can compact history once and still resume correctly
from the compacted lineage

### Tests for User Story 4

- [ ] T024 [P] [US4] Add compaction and replay-regression coverage in
      `crates/mister-smith-persistence/tests/history_compaction_tests.rs`

### Implementation for User Story 4

- [ ] T025 [P] [US4] Add bounded compaction metadata and replay keys in
      `crates/mister-smith-persistence/src/kv/state.rs`
- [ ] T026 [US4] Integrate compaction lineage handling in
      `crates/mister-smith-agents/src/orchestrator.rs` and
      `crates/mister-smith-app/src/execution.rs`

**Checkpoint**: Replay cost is bounded without losing resumability or explainable lineage

---

## Phase 7: Polish & Cross-Cutting Closure

- [ ] T027 Refresh `specs/022-durable-workflow-core/spec.md`,
      `specs/022-durable-workflow-core/design.md`,
      `specs/022-durable-workflow-core/plan.md`, and
      `specs/022-durable-workflow-core/contracts/durable-workflow-contract.md` to match landed
      implementation truth
- [ ] T028 Run `cargo test -p mister-smith-agents`
- [ ] T029 Run `cargo test -p mister-smith-persistence`
- [ ] T030 Run `cargo test -p mister-smith-app`
- [ ] T031 Run `cargo test -p mister-smith-events`
- [ ] T032 Run `cargo clippy -p mister-smith-agents -- -D warnings`
- [ ] T033 Run `cargo clippy -p mister-smith-persistence -- -D warnings`
- [ ] T034 Run `cargo clippy -p mister-smith-app -- -D warnings`
- [ ] T035 Run `cargo clippy -p mister-smith-events -- -D warnings`
- [ ] T036 Run `npx markdownlint-cli2 "specs/022-durable-workflow-core/**/*.md" --config .markdownlint.json`

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** blocks everything else until the scaffold is refreshed
- **Phase 2** blocks all user stories until the shared durable contract is frozen
- **User Stories** can only start after Phase 2, and only on refreshed packet truth
- **Phase 7** happens after the selected user stories are implemented

### User Story Dependencies

- **US1** can start after Phase 2 and is the MVP slice
- **US2** depends on the shared durable history and effect-boundary contract from Phase 2
- **US3** depends on the shared lifecycle vocabulary from Phase 2 and should compose with US1
- **US4** depends on the durable history baseline from US1 and the shared compaction posture from
  Phase 2

### Parallel Opportunities

- Phase 1 refresh tasks marked `[P]` can run in parallel inside the packet directory
- Once Phase 2 is complete:
  - persistence work can split from projection work
  - agent replay work can split from HTTP projection work
- `crates/mister-smith-app/src/execution.rs`,
  `crates/mister-smith-agents/src/orchestrator.rs`, and
  `crates/mister-smith-persistence/src/repository/task.rs` are choke points and should stay
  single-owner at any given time

## Implementation Strategy

### MVP First

1. Complete the refresh gate
2. Freeze the shared contract
3. Deliver User Story 1
4. Stop and validate replay plus session continuity before widening further

### Incremental Delivery

1. Refresh the scaffold
2. Add durable replay
3. Add effect boundaries
4. Add lifecycle verbs
5. Add bounded compaction

### Important Note

If earlier packet work changes the touched seams materially, update this file before starting
implementation instead of forcing the existing task graph onto stale repo truth.

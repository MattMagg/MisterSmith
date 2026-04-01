# Tasks: First Real Coordinator-Subagent Runtime

**Input**: Design documents from `/specs/026-first-real-coordinator-subagent-runtime/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. This scaffold names the future validation tasks now, but no implementation
should begin until the revision gate is complete.

**Organization**: Tasks are grouped by the packet revision gate, one shared contract phase, and
the three packet user stories so later implementation can start from a ready scaffold instead of a
blank page.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  complete and the write set is disjoint
- **[Story]**: Which user story this task belongs to (`US1` through `US3`)
- Include exact file paths in descriptions

## Status Reconciliation (2026-04-01)

- this is a scaffold task list, not an implementation-complete packet
- packet `022` through `025` are still in progress
- packet `026` must not start code work until the revision gate below is complete

---

## Phase 0: Revision And Reconcile (Blocking)

**Goal**: refresh the scaffold against the real landed truth of packets `022` through `025`
before any implementation begins.

**CRITICAL**: No implementation task may begin until this phase is complete.

- [ ] T001 Review `docs/current-state.md`, `docs/direction.md`, and
      `docs/packet-prep/README.md` against the landed truth for packets `022` through `025`
- [ ] T002 Update
      `/Users/macmain/.local/share/symphony-workspaces/026-first-real-coordinator-subagent-runtime/specs/026-first-real-coordinator-subagent-runtime/spec.md`
      to match the reconciled upstream packet outputs
- [ ] T003 Update
      `/Users/macmain/.local/share/symphony-workspaces/026-first-real-coordinator-subagent-runtime/specs/026-first-real-coordinator-subagent-runtime/plan.md`,
      `research.md`, `data-model.md`, `quickstart.md`, and
      `contracts/coordinator-subagent-runtime-contract.md` to match the reconciled upstream packet
      outputs
- [ ] T004 Update
      `/Users/macmain/.local/share/symphony-workspaces/026-first-real-coordinator-subagent-runtime/specs/026-first-real-coordinator-subagent-runtime/tasks.md`,
      `analyze.md`, and `checklists/scaffold.md` to match the reconciled upstream packet outputs
- [ ] T005 Re-run `./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`
      and confirm packet `026` is still bounded before any coding starts

**Checkpoint**: packet `026` is revised against real upstream truth and is ready for actual
implementation planning.

---

## Phase 1: Shared Contract Freeze (Blocking Prerequisites)

**Goal**: freeze the shared coordinator-runtime contract and proof standard once the revision gate
is done.

- [ ] T006 [P] Create or update shared value objects in
      `crates/mister-smith-core/src/autonomy.rs` and `crates/mister-smith-core/src/lib.rs`
- [ ] T007 [P] Freeze graph-facing delegation and subagent-state contract seams in
      `crates/mister-smith-agents/src/execution_graph.rs`
- [ ] T008 Freeze the runtime contract in
      `/Users/macmain/.local/share/symphony-workspaces/026-first-real-coordinator-subagent-runtime/specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`

**Checkpoint**: shared packet `026` contract and core value objects are stable.

---

## Phase 2: User Story 1 - See Real Delegation And Subagent State (Priority: P1)

**Goal**: make coordinator-owned delegation and subagent state visible on the runtime path.

**Independent Test**: one bounded delegated run exposes at least one delegation record and at
least two visible subagent state transitions without fake fan-out on sequential tasks.

### Tests For User Story 1

- [ ] T009 [P] [US1] Add delegation and subagent-state runtime tests in
      `crates/mister-smith-agents/tests/coordinator_runtime_tests.rs`
- [ ] T010 [P] [US1] Add task-path delegation projection tests in
      `crates/mister-smith-app/tests/coordinator_runtime_tests.rs`

### Implementation For User Story 1

- [ ] T011 [P] [US1] Add `CoordinatorDelegationRecord` and `SubagentStateRecord` support in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T012 [P] [US1] Extend delegation-aware graph structures in
      `crates/mister-smith-agents/src/execution_graph.rs`
- [ ] T013 [US1] Wire coordinator-owned delegation and subagent state updates in
      `crates/mister-smith-agents/src/orchestrator.rs` and
      `crates/mister-smith-agents/src/roles/coordinator.rs`
- [ ] T014 [US1] Project delegation and subagent state into runtime task output in
      `crates/mister-smith-app/src/execution.rs`

**Checkpoint**: operators can inspect real delegation and subagent state on the runtime path.

---

## Phase 3: User Story 2 - Prove Grounded Delegated Work And Feedback Loops (Priority: P1)

**Goal**: distinguish grounded delegated work from placeholder completion and surface visible
coordinator feedback decisions.

**Independent Test**: one bounded delegated run records grounded evidence for at least one
delegated job and records one visible coordinator merge or recovery decision.

### Tests For User Story 2

- [ ] T015 [P] [US2] Add grounded delegated work proof tests in
      `crates/mister-smith-app/tests/coordinator_runtime_tests.rs`
- [ ] T016 [P] [US2] Add feedback-loop and partial-failure tests in
      `crates/mister-smith-agents/tests/coordinator_runtime_tests.rs`

### Implementation For User Story 2

- [ ] T017 [P] [US2] Add `DelegatedWorkEvidenceRef` and `CoordinatorMergeDecision` support in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T018 [P] [US2] Replace placeholder-only delegated completion handling in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T019 [US2] Wire clarify, reassign, stop, collapse, and merge decision capture in
      `crates/mister-smith-agents/src/orchestrator.rs`,
      `crates/mister-smith-agents/src/roles/coordinator.rs`, and
      `crates/mister-smith-agents/src/roles/executor.rs`
- [ ] T020 [US2] Extend session-aware follow-up references in
      `crates/mister-smith-app/src/conversation.rs`

**Checkpoint**: packet `026` can distinguish grounded delegated work from placeholder delegated
completion and can surface visible feedback loops.

---

## Phase 4: User Story 3 - Inspect Proof Boundaries And Session-Aware Follow-Up (Priority: P2)

**Goal**: make proof-boundary and session-follow-up semantics visible on the operator path.

**Independent Test**: task result, autonomy status, and run detail show the same proof story and
the same session carry-forward assumptions.

### Tests For User Story 3

- [ ] T021 [P] [US3] Add packet `026` proof-view tests in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T022 [P] [US3] Add event aggregation tests in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [ ] T023 [P] [US3] Add operator-console run-detail tests in
      `apps/operator-console/src/views/RunsView.test.tsx`

### Implementation For User Story 3

- [ ] T024 [P] [US3] Add `CoordinatorRuntimeProofView` support in
      `crates/mister-smith-core/src/autonomy.rs` and `crates/mister-smith-events/src/bus.rs`
- [ ] T025 [P] [US3] Project packet `026` proof-boundary data in
      `crates/mister-smith-app/src/autonomy.rs` and `crates/mister-smith-app/src/execution.rs`
- [ ] T026 [US3] Render delegation, state, and proof summaries in
      `apps/operator-console/src/views/RunsView.tsx` and `apps/operator-console/src/types.ts`

**Checkpoint**: the operator path exposes one coherent packet `026` proof story.

---

## Final Validation And Evidence

- [ ] T027 Run `cargo test -p mister-smith-core`
- [ ] T028 Run `cargo test -p mister-smith-agents`
- [ ] T029 Run `cargo test -p mister-smith-events`
- [ ] T030 Run `cargo test -p mister-smith-app`
- [ ] T031 Run `npm --prefix apps/operator-console test`
- [ ] T032 Run `npm --prefix apps/operator-console run build`
- [ ] T033 Run `npx markdownlint-cli2 "specs/026-first-real-coordinator-subagent-runtime/**/*.md" --config .markdownlint.json`
- [ ] T034 Run `git diff --check`
- [ ] T035 Capture the final proof-boundary note under `docs/plans/` and sync
      `docs/current-state.md` only if packet `026` actually lands

## Parallel Staging Directive

`[P]` means a task may run in parallel only when:

- the revision gate is complete
- every blocking checkpoint in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for packet `026`:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/execution_graph.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `apps/operator-console/src/views/RunsView.tsx`

## Explicitly Out Of Scope For This Packet

- federation, capability discovery, or generic interoperability work
- default fan-out or fixed multi-worker topology
- redefining packet `022` through `025` ownership
- live runtime-proof claims before the revision gate and later implementation work are complete

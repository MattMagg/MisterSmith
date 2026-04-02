# Tasks: Runtime Truth And Run Trace

**Input**: Design documents from `specs/023-runtime-truth-and-run-trace/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. Use targeted truth-surface checks before any broader validation.

**Organization**: Tasks are grouped by bounded checkpoints so packet `023` becomes
implementation-ready first, then lands one shared runtime-truth contract and projection.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  complete and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in every task description

## Phase 1: Packet Revision And Truth Sync (Blocking)

**Purpose**: Convert packet `023` from scaffold to implementation-ready and remove checklist risk
before code work starts.

**CRITICAL**: No code task may begin until this phase is complete.

- [x] T001 Revise `specs/023-runtime-truth-and-run-trace/spec.md`,
      `specs/023-runtime-truth-and-run-trace/plan.md`, and
      `specs/023-runtime-truth-and-run-trace/tasks.md` to current `main` truth and implementation
      posture
- [x] T002 Revise `specs/023-runtime-truth-and-run-trace/research.md`,
      `specs/023-runtime-truth-and-run-trace/data-model.md`,
      `specs/023-runtime-truth-and-run-trace/quickstart.md`, and
      `specs/023-runtime-truth-and-run-trace/contracts/run-trace-proof-boundary-contract.md` to
      match packet `021` and packet `022` landed truth
- [x] T003 Finish `specs/023-runtime-truth-and-run-trace/checklists/runtime-truth-proof-boundary.md`
      and update `specs/023-runtime-truth-and-run-trace/checklists/requirements.md` notes to
      implementation-ready language
- [x] T004 Refresh `specs/023-runtime-truth-and-run-trace/analyze.md` so it reflects the
      implementation-ready packet instead of the earlier scaffold pass
- [x] T005 Sync packet truth drift in `docs/current-state.md` and `docs/direction.md` for packet
      `021`, packet `022`, and packet `023`

**Checkpoint**: Packet `023` is implementation-ready and all packet checklists pass.

---

## Phase 2: Foundational Runtime-Truth Contract (Blocking)

**Purpose**: Add the shared packet-023 truth contract before surface-specific work.

**CRITICAL**: No projection work may begin until this phase is complete.

- [x] T006 Add packet-023 runtime-truth value objects in
      `crates/mister-smith-core/src/autonomy.rs`
- [x] T007 [P] Add serialization and contract coverage for the new packet-023 value objects in
      `crates/mister-smith-core/tests/trait_compilation_tests.rs`
- [x] T008 Add runtime-truth synthesis to `crates/mister-smith-agents/src/orchestrator.rs`
- [x] T009 Add event-bus runtime-truth synthesis and merge behavior in
      `crates/mister-smith-events/src/bus.rs`
- [x] T010 [P] Add runtime-truth event projection coverage in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`

**Checkpoint**: One shared packet-023 contract exists and is synthesized from the current runtime
state without changing transport schema.

---

## Phase 3: User Story 1 - Honest Proof Boundaries (Priority: P1) 🎯 MVP

**Goal**: Make placeholder-boundary runs clearly non-grounded on supported result surfaces.

**Independent Test**: A run that completes through the current `workflow.execute_step` path still
states graph success without claiming grounded task proof.

### Tests for User Story 1

- [x] T011 [P] [US1] Extend `crates/mister-smith-app/tests/autonomy_status_tests.rs` with
      runtime-truth proof-boundary coverage
- [x] T012 [P] [US1] Extend `scripts/tests/test_live_runtime_proof_smoke.py` to assert the new
      packet-023 proof-boundary shape and wording

### Implementation for User Story 1

- [x] T013 [US1] Add packet-023 proof-boundary projection to
      `crates/mister-smith-app/src/execution.rs`
- [x] T014 [US1] Render the new packet-023 truth block in
      `crates/mister-smith-app/src/autonomy.rs`
- [x] T015 [US1] Preserve the frozen placeholder-step wording from the shared core contract instead
      of duplicating raw strings in app or event code

**Checkpoint**: Placeholder-boundary runs are clearly labeled as orchestration proof only.

---

## Phase 4: User Story 2 - Canonical Run-Trace Taxonomy (Priority: P1)

**Goal**: Freeze one bounded taxonomy for run-trace relationships without widening packet scope.

**Independent Test**: The shared runtime-truth block can represent graph, branch, node, tool,
handoff, repair, retry, fan-out, join, and supervision relationships.

### Tests for User Story 2

- [x] T016 [P] [US2] Add additional packet-023 run-trace contract coverage in
      `crates/mister-smith-core/tests/trait_compilation_tests.rs`
- [x] T017 [P] [US2] Add run-trace relationship synthesis coverage in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [x] T018 [P] [US2] Add orchestrator runtime-truth synthesis coverage in
      `crates/mister-smith-agents` tests

### Implementation for User Story 2

- [x] T019 [US2] Synthesize bounded run-trace relationship kinds from current graph, repair, retry,
      and supervision state in `crates/mister-smith-agents/src/orchestrator.rs` and
      `crates/mister-smith-events/src/bus.rs`
- [x] T020 [US2] Keep `crates/mister-smith-transport/src/envelope.rs` unchanged and document that
      packet `023` reuses existing `workflow_id` and `trace_id` inputs instead of widening the
      transport schema

**Checkpoint**: Packet `023` run-trace taxonomy is shared, bounded, and honest.

---

## Phase 5: User Story 3 - Consistent Surface Projection (Priority: P2)

**Goal**: Keep task, session, autonomy, and operator run-detail surfaces aligned without turning
the packet into a UI redesign.

**Independent Test**: The same run tells the same bounded runtime-truth story across task,
session, autonomy, and operator surfaces while predictive supervision stays separate.

### Tests for User Story 3

- [x] T021 [P] [US3] Extend session and retained-result coverage in
      `crates/mister-smith-app/src/conversation.rs` tests or a dedicated session projection test
      file
- [x] T022 [P] [US3] Extend operator-console coverage in `apps/operator-console/src/App.test.tsx`
      and any view-local test file if needed

### Implementation for User Story 3

- [x] T023 [US3] Project `runtime_truth` through task, session, and autonomy surfaces in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-app/src/conversation.rs`
- [x] T024 [US3] Render a separate runtime-truth panel in `apps/operator-console/src/views/RunsView.tsx`
      and update `apps/operator-console/src/types.ts`
- [x] T025 [US3] Keep packet `021` predictive supervision separate in the operator console and CLI
      output
- [x] T026 [US3] Confirm packet-owned notes under `docs/plans/` do not need changes because the
      implemented truth story kept the current rendered wording

**Checkpoint**: Supported surfaces tell one shared runtime-truth story for the same run.

---

## Final Validation And Evidence

- [x] T027 Run `cargo test -p mister-smith-core`
- [x] T028 Run `cargo test -p mister-smith-agents`
- [x] T029 Run `cargo test -p mister-smith-events --test autonomy_event_tests`
- [x] T030 Run `cargo test -p mister-smith-app --test autonomy_status_tests`
- [x] T031 Run `cargo test -p mister-smith-app workflow_step_tool_marks_payload_as_tool_bus_completed`
- [x] T032 Run `npm --prefix apps/operator-console test`
- [x] T033 Run `npm --prefix apps/operator-console run build`
- [x] T034 Run `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- [x] T035 Run `npx markdownlint-cli2 "specs/023-runtime-truth-and-run-trace/**/*.md" --config .markdownlint.json`
- [x] T036 Run `git diff --check`
- [x] T037 Refresh packet-owned analysis and confirm no extra proof-boundary note under
      `docs/plans/` is needed for this slice

## Parallel Staging Directive

`[P]` means the task may run in parallel only when its write set is disjoint from every other
active lane and all blocking checkpoint tasks in the current section are already complete.

Shared-write choke points for packet `023` implementation:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- packet `023` docs during Phase 1

Only one active lane may own a choke-point file at a time.

## Explicitly Out Of Scope For This Packet

- packet `022` durable lifecycle, event-history, compaction, or effect-boundary implementation
- packet `021` predictive-supervision semantics
- `MessageEnvelope` schema expansion
- generic observability-platform or export-pipeline work
- coordinator-runtime or real subagent-runtime implementation
- pretending placeholder step completion is grounded task proof

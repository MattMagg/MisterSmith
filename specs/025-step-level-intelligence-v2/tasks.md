# Tasks: Step-Level Intelligence v2

**Input**: Design documents from `/specs/025-step-level-intelligence-v2/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. This packet requires targeted deterministic coverage for scoring and summary
projection plus smoke-harness summary assertions and bounded UI validation if the operator-facing
summary lane is used.

**Organization**: Tasks are grouped by bounded implementation checkpoints so the shared step-policy
contract lands first, then deterministic scoring and summary lanes can proceed without conflicting
with packet `023` truth ownership.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already landed and the write set is disjoint from every other active lane
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in every task description

---

## Subphase 25.0 — Shared Step-Policy Contract Freeze

**Goal**: freeze the shared step-policy contract before any implementation lane starts.

**CRITICAL**: No `[P]` lane may begin until this checkpoint is complete.

- [ ] T001 Freeze the shared step-policy contract in
      `specs/025-step-level-intelligence-v2/contracts/step-policy-contract.md`
- [ ] T002 Freeze packet-owned score, budget, and summary entities in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T003 Extend current summary projections to consume the frozen packet contract in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-events/src/autonomy.rs`
- [ ] T004 Add shared contract coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T005 Reconfirm packet `023` proof-boundary ownership and packet `020` repair-lineage
      ownership in `specs/025-step-level-intelligence-v2/spec.md` and
      `specs/025-step-level-intelligence-v2/plan.md`

**Checkpoint**: step-policy fields, packet `023` ownership boundaries, and packet `020`
intersections are frozen once before later lanes begin.

---

## User Story 1 — Deterministic Step Scoring (Priority: P1)

**Goal**: derive one deterministic step score from the current step-evaluation seam.

**Independent Test**: deterministic inputs can produce the same score repeatedly, including one
low-risk `keep` case and one higher-risk case, without regressing current fallback behavior.

### Tests For User Story 1

- [ ] T006 [P] [US1] Add deterministic step-scoring coverage in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T007 [P] [US1] Extend task and autonomy summary coverage for step scores in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`

### Implementation For User Story 1

- [ ] T008 [P] [US1] Add `StepDifficultyAssessment` fields and exports in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T009 [US1] Build deterministic score assembly from current runtime inputs in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T010 [US1] Project packet-owned step scores through existing summary surfaces in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-events/src/autonomy.rs`

**Checkpoint**: the runtime can derive a bounded deterministic step score without widening packet
scope or redefining proof ownership.

---

## User Story 2 — Budget-Aware Step Action Policy (Priority: P1)

**Goal**: choose one bounded action across keep, retry, clarify, downgrade, and escalate using
deterministic policy rules and budget-aware hints.

**Independent Test**: deterministic inputs can prove at least one retry or clarify path and at
least one downgrade or escalate path without inventing a new trace schema.

### Tests For User Story 2

- [ ] T011 [P] [US2] Add bounded action-ladder tests in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T012 [P] [US2] Add smoke-harness summary assertions for score and action wording in
      `scripts/tests/test_live_runtime_proof_smoke.py`

### Implementation For User Story 2

- [ ] T013 [P] [US2] Add `StepBudgetPressureSummary` and `StepPolicyDecision` fields in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T014 [US2] Implement the deterministic action ladder in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T015 [US2] Reconcile packet-020 repair-lineage references with packet-owned step policy in
      `crates/mister-smith-app/src/autonomy.rs`

**Checkpoint**: the runtime can choose a bounded step action from deterministic inputs while
preserving packet `020` ownership and packet `023` proof boundaries.

---

## User Story 3 — Honest Operator-Facing Step Summaries (Priority: P2)

**Goal**: expose step score, chosen action, and proof-honesty wording through current summary
surfaces without making placeholder completion look like grounded task proof.

**Independent Test**: existing inspect surfaces can show the packet-owned summary and explicit
placeholder-versus-grounded wording without raw log archaeology.

### Tests For User Story 3

- [ ] T016 [P] [US3] Extend summary rendering coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T017 [P] [US3] Add bounded operator-summary coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 3

- [ ] T018 [P] [US3] Extend task and autonomy summary assembly in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-events/src/autonomy.rs`
- [ ] T019 [P] [US3] Render the packet-owned step-policy summary in
      `apps/operator-console/src/views/RunsView.tsx` and
      `apps/operator-console/src/types.ts`
- [ ] T020 [US3] Add explicit placeholder-versus-grounded wording to current proof summary
      assertions in `scripts/tests/test_live_runtime_proof_smoke.py`

**Checkpoint**: operators can inspect step-policy summaries without mistaking placeholder
completion for grounded task proof.

---

## Final Validation And Packet Note Sync

- [ ] T021 Run `cargo test -p mister-smith-core`
- [ ] T022 Run `cargo test -p mister-smith-app`
- [ ] T023 Run `cargo test -p mister-smith-events --test autonomy_event_tests`
- [ ] T024 Run `cargo test -p mister-smith-app --test autonomy_status_tests`
- [ ] T025 Run `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- [ ] T026 Run `npm --prefix apps/operator-console run build`
- [ ] T027 Run `git diff --check`
- [ ] T028 Capture one bounded packet note under `docs/plans/` that keeps deterministic-only
      versus live-proof wording honest

## Parallel Staging Directive

`[P]` means the task may run in parallel only when its write set is disjoint from every other
active lane and all blocking checkpoint tasks in the current section are already landed.

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- the active `docs/plans/...` packet note

Only one active lane may own a choke-point file at a time.

## Explicitly Out Of Scope For This Packet

- grounded step execution beyond the current placeholder `workflow.execute_step` seam
- packet `023` trace taxonomy or proof-boundary schema work
- coordinator runtime, subagent runtime, or interoperability work
- benchmark work, PRM training, or judge-heavy scoring programs

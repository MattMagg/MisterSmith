# Tasks: Step-Level Intelligence v2

**Input**: Design documents from `/specs/025-step-level-intelligence-v2/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. Use targeted deterministic Rust coverage for scoring and summary projection,
smoke-harness truth checks, and bounded operator-surface validation if the UI lane is used.

**Organization**: Tasks are grouped by the packet-authority gate first, then by the three bounded
stories frozen in the packet.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only after all blocking tasks in the current section are complete
  and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Every task includes exact file paths

---

## Phase 0: Packet Revision And Truth Sync (Blocking)

**Purpose**: Make packet `025` implementation-ready before any code changes start.

- [x] T001 Revise `specs/025-step-level-intelligence-v2/spec.md` and
      `specs/025-step-level-intelligence-v2/plan.md` to current `main` truth, replacing stale
      scaffold inputs with live repo authority docs and current step-policy seams
- [x] T002 Revise `specs/025-step-level-intelligence-v2/research.md`,
      `specs/025-step-level-intelligence-v2/data-model.md`,
      `specs/025-step-level-intelligence-v2/quickstart.md`, and
      `specs/025-step-level-intelligence-v2/contracts/step-policy-contract.md` to match landed
      packets `019`, `020`, `023`, and `024`
- [x] T003 Rewrite `specs/025-step-level-intelligence-v2/tasks.md` so the packet authority gate is
      checklist-first, followed by contract freeze, deterministic scoring, bounded action policy,
      and honest summary projection
- [x] T004 Finish `specs/025-step-level-intelligence-v2/checklists/step-policy.md` and update
      `specs/025-step-level-intelligence-v2/checklists/requirements.md` notes to
      implementation-ready language
- [x] T005 Refresh `specs/025-step-level-intelligence-v2/analyze.md` so it reflects the
      implementation-ready packet instead of the earlier scaffold pass
- [x] T006 Sync packet-025 status wording in `docs/current-state.md` and `docs/direction.md`

**Checkpoint**: Packet `025` is implementation-ready and all packet checklists pass.

---

## Phase 1: Shared Step-Policy Contract Freeze (Blocking)

**Purpose**: Freeze the shared packet-025 contract before runtime or surface work begins.

**CRITICAL**: No projection or runtime task may begin until this phase is complete.

- [ ] T007 Add packet-025 step-policy value objects in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T008 [P] Add step-policy summary fields to current event and app projections in
      `crates/mister-smith-events/src/autonomy.rs` and
      `crates/mister-smith-app/src/autonomy.rs`
- [ ] T009 [P] Add shared contract coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T010 Freeze packet-owned field names and authoritative payload shape in
      `specs/025-step-level-intelligence-v2/contracts/step-policy-contract.md` and
      `specs/025-step-level-intelligence-v2/data-model.md`

**Checkpoint**: One shared packet-025 contract exists before scoring and UI lanes begin.

---

## User Story 1 - Deterministic Step Scoring (Priority: P1)

**Goal**: Derive one deterministic step score from current verifier, routing, budget, supervision,
and runtime-truth inputs.

**Independent Test**: Deterministic inputs produce the same score repeatedly, including one
low-risk `keep` case and one higher-risk case, without regressing current fallback behavior.

### Tests For User Story 1

- [ ] T011 [P] [US1] Add deterministic step-scoring coverage in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T012 [P] [US1] Extend current summary projection coverage for step scores in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`

### Implementation For User Story 1

- [ ] T013 [P] [US1] Add `StepDifficultyAssessment` fields and exports in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T014 [US1] Build deterministic score assembly from current runtime inputs in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T015 [US1] Project packet-owned step scores through existing summary surfaces in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-events/src/autonomy.rs`

**Checkpoint**: The runtime can derive a bounded deterministic step score without widening packet
scope or redefining ownership boundaries.

---

## User Story 2 - Budget-Aware Step Action Policy (Priority: P1)

**Goal**: Choose one bounded action across `keep`, `retry`, `clarify`, `downgrade`, and
`escalate` using deterministic policy rules and budget-aware hints.

**Independent Test**: Deterministic inputs can prove at least one retry or clarify path and at
least one downgrade or escalate path without inventing a new trace or routing schema.

### Tests For User Story 2

- [ ] T016 [P] [US2] Add bounded action-ladder tests in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T017 [P] [US2] Add smoke-harness assertions for step-policy wording and proof honesty in
      `scripts/tests/test_live_runtime_proof_smoke.py`

### Implementation For User Story 2

- [ ] T018 [P] [US2] Add `StepBudgetHint` and `StepPolicyDecision` fields in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T019 [US2] Implement the deterministic action ladder in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T020 [US2] Reconcile packet-020 repair lineage and packet-023 truth references with the
      packet-owned step-policy summary in `crates/mister-smith-app/src/autonomy.rs`

**Checkpoint**: The runtime can choose a bounded step action from deterministic inputs while
preserving packet `020` and packet `023` ownership.

---

## User Story 3 - Honest Operator-Facing Step Summaries (Priority: P2)

**Goal**: Expose step score, chosen action, and proof-honesty wording through current summary
surfaces without making placeholder completion look like grounded task proof.

**Independent Test**: Existing inspect surfaces can show the packet-owned summary and explicit
placeholder-versus-grounded wording without raw log archaeology.

### Tests For User Story 3

- [ ] T021 [P] [US3] Extend summary rendering coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T022 [P] [US3] Add bounded operator-summary coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 3

- [ ] T023 [P] [US3] Extend task and autonomy summary assembly in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-events/src/autonomy.rs`
- [ ] T024 [P] [US3] Render the packet-owned step-policy summary in
      `apps/operator-console/src/views/RunsView.tsx` and
      `apps/operator-console/src/types.ts`
- [ ] T025 [US3] Keep packet-023 placeholder-versus-grounded wording explicit in
      `scripts/tests/test_live_runtime_proof_smoke.py`

**Checkpoint**: Operators can inspect step-policy summaries without mistaking placeholder
completion for grounded task proof.

---

## Final Validation And Evidence

- [ ] T026 Run `cargo test -p mister-smith-core`
- [ ] T027 Run `cargo test -p mister-smith-events --test autonomy_event_tests`
- [ ] T028 Run `cargo test -p mister-smith-app --test autonomy_status_tests`
- [ ] T029 Run `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- [ ] T030 Run `npm --prefix apps/operator-console test`
- [ ] T031 Run `npx markdownlint-cli2 "specs/025-step-level-intelligence-v2/**/*.md" --config .markdownlint.json`
- [ ] T032 Run `git diff --check`

## Parallel Staging Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `specs/025-step-level-intelligence-v2/contracts/step-policy-contract.md`

Only one active lane may own a choke-point path at a time.

## Explicitly Out Of Scope For This Packet

- packet `022` durable workflow ownership work
- packet `023` truth or proof-boundary schema work
- packet `024` boundary model changes
- grounded execution beyond the current placeholder `workflow.execute_step` seam
- coordinator runtime, subagent runtime, or interoperability work
- benchmark work, PRM training, or judge-heavy scoring programs

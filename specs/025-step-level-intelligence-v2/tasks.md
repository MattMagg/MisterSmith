# Tasks: Step-Level Intelligence v2

**Input**: Design documents from `/specs/025-step-level-intelligence-v2/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. Use targeted Rust tests for deterministic step-policy assembly and result
projection, plus smoke-harness unit coverage, bounded operator-console validation if UI files
move, and bounded doc hygiene.

**Organization**: Tasks are grouped by packet-authority preparation first, then by the three
bounded stories frozen for packet `025`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already complete and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Every task includes exact file paths

---

## Phase 0: Packet Authority And Checklist Completion (Blocking)

**Purpose**: Make packet `025` implementation-ready before `speckit.implement` starts code work.

- [x] T001 Revise `specs/025-step-level-intelligence-v2/spec.md` and
      `specs/025-step-level-intelligence-v2/plan.md` to current `main` truth, replacing stale
      scaffold wording and removed prep-doc anchors with live repo authority docs and current code
      seams
- [x] T002 Revise `specs/025-step-level-intelligence-v2/research.md`,
      `specs/025-step-level-intelligence-v2/data-model.md`,
      `specs/025-step-level-intelligence-v2/quickstart.md`,
      `specs/025-step-level-intelligence-v2/contracts/step-policy-contract.md`, and
      `specs/025-step-level-intelligence-v2/analyze.md` so they match landed packet `022` through
      packet `024` truth
- [x] T003 Rewrite `specs/025-step-level-intelligence-v2/tasks.md` so packet authority is settled
      first and the remaining tasks point directly at current implementation seams
- [x] T004 Finish `specs/025-step-level-intelligence-v2/checklists/step-policy.md` and refresh
      notes in `specs/025-step-level-intelligence-v2/checklists/requirements.md`
- [x] T005 Update `docs/current-state.md` and `docs/direction.md` only if packet `025` is being
      landed as the next implementation-ready packet on `main`

**Checkpoint**: Packet `025` is implementation-ready and the packet-owned checklists are complete.

---

## User Story 1 - Score a step deterministically from landed runtime signals (Priority: P1)

**Goal**: derive one deterministic step-difficulty assessment from the current step-evaluation,
step-routing, supervision, budget-pressure, lifecycle, and runtime-truth seams.

**Independent Test**: deterministic fixtures produce the same difficulty bucket repeatedly,
including one low-risk `keep` case and one higher-risk case, without regressing current fallback
behavior.

### Tests For User Story 1

- [ ] T006 [P] [US1] Extend packet-owned assessment coverage in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T007 [P] [US1] Extend summary projection coverage for step difficulty in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`

### Implementation For User Story 1

- [ ] T008 [P] [US1] Add `StepDifficultyAssessment` and `StepPolicySummaryView` fields and exports
      in `crates/mister-smith-core/src/autonomy.rs`
- [ ] T009 [US1] Build deterministic difficulty assessment from current runtime inputs in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T010 [US1] Project packet-owned step difficulty through current result surfaces in
      `crates/mister-smith-app/src/autonomy.rs` and
      `crates/mister-smith-events/src/autonomy.rs`

**Checkpoint**: the runtime can derive one bounded deterministic step assessment without widening
packet scope or redefining packet `022`, packet `023`, or packet `024` ownership.

---

## User Story 2 - Choose a bounded step action under repair and budget pressure (Priority: P1)

**Goal**: choose one deterministic action across `keep`, `retry`, `clarify`, `downgrade`, and
`escalate` using landed repair, routing, budget, supervision, lifecycle, and runtime-truth
signals.

**Independent Test**: deterministic inputs prove at least one `retry` or `clarify` path and at
least one `downgrade` or `escalate` path without inventing a new durable-workflow,
runtime-truth, or boundary-security schema.

### Tests For User Story 2

- [ ] T011 [P] [US2] Add bounded action-ladder coverage in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T012 [P] [US2] Extend event-surface coverage for step-policy action projection in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [ ] T013 [P] [US2] Add smoke-harness assertions for packet-owned step-policy wording in
      `scripts/tests/test_live_runtime_proof_smoke.py`

### Implementation For User Story 2

- [ ] T014 [P] [US2] Add `StepBudgetPressureSummary` and `StepPolicyDecision` fields in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T015 [US2] Implement the deterministic action ladder in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T016 [US2] Reconcile packet `020` repair-lineage, packet `022` lifecycle, and packet `023`
      runtime-truth inputs inside the packet-owned summary in
      `crates/mister-smith-app/src/autonomy.rs`

**Checkpoint**: the runtime can choose one bounded step action from deterministic inputs while
preserving upstream packet ownership.

---

## User Story 3 - Inspect honest step-policy summaries on current result surfaces (Priority: P2)

**Goal**: expose the latest step assessment, action, budget hint, and proof-honesty wording
through current task, session, autonomy, and operator-facing result surfaces.

**Independent Test**: existing task, session, autonomy, and operator surfaces show the same latest
packet-owned summary and explicit placeholder-versus-grounded wording without raw log archaeology.

### Tests For User Story 3

- [ ] T017 [P] [US3] Extend result-surface rendering coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T018 [P] [US3] Add bounded operator-summary coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 3

- [ ] T019 [P] [US3] Extend task, session, and autonomy summary assembly in
      `crates/mister-smith-app/src/autonomy.rs`
- [ ] T020 [P] [US3] Extend autonomy event and operator-preview payloads in
      `crates/mister-smith-events/src/autonomy.rs`
- [ ] T021 [P] [US3] Render the packet-owned step-policy summary in
      `apps/operator-console/src/views/RunsView.tsx` and
      `apps/operator-console/src/types.ts`

**Checkpoint**: operators can inspect step-policy summaries without mistaking placeholder
completion for grounded task proof.

---

## Final Validation And Packet Note Sync

- [ ] T022 Run `npx markdownlint-cli2 "specs/025-step-level-intelligence-v2/**/*.md" --config .markdownlint.json`
- [ ] T023 Run `cargo test -p mister-smith-events --test autonomy_event_tests`
- [ ] T024 Run `cargo test -p mister-smith-app --test autonomy_status_tests`
- [ ] T025 Run `cargo test -p mister-smith-app`
- [ ] T026 Run `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- [ ] T027 Run `npm --prefix apps/operator-console run build`
- [ ] T028 Run `cargo build --workspace`
- [ ] T029 Run `git diff --check`
- [ ] T030 Capture one bounded packet note under `docs/plans/` that keeps deterministic-only
      versus live-proof wording honest if implementation lands

## Parallel Staging Directive

`[P]` means the task may run in parallel only when its write set is disjoint from every other
active lane and all blocking checkpoint tasks in the current section are already complete.

Shared-write choke points for this packet:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- the active packet-owned note under `docs/plans/`

Only one active lane may own a choke-point file at a time.

## Explicitly Out Of Scope For This Packet

- packet `022` durable-workflow, packet `023` runtime-truth, or packet `024` boundary-policy
  changes
- grounded step execution beyond the current placeholder `workflow.execute_step` seam
- a new raw streaming-event parser, PRM training loop, benchmark program, coordinator runtime,
  subagent runtime, or interoperability work

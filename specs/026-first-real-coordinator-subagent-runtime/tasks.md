# Tasks: First Real Coordinator-Subagent Runtime

**Input**: Design documents from `/specs/026-first-real-coordinator-subagent-runtime/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. Use targeted Rust coverage for delegation, child-state, proof projection, and
operator-surface rendering plus bounded packet-doc validation.

**Organization**: Tasks are grouped by the completed packet-authority refresh first, then by the
shared contract freeze and the three bounded packet stories.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only after all blocking tasks in the current section are complete
  and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Every task includes exact file paths

---

## Phase 0: Completed Packet Revision And Truth Sync

**Purpose**: Record the completed doc refresh that made packet `026` implementation-ready before
code changes start.

- [x] T001 Revise `specs/026-first-real-coordinator-subagent-runtime/spec.md` and
      `specs/026-first-real-coordinator-subagent-runtime/plan.md` to current `main` truth,
      replacing scaffold-only inputs with live repo authority docs and current coordinator-runtime
      seams
- [x] T002 Refresh `specs/026-first-real-coordinator-subagent-runtime/research.md`,
      `specs/026-first-real-coordinator-subagent-runtime/data-model.md`,
      `specs/026-first-real-coordinator-subagent-runtime/quickstart.md`, and
      `specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`
      to match landed packets `022` through `025` and the OpenClaude transfer backlog
- [x] T003 Rewrite `specs/026-first-real-coordinator-subagent-runtime/tasks.md` so the packet
      authority gate is checklist-first, followed by shared contract freeze, visible delegation,
      grounded delegated work, and honest proof projection
- [x] T004 Finish `specs/026-first-real-coordinator-subagent-runtime/checklists/scaffold.md` and
      update `specs/026-first-real-coordinator-subagent-runtime/checklists/requirements.md` notes
      to implementation-ready language
- [x] T005 Refresh `specs/026-first-real-coordinator-subagent-runtime/analyze.md` so it reflects
      the implementation-ready packet instead of the earlier scaffold pass
- [x] T006 Sync packet-026 readiness wording in `docs/current-state.md` and `docs/direction.md`

**Checkpoint**: Packet `026` is implementation-ready and all packet checklists pass.

---

## Phase 1: Shared Coordinator-Runtime Contract Freeze (Blocking)

**Purpose**: Freeze the shared packet-026 contract before runtime or surface work begins.

**CRITICAL**: No runtime projection or operator task may begin until this phase is complete.

- [ ] T007 Add packet-026 coordinator-runtime value objects in
      `crates/mister-smith-core/src/autonomy.rs` and `crates/mister-smith-core/src/lib.rs`
- [ ] T008 [P] Extend shared event and preview contract seams in
      `crates/mister-smith-events/src/autonomy.rs` and `crates/mister-smith-events/src/bus.rs`
- [ ] T009 [P] Add shared contract coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs` and
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [ ] T010 Freeze authoritative packet-owned payload shape in
      `specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`
      and `specs/026-first-real-coordinator-subagent-runtime/data-model.md`

**Checkpoint**: One shared packet-026 contract exists before delegation and proof lanes begin.

---

## User Story 1 - See Real Delegation And Child State (Priority: P1)

**Goal**: make coordinator-owned delegation, subordinate inbox activity, and child state visible on
the runtime path.

**Independent Test**: one bounded delegated run exposes at least one delegation record, at least
two visible child state transitions, and one honest sequential-collapse path without fake fan-out.

### Tests For User Story 1

- [ ] T011 [P] [US1] Add delegation and child-state coverage in
      `crates/mister-smith-agents/tests/execution_graph_tests.rs` and
      `crates/mister-smith-agents/tests/team_tests.rs`
- [ ] T012 [P] [US1] Add task and autonomy delegation projection coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`

### Implementation For User Story 1

- [ ] T013 [P] [US1] Add `CoordinatorDelegationRecord`,
      `CoordinatorSubordinateInboxRecord`, and `SubagentStateRecord` support in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T014 [P] [US1] Extend coordinator-owned delegation and ordered child event intake in
      `crates/mister-smith-agents/src/execution_graph.rs` and
      `crates/mister-smith-agents/src/orchestrator.rs`
- [ ] T015 [US1] Project delegation, subordinate inbox, and child-state summaries in
      `crates/mister-smith-events/src/autonomy.rs`,
      `crates/mister-smith-events/src/bus.rs`, and
      `crates/mister-smith-app/src/autonomy.rs`

**Checkpoint**: operators can inspect real delegation, child state, and subordinate inbox activity
on the runtime path.

---

## User Story 2 - Prove Grounded Delegated Work And Feedback Loops (Priority: P1)

**Goal**: distinguish grounded delegated work from placeholder completion and surface visible
coordinator feedback decisions.

**Independent Test**: one bounded delegated run records grounded evidence for at least one
delegated job and records one visible coordinator merge or recovery decision.

### Tests For User Story 2

- [ ] T016 [P] [US2] Add grounded delegated-work and placeholder-only proof coverage in
      `crates/mister-smith-app/tests/effect_boundary_projection_tests.rs`
- [ ] T017 [P] [US2] Add feedback-loop, collapse, and sibling-abort coverage in
      `crates/mister-smith-agents/tests/execution_graph_tests.rs` and
      `crates/mister-smith-agents/tests/team_tests.rs`

### Implementation For User Story 2

- [ ] T018 [P] [US2] Add `DelegatedWorkEvidenceRef` and `CoordinatorMergeDecision` support in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T019 [P] [US2] Replace placeholder-only delegated completion handling and session follow-up
      references in `crates/mister-smith-app/src/execution.rs` and
      `crates/mister-smith-app/src/conversation.rs`
- [ ] T020 [US2] Wire clarify, reassign, stop, collapse, merge, and deterministic sibling-cancel
      outcomes in `crates/mister-smith-agents/src/orchestrator.rs`,
      `crates/mister-smith-agents/src/roles/coordinator.rs`, and
      `crates/mister-smith-agents/src/roles/executor.rs`
- [ ] T021 [US2] Add explorer, planner, and verifier-style child-role mapping in
      `crates/mister-smith-agents/src/roles/coordinator.rs`,
      `crates/mister-smith-agents/src/roles/planner.rs`,
      `crates/mister-smith-agents/src/roles/worker.rs`, and
      `crates/mister-smith-agents/src/roles/critic.rs`

**Checkpoint**: packet `026` can distinguish grounded delegated work from placeholder delegated
completion and can surface visible feedback loops.

---

## User Story 3 - Inspect Proof Boundaries And Session-Aware Follow-Up (Priority: P2)

**Goal**: make proof-boundary and session-follow-up semantics visible on the operator path.

**Independent Test**: task result, autonomy status, and run detail show the same proof story and
the same session carry-forward assumptions.

### Tests For User Story 3

- [ ] T022 [P] [US3] Add packet-026 proof-view coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T023 [P] [US3] Add event aggregation coverage in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [ ] T024 [P] [US3] Add operator-console run-detail coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 3

- [ ] T025 [P] [US3] Add `CoordinatorRuntimeProofView` support in
      `crates/mister-smith-core/src/autonomy.rs` and `crates/mister-smith-events/src/bus.rs`
- [ ] T026 [P] [US3] Project packet-026 proof-boundary and session-follow-up data in
      `crates/mister-smith-app/src/autonomy.rs` and `crates/mister-smith-app/src/execution.rs`
- [ ] T027 [US3] Render delegation, subordinate inbox, child-state, and proof summaries in
      `apps/operator-console/src/views/RunsView.tsx` and `apps/operator-console/src/types.ts`

**Checkpoint**: the operator path exposes one coherent packet-026 proof story.

---

## Final Validation And Evidence

- [ ] T028 Run `cargo test -p mister-smith-core`
- [ ] T029 Run `cargo test -p mister-smith-agents`
- [ ] T030 Run `cargo test -p mister-smith-events --test autonomy_event_tests`
- [ ] T031 Run `cargo test -p mister-smith-app --test autonomy_status_tests`
- [ ] T032 Run `npm --prefix apps/operator-console test`
- [ ] T033 Run `npm --prefix apps/operator-console run build`
- [ ] T034 Run
      `SPECIFY_FEATURE=026-first-real-coordinator-subagent-runtime ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`
- [ ] T035 Run
      `npx markdownlint-cli2 "specs/026-first-real-coordinator-subagent-runtime/**/*.md" docs/current-state.md docs/direction.md --config .markdownlint.json`
- [ ] T036 Run `git diff --check`
- [ ] T037 Capture the final proof-boundary note under `docs/plans/` and sync
      `docs/current-state.md` only if packet `026` actually lands

## Parallel Staging Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/execution_graph.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-events/src/bus.rs`
- `apps/operator-console/src/views/RunsView.tsx`

Only one active lane may own a choke-point path at a time.

## Explicitly Out Of Scope For This Packet

- federation, capability discovery, or generic interoperability work
- default fan-out or fixed multi-worker topology
- packet `022` through `025` ownership changes
- remote-executor or secret-minimized bridge work
- a new endpoint or broader operator-console redesign
- live runtime-proof claims before the later implementation work and rerun are complete

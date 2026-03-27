# Tasks: Profile-Aware Predictive Runtime Supervision

**Input**: Design documents from `/specs/021-profile-aware-predictive-runtime-supervision/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`

**Tests**: Included. This packet requires targeted supervision tests in `mister-smith-core`,
`mister-smith-agents`, `mister-smith-events`, and `mister-smith-app`, plus bounded
operator-console validation and repo hygiene checks.

**Organization**: Tasks are grouped by bounded implementation checkpoints so the shared predictive
supervision contract lands first, then disjoint lanes can proceed without overlapping write
ownership.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already landed and the write set is disjoint from every other active lane
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in every task description

## Status Reconciliation (2026-03-27)

- packet `020` is already landed on `main`
- the March 27 runtime-planning simplification pass is already landed on `main`
- packet `021` is now frozen on `main` as the next bounded post-packet-020 phase
- the implementation tasks below remain unchecked

---

## Subphase 21.0 — Packet Freeze And Router Sync — DONE

**Goal**: freeze the next bounded packet and sync the repo router.

- [x] T001 Add packet `021` artifacts under `specs/021-profile-aware-predictive-runtime-supervision/`
- [x] T002 Update `docs/current-state.md` so the repo no longer claims that no post-packet-020
      packet is frozen

**Checkpoint**: packet `021` is the durable next-phase authority on `main`.

---

## Subphase 21.1 — Shared Supervision Contract Freeze (Blocking Prerequisites)

**Goal**: define one stable predictive-supervision contract before any parallel lane starts.

**CRITICAL**: No `[P]` lane may begin until this checkpoint is complete.

- [ ] T003 Add `ProfileFingerprint` and any shared supervision evidence value objects in
      `crates/mister-smith-core/src/autonomy.rs` and re-export them from
      `crates/mister-smith-core/src/lib.rs`
- [ ] T004 Extend typed autonomy summaries with the frozen supervision-evidence contract in
      `crates/mister-smith-events/src/autonomy.rs` and `crates/mister-smith-events/src/bus.rs`
- [ ] T005 Extend status assembly to emit the frozen supervision contract in
      `crates/mister-smith-agents/src/orchestrator.rs`
- [ ] T006 Add shared contract coverage in `crates/mister-smith-core/tests/trait_compilation_tests.rs`
      and `crates/mister-smith-events/tests/autonomy_event_tests.rs`

**Checkpoint**: predictive-supervision fields are frozen once in `core`, `events`, and
`orchestrator`.

---

## User Story 1 — Supported-Ingress Predictive Supervision (Priority: P1)

**Goal**: move supported runtime supervision from provider-only scope to branch- and node-aware
runtime targets.

**Independent Test**: simulate recoverable and unrecoverable degradation on the supported ingress
and verify the runtime records profile, guard, and intervention evidence without regressing the
current happy path.

### Tests For User Story 1

- [ ] T007 [P] [US1] Add branch- and node-scoped supervision coverage in
      `crates/mister-smith-agents/tests/` and extend existing orchestrator tests as needed
- [ ] T008 [P] [US1] Extend runtime task-path supervision coverage in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T009 [P] [US1] Extend autonomy rendering and result-contract coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`

### Implementation For User Story 1

- [ ] T010 [P] [US1] Update runtime supervision target selection in
      `crates/mister-smith-app/src/execution.rs`
- [ ] T011 [P] [US1] Extend profile capture and target mapping in
      `crates/mister-smith-agents/src/profile.rs`
- [ ] T012 [US1] Integrate branch- and node-scoped predictive supervision into
      `crates/mister-smith-agents/src/orchestrator.rs`,
      `crates/mister-smith-agents/src/guard.rs`, and
      `crates/mister-smith-agents/src/intervention.rs`

**Checkpoint**: the supported runtime path emits first-class predictive-supervision evidence.

---

## User Story 2 — Bounded Profile Fingerprints (Priority: P1)

**Goal**: seed advisory performance fingerprints from replayable evidence and let them reinforce
Guard decisions without creating a learned control plane.

**Independent Test**: deterministic fixtures prove that a current fingerprint can reinforce at
least one intervention decision while stale fingerprints fall back cleanly.

### Tests For User Story 2

- [ ] T013 [P] [US2] Add fingerprint serialization and guard-evidence tests in
      `crates/mister-smith-core/tests/trait_compilation_tests.rs` and
      `crates/mister-smith-agents/tests/`
- [ ] T014 [P] [US2] Add JetStream KV fingerprint coverage, including structured-summary-only
      storage rules, in `crates/mister-smith-persistence/tests/kv_tests.rs`

### Implementation For User Story 2

- [ ] T015 [P] [US2] Add fingerprint storage helpers in
      `crates/mister-smith-persistence/src/kv/`
- [ ] T016 [P] [US2] Extend profile and Guard decision logic to consume fingerprints in
      `crates/mister-smith-agents/src/profile.rs` and
      `crates/mister-smith-agents/src/guard.rs`
- [ ] T017 [US2] Wire fingerprint loading and save/update flow into
      `crates/mister-smith-app/src/execution.rs`

**Checkpoint**: runtime supervision can use bounded advisory fingerprints grounded in replayable
evidence.

---

## User Story 3 — Operator Evidence And Proof Boundary (Priority: P2)

**Goal**: make supervisory evidence operator-visible and keep proof boundaries explicit.

**Independent Test**: inspect one task, one autonomy view, and one operator-console run detail
that all show coherent supervisory evidence and proof-boundary text.

### Tests For User Story 3

- [ ] T018 [P] [US3] Extend typed event/view coverage for supervision evidence in
      `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- [ ] T019 [P] [US3] Extend app status-view rendering coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] T020 [P] [US3] Add operator-console view coverage in
      `apps/operator-console/src/views/` test files or the existing UI test lane

### Implementation For User Story 3

- [ ] T021 [P] [US3] Extend autonomy event aggregation in `crates/mister-smith-events/src/bus.rs`
      and `crates/mister-smith-app/src/autonomy.rs`
- [ ] T022 [P] [US3] Render supervisory evidence in
      `apps/operator-console/src/views/RunsView.tsx` and `apps/operator-console/src/types.ts`
- [ ] T023 [US3] Capture one durable proof-boundary note under `docs/plans/` when implementation
      lands

**Checkpoint**: operators can inspect predictive-supervision evidence without raw payload digging.

---

## Final Validation And Evidence

- [ ] T024 Run `cargo test -p mister-smith-core`
- [ ] T025 Run `cargo test -p mister-smith-agents`
- [ ] T026 Run `cargo test -p mister-smith-events`
- [ ] T027 Run `cargo test -p mister-smith-app`
- [ ] T028 Run `cargo clippy -p mister-smith-core -- -D warnings`
- [ ] T029 Run `cargo clippy -p mister-smith-agents -- -D warnings`
- [ ] T030 Run `cargo clippy -p mister-smith-events -- -D warnings`
- [ ] T031 Run `cargo clippy -p mister-smith-app -- -D warnings`
- [ ] T032 Run `npm --prefix apps/operator-console run build`
- [ ] T033 Run `git diff --check`

## Parallel Staging Directive

`[P]` means the task may run in parallel only when its write set is disjoint from every other
active lane and all blocking checkpoint tasks in the current section are already landed.

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- the active proof note under `docs/plans/`
- `docs/current-state.md`

Only one active lane may own a choke-point file at a time.

## Explicitly Out Of Scope For This Packet

- defaultizing packet `019` routing when `llm.runtime_routing_profile` is absent
- reopening `MS-110` adaptive-topology work without new live evidence
- CKM training, MetaOrch fuzzy-model training, or any RL orchestration policy
- CRDT coordination, MPST protocol work, or event-triggered consensus
- a new runtime ingress, new benchmark program, or generic framework-imitation abstractions

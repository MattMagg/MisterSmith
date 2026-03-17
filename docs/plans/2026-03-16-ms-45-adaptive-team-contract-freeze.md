# MS-45 Adaptive-Team Contract Freeze

Date: March 16, 2026
Status: Complete

## Objective

Implement the first executable milestone of `MS-45` after the `014` SpecKit packet landed:
freeze one shared adaptive-team contract across `core`, `events`, and `orchestrator`, then prove
the contract with narrow serde/trait coverage.

This note corresponds to `T004` through `T007` in
`specs/014-task-shape-aware-orchestration/tasks.md`.

## Scope

- add shared adaptive-team value objects in `crates/mister-smith-core/src/autonomy.rs`
- expose the frozen operator-visible summary shape in `crates/mister-smith-events/src/autonomy.rs`
- project the frozen summary from `crates/mister-smith-agents/src/orchestrator.rs`
- update targeted tests in `crates/mister-smith-core/tests/trait_compilation_tests.rs` and
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- carry the new additive field through `crates/mister-smith-events/src/bus.rs` and
  `crates/mister-smith-app/tests/autonomy_status_tests.rs` as compile fallout without widening into
  the later operator-rendering lane
- update the `MS-45` workpad after validation so the parent checkpoint state stays honest

## Assumptions

- `MS-60` remains landed truth and should not be reworked during this milestone
- dynamic team sizing logic itself stays out of this slice; this milestone only freezes the shared
  contract used by later runtime and operator lanes
- `MS-61` and `MS-62` stay in `Backlog` until this checkpoint is validated

## Constraints

- keep the write set inside the packet's named choke-point files and test files
- do not widen into `team.rs`, `scheduler.rs`, `app/autonomy.rs`, or benchmark harness work yet
- preserve current autonomy-status and topology contracts unless the new adaptive-team fields are
  additive

## Non-Goals

- implementing adaptive worker-count selection
- changing queue state or adding `Symphony Candidate`
- creating new user-facing endpoints or a second operator surface

## Milestones

### 1. Contract definition

- introduce the shared adaptive-team value object in `core`
- re-export it from `lib.rs`

Validation:

- core types compile and serialize

### 2. Operator summary freeze

- add the operator-visible summary field in `events`
- project the summary in `orchestrator`

Validation:

- event and status views round-trip with the new field

### 3. Narrow proof and control-plane update

- run targeted crate tests for the touched seams
- update the `MS-45` workpad to reflect the checkpoint state

Validation:

- `cargo test -p mister-smith-core --test trait_compilation_tests`
- `cargo test -p mister-smith-events --test autonomy_event_tests`

## Decisions

- `TeamSizingDecision` follows the packet data model directly and keeps `decision_phase` as a
  frozen string field (`initial` for this slice).
- `AutonomyStatusView.team_sizing` is additive and optional in this slice so the orchestrator can
  emit the frozen contract now while full typed-event aggregation remains deferred to `T016`.
- The event bus only carries through snapshot-sourced `team_sizing` in this milestone; it does not
  yet synthesize the field from incremental autonomy events.

## Outcome

- Added the shared `TeamSizingDecision` contract in `mister-smith-core` and re-exported it.
- Added `team_sizing` to `AutonomyStatusView` and carried it through `StatusUpdated` snapshots.
- Extended `Orchestrator::autonomy_status()` to emit one baseline `initial` sizing decision from
  topology width, branch assignment availability, routing depth, and conservative reasons.
- Added compile/serde coverage in `trait_compilation_tests` and `autonomy_event_tests`.
- Added one agents-side status assertion and one app-side fixture update so the widened typed view
  stays validated across crate boundaries.

## Validation Evidence

- `cargo test -p mister-smith-core --test trait_compilation_tests`
- `cargo test -p mister-smith-events --test autonomy_event_tests`
- `cargo test -p mister-smith-agents gate10_mixed_dependency_resume_preserves_completed_branches -- --exact`
- `cargo test -p mister-smith-app --test autonomy_status_tests`
- `cargo build --workspace`

## Next Step

- Update the `MS-45` workpad to mark `T004` through `T007` complete.
- Re-evaluate `MS-61` queue posture now that the parent contract-freeze checkpoint is landed.

## Stop Conditions

- stop if the new summary requires non-additive changes to existing topology or routing contracts
- stop if the milestone starts requiring `team.rs`, `scheduler.rs`, or `app/autonomy.rs` changes
- stop if the new contract cannot be expressed as an additive status field and shared value object

# Packet 025 Implementation Status

Date: April 2, 2026
Branch: `025-step-level-intelligence-v2`
Packet: `specs/025-step-level-intelligence-v2/`
Status: Implemented and validation-complete for packet scope

## Objective

Initialize and execute packet `025` implementation from the current packet docs, keeping scope
limited to deterministic step-level policy and honest evidence projection only.

## Scope

- packet-owned `step_policy` contract and value objects
- deterministic step difficulty and bounded action selection
- projection through current task, autonomy, and operator surfaces
- targeted deterministic validation and packet evidence updates required by `tasks.md`

## Constraints

- do not redesign runtime execution or workflow ownership
- do not change packet `020` verifier or repair ownership
- do not change packet `023` runtime-truth or proof-boundary ownership
- do not change packet `024` boundary policy or execution authority
- do not add new endpoints, persistence, trace schema, or live-proof claims

## Non-goals

- runtime redesign
- coordinator or subagent runtime work
- interoperability work
- benchmark or training systems
- watched-queue or Linear lifecycle mutations for this run

## Assumptions

- packet `025` docs under `specs/025-step-level-intelligence-v2/` are the implementation
  authority for this run
- no active Linear issue or Codex workpad currently owns this packet lane, so this status note and
  `tasks.md` are the durable execution breadcrumbs
- the repo-local `/speckit.implement` flow applies after Smith-first preflight

## Preflight

- created feature branch `025-step-level-intelligence-v2` so
  `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` can run
- confirmed packet prerequisites resolve to
  `/Users/macmain/MisterSmith/specs/025-step-level-intelligence-v2`
- confirmed both packet checklists pass:
  - `requirements.md`: 16/16 complete
  - `step-policy.md`: 19/19 complete
- reviewed packet `spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, and
  `contracts/step-policy-contract.md`
- reviewed current code seams in:
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-app/src/autonomy.rs`
  - `crates/mister-smith-events/src/autonomy.rs`
  - `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - `apps/operator-console/src/types.ts`
  - `apps/operator-console/src/views/RunsView.tsx`

## Milestones

### Milestone 1: Shared contract freeze

Tasks:

- `T007` through `T010`

Validation:

- targeted Rust tests touching the new packet-owned contract
- packet docs still match the frozen payload shape

### Milestone 2: Deterministic scoring and bounded action policy

Tasks:

- `T011` through `T020`

Validation:

- targeted `mister-smith-app` test coverage for keep and non-keep decisions
- smoke-harness wording remains honest

### Milestone 3: Operator-facing summary projection

Tasks:

- `T021` through `T025`

Validation:

- task and autonomy projections stay aligned
- operator selected-run detail renders the same summary fields

### Milestone 4: Final validation

Tasks:

- `T026` through `T032`

Validation:

- all packet validation commands in `tasks.md`

## Stop Conditions

- packet scope starts drifting into runtime redesign, new endpoints, or ownership changes outside
  packet `025`
- the shared contract freeze reveals a conflict with packet `020`, `023`, or `024` authority
- targeted deterministic validation fails in a way that cannot be repaired inside packet scope

## Execution Summary

- Phase 1 contract freeze completed:
  - packet-owned step-policy value objects landed in `mister-smith-core`
  - shared `step_policy` fields landed in task, autonomy, and operator-preview projections
  - packet contract docs were tightened to the frozen field shape
- User Story 1 completed:
  - the terminal result path now assembles a deterministic packet-owned difficulty summary from
    verifier, routing, supervision, runtime-truth, and budget-root hints
  - `task.result.step_policy` now carries the packet-owned score summary when a terminal step
    evaluation exists
  - autonomy preview recovery promotes that summary back onto `AutonomyStatusView.step_policy`
- User Story 2 completed:
  - the step-policy decision ladder now keeps local clarification inside the bounded correction
    path and only widens to downgrade or escalate when the budget or risk signals justify it
  - hard-stop budget pressure is now reflected in packet-owned `budget_pressure` summaries
  - smoke-harness assertions now reject any packet-025 surface that weakens the packet-023
    task-proof wording
- User Story 3 completed:
  - human-readable autonomy status rendering now includes the packet-owned step-policy summary
  - operator previews keep packet-025 step-policy provenance visible when previews are inferred
  - the operator console selected-run detail now renders step, difficulty, chosen action, budget
    posture, input refs, and the packet-023 proof-boundary wording from the inspect payload
  - targeted UI and summary tests now cover the new packet-owned fields

## Validation Evidence

- `cargo test -p mister-smith-core`
- `cargo test -p mister-smith-events --test autonomy_event_tests`
- `cargo test -p mister-smith-app --test autonomy_status_tests`
- `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- `npm --prefix apps/operator-console test`
- `npx markdownlint-cli2 "specs/025-step-level-intelligence-v2/**/*.md" --config .markdownlint.json`
- `git diff --check`

## Blockers

- none

## Remaining Risks

- packet `025` still reports deterministic step-policy posture on top of packet-023 placeholder
  execution seams; it does not upgrade placeholder orchestration proof into grounded task proof
- no live runtime-proof claim was attempted in this lane

## Next Step

- optional review and commit/push lane if this packet slice should be landed next

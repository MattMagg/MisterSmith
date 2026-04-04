# Packet 026 Coordinator Runtime Proof Boundary

## Scope

This branch implements the bounded packet `026` slice only:

- coordinator-owned delegation visibility
- subordinate inbox and child-state visibility
- grounded delegated-work evidence and coordinator decisions
- one shared coordinator-runtime proof view on task result, autonomy status, and run detail
- bounded session follow-up references limited to stable IDs and evidence refs

This branch does not claim a new live runtime proof run. The validation below is deterministic
repo-local proof that packet `026` is wired into the current product surfaces.

## What Changed

- Added packet-026 shared value objects and proof view support in `mister-smith-core`
- Extended event, preview, and task-result contracts to carry `coordinator_runtime_proof`
- Synthesized packet-026 delegation, inbox, child-state, evidence, and coordinator-decision
  summaries from the current orchestrator and execution graph without reopening runtime design
- Added bounded `coordinator_runtime_follow_up` projection for retained assistant results
- Rendered packet-026 proof details in operator-console selected run detail
- Synced packet docs so the contract field names match the shipped payload shape

## Validation

- `cargo test -p mister-smith-core`
- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-events --test autonomy_event_tests`
- `cargo test -p mister-smith-app --test autonomy_status_tests`
- `cargo test -p mister-smith-app --test effect_boundary_projection_tests`
- `npm --prefix apps/operator-console test`
- `npm --prefix apps/operator-console run build`

## Proof Boundary

- Packet `026` success in this branch means the shared product surfaces can now show bounded
  coordinator-owned delegation records, child-state summaries, delegated-work evidence, and
  coordinator decisions.
- Sequential collapse remains an honest visible outcome and does not fabricate delegation.
- Session follow-up stays bounded to stable identifiers and evidence refs. It does not imply raw
  child transcript replay.
- Live runtime proof still requires a later real rerun on a selected provider and model path.

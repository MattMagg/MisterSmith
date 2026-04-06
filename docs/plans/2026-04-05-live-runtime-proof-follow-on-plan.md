# Live Runtime Proof Follow-On Plan

Updated: `2026-04-05T18:20:34Z`

## Objective

Implement and prove the next live-runtime lanes in this order:

1. packet `022`
2. packet `025`
3. packet `024`
4. packet `026`

## Scope

- add one harness-only runtime delay hook
- extend the repo-owned smoke harness with the four requested scenario names
- add a real session restart-resume lane for packet `022`
- make packet `025` step-policy show up on the supported placeholder runtime path without
  overstating proof
- add a repo-owned packet `024` MCP boundary probe
- add a stronger packet `026` explicit-parallel probe
- update the April 5 evaluation note and artifact tree as each lane lands

## Constraints

- keep prompts domain-neutral and not about Mister Smith repo work
- keep packet `023` repaired baseline as the control lane before each packet probe
- do not add new public product HTTP APIs unless a current surface is truly missing
- do not force a packet to pass if the current live surfaces cannot honestly prove it
- leave existing unrelated `.specify/` workspace changes untouched

## Non-Goals

- no new operator-console redesign
- no broad architecture rewrite
- no fake proof surfaces
- no repo-wide cleanup unrelated to runtime proof

## Milestones

### M1. Packet `022` and packet `025` runtime and harness repair

- add `MISTER_SMITH_LIVE_PROOF_DELAY_MS`
- add session-aware restart helpers to the smoke harness
- switch durable-history checks to the real `workflow_history` source
- project packet-025 `step_policy` on the supported placeholder path

Validation:

- targeted Rust tests for touched runtime logic
- `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- repaired baseline live lane
- packet `022` durable-resume live lane
- packet `025` step-policy live lane

Stop condition:

- baseline regresses
- restart-resume cannot keep stable `session_id` and `coordinator_agent_id`
- packet-025 proof wording drifts away from packet `023`

### M2. Packet `024` boundary probe and packet `026` parallel probe

- add repo-owned MCP boundary probe for allowed discover, allowed execute, and rejected execute
- add stronger coordinator-parallel lane and assertions
- fix explicit-parallel normalization if the runtime still collapses honest fan-out work

Validation:

- targeted Rust tests for touched crates
- `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- repaired baseline live lane
- packet `024` boundary probe
- packet `026` coordinator-parallel probe

Stop condition:

- MCP probe cannot exercise the actual descriptor/action boundary
- coordinator proof still reflects honest sequential collapse after explicit-parallel input

### M3. Notes and artifact closure

- update `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md`
- keep packet-lane artifacts under the April 5 artifact root
- update `docs/current-state.md` only if a repo-truth statement is false

Validation:

- `git diff --check`

## Current Next Step

Implement M1 in one patch set, then rerun the repaired baseline before the new packet lanes.

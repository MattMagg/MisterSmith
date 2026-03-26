# Tasks: Live Runtime Proof Smoke Harness

## T1. Freeze packet and planning artifacts

- [x] Add packet `018` artifacts
- [x] Add repo planning note in `docs/plans/`
- [x] Register the slice in validated backlog tracking

## T2. Add the smoke harness script

- [x] Add one repo-owned smoke harness under `scripts/`
- [x] Verify local prerequisites honestly and select a truthful NATS/JetStream evidence path
- [x] Submit one real task through `POST /api/v1/tasks`
- [x] Poll task and autonomy status to terminal state
- [x] Capture artifacts in a predictable output directory

## T3. Assert proof markers

- [x] Assert `runtime_execution_mode` markers on the terminal task result
- [x] Assert `execution_boundary = tool_bus` on step results
- [x] Assert basic autonomy-status invariants for the completed workflow

## T4. Add deterministic script coverage

- [x] Add helper/unit tests under `scripts/tests/`
- [x] Keep the script dry-run or helper paths testable without requiring a live runtime

## T5. Validate and refresh state if required

- [x] Run script tests
- [x] Run one local smoke pass if the environment supports it
- [x] Run Rust validation only if the slice touches Rust code
- [x] Update state-bearing docs only if shipped truth changed materially

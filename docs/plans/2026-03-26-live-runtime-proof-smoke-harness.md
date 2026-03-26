# 2026-03-26 Live Runtime Proof Smoke Harness

## Status

Implemented on branch and locally proven on 2026-03-26

## Objective

Turn the existing manual provider-backed live runtime proof path into one repeatable repo-owned
smoke harness that produces honest proof artifacts without depending on the flaky
`8222/healthz` probe.

## Repo-Grounded Current Truth

- `docs/plans/2026-03-19-live-run-trace-evaluation.md` records an honest manual live proof for the
  current `openai_chatgpt` / `gpt-5.4` runtime path and explicitly recommends a repo-owned smoke
  harness as the narrowest next step.
- The same evaluation records that `http://127.0.0.1:8222/healthz` was not a reliable verification
  surface in that run; JetStream availability had to be proved through runtime behavior and logs.
- `docs/current-state.md` still records the live proof baseline only for the default
  `openai_chatgpt` / `gpt-5.4` path and says fresh bounded product gaps must come from current repo
  and runtime evidence.
- There is no existing smoke script under `scripts/` that boots/verifies the local stack, submits a
  real task, and asserts the runtime/autonomy proof markers end to end.

## Scope

- add one repo-owned smoke harness script under `scripts/` for the current live runtime proof path
- verify or bootstrap the local Docker-backed prerequisites needed for the smoke run
- submit a real task through `POST /api/v1/tasks`, poll task completion, and fetch
  autonomy-status/task evidence
- assert the key `runtime_execution_mode` and `execution_boundary` proof markers plus basic
  autonomy-status invariants
- replace the old `8222/healthz` dependency with an honest NATS/JetStream verification step that
  matches what the local stack can actually prove
- capture artifacts in a predictable path so later operators can inspect or diff the results

## Assumptions

- the manual run shape from the March 19 evaluation is still the correct baseline for the current
  default runtime path
- a Python smoke harness is acceptable if it keeps process management and artifact assertions clear
- the current live proof surface should stay on `openai_chatgpt` / `gpt-5.4` unless deterministic
  script work reveals a reason to widen later

## Constraints

- no new runtime feature work
- no alternate-provider live proof claims in this slice
- no queue-stage or Symphony automation changes
- keep the initial write set bounded to `scripts/`, script tests, and only the docs needed to
  reflect the new repeatable proof surface

## Non-Goals

- no alternate-provider runtime proof
- no session-route proof expansion
- no budget-control-loop activation
- no permanent runtime daemon orchestration changes beyond what the smoke harness needs to run and
  shut down cleanly

## Milestones

### Milestone 1: Freeze packet and issue framing

Deliverables:

- this planning note
- packet `018` under `specs/`
- validated backlog issue `MS-102` with workpad context

Validation:

- note and packet cite the March 19 live proof note, current-state router, and explicit non-goals

### Milestone 2: Implement the smoke harness script

Deliverables:

- one script that verifies local prerequisites, creates or selects a fresh proof database, runs the
  runtime, submits a live task, polls results, and writes artifacts
- one honest NATS/JetStream verification path that does not depend on the flaky `8222/healthz`
  surface

Validation:

- deterministic unit tests for script helper logic
- one local scripted smoke run if the environment is available

### Milestone 3: Document the repeatable proof surface

Deliverables:

- predictable artifact output and usage guidance
- state-bearing docs updated only where the repo truth changed from "manual proof only" to
  "repeatable smoke harness exists"

Validation:

- script tests
- `cargo build --workspace` only if the slice touches Rust code

## Validation Evidence

- helper tests passed:
  - `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`
- live smoke proof passed:
  - `python3 scripts/live_runtime_proof_smoke.py`
- durable artifacts captured under:
  - `docs/plans/artifacts/live-runtime-proof-smoke/20260326T154005Z/`

## Landed Surface

- repo-owned harness: `scripts/live_runtime_proof_smoke.py`
- helper coverage: `scripts/tests/test_live_runtime_proof_smoke.py`
- proof flow:
  - Docker-backed `postgres` and `nats` bootstrap through `deploy/docker-compose.yml`
  - internal NATS `varz` verification from inside the container instead of the flaky host-side
    `8222/healthz` path
  - fresh proof database creation
  - provider-auth check for the bounded `openai_chatgpt` / `gpt-5.4` live path
  - real `POST /api/v1/tasks` submission, task polling, and autonomy-status capture
  - predictable artifact output under `docs/plans/artifacts/live-runtime-proof-smoke/`

## Stop Conditions

- the harness requires new runtime features instead of exercising current shipped behavior
- the environment cannot support a truthful local proof run without broader deploy/runtime changes
- the slice would widen into alternate-provider proof or control-loop work

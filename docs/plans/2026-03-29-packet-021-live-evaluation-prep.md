# 2026-03-29 Packet 021 Live Evaluation Preparation

## Summary

Packet `021` is closed on `main`, but its closure note is explicit that only deterministic proof
exists so far. This note prepares the next honest live evaluation pass for the new predictive
supervision surfaces without claiming that the pass has already succeeded.

Prepared probe surfaces:

- repo-owned smoke harness scenario: `packet021_supervision_probe`
- task inspect parity: `GET /api/v1/tasks/{task_id}`
- autonomy parity: `GET /api/v1/autonomy/status/{workflow_id}`
- operator run-detail verification: `apps/operator-console/` selected-run panel

Primary artifact root for the future live session:

- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/`

## Objective

Capture one bounded supported-ingress live run that either:

- proves packet-021 `supervision_evidence` appears as first-class runtime output on task inspect
  and autonomy status, with coherent proof-boundary fields, or
- fails honestly with artifacts that explain which packet-021 surface still remains live-unproven

## Scope

- reuse the existing repo-owned live smoke harness rather than inventing a separate runner
- add one packet-021-specific harness scenario that expects supervision evidence on both task and
  autonomy surfaces
- define the operator-console live check that should follow a successful task/autonomy probe
- keep packet-020 repair lineage as a coherence question, not a guaranteed required field for every
  run

## Assumptions

- the supported live provider path remains `openai_chatgpt` / `gpt-5.4`
- Docker-backed Postgres and NATS are still the local runtime prerequisites
- packet-021 evidence is reachable through the current `POST /api/v1/tasks` ingress rather than a
  new private probe route

## Constraints

- do not widen claims beyond one bounded supported-ingress run
- do not imply CKM, topology-search, benchmark, or alternate-provider proof
- do not claim packet-020 repair lineage must appear unless the live run actually exercises it
- keep task/autonomy proof separate from the operator-console visual check

## Non-Goals

- no new runtime feature work beyond evaluation preparation
- no queue, Symphony, or Linear mutation
- no new packet freeze or reopening of `MS-110`

## Evaluation Matrix

### Run 1: Supported baseline re-check

Purpose:

- confirm the known live provider path still boots, accepts work, and produces baseline artifacts
  before packet-021-specific interpretation

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --scenario baseline \
  --artifact-root docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/baseline
```

Expected outcome:

- runtime readiness markers appear
- the task reaches terminal state
- task inspect and autonomy status remain reachable

### Run 2: Packet-021 supervision probe

Purpose:

- attempt one bounded live run that should surface packet-021 predictive-supervision evidence on
  the supported ingress

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --scenario packet021_supervision_probe \
  --artifact-root docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/probe
```

The prepared harness now checks:

- `task.result.supervision_evidence` exists
- `autonomy_status.supervision_evidence` exists
- both surfaces agree on target scope, decision basis, proof boundary, fingerprint key, and any
  packet-020 repair-lineage reference that appears
- the supervision target kind is one of `branch`, `node`, or `graph`
- at least one detailed packet-021 payload block exists:
  - `fingerprint_ref`
  - `profile_snapshot`
  - `guard_decision`
  - `intervention_record`

Interpretation rules:

- if this run passes, packet-021 earns one bounded live task/autonomy proof on the supported
  ingress
- if this run fails because supervision evidence never appears, treat packet-021 as still
  deterministically proven only; do not label it a regression unless Run 1 also fails or the
  runtime contradicts landed packet-021 task/autonomy contracts
- if the run emits supervision evidence but no packet-020 repair lineage, record that absence
  honestly instead of forcing a lineage claim

### Run 3: Operator-console selected-run verification

Purpose:

- verify that the live task from Run 2 renders the packet-021 panel in the shipped UI, not only in
  raw JSON payloads

Suggested sequence:

1. Start the operator console against the live runtime used for Run 2.
2. Open the completed run by `task_id`.
3. Confirm the `Predictive supervision` panel renders the same target scope, decision basis,
   proof-boundary text, and any fingerprint or repair-lineage details observed in Run 2.
4. Save one screenshot plus any browser-console output under
   `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/ui/`.

## Questions The Live Session Must Answer

1. Did the supported ingress still run on `openai_chatgpt` / `gpt-5.4`?
2. Did packet-021 supervision evidence appear on task inspect?
3. Did the same supervision evidence appear on autonomy status with matching proof fields?
4. What was the observed target scope: `branch`, `node`, or `graph`?
5. Which packet-021 evidence blocks were directly observed live?
6. Did packet-020 repair lineage appear, and if so, was it coherent with packet-021?
7. Did the operator console render the supervision panel from the live run without needing raw JSON?
8. If the probe failed, did it fail before runtime execution, before supervision emission, or only
   at the UI/rendering layer?

## Prepared Validation For This Prep Slice

These checks validate the preparation work itself, not the future live packet-021 proof:

```bash
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
python3 -m py_compile scripts/live_runtime_proof_smoke.py
git diff --check -- scripts/live_runtime_proof_smoke.py scripts/tests/test_live_runtime_proof_smoke.py docs/plans/2026-03-29-packet-021-live-evaluation-prep.md
```

## Stop Conditions

- the baseline run fails, which makes packet-021-specific interpretation premature
- the packet-021 probe requires a new private ingress or unsupported operator injection path
- the only way to force supervision evidence is by widening scope into new runtime feature work

# 2026-03-29 Packet 021 Live Evaluation

## Summary

Current `main` already includes the packet-021 follow-up fix from `2b0d17a` (`fix: populate
packet-021 live supervision evidence (#251)`) plus the later docs sync on `8346420`.

Provider/model exercised for this evaluation:

- `openai_chatgpt`
- `gpt-5.4`

Bottom line from the original live pass:

- the supported ingress still boots and completes work on the current head
- the packet-021 probe can now reach a completed hybrid graph when the planner wording stays within
  the supported topology contract
- no completed live run in this pass emitted non-null `supervision_evidence` on either
  `GET /api/v1/tasks/{task_id}` or `GET /api/v1/autonomy/status/{workflow_id}`
- the only live recovery lineage observed was packet-020-style `orchestration_quality` inferred
  from a planner repair step, not packet-021 predictive-supervision evidence

Packet-021 therefore remained not live-proven on the supported task ingress at the start of this
note. The later follow-up section records the local root-cause fixes and the newer live rerun.

## Baseline

Artifact:

- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/baseline/20260330T005745Z/`

Observed result:

- scenario: `baseline`
- task: `a7e4e7fd-3d15-4efa-b2db-16c8d8fcae80`
- proof outcome: `graph_formed_and_completed`
- topology: `Sequential`
- parallelism width: `1`
- task supervision: `null`
- autonomy supervision: `null`

This re-confirmed the known supported provider path before packet-021-specific interpretation.

## Deterministic Validation

No code changed during this evaluation pass. The repo-owned smoke harness had already been
deterministically validated in the preparation slice recorded in
`docs/plans/2026-03-29-packet-021-live-evaluation-prep.md`.

This note records live runtime execution only.

## Live Runs

### Run 1: Packet-021 probe with unsupported topology wording

Artifact:

- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/probe/20260330T005841Z/`

Observed result:

- status: `failed`
- proof outcome: `failed_before_graph`
- runtime error:
  `workflow planning produced an invalid execution graph during topology compilation:
  Unsupported topology contract: unsupported topology hint 'two parallel worker branches, then one
  coordinator synthesis step'`

Interpretation:

- this was a prompt-shape failure at topology compilation time
- it did not exercise packet-021 runtime supervision

### Run 2: Packet-021 hybrid probe with supported explicit-parallel wording

Artifact:

- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/probe/20260330T010002Z/`

Observed result:

- status: `completed`
- proof outcome: `graph_formed_and_completed`
- topology: `Hybrid`
- parallelism width: `2`
- task result `supervision_evidence`: `null`
- autonomy `profiles`: `[]`
- autonomy `guard_decisions`: `[]`
- autonomy `interventions`: `[]`
- autonomy `supervision_evidence`: `null`

Live packet-020 repair telemetry did appear:

- `orchestration_quality.step_id = result_track`
- `orchestration_quality.repair_action = retry_step`
- `orchestration_quality.failure_context_ref = planner:result_track/retry_step`
- autonomy provenance explicitly said the repair summary was inferred from a planner repair step
  without verifier policy

Interpretation:

- the runtime completed a supported hybrid graph
- packet-020 repair provenance remained visible
- packet-021 predictive-supervision state did not appear on task or autonomy surfaces

### Run 3: Repo-owned repair probe

Artifact:

- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/repair-probe/20260330T010615Z/`

Observed result:

- scenario: `repair_probe`
- task: `b3f782ec-dcfd-40e0-a150-95d1f8afd63b`
- proof outcome: `graph_formed_and_completed`
- topology: `Sequential`
- task supervision: `null`
- autonomy supervision: `null`

Interpretation:

- the repair-oriented prompt still did not surface live packet-021 evidence

### Run 4: Signal-oriented sequential probe

Artifact:

- `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/signal-probe/20260330T010707Z/`

Observed result:

- scenario: `baseline` with a custom signal-oriented task description
- task: `753552e6-ccea-4006-b207-2c6661ba602e`
- proof outcome: `graph_formed_and_completed`
- topology: `Sequential`
- task supervision: `null`
- autonomy supervision: `null`

Interpretation:

- even a prompt that tried to provoke a stream-level degradation signal did not produce observable
  packet-021 supervision on the supported ingress

## Evaluation Result

The packet-021 repair-lineage/proof-boundary fix is present on the evaluated head, but this live
pass still did not produce any non-empty packet-021 runtime evidence.

What the current head now proves live:

- the supported task ingress still runs on `openai_chatgpt` / `gpt-5.4`
- baseline sequential execution still completes
- supported hybrid planning still completes when the prompt uses the accepted topology wording
- packet-020 planner-repair provenance can still appear on the live result path

What this live pass did not prove:

- a non-empty `SupervisionEvidenceView` on task inspect
- matching packet-021 supervision evidence on autonomy status
- live profile snapshot, guard decision, or intervention records on the supported ingress
- operator-console rendering of a live packet-021 supervision panel

Current honest boundary:

- packet-021 remains deterministically proven
- packet-021 remains live-unproven on the supported HTTP task path

## Root Cause Follow-up

Follow-up investigation on the workspace after the live pass found two separate local runtime gaps.

### Gap 1: completed repair steps never entered packet-021 supervision

- planner-directed repair steps on the description-only ingress were producing an accepted
  `StepEvaluationRecord` through `runtime_repair_step_evaluation(...)`
- that accepted repair evaluation was enough for packet-020 `orchestration_quality` inference
- the runtime only invoked packet-021 supervision recording on `Continue` and `Failed` step
  dispositions, not on `Completed` steps carrying a repair directive
- result: a live run could show packet-020 planner-repair provenance while still never recording
  packet-021 profile, guard, intervention, or `supervision_evidence`

Bounded workspace fix applied after this evaluation note was first written:

- the completed-step execution branch now calls runtime supervision when a completed step carries
  a repair evaluation
- a focused unit test now proves that a description-only planner repair step can build a
  packet-021 runtime supervision plan

### Gap 2: fingerprint persistence reused an invalid KV key format

The first seeded rerun after Gap 1 exposed a second live blocker:

- artifact:
  `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/repair-seeded/20260330T011640Z/`
- proof outcome: `failed_before_graph`
- live error:
  `runtime supervision failed for completed step ...: runtime fingerprint lookup failed for workflow target agent:...: Database operation failed: key cannot be empty or start/end with \`.\``

Root cause:

- `ProfileFingerprintStore` reused the same helper for both the persisted KV key and the
  operator-visible fingerprint label
- that helper produced `profile-fingerprint:agent:<selector>`
- async-nats JetStream KV only accepts key characters matching `[-/_=.a-zA-Z0-9]`
- the colon-delimited key therefore failed validation even though the error text only mentioned
  empty keys or leading/trailing dots

Bounded workspace fix:

- fingerprint persistence now uses a KV-safe store key
  `profile-fingerprint/<kind>/<selector>`
- operator-visible references keep the packet-021 contract shape
  `<kind>:<selector>`
- focused unit coverage now proves the storage key and operator label are distinct

Deterministic follow-up validation on the patched workspace passed:

```bash
cargo fmt --all
cargo test -p mister-smith-persistence profile_fingerprint_ -- --nocapture
cargo test -p mister-smith-app runtime_supervision_plan_targets_node_for_description_only_repair_step -- --nocapture
cargo test -p mister-smith-app description_only_repair_step_records_runtime_generated_step_evaluation -- --nocapture
```

The immediate topology-sensitive rerun after Gap 1 still did not reach graph execution because the
planner drifted back to an unsupported topology hint:

- artifact:
  `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/probe/20260330T011341Z/`
- proof outcome: `failed_before_graph`
- runtime error:
  `workflow planning produced an invalid execution graph during topology compilation:
  Unsupported topology contract: unsupported topology hint 'two parallel branches followed by one
  coordinator merge'`

### Latest seeded live rerun after both fixes

The latest seeded sequential rerun proves the original null-supervision problem is fixed locally:

- artifact:
  `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/repair-seeded/20260330T012151Z/`
- task result now carries non-null packet-021 `supervision_evidence`
- the payload includes:
  - `target_scope.kind = node`
  - `decision_basis = live_signals_only`
  - non-null `guard_decision`
  - non-null `intervention_record`
  - non-null `profile_snapshot`
  - `proof_boundary = supported task path`

The rerun still failed, but for a different reason:

- runtime error:
  `failed to complete scheduled task ...: Scheduling error: Cannot complete task in Pending state`

Observed cause from the recorded `intervention_record` plus runtime code path:

- completed-step supervision now runs before `record_completed_step(...)`
- `orchestrator.supervise(...)` applies the chosen `ContextRefresh` intervention immediately
- node-scoped `ContextRefresh` resets the scheduled task from `Running` back to `Pending`
- the executor then calls `scheduler.complete(...)` for the same task and hits the state
  transition failure

Current honest status after the follow-up:

- the original packet-021 null-payload cause is understood and fixed locally
- fingerprint persistence no longer blocks completed-step supervision
- packet-021 supervision can now be emitted on the supported task path
- a new scheduler-state conflict now blocks a clean successful live proof for the seeded repair run

## Remaining Limits

- The supported description-only task ingress does not currently provide a reliable operator-facing
  way to trigger verifier-policy local repair from the submitted request alone.
- The successful hybrid run earlier in this pass still only showed packet-020 planner-repair
  provenance; it did not pass through the packet-021 live supervision path before the local fixes.
- The latest seeded rerun now exposes live packet-021 supervision, but it still fails because the
  applied `ContextRefresh` intervention resets the same node to `Pending` before completion is
  recorded.
- The operator-console verification from the prep note is still intentionally skipped because
  there is not yet one clean successful live run that both emits packet-021 supervision and
  reaches terminal completion on the supported task path.

## Cleanup

- each runtime launched by the smoke harness was stopped cleanly
- temporary evaluation databases were isolated per run under the harness-managed naming scheme
- durable artifacts were kept under `docs/plans/artifacts/2026-03-29-packet-021-live-evaluation/`
- worktree delta from this evaluation consists of this note plus the artifact bundle

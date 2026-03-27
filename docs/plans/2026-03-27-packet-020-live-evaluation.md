# 2026-03-27 Packet 020 Live Evaluation

## Summary

Packet `020` was evaluated on `main` at `b66766a92d9d5909e42d906e5520ac5a86f75c40` with the
runtime still on the supported provider-backed baseline `openai_chatgpt` / `gpt-5.4`.

This session ran:

- one baseline smoke-harness run through the shipped proof path
- one manual packet-020-focused probe through the same supported `POST /api/v1/tasks` ingress

Bottom line:

- the shipped baseline live path still works and completes real multi-step workflows
- the baseline harness currently exits non-zero on current `main` even when the live run itself
  succeeds, because it now treats a missing `triggered_checkpoints` array as an error on the
  round-robin baseline path
- the bounded packet-020 probe failed before graph publication with
  `Unsupported topology contract: unsupported planner role 'join'`
- no live verifier verdict, repair action, clarification attempt count, checkpoint reference,
  failure-context reference, or orchestration-quality outcome summary appeared in either run
- packet `020` remains live-unproven for verifier-gated clarification or repair behavior on the
  supported description-only ingress exercised here

Primary artifact lane:

- `docs/plans/artifacts/2026-03-27-packet-020-live-evaluation/`

## Baseline

- Repo state at start:
  - branch `main`
  - head `b66766a92d9d5909e42d906e5520ac5a86f75c40`
  - `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync` passed
- Control-plane checkpoint:
  - `MS-103` remained `Done`
  - no Linear or queue mutation was required
- Provider auth:
  - `cargo run -q -p mister-smith-app -- auth openai-chatgpt status`
  - result: authenticated ChatGPT account present
- Infra before live runs:
  - `deploy-postgres-1` healthy on `127.0.0.1:5432`
  - `deploy-nats-1` healthy on `127.0.0.1:4222`
  - NATS monitor available on `127.0.0.1:8222`
  - `curl http://127.0.0.1:8222/jsz` showed JetStream enabled with `streams=2`
- Current operator surfaces used:
  - `POST /api/v1/tasks`
  - `GET /api/v1/tasks/{task_id}`
  - `GET /api/v1/autonomy/status/{workflow_id}`
  - `mister-smith autonomy list`
  - `mister-smith autonomy status --workflow-id <id>`

Files read for grounding:

- `AGENTS.md`
- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `docs/current-state.md`
- `docs/ms_recent_context.md`
- `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
- `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
- `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
- `specs/020-verifier-gated-adaptive-orchestration/spec.md`
- `specs/020-verifier-gated-adaptive-orchestration/quickstart.md`
- `specs/020-verifier-gated-adaptive-orchestration/tasks.md`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/src/agent_inspection.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-core/src/supervision.rs`
- `scripts/live_runtime_proof_smoke.py`

Preflight environment capture:

- `docs/plans/artifacts/2026-03-27-packet-020-live-evaluation/manual-env.txt`

## Deterministic Validation

These packet-020 quickstart checks passed before the live proof:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-app
cargo clippy -p mister-smith-core -- -D warnings
cargo clippy -p mister-smith-app -- -D warnings
git diff --check
```

These checks prove the verifier and repair control-loop semantics deterministically, but they do
not by themselves prove live packet-020 behavior.

## Live Runs

### Run 1: Baseline smoke harness

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --artifact-root docs/plans/artifacts/2026-03-27-packet-020-live-evaluation/baseline
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-packet-020-live-evaluation/baseline/20260327T161631Z/`

Observed live evidence:

- runtime startup log showed:
  - `JetStream stream created/updated`
  - `Runtime task execution service ready`
  - `Mister Smith ready`
- provider/model:
  - `provider_kind=openai_chatgpt`
  - `model_id=gpt-5.4`
- routing/runtime mode:
  - `routing_policy=round_robin`
  - `registered_provider_count=1`
  - `budget_root=disabled`
  - `execution_boundary=tool_bus`
  - `workflow_runner=tokio_task`
  - `planner_lifecycle=supervised_actor`
  - `executor_lifecycle=supervised_actor`
- accepted workflow/task id:
  - `f0cb17f4-ee67-406f-9b94-bb70979e5aa6`
- terminal result:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - `step_result_count=3`
  - topology `Hybrid`
- autonomy status was present and agreed with the task result

What this run proved live:

- the current shipped baseline path still accepts a real task, executes parallel worker steps plus
  a merge step, and projects terminal autonomy/task evidence on the supported
  `openai_chatgpt` / `gpt-5.4` path

Packet-020-specific evidence from this run:

- directly observed:
  - none of the packet-020 verifier/repair fields
- absent:
  - `orchestration_quality`
  - verifier `verdict`
  - `repair_action`
  - `clarification_attempt_count`
  - `checkpoint_ref`
  - `last_stable_step_id`
  - `failure_context_ref`
  - `outcome_summary`
- observed provenance still showed:
  - `latest step routing tier=direct action=continue checkpoints=none`

Harness mismatch:

- the harness exited with:

  ```text
  ERROR: latest step_routing_history entry was missing checkpoints
  ```

- the generated `smoke-run-config.json` had `required_step_checkpoints: []`, so this non-zero
  exit was stricter than the configured baseline requirement
- the live run itself completed; the failure was in post-run harness validation, not runtime
  startup or workflow completion

### Run 2: Manual packet-020-focused probe

Runtime command:

```bash
env DATABASE_URL='postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_packet020_live_eval_20260327_probe' \
  MISTER_SMITH_TRANSPORT__NATS_URL='nats://127.0.0.1:4222' \
  MISTER_SMITH_TRANSPORT__HTTP_PORT='63160' \
  cargo run -q -p mister-smith-app -- run
```

Probe request:

- `docs/plans/artifacts/2026-03-27-packet-020-live-evaluation/probe/task-request.json`

Accepted workflow/task id:

- `83531132-4f6b-414e-84e1-36852c1dfac6`

Observed live evidence:

- runtime startup again showed:
  - `JetStream stream created/updated`
  - `Runtime task execution service ready`
  - `Mister Smith ready`
- provider/model:
  - `provider_kind=openai_chatgpt`
  - `model_id=gpt-5.4`
- the request was accepted through `POST /api/v1/tasks`
- the run terminated as:
  - `status=failed`
  - `proof_outcome=failed_before_graph`
- task result error:

  ```text
  workflow planning produced an invalid execution graph during topology compilation: Unsupported topology contract: unsupported planner role 'join'
  ```

- runtime log matched the task error:

  ```text
  Workflow run failed ... Unsupported topology contract: unsupported planner role 'join'
  ```

- autonomy status was still available and preserved parity with the failure-visible result
- the planner output in `task-status-latest.json` showed the final merge step persisted as
  `role: "join"` and triggered the same compile-time rejection

What this run proved live:

- the supported description-only ingress can still hit a real `failed_before_graph` path and
  preserve failure-visible task plus autonomy parity on current `main`

What this run did not prove live:

- no verifier gate fired before topology compilation
- no clarification request surfaced
- no retry or re-plan from checkpoint occurred
- no packet-020 orchestration-quality view appeared in the task or autonomy outputs

Packet-020-specific evidence from this run:

- directly observed:
  - none of the packet-020 verifier/repair fields
- absent:
  - `orchestration_quality`
  - verifier `verdict`
  - `repair_action`
  - `clarification_attempt_count`
  - `checkpoint_ref`
  - `last_stable_step_id`
  - `failure_context_ref`
  - `outcome_summary`
  - `clarification_request`
  - `repair_directive`
  - `failure_context_checkpoint`
- inferred from code, not observed live:
  - packet-020 fields are available on task/autonomy views when a step evaluation actually occurs
  - current supported live ingress does not provide a dedicated structured verifier-policy input

## Evaluation Result

Answers to the required evaluation questions:

1. **Did the live runs use the current runtime path described by the code?**
   - Yes. Both runs used the supported runtime-backed task path:
     `POST /api/v1/tasks` with the current app binary, supervised planner/executor, ToolBus
     execution, and autonomy inspection via `GET /api/v1/autonomy/status/{workflow_id}`.
2. **What exact provider/model path did each run use?**
   - Both runs used `openai_chatgpt` / `gpt-5.4`.
3. **What did the baseline run prove?**
   - It re-proved the packet-019-style shipped baseline path: readiness, real task acceptance,
     multi-step completion, task result retrieval, and autonomy visibility on the default
     round-robin single-provider path.
4. **Did any run prove packet-020-specific live behavior?**
   - No.
5. **Which packet-020 fields or behaviors were directly observed, absent, or only inferred from
   code?**
   - Directly observed: none of the packet-020 verifier/repair fields.
   - Absent: verifier verdict, repair action, clarification count, checkpoint reference, last
     stable step, failure-context reference, outcome summary, clarification request, repair
     directive, failure-context checkpoint.
   - Only inferred from code: these fields exist in current `main` and are projected when a real
     step evaluation occurs.
6. **If packet-020 behavior did not appear, what prevented an honest proof?**
   - The baseline run stayed on the normal happy path and never triggered verifier or repair
     lineage.
   - The bounded probe failed earlier, during topology compilation, with an unsupported planner
     role `join`, before any verifier-gated step evaluation could occur.
   - The supported live ingress is still description-only, so no supported operator input exists to
     inject a verifier-policy sequence directly.
7. **What parts of packet `020` remain deterministic-only on current evidence?**
   - live verifier verdict proof
   - live clarification proof
   - live retry-step proof
   - live replan-from-checkpoint proof
   - live task/autonomy `OrchestrationQualityView` proof
8. **Do the observed results match `docs/current-state.md` and the packet-020 closure note?**
   - Yes on broad routing truth: packet `020` is landed on `main`, the runtime path is still on
     the `openai_chatgpt` / `gpt-5.4` baseline, and the packet-020 closure note explicitly says it
     introduced no new live-proof claim by itself.
   - No new contradiction was found there.
   - The live probe did expose one practical mismatch in repo-owned proof tooling: the baseline
     smoke harness currently treats missing baseline checkpoints as an error.
9. **What is the narrowest honest next step if a proof gap or regression remains?**
   - First, fix the baseline smoke harness so a round-robin baseline run with no
     `triggered_checkpoints` does not fail when no checkpoint requirement was configured.
   - Then, add or identify one supported repeatable live probe that can reach a verifier-gated
     step evaluation without relying on unsupported structured policy injection or on planner
     output that fails earlier at join-role compilation.

## Remaining Limits

- This session did not produce a bounded live transcript that showed clarification, retry, or
  re-plan behavior on the supported ingress.
- The manual probe failure remained in the older planner/compiler failure family rather than the
  verifier/repair family.
- The only live-proven provider/model path in this note remains `openai_chatgpt` / `gpt-5.4`.
- No benchmark, broader orchestration-quality, or production-readiness claim is justified from
  this evidence.

## Cleanup

- the baseline smoke harness shut its runtime down automatically
- the manual probe runtime was stopped cleanly with `SIGINT`
- the temporary probe database `mistersmith_packet020_live_eval_20260327_probe` was dropped
- the durable artifacts remain under `docs/plans/artifacts/2026-03-27-packet-020-live-evaluation/`

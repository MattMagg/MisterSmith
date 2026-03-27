# 2026-03-27 Runtime Planning Simplification

## Summary

This pass removed the rigid live-task shaping that was forcing the runtime toward a fixed
two-worker-plus-merge pattern, normalized merge steps structurally, and aligned the smoke harness
with what the live runtime now actually does.

Changed code surfaces:

- `crates/mister-smith-agents/src/roles/planner.rs`
- `crates/mister-smith-app/src/execution.rs`
- `scripts/live_runtime_proof_smoke.py`

Primary artifact lane:

- `docs/plans/artifacts/2026-03-27-runtime-planning-simplification/`

Follow-up issues logged in Linear:

- `MS-108` umbrella: reduce over-shaped runtime planning and restore natural task completion
- `MS-109`: fix merge-step normalization and reject illegal join roles on the live path
- `MS-110`: adaptive runtime topology selection planning follow-up
- `MS-111`: simplify planner prompt contract so workflows are less programmatic
- `MS-112`: align smoke harness and runtime-proof claims with actual topology behavior

Bottom line:

- simple prompts now stay sequential on the live path
- explicitly parallel prompts still produce a legal fanout-plus-coordinator workflow
- non-memo prompts preserve the requested output shape
- packet-020-style repair probes no longer fail on `unsupported planner role 'join'`
- packet-020 verifier and repair provenance fields still did not surface on the supported live
  ingress, so that proof remains incomplete

## Baseline

The implementation goal was to make the runtime choose the smallest workflow that can finish the
task instead of imposing a canned shape.

Concretely, this pass changed three things:

1. Runtime planning context:
   - removed the hardcoded `require_parallel_workers = 2`
   - removed the hardcoded `require_join_step = true`
   - replaced that with advisory workflow policy guidance:
     - prefer the smallest workflow
     - default to sequential
     - branch only for clearly independent work
     - merge only when multiple branch outputs need consolidation
     - preserve the requested output shape
2. Explicit-plan normalization:
   - multi-dependency steps now normalize to `role: "coordinator"` by structure
   - merge branches normalize internally to join-style labels
   - the old live-path trap where a merge step could survive as `role: "join"` is no longer the
     normal outcome
3. Proof harness:
   - missing optional `triggered_checkpoints` no longer causes a false harness failure
   - the harness now supports a repeatable scenario matrix instead of only one rigid baseline prompt

## Deterministic Validation

These deterministic checks passed on the implementation worktree:

```bash
cargo fmt --all
cargo test -p mister-smith-app
cargo test -p mister-smith-agents
cargo clippy -p mister-smith-app -- -D warnings
cargo clippy -p mister-smith-agents -- -D warnings
python3 -m py_compile scripts/live_runtime_proof_smoke.py
git diff --check
```

These checks support the implementation but do not replace live proof.

## Live Runs

All live scenarios below ran on the supported provider path:

- provider: `openai_chatgpt`
- model: `gpt-5.4`
- ingress: `POST /api/v1/tasks`
- autonomy inspection: `GET /api/v1/autonomy/status/{workflow_id}`

### Baseline scenario

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --scenario baseline \
  --artifact-root docs/plans/artifacts/2026-03-27-runtime-planning-simplification/baseline
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-runtime-planning-simplification/baseline/20260327T171258Z/`

Observed live result:

- topology `Sequential`
- `parallelism_width = 1`
- `branch_count = 1`
- `node_count = 3`
- the generated plan was a three-step sequential chain on one branch
- no forced merge step was synthesized

### Explicit parallel scenario

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --scenario explicit_parallel \
  --artifact-root docs/plans/artifacts/2026-03-27-runtime-planning-simplification/explicit_parallel
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-runtime-planning-simplification/explicit_parallel/20260327T171355Z/`

Observed live result:

- topology `Hybrid`
- `parallelism_width = 2`
- `branch_count = 3`
- `node_count = 3`
- the two independent tracks remained parallel workers
- the final consolidation step compiled and ran as `role: "coordinator"` on branch `join`

This proved that removing the fixed shaping did not break legitimate parallel work.

### Non-memo scenario

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --scenario non_memo \
  --artifact-root docs/plans/artifacts/2026-03-27-runtime-planning-simplification/non_memo
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-runtime-planning-simplification/non_memo/20260327T171444Z/`

Observed live result:

- topology `Sequential`
- `parallelism_width = 1`
- planner steps were shaped around the requested sections:
  - `Problem`
  - `Evidence`
  - `Smallest Fix`
- the workflow goal explicitly preserved the requested answer shape instead of reverting to a canned
  operator memo

### Packet-020-style repair probe

Command:

```bash
python3 scripts/live_runtime_proof_smoke.py \
  --profile baseline \
  --scenario repair_probe \
  --artifact-root docs/plans/artifacts/2026-03-27-runtime-planning-simplification/repair_probe
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-runtime-planning-simplification/repair_probe/20260327T171540Z/`

Observed live result:

- topology `Sequential`
- `parallelism_width = 1`
- the old topology failure did not recur
- the middle step was planned as `resolve_missing_context`
- the run completed with `proof_outcome = graph_formed_and_completed`

What this did prove:

- packet-020-style instructions no longer get masked by the earlier `role: "join"` topology
  failure
- the planner/runtime can now carry a bounded missing-context step through to completion on the
  live path

What this did not prove:

- no `orchestration_quality` field was emitted
- no verifier `verdict` surfaced
- no `clarification_attempt_count` surfaced
- no `repair_action` surfaced
- no `checkpoint_ref` or `failure_context_ref` surfaced

So the live repair probe is now topology-clean, but packet-020 proof is still not complete at the
operator-surface level.

## Evaluation Result

This implementation achieved the immediate runtime-planning simplification goals:

- the pre-shaped live planning contract is removed
- the runtime now defaults to the smallest workflow that fits the task
- merge behavior is legal and structural rather than dependent on brittle role wording
- the planner prompt is less programmatic and preserves requested output shape
- the smoke harness now evaluates scenario-specific topology expectations without false checkpoint
  failures

The live matrix also changed the failure story materially:

- before this pass, the packet-020-style probe failed before graph formation on
  `unsupported planner role 'join'`
- after this pass, the same class of probe forms a graph and completes

That is real progress, but it is not the same as proving full packet-020 verifier or repair
provenance on the supported ingress.

## Remaining Limits

- Packet-020-specific operator fields are still absent on the supported task and autonomy surfaces
  used here.
- The broader question of when the runtime should choose sequential versus parallel versus merged
  topology still deserves a scoped planning packet; that is tracked in `MS-110`.
- Proof claims should stay bounded to what was actually observed in
  `docs/plans/artifacts/2026-03-27-runtime-planning-simplification/`.

## Cleanup

- All live runs were executed through the harness and left artifact bundles under the dated lane.
- No temporary pycache output from the harness was kept.
- Remaining worktree delta from this pass is limited to:
  - the three implementation files above
  - this note
  - the new runtime-planning artifact lane
  - the previously created packet-020 evaluation note and artifact lane

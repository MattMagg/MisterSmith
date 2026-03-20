# 2026-03-20 MS-95 Post-Merge Re-Evaluation

## Summary

This is a fresh live re-evaluation of the merged `MS-95` state on `main` at `b9aefc30e9f65efe1d4edf8a1d413ec99b5ad055`.
The goal was to verify, from a new runtime-backed run, whether the bounded packet-015 follow-up
gap is actually closed:

- preserve `failed_before_graph` on the task surface
- retain the same result on the session surface
- expose a bounded autonomy status for the failed workflow

The bounded `MS-95` fix is real on the workflow-level autonomy surface.

Fresh live result:

- `failed_before_graph` is now reproducible and preserved on task, session, and workflow-level
  autonomy status
- the prior March 20 failure-visible parity gap is closed for the supported autonomy HTTP/CLI path
- trivial prompts used to recheck `collapsed_to_sequential` no longer collapse in fresh live runs;
  they now form small hybrid graphs instead

Artifacts:
`docs/plans/artifacts/2026-03-20-ms-95-post-merge-re-evaluation/`

## Baseline

- Repo state at start: clean `main`, synced to `origin/main`
- Head commit subject: `fix(app): preserve failed-before-graph autonomy status (#225)`
- Provider/model: `openai_chatgpt` / `gpt-5.4`
- Runtime path under test: `target/debug/mister-smith run`
- Runtime execution mode observed in task/session/autonomy payloads:
  - `execution_boundary=tool_bus`
  - `workflow_runner=tokio_task`
  - `planner_lifecycle=supervised_actor`
  - `executor_lifecycle=supervised_actor`
- Temporary database: `mistersmith_ms95_post_merge_eval_20260320`
- HTTP port: `63130`

## Deterministic Validation

Re-ran the landed local bundle before the live proof:

```bash
cargo fmt --all
cargo build --workspace
cargo test -p mister-smith-app
git diff --check
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

Results:

- `cargo fmt --all`: passed
- `cargo build --workspace`: passed
- `cargo test -p mister-smith-app`: passed
- `git diff --check`: passed
- `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`: failed only after
  this evaluation started creating new artifact files

Notable passing coverage in `mister-smith-app`:

- `recover_persisted_autonomy_status_falls_back_to_metadata_final_result_when_task_result_mismatches`
- `recover_persisted_autonomy_status_synthesizes_failed_before_graph_without_snapshot`
- `synthesize_failed_before_graph_status_preserves_hybrid_fanout_join_width`
- `synthesize_failed_before_graph_status_uses_frontier_width_for_single_root_fanout`

## Important Route Clarification

The supported HTTP autonomy status route on this merged head is:

- `GET /api/v1/autonomy/status/{workflow_id}`

The following paths returned `404` during this rerun and should not be treated as the active
contract:

- `GET /api/v1/autonomy/sessions/{session_id}/turns/{turn}/status`
- `GET /api/v1/autonomy/workflows/{workflow_id}/status`

The CLI matches the supported route:

- `mister-smith autonomy list --base-url http://127.0.0.1:63130`
- `mister-smith autonomy status --workflow-id <workflow_id> --base-url http://127.0.0.1:63130`

## Live Runs

### Run 1: Success Case Still Works

- Session id: `9d86d915-5d89-42d7-ba65-7f020a5b12ed`
- Workflow id: `715f60e8-d9a7-4c7b-aae0-71ddb8078932`
- Prompt shape: three parallel evidence tracks plus one join memo

Observed result:

- task surface:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - `execution_plan.steps=4`
  - `step_results=4`
- session surface:
  - `turn_count=1`
  - `last_assistant_result.assistant_result.proof_outcome=graph_formed_and_completed`
  - provenance source fields:
    - `metadata.final_result`
    - `metadata.aggregated_result`
- workflow autonomy surface:
  - `graph.state=Completed`
  - `graph.active_topology=Hybrid`
  - `graph.branch_count=4`
  - `graph.node_count=4`
  - `result_preview.proof_outcome=graph_formed_and_completed`
  - `result_preview.payload_location=task.result`

Runtime log evidence:

- `12:20:10.773937Z` first worker step executing
- `12:20:10.773978Z` second worker step executing in parallel
- `12:20:10.895189Z` third worker step executing
- `12:20:11.011756Z` join step executing
- `12:20:11.139980Z` workflow completed

### Run 2: Earlier Failure Prompt No Longer Fails

This rerun reused the earlier session-continuation incident prompt that previously exercised the
packet-015 failure-visible gap.

- Same session: `9d86d915-5d89-42d7-ba65-7f020a5b12ed`
- Turn 2 workflow id: `ab410679-7da7-4c11-9c16-db81af6c5051`

Observed result:

- task surface:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - `execution_plan.steps=4`
  - `step_results=4`
- session surface:
  - `turn_count=2`
  - `last_assistant_result.assistant_result.proof_outcome=graph_formed_and_completed`
- workflow autonomy surface:
  - `graph.state=Completed`
  - `graph.active_topology=Hybrid`
  - `graph.branch_count=4`
  - `graph.node_count=4`

Interpretation:

- the old continuation prompt is no longer an honest failure trigger on this head
- that is improvement, not a regression

### Run 3: Failure-Visible Case Reproduced And Preserved

To verify the bounded `MS-95` fix honestly, a stricter joiner-shaped workload was submitted.

- Session id: `0ace8398-6066-47b4-8cc2-f09650aba4a9`
- Workflow id: `bae0a79c-5206-4d1a-a334-3b651e9a9b7e`
- Prompt shape: explicit multi-agent joiner-style incident analysis

Observed result:

- task surface:
  - `status=failed`
  - `proof_outcome=failed_before_graph`
  - `execution_plan.steps=0`
  - `step_results=0`
  - `aggregated_result.error=planner execution failed: Ask operation timed out`
- session surface:
  - `turn_count=1`
  - `last_assistant_result.status=failed`
  - `last_assistant_result.assistant_result.proof_outcome=failed_before_graph`
  - `last_assistant_result.preview=planner execution failed: Ask operation timed out`
  - provenance source fields:
    - `metadata.final_result`
    - `metadata.aggregated_result`
- workflow autonomy surface:
  - `graph.state=Failed`
  - `graph.active_topology=Sequential`
  - `graph.branch_count=0`
  - `graph.node_count=0`
  - `result_preview.proof_outcome=failed_before_graph`
  - `result_preview.payload_location=task.result`
  - provenance includes:
    - `workflow failed before usable graph formation`
    - `canonical result stored in metadata.final_result`
    - `aggregated payload nested under metadata.aggregated_result`
    - `session assistant_result derives from the canonical result object`

Runtime log evidence:

- `12:24:41.001138Z` planner actor stop timed out
- `12:24:41.003812Z` workflow failed
- error: `planner execution failed: Ask operation timed out`

Conclusion: this is the live proof that `MS-95` closed the bounded gap.

### Run 4: Trivial Prompts No Longer Produce Collapse

Two trivial prompts were used to try to re-exercise `collapsed_to_sequential`:

- `Reply with exactly READY.`
  - session `7cc9e69e-5451-40aa-b4dc-9e91fa41bfc3`
  - workflow `a605d310-fc3e-46e7-8d86-041566737e94`
- `Hi.`
  - session `c522dac3-0306-4ab9-949c-974efb994adf`
  - workflow `15f2febc-ae16-430a-957b-acec5ca450ba`

Observed result for both:

- `status=completed`
- `proof_outcome=graph_formed_and_completed`
- small multi-step plans instead of one-step sequential plans

Examples:

- `Reply with exactly READY.` produced a 3-step single-branch plan:
  - validate response target
  - check contract constraints
  - synthesize final handoff
- `Hi.` produced a 3-step hybrid plan:
  - draft greeting
  - check tone
  - merge and send reply

Interpretation:

- the fresh live path still supports success and failure-visible classification
- the fresh live path did **not** reproduce `collapsed_to_sequential`
- that may reflect planner behavior drift, prompt sensitivity drift, or a follow-up change in how
  trivial prompts are decomposed

## Evaluation Result

What this rerun proves:

- `MS-95` did close the packet-015 failure-visible parity gap on the supported autonomy surface
- task, session, and workflow-level autonomy status now agree on `failed_before_graph`
- the canonical result/provenance mapping is preserved through `metadata.final_result`,
  `metadata.aggregated_result`, `task.result`, and retained `assistant_result`

What changed relative to the earlier March 20 evaluation:

- the old failure prompt no longer fails
- the old trivial collapse probes no longer collapse
- the active autonomy HTTP contract is workflow-id based, not session-turn based

## Remaining Limits

- This rerun did not reproduce `collapsed_to_sequential` on any fresh live prompt. That outcome is
  still required by packet 015, but it was not naturally reachable through the old trivial probes on
  this head.
- The earlier March 20 note used a session-turn autonomy path that is not the active HTTP contract
  on this merged head. Future live proof notes should use the workflow-id autonomy route instead.
- This rerun stayed on the direct `openai_chatgpt` / `gpt-5.4` path and did not exercise
  provider-routing, cross-host, or external-agent scenarios.

## Cleanup

Runtime was shut down after the evaluation and the temporary database was dropped.
At handoff, the worktree delta consists of this note plus the new re-evaluation artifact bundle.

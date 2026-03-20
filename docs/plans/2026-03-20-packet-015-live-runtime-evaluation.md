# 2026-03-20 Packet 015 Live Runtime Evaluation

## Summary

Packet 015 was re-evaluated on `main` after PR #224 merged at `f7e04396c69a1dcc87ab2dc514cb91dc6635b074`.
This run rechecked the narrow deterministic validation bundle and then exercised the live runtime
through a three-case proof matrix:

- `graph_formed_and_completed`
- `collapsed_to_sequential`
- `failed_before_graph`

The packet is materially working on the default live path. Task status, retained session state, and
operator autonomy previews now carry a shared proof-outcome/result-projection contract for the
success and collapse cases, and task plus session surfaces retain the same contract for the
failure-visible case. The remaining gap is that `failed_before_graph` still does not materialize an
autonomy status document; the task and session surfaces preserve the failure result, but the
autonomy endpoint returns `404`.

Artifacts:
`docs/plans/artifacts/2026-03-20-packet-015-live-runtime-evaluation/`

## Baseline

- Repo state at start: clean `main`, synced to `origin/main`, no open PRs.
- Runtime path under test: `mister-smith run`
- Provider/model: `openai_chatgpt` / `gpt-5.4`
- Transport/runtime mode observed on live runs:
  - `execution_boundary=tool_bus`
  - `workflow_runner=tokio_task`
  - `planner_lifecycle=supervised_actor`
  - `executor_lifecycle=supervised_actor`
- Temporary database: `mistersmith_packet015_live_eval_20260320`
- HTTP port: `63120`

## Deterministic Validation

Re-ran the packet 015 local bundle before the live proof:

```bash
cargo test -p mister-smith-agents
cargo test -p mister-smith-events
cargo test -p mister-smith-app
cargo build --workspace
```

All four commands passed.

Notable coverage in the passing bundle:

- `gate10_completed_graph_status_includes_result_preview`
- `gate10_collapsed_graph_status_includes_result_preview`
- `gate10_failed_graph_status_includes_result_preview_for_partial_graph_evidence`
- `proof_outcome_classification_freezes_the_three_packet_labels`
- `operator_result_preview_roundtrips_with_shared_contract_fields`
- `terminal_result_views_preserve_proof_outcome_across_task_and_final_results`
- `retained_result_for_turn_uses_stored_projection_with_proof_outcome`

## Live Proof Matrix

### Case 1: Graph Formed And Completed

- Session id: `e6b8d64e-ad95-4494-8b19-ba475fbb65f4`
- Workflow id: `87bb38c1-73ca-4151-90c6-d7d981613faf`
- Prompt shape: three parallel worker tracks plus a join memo
- Task status:
  - `status=completed`
  - `proof_outcome=graph_formed_and_completed`
  - `execution_plan.steps=4`
  - `step_results=4`
- Autonomy status:
  - `graph.state=Completed`
  - `graph.active_topology=Hybrid`
  - `graph.branch_count=3`
  - `graph.node_count=4`
  - `topology.parallelism_width=3`
  - `result_preview.payload_location=task.result`
- Session inspect:
  - `turn_count=1`
  - `last_completed_workflow_id=87bb38c1-73ca-4151-90c6-d7d981613faf`
  - `last_assistant_result.assistant_result.proof_outcome=graph_formed_and_completed`
  - provenance source fields:
    - `metadata.final_result`
    - `metadata.aggregated_result`

Runtime log evidence:

- `05:53:21.826068Z` first worker step executing
- `05:53:21.826138Z` second worker step executing in parallel
- `05:53:21.944085Z` third worker step executing
- `05:53:22.058216Z` join step executing
- `05:53:22.184176Z` workflow completed

Conclusion: the live path now proves a real multi-step graph and projects the canonical result on
task, session, and autonomy operator surfaces.

### Case 2: Collapsed To Sequential

- Session id: `eb90a1d6-5beb-4ca5-81c7-036a69403b80`
- Workflow id: `e1eec258-a818-470b-8115-04d2f86dbf86`
- Prompt shape: minimal request, `Reply with exactly READY.`
- Task status:
  - `status=completed`
  - `proof_outcome=collapsed_to_sequential`
  - `execution_plan.steps=1`
  - `step_results=1`
- Autonomy status:
  - `graph.state=Completed`
  - `graph.active_topology=Sequential`
  - `graph.branch_count=1`
  - `graph.node_count=1`
  - `topology.parallelism_width=1`
  - provenance explicitly states `planner emitted one sequential step`
- Session inspect:
  - `turn_count=1`
  - `last_completed_workflow_id=e1eec258-a818-470b-8115-04d2f86dbf86`
  - `last_assistant_result.assistant_result.proof_outcome=collapsed_to_sequential`
  - provenance source fields:
    - `metadata.final_result`
    - `metadata.aggregated_result`

Runtime log evidence:

- `05:55:39.016749Z` sequential step executing
- `05:55:39.132936Z` workflow completed

Conclusion: the live path now surfaces an honest collapse outcome rather than pretending a
multi-agent graph happened.

### Case 3: Failed Before Graph

- Session id: `e6b8d64e-ad95-4494-8b19-ba475fbb65f4` (continued session)
- Workflow id: `71f3ee66-3f55-4561-a484-c4d00a97b233`
- Prompt shape: incident analysis that requested three parallel tracks plus a join memo
- Task status:
  - `status=failed`
  - `proof_outcome=failed_before_graph`
  - `execution_plan.steps=4`
  - `step_results=0`
  - `aggregated_result.error=execution graph compile failed: Unsupported topology contract: unsupported planner role 'joiner'`
- Session inspect after failure:
  - `turn_count=2`
  - `last_completed_workflow_id=71f3ee66-3f55-4561-a484-c4d00a97b233`
  - `last_assistant_result.status=failed`
  - `last_assistant_result.assistant_result.proof_outcome=failed_before_graph`
  - preview contains the compile error string
  - provenance source fields:
    - `metadata.final_result`
    - `metadata.aggregated_result`
- Autonomy status:
  - `GET /api/v1/autonomy/sessions/.../turns/2/status` returned no document
  - captured body: `{"error":"no autonomy status found for workflow 71f3ee66-3f55-4561-a484-c4d00a97b233"}`

Runtime log evidence:

- `05:54:34.627439Z` `Workflow run failed`
- error text matches task/session retained result:
  `execution graph compile failed: Unsupported topology contract: unsupported planner role 'joiner'`

Conclusion: packet 015 now preserves the failed-before-graph result on task and session surfaces,
but the autonomy operator surface still has a visibility gap for that case.

## Evaluation Result

Packet 015 is closed honestly for the core result-surface contract it set out to land:

- success and collapse cases now carry consistent proof-outcome classification
- task status exposes the canonical result contract with proof outcome and runtime mode
- session inspection retains assistant results with proof outcome and provenance
- autonomy inspection exposes bounded result preview plus provenance for completed graph and
  sequential-collapse outcomes

The packet is not a full operator-proof closure for failure-visible autonomy parity yet. The live
runtime still lacks an autonomy status document when graph compilation fails before publication.

## Remaining Limits

- The stored previews are still compact payload-oriented previews, not a clean operator-written memo
  string. The final result is inspectable, but still closer to structured payload evidence than a
  polished human-facing answer surface.
- `failed_before_graph` preserves task and session results but does not yet project a corresponding
  autonomy status document.
- This evaluation stayed on the direct `openai_chatgpt` / `gpt-5.4` path and did not attempt any
  provider-routing, cross-host, or expanded external-agent scenarios.

## Cleanup

Runtime was shut down after the evaluation and the temporary database was dropped.
The artifact bundle remains under `docs/plans/artifacts/2026-03-20-packet-015-live-runtime-evaluation/`.
At handoff, the only worktree delta was this new evaluation note.

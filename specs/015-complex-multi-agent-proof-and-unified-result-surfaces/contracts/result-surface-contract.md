# Contract: Result Surface Contract

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Unify the existing result forms already present on `main` so future runtime proof, task status,
session retention, and operator inspection all depend on one canonical result contract.

This packet does **not** invent a new result subsystem. It freezes the relationship between result
forms that already exist and extends current surfaces with bounded preview and provenance.

## Canonical Mapping

The contract for this packet is:

- metadata `final_result` is the authoritative persisted runtime result object
- metadata `aggregated_result` remains the execution-produced payload nested inside `final_result`
- `task.result` is the task-facing envelope that mirrors or derives directly from `final_result`
- session `assistant_result` and `last_assistant_result` are retained projections derived from
  `final_result`
- operator result preview and provenance are compact projections derived from `final_result`

No other result form may become a competing top-level contract in this packet.

## Canonical Result Shape

Example authoritative payload shape:

```json
{
  "workflow_id": "11111111-1111-1111-1111-111111111111",
  "provider_kind": "openai_chatgpt",
  "model_id": "gpt-5.4",
  "description": "Analyze an incident packet",
  "runtime_execution_mode": {
    "workflow_runner": "tokio_task",
    "planner_lifecycle": "supervised_actor",
    "executor_lifecycle": "supervised_actor",
    "execution_boundary": "tool_bus",
    "tool_name": "workflow.execute_step"
  },
  "planner_output": {},
  "execution_plan": {},
  "step_results": [],
  "aggregated_result": {
    "summary": "bounded final payload"
  },
  "proof_outcome": "graph_formed_and_completed"
}
```

Behavior:

- `aggregated_result` must stay nested inside the canonical result object
- task, session, and operator surfaces may project or compact the result, but they must be able to
  trace back to the canonical result object

## Task Surface Contract

`task.result` remains the task-facing result envelope.

Expected behavior:

- terminal task inspection returns the canonical result object or a lossless direct wrapper around
  it
- task surfaces must expose enough provenance to verify provider and execution mode markers
- the task surface is the authoritative place to recover the full canonical result without reading
  raw logs

## Session Surface Contract

Session retention keeps a bounded projection of the canonical result object.

Expected behavior:

- `assistant_result` and `last_assistant_result` derive from the canonical result object
- retained session storage and session inspect surfaces serialize the full retained-result
  projection at `assistant_result` / `last_assistant_result`, not only the inner assistant payload
- retained session views must preserve result preview plus enough provenance to correlate with the
  owning workflow
- session inspection must not silently drop assistant-result material once it was retained

Example retained projection:

```json
{
  "workflow_id": "11111111-1111-1111-1111-111111111111",
  "status": "completed",
  "assistant_result": {
    "preview": "bounded answer preview",
    "aggregated_result": {
      "summary": "bounded final payload"
    }
  },
  "provenance": {
    "source_fields": [
      "metadata.final_result",
      "metadata.aggregated_result"
    ]
  }
}
```

## Operator Surface Contract

Operator status remains a bounded summary surface, not a raw payload dump.

Expected behavior:

- autonomy inspection renders a result preview and provenance block correlated with graph state,
  topology, and proof outcome classification
- the typed event/status surface carries this block as
  `AutonomyStatusView.result_preview: Option<OperatorResultPreview>`
- operators can distinguish:
  - `graph_formed_and_completed`
  - `collapsed_to_sequential`
  - `failed_before_graph`
- the operator surface points back to the canonical task result location when deeper inspection is
  needed

Example operator projection:

```json
{
  "workflow_id": "11111111-1111-1111-1111-111111111111",
  "proof_outcome": "collapsed_to_sequential",
  "preview_text": "completed with one sequential step",
  "payload_location": "task.result",
  "provenance_lines": [
    "planner emitted one sequential step",
    "canonical result stored in metadata.final_result",
    "aggregated payload nested under metadata.aggregated_result"
  ]
}
```

## Proof Outcome Taxonomy

This packet uses exactly three proof outcome classes:

- `graph_formed_and_completed`
- `collapsed_to_sequential`
- `failed_before_graph`

These classes must be stable across:

- task-facing result inspection
- session-facing retained result views
- operator preview and provenance output
- evaluation artifacts under `docs/plans/`

## Relationship To Existing Surfaces

The following existing surfaces remain authoritative baseline:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- the existing bounded post-`MS-77` MCP capability surface

This packet only extends them with:

- one explicit result contract
- one proof-outcome taxonomy
- bounded result preview and provenance projections

## External-Agent Non-Regression Rule

Broader external-agent work is deferred.

Only add an MCP or external-agent non-regression check when:

- the new result-surface fields intersect the existing bounded discovery or delegation path
- or the operator result projection touches the already-landed external capability catalog surface

If there is no such intersection, do not widen this packet into external-agent work.

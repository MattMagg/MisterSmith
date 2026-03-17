# Contract: Adaptive Orchestration Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Extend the current topology- and autonomy-status contract so operators can inspect adaptive team
size decisions without introducing a second operator subsystem or a new runtime path.

This packet does **not** define a new session or workflow API. It extends the existing
workflow-scoped orchestration and autonomy surfaces.

## Internal Decision Contract

The runtime must materialize one operator-visible team-sizing decision before adaptive branch
dispatch begins.

Example payload shape:

```json
{
  "workflow_id": "11111111-1111-1111-1111-111111111111",
  "graph_id": "22222222-2222-2222-2222-222222222222",
  "decision_phase": "initial",
  "desired_workers": 4,
  "selected_workers": 3,
  "available_workers": 3,
  "branch_frontier_width": 4,
  "dependency_depth": 2,
  "conservative_mode": false,
  "budget_pressure": 38,
  "cap_reason": "available worker pool smaller than structural width",
  "rationale_lines": [
    "task shape parallel-fanout with frontier width 4",
    "dependency depth 2 keeps coordination cost acceptable",
    "selected 3 workers because only 3 workers are currently available"
  ]
}
```

Behavior:

- decision is emitted once before the first adaptive branch-routing pass
- later `frontier_rebalance` decisions may be emitted only at bounded routing transitions
- any cap reason must be operator-visible when `selected_workers < desired_workers`

## Autonomy Status Extension

`AutonomyStatusView` remains the main operator surface.

This packet adds a `team_sizing` section conceptually shaped like:

```json
{
  "graph": {
    "workflow_id": "11111111-1111-1111-1111-111111111111",
    "state": "running",
    "active_topology": "parallel"
  },
  "topology": {
    "topology_kind": "parallel",
    "parallelism_width": 4,
    "task_shape": {
      "kind": "parallel_fanout"
    }
  },
  "team_sizing": {
    "desired_workers": 4,
    "selected_workers": 3,
    "available_workers": 3,
    "cap_reason": "available worker pool smaller than structural width",
    "rationale_lines": [
      "task shape parallel-fanout with frontier width 4",
      "selected 3 workers because only 3 workers are currently available"
    ]
  },
  "routing_history": [
    {
      "selected_agent": "33333333-3333-3333-3333-333333333333",
      "dependency_depth": 2
    }
  ]
}
```

Behavior:

- the operator must be able to inspect desired versus selected worker width
- task shape, topology, team sizing, and routing rationale must remain correlated inside one
  workflow status view
- no new endpoint is required if the existing HTTP and CLI autonomy surfaces can render the new
  fields

## Evaluation Harness Contract

The packet requires a repeatable comparison artifact, not just local console output.

The implementation must produce one dated evidence note under `docs/plans/` containing:

- workload class name
- baseline mode and team size
- adaptive mode and selected team size
- timing or step-count comparison
- outcome (`improved`, `matched`, or `regressed`)
- short explanation of why the result matters

Example record shape:

```json
{
  "workload_class": "parallel-fanout",
  "baseline_mode": "sequential",
  "adaptive_mode": "adaptive-team-sizing",
  "baseline_team_size": 1,
  "adaptive_team_size": 3,
  "baseline_duration_ms": 1800,
  "adaptive_duration_ms": 900,
  "outcome": "improved"
}
```

## Relationship To Existing Surfaces

The following existing surfaces remain authoritative:

- `TopologyCompiler` and existing graph compilation in `crates/mister-smith-agents/src/topology.rs`
- workflow-scoped autonomy status in `crates/mister-smith-events/src/autonomy.rs`
- operator rendering in `crates/mister-smith-app/src/autonomy.rs`
- the existing runtime-backed task path

The packet only extends those surfaces with:

- a frozen team-sizing decision contract
- a correlated autonomy-status projection of that decision
- a deterministic evaluation artifact

## Parallel Symphony Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already landed
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- the active `docs/plans/...` evaluation artifact
- the active `## Codex Workpad` comment for the parent issue

Only one Symphony run may own a choke-point file at a time.

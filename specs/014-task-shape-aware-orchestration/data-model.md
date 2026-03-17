# Data Model: Task-Shape-Aware Orchestration and Dynamic Team Sizing

**Date**: 2026-03-16  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `crates/mister-smith-agents/src/topology.rs` | Existing task-shape and topology plan fields remain the foundation of the packet. |
| `crates/mister-smith-agents/src/orchestrator.rs` | Team-sizing decisions must fit the current routing and autonomy-status assembly seam. |
| `crates/mister-smith-agents/src/team.rs` | Team membership exists today and needs an explicit adaptive sizing decision. |
| `crates/mister-smith-agents/src/scheduler.rs` | Worker-load tracking is the runtime boundary that adaptive team sizing should reuse. |
| `crates/mister-smith-events/src/autonomy.rs` | Operator-visible summaries must carry task shape, topology, and team-size rationale together. |
| `docs/plans/2026-03-16-frontier-direction.md` | Requires operator-visible rationale plus validation evidence for adaptive behavior. |

## Entities

### TaskShapeAssessment

Durable or reconstructible summary of the graph structure that informs adaptive execution.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Workflow that owns the assessment |
| graph_id | `ExecutionGraphId` | Required | Graph analyzed for task shape |
| task_shape | `TaskShapeClassification` | Required | Existing dependency-shape classification |
| branch_frontier_width | `usize` | Required | Width of the ready frontier used for sizing |
| dependency_depth | `usize` | Required | Deepest dependency depth considered for the decision |
| active_topology | `TopologyKind` | Required | Selected topology for this graph |
| rationale | `TopologyRationale` | Required | Structured rationale already emitted by topology selection |
| emitted_at | `DateTime<Utc>` | Required | When the assessment became operator-visible |

**Invariant**: one assessment belongs to one workflow graph and is consistent with the selected
topology.

---

### TeamSizingDecision

Operator-visible sizing decision for one workflow or frontier transition.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| workflow_id | `TaskId` | Required | Workflow that owns the decision |
| graph_id | `ExecutionGraphId` | Required | Graph the decision applies to |
| decision_phase | `String` | Required | `initial` or `frontier_rebalance` in this slice |
| desired_workers | `usize` | Required, `>= 1` | Width implied by structure before caps |
| selected_workers | `usize` | Required, `>= 1` | Final worker count after caps |
| available_workers | `usize` | Required | Workers available to the runtime at decision time |
| branch_frontier_width | `usize` | Required | Frontier width that shaped the decision |
| dependency_depth | `usize` | Required | Depth signal that shaped the decision |
| conservative_mode | `bool` | Required | Whether the runtime narrowed posture conservatively |
| budget_pressure | `Option<u8>` | Optional | Budget-pressure signal used for capping when present |
| cap_reason | `Option<String>` | Optional | Main explanation when `selected_workers < desired_workers` |
| rationale_lines | `Vec<String>` | Required | Operator-visible explanation of the decision |
| decided_at | `DateTime<Utc>` | Required | Decision timestamp |

**Invariant**: `selected_workers <= desired_workers`.

**Invariant**: `selected_workers <= available_workers` whenever `available_workers > 0`.

---

### AdaptiveTeamPlan

Runtime-facing team membership assembled from a team-sizing decision.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| team_id | `Uuid` | Required | Team instance for the workflow |
| workflow_id | `TaskId` | Required | Owning workflow |
| coordinator_id | `AgentId` | Required | Coordinator that owns the team |
| supervisor_id | `Option<AgentId>` | Optional | Supervisor if the pattern uses one |
| worker_ids | `Vec<AgentId>` | Required | Selected active workers for the current frontier |
| sizing_decision | `TeamSizingDecision` | Required | Decision that justified the membership width |
| topology_kind | `TopologyKind` | Required | Topology active for the team plan |

**Invariant**: `worker_ids.len() == selected_workers` from the attached sizing decision.

---

### AdaptiveDecisionView

Operator-facing projection that joins graph, topology, and adaptive-team decisions.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| assessment | `TaskShapeAssessment` | Required | Structural view of the workload |
| team_sizing | `TeamSizingDecision` | Required | Final adaptive sizing decision |
| routing_history | `Vec<RoutingDecisionSummary>` | Required | Branch routing history for the workflow |
| conservative_reasons | `Vec<String>` | Required | Reasons the runtime narrowed autonomy |
| updated_at | `DateTime<Utc>` | Required | Last projection update time |

**Invariant**: the projection references one workflow and one current graph.

---

### EvaluationHarnessRun

Durable evidence record for one comparison between adaptive sizing and a baseline.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| run_id | `Uuid` | Required | Stable comparison identifier |
| workload_class | `String` | Required | Human-readable class such as `parallel-fanout` or `strict-chain` |
| baseline_mode | `String` | Required | Fixed or sequential comparison mode |
| adaptive_mode | `String` | Required | Adaptive sizing comparison mode |
| baseline_team_size | `usize` | Required | Worker count used in the baseline |
| adaptive_team_size | `usize` | Required | Worker count chosen by adaptive logic |
| baseline_duration_ms | `u64` | Required | Measured or simulated baseline duration |
| adaptive_duration_ms | `u64` | Required | Measured or simulated adaptive duration |
| outcome | `String` | Required | `improved`, `matched`, or `regressed` |
| evidence_note_path | `String` | Required | Repo path for the durable evidence note under `docs/plans/` |

**Invariant**: the evidence note path is stable and points to one repo artifact for later review.

## Relationships

```text
TaskShapeAssessment 1 --- 1 TeamSizingDecision
TeamSizingDecision 1 --- 1 AdaptiveTeamPlan
AdaptiveTeamPlan 1 --- N RoutingDecisionSummary
AdaptiveDecisionView aggregates TaskShapeAssessment + TeamSizingDecision + routing history
EvaluationHarnessRun records a durable comparison for one workload class
```

## Lifecycle Rules

### Adaptive sizing lifecycle

`task-shape assessment` -> `initial team-sizing decision` -> `adaptive team plan` ->
`frontier rebalance decision` (optional) -> `terminal evidence`

Notes:

- the first slice requires an initial sizing decision and may allow a frontier rebalance decision
  only at bounded routing transitions
- completed branches are not reopened solely because a later sizing decision narrows or widens the
  active team

### Evaluation lifecycle

`define workload fixture` -> `run baseline` -> `run adaptive mode` -> `compare` -> `write durable evidence`

Notes:

- the harness may be deterministic and fixture-driven
- the durable evidence note is required even when the adaptive path does not outperform the
  baseline

## Identifier Guarantees

- `workflow_id` remains the canonical workflow identifier for the adaptive decision view
- `graph_id` remains the canonical topology and task-shape identifier
- one `TeamSizingDecision` may be recorded for the initial workflow frontier and for later bounded
  frontier rebalance points
- `run_id` identifies one evaluation comparison and should be stable in the durable evidence note

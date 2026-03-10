# Data Model: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

**Date**: 2026-03-10
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `docs/research-output/consolidated/02-orchestration-and-self-organization.md` | Drives `ExecutionGraph`, `TopologyPlan`, and branch topology selection |
| `docs/research-output/consolidated/03-supervision-and-resilience.md` | Drives `GuardDecision`, `ProfileSnapshot`, and failure classification |
| `docs/research-output/consolidated/07-memory-and-context.md` | Drives `MemoryFragment`, `ContextBudget`, and `MemorySnapshot` |
| `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` | Drives `DelegationCapability`, `ProvenanceChain`, and operator-facing autonomy invariants |
| `spec/data-management/agent-orchestration.md` | Constrains role boundaries and context-management behavior |
| `spec/data-management/message-schemas.md` | Constrains workflow coordination and event/routing subject boundaries |
| `spec/core-architecture/supervision-trees.md` | Constrains restart/isolation boundaries for branch execution |

## Entities

### ExecutionGraph

Canonical representation of a workflow after planner output is normalized for execution.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| graph_id | `ExecutionGraphId` | Required | Stable graph identifier |
| workflow_id | `TaskId` or equivalent | Required | Parent workflow / task identifier |
| branches | `Vec<ExecutionBranch>` | Non-empty | Checkpointable execution groups within the workflow |
| nodes | `Vec<ExecutionNode>` | Non-empty | Executable nodes in the workflow |
| edges | `Vec<ExecutionEdge>` | Required | Directed dependency edges |
| topology_plan | `TopologyPlan` | Required | Selected execution shape and rationale |
| state | `GraphState` | Required | Lifecycle status for the whole graph |
| checkpoint_lineage | `Vec<BranchCheckpoint>` | Optional | Branch-level checkpoint history |

**Invariant**: An `ExecutionGraph` must be acyclic before dispatch.

**Invariant**: Topology selection happens against the fully validated graph, not raw planner text.

---

### ExecutionNode

Executable unit inside an `ExecutionGraph`.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| node_id | `ExecutionNodeId` | Required | Stable node identifier |
| role | `AgentType` | Required | Requested execution role |
| branch_id | `ExecutionBranchId` | Required | Branch the node belongs to |
| dependencies | `Vec<ExecutionNodeId>` | Optional | Direct upstream dependencies |
| state | `NodeState` | Required | Pending, running, checkpointed, completed, failed, etc. |
| budget | `ContextBudget` | Required | Context allowance for this node |
| delegation_requirement | `Option<DelegationScope>` | Optional | Required authority scope, if privileged |

**Invariant**: A node cannot enter `Running` until all dependencies are satisfied.

---

### ExecutionEdge

Directed dependency between nodes.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| from | `ExecutionNodeId` | Required | Upstream node |
| to | `ExecutionNodeId` | Required | Downstream node |
| edge_type | `DependencyType` | Required | Completion, data, checkpoint, or policy dependency |

**Invariant**: `Completion` dependencies gate dispatch. `Data` dependencies gate context assembly.

---

### TopologyPlan

Chosen execution shape for the graph.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| topology_kind | `TopologyKind` | Required | Sequential, parallel, pipeline, hierarchical, hybrid |
| parallelism_width | `usize` | >= 1 | Maximum concurrent execution width |
| rationale | `TopologyRationale` | Required | Why this topology was selected |
| coordination_policy | `CoordinationPolicy` | Required | Synchronization/aggregation behavior |
| fallback_topology | `Option<TopologyKind>` | Optional | Conservative fallback if signals degrade |

**Invariant**: A `hybrid` topology must declare its branch-specific coordination policy explicitly.

---

### ExecutionBranch

Checkpointable unit of work inside an `ExecutionGraph`.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| branch_id | `ExecutionBranchId` | Required | Stable branch identifier |
| graph_id | `ExecutionGraphId` | Required | Owning execution graph |
| node_ids | `Vec<ExecutionNodeId>` | Non-empty | Nodes assigned to this branch |
| state | `BranchState` | Required | Pending, running, checkpointed, isolated, completed, failed |
| checkpoint_policy | `CheckpointPolicy` | Required | When checkpoints are captured for this branch |
| assigned_agents | `Vec<AgentId>` | Optional | Agents currently executing or recovering the branch |
| recovery_strategy | `RecoveryStrategy` | Required | Resume, reassign, isolate, or escalate behavior |

**Invariant**: A branch must remain independently resumable without re-running nodes recorded as
completed in its latest checkpoint.

**Invariant**: Branch-level reassignment must preserve the same `branch_id` so checkpoints,
provenance, and operator inspection remain stable.

---

### BranchCheckpoint

Checkpoint for a branch or subgraph.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| checkpoint_id | `CheckpointId` | Required | Stable checkpoint identifier |
| branch_id | `ExecutionBranchId` | Required | Branch being checkpointed |
| completed_nodes | `Vec<ExecutionNodeId>` | Required | Nodes safely completed at checkpoint time |
| pending_nodes | `Vec<ExecutionNodeId>` | Required | Remaining work from this point |
| memory_snapshot_id | `MemorySnapshotId` | Required | Associated context snapshot |
| failure_context | `Option<FailureContext>` | Optional | Last-known failure/intervention evidence |
| created_at | `DateTime<Utc>` | Required | Timestamp |

**Invariant**: Resuming from a `BranchCheckpoint` must not re-run nodes recorded in
`completed_nodes`.

---

### ContextBudget

Bounded context allowance for a role, node, or branch.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| budget_id | `ContextBudgetId` | Required | Stable identifier |
| scope | `BudgetScope` | Required | Role, node, branch, or workflow |
| max_units | `u64` | Required | Maximum context allowance |
| reserved_units | `u64` | Default 0 | Currently reserved context usage |
| policy | `BudgetPolicy` | Required | Evict, summarize, consolidate, or reject |

**Invariant**: Delivered context must not exceed `max_units`; policy determines what happens when
requested context is larger.

---

### MemoryFragment

Managed unit of stored context.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| fragment_id | `MemoryFragmentId` | Required | Stable fragment identifier |
| content | `serde_json::Value` or equivalent | Required | Stored context payload |
| provenance | `FragmentProvenance` | Required | Source role/branch/tool lineage |
| freshness | `FreshnessPolicy` | Required | TTL/recency metadata |
| access_policy | `AccessPolicy` | Required | Role or branch visibility limits |
| version | `u64` | Required | Monotonic version |
| fragment_class | `FragmentClass` | Required | Working, episodic, summary, checkpoint, audit |

**Invariant**: `MemoryFragment` metadata is mandatory; raw context without provenance/policy is not
valid managed memory.

---

### MemorySnapshot

Checkpoint-ready reconstruction of an agent or branch context.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| snapshot_id | `MemorySnapshotId` | Required | Stable snapshot identifier |
| target_scope | `SnapshotScope` | Required | Agent, node, branch, or workflow |
| fragment_ids | `Vec<MemoryFragmentId>` | Required | Included fragments |
| summary | `Option<MemorySummary>` | Optional | Consolidated summary of older fragments |
| created_at | `DateTime<Utc>` | Required | Timestamp |

**Invariant**: A snapshot must be reconstructable without replaying the entire raw workflow history.

---

### ProfileSnapshot

Telemetry/performance state used by routing and supervision.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| profile_id | `ProfileSnapshotId` | Required | Stable identifier |
| target | `ProfileTarget` | Required | Agent, branch, topology, or provider |
| health_state | `HealthState` | Required | Current operational health |
| latency_window | `MetricWindow` | Optional | Recent latency measurements |
| error_window | `MetricWindow` | Optional | Recent failure/error measurements |
| semantic_signals | `Vec<SemanticSignal>` | Optional | Step-level degradation signals |
| updated_at | `DateTime<Utc>` | Required | Timestamp |

**Invariant**: Missing profile data is allowed, but consumers must fall back to conservative policy.

---

### GuardDecision

Supervisory decision produced by the Guard/Advisor layer.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| decision_id | `GuardDecisionId` | Required | Stable identifier |
| failure_class | `FailureClass` | Required | Transient, structural, streaming, semantic |
| intervention | `InterventionType` | Required | Retry, failover, context refresh, isolate, escalate, etc. |
| evidence | `GuardEvidence` | Required | Signals supporting the decision |
| target_scope | `GuardTarget` | Required | Node, branch, graph, or provider |
| operator_visibility | `bool` | Required | Whether surfaced immediately to operators |

**Invariant**: Every non-trivial intervention must emit a corresponding `InterventionRecord`.

---

### InterventionRecord

Operator-facing audit record of a supervisory action.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| record_id | `InterventionRecordId` | Required | Stable identifier |
| decision_id | `GuardDecisionId` | Required | Source Guard decision |
| before_state | `serde_json::Value` | Required | Relevant pre-intervention state |
| after_state | `serde_json::Value` | Optional | Post-intervention state, when known |
| rationale | `String` | Required | Human-readable explanation |
| emitted_at | `DateTime<Utc>` | Required | Timestamp |

---

### DelegationCapability

Bounded authority token/record for privileged autonomous work.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| capability_id | `CapabilityId` | Required | Stable capability identifier |
| issuer | `AgentId` or policy principal | Required | Source authority |
| recipient | `AgentId` | Required | Receiving actor |
| scope | `DelegationScope` | Required | What actions are allowed |
| expires_at | `DateTime<Utc>` | Required | Expiry time |
| parent_capability | `Option<CapabilityId>` | Optional | Parent in the provenance chain |
| revocation_state | `RevocationState` | Required | Active, revoked, expired |

**Invariant**: A capability cannot outlive its parent if one exists.

---

### ProvenanceChain

Linked record of authority transfers.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| root_issuer | `AgentId` or policy principal | Required | Origin of authority |
| links | `Vec<ProvenanceLink>` | Non-empty for delegated work | Ordered authority transfers |
| terminal_capability | `CapabilityId` | Required | Capability used at execution time |

**Invariant**: Cycles are invalid. Broken or revoked links invalidate downstream execution.

## Relationships

```text
ExecutionGraph 1──* ExecutionBranch
ExecutionGraph 1──* ExecutionNode
ExecutionGraph 1──* ExecutionEdge
ExecutionGraph 1──1 TopologyPlan
ExecutionBranch 1──* BranchCheckpoint
ExecutionNode 1──1 ContextBudget
ExecutionNode *──* MemoryFragment (via MemorySnapshot assembly)
BranchCheckpoint 1──1 MemorySnapshot
ProfileSnapshot ──informs──> TopologyPlan
ProfileSnapshot ──informs──> GuardDecision
GuardDecision 1──1 InterventionRecord
DelegationCapability *──1 ProvenanceChain
ExecutionNode 0..1──1 DelegationCapability
```

## State Transitions

### Graph / Branch Execution

```text
GraphCreated -> Validated -> Dispatched -> Running
Running -> Checkpointed -> Resumed
Running -> FailedBranch -> GuardDecision -> Resumed / Reassigned / Escalated
Running -> Completed
```

### Memory Lifecycle

```text
WorkingFragment -> EpisodicFragment -> SummaryFragment -> SnapshotIncluded
WorkingFragment -> Rejected (policy/validation failure)
```

### Delegation Lifecycle

```text
Issued -> Active -> Delegated -> Active
Active -> Revoked
Active -> Expired
Revoked/Expired -> InvalidForExecution
```

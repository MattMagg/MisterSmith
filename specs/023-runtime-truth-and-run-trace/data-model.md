# Data Model: Runtime Truth And Run Trace

## Core packet-023 entities

### `RuntimeTruthView`

- `trace_root_id`: canonical run-trace root for the workflow run
- `evidence_class`: strongest evidence class the run actually produced
- `proof_boundary`: explicit statement of what the run proved and did not prove
- `run_trace`: bounded summary of observed run-trace relationships
- `grounded_evidence_refs`: stable references to grounded work when such evidence exists

### `ProofBoundaryView`

- `execution_summary`: short statement of orchestration-substrate completion
- `semantic_status`: short statement of whether grounded task completion was proven
- `grounded_tool_execution`: short statement of grounded tool evidence strength
- `summary`: operator-facing one-line explanation of the current proof boundary

### `RunTraceSummaryView`

- `workflow_id`: canonical run anchor
- `trace_root_id`: stable run-trace root identifier
- `observed_relationships`: ordered unique list of observed relationship kinds
- `graph_id`: graph identifier when one exists
- `branch_ids`: branch identifiers when known
- `node_ids`: node identifiers when known

### `ExecutionEvidenceClass`

- `orchestration_substrate_completion`
- `placeholder_or_simulated_step_completion`
- `grounded_tool_execution`
- `grounded_task_proof`

The first slice only promotes the class when current runtime evidence actually justifies it.

### `GroundedEvidenceReference`

- `kind`: stable evidence class such as `file`, `endpoint`, `artifact`, or `checkpoint`
- `reference`: stable identifier, path, URL, or artifact key
- `label`: short human-readable explanation for the evidence

### `RunTraceRelationshipKind`

- `graph`
- `branch`
- `node`
- `tool_boundary`
- `handoff`
- `repair`
- `retry`
- `fanout`
- `join`
- `supervision`

## Projection targets

### `TaskResultView`

- adds `runtime_truth`
- keeps `supervision_evidence` unchanged and separate

### `SessionRetainedResultView`

- adds `runtime_truth`
- keeps the retained result preview and provenance model

### `OperatorResultPreview`

- adds `runtime_truth`
- keeps `orchestration_quality` unchanged

### `AutonomyStatusView`

- adds `runtime_truth`
- keeps `supervision_evidence` unchanged and separate

## Invariants

- packet `021` `supervision_evidence` remains predictive-supervision data, not the packet-023
  runtime-truth contract
- packet `022` remains the owner of durable lifecycle, event-history, compaction, and effect
  boundary semantics
- placeholder `workflow.execute_step` completion is not enough to classify a run as grounded task
  proof
- packet `019` and packet `020` remain the last fresh live baseline; packet `021`, packet `022`,
  and packet `023` remain deterministic-only unless a fresh live rerun is captured
- transport schema stays unchanged in the first slice
- all rendered proof-boundary views should tell the same bounded truth story across task, session,
  autonomy, and operator surfaces

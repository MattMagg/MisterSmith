# Data Model: Runtime Truth And Run Trace

## Core trace entities

### `RunTraceRecord`

- `workflow_id`: canonical workflow identifier and primary run anchor
- `trace_root_id`: canonical trace-root identifier for the run
- `proof_boundary`: latest bounded proof statement for the run
- `evidence_class`: current class of execution evidence for the run
- `surface_projection_targets`: task, session, autonomy, and operator run-detail surfaces that
  should render the same truth story
- `source_refs`: packet notes, artifact bundles, or future runtime references used to justify the
  rendered truth story

### `TraceRoot`

- represents the top-level trace identity for one workflow run
- may reuse packet `022` durable identifiers later, but does not define packet `022` ownership
- remains distinct from any external tracing vendor or export identifier

### `TraceEvent`

- typed event in the run trace
- expected event classes include:
  - graph formation
  - branch execution
  - node execution
  - tool-boundary crossing
  - handoff
  - repair
  - retry
  - supervision
  - join or reconvergence
- may point to grounded evidence references when such evidence actually exists

### `TraceLink`

- explicit relationship between two trace events
- supported relationship types include:
  - parent-child execution flow
  - fan-out
  - join or reconvergence
  - retry of prior work
  - repair of prior work
  - supervision attached to an execution edge

## Proof-boundary entities

### `ProofBoundaryView`

- operator-facing summary of what the run proved and what it did not prove
- should preserve the packet-owned conservative phrases when the placeholder step boundary is still
  active
- should carry current proof status without collapsing packet `019`, `020`, and `021` into one
  vague label

### `ExecutionEvidenceClass`

- classifies the strongest evidence the run actually produced
- expected classes for this packet:
  - `substrate_completion`
  - `placeholder_or_simulated_step_completion`
  - `grounded_tool_execution`
  - `grounded_task_proof`
- packet `023` defines the naming and semantics of these classes, not the later implementation
  mechanics

### `GroundedEvidenceReference`

- stable reference to real files, endpoints, artifacts, or other grounded work touched during the
  run
- may be absent when the run only proved orchestration-substrate completion
- absence is meaningful and must not be silently backfilled with optimistic language

## Invariants

- packet `022` remains the owner of durable lifecycle, event-history, compaction, and effect
  boundary semantics
- placeholder `workflow.execute_step` completion is not enough to classify a run as grounded task
  proof
- packet `019` and `020` live-proof notes remain distinct from packet `021` deterministic-only
  proof for newer supervision-evidence claims
- external tracing docs may shape names and link concepts, but they do not prove a full emitted
  span model exists in the repo today
- all rendered proof-boundary views should tell the same bounded truth story across task, session,
  autonomy, and operator surfaces

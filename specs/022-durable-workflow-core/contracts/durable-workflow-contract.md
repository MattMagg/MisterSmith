# Durable Workflow Contract

## Status

Implementation contract for packet `022`.
This contract is ready for the first bounded implementation slice on current `main`.

## Purpose

Freeze one bounded contract for:

- durable workflow history semantics
- replay-safe lifecycle state transitions
- idempotent effect boundaries
- bounded compaction and replay governance

## Contract Rules

### 1. History Is Canonical

- Accepted workflow history is the semantic source of truth.
- Derived task, session, and autonomy views are projections of that accepted history.
- Checkpoints, snapshots, and compaction records support recovery, but they do not replace the
  requirement for canonical durable history semantics.
- The first slice stores canonical accepted history on the SQL-backed workflow record, with KV
  replay state used only as assistive cache or pointer state.

### 2. Replay Must Be Deterministic

- Replaying the same accepted history must rebuild the same durable workflow state.
- Replay-safe transition rules must be defined for workflow, branch, node, and lifecycle changes.
- Replay-version changes must be guarded by explicit replay-regression posture before the packet is
  widened.

### 3. Effects Cross an Explicit Boundary

- Accepted state transitions and external side effects are not the same contract.
- Every effect boundary must support durable intent and durable outcome tracking.
- Effect-outcome correctness must not be overstated as exactly-once state-transition correctness.
- The first slice stores effect intent and effect outcome as persistence-owned records keyed by
  workflow and effect boundary identity plus idempotency or dedup reference.

### 4. Lifecycle Verbs Need One Meaning

- `pause`, `resume`, `cancel`, and `terminate` must have one explicit meaning wherever packet
  `022` projects them.
- Task, session, and autonomy surfaces must not present contradictory lifecycle meanings for the
  same workflow.
- The first slice records lifecycle decisions durably and may return `applied`, `noop`, or
  `deferred` outcomes. It does not by itself prove live runner pause, resume, cancel, or
  terminate control.
- `reset/rewind` is intentionally deferred in the first slice and must be called out plainly.

### 5. Compaction Must Stay Bounded

- Long-running workflow replay cost must have a bounded strategy.
- The first compaction mechanism is a minimal lineage-preserving compaction record with source
  range, replay start pointer, and preserved lineage note.
- This packet does not add a broader snapshot platform or storage redesign.

## Proof Boundary

- This packet does not claim new live-default truth just by existing.
- Existing restart-resume proof remains the current live baseline that packet `022` must preserve.
- Any later implementation must distinguish deterministic validation from fresh live runtime proof.

## Explicit Non-Goals

- coordinator-runtime expansion
- interoperability or federation contracts
- strong coordination, consensus, CRDT, or MPST work
- cloning the structure of Temporal or Azure Durable Functions

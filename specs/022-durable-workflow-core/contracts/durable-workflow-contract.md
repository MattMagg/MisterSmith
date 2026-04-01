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

### 2. Replay Must Be Deterministic

- Replaying the same accepted history must rebuild the same durable workflow state.
- Replay-safe transition rules must be defined for workflow, branch, node, and lifecycle changes.
- Replay-version changes must be guarded by explicit replay-regression posture before the packet is
  widened.

### 3. Effects Cross an Explicit Boundary

- Accepted state transitions and external side effects are not the same contract.
- Every effect boundary must support durable intent and durable outcome tracking.
- Effect-outcome correctness must not be overstated as exactly-once state-transition correctness.

### 4. Lifecycle Verbs Need One Meaning

- Pause, resume, cancel, terminate, and reset or rewind posture must have one explicit meaning
  wherever packet `022` projects them.
- Task, session, and autonomy surfaces must not present contradictory lifecycle meanings for the
  same workflow.
- Any lifecycle verb intentionally deferred in the first slice must be called out plainly.

### 5. Compaction Must Stay Bounded

- Long-running workflow replay cost must have a bounded strategy.
- The first compaction mechanism may be minimal, but it must preserve resumability and explainable
  lineage.
- The exact first compaction mechanism remains open for the first implementation slice.

## Proof Boundary

- This packet does not claim new live-default truth just by existing.
- Existing restart-resume proof remains the current live baseline that packet `022` must preserve.
- Any later implementation must distinguish deterministic validation from fresh live runtime proof.

## Explicit Non-Goals

- coordinator-runtime expansion
- interoperability or federation contracts
- strong coordination, consensus, CRDT, or MPST work
- cloning the structure of Temporal or Azure Durable Functions

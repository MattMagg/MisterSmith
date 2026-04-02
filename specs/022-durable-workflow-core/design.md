# Design Note: Durable Workflow Core

## Status

This is the active design note for packet `022`.
It is the implementation authority for the first packet-022 pass on current `main`.

## Purpose

This file captures the cross-crate design choices that are too important to leave implied in
`spec.md` alone:

- what becomes canonical durable history
- what remains derived projection state
- where lifecycle commands and decisions cross crate boundaries
- where effect intent and effect completion must stay separate
- which seams need first-slice narrowing while coding begins

## Design Invariants

1. Accepted workflow history is the semantic source of truth.
2. Replay must be deterministic from accepted history.
3. Session continuity is preserved baseline behavior, not redesign scope.
4. State-transition correctness and effect-outcome correctness stay separate.
5. Compaction is bounded and lineage-preserving.
6. Packet `022` stays out of coordinator-runtime, interoperability, and strong coordination work.

## Proposed Boundary Map

### History

- Canonical durable records should describe accepted workflow progress, lifecycle changes, and
  lineage needed for replay.
- Derived operator-facing views should stay projections over that history, not rival truth stores.
- The first packet-022 pass stores canonical accepted history in SQL-backed workflow metadata, with
  KV used only as replay assist or cache where it adds operator or runtime value.

### Lifecycle

- Operator-facing lifecycle verbs should map to one durable meaning before any surface-specific
  formatting is applied.
- Task, session, and autonomy views should project the same durable lifecycle result.
- The first slice supports `pause`, `resume`, `cancel`, and `terminate`.
- `reset/rewind` is explicitly deferred and must surface as deferred or unsupported if referenced.

### Effect Boundary

- Effect intent and effect completion must be durably distinct.
- Replay can reuse durable completion evidence, but it cannot silently convert missing completion
  into success.
- The first slice keeps effect intent and outcome in persistence-owned records attached to the
  workflow record, keyed by workflow plus effect boundary identity and idempotency reference.
- JetStream stays transport and replay assist, not the effect source of truth.

### Compaction

- The first compaction slice should bound replay cost without erasing explainable lineage.
- The first compaction mechanism is one lineage-preserving compaction record stored with the
  workflow history, including source range, replay start pointer, and preserved lineage note.
- This packet does not add a broader snapshot platform or storage redesign.

## Candidate Write Seams

- `crates/mister-smith-agents/src/branch_checkpoint.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-persistence/src/kv/state.rs`
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
- `crates/mister-smith-persistence/src/repository/task.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-http/tests/lifecycle_handler_tests.rs`

## First-Slice Narrowing Decisions

- Canonical history event shape:
  `workflow_id`, `event_id`, monotonic replay position, `event_type`, `recorded_at`, optional
  actor or source identity, replay payload, and optional lineage, lifecycle, effect, or
  compaction references.
- Supported lifecycle verbs now:
  `pause`, `resume`, `cancel`, `terminate`.
  `reset/rewind` stays deferred.
- The first slice records lifecycle decisions durably and projects `applied`, `noop`, or
  `deferred` outcomes. It does not claim live runner pause, resume, cancel, or terminate control
  just by recording those decisions.
- Effect boundary placement:
  SQL-backed persistence-owned intent and outcome records keyed by workflow and effect identity
  plus idempotency reference, with KV optional as replay assist.
- Replay-regression fixtures:
  deterministic crate-level fixtures for raw history replay, repeated lifecycle commands,
  completed-effect replay, and post-compaction replay.

## Kickoff Use

At implementation kickoff:

1. re-read the repo truth docs and transfer brief
2. compare the current seams against this note
3. update this file only if current `main` materially contradicts packet `022`
4. then freeze the first implementation slice

# Design Note: Durable Workflow Core

## Status

This is a scaffold design note for packet `022`.
It is useful now for planning speed, but it must be refreshed before implementation because
earlier packet work is still in flight.

## Purpose

This file captures the cross-crate design choices that are too important to leave implied in
`spec.md` alone:

- what becomes canonical durable history
- what remains derived projection state
- where lifecycle commands and decisions cross crate boundaries
- where effect intent and effect completion must stay separate
- which seams need refresh before coding

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

### Lifecycle

- Operator-facing lifecycle verbs should map to one durable meaning before any surface-specific
  formatting is applied.
- Task, session, and autonomy views should project the same durable lifecycle result.

### Effect Boundary

- Effect intent and effect completion must be durably distinct.
- Replay can reuse durable completion evidence, but it cannot silently convert missing completion
  into success.

### Compaction

- The first compaction slice should bound replay cost without erasing explainable lineage.
- The exact first mechanism is still refresh-required.

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

## Refresh-Required Decisions

- the exact durable event shape
- the first compaction mechanism
- the first placement of the intent/effect boundary across PostgreSQL and JetStream
- the first replay-regression fixture model

## Pre-Implementation Use

Before any coding starts:

1. re-read the repo truth docs and transfer brief
2. compare the current seams against this note
3. update this file if upstream packet work changed the touched surfaces
4. only then freeze the first implementation slice

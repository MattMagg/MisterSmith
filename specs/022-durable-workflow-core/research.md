# Research Notes: Durable Workflow Core

## Purpose

This file records the decision-quality research for packet `022` as a scaffold packet.
Where the repo is not ready to freeze an exact implementation choice yet, this file says that
plainly instead of guessing.

## Decision 1: Event history should be the semantic source of truth

**Decision**: Keep the packet centered on one durable workflow history contract and treat derived
workflow views as projections of accepted history.

**Rationale**: The March 28 durable-workflows transfer brief says the strongest durable-workflow
transfer is event history plus deterministic replay. Current repo seams already have checkpoint and
resume metadata, but they do not yet form one frozen history contract.

**Alternatives considered**:

- Treat checkpoints alone as the durable source of truth.
  Rejected because checkpoints are useful recovery artifacts, not a full durable workflow semantic
  contract.
- Copy the full structure of Temporal or Azure Durable Functions.
  Rejected because the product goal is semantic strength, not framework imitation.

## Decision 2: Effect correctness needs explicit intent and completion boundaries

**Decision**: Keep state-transition correctness and effect-outcome correctness as separate packet
concerns, joined by one explicit effect boundary contract.

**Rationale**: The durable-workflows transfer brief is clear that broker-level dedup alone is not
enough. Current repo truth includes persistence, transport, and some idempotency-related substrate,
but not one frozen durable workflow effect boundary.

**Alternatives considered**:

- Treat JetStream deduplication as enough.
  Rejected because it does not solve durable intent-versus-effect correctness.
- Defer all effect-boundary work to a later packet.
  Rejected because durable replay semantics are incomplete without it.

## Decision 3: Existing session restart-resume behavior is baseline truth

**Decision**: Preserve existing session continuity and restart-resume semantics as a hard packet
constraint.

**Rationale**: The March 19 live proof already shows stable `session_id`,
`coordinator_agent_id`, and resumed lineage through a restart. Packet `022` should build under
that proof surface, not redefine it.

**Alternatives considered**:

- Redesign session semantics at the same time as durable workflow semantics.
  Rejected because it would widen the packet and erase a useful proof baseline.

## Decision 4: Lifecycle verbs must be frozen, but some exact posture may remain refresh-required

**Decision**: Freeze the requirement that pause, resume, cancel, terminate, and reset or rewind
posture must be made explicit, but leave the exact first-cycle implementation posture to the
refresh gate if upstream work materially changes touched seams.

**Rationale**: The packet must own lifecycle semantics, but the user also asked for honest
scaffolding while adjacent packet work is still moving.

**Alternatives considered**:

- Force exact lifecycle meanings now even if upstream seams shift.
  Rejected because it risks writing false finality into a scaffold packet.
- Leave lifecycle verbs fully unspecified.
  Rejected because the packet would lose one of its main reasons to exist.

## Decision 5: Compaction must be planned now, but the first exact mechanism remains open

**Decision**: Require a bounded compaction rule in the packet, while leaving the first exact
mechanism open until the pre-implementation refresh.

**Rationale**: Replay cost cannot grow forever. At the same time, the repo does not yet prove one
best mechanism for the first slice.

**Alternatives considered**:

- Ignore compaction in the first packet.
  Rejected because that would leave a known substrate gap in place.
- Force one exact compaction mechanism now.
  Rejected because the user asked to keep open questions explicit instead of guessing.

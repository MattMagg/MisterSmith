# Research Notes: Selective Strong Coordination

## Current repo truth

- the repo already has landed JetStream KV compare-and-swap behavior for strict serialized state
- the repo already has landed SQL-plus-KV routing seams that distinguish some strict state from
  other shared state in practice
- the repo already has landed durable transport subjects and transport metadata that could support
  stronger coordination later
- the repo does not yet have a frozen state taxonomy, a reusable strong-coordination layer, or a
  default-path live runtime feature that proves stronger coordination is required now

## Decision 1: Freeze taxonomy before mechanisms

**Decision**: Start packet `028` with state taxonomy and invariant rules, not with CRDT-first or
MPST-first design.

**Rationale**: The packet-prep dossier and transfer brief both say the honest first move is to
classify state by correctness need. That avoids turning the packet into a broad mechanism chase.

**Alternatives considered**:

- CRDT-first framing: rejected because the packet-prep boundary explicitly says not to widen into a
  repo-wide CRDT rollout
- MPST-first framing: rejected because protocol safety is only a later follow-on if packet `027`
  proves a stable seam worth protecting

## Decision 2: Use an invariant-driven coordination choice rule

**Decision**: Strong coordination is required only when concurrent updates can violate an explicit
invariant.

**Rationale**: The coordination transfer brief sharpens the repo's existing direction: selective
coordination is a better fit than convergent state everywhere or strict coordination everywhere.

**Alternatives considered**:

- default convergent state: rejected because some repo-owned state already depends on strict CAS
  semantics
- default strict serialization: rejected because not every shared artifact needs the latency and
  operational cost of stronger coordination

## Decision 3: Freeze one reusable primitive only

**Decision**: Freeze one reusable primitive, `InvariantCell`, as the first bounded outcome for
packet `028`.

**Rationale**: One primitive is enough to make the packet actionable later without turning it into
an open-ended coordination subsystem design.

**Alternatives considered**:

- freezing two or more primitives now: rejected because upstream packet work is still moving and
  the packet must stay easy to revise
- freezing no reusable primitive: rejected because the packet would become pure taxonomy with no
  reusable implementation target

## Decision 4: Ground `InvariantCell` in current CAS substrate

**Decision**: `InvariantCell` is defined as a CAS-guarded invariant state object using the repo's
existing KV compare-and-swap and reject-on-conflict behavior.

**Rationale**: The strict-state substrate is already landed. Reusing it keeps the packet honest
about what exists now and avoids inventing a brand-new coordination plane in the scaffold.

**Alternatives considered**:

- a new replicated-object layer: rejected because the repo does not yet have a frozen shared-state
  coordination layer
- consensus-first design: rejected because the packet-prep boundary says not to widen into generic
  coordination research

## Decision 5: Keep protocol safety behind a seam gate

**Decision**: Protocol safety and MPST stay deferred unless packet `027` later freezes a stable
protocol seam worth protecting.

**Rationale**: The packet-prep dossier is explicit that packet `028` should consume a stable seam,
not invent one. This keeps packet `027` as the owner of interoperability freeze decisions.

**Alternatives considered**:

- freezing protocol safety now: rejected because it would turn packet `028` into hidden packet `027`
  follow-on work
- removing protocol safety from the packet entirely: rejected because it is still a valid later
  child slice if the seam becomes real

## Decision 6: Treat this packet as a scaffold that must be revised later

**Decision**: The packet may be authored now for speed, but it must be revised before any
implementation work begins.

**Rationale**: Earlier packet work is still in flight, and the current packet truth must stay
honest about that dependency state.

**Alternatives considered**:

- pretending the packet is implementation-ready now: rejected because it would overstate repo
  truth
- delaying authoring entirely: rejected because the user wants the scaffold in place now to reduce
  future lead time

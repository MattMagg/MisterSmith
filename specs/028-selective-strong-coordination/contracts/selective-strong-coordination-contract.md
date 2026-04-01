# Contract: Selective Strong Coordination

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one bounded coordination contract for packet `028` so later work can classify shared state,
choose stronger coordination only when an invariant requires it, and reuse one packet-owned
primitive without widening into a broader coordination program.

This packet is a scaffold contract. It must be revised before implementation begins.

## Canonical Taxonomy

The packet-owned state classes are:

- `Convergent shared artifact`
  - merge is allowed when correctness is preserved
  - not used for durable external effects
- `Coordinated invariant state`
  - stronger coordination is required because concurrent updates can violate an explicit invariant
  - `InvariantCell` is the first reusable primitive for this class
- `Effectful state`
  - durable or irreversible effects stay on the effect path
  - this class is not handled as CRDT-style mergeable shared state

No fourth packet-owned state class is introduced in the first slice.

## Canonical Coordination Rule

The packet-owned choice rule is:

1. classify the state surface into one `StateClass`
2. identify whether concurrent updates can violate an explicit invariant
3. if no invariant can be violated, convergent handling remains allowed
4. if an invariant can be violated, coordinated invariant handling is required
5. if the state transition drives durable or irreversible effects, keep that work on the effect
   path instead of on merge logic

This rule is driven by invariants, not by mechanism preference alone.

## Canonical Primitive

The first packet-owned reusable primitive is:

- `InvariantCell`
  - CAS-guarded
  - reject-on-conflict
  - bounded to invariant-critical shared state
  - not a durable effect executor

Example authoritative shape:

```json
{
  "cell_key": "workflow.quota.global",
  "invariant_refs": ["quota-must-not-go-negative"],
  "current_revision": 14,
  "conflict_policy": "reject",
  "allowed_transitions": ["decrement", "restore"],
  "effect_boundary_ref": null
}
```

Behavior:

- concurrent conflicting updates are rejected rather than silently merged
- the primitive is grounded in existing KV compare-and-swap behavior
- the primitive does not imply repo-wide adoption for all state surfaces

## Protocol Seam Gate

Protocol safety and MPST stay deferred unless the packet `027` seam check later says otherwise.

The gate is:

- if packet `027` has **not** frozen a stable protocol seam, packet `028` keeps protocol safety
  fully deferred
- if packet `027` **has** frozen a stable protocol seam, a later child slice may revisit protocol
  safety without widening the first packet `028` implementation slice

## Relationship To Existing Seams

This scaffold assumes and reuses the following repo seams:

- `crates/mister-smith-persistence/src/kv/state.rs`
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
- `crates/mister-smith-persistence/src/hybrid/router.rs`
- `crates/mister-smith-transport/src/durable.rs`
- `crates/mister-smith-transport/src/subject.rs`
- `crates/mister-smith-transport/src/envelope.rs`

This packet does **not** claim those seams already form a complete live strong-coordination layer.
It uses them as substrate only.

## Non-Goals Locked By Contract

- no repo-wide CRDT rollout
- no MPST-first packet scope
- no generic distributed-systems experimentation
- no claim that stronger coordination is already part of the default live runtime path
- no second packet-owned reusable primitive in the first slice

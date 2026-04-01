# Implementation Plan: Selective Strong Coordination

**Branch**: `028-selective-strong-coordination` | **Date**: 2026-04-01 | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from
`/specs/028-selective-strong-coordination/spec.md`

## Summary

This packet is a scaffold-first coordination packet, not an implementation-ready freeze. It exists
to lock three things early so later work can move faster: one three-class state taxonomy, one
invariant-driven coordination choice rule, and one reusable `InvariantCell` primitive grounded in
existing KV CAS behavior. Before any code work starts, the packet must be revised against the then
current state of packets `022`, `023`, `024`, and `027`.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned markdown packet artifacts
**Primary Dependencies**: `mister-smith-core`, `mister-smith-persistence`,
`mister-smith-transport`, `mister-smith-agents`, repo router docs, and the packet-prep research
set for packet `028`
**Storage**: existing JetStream KV CAS substrate, SQL-plus-KV routing seams, and packet docs only;
no new runtime store is introduced by this scaffold
**Testing**: packet-artifact consistency review, `git diff --check`, targeted markdown lint, and a
read-only cross-artifact analysis report
**Target Platform**: Mister Smith Rust workspace and its existing packet/doc pipeline
**Project Type**: later-gated scaffold packet with future Rust workspace implementation tasks
**Performance Goals**: make coordination-classification decisions explicit without widening the
live runtime claim surface
**Constraints**: later-gated only, no live-default claim, no repo-wide CRDT rollout, no MPST core
slice, no generic coordination research program, and no code work before revalidation
**Scale/Scope**: one bounded packet scaffold covering taxonomy, decision rule, and one reusable
primitive

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, the packet-prep dossier, and current persistence/transport seams. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `tasks.md`, and `analyze.md` are authored before any implementation work. |
| III. Phase-And-Packet-Gated Delivery | PASS | The packet stays later-gated and explicitly depends on upstream packet outcomes before implementation. |
| IV. Model-Agnostic Architecture | PASS | The scaffold is about state and coordination rules, not provider-specific behavior. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet preserves durable effect and state boundaries instead of weakening them through merge-everywhere logic. |
| VI. Evidence-Based Validation | PASS | The scaffold keeps landed substrate, deterministic-only justification, and live-runtime truth explicitly separated. |
| VII. Explicit Dependency Management | PASS | Dependency gates on packets `022`, `023`, `024`, and `027` are explicit and blocking. |
| VIII. Clean Closure And Resumability | PASS | The scaffold produces a cold-start packet set and requires a revalidation pass before code work begins. |

Re-check after Phase 1 design: still PASS. The scaffold remains authorable now, but upstream
packet completion is a gate for implementation, not a gate for writing this packet.

## Project Structure

### Documentation (this feature)

```text
specs/028-selective-strong-coordination/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── selective-strong-coordination-contract.md
├── checklists/
│   ├── requirements.md
│   └── coordination.md
├── tasks.md
└── analyze.md
```

### Source Code (future implementation seams only)

```text
crates/mister-smith-core/
├── src/autonomy.rs
└── src/lib.rs

crates/mister-smith-persistence/
├── src/kv/state.rs
├── src/hybrid/manager.rs
├── src/hybrid/router.rs
└── tests/

crates/mister-smith-transport/
├── src/durable.rs
├── src/subject.rs
└── src/envelope.rs

crates/mister-smith-agents/
└── src/
```

**Structure Decision**: This scaffold packet writes only under
`specs/028-selective-strong-coordination/` now. The code paths above are the future implementation
seams, not the current write set.

## Design Decisions

### D1: Freeze taxonomy before choosing mechanisms

**Decision**: Packet `028` starts with state taxonomy and invariant rules, not with CRDT-first or
MPST-first design.

**Rationale**: The packet-prep dossier and coordination transfer brief both show that the honest
missing seam is classification and choice discipline, not another abstract coordination framework.

### D2: Freeze one primitive only

**Decision**: The first slice freezes one reusable strong-coordination primitive, `InvariantCell`,
and defers any second primitive until after revalidation.

**Rationale**: One bounded reusable outcome keeps the packet useful without turning it into a broad
coordination subsystem design.

### D3: Keep protocol safety behind a seam gate

**Decision**: Protocol safety and MPST remain deferred unless packet `027` later proves a stable
protocol seam worth protecting.

**Rationale**: The packet-prep boundary is explicit that packet `028` should consume a stable seam,
not invent one.

### D4: Treat this packet as scaffolding, not execution authority

**Decision**: This packet may be written now for speed, but it must be revised before
implementation starts.

**Rationale**: Upstream packet work is still moving, so pretending this scaffold is final would
violate the repo's truth and gating rules.

## Milestones

### Milestone 0: Pre-Implementation Revalidation

**Scope**: confirm current repo truth, upstream packet outcomes, and whether packet `027` froze a
stable protocol seam.

**Validation**:

- `docs/direction.md` and `docs/current-state.md` still support the packet's later-gated posture
- packets `022`, `023`, `024`, and `027` are far enough along to confirm the dependency map
- this scaffold is refreshed if upstream wording or proof boundaries moved

### Milestone 1: Freeze taxonomy and choice rule

**Scope**: lock the three state classes, representative examples, and the invariant-driven
coordination choice rule.

**Validation**:

- representative state examples map to exactly one class
- the decision rule clearly distinguishes convergent, coordinated, and effectful handling

### Milestone 2: Freeze `InvariantCell`

**Scope**: define the first reusable strong-coordination primitive and its effect-path boundary.

**Validation**:

- `InvariantCell` remains the only packet-owned primitive in the first slice
- the primitive stays grounded in existing CAS and reject-on-conflict behavior

### Milestone 3: Decide whether protocol safety stays deferred

**Scope**: consume the packet `027` seam outcome and either keep protocol safety deferred or open a
later child slice.

**Validation**:

- the seam decision is based on then-current packet `027` truth
- packet `028` still does not silently widen into MPST-first scope

## Parallel Staging Posture

- No implementation lane may start before Milestone 0 is complete.
- Milestone 1 and Milestone 2 can split across doc refresh and code-prep lanes only after the
  revalidation gate passes.
- Shared choke points for any future implementation are:
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-persistence/src/kv/state.rs`
  - `crates/mister-smith-persistence/src/hybrid/router.rs`
  - `crates/mister-smith-transport/src/durable.rs`
  - `crates/mister-smith-transport/src/envelope.rs`

## Explicitly Deferred

- repo-wide CRDT rollout
- protocol safety and MPST as part of the first packet `028` slice
- generic distributed-systems experimentation
- any claim that stronger coordination is already part of the default live runtime path
- any implementation start before the revalidation gate passes

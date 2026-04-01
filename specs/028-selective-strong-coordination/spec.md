# Feature Specification: Selective Strong Coordination

**Feature Branch**: `028-selective-strong-coordination`
**Created**: 2026-04-01
**Status**: Draft scaffold
**Input**: `docs/direction.md`, `docs/current-state.md`, `docs/packet-prep/README.md`,
`docs/packet-prep/028-selective-strong-coordination.md`,
`docs/packet-prep/027-capability-discovery-and-interoperability.md`,
`docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`,
`docs/research-output/consolidated/05-coordination-and-state.md`, and the current coordination
substrate in `crates/mister-smith-persistence/src/kv/state.rs`,
`crates/mister-smith-persistence/src/hybrid/manager.rs`,
`crates/mister-smith-persistence/src/hybrid/router.rs`,
`crates/mister-smith-transport/src/durable.rs`,
`crates/mister-smith-transport/src/subject.rs`, and
`crates/mister-smith-transport/src/envelope.rs`

## Current Truth & Scaffolding Posture

This packet is a scaffold spec written early to speed later packet work while earlier packet
implementation work is still in flight.

- It freezes packet `028` scope, naming, taxonomy, and gating language now.
- It does not claim packet `028` is implementation-ready or already validated for execution.
- It must be revised before any implementation starts, after upstream packet implementation work is
  far enough along to confirm the real dependency state.

Current repo truth already includes:

- landed JetStream KV compare-and-swap substrate for strict serialized state updates
- landed SQL-plus-KV routing seams for state that already needs stronger control in practice
- landed durable transport subjects and transport metadata that can carry stronger coordination
  identity later
- landed task, session, and autonomy substrate through packet `021`, with current live proof still
  bounded to the earlier supported-path runtime packets
- landed bounded external-capability discovery seams, but no frozen broad interoperability contract
  on the default runtime path

What is still missing is one frozen packet contract for:

- a canonical state taxonomy that distinguishes convergent, coordinated, and effectful state
- one invariant-driven rule for choosing stronger coordination instead of convergent state
- one reusable strong-coordination primitive grounded in existing CAS behavior
- one explicit gate that keeps protocol safety and MPST as deferred follow-on work unless packet
  `027` later proves a stable seam worth protecting

This scaffold packet therefore freezes one bounded slice:

1. define one three-class state taxonomy for future coordination work
2. define one coordination choice rule that starts from invariants, not from CRDT or MPST zeal
3. define one reusable `InvariantCell` primitive for invariant-critical shared state

This is not:

- a repo-wide CRDT rollout
- MPST-first or protocol-safety-first design
- generic distributed-systems experimentation
- a claim that stronger coordination is already part of the default live runtime path
- the next frozen implementation phase after packet `021`

## Before Implementation Revalidation Gate

Before any future implementation starts, the next session must:

1. reread `docs/direction.md`
2. reread `docs/current-state.md`
3. reread `docs/packet-prep/028-selective-strong-coordination.md`
4. reread `docs/packet-prep/027-capability-discovery-and-interoperability.md`
5. confirm the current state of packets `022`, `023`, `024`, and `027`
6. rerun `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`, and `/speckit.analyze` if repo
   truth or upstream packet scope has moved

If those checks fail, this scaffold must be revised before code work begins.

## Deferred Revision Points

- recheck whether packet `027` actually froze a stable protocol seam worth protecting
- recheck whether packet `022`, packet `023`, and packet `024` changed lifecycle, proof-boundary,
  or security wording that packet `028` depends on
- recheck whether the best first strong-coordination example is still a shared invariant cell or a
  narrower packet-owned variant
- recheck whether any later live proof changed the boundary between landed substrate and
  deterministic-only justification for stronger coordination

## Clarifications

### Session 2026-04-01

- Q: Is packet `028` implementation-ready now? → A: No. This is a scaffold packet only and it
  must be refreshed after upstream packet work settles and before any coding starts.
- Q: Does packet `028` freeze protocol safety or MPST as part of the core slice? → A: No.
  Protocol safety stays deferred unless packet `027` later proves a stable seam worth protecting.
- Q: How many reusable strong-coordination primitives are frozen in this scaffold? → A: One.
  `InvariantCell` is the only frozen primitive in the first slice.
- Q: Are earlier packet completions an authoring gate or an implementation gate? → A: An
  implementation gate. This scaffold can be written now, but implementation must wait for a later
  revalidation pass.

## Refresh-Required Questions

- Which concrete repo-owned invariants still matter most after packets `022` through `024` finish
  landing?
- Does packet `027` actually freeze a protocol seam, or should protocol safety remain fully out of
  packet `028`?
- Which representative state surfaces belong in each taxonomy class once upstream packet wording is
  stable?
- What is the smallest proof artifact that would justify moving packet `028` beyond a scaffold
  design argument?

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Classify Shared State By Coordination Need (Priority: P1)

A future packet author or reviewer can classify shared runtime state into one of three classes so
later coordination work starts from the state's correctness needs instead of from a preferred
algorithm.

**Why this priority**: If the state taxonomy is not frozen first, later coordination work will
keep drifting between CRDT rhetoric, strict coordination, and effect handling.

**Independent Test**: A reviewer can take representative repo-owned state examples and place each
of them into exactly one taxonomy class using only the packet's written rules.

**Acceptance Scenarios**:

1. **Given** a shared artifact that can tolerate concurrent merges without violating correctness,
   **When** the packet taxonomy is applied, **Then** that state is classified as `Convergent
   shared artifact`.
2. **Given** a shared state object whose concurrent updates can violate an invariant, **When** the
   packet taxonomy is applied, **Then** that state is classified as `Coordinated invariant state`.
3. **Given** a state transition that can trigger durable or irreversible external work, **When**
   the packet taxonomy is applied, **Then** that state is classified as `Effectful state` rather
   than as mergeable shared state.

---

### User Story 2 - Choose Strong Coordination Only When An Invariant Requires It (Priority: P1)

A future implementation team can decide when strong coordination is justified by using one packet
rule rooted in invariant risk instead of defaulting to convergent state or defaulting to strict
serialization everywhere.

**Why this priority**: This is the packet's core decision rule. Without it, the packet becomes a
general coordination essay instead of a bounded design scaffold.

**Independent Test**: A reviewer can examine representative invariant cases and determine whether
strong coordination is required using only the packet's written decision rule.

**Acceptance Scenarios**:

1. **Given** a state update where concurrent merges cannot break correctness, **When** the packet
   rule is applied, **Then** the packet does not require strong coordination for that case.
2. **Given** a state update where concurrent writes can violate exclusivity, ordering, or another
   correctness invariant, **When** the packet rule is applied, **Then** the packet requires
   coordinated invariant handling instead of convergent merge semantics.
3. **Given** a state update tied to durable external effects, **When** the packet rule is applied,
   **Then** the packet keeps that flow on the effect path instead of treating it as CRDT-style
   shared state.

---

### User Story 3 - Reuse One Strong-Coordination Primitive Without Widening Packet Scope (Priority: P2)

A future implementation team can reuse one bounded primitive for invariant-critical shared state
without turning packet `028` into a repo-wide coordination program.

**Why this priority**: The packet should produce one reusable outcome, but it must stay small
enough to revise safely after upstream packet work settles.

**Independent Test**: A reviewer can explain how the packet's `InvariantCell` primitive works and
where it applies without inventing extra packet-owned coordination mechanisms.

**Acceptance Scenarios**:

1. **Given** an invariant-critical shared state surface, **When** the packet primitive is applied,
   **Then** the packet describes one `InvariantCell` with compare-and-swap style update rules and
   reject-on-conflict behavior.
2. **Given** a request to add protocol safety, CRDT rollout, or generic consensus work to the same
   slice, **When** the packet boundary is checked, **Then** that work is deferred instead of being
   absorbed into the first packet `028` implementation slice.
3. **Given** upstream packet wording changes before implementation starts, **When** the revalidation
   gate runs, **Then** the packet can be revised without pretending the original scaffold was final.

## Edge Cases

- a state surface appears mergeable at first but also controls an external effect boundary
- two state surfaces share identifiers or transport metadata but belong to different taxonomy
  classes
- packet `027` never freezes a stable protocol seam, leaving protocol safety fully out of packet
  `028`
- upstream packet work changes lifecycle, proof-boundary, or security wording before packet `028`
  is implemented
- a later session confuses landed strict-state substrate with proof that stronger coordination is
  already live by default
- a proposal tries to add more than one packet-owned strong-coordination primitive without first
  revising this scaffold

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Packet `028` MUST define exactly three state classes: `Convergent shared artifact`,
  `Coordinated invariant state`, and `Effectful state`.
- **FR-002**: Packet `028` MUST define each state class in terms of correctness and effect
  boundaries, not in terms of a preferred implementation pattern alone.
- **FR-003**: Packet `028` MUST define one coordination choice rule that uses invariant risk to
  decide whether stronger coordination is required.
- **FR-004**: Packet `028` MUST state that convergent state is allowed only when concurrent merge
  cannot violate correctness.
- **FR-005**: Packet `028` MUST state that coordinated invariant handling is required when
  concurrent updates can violate exclusivity, ordering, quota, or another explicit invariant.
- **FR-006**: Packet `028` MUST state that effectful state stays on the durable effect path and
  MUST NOT be treated as CRDT-style mergeable shared state.
- **FR-007**: Packet `028` MUST freeze exactly one reusable strong-coordination primitive named
  `InvariantCell`.
- **FR-008**: `InvariantCell` MUST be defined as a CAS-guarded invariant state object grounded in
  the repo's existing KV compare-and-swap and reject-on-conflict substrate.
- **FR-009**: Packet `028` MUST keep landed substrate, deterministic-only justification, and live
  runtime claims explicitly separated in packet wording and downstream notes.
- **FR-010**: Packet `028` MUST state clearly that this scaffold must be revised before any future
  implementation starts.
- **FR-011**: Packet `028` MUST treat completion of earlier packet work as an implementation gate,
  not as an authoring gate for this scaffold.
- **FR-012**: Packet `028` MUST keep protocol safety and MPST deferred unless packet `027` later
  proves a stable seam worth protecting.
- **FR-013**: Packet `028` MUST remain later-gated and MUST NOT be described as the next frozen
  implementation phase after packet `021`.
- **FR-014**: Packet `028` MUST document explicit dependency gates on upstream packet work from
  packet `022`, packet `023`, packet `024`, and packet `027`.
- **FR-015**: Packet `028` MUST keep CRDT rollout, generic distributed-systems experimentation,
  and additional strong-coordination primitives out of the first slice.
- **FR-016**: Any implementation plan or task set derived from packet `028` MUST start with a
  blocking pre-implementation revalidation phase before any code task begins.

### Key Entities *(include if feature involves data)*

- **StateClass**: the canonical packet-owned classification for one shared state surface,
  identifying whether it is convergent, coordinated, or effectful
- **Invariant**: one correctness rule whose violation would make concurrent merge unsafe
- **CoordinationDecisionRule**: the packet-owned rule that maps a state surface and its invariants
  to the required coordination posture
- **InvariantCell**: one reusable CAS-guarded state object for invariant-critical shared state,
  with reject-on-conflict semantics and a bounded ownership surface
- **ProtocolSeamGate**: the explicit precondition that determines whether protocol safety remains
  deferred or can enter a later child slice after packet `027`
- **RevalidationGate**: the required pre-implementation check that confirms this scaffold still
  matches current repo truth and upstream packet outcomes

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reviewer can classify 100 percent of the representative packet examples into
  exactly one of the three state classes without needing a fourth class.
- **SC-002**: A reviewer can determine whether strong coordination is required for each documented
  invariant example using the packet's written decision rule alone.
- **SC-003**: The packet defines one reusable primitive only, and all packet artifacts use the same
  name and scope for that primitive.
- **SC-004**: Every core packet artifact states that packet `028` is a scaffold that must be
  revised before implementation starts.
- **SC-005**: No packet artifact claims that strong coordination is already part of the default
  live runtime path.

## Assumptions

- Earlier packet implementation work is still moving, so the exact implementation seam for packet
  `028` may need revision later.
- The current repo substrate already provides enough strict-state and durable-effect grounding to
  write a bounded scaffold packet now.
- Packet `027` may or may not later freeze a stable protocol seam; this scaffold assumes it has
  not done so yet.
- One reusable primitive is enough for the first packet `028` slice.

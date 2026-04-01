# Data Model: Selective Strong Coordination

## Coordination taxonomy entities

### `StateClass`

- `class_name`: canonical packet-owned class name
- `definition`: concise correctness-oriented meaning
- `merge_allowed`: whether concurrent merge is allowed in principle
- `effect_boundary_allowed`: whether the class may directly drive durable external effects
- `representative_examples`: bounded example list used to keep classification honest

Canonical values:

- `Convergent shared artifact`
- `Coordinated invariant state`
- `Effectful state`

### `Invariant`

- `invariant_id`: stable identifier for one correctness rule
- `statement`: human-readable invariant statement
- `failure_if_violated`: what goes wrong if concurrent updates break the rule
- `affected_state_surface`: the state object or surface the invariant protects
- `effect_linkage`: whether violating the invariant can trigger or corrupt external effects

### `CoordinationDecisionRule`

- `rule_id`: stable identifier for the packet-owned rule
- `state_class`: the starting taxonomy classification
- `invariant_refs`: invariants considered by the rule
- `required_posture`: `convergent`, `coordinated`, or `effect-path only`
- `why`: plain-language explanation for the choice
- `deferred_reason`: optional reason why the final choice cannot be frozen until revalidation

## Strong-coordination primitive

### `InvariantCell`

- `cell_key`: stable key for the invariant-critical shared state
- `invariant_refs`: invariants enforced by the cell
- `current_revision`: last accepted CAS revision
- `conflict_policy`: reject-on-conflict behavior
- `allowed_transitions`: bounded transition set the cell may accept
- `effect_boundary_ref`: optional link to an external effect boundary that must stay outside merge
  logic

Interpretation:

- `InvariantCell` exists only for `Coordinated invariant state`
- it uses compare-and-swap style revision control
- it rejects conflicting concurrent writes instead of auto-merging them
- it does not absorb durable effect execution

## Gating entities

### `ProtocolSeamGate`

- `gate_id`: stable identifier for the packet `027` seam check
- `seam_status`: whether a stable protocol seam exists yet
- `source_packet_ref`: packet `027` artifact or note used for the decision
- `resolution`: `defer protocol safety` or `open later child slice`

### `RevalidationGate`

- `gate_id`: stable identifier for the pre-implementation refresh pass
- `required_inputs`: authoritative docs and upstream packets to reread
- `dependency_status`: current state of packets `022`, `023`, `024`, and `027`
- `result`: `revise scaffold` or `safe to continue`

## Invariants

- each representative state example maps to exactly one `StateClass`
- `Effectful state` is never handled as CRDT-style mergeable shared state
- `InvariantCell` is the only frozen reusable primitive in the first slice
- protocol safety remains deferred until `ProtocolSeamGate` passes
- implementation cannot start until `RevalidationGate` passes

# MS-75 External Capability Decision Visibility

Date: March 19, 2026
Issue: `MS-75`
Status: implemented and locally revalidated

## Objective

Expose one operator-visible explanation for why an external capability call was allowed or rejected,
then capture one repeatable proof artifact for that interoperability boundary without changing
delegation policy semantics.

## Scope

- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- targeted status/event tests plus one deterministic metadata-recovery proof

## Assumptions

- `MS-73` already landed descriptor-aware capability metadata and delegation decision plumbing
- `MS-74` already landed external delegation envelopes with typed delegated actions
- the correct proof level for this slice is deterministic status/event validation, not a new live
  provider runtime proof
- autonomy status may enrich live or persisted views from workflow metadata without altering the
  underlying security decision

## Constraints

- no new authorization or revocation semantics
- no new MCP or HTTP execution policy
- keep the slice bounded to operator visibility and proof only

## Non-Goals

- changing capability issuance or validation rules
- provider benchmarking or live interoperability demos
- queue/lifecycle workflow changes outside the normal issue execution flow

## Visibility Surface

Autonomy status now carries `external_capability_decisions` as a typed operator-facing summary.
Each summary records:

- the capability and action descriptor seen at the external boundary
- the required scope and policy binding carried by the delegated action
- the effective revocation state and authority-chain depth
- the final `allowed` or `rejected` outcome
- rationale lines that explain the decision

Two status paths now expose the same decision surface:

- live status snapshots can preserve ToolBus-published external capability decisions
- recovered status views derive the same decision summary from persisted `external_delegation`
  metadata when a live snapshot is unavailable

The renderer now prints an `external capability decisions:` section so operators can inspect one
line of allow/reject evidence directly from `autonomy status`.

## Deterministic Proof

Primary proof harnesses:

- `crates/mister-smith-agents/tests/tool_bus_tests.rs`
- `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `crates/mister-smith-app/src/execution.rs` unit tests:
  - `recover_persisted_autonomy_status_enriches_allowed_external_capability_decision`
  - `recover_persisted_autonomy_status_enriches_rejected_external_capability_decision`

Representative proof outcomes:

- allowed case:
  - metadata recovery derives `outcome = Allowed`
  - rationale includes descriptor match, scope match, and active chain-depth state
- rejected case:
  - metadata recovery derives `outcome = Rejected`
  - rationale includes the descriptor mismatch rejection string used at the boundary
- rejected-without-capability case:
  - privileged-tool rejection still emits an operator-visible `Rejected` boundary decision
  - rationale records that no bounded delegation capability was present at the external boundary
- rendered operator view:
  - status output includes `external capability decisions:`
  - rendered line includes outcome, descriptor/action identifiers, policy binding, and rationale

## Validation

```bash
cargo test -p mister-smith-agents --test tool_bus_tests -- --nocapture
cargo test -p mister-smith-events --test autonomy_event_tests -- --nocapture
cargo test -p mister-smith-app --test autonomy_status_tests -- --nocapture
cargo test -p mister-smith-app \
  recover_persisted_autonomy_status_enriches_allowed_external_capability_decision -- --nocapture
cargo test -p mister-smith-app \
  recover_persisted_autonomy_status_enriches_rejected_external_capability_decision -- --nocapture
cargo build --workspace
```

Observed local result in this session:

- all listed tests passed
- event projection preserved the typed decision summary
- autonomy status rendering exposed the new operator-facing section
- metadata recovery proved both allowed and rejected interoperability outcomes without widening the
  underlying policy contract

## Validation Boundary

What this note proves:

- operators can inspect why an external capability call was allowed or rejected from autonomy
  status
- the same allow/reject summary survives both live event projection and persisted metadata recovery
- one deterministic harness exists for the interoperability boundary

What this note does not prove:

- a new live provider-backed external interoperability session
- any change to capability validation semantics
- multi-hop external delegation beyond the currently bounded descriptor and scope contract

## Stop Conditions

- stop if autonomy status does not surface a dedicated external capability decision section
- stop if the allowed and rejected cases cannot be reproduced deterministically from repo tests
- stop before widening the slice into new delegation policy behavior or runtime-provider proof

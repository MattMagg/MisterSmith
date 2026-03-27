# Contract: Supervision Evidence Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one shared predictive-supervision contract for the supported runtime path so task,
autonomy, and operator run-detail surfaces all project the same runtime truth.

This packet does **not** create a second repair subsystem. Packet `020` remains canonical for
verifier-driven repair. Packet `021` adds bounded predictive-supervision evidence that composes
with that lineage.

## Canonical Mapping

The contract for this packet is:

- pre-graph supervision may remain provider-scoped
- once graph context exists, supervision targets branch or node scope rather than staying
  provider-only
- `ProfileFingerprint` is advisory persisted context keyed by supported role or target class
- `ProfileSnapshot` is the live health snapshot derived from current runtime evidence
- `GuardDecision` is the typed classification and chosen intervention rationale
- `InterventionRecord` is the applied local recovery action and before/after state
- task, autonomy, and run-detail surfaces expose bounded projections derived from the same
  canonical supervision state

No other predictive-supervision surface may become a competing top-level contract in this packet.

## Canonical Evidence Shape

Example authoritative payload shape:

```json
{
  "workflow_id": "11111111-1111-1111-1111-111111111111",
  "target_scope": {
    "kind": "branch",
    "branch_id": "22222222-2222-2222-2222-222222222222",
    "node_id": "33333333-3333-3333-3333-333333333333"
  },
  "fingerprint_ref": {
    "fingerprint_key": "executor:branch",
    "confidence": 0.81,
    "expires_at": "2026-03-28T15:00:00Z"
  },
  "profile_snapshot": {
    "health": "degraded",
    "failure_tendency": "context_stall"
  },
  "guard_decision": {
    "failure_class": "recoverable_context_loss",
    "action": "context_refresh"
  },
  "intervention_record": {
    "action": "context_refresh",
    "result": "applied"
  },
  "repair_lineage_ref": {
    "source": "packet-020",
    "checkpoint_id": "last-stable-checkpoint"
  }
}
```

Behavior:

- branch and node targets are preferred whenever graph context exists
- provider scope is allowed only before graph context is available
- fingerprint references are advisory and may reinforce, but not override, live failure signals
- packet `020` repair lineage may be linked or projected alongside supervision evidence, but it
  is not replaced

## Fingerprint Storage Contract

`ProfileFingerprint` is stored only through the existing JetStream KV seam in
`crates/mister-smith-persistence/src/kv/`.

Expected behavior:

- fingerprints store structured summaries, confidence, provenance, expiry, and source references
- fingerprints do **not** duplicate raw transcripts outside existing audit or replay surfaces
- invalid or structurally incomplete fingerprint payloads fail explicitly
- stale or contradicted fingerprints degrade cleanly to live-signal-only supervision

## Task Surface Contract

`task.result` remains the task-facing authoritative inspection surface for the latest supervision
state.

Expected behavior:

- terminal task inspection exposes the latest bounded supervision evidence view
- task result preserves enough provenance to correlate target scope, fingerprint reference, guard
  decision, intervention, and any linked packet `020` repair lineage
- operators can recover the full canonical supervision state from task inspection without reading
  raw logs

## Autonomy Surface Contract

`AutonomyStatusView` remains the main operator-facing status surface.

Expected behavior:

- autonomy status carries the latest supervision summary derived from the canonical runtime state
- autonomy views expose target scope, current health, selected intervention, and fingerprint
  reference when present
- autonomy status remains bounded and points back to task result or run detail for deeper
  inspection

## Operator Run-Detail Contract

The operator console run detail remains a bounded summary surface, not a raw payload dump.

Expected behavior:

- run detail renders predictive-supervision evidence as first-class summary content
- the view distinguishes packet `020` verifier/repair lineage from packet `021`
  predictive-supervision lineage
- the view does not widen into a new dashboard mode or generic observability redesign

## Relationship To Existing Surfaces

The following existing surfaces remain authoritative baseline:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/profile.rs`
- `crates/mister-smith-agents/src/guard.rs`
- `crates/mister-smith-agents/src/intervention.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `apps/operator-console/src/views/RunsView.tsx`

This packet only extends them with:

- one explicit supervision-evidence contract
- one bounded fingerprint storage rule
- one coherent projection across task, autonomy, and run-detail surfaces

## Parallel Symphony Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already landed
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- the active `docs/plans/...` proof note

Only one active lane may own a choke-point file at a time.

# Contract: Quarantine And Schema Enforcement

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one deterministic mediation contract for cross-boundary payloads and shared-state reads so
content is validated before agent consumption and boundary outcomes are explicit.

## Validation Pipeline

Packet `024` keeps the current validation pipeline as the canonical baseline:

1. size check
2. deterministic sanitization of disallowed control content
3. schema validation against the expected payload shape
4. malicious-pattern inspection
5. taint-label classification
6. quarantine action mapping

This contract remains grounded in:

- `crates/mister-smith-security/src/state_validator.rs`
- `crates/mister-smith-security/src/quarantine.rs`
- `crates/mister-smith-security/src/sandbox.rs`
- `crates/mister-smith-agents/src/sandbox.rs`
- `crates/mister-smith-persistence/src/repository/agent.rs`

## Canonical Outcomes

The packet freezes these canonical outcomes:

- `Pass`
  - clean payload, forward unchanged
- `Sanitize`
  - payload forwarded after deterministic sanitization with a human-readable reason
- `Suspicious`
  - payload may continue only under monitored handling with a human-readable reason
- `Reject`
  - boundary blocks the payload, returns an error, and records a human-readable reason
- `Quarantine`
  - boundary isolates the payload for investigation, blocks it, and records a human-readable
    reason

The packet also preserves the existing taint labels:

- `Clean`
- `Sanitized`
- `Suspicious`
- `Rejected`

## Shared-State Contract

Expected behavior:

- shared-state reads and writes that cross a protected boundary remain mediated through the same
  validation and quarantine rules
- no shared-state payload is passed directly into agent context without deterministic mediation
- validated state returned to an agent includes enough classification data for downstream audit or
  monitoring

## Cross-Boundary Contract

Expected behavior:

- when a crossing rule says quarantine is required, the boundary must route through a quarantine
  actor
- if quarantine is required and no quarantine actor is attached, the crossing fails explicitly
- same-account traffic may pass only when the current sandbox rules allow it

## Canonical Evidence Shape

Example authoritative shape:

```json
{
  "boundary": "shared_state",
  "source": "shared_state",
  "target": "agent",
  "resource": "read:memory.snapshot",
  "action": "sanitize",
  "taint_label": "sanitized",
  "reason": "control characters removed during sanitization",
  "detected_pattern": null,
  "monitored": true
}
```

Behavior:

- the action and taint label must stay coherent
- sanitized outcomes always carry a reason
- monitored suspicious outcomes always carry a reason
- rejected and quarantined outcomes always carry a reason
- suspicious or sanitized content remains visible to audit and monitoring

## Source Rule

Packet `024` uses JSON Schema as the structural-validation reference.
It does not widen into a generic semantic-policy engine or a model-enforced safety layer.

## Deferred

This contract does not freeze:

- a new classifier service
- ML-based semantic filtering
- a broader content moderation program
- UI work beyond boundary evidence already justified by existing runtime seams

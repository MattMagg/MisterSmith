# Contract: Quarantine And Schema Enforcement

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one deterministic mediation contract for cross-boundary payloads and shared-state reads so
content is validated before agent consumption and boundary outcomes are explicit.

This contract is part of draft packet scaffolding and must be refreshed before implementation if
earlier packet work changes the reused validator, quarantine, or sandbox seams.

## Validation Pipeline

Packet `024` keeps the current validation pipeline as the canonical baseline:

1. size check
2. sanitization of disallowed control content
3. schema validation
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
  - payload forwarded after deterministic sanitization
- `Reject`
  - boundary blocks the payload and returns an error
- `Quarantine`
  - boundary isolates the payload for investigation and blocks it

The packet also preserves the existing taint labels:

- `Clean`
- `Sanitized`
- `Suspicious`
- `Rejected`

`Suspicious` remains a monitored state, not silent success.

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
- non-pass outcomes always carry a reason
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

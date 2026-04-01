# Contract: Identity And Sandbox Boundary

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one least-privilege identity and sandbox contract across delegation, auth callout, and
persistent-versus-ephemeral sandbox boundaries without widening into a new IAM program.

This contract is part of draft packet scaffolding and must be refreshed before implementation if
earlier packet work changes reused identity or continuity seams.

## Identity Baseline

Packet `024` keeps this current identity baseline:

- JWT-backed principal claims
- auth-callout-issued least-privilege NATS credentials
- bounded delegation capabilities and provenance chains
- external delegation envelopes for boundary crossings

This packet does **not** adopt a new SPIFFE-based identity rollout.
SPIFFE remains comparator guidance only for later work.

## Auth Callout Contract

Expected behavior from `crates/mister-smith-security/src/auth_callout.rs`:

- permission tiers remain `full`, `standard`, `restricted`, and `quarantined`
- fallback stays on the minimal quarantined posture
- `$SYS.>` and `$JS.>` remain denied for narrowed credentials
- TTL remains bounded by current tier rules
- delegation references, when present, remain tied to the current validated authority chain

## Delegation Contract

Expected behavior from `crates/mister-smith-security/src/delegation.rs`:

- delegated authority stays bound to exact action requirements
- revocation and expiry remain first-class enforcement paths
- external envelopes remain bounded wrappers around current capability and provenance data
- provenance depth and chain validation remain deterministic

## Sandbox Contract

Expected behavior from:

- `crates/mister-smith-security/src/sandbox.rs`
- `crates/mister-smith-agents/src/sandbox.rs`

The packet freezes:

- persistent and ephemeral agent classes as a boundary rule
- least-privilege subject reach by class
- explicit crossing rules for cross-account movement
- required cleanup of ephemeral credentials
- shared-state mediation rather than direct unrestricted access

Persistent-versus-ephemeral separation is a boundary-hardening rule here, not a broader redesign of
all runtime roles.

## Packet 016 Continuity Rule

Packet `024` preserves packet `016` truth:

- accepted delegated `POST /api/v1/tasks` continuity remains the landed baseline
- rejected delegated requests still do not imply a workflow-backed live reject surface unless the
  runtime actually grows one
- packet `024` may harden identity and delegation checks, but it may not invent a new continuity
  story from metadata alone

## Canonical Evidence Shape

Example authoritative shape:

```json
{
  "principal_id": "agent-123",
  "permission_tier": "restricted",
  "delegation_ref": {
    "descriptor_id": "tool:smith.resolve_issue_lifecycle",
    "action_id": "tool:smith.resolve_issue_lifecycle#execute"
  },
  "agent_class": "ephemeral",
  "fallback_applied": false,
  "continuity_ref": "packet-016-accepted-ingress"
}
```

Behavior:

- identity evidence must remain least-privilege
- fallback posture must be explicit
- continuity references must not fabricate a missing live reject surface

## Deferred

This contract does not freeze:

- enterprise IAM rollout
- SPIFFE implementation
- a broader trust-scoring program beyond the current auth-callout seam
- generic operator-facing identity dashboards

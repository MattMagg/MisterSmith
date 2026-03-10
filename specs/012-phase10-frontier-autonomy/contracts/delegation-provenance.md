# Contract: Delegation & Provenance

## Overview

The Delegation & Provenance contract defines bounded authority transfer for privileged autonomous
actions. It extends the Phase 9.1 delegation-chain substrate into enforceable execution-time
validation and operator-visible provenance.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` | Revocable, observable capability as autonomy prerequisite |
| `docs/research-output/consolidated/04-security-and-trust.md` | Capability/provenance requirements and zero-trust execution |
| `specs/011-phase9.1-security-hardening/spec.md` | Phase 9.1 delegation-chain foundation |

## Public API

```rust
pub trait DelegationService: Send + Sync {
    fn issue(
        &self,
        issuer: AgentId,
        recipient: AgentId,
        scope: DelegationScope,
        expiry: DateTime<Utc>,
        parent: Option<CapabilityId>,
    ) -> Result<DelegationCapability, DelegationError>;

    fn validate(
        &self,
        capability: &DelegationCapability,
        action: &DelegatedAction,
    ) -> Result<(), DelegationError>;

    fn revoke(&self, capability_id: CapabilityId) -> Result<(), DelegationError>;
}
```

## Capability Properties

1. A capability MUST record issuer, recipient, scope, expiry, and parent link.
2. A child capability MUST NOT outlive or outscope its parent.
3. Revoked or expired capability chains MUST invalidate downstream execution.
4. Every privileged action MUST be linkable to a terminal capability in a provenance chain.

## Behavioral Requirements

- Validation MUST reject cycles or broken parent links.
- Validation MUST reject expired or revoked chains before action execution.
- Provenance information MUST be surfaced to operator/autonomy views.
- Delegation must remain compatible with Phase 9.1 claims and Auth Callout substrate.

## Validation Requirements

- Fresh valid delegation chain allows execution.
- Revoked chain blocks execution.
- Expired chain blocks execution.
- Operator can reconstruct the chain for a completed privileged action.

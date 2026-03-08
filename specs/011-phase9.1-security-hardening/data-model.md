# Data Model: Phase 9.1 — Security Hardening

**Date**: 2026-03-07
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `docs/research-output/consolidated/04-security-and-trust.md` | Drives all security entity designs — AgentSandbox, Auth Callouts, quarantine patterns |
| `spec/data-management/agent-orchestration.md` | Flags MessageEnvelope security as "CRITICAL GAP" — grounds signature/nonce/capability_token fields |
| `spec/security/` | Existing SecurityLayer, JWT, RBAC patterns that Phase 9.1 extends |
| `spec/core-architecture/type-definitions.md` | Constrains error types and shared conventions |

## Entities

### MessageSigner

HMAC-SHA256 signing and verification for `MessageEnvelope` contents.

| Field / Method | Type | Constraints | Description |
| -------------- | ---- | ----------- | ----------- |
| `sign(envelope) -> signature` | `fn(&MessageEnvelope) -> String` | Required | Produces HMAC-SHA256 signature over envelope contents |
| `verify(envelope, signature) -> bool` | `fn(&MessageEnvelope, &str) -> bool` | Required | Verifies signature matches envelope contents |
| `generate_nonce() -> String` | `fn() -> String` | Monotonic, unique | Produces a monotonic nonce for replay prevention |
| active_key | `HmacKey` | Required | Current signing key |
| grace_keys | `Vec<HmacKey>` | Optional | Previous keys accepted during rotation grace period |
| nonce_window | `HashSet<String>` | Bounded | Recently-seen nonces for replay detection |
| nonce_window_size | `usize` | Default 10000 | Maximum nonces tracked before oldest are evicted |

**Invariant**: Signing uses symmetric HMAC-SHA256 (not RSA/EdDSA) — optimized for per-message
signing at high throughput. Asymmetric signatures are too expensive for inter-agent messages.

**Invariant**: Key rotation grace period accepts both `active_key` and all `grace_keys` during
verification. Grace keys are removed after the configured TTL.

---

### AuthCalloutHandler

NATS Auth Callout service that generates dynamic JWTs from agent trust profiles.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| trust_store | `HashMap<String, TrustProfile>` | Required | Per-agent trust profiles |
| signing_key | `nkeys::KeyPair` | Required | NATS account signing key for JWT generation |
| default_permissions | `Permissions` | Required | Minimal fallback permissions |
| max_jwt_ttl_secs | `u64` | Default 300 | Maximum JWT lifetime |

**Invariant**: The Auth Callout service runs as a separate Rust service, not embedded in the
main binary. This provides operational independence and limits blast radius.

**Invariant**: When the service is unavailable, agents receive `default_permissions` (minimal
access), not their last-known elevated permissions.

---

### TrustProfile

Per-agent behavioral trust assessment used by the Auth Callout service.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| agent_id | `String` | Required | Agent identifier |
| trust_score | `f64` | 0.0-1.0 | Current trust level (1.0 = fully trusted) |
| permission_tier | `PermissionTier` | Required | Mapped from trust_score to permission set |
| violation_count | `u32` | Required | Number of recorded security violations |
| last_assessment | `u64` | Epoch ms | Timestamp of last trust assessment |

---

### PermissionTier

Trust-to-permission mapping.

```text
#[non_exhaustive]
Full        — All authorized subjects (trust_score >= 0.9)
Standard    — Normal operational subjects (trust_score >= 0.5)
Restricted  — Limited subjects, shortened JWT TTL (trust_score >= 0.2)
Quarantined — Minimal subjects, very short TTL (trust_score < 0.2)
```

---

### StateValidator

Pluggable trait for schema-based data sanitization at persistence boundaries.

| Method | Signature | Description |
| ------ | --------- | ----------- |
| `validate` | `fn(&Value, &SchemaRef) -> Result<ValidatedState, ValidationError>` | Validate state against registered schema |
| `check_size` | `fn(&Value, max_bytes: usize) -> Result<(), ValidationError>` | Enforce size limits before schema validation |

**Invariant**: Size checking happens BEFORE schema validation to prevent resource exhaustion
from parsing extremely large payloads.

**Invariant**: `StateValidator` is a trait, not a concrete type — implementations can evolve
(JSON Schema, custom validators, ML-based detection) without changing the interface.

---

### ValidatedState

Result of successful state validation.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| data | `serde_json::Value` | Required | Validated state data |
| schema_version | `String` | Required | Schema version used for validation |
| taint_label | `TaintLabel` | Required | Classification of the validated data |

---

### TaintLabel

Classification tag for data passing through quarantine actors.

```text
#[non_exhaustive]
Clean       — Passed all validation checks
Sanitized   — Modified during sanitization (e.g., truncated, fields removed)
Suspicious  — Passed validation but flagged for monitoring
Rejected    — Failed validation — not forwarded to agent
```

---

### QuarantineAction

Decision enum for quarantine outcomes.

```text
#[non_exhaustive]
Pass        — Forward data unchanged
Sanitize    — Forward data after removing flagged content
Reject      — Block data entirely, return error to requester
Quarantine  — Store data in quarantine log for analysis, return error to requester
```

---

### IOFirewall

Boundary enforcement between persistent and ephemeral agent contexts.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| persistent_account | `String` | Required | NATS account for persistent agents |
| ephemeral_account | `String` | Required | NATS account for ephemeral agents |
| allowed_crossings | `Vec<CrossingRule>` | Required | Explicit rules for permitted cross-boundary communication |

**Invariant**: The I/O firewall uses NATS accounts (not just subject ACLs) for credential-level
isolation. NATS accounts provide true isolation — agents in different accounts cannot see each
other's subjects by default.

---

### CrossingRule

Explicit rule for permitted cross-boundary communication.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| source_account | `String` | Required | Source NATS account |
| target_account | `String` | Required | Target NATS account |
| subject_pattern | `String` | Required | NATS subject pattern permitted for crossing |
| requires_quarantine | `bool` | Default `true` | Whether data must pass through quarantine actor |

---

### AgentMemoryAccess

Classification of agent state access patterns for audit.

```text
#[non_exhaustive]
Read            — Agent reads its own state
Write           — Agent writes its own state
CrossBoundary   — Agent accesses state across persistent/ephemeral boundary
Shared          — Agent accesses shared/global state
```

---

### SandboxCredentials

Per-agent NATS credentials scoped by agent lifecycle.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| agent_id | `String` | Required | Agent identifier |
| agent_class | `AgentClass` | Required | `Persistent` or `Ephemeral` |
| nats_account | `String` | Required | NATS account assignment |
| nats_user | `String` | Required | NATS user within the account |
| jwt | `String` | Required | Dynamic JWT from Auth Callout |
| created_at | `u64` | Epoch ms | Credential creation time |
| expires_at | `u64` | Epoch ms | Credential expiration |

---

### AgentClass

Classification of agent lifecycle for sandbox assignment.

```text
#[non_exhaustive]
Persistent  — Stable identity, durable state, long-lived credentials
Ephemeral   — Short-lived, isolated, restricted permissions, auto-cleanup
```

## MessageEnvelope Security Fields

Phase 9.1 adds three fields to `MessageEnvelope` in `crates/mister-smith-transport/src/envelope.rs`:

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| signature | `Option<String>` | `#[serde(default)]` | HMAC-SHA256 signature over envelope contents |
| nonce | `Option<String>` | `#[serde(default)]` | Monotonic nonce for replay prevention |
| capability_token | `Option<String>` | `#[serde(default)]` | Fine-grained capability delegation token |

**Invariant**: All three fields use `Option<T>` with `#[serde(default)]` for backward
compatibility with pre-Phase-9.1 messages.

**Invariant**: `signature` and `nonce` are independent of `capability_token`.
Signing can be enabled without capability delegation.

## Relationships

```text
MessageSigner 1──* MessageEnvelope (signs)
AuthCalloutHandler 1──* TrustProfile
TrustProfile 1──1 PermissionTier
StateValidator 1──* ValidatedState
ValidatedState 1──1 TaintLabel
IOFirewall 1──* CrossingRule
IOFirewall 1──1 SandboxCredentials (per agent)
SandboxCredentials 1──1 AgentClass
QuarantineAction ──decides──> TaintLabel
```

## State Transitions

### Message Verification Flow

```text
Received -> SignatureCheck -> [valid] -> NonceCheck -> [fresh] -> Accepted
                           -> [invalid] -> Rejected (audit event)
                                        -> [replay] -> Rejected (audit event)
```

### Trust Assessment Flow

```text
Connected -> AuthCallout -> TrustProfile lookup -> PermissionTier mapping -> JWT generation -> Granted
                         -> Service unavailable -> Default permissions -> Granted (minimal)
```

### Data Quarantine Flow

```text
Retrieved -> SizeCheck -> [ok] -> SchemaValidation -> [valid] -> TaintLabel(Clean) -> Pass
                       -> [oversized] -> QuarantineAction(Reject) -> Error
                                      -> [invalid] -> QuarantineAction(Sanitize|Reject|Quarantine)
```

### Agent Sandbox Lifecycle

```text
Spawned -> ClassifyAgent -> [Persistent] -> PersistentAccount + DurableCredentials
                         -> [Ephemeral] -> EphemeralAccount + TempCredentials
Completed -> [Ephemeral] -> CleanupCredentials + CleanupState
          -> [Persistent] -> PersistState
```

## Validation Rules

1. `MessageSigner` must use HMAC-SHA256 (symmetric) — not RSA/EdDSA (too expensive per-message).
2. Nonce tracking window must be bounded to prevent memory exhaustion.
3. Auth Callout service must be operationally independent (separate process).
4. `StateValidator` must check size BEFORE schema validation.
5. NATS account isolation must be used for AgentSandbox — not just subject ACLs.
6. All cross-boundary data transfers must route through quarantine actors when
   `requires_quarantine = true`.
7. `AgentClaims.delegation_chain` must be validated for non-empty entries, maximum depth, and
   no circular references if retained.
8. Deploy artifacts must enforce NATS >= v2.11.1.
9. No agent configuration may include wildcard `>` or `$JS.>` permissions.

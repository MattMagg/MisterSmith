# Implementation Plan: Phase 9.1 — Security Hardening

**Branch**: `011-phase9.1-security-hardening` | **Date**: 2026-03-07 | **Spec**: [spec.md](spec.md)

## Summary

Phase 9.1 hardens the Mister Smith multi-agent framework against the security threats identified
by a 7-round research phase (2,000+ papers, 120+ security papers synthesized). It addresses 7
critical audit findings spanning inter-agent message authentication, data sanitization, agent
isolation, and infrastructure-level vulnerability mitigation.

The implementation extends the existing Phase 5 security infrastructure (`mister-smith-security`)
and Phase 6 persistence layer (`mister-smith-persistence`) while adding new security primitives
to `mister-smith-transport` (MessageEnvelope security fields) and `mister-smith-agents`
(AgentSandbox, quarantine actors).

## Technical Context

- **Language/Version**: Rust, MSRV 1.88.0
- **Primary Dependencies**: `ring` 0.17 (HMAC-SHA256), `nkeys` (NATS account signing),
  `jsonschema` (state validation), existing `async-nats` for Auth Callout protocol
- **Security Foundation**: Phase 5 `mister-smith-security` (JWT, RBAC, TLS/mTLS, audit logging)
- **Persistence Foundation**: Phase 6 `mister-smith-persistence` (AgentRepository, HybridStateManager)
- **Testing**: Unit tests for signing/verification, Auth Callout integration tests (NATS required),
  StateValidator schema tests, AgentSandbox isolation tests, quarantine actor tests
- **Constraints**: no ML-based detection, no Macaroon delegation, no eBPF observability

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | SecurityError stays in core, new types follow established patterns |
| II. Spec-First Design | PASS | Spec defines 7 audit findings, 6 user stories, 13 FRs |
| III. Phase-Gated Build Order | PASS | Phase 9.1 follows Phase 9 MessageEnvelope changes |
| IV. Model-Agnostic Architecture | PASS | Security is provider-neutral |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Quarantine actors use supervision, Auth Callout has fallback |
| VI. Evidence-Based Validation | PASS | All findings cite specific research papers with evidence levels |
| VII. Explicit Dependency Management | PASS | MessageEnvelope backward compatibility via Option fields |

## Project Structure

### Source Code

```text
crates/mister-smith-transport/
+-- src/envelope.rs                     # Add signature, nonce, capability_token to MessageEnvelope

crates/mister-smith-security/
+-- src/
|   +-- message_signer.rs              # MessageSigner trait, HMAC-SHA256, nonce generation (NEW)
|   +-- auth_callout.rs                # AuthCalloutHandler, TrustProfile, dynamic JWT gen (NEW)
|   +-- state_validator.rs             # StateValidator trait, schema validation (NEW)
|   +-- sandbox.rs                     # AgentSandbox, IOFirewall, SandboxCredentials (NEW)
|   +-- quarantine.rs                  # Quarantine actor, TaintLabel, QuarantineAction (NEW)
|   +-- jwt/claims.rs                  # Validate/propagate delegation_chain (MODIFY)
|   +-- middleware/nats_mw.rs          # Integrate MessageSigner into SecureTransport (MODIFY)
+-- tests/
    +-- signer_tests.rs                # Signing, verification, rotation, replay (NEW)
    +-- auth_callout_tests.rs          # Dynamic JWT, trust tiers, fallback (NEW)
    +-- validator_tests.rs             # Schema validation, size limits, taint labels (NEW)
    +-- sandbox_tests.rs               # Account isolation, credential lifecycle (NEW)
    +-- quarantine_tests.rs            # Cross-boundary inspection, clean/malicious data (NEW)

crates/mister-smith-persistence/
+-- src/repository/agent.rs            # Integrate StateValidator into get_state() (MODIFY)

crates/mister-smith-agents/
+-- src/
|   +-- sandbox.rs                     # AgentSandbox integration, agent class assignment (NEW)
|   +-- quarantine.rs                  # Quarantine actor for cross-boundary transfers (NEW)

deploy/
+-- nats-server.conf                   # Pin version >= v2.11.1, audit wildcards (MODIFY)
+-- docker-compose.yml                 # Pin NATS image version (MODIFY)
+-- kubernetes/                        # Update NATS manifests (MODIFY)
```

## Design Decisions

### D1: HMAC-SHA256 for Internal Messages

**Decision**: Symmetric signing for high-throughput inter-agent messages.
See [research.md](research.md) for full rationale.

### D2: Auth Callout as Separate Service

**Decision**: Operational independence from main binary.

### D3: StateValidator as Pluggable Trait

**Decision**: Schema implementations can evolve without interface changes.

### D4: NATS Accounts for Sandbox Isolation

**Decision**: Credential-level isolation, not just subject ACLs.

### D5: Delegation Chain Validated, Not Removed

**Decision**: Wire up validation and propagation for Phase 10 Macaroon foundation.

## Dependency Changes

### New Dependencies

- `jsonschema` — JSON Schema validation for `StateValidator`
- `nkeys` — NATS account signing (may already be transitive via `async-nats`)

### Existing Crates Touched

- `mister-smith-transport` — MessageEnvelope security fields
- `mister-smith-security` — MessageSigner, AuthCalloutHandler, StateValidator, sandbox, quarantine
- `mister-smith-persistence` — StateValidator integration in AgentRepository
- `mister-smith-agents` — AgentSandbox, quarantine actors
- Deploy artifacts — NATS version pinning, permission audit

## Subphase Execution Plan

### 9.1.1 Message Signing (Finding F1)

**Scope**: `MessageSigner` trait, HMAC-SHA256 signing/verification, nonce generation, replay
detection, key rotation with grace period, MessageEnvelope security field additions.

**Outputs**: Sign/verify round-trip tests, replay rejection tests, key rotation tests.

- **Depends on**: Phase 9 MessageEnvelope `plane`/`stream_class` fields stable
- **Crates**: `mister-smith-security`, `mister-smith-transport`

### 9.1.2 State Validation (Finding F3)

**Scope**: `StateValidator` trait, JSON Schema validation, size limits, taint labeling,
`AgentRepository` integration.

**Outputs**: Schema validation tests, size limit tests, taint label tests.

- **Depends on**: 9.1.1 (for MessageEnvelope field layout)
- **Crates**: `mister-smith-security`, `mister-smith-persistence`

### 9.1.3 Auth Callout Service (Finding F4)

**Scope**: `AuthCalloutHandler`, `TrustProfile`, `PermissionTier`, dynamic JWT generation,
minimal-permission fallback, NATS Auth Callout protocol implementation.

**Outputs**: Dynamic JWT tests, trust tier mapping tests, fallback behavior tests.

- **Depends on**: 9.1.1 (MessageSigner for signed JWTs)
- **Crates**: `mister-smith-security`

### 9.1.4 AgentSandbox (Finding F2)

**Scope**: Persistent/ephemeral agent classification, NATS account isolation, `SandboxCredentials`,
I/O firewall with `CrossingRule`.

**Outputs**: Account isolation tests, credential lifecycle tests, firewall tests.

- **Depends on**: 9.1.3 (Auth Callout for dynamic credentials)
- **Crates**: `mister-smith-security`, `mister-smith-agents`

### 9.1.5 Quarantine Actors (Finding F5)

**Scope**: Quarantine actor for cross-boundary data inspection, `QuarantineAction`,
`TaintLabel`, COWPOX-inspired edge monitoring patterns.

**Outputs**: Cross-boundary inspection tests, clean/malicious data tests, sub-millisecond
overhead for clean data.

- **Depends on**: 9.1.4 (AgentSandbox for cross-boundary context)
- **Crates**: `mister-smith-agents`, `mister-smith-security`

### 9.1.6 CVE Mitigation + Delegation Chain (Findings F6, F7)

**Scope**: NATS server version pinning in deploy artifacts, wildcard permission audit, ACL
review, `AgentClaims.delegation_chain` validation (non-empty, max depth, no cycles) and
propagation across agent boundaries.

**Outputs**: Deploy artifact verification, permission audit script, delegation chain
validation tests.

- **Depends on**: 9.1.1 (MessageEnvelope changes)
- **Crates**: `mister-smith-security` (jwt/claims), deploy artifacts

## Blockers and Deferred Work

### Prerequisites

- Phase 9 `MessageEnvelope` additions (`plane`, `stream_class`) must be stable
- Phase 5 security crate must be operational
- Phase 6 persistence crate must be operational

### Explicit Deferred Scope

- Macaroon-based capability delegation (Phase 10+)
- eBPF-based agent observability (Phase 12+)
- Progent DSL for capability policies (Phase 12+)
- ML-based behavioral anomaly detection (Phase 12+)
- Distributed backdoor correlation monitoring (Phase 14+)

## Complexity Tracking

No constitution violations. The security hardening extends existing Phase 5 patterns (JWT, RBAC,
audit) rather than replacing them. MessageEnvelope changes are backward-compatible via
`Option<T>` with `#[serde(default)]`.

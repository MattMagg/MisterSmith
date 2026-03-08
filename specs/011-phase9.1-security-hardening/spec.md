# Feature Specification: Phase 9.1 — Security Hardening

**Feature Branch**: `011-phase9.1-security-hardening`
**Created**: 2026-03-07
**Status**: Draft
**Input**: 5-crate security audit comparing existing implementation against 7-round research
findings (2,000+ papers), `docs/research-output/consolidated/04-security-and-trust.md`,
`specs/009-phase9-llm-provider-integration/spec.md` (Phase 9 MessageEnvelope changes).

## Scope & Traceability

### Governing Sources

1. `docs/research-output/consolidated/04-security-and-trust.md` — comprehensive security research
2. `docs/research-output/consolidated/00-MASTER-FINDINGS.md` — findings #6 and #7
3. `spec/data-management/agent-orchestration.md` — explicitly flags MessageEnvelope security as
   "CRITICAL GAP"
4. `spec/security/` — existing security architecture
5. Canonical architecture sources in `spec/`

### Research Grounding

Phase 9.1 is driven by 7 audit findings from comparing existing implementation against research:

| Finding | Source | Severity | Summary |
| ------- | ------ | -------- | ------- |
| **F1** | consolidated/04-security-and-trust.md | CRITICAL | Inter-agent message authentication absent — 97% ASR for orchestrator hijacking (COLM 2025) |
| **F2** | consolidated/04-security-and-trust.md | CRITICAL | No persistent/ephemeral agent separation — AgentSandbox reduces ASR from 58.8% to 4.34% (13x) |
| **F3** | consolidated/04-security-and-trust.md | CRITICAL | No data sanitization between persistence retrieval and agent consumption — infectious jailbreak vector |
| **F4** | consolidated/04-security-and-trust.md | HIGH | No NATS Auth Callout service for dynamic capability scoping |
| **F5** | consolidated/04-security-and-trust.md | HIGH | Infectious jailbreak defense absent — exponential propagation via shared memory (Agent Smith attack) |
| **F6** | CVE-2025-30215 | HIGH | NATS server version not pinned; JetStream Admin API flaw enables cross-account stream purge |
| **F7** | crate audit | MODERATE | `AgentClaims.delegation_chain` exists but is never validated or propagated — dead code |

### In Scope

- Inter-agent message authentication via `MessageSigner` trait (HMAC-SHA256)
- `MessageEnvelope` security fields: `signature`, `nonce`, `capability_token`
- NATS Auth Callout service for dynamic per-request capability scoping
- `StateValidator` trait for data quarantine between persistence and agent consumption
- AgentSandbox architecture: persistent/ephemeral agent separation with I/O firewall
- Quarantine actors for cross-boundary data transfers (infectious jailbreak defense)
- CVE-2025-30215 mitigation: NATS server version pinning >= v2.11.1, wildcard permission audit
- `AgentClaims.delegation_chain` validation and propagation (or removal if not viable)

### Explicitly Deferred

- Macaroon-based capability delegation (Phase 10+ — maps to delegation_chain enhancement)
- eBPF-based agent observability / AgentSight (Phase 12+)
- Progent DSL for capability policies (Phase 12+)
- Behavioral anomaly detection / ML-based security monitoring (Phase 12+)
- CRDT-based capability registry (Phase 13)
- Cross-agent behavioral correlation monitoring for distributed backdoor detection (Phase 14+)

### Prerequisites & Dependencies

- Phase 9 `MessageEnvelope` additions (`plane`, `stream_class`) must be stable before Phase 9.1
  adds security fields (`signature`, `nonce`, `capability_token`)
- Phase 5 `mister-smith-security` crate provides the foundation (JWT, RBAC, TLS, audit)
- Phase 6 `mister-smith-persistence` provides `AgentRepository` and `HybridStateManager`

## User Scenarios & Testing

### User Story 1 — Inter-Agent Message Authentication (Priority: P1)

A framework operator enables message signing so that all inter-agent messages carry HMAC-SHA256
signatures and monotonic nonces. Receiving agents verify signatures and reject forged or replayed
messages.

**Why this priority**: 97% ASR for inter-agent hijacking attacks when messages are unsigned
(COLM 2025). This is the most critical security gap.

**Independent Test**: Send a signed message between two agents, verify signature validation
succeeds. Send a forged message, verify rejection. Send a replayed message (same nonce), verify
rejection.

**Acceptance Scenarios**:

1. **Given** message signing is enabled, **When** an agent sends a message, **Then** the
   `MessageEnvelope` includes a valid `signature` (HMAC-SHA256) and a monotonic `nonce`.
2. **Given** a received message with a valid signature and fresh nonce, **When** the receiver
   verifies it, **Then** the message is accepted and processed normally.
3. **Given** a received message with an invalid or missing signature, **When** verification
   fails, **Then** the message is rejected with a typed security error and an audit event.
4. **Given** a received message with a previously-seen nonce, **When** replay detection triggers,
   **Then** the message is rejected as a replay attack.
5. **Given** key rotation, **When** the signing key changes, **Then** both old and new keys are
   accepted during a configurable grace period to avoid message loss.

---

### User Story 2 — NATS Auth Callout Service (Priority: P1)

A framework operator deploys a NATS Auth Callout service that dynamically scopes agent
capabilities based on behavioral trust signals. Instead of static JWTs, agents receive
dynamically-generated JWTs with permissions tailored to their current trust level.

**Why this priority**: Static RBAC cannot adapt to changing trust conditions. Auth Callouts
enable per-request capability scoping — a capability absent from all competing frameworks.

**Independent Test**: Configure a NATS server with Auth Callout pointing to the Mister Smith
auth service. Connect an agent, verify it receives dynamically-scoped permissions. Simulate
trust degradation, verify permissions are narrowed on the next connection.

**Acceptance Scenarios**:

1. **Given** a NATS server configured with Auth Callout, **When** an agent connects, **Then** the
   Auth Callout service generates a JWT with permissions based on the agent's trust profile.
2. **Given** an agent with degraded trust (e.g., repeated failures, suspicious output patterns),
   **When** it reconnects, **Then** the Auth Callout service issues a JWT with narrowed
   permissions (fewer subjects, shorter TTL).
3. **Given** the Auth Callout service is unavailable, **When** an agent connects, **Then** the
   agent receives a minimal-permission fallback JWT rather than full access.

---

### User Story 3 — Data Quarantine and State Validation (Priority: P1)

A framework developer retrieves agent state from persistence (PostgreSQL or JetStream KV) and
it passes through a `StateValidator` before entering the agent's working context. Invalid,
oversized, or potentially malicious data is quarantined.

**Why this priority**: Raw persistence retrievals passed directly to agent context enable
infectious jailbreak propagation and memory injection attacks (MINJA — 18 citations).

**Independent Test**: Store a valid agent state and an oversized/malformed state in persistence.
Retrieve both through the `StateValidator`. Verify valid state passes, invalid state is
quarantined with an audit event.

**Acceptance Scenarios**:

1. **Given** valid agent state in persistence, **When** retrieved through the `StateValidator`,
   **Then** the state passes validation and is available to the agent.
2. **Given** oversized state exceeding configured bounds, **When** retrieved, **Then** the state
   is quarantined and the agent receives a typed error.
3. **Given** state containing known malicious patterns, **When** retrieved, **Then** the state is
   rejected with an audit event describing the detected pattern.
4. **Given** a schema-registered state type, **When** the retrieved state does not match the
   schema, **Then** validation fails with a typed schema mismatch error.

---

### User Story 4 — AgentSandbox (Persistent/Ephemeral Separation) (Priority: P2)

A framework operator deploys agents with persistent/ephemeral separation. Persistent agents have
stable identities and durable state. Ephemeral agents are short-lived, isolated, and operate with
restricted permissions. An I/O firewall enforces boundaries between agent types.

**Why this priority**: AgentSandbox reduces ASR from 58.8% to 4.34% (13x improvement). This is
the single most effective defense architecture documented in the research.

**Independent Test**: Deploy a persistent agent and an ephemeral agent. Verify the ephemeral agent
cannot access the persistent agent's NATS subjects or state. Verify the I/O firewall blocks
unauthorized cross-boundary communication.

**Acceptance Scenarios**:

1. **Given** an ephemeral agent, **When** it attempts to subscribe to a persistent agent's NATS
   subjects, **Then** the subscription is denied by NATS account-level isolation.
2. **Given** a persistent agent, **When** it stores state, **Then** the state is durable and
   survives agent restarts.
3. **Given** an ephemeral agent, **When** its task completes or times out, **Then** all its state
   and NATS credentials are cleaned up.
4. **Given** cross-boundary data transfer (persistent -> ephemeral), **When** data crosses the
   I/O firewall, **Then** it passes through a quarantine actor for sanitization.

---

### User Story 5 — Infectious Jailbreak Defense (Priority: P2)

A framework operator deploys quarantine actors that monitor cross-boundary data transfers for
infectious jailbreak patterns. Contaminated data is isolated before it can propagate through the
agent network.

**Why this priority**: The "Agent Smith" attack achieves exponential propagation through shared
memory. COWPOX defense (ICML 2025 poster) provides the architectural pattern.

**Independent Test**: Inject a known malicious payload into shared state. Verify the quarantine
actor detects it before it reaches any agent's working context. Verify a clean payload passes
through without delay.

**Acceptance Scenarios**:

1. **Given** a cross-boundary data transfer, **When** the data passes through a quarantine actor,
   **Then** it is inspected for known malicious patterns before forwarding.
2. **Given** data containing a detected malicious payload, **When** the quarantine actor flags it,
   **Then** the data is quarantined, an audit event is emitted, and the target agent receives a
   sanitized or rejected response.
3. **Given** clean data, **When** it passes through the quarantine actor, **Then** the overhead
   is sub-millisecond (Rust schema validation at 645x speed of legacy validators).

---

### User Story 6 — CVE Mitigation and Delegation Chain (Priority: P2)

A framework operator deploys with NATS server >= v2.11.1, wildcard permissions are audited, and
the `AgentClaims.delegation_chain` field is either validated and propagated or removed.

**Why this priority**: CVE-2025-30215 enables cross-account JetStream stream destruction via
wildcard `>` and `$JS.>` permissions. The delegation chain is dead code that creates a false
sense of security.

**Acceptance Scenarios**:

1. **Given** deploy artifacts, **When** the NATS server is configured, **Then** the minimum
   version is >= v2.11.1 and deployment docs specify this requirement.
2. **Given** NATS authorization configuration, **When** permissions are reviewed, **Then** no
   agent has wildcard `>` or `$JS.>` permissions.
3. **Given** `AgentClaims.delegation_chain`, **When** a token is issued, **Then** the chain is
   validated (non-empty entries, maximum depth, no circular references) and propagated across
   agent boundaries, OR the field is removed entirely if validation is not viable in Phase 9.1.

## Edge Cases

- Signing key rotation happens while messages are in flight — grace period must handle both keys.
- Auth Callout service crashes during agent connection — minimal-permission fallback must not
  leave agents with elevated permissions from a cached previous JWT.
- `StateValidator` encounters a valid but extremely large state object that would exhaust agent
  memory — size limits must be enforced before schema validation.
- Ephemeral agent crashes before cleanup — orphaned NATS credentials and subjects must be cleaned
  up by the supervision system.
- Quarantine actor itself is compromised — quarantine must be a separate process or actor with
  minimal attack surface, not a library called within the agent.
- Two agents attempt simultaneous cross-boundary transfers — quarantine actors must handle
  concurrent sanitization without deadlock.
- `delegation_chain` contains circular references (A delegates to B delegates to A) — validation
  must detect and reject cycles.

## Requirements

### Functional Requirements

- **FR-101**: Phase 9.1 MUST implement a `MessageSigner` trait providing HMAC-SHA256 signing and
  verification for `MessageEnvelope` contents.
- **FR-102**: Phase 9.1 MUST add `signature: Option<String>`, `nonce: Option<String>`, and
  `capability_token: Option<String>` fields to `MessageEnvelope` with `#[serde(default)]`
  for backward compatibility.
- **FR-103**: Phase 9.1 MUST implement monotonic nonce generation and replay detection with a
  configurable window for nonce tracking.
- **FR-104**: Phase 9.1 MUST implement signing key rotation with a configurable grace period
  accepting both old and new keys.
- **FR-105**: Phase 9.1 MUST implement a NATS Auth Callout service as a separate Rust service
  that dynamically generates JWTs based on agent trust profiles.
- **FR-106**: The Auth Callout service MUST provide minimal-permission fallback when the service
  is unavailable, rather than full-access fallback.
- **FR-107**: Phase 9.1 MUST implement a `StateValidator` trait with pluggable schema validation
  that sanitizes all persistence-to-agent data flows.
- **FR-108**: The `StateValidator` MUST enforce size limits before schema validation and MUST
  quarantine invalid state with typed errors and audit events.
- **FR-109**: Phase 9.1 MUST implement AgentSandbox architecture with persistent/ephemeral agent
  separation using NATS accounts for credential-level isolation, not just subject ACLs.
- **FR-110**: Phase 9.1 MUST implement an I/O firewall between persistent and ephemeral agent
  boundaries, with all cross-boundary data transfers routed through quarantine actors.
- **FR-111**: Phase 9.1 MUST implement quarantine actors for cross-boundary data inspection,
  with sub-millisecond overhead for clean data.
- **FR-112**: Deploy artifacts MUST specify NATS server >= v2.11.1 and MUST NOT include wildcard
  `>` or `$JS.>` permissions in agent authorization configurations.
- **FR-113**: `AgentClaims.delegation_chain` MUST be either validated (non-empty entries, maximum
  depth, no cycles) and propagated across agent boundaries, or removed as dead code with a
  migration note.

### Key Entities

- **MessageSigner**: Trait for HMAC-SHA256 message signing and verification.
- **AuthCalloutHandler**: NATS Auth Callout service that generates dynamic JWTs from trust profiles.
- **StateValidator**: Pluggable trait for schema-based data sanitization at persistence boundaries.
- **IOFirewall**: Boundary enforcement between persistent and ephemeral agent contexts.
- **TaintLabel**: Classification tag for data passing through quarantine actors.
- **QuarantineAction**: Decision enum for quarantine outcomes (Pass, Sanitize, Reject, Quarantine).
- **AgentMemoryAccess**: Classification of agent state access patterns (Read, Write, CrossBoundary).
- **SandboxCredentials**: Per-agent NATS credentials scoped by agent lifecycle (persistent vs
  ephemeral).

## Success Criteria

- **SC-101**: Inter-agent messages carry HMAC-SHA256 signatures that prevent forged message
  acceptance.
- **SC-102**: Replay attacks using previously-valid nonces are rejected.
- **SC-103**: The Auth Callout service dynamically scopes agent permissions based on trust
  profiles.
- **SC-104**: No raw persistence retrieval reaches agent working context without passing through
  `StateValidator`.
- **SC-105**: Ephemeral agents cannot access persistent agent NATS subjects or state.
- **SC-106**: Cross-boundary data transfers pass through quarantine actors with sub-millisecond
  overhead for clean data.
- **SC-107**: Deploy artifacts enforce NATS >= v2.11.1 with no wildcard permissions.
- **SC-108**: `AgentClaims.delegation_chain` is either actively validated or removed.

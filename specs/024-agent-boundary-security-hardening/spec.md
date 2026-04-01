# Feature Specification: Agent-Boundary Security Hardening

**Feature Branch**: `024-agent-boundary-security-hardening`
**Created**: 2026-04-01
**Status**: Draft
**Input**: `docs/direction.md`, `docs/current-state.md`, `docs/packet-prep/README.md`,
`docs/packet-prep/024-agent-boundary-security-hardening.md`,
`docs/research-output/consolidated/04-security-and-trust.md`,
`docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`,
`docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`,
`specs/011-phase9.1-security-hardening/contracts/auth-callout.md`,
`specs/011-phase9.1-security-hardening/contracts/agent-sandbox.md`,
`specs/011-phase9.1-security-hardening/contracts/state-validator.md`, and the current runtime
seams in `crates/mister-smith-agents/src/tool_bus.rs`,
`crates/mister-smith-mcp/src/server.rs`,
`crates/mister-smith-mcp/src/compatibility.rs`,
`crates/mister-smith-security/src/delegation.rs`,
`crates/mister-smith-security/src/auth_callout.rs`,
`crates/mister-smith-security/src/quarantine.rs`,
`crates/mister-smith-security/src/state_validator.rs`,
`crates/mister-smith-security/src/sandbox.rs`,
`crates/mister-smith-agents/src/sandbox.rs`, and
`crates/mister-smith-persistence/src/repository/agent.rs`

## Draft Status And Revision Gate

This packet is draft scaffolding for future work.

- packet `024` is being scaffolded before earlier packets are fully complete
- claims are based on current repo truth and current dossiers
- before implementation, this packet MUST be revised against the then-current
  `docs/current-state.md`, `docs/direction.md`, and any newly landed earlier packet artifacts
- if earlier packet work changes reused contracts, packet `024` wins no authority over those
  contracts until revised

## Current Truth & Scope

Current repo truth already includes baseline behavior that this packet must reuse instead of
reopening:

- ToolBus capability descriptors already separate discover from execute actions and can require
  delegated authority per action
- the MCP boundary already has bounded discovery through `describe_external_capabilities`, plus
  descriptor-and-action-bound delegated invocation checks
- the current delegation layer already validates exact `DelegatedAction` bindings, revocation, and
  bounded external envelopes
- Phase 9.1 already landed auth callout, quarantine, sandbox, and state-validator contracts
- packet `016` already proved accepted delegated HTTP task-ingress continuity and explicitly kept
  live rejection proof out of scope because the current runtime does not create a workflow-backed
  reject surface

The remaining gap is narrower than generic security or interoperability work:

- the repo does not yet freeze one coherent least-privilege runtime contract across ToolBus, MCP,
  quarantine, schema enforcement, and identity boundaries
- the current boundary hardening posture is spread across multiple crates, tests, and closure notes
  instead of one bounded packet
- later packets will widen delegation, runtime coordination, or interop reach, so boundary rules
  should be frozen before that widening happens

This packet therefore freezes one bounded security-hardening packet with three stories:

1. least-privilege capability boundaries across ToolBus and MCP
2. quarantine and schema enforcement before agent consumption
3. identity, auth-callout, delegation continuity, and boundary evidence

This is not:

- a general IAM redesign
- a compliance or policy program
- a wider interoperability packet
- a new live rejection proof packet
- a broader runtime-truth, observability, or operator-console redesign packet

## Clarifications

### Session 2026-04-01

- Q: What identity posture is in scope for this draft packet? → A: Keep the current JWT,
  auth-callout, and delegation-envelope posture as the implementation baseline.
- Q: Is SPIFFE part of packet `024`? → A: No. SPIFFE stays comparator guidance only and is out of
  scope for this packet.
- Q: How far does persistent versus ephemeral separation go here? → A: It is frozen as a boundary
  rule for credentials, subject reach, and shared-state mediation, not widened into a broader IAM
  redesign.
- Q: Does packet `024` reopen packet `016` rejection proof? → A: No. Packet `016` keeps the
  current rule that live rejection proof stays out of scope unless a workflow-backed reject surface
  actually exists.
- Q: Which MCP sources are authoritative for this packet? → A: Use the MCP `2025-11-25`
  versioned pages as the protocol baseline and treat MCP security best-practices pages as
  operational guidance only.

## User Scenarios & Testing

### User Story 1 - Keep capability boundaries least-privilege and action-bound (Priority: P1)

An operator or framework maintainer needs ToolBus and MCP boundaries to keep discover permission
separate from execute permission and reject delegated authority that does not match the exact
descriptor and action being used.

**Why this priority**: later packets will widen delegation and external capability reach, so the
descriptor-and-action boundary needs to be frozen first.

**Independent Test**: targeted ToolBus and MCP tests prove that discover and execute stay separate,
that mismatched or revoked delegated actions are rejected, and that bounded discovery still works
without widening execution authority.

**Acceptance Scenarios**:

1. **Given** a caller lists or inspects capabilities, **When** it uses a discover path,
   **Then** the packet keeps discover permission separate from execute permission.
2. **Given** a caller invokes a ToolBus or MCP action with a mismatched descriptor, action, or
   revoked authority, **When** the boundary validates the request, **Then** the request is rejected
   before handler execution.
3. **Given** a caller uses the bounded MCP discovery surface, **When** it inspects the catalog,
   **Then** it sees the exact descriptor and action requirements without gaining execute authority.

---

### User Story 2 - Quarantine and validate cross-boundary content before agent use (Priority: P1)

An operator or framework maintainer needs all cross-boundary payloads and shared-state reads to
pass through deterministic quarantine and schema enforcement before they enter agent working
context.

**Why this priority**: the security research and current repo seams both show that agents cannot be
trusted to enforce this boundary for themselves.

**Independent Test**: targeted quarantine, validator, sandbox, and persistence tests prove that
clean payloads pass, sanitized payloads are marked, suspicious payloads are monitored, and rejected
or quarantined payloads do not reach agent context.

**Acceptance Scenarios**:

1. **Given** a cross-boundary payload that matches allowed structure and patterns, **When** it
   crosses the boundary, **Then** it is forwarded with the correct validation outcome.
2. **Given** a payload that violates size, schema, or malicious-pattern checks, **When** the
   validator inspects it, **Then** the packet defines whether the result is sanitize, reject, or
   quarantine and preserves the audit reason.
3. **Given** a shared-state read from persistence, **When** the state is returned to an agent,
   **Then** the read passes through the same validation and quarantine mediation before agent
   consumption.

---

### User Story 3 - Keep identity, auth-callout, and delegation continuity bounded (Priority: P2)

An operator or framework maintainer needs least-privilege identity and delegation rules to stay
bounded across auth callout, sandbox credentials, and the already-landed packet `016` continuity
surface.

**Why this priority**: packet `024` should harden the existing identity boundary without widening
into a new identity program or breaking packet `016` truth.

**Independent Test**: targeted auth-callout, delegation, sandbox, and packet `016` continuity
checks prove that least-privilege credentials stay narrow, quarantined fallback remains minimal,
and packet `016` continuity rules remain intact.

**Acceptance Scenarios**:

1. **Given** an authenticated boundary principal with current delegated authority, **When** auth
   callout or sandbox credentials are issued, **Then** permissions stay least-privilege and
   revocation remains enforceable.
2. **Given** a low-trust, missing-profile, or unavailable-auth-callout path, **When** fallback
   applies, **Then** the result stays on the current minimal quarantined posture rather than a
   broader default.
3. **Given** the packet `016` accepted task-ingress path, **When** packet `024` tightens identity
   and delegation rules, **Then** it preserves accepted-ingress continuity and does not invent a
   workflow-backed live reject surface.

## Edge Cases

- a caller has discover delegation for a capability but tries to execute it
- descriptor identifiers match while the action identifier or revocation key does not
- a payload is schema-valid but still contains known malicious markers
- sanitization changes object keys or payload shape in a way that creates conflicts
- a cross-boundary payload requires quarantine, but no quarantine actor is attached
- a shared-state read returns structurally valid but stale or suspicious content
- auth callout is unavailable and fallback permissions must remain minimal
- persistent and ephemeral agents need to communicate across an allowed rule, but only through
  quarantine
- earlier packet work changes a reused contract before packet `024` implementation begins

## Requirements

### Functional Requirements

- **FR-001**: System MUST keep discover and execute permissions separate across ToolBus, MCP
  `tools/list`, MCP `tools/call`, and the future adapter contracts frozen by this packet.
- **FR-002**: System MUST bind delegated authority to the exact descriptor and action required at
  the ToolBus and MCP call boundary rather than descriptor-only matching.
- **FR-003**: System MUST treat MCP `2025-11-25` versioned pages as the protocol baseline for this
  packet and MUST treat MCP security best-practices pages as operational guidance only.
- **FR-004**: System MUST preserve packet `016` accepted delegated HTTP task-ingress continuity as
  existing repo truth and MUST NOT invent a workflow-backed live rejection surface.
- **FR-005**: System MUST keep the current JWT, auth-callout, and delegation-envelope posture as
  the identity baseline for this packet and MUST NOT widen into a new SPIFFE rollout or generic
  IAM program.
- **FR-006**: System MUST freeze persistent-versus-ephemeral agent separation as a boundary rule
  for credentials, subject reach, and shared-state mediation.
- **FR-007**: System MUST route cross-boundary payloads through quarantine inspection whenever the
  crossing rule requires quarantine.
- **FR-008**: System MUST apply deterministic size, schema, and malicious-pattern validation before
  shared-state or cross-boundary payloads enter agent working context.
- **FR-009**: System MUST define explicit pass, sanitize, reject, and quarantine outcomes, plus
  the audit reason and monitoring posture for each non-pass case.
- **FR-010**: System MUST preserve revocation, audit, and no-fabrication rules as first-class
  runtime facts rather than optional logs.
- **FR-011**: System MUST keep packet scope bounded to ToolBus, MCP, delegation, quarantine, auth
  callout, schema enforcement, and identity boundaries.
- **FR-012**: System MUST NOT widen into generic IAM, compliance, broader interop design,
  operator-console redesign, or unrelated observability work.
- **FR-013**: System MUST carry a pre-implementation refresh gate that requires re-reading
  `docs/current-state.md`, `docs/direction.md`, the packet dossier, and any newly landed earlier
  packet artifacts before implementation starts.
- **FR-014**: System MUST tie major packet claims to the exact repo anchors named in this packet,
  its contracts, and its future implementation tasks.
- **FR-015**: System MUST preserve the current deny-wins, least-privilege, and quarantined fallback
  posture already present in the current authorization and auth-callout seams.

### Key Entities

- **BoundaryCapabilityDescriptor**: stable capability surface description for ToolBus or MCP that
  distinguishes discover and execute actions.
- **BoundaryActionBinding**: the exact descriptor, action, scope, and revocation key tuple required
  to authorize one boundary crossing or tool invocation.
- **BoundaryCredentialLease**: current least-privilege identity material issued through auth
  callout or sandbox credentials, including TTL, subject reach, and fallback posture.
- **QuarantineInspectionRecord**: deterministic inspection result carrying the action, taint label,
  monitoring flag, reason, and any detected malicious pattern.
- **ValidatedSharedState**: shared-state payload that has passed size, schema, and malicious-pattern
  checks before agent consumption.
- **BoundaryEvidenceSummary**: operator-facing and audit-facing summary of boundary decisions,
  revocation, and continuity facts that must not fabricate state from raw metadata alone.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The packet remains bounded to ToolBus, MCP, delegation, quarantine, auth callout,
  schema enforcement, and identity boundaries, with no broader IAM or interop work pulled in.
- **SC-002**: Discover-versus-execute separation is explicit in the packet spec, contracts, and
  tasks for every relevant boundary surface.
- **SC-003**: The packet carries a visible draft-status note and revision-before-implementation gate
  in `spec.md`, `plan.md`, and `tasks.md`.
- **SC-004**: Every major packet claim maps to one or more exact repo anchors already named in the
  dossier and frozen in the packet artifacts.
- **SC-005**: The packet preserves packet `016` continuity and does not claim a new workflow-backed
  live rejection surface.
- **SC-006**: The packet defines deterministic quarantine and schema-enforcement outcomes for clean,
  sanitized, suspicious, rejected, and quarantined boundary content.
- **SC-007**: The packet `024` task plan maps every functional requirement to at least one future
  implementation task and the packet analysis reports no critical internal contradictions.

## Assumptions

- earlier packets may still change reused contracts or truth-status details before packet `024`
  implementation begins
- current security, delegation, and MCP boundary seams remain the authoritative baseline until a
  later landed packet explicitly changes them
- packet `024` is being authored now to reduce future startup time, not because it is immediately
  implementation-ready
- no new live runtime proof claim is created by this scaffolding pass

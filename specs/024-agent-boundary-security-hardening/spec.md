# Feature Specification: Agent-Boundary Security Hardening

**Feature Branch**: `024-agent-boundary-security-hardening`
**Created**: 2026-04-01
**Status**: Implementation-ready
**Input**: `docs/current-state.md`, `docs/direction.md`,
`docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`,
`docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`,
`specs/022-durable-workflow-core/`,
`specs/011-phase9.1-security-hardening/contracts/auth-callout.md`,
`specs/011-phase9.1-security-hardening/contracts/agent-sandbox.md`,
`specs/011-phase9.1-security-hardening/contracts/state-validator.md`, and the current runtime
seams in `crates/mister-smith-agents/src/tool_bus.rs`,
`crates/mister-smith-mcp/src/client.rs`,
`crates/mister-smith-mcp/src/bridge.rs`,
`crates/mister-smith-mcp/src/server.rs`,
`crates/mister-smith-mcp/src/compatibility.rs`,
`crates/mister-smith-security/src/delegation.rs`,
`crates/mister-smith-security/src/auth_callout.rs`,
`crates/mister-smith-security/src/quarantine.rs`,
`crates/mister-smith-security/src/state_validator.rs`,
`crates/mister-smith-security/src/sandbox.rs`,
`crates/mister-smith-agents/src/sandbox.rs`, and
`crates/mister-smith-persistence/src/repository/agent.rs`

## Current Truth And Scope

Current `main` already includes the baseline behavior that packet `024` must keep and tighten,
not reopen:

- ToolBus already models capability descriptors with separate discover and execute actions in
  `crates/mister-smith-agents/src/tool_bus.rs`.
- The MCP boundary already has bounded discovery through `describe_external_capabilities`, plus
  descriptor-and-action-bound execution checks in `crates/mister-smith-mcp/src/server.rs` and
  `crates/mister-smith-mcp/src/compatibility.rs`.
- The delegation layer already validates exact `DelegatedAction` bindings, revocation, expiry,
  and bounded external envelopes in `crates/mister-smith-security/src/delegation.rs`.
- Phase 9.1 already landed auth callout, quarantine, sandbox, and state-validator contracts.
- Packet `016` already proved accepted delegated HTTP task-ingress continuity and explicitly kept
  live rejection proof out of scope because the runtime does not create a workflow-backed reject
  surface.
- The clean packet-024 worktree is based on `origin/main`, but the primary checkout now carries
  the newer user-owned packet-022 truth in `docs/current-state.md` and
  `docs/plans/2026-04-01-packet-022-durable-workflow-core.md`; packet `024` uses that durable
  runtime baseline without rewriting packet `022` in this lane.

The remaining packet `024` gaps are narrow and repo-grounded:

- ToolBus and MCP execution still preserve a legacy descriptorless delegation allowance that should
  no longer authorize action-bound execute paths.
- MCP capability metadata still collapses the surface to one action summary instead of publishing
  both discover and execute actions as first-class metadata.
- Quarantine evidence does not consistently explain sanitized or monitored pass-through outcomes in
  a human-readable way.
- Auth-callout fallback can be overridden through `with_default_permissions`, but packet `024`
  requires the fallback ceiling to stay no broader than quarantined access.

Packet `024` therefore owns exactly three stories:

1. least-privilege capability boundaries across ToolBus and MCP
2. quarantine and schema enforcement before agent consumption
3. identity, auth-callout, sandbox, delegation continuity, and boundary evidence

Packet `024` does not own:

- generic IAM redesign
- SPIFFE rollout work
- broader interoperability design
- runtime truth or run-trace ownership
- packet `022` durable semantics ownership
- a new live rejection-proof packet unless the repo really grows a workflow-backed reject surface

## Clarifications

### Session 2026-04-01

- Q: What identity posture is in scope for packet `024`? → A: Keep the current JWT,
  auth-callout, and delegation-envelope posture as the implementation baseline.
- Q: Is SPIFFE part of packet `024`? → A: No. SPIFFE stays comparator guidance only and is out of
  scope for this packet.
- Q: How far does persistent versus ephemeral separation go here? → A: It stays a boundary rule
  for credentials, subject reach, and shared-state mediation, not a broader IAM redesign.
- Q: Does packet `024` reopen packet `016` rejection proof? → A: No. Packet `016` keeps the
  current rule that live rejection proof stays out of scope unless a workflow-backed reject
  surface actually exists.
- Q: Which MCP sources are authoritative for this packet? → A: Use the MCP `2025-11-25`
  versioned pages as the protocol baseline and treat MCP security best-practices pages as
  operational guidance only.

## User Scenarios And Testing

### User Story 1 - Keep capability boundaries least-privilege and action-bound (Priority: P1)

An operator or framework maintainer needs ToolBus and MCP boundaries to keep discover permission
separate from execute permission and reject delegated authority that does not match the exact
descriptor and action being used.

**Why this priority**: later packets widen delegation and external capability reach, so the
descriptor-and-action boundary has to stay strict now.

**Independent Test**: targeted ToolBus and MCP tests prove that discover and execute stay
separate, mismatched or revoked delegated actions are rejected, descriptorless legacy capabilities
do not authorize execute paths, and bounded discovery still works without widening execution
authority.

**Acceptance Scenarios**:

1. **Given** a caller lists or inspects capabilities, **When** it uses a discover path,
   **Then** discover permission stays separate from execute permission.
2. **Given** a caller invokes a ToolBus or MCP action with a mismatched descriptor, wrong action,
   revoked authority, or missing descriptor binding, **When** the boundary validates the request,
   **Then** the request is rejected before handler execution.
3. **Given** a caller uses the bounded MCP discovery surface, **When** it inspects the catalog,
   **Then** it sees both discover and execute action requirements without gaining execute
   authority.

---

### User Story 2 - Quarantine and validate cross-boundary content before agent use (Priority: P1)

An operator or framework maintainer needs all cross-boundary payloads and shared-state reads to
pass through deterministic quarantine and schema enforcement before they enter agent working
context.

**Why this priority**: the security research and current repo seams both show that agents cannot be
trusted to enforce this boundary for themselves.

**Independent Test**: targeted quarantine, validator, sandbox, and persistence tests prove that
clean payloads pass, sanitized payloads are marked with a reason, suspicious payloads stay
monitored with a reason, and rejected or quarantined payloads do not reach agent context.

**Acceptance Scenarios**:

1. **Given** a cross-boundary payload that matches allowed structure and patterns, **When** it
   crosses the boundary, **Then** it is forwarded with the correct validation outcome.
2. **Given** a payload that violates size, schema, or malicious-pattern checks, **When** the
   validator inspects it, **Then** the result is sanitize, reject, or quarantine and the audit
   reason is preserved.
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
checks prove that least-privilege credentials stay narrow, fallback remains capped at the
quarantined ceiling, and packet `016` continuity rules remain intact.

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
- a capability has the right scope but no descriptor binding on an execute path
- a payload is schema-valid but still contains known malicious markers
- sanitization changes object keys or payload shape in a way that creates conflicts
- a cross-boundary payload requires quarantine, but no quarantine actor is attached
- a shared-state read returns structurally valid but stale or suspicious content
- auth callout is unavailable and fallback permissions must remain minimal
- persistent and ephemeral agents need to communicate across an allowed rule, but only through
  quarantine

## Requirements

### Functional Requirements

- **FR-001**: System MUST keep discover and execute permissions separate across ToolBus, MCP
  `tools/list`, MCP `tools/call`, and the external capability discovery contract.
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
- **FR-008**: System MUST apply deterministic size, sanitization, schema, and malicious-pattern
  validation before shared-state or cross-boundary payloads enter agent working context.
- **FR-009**: System MUST define explicit pass, sanitize, reject, and quarantine outcomes, plus
  the audit reason and monitoring posture for each non-clean case.
- **FR-010**: System MUST preserve revocation, audit, and no-fabrication rules as first-class
  runtime facts rather than optional logs.
- **FR-011**: System MUST reject action-bound execute paths when the delegated capability does not
  carry the matching descriptor binding.
- **FR-012**: System MUST publish both discover and execute actions in MCP capability metadata and
  in `describe_external_capabilities` output.
- **FR-013**: System MUST cap auth-callout fallback at the quarantined permission ceiling even if
  a caller configures broader default permissions.
- **FR-014**: System MUST keep packet scope bounded to ToolBus, MCP, delegation, quarantine, auth
  callout, schema enforcement, and identity boundaries.
- **FR-015**: System MUST NOT widen into generic IAM, compliance, broader interop design,
  operator-console redesign, or unrelated observability work.
- **FR-016**: System MUST tie major packet claims to the exact repo anchors named in this packet,
  its contracts, and its task plan.

### Key Entities

- **BoundaryCapabilityDescriptor**: stable capability surface description for ToolBus or MCP that
  distinguishes discover and execute actions.
- **BoundaryActionBinding**: the exact descriptor, action, scope, and revocation key tuple required
  to authorize one boundary crossing or tool invocation.
- **BoundaryCredentialLease**: current least-privilege identity material issued through auth
  callout or sandbox credentials, including TTL, subject reach, and fallback posture.
- **QuarantineInspectionRecord**: deterministic inspection result carrying the action, taint label,
  monitoring flag, reason, and any detected malicious pattern.
- **ValidatedSharedState**: shared-state payload that has passed size, schema, sanitization, and
  malicious-pattern checks before agent consumption.
- **BoundaryEvidenceSummary**: operator-facing and audit-facing summary of boundary decisions,
  revocation, and continuity facts that must not fabricate state from raw metadata alone.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The packet remains bounded to ToolBus, MCP, delegation, quarantine, auth callout,
  schema enforcement, and identity boundaries, with no broader IAM or interop work pulled in.
- **SC-002**: Discover-versus-execute separation is explicit in the packet spec, contracts, MCP
  metadata shape, and task plan for every relevant boundary surface.
- **SC-003**: Descriptorless legacy capabilities no longer authorize action-bound execute paths in
  ToolBus or MCP enforcement tests.
- **SC-004**: Every major packet claim maps to one or more exact repo anchors named in the packet
  artifacts.
- **SC-005**: The packet preserves packet `016` continuity and does not claim a new
  workflow-backed live rejection surface.
- **SC-006**: The packet defines deterministic quarantine and schema-enforcement outcomes for
  clean, sanitized, suspicious, rejected, and quarantined boundary content, including audit
  reasons.
- **SC-007**: The packet `024` task plan maps every functional requirement to at least one
  implementation task and the packet analysis reports no critical internal contradictions.

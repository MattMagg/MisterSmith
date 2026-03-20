# Feature Specification: External-Agent Boundary Continuity And Runtime Proof

**Feature Branch**: `016-external-agent-boundary-continuity-and-runtime-proof`  
**Created**: 2026-03-20  
**Status**: Draft  
**Input**: `docs/plans/2026-03-20-ms-96-external-agent-pre-spec-decision.md`,
`docs/plans/2026-03-19-central-development-checkpoint.md`,
`docs/current-state.md`,
`docs/ms_recent_context.md`,
`docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`,
`docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md`,
`docs/plans/2026-03-20-ms-95-post-merge-re-evaluation.md`, and current runtime/operator code in
`crates/mister-smith-http/`, `crates/mister-smith-app/`, `crates/mister-smith-agents/`,
`crates/mister-smith-events/`, and `crates/mister-smith-security/`

## Current Truth & Scope

This packet formalizes the bounded post-`MS-77` follow-on after packet `015` and `MS-95`.

Current repo truth on `main` already includes baseline behavior that this packet must not reopen:

- bounded delegation capability issuance, provenance, and revocation checks
- the bounded MCP discovery and enforcement surface from `MS-77`
- persisted raw `external_delegation` context in workflow metadata
- operator-visible `external_capability_decisions` for the bounded MCP and ToolBus boundary
- workflow-level autonomy inspection through `GET /api/v1/autonomy/status/{workflow_id}`
- packet `015` plus `MS-95` closure for result-surface and failure-visible autonomy parity

The remaining gap is narrower than generic external-agent interoperability:

- accepted delegated HTTP task ingress via `POST /api/v1/tasks` is not yet carried through
  persisted workflow metadata and projected onto workflow-level autonomy status as a first-class
  operator-visible boundary decision with preserved provenance and policy continuity

This packet therefore defines one bounded epic with three stories:

1. accepted delegated HTTP task ingress retains enough persisted workflow metadata to support a
   first-class operator-visible boundary decision later in the workflow
2. workflow-level autonomy inspection and CLI parity surface that accepted ingress decision without
   fabricating decisions from metadata-only delegation context
3. the packet proves one accepted live ingress path and keeps deterministic rejection coverage in
   scope without widening into live rejection proof unless a workflow-backed reject surface already
   exists

This is not a reopening of packet `015`, not a new MCP program, not a broader HTTP-ingress sweep,
and not provider, router, budget, JetStream KV, A2A, mesh, CRDT, or MPST work.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Persist Accepted Delegated HTTP Task-Ingress Continuity (Priority: P1)

An operator or external caller submits one delegated HTTP task request through `POST /api/v1/tasks`
and the accepted workflow retains enough metadata to prove that the request crossed the boundary
with preserved provenance and policy continuity.

**Why this priority**: the unresolved gap starts at accepted delegated task ingress, not at MCP
discovery and not at generic external protocols.

**Independent Test**: submit one accepted delegated task request, capture the returned
`workflow_id`, and verify that persisted workflow metadata preserves the accepted boundary context
without fabricating an operator-visible decision from raw metadata alone.

**Acceptance Scenarios**:

1. **Given** an accepted delegated `POST /api/v1/tasks` request, **When** the workflow record is
   persisted, **Then** the record retains the delegated boundary context needed for later
   workflow-level inspection.
2. **Given** that same accepted request, **When** later projections are built, **Then** the
   runtime can distinguish real accepted ingress continuity from raw stored delegation metadata.
3. **Given** a workflow created without accepted delegated ingress, **When** stored metadata is
   inspected, **Then** no accepted ingress decision is fabricated.

---

### User Story 2 - Surface Accepted Ingress Decisions On Workflow-Level Autonomy Inspection (Priority: P1)

An operator inspects the accepted workflow through the supported status route and CLI and sees a
first-class operator-visible boundary decision with preserved provenance and policy continuity.

**Why this priority**: the active operator surface is workflow-level autonomy inspection, so the
packet must close the continuity gap there rather than inventing a parallel surface.

**Independent Test**: inspect one accepted delegated workflow through
`GET /api/v1/autonomy/status/{workflow_id}` and
`mister-smith autonomy status --workflow-id ...`, and verify they surface the same accepted
boundary decision.

**Acceptance Scenarios**:

1. **Given** an accepted delegated task-ingress workflow, **When**
   `GET /api/v1/autonomy/status/{workflow_id}` is inspected, **Then** the response includes one
   first-class operator-visible accepted boundary decision with preserved provenance and policy
   continuity.
2. **Given** the same workflow, **When** CLI autonomy status is rendered, **Then** the same
   accepted decision is visible without requiring raw metadata inspection.
3. **Given** existing retained session continuity rules, **When** the accepted workflow later
   intersects session-facing continuity, **Then** the packet preserves those rules and does not
   relabel task or session views as autonomy-status surfaces.

---

### User Story 3 - Prove The Accepted Live Path And Keep Rejection Proof Deterministic (Priority: P2)

An operator can prove the accepted delegated task-ingress path end to end while keeping rejection
coverage bounded and deterministic.

**Why this priority**: the packet needs honest runtime proof, but live rejection proof should not
be invented if the runtime does not already expose a workflow-backed reject surface.

**Independent Test**: run one accepted delegated live task-ingress call, capture the returned
`workflow_id`, inspect workflow-level autonomy status and CLI parity, and separately run
deterministic rejection coverage for missing, wrong-route, revoked, or mismatched delegated
authority.

**Acceptance Scenarios**:

1. **Given** one accepted delegated task-ingress call, **When** live runtime proof is captured,
   **Then** the packet records the returned `workflow_id`, the workflow-level status response, and
   the CLI status output.
2. **Given** deterministic rejection cases, **When** validation runs, **Then** missing, revoked,
   wrong-route, or mismatched authority stays covered without widening into live rejection proof.
3. **Given** research finds no workflow-backed reject surface on `main`, **When** packet scope is
   finalized, **Then** live rejection proof remains explicitly out of scope.

### Edge Cases

- accepted delegated ingress persists raw `external_delegation` but still lacks a distinguishable
  operator-visible accepted boundary decision
- the accepted ingress decision collides with existing outbound ToolBus decision summaries
- workflow-level autonomy status and CLI render different decision/provenance summaries for the
  same `workflow_id`
- retained session continuity rules accidentally fabricate an accepted decision from raw metadata
- deterministic rejection tests pass, but no workflow-backed reject surface exists for honest live
  proof
- session routes also accept delegation, but this packet must stay frozen around `POST /api/v1/tasks`

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST treat `MS-77` as landed baseline and MUST NOT reopen the bounded MCP
  discovery or enforcement surface in this packet.
- **FR-002**: System MUST freeze the packet around accepted delegated HTTP task ingress via
  `POST /api/v1/tasks`.
- **FR-003**: System MUST preserve raw delegated ingress context in persisted workflow metadata as
  existing baseline truth rather than new scope.
- **FR-004**: System MUST carry accepted delegated task-ingress continuity from persisted workflow
  metadata onto workflow-level autonomy inspection as a first-class operator-visible boundary
  decision with preserved provenance and policy continuity.
- **FR-005**: System MUST use `GET /api/v1/autonomy/status/{workflow_id}` as the active HTTP
  inspection contract for this packet.
- **FR-006**: System MUST require CLI parity through
  `mister-smith autonomy status --workflow-id ...`.
- **FR-007**: System SHOULD prefer reusing `external_capability_decisions` for the accepted
  ingress decision surface.
- **FR-008**: System MUST NOT freeze a new operator-visible JSON contract up front; a
  backward-compatible discriminator or shape extension is allowed only if research proves the
  current summary cannot distinguish ingress decisions from outbound ToolBus decisions without
  ambiguity.
- **FR-009**: System MUST preserve the current rule that raw metadata-only delegation context does
  not fabricate an allowed or rejected boundary decision.
- **FR-010**: System MUST keep deterministic rejection tests in scope for missing, wrong-route,
  revoked, or mismatched delegated authority.
- **FR-011**: System MUST keep live rejection proof out of scope unless a real workflow-backed
  reject surface already exists on `main`.
- **FR-012**: System MUST preserve packet `015` plus `MS-95` closure as baseline truth and MUST
  NOT reopen the prior failure-visible result-surface gap.
- **FR-013**: System MUST use precise surface language: workflow metadata, retained session
  continuity, and workflow-level autonomy status.
- **FR-014**: System MUST keep provider, router, budget, JetStream KV, A2A, mesh, CRDT, and MPST
  work explicitly out of scope.
- **FR-015**: System MUST define explicit write-set boundaries for future `[P]` tasks so ingress
  continuity, operator projection, and proof lanes only run in parallel when their files are
  disjoint.

### Key Entities *(include if feature involves data)*

- **PersistedExternalDelegationContext**: the already-landed raw delegated ingress context stored
  in workflow metadata.
- **ExternalCapabilityDecisionSummary**: the preferred existing operator-visible summary surface
  for accepted ingress continuity, pending research on whether it can distinguish ingress decisions
  from outbound ToolBus decisions without ambiguity.
- **IngressBoundaryProofRun**: one durable live-proof record tying the accepted delegated request,
  returned `workflow_id`, workflow-level status response, CLI output, and artifact path together.
- **RetainedSessionContinuity Rule**: the existing rule that session-facing continuity must not
  fabricate accepted or rejected boundary decisions from raw stored delegation context alone.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The packet remains bounded to delegated HTTP task ingress, workflow metadata,
  retained session continuity rules, and workflow-level autonomy inspection.
- **SC-002**: The packet does not reopen packet `015`, `MS-77`, or any broader external-agent
  program.
- **SC-003**: The packet defines one accepted-ingress live proof path:
  `POST /api/v1/tasks` -> returned `workflow_id` ->
  `GET /api/v1/autonomy/status/{workflow_id}` ->
  `mister-smith autonomy status --workflow-id ...`.
- **SC-004**: The packet keeps deterministic rejection coverage in scope and explicitly keeps live
  rejection proof out of scope unless a workflow-backed reject surface is proven to exist.
- **SC-005**: The packet records whether `external_capability_decisions` can be reused as-is for
  accepted ingress continuity or needs a minimal backward-compatible discriminator.
- **SC-006**: The packet preserves the no-fabrication rule for metadata-only delegation context.

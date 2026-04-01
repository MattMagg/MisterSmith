# Feature Specification: Capability Discovery And Interoperability

**Feature Branch**: `027-capability-discovery-and-interoperability`  
**Created**: 2026-04-01  
**Status**: Draft  
**Input**: `docs/direction.md`, `docs/current-state.md`,
`specs/022-durable-workflow-core/spec.md`,
`specs/023-runtime-truth-and-run-trace/spec.md`,
`specs/024-agent-boundary-security-hardening/spec.md`,
`docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`, and
the current capability and status seams in `crates/mister-smith-agents/src/tool_bus.rs`,
`crates/mister-smith-mcp/src/server.rs`, `crates/mister-smith-mcp/src/client.rs`,
`crates/mister-smith-mcp/src/compatibility.rs`, and `crates/mister-smith-core/src/autonomy.rs`

## Current Truth & Scope

This packet is being written now as scaffolding so later implementation can move faster. It is not
immediate implementation approval.

Current repo truth already includes:

- bounded local capability descriptors in `ToolBus`
- bounded MCP catalog exposure and delegated discovery enforcement from `MS-77`
- packet `016` accepted delegated-ingress continuity and operator-visible provenance
- workflow, task, session, and autonomy identifiers that can support later cross-boundary mapping

What the repo does not yet have as one frozen contract is narrower than generic federation:

- one shared capability normalization model across local tools, MCP, and one remote protocol input
- one explicit lifecycle mapping from a remote task model into Mister Smith workflow, result, and
  autonomy surfaces
- one pinned interoperability baseline that does not drift across mixed protocol pages

This packet therefore freezes one bounded next step only:

1. normalize capability discovery metadata across local ToolBus entries, MCP catalog entries, and
   one A2A `v0.3.0` agent-card input path
2. map the A2A `v0.3.0` task lifecycle into Mister Smith workflow, result, and autonomy surfaces
   using existing lifecycle, proof-boundary, and security contracts as inherited inputs
3. keep discovery metadata, execution permission, and operator trust as separate concepts

This packet is scaffolded from the current finished packet outputs for packets `022`, `023`, and
`024`.
Packet `027` still MUST be revised before implementation so its contracts line up with any later
changes to those upstream packet outputs.

This is not:

- a generic federation or mesh packet
- a broad remote-agent lifecycle proof claim
- a packet that redefines packet `022`, `023`, or `024`
- a packet that treats packet `016` as broad remote-agent lifecycle proof
- a packet that widens discovery into implicit execution permission
- a packet that mixes `latest`, `dev`, and pinned protocol pages

## Clarifications

### Session 2026-04-01

- Q: Is packet `027` being written as immediate implementation approval or as scaffolding? → A:
  It is scaffolding only and must be revised before implementation.
- Q: What should packet `027` use as its upstream contract baseline? → A: Use the current finished
  packet outputs for `022`, `023`, and `024`, then refresh this packet again before
  implementation if those upstream packets change.
- Q: Which first interoperability slice is in scope? → A: A2A `v0.3.0` discovery plus A2A task
  lifecycle mapping.
- Q: What role does MCP play in this packet? → A: MCP `2025-11-25` remains a pinned baseline
  input and policy boundary, not the first new interop slice.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Normalize Capability Discovery (Priority: P1)

An operator or future implementer can use one frozen descriptor model to compare local ToolBus
tools, MCP-discovered capabilities, and one A2A agent-card input without assuming that discovery
metadata grants permission to execute anything.

**Why this priority**: capability normalization is the first contract needed before any remote
lifecycle mapping can stay bounded and comparable across surfaces.

**Independent Test**: inspect the packet contracts and confirm they define one normalized
descriptor model, one explicit source marker, and one explicit separation between discovery and
execution authority.

**Acceptance Scenarios**:

1. **Given** a local ToolBus capability, an MCP capability catalog entry, and an A2A agent card,
   **When** packet `027` is read, **Then** all three can be described through one normalized
   capability model with explicit source attribution.
2. **Given** any discovered capability, **When** an operator or implementer reads the packet,
   **Then** it is clear that discovery metadata alone does not authorize execution.
3. **Given** upstream packet `024` boundary rules, **When** this packet defines the normalized
   descriptor, **Then** it reuses discover-vs-execute separation instead of redefining it.

---

### User Story 2 - Map One Remote Lifecycle Model (Priority: P1)

An operator or future implementer can map one A2A `v0.3.0` remote task lifecycle into Mister Smith
workflow, result, and autonomy surfaces without broadening the packet into multi-protocol runtime
proof.

**Why this priority**: the first interoperability slice needs one honest lifecycle mapping, not
just discovery metadata.

**Independent Test**: inspect the packet contracts and confirm they define one lifecycle bridge
from A2A task states into Mister Smith workflow, result, and autonomy views using inherited
lifecycle and proof-boundary language.

**Acceptance Scenarios**:

1. **Given** an A2A task that progresses through non-terminal and terminal states, **When** the
   packet is read, **Then** the mapping into Mister Smith workflow and status surfaces is explicit.
2. **Given** an A2A task state that has no exact local match, **When** the lifecycle contract is
   read, **Then** the packet defines an explicit mapping rule or an explicit unsupported-state
   boundary.
3. **Given** upstream packet `022` and `023` contracts are still provisional, **When** this packet
   defines lifecycle mapping, **Then** it states that those inherited terms must be refreshed
   before implementation.

---

### User Story 3 - Preserve Provenance And Scope Boundaries (Priority: P2)

An operator or future implementer can see exactly what packet `027` does and does not claim,
including protocol pins, packet `016` continuity boundaries, and the required refresh gate before
implementation.

**Why this priority**: the main risk in this packet is scope drift and overclaim, not missing
syntax.

**Independent Test**: inspect `spec.md`, `plan.md`, and `tasks.md` and confirm they all carry the
same revision-before-implementation note, the same protocol pins, and the same non-goals.

**Acceptance Scenarios**:

1. **Given** packet `027` is read without the earlier dossier context, **When** the reader inspects
   the packet, **Then** they can see that it is a scaffold based on provisional upstream inputs and
   must be revised before implementation.
2. **Given** packet `016` continuity evidence exists, **When** packet `027` references it,
   **Then** the packet limits that reference to continuity and provenance rather than broad remote
   lifecycle proof.
3. **Given** pinned MCP and A2A sources are part of the packet, **When** protocol references are
   inspected, **Then** no mixed-version or unpinned protocol pages appear.

### Edge Cases

- an A2A agent card advertises a capability that lacks enough metadata to normalize safely
- discovery is allowed on a remote surface, but execution remains blocked by local policy
- an A2A task enters a lifecycle state that does not map cleanly onto current Mister Smith status
  language
- the final packet `022`, `023`, or `024` contracts change enough that the packet `027` scaffold
  needs a revision before implementation
- an MCP descriptor and an A2A descriptor appear similar at discovery time but imply different
  execution permission rules
- a later reader treats packet `016` as broad interop proof unless packet `027` calls out the
  narrower continuity boundary explicitly

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Packet `027` MUST state that it is a scaffold built from current repo truth and the
  finished packet outputs for `022`, `023`, and `024`, and MUST be revised before implementation
  if those upstream packet contracts change.
- **FR-002**: Packet `027` MUST freeze one bounded interoperability scope only: capability
  normalization plus one remote lifecycle mapping slice.
- **FR-003**: Packet `027` MUST define one normalized capability descriptor that can represent
  local ToolBus entries, MCP catalog entries, and one A2A `v0.3.0` agent-card input path.
- **FR-004**: Packet `027` MUST make capability source explicit so local, MCP, and A2A discovery
  inputs remain distinguishable after normalization.
- **FR-005**: Packet `027` MUST keep discovery metadata separate from execution permission and
  operator trust.
- **FR-006**: Packet `027` MUST freeze A2A `v0.3.0` as the first new interoperability slice for
  agent discovery and remote task lifecycle mapping.
- **FR-007**: Packet `027` MUST keep MCP pinned to `2025-11-25` and MUST use MCP as an existing
  normalization and policy-boundary input rather than the first new interop slice.
- **FR-008**: Packet `027` MUST NOT mix `latest`, `dev`, and pinned protocol pages inside the
  packet artifact set.
- **FR-009**: Packet `027` MUST define one lifecycle binding from A2A `v0.3.0` task states into
  Mister Smith workflow, result, and autonomy surfaces.
- **FR-010**: Packet `027` MUST reuse packet `022` lifecycle and durable-identifier language as
  provisional upstream input instead of redefining it.
- **FR-011**: Packet `027` MUST reuse packet `023` proof-boundary and run-truth language as
  provisional upstream input instead of redefining it.
- **FR-012**: Packet `027` MUST reuse packet `024` security and discover-vs-execute boundary
  language as provisional upstream input instead of redefining it.
- **FR-013**: Packet `027` MUST preserve packet `016` as continuity and provenance evidence only
  and MUST NOT describe it as broad remote-agent lifecycle proof.
- **FR-014**: Packet `027` MUST define one operator-visible provenance projection for remote
  capability use and remote lifecycle state mapping.
- **FR-015**: Packet `027` MUST explicitly defer generic federation, multi-protocol execution
  permission, extra remote protocols, strong-coordination policy, and live multi-remote runtime
  proof.
- **FR-016**: Packet `027` MUST include a blocking refresh gate in its plan and task artifacts so
  no implementation begins until the final packet `022`, `023`, and `024` outputs are reconciled.

### Key Entities *(include if feature involves data)*

- **NormalizedCapabilityDescriptor**: a shared discovery record that captures stable identifier,
  title, description, source, lifecycle hints, schema references, and permission references
  without granting execution authority by itself.
- **CapabilitySource**: a typed source marker that distinguishes local ToolBus, MCP
  `2025-11-25`, and A2A `v0.3.0` discovery inputs.
- **RemoteTaskLifecycleBinding**: a mapping record that translates A2A task states and lifecycle
  events into Mister Smith workflow, result, and autonomy projections.
- **RemoteCapabilityUseProvenance**: an operator-visible summary that records where a remote
  capability came from, which lifecycle binding applied, and which boundary rules still controlled
  execution permission.
- **InteropRevisionGate**: a packet-level rule that blocks implementation until packet `027` is
  refreshed against the finished packet `022`, `023`, and `024` outputs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `spec.md`, `plan.md`, and `tasks.md` all state that packet `027` is scaffold-only and
  must be revised before implementation.
- **SC-002**: every protocol reference in the packet artifact set resolves only to MCP
  `2025-11-25` or A2A `v0.3.0`, with no mixed-version protocol references.
- **SC-003**: the packet artifact set freezes one normalized capability contract and one A2A
  lifecycle-mapping contract without widening into generic federation work.
- **SC-004**: the packet artifact set preserves explicit discovery-vs-execute separation in the
  normalized descriptor and lifecycle-mapping language.
- **SC-005**: the packet artifact set uses packet `016` only for continuity and provenance
  boundary language.
- **SC-006**: the packet tasks include one blocking refresh gate ahead of any future
  implementation work and cover every functional requirement in this packet.

## Assumptions

- packets `022`, `023`, and `024` are the current upstream contract baseline for this scaffold
- packet `027` will be revised before implementation so it still matches the then-current packet
  `022`, `023`, and `024` outputs
- the first interoperable remote input path is A2A `v0.3.0` agent discovery plus A2A task
  lifecycle mapping, not broad multi-protocol federation
- MCP `2025-11-25` remains a pinned discovery and policy-boundary input for this packet
- discovery metadata is never sufficient on its own to authorize execution

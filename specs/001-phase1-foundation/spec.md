# Feature Specification: Phase 1 Foundation Contracts

**Feature Branch**: `001-phase1-foundation`  
**Created**: 2026-03-04  
**Status**: Draft  
**Input**: User description: "Create Phase 1 foundation contracts for canonical types, traits, errors, and configuration with Gate 1 validation."

## Scope

### In Scope

- Phase 1.1 core IDs, enums, supervision contracts, and error/result contract surface.
- Phase 1.2 core trait contract surface (`Actor`, `Agent`, `Tool`, `Resource`, `Supervisor`, `Transport`).
- Phase 1.3 configuration contract surface for typed config, validation, and layered overrides.
- Validation and acceptance criteria that map directly to Gate 1 checks.
- Constitution-aligned quality, testing, UX consistency, and performance constraints.

### Out of Scope

- Runtime startup or shutdown behavior.
- Actor execution loops and scheduling behavior.
- External transports, persistence implementations, and any Phase 2+ behavior.
- Refactoring canonical source documents during this feature.

## Clarifications

### Session 2026-03-04

- Q: Should Phase 1 require exact config struct names or domain-level typed config coverage?
  → A: Domain-level typed config coverage with deterministic validation and layering
  rules, without requiring exact struct names.
- Q: How strict should cross-document contract consistency be for active versus legacy
  illustrative content?
  → A: Strict for active Phase 1 references; legacy illustrative snippets are allowed
  only when they include an explicit note pointing to canonical definitions.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Canonical Type and Error Baseline (Priority: P1)

As a framework engineer, I need one canonical type and error contract surface so all downstream crates compile against one source of truth.

**Why this priority**: Every later phase depends on these contracts; inconsistency here cascades into all modules.

**Independent Test**: Can be fully tested by validating canonical type and error definitions in Phase 1 sources and running Gate 1 compile checks.

**Acceptance Scenarios**:

1. **Given** the canonical Phase 1 sources, **When** I inspect core type definitions,
   **Then** there is exactly one canonical definition for `AgentId`, `TaskId`, `MessageId`,
   `ToolId`, `MessagePriority`, `AgentState`, `AgentAvailability`, `AgentType`,
   `RestartPolicy`, `RestartScope`, and `SupervisionStrategy`.
2. **Given** message priority usage across core, testing, and transport docs,
   **When** I verify enum levels, **Then** `MessagePriority` is consistently five levels
   with discriminants `Critical=0`, `High=1`, `Normal=2`, `Low=3`, `Bulk=4`.
3. **Given** lifecycle and transport status references, **When** I validate naming,
   **Then** lifecycle state remains `AgentState` and transport presence status remains
   `AgentAvailability` without semantic collision.
4. **Given** Gate 1 compile expectations, **When** I build foundational crates, **Then** `mister-smith-core` and `mister-smith-config` compile cleanly.

---

### User Story 2 - Stable Core Trait Contracts (Priority: P2)

As a subsystem implementer, I need stable core trait signatures so implementations remain compatible across modules.

**Why this priority**: Trait signature drift causes integration breakage and inconsistent implementations in downstream phases.

**Independent Test**: Can be fully tested by comparing trait signatures in canonical trait sources and running contract consistency checks.

**Acceptance Scenarios**:

1. **Given** canonical trait sources, **When** I validate trait signatures, **Then** `Actor`, `Agent`, `Tool`, `Resource`, `Supervisor`, and `Transport` contracts match declared canonical definitions.
2. **Given** tool contract references, **When** I compare canonical and integration references, **Then** `Tool` signature fields (`execute`, `schema`, `capabilities`, `tool_id`, `version`) remain consistent.
3. **Given** Phase 1 scope boundaries, **When** trait contracts are reviewed, **Then** contracts define interfaces only and do not introduce runtime behavior requirements.

---

### User Story 3 - Typed Configuration Contracts and Validation Rules (Priority: P3)

As an operator and developer, I need typed config schemas and validation and override rules so startup configuration is predictable and safe.

**Why this priority**: Misconfigured foundational systems create non-deterministic startup behavior and expensive downstream failures.

**Independent Test**: Can be fully tested by validating config schema expectations, override precedence rules, and explicit validation failure behavior.

**Acceptance Scenarios**:

1. **Given** configuration contracts, **When** I review foundational config structures, **Then** Phase 1 includes typed contract coverage for runtime, agent, transport-shape, and security-shape settings.
2. **Given** layered configuration behavior, **When** file and environment sources are applied, **Then** precedence and merge behavior are explicit and deterministic.
3. **Given** malformed or incomplete configuration input, **When** validation is applied, **Then** failures are surfaced explicitly with actionable error semantics and no silent fallback.
4. **Given** Gate 1 scope, **When** configuration contracts are implemented, **Then** load-time validation is required before runtime behavior is allowed.

---

### Edge Cases

- Conflicting type names appear in non-canonical or illustrative snippets; these are
  acceptable only when explicitly marked as illustrative and linked to canonical
  Phase 1 definitions.
- `MessagePriority` is defined without explicit discriminants or with incorrect numeric mapping in one or more references.
- `RestartPolicy` appears as both enum and struct name across active docs and creates contract ambiguity.
- `AgentState` and transport-status naming are merged or cross-used, causing lifecycle and availability semantic confusion.
- Trait signatures diverge between canonical trait docs and integration references.
- Required configuration fields are missing, malformed, or supplied with wrong types via environment overrides.
- Environment variable overlays provide values that parse but fail semantic validation constraints.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST define one canonical Phase 1 contract surface for core IDs, enums, supervision contracts, and error/result types.
- **FR-002**: The system MUST preserve UUID-based core ID newtypes for `AgentId`, `TaskId`, `MessageId`, and `ToolId`.
- **FR-003**: The system MUST define `MessagePriority` as a five-level enum with explicit discriminants `0..4` mapped to `Critical`, `High`, `Normal`, `Low`, `Bulk`.
- **FR-004**: The system MUST keep lifecycle and transport presence semantics separate via `AgentState` and `AgentAvailability`.
- **FR-005**: The system MUST define supervision contracts using `RestartPolicy`,
  `RestartScope`, and `SupervisionStrategy` without conflicting type names in active
  Phase 1 references; any legacy illustrative snippets MUST explicitly point to
  canonical definitions.
- **FR-006**: The system MUST define and publish stable Phase 1 trait contracts for `Actor`, `Agent`, `Tool`, `Resource`, `Supervisor`, and `Transport` with canonical signatures.
- **FR-007**: The system MUST maintain `Tool` contract signature consistency between canonical trait source and cross-referenced integration docs.
- **FR-008**: The system MUST define a typed configuration contract baseline covering
  runtime, agent, transport-domain, and security-domain settings, without requiring
  exact struct-name parity across all canonical documents.
- **FR-009**: The system MUST enforce deterministic configuration layering and precedence for base defaults, file configuration, and environment overrides.
- **FR-010**: The system MUST fail configuration validation explicitly for malformed or semantically invalid values and MUST NOT allow silent success-shaped fallbacks.
- **FR-011**: The system MUST define Gate 1 validation commands for compile checks and contract consistency checks as part of acceptance evidence.
- **FR-012**: The system MUST align with constitution requirements for code quality, testing standards, UX consistency of naming and error semantics, and Phase 1 performance constraints.

### Constitution Alignment Requirements

- **CAR-001 (Code Quality)**: Canonical definitions MUST remain unambiguous, non-duplicative in active references, and traceable to authoritative documents.
- **CAR-002 (Testing)**: Gate 1 compile and contract consistency checks MUST be part of required validation evidence.
- **CAR-003 (UX Consistency)**: Contract naming and error semantics MUST remain consistent across user-facing specs and cross-references.
- **CAR-004 (Performance)**: Phase 1 contracts MUST remain compile-time focused and MUST NOT add runtime overhead requirements outside declared scope.

### Validation Command Set (Required Evidence)

- Core type presence:
  - `rg -n "pub enum AgentState|pub enum AgentAvailability|pub enum MessagePriority|pub enum AgentType|pub enum RestartPolicy|pub enum RestartScope" spec/core-architecture/type-definitions.md`
- Restart policy collision check:
  - `rg -n "pub struct RestartPolicy\\b|pub enum RestartPolicy\\b" spec/data-management spec/core-architecture`
- Tool signature consistency:
  - `rg -n "pub trait Tool" spec/core-architecture/module-organization-type-system.md spec/core-architecture/system-integration.md`
- Message priority cross-reference check:
  - `rg -n "MessagePriority" spec/testing/test-schemas.md spec/data-management/message-schemas.md spec/transport/nats-transport.md`
- Gate 1 compile checks:
  - `cargo build -p mister-smith-core`
  - `cargo build -p mister-smith-config`
- Spec quality check:
  - `npx markdownlint-cli2 "specs/001-phase1-foundation/spec.md" --config .markdownlint.json`

### Key Entities *(include if feature involves data)*

- **CanonicalCoreTypeSet**: The authoritative set of Phase 1.1 IDs, enums, supervision contracts, and error/result aliases used by downstream phases.
- **CoreTraitContractSet**: The authoritative Phase 1.2 trait signatures that define implementation interfaces without runtime behavior.
- **ConfigurationContractSet**: The authoritative Phase 1.3 typed configuration structures, precedence rules, and validation constraints.
- **Gate1ValidationEvidence**: The command-based evidence set proving compile readiness and cross-document contract consistency.
- **ConstitutionComplianceSet**: Quality, testing, UX consistency, and performance constraint checks required for Phase 1 acceptance.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Gate 1 compile checks complete successfully for `mister-smith-core` and `mister-smith-config` with no compile failures.
- **SC-002**: Canonical type and trait consistency checks complete with expected outputs and no unresolved naming collisions in active Phase 1 references.
- **SC-003**: Configuration validation behavior is explicitly specified such that malformed config inputs fail with actionable error semantics.
- **SC-004**: Lifecycle versus transport status naming consistency is preserved (`AgentState` for lifecycle, `AgentAvailability` for transport presence).
- **SC-005**: Every functional requirement in this spec maps to at least one acceptance scenario and at least one validation command.
- **SC-006**: Markdown lint passes for this spec with zero errors.

<!--
Sync Impact Report
====================
Version change: 0.0.0 (template) → 1.0.0 (initial ratification)
Modified principles: N/A (initial creation from template)
Added sections:
  - 7 Core Principles (I–VII)
  - Technology Stack Constraints
  - Specification-to-Implementation Workflow
  - Governance rules
Removed sections: None
Templates requiring updates:
  - .specify/templates/plan-template.md — ✅ no changes needed
    (Constitution Check section already references this file generically)
  - .specify/templates/spec-template.md — ✅ no changes needed
    (User Scenarios and Requirements sections align with principles)
  - .specify/templates/tasks-template.md — ✅ no changes needed
    (Phase-gated structure and dependency ordering align with principles)
Follow-up TODOs: None
-->

# Mister Smith Constitution

## Core Principles

### I. Canonical Single Source of Truth

Every core type, enum, and trait MUST have exactly one canonical
definition. That definition lives in the authoritative spec file
(e.g., `spec/core-architecture/type-definitions.md` for types,
`spec/core-architecture/module-organization-type-system.md` for
traits). All other documents MUST import from or reference the
canonical source — never redefine.

**Rationale**: Duplicate definitions drift. The 2026-03-03 validation
found AgentState, MessagePriority, and SupervisionStrategy defined
inconsistently across files. Canonical sourcing prevents this class
of defect entirely.

### II. Spec-First Design

No implementation code MUST be written without a corresponding
specification document. Every public API surface, type, trait, and
behavioral contract MUST trace back to a spec file. The spec is the
contract; the code is the implementation of that contract.

**Rationale**: 65+ spec files exist before any Rust code. This is
deliberate. Specifications enable parallel review, validation, and
course-correction before the cost of implementation is incurred.

### III. Phase-Gated Build Order

Implementation MUST follow the 8-phase dependency order defined in
`ROADMAP.md`. No phase may begin implementation until its upstream
gate criteria are satisfied. Gate criteria MUST be validated with
concrete, reproducible checks (grep commands, compilation, tests) —
not assertions or self-certifications.

**Rationale**: The framework has deep dependency chains (types →
runtime → actors → supervision → transport → agents). Implementing
out of order creates rework. Gate criteria enforce this discipline
with evidence.

### IV. Model-Agnostic Architecture

The framework MUST NOT depend on any specific LLM provider. All
provider-specific integrations MUST be implemented as pluggable
adapters behind provider-neutral trait interfaces. Core framework
code MUST NOT import, reference, or assume any particular model
API.

**Rationale**: The framework orchestrates agents — it does not
provide intelligence. Coupling to a single provider creates vendor
lock-in and limits adoption.

### V. Erlang/OTP-Style Fault Tolerance

The supervision tree architecture MUST implement hierarchical fault
isolation using Rust's ownership model. Failures MUST be contained
at the appropriate supervision level and handled via configurable
restart policies (OneForOne, OneForAll, RestForOne). The actor model
MUST use message-passing with bounded channels — never shared
mutable state.

**Rationale**: This is the framework's core architectural commitment
and highest-risk design decision. Erlang/OTP semantics in Rust have
no established library. Getting supervision trees right determines
whether the framework delivers on its fault-tolerance promise.

### VI. Evidence-Based Validation

All specification changes, phase completions, and readiness claims
MUST be backed by reproducible evidence. Validation checks MUST be
executable (grep patterns, compilation commands, test suites).
Claims of consistency MUST be verified against the actual file
contents, not prior reports.

**Rationale**: The 2026-03-03 validation report claimed resolution
of several issues that grep searches later found still present.
Evidence-based validation prevents this discrepancy between
documented and actual state.

### VII. Explicit Dependency Management

Every crate, spec file, type, and trait MUST have its dependency
relationships explicitly documented. `VERSION_REFERENCE.md` MUST
be the single authoritative source for crate versions. Breaking
changes MUST include cascade analysis identifying all affected
downstream consumers.

**Rationale**: The framework spans 13 workspace crates with 30+
external dependencies. Implicit dependencies create surprise
breakage. The async-nats 0.37→0.46 migration demonstrated how a
single crate update cascades across 9+ specification files.

## Technology Stack Constraints

- **Language**: Rust, MSRV 1.88.0 (binding constraint: async-nats
  0.46.0)
- **Async runtime**: Tokio 1.49.0, single runtime boundary per
  process
- **Messaging**: async-nats 0.46.0 with JetStream, KV, and
  object-store feature gates
- **Serialization**: serde with derive macros; MessagePack for wire
  format, JSON for configuration
- **Error handling**: `thiserror` 1.x for domain errors with
  explicit conversion paths; no `anyhow` in library crates
- **HTTP**: Axum 0.8.x (not Actix, not Warp)
- **gRPC**: Tonic 0.14.x with prost 0.14.x
- **Storage**: PostgreSQL via sqlx 0.8.x, Redis via redis 1.0.x
- **Security**: TLS 1.3 via rustls; JWT via jsonwebtoken 10.x;
  mTLS for agent-to-agent communication
- **Observability**: tracing 0.1.x ecosystem (tracing-subscriber,
  tracing-opentelemetry); OpenTelemetry 0.31.x with OTLP exporter

Stack changes MUST be proposed as amendments to this constitution
and reflected in `VERSION_REFERENCE.md` before implementation.

## Specification-to-Implementation Workflow

1. **Spec validation**: Before implementing any phase, validate all
   referenced spec files for internal consistency, cross-reference
   integrity, and alignment with `type-definitions.md` canonical
   types.
2. **Phase document review**: Read the corresponding
   `plans/roadmap-phases/phase-N-*.md` document to understand
   scope boundaries, inputs, outputs, and gate criteria.
3. **Gate check (entry)**: Verify all upstream gate criteria are
   satisfied with evidence before starting implementation.
4. **Implementation**: Write code that traces to spec contracts.
   Each public API element MUST reference its spec source.
5. **Gate check (exit)**: Run all gate validation commands from
   `ROADMAP.md`. Pass every check before declaring phase complete.
6. **Cascade audit**: After completing a phase, verify that
   downstream phase documents remain consistent with any
   refinements made during implementation.

Skipping steps in this workflow MUST be treated as a process
failure, not a time optimization.

## Governance

This constitution is the highest-authority document for the Mister
Smith framework. It supersedes conflicting guidance in spec files,
implementation plans, and ad-hoc decisions.

- **Amendments**: Any change to principles or technology stack
  constraints MUST be documented with rationale, approved by the
  project maintainer, and reflected in a version increment.
- **Versioning**: Constitution versions follow semantic versioning.
  MAJOR for principle removal or redefinition, MINOR for new
  principles or material expansion, PATCH for clarifications.
- **Compliance review**: Every implementation PR MUST be verifiable
  against these principles. Reviewers SHOULD reference specific
  principle numbers (I–VII) when flagging violations.
- **Conflict resolution**: When spec files conflict with each
  other, the canonical source identified in Principle I wins. When
  specs conflict with the constitution, the constitution wins.

**Version**: 1.0.0 | **Ratified**: 2026-03-04 | **Last Amended**: 2026-03-04

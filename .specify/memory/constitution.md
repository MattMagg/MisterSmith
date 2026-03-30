<!--
Sync Impact Report
====================
Version change: 1.1.0 -> 1.2.0
Modified principles:
  - I. Canonical Single Source of Truth
  - III. Phase-Gated Build Order -> III. Phase-And-Packet-Gated Delivery
  - VII. Explicit Dependency Management
  - Governance
Added sections:
Updated sections:
  - Technology Stack Constraints
  - Specification-to-Implementation Workflow
Templates requiring updates:
Follow-up TODOs:
  - None
-->

# Mister Smith Constitution

## Core Principles

### I. Canonical Single Source of Truth

Every core type, enum, trait, status-bearing repo claim, and dependency version MUST have exactly
one canonical definition. That definition lives in the authoritative source for that surface:
`spec/` for architecture and contracts, `docs/direction.md` for strategic sequencing,
`docs/current-state.md` for live repo truth, and Cargo manifests for actual dependency versions.
Derived reports and packet documents MUST reference those sources and MUST NOT silently redefine
them.

**Rationale**: Duplicate definitions drift. Canonical sourcing prevents contradictions between
specs, packets, state docs, and runtime code.

### II. Spec-First Design

No implementation code MUST be written without a corresponding specification or bounded packet
artifact. Every public API surface, type, behavioral contract, or workflow contract MUST trace
back to a spec file, packet, or current-state authority note.

**Rationale**: Mister Smith relies on spec-first planning to keep architecture, runtime behavior,
and workflow changes reviewable before implementation cost is incurred.

### III. Phase-And-Packet-Gated Delivery

Implementation MUST respect the landed phase order in `ROADMAP.md`, the strategic sequencing in
`docs/direction.md`, the live-truth routing in `docs/current-state.md`, and the active scope-freeze
note. Once the substrate phases are landed, new work MUST enter as bounded packets with explicit
scope, sequencing, and dependency rules instead of vague epics. Multiple packets MAY proceed in
parallel only when scopes are disjoint, validation can close independently, and the work does not
silently reopen a more foundational packet. Gate criteria MUST be validated with concrete,
reproducible checks rather than assertions.

**Rationale**: The repo no longer lives in the original eight-phase buildout alone. Future work is
packet-driven, and disciplined scope freezing prevents benchmark or frontier goals from dissolving
into unbounded side quests.

### IV. Model-Agnostic Architecture

The framework MUST NOT depend on any specific LLM provider. Provider-specific integrations MUST be
implemented as pluggable adapters behind provider-neutral trait interfaces. Core framework code
MUST NOT import, reference, or assume any particular model API.

**Rationale**: Mister Smith orchestrates agents. It should not collapse the control plane or the
runtime into one provider silo.

### V. Erlang/OTP-Style Fault Tolerance

The supervision tree architecture MUST implement hierarchical fault isolation using Rust's
ownership model. Failures MUST be contained at the appropriate supervision level and handled via
configurable restart policies. The actor model MUST use message-passing with bounded channels and
MUST avoid shared mutable state.

**Rationale**: Fault isolation and recoverability are core product promises, not optional cleanup
work after new orchestration features land.

### VI. Evidence-Based Validation

All specification changes, packet completions, and readiness claims MUST be backed by reproducible
evidence. Validation checks MUST be executable. Any claim that crosses from deterministic checks to
live runtime proof MUST state that boundary explicitly and MUST NOT overstate what the environment
actually proved.

**Rationale**: Mister Smith frequently mixes deterministic coverage, runtime smoke, and operator
evidence. Honest boundaries are necessary to avoid fake closure and misleading benchmark claims.

### VII. Explicit Dependency Management

Every crate, packet, spec file, type, and workflow surface MUST have its dependency relationships
documented explicitly. Cargo manifests remain the authoritative source for actual dependency
versions. `VERSION_REFERENCE.md` is a derived audit matrix for spec and migration alignment and
MUST be refreshed when version-bearing decisions materially change. Breaking changes MUST include
cascade analysis for all affected downstream consumers.

**Rationale**: The workspace spans many crates and multiple orchestration surfaces. Implicit
dependencies create surprise breakage and invalid packet assumptions.

### VIII. Clean Closure And Resumability

No task, packet, or review handoff may end with task-owned dirty repo state, stale status docs, or
unframed evidence. Closure MUST leave the repo intelligible to a cold future session, and git
closure MUST include the repo's closure gate script before work is declared done.

**Rationale**: Half-closed work destroys autonomy. Clean closure is a prerequisite for reliable
multi-session orchestration and honest benchmark iteration.

## Technology Stack Constraints

- **Language**: Rust, MSRV 1.88.0
- **Async runtime**: Tokio 1.49.0, single runtime boundary per process
- **Messaging**: async-nats 0.46.0 with JetStream, KV, and object-store feature gates
- **Serialization**: serde with derive macros; MessagePack for wire format, JSON for configuration
- **Error handling**: `thiserror` 1.x for domain errors with explicit conversion paths; no
  `anyhow` in library crates
- **HTTP**: Axum 0.8.x
- **gRPC**: Tonic 0.14.x with prost 0.14.x
- **Storage**: PostgreSQL via sqlx 0.8.x; JetStream KV is the current distributed runtime state
  substrate. Redis remains historical or spec-level only unless a future ratified packet
  reintroduces it into the live workspace.
- **Security**: TLS 1.3 via rustls; JWT via jsonwebtoken 10.x; mTLS where the transport security
  contract requires it
- **Observability**: tracing 0.1.x ecosystem with OpenTelemetry 0.31.x and OTLP export

Stack changes MUST be proposed as amendments to this constitution and reflected in Cargo manifests
as the implementation source of truth. `VERSION_REFERENCE.md` SHOULD be refreshed before closure so
spec and migration notes stay aligned.

## Specification-to-Implementation Workflow

1. **Repo authority pass**: Read `AGENTS.md`, `docs/direction.md`, `docs/current-state.md`, the
   active scope-freeze or closure note, and any workflow contract docs relevant to the packet.
2. **Spec or packet validation**: Confirm the packet scope matches current repo truth and does not
   silently reopen landed work.
3. **Gate check (entry)**: Verify all upstream dependencies and bounded non-goals with evidence
   before implementation starts.
4. **Implementation**: Write code and docs that trace back to the packet, spec, and current-state
   authority.
5. **Gate check (exit)**: Run the narrowest meaningful validation that proves the changed behavior.
   Distinguish deterministic checks from live proof.
6. **Cascade audit**: Reconcile any touched state-bearing docs, packet notes, or artifact indexes
   so they match landed truth.
7. **Clean closure**: Before declaring completion, run the repo closure gate
   `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync` and leave the
   repo clean.

Skipping steps in this workflow MUST be treated as a process failure, not a time optimization.

## Governance

This constitution is the highest-authority SpecKit companion document for Mister Smith packet
planning, scope discipline, validation posture, and closure expectations. It does not replace the
repo-wide authority routers for strategic direction, live truth, architecture contracts, or
compiler-enforced dependency versions.

- **Amendments**: Any change to principles or stack constraints MUST be documented with rationale
  and reflected in a version increment.
- **Versioning**: Constitution versions follow semantic versioning. MAJOR for principle removal or
  redefinition, MINOR for new principles or material expansion, PATCH for clarifications.
- **Compliance review**: Every implementation PR or direct-to-main bounded slice SHOULD be
  reviewable against these principles.
- **Conflict resolution**: `docs/direction.md` wins for strategic sequencing, `docs/current-state.md`
  wins for live repo truth, `spec/` wins for architecture and type contracts, and Cargo manifests
  win for actual dependency versions. This constitution governs how SpecKit work is planned,
  validated, and closed once those sources are known.

**Version**: 1.2.0 | **Ratified**: 2026-03-04 | **Last Amended**: 2026-03-29

<!--
Sync Impact Report
====================
Version change: 1.0.0 -> 1.1.0
Modified principles:
  - III. Phase-Gated Build Order -> III. Phase-And-Packet-Gated Delivery
  - VI. Evidence-Based Validation (expanded to require honest deterministic/live-proof boundaries)
Added sections:
  - VIII. Clean Closure And Resumability
Updated sections:
  - Specification-to-Implementation Workflow
Templates requiring updates:
  - .specify/templates/spec-template.md — updated for Mister Smith packet shape
  - .specify/templates/plan-template.md — updated for bounded milestone and deferral structure
  - .specify/templates/tasks-template.md — updated for blocking freeze, bounded lanes, and closure gates
Follow-up TODOs:
  - Keep prompt wrappers aligned with these packet rules
-->

# Mister Smith Constitution

## Core Principles

### I. Canonical Single Source of Truth

Every core type, enum, trait, and status-bearing repo claim MUST have exactly one canonical
definition. That definition lives in the authoritative spec or router document for that surface.
All other documents MUST import from or reference the canonical source and MUST NOT silently
redefine it.

**Rationale**: Duplicate definitions drift. Canonical sourcing prevents contradictions between
specs, packets, state docs, and runtime code.

### II. Spec-First Design

No implementation code MUST be written without a corresponding specification or bounded packet
artifact. Every public API surface, type, behavioral contract, or workflow contract MUST trace
back to a spec file, packet, or current-state authority note.

**Rationale**: Mister Smith relies on spec-first planning to keep architecture, runtime behavior,
and workflow changes reviewable before implementation cost is incurred.

### III. Phase-And-Packet-Gated Delivery

Implementation MUST respect the landed phase order in `ROADMAP.md` and the current packet-based
forward direction in repo authority docs such as `docs/current-state.md` and the active scope-freeze
note. Once the substrate phases are landed, new work MUST enter as one bounded packet at a time
instead of vague parallel epics. Gate criteria MUST be validated with concrete, reproducible
checks rather than assertions.

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
documented explicitly. `VERSION_REFERENCE.md` remains the single authoritative source for crate
versions. Breaking changes MUST include cascade analysis for all affected downstream consumers.

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
- **Storage**: PostgreSQL via sqlx 0.8.x, Redis via redis 1.0.x
- **Security**: TLS 1.3 via rustls; JWT via jsonwebtoken 10.x; mTLS for agent-to-agent
  communication
- **Observability**: tracing 0.1.x ecosystem with OpenTelemetry 0.31.x and OTLP export

Stack changes MUST be proposed as amendments to this constitution and reflected in
`VERSION_REFERENCE.md` before implementation.

## Specification-to-Implementation Workflow

1. **Repo authority pass**: Read `AGENTS.md`, `docs/current-state.md`, the active scope-freeze or
   closure note, and any workflow contract docs relevant to the packet.
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

This constitution is the highest-authority SpecKit companion document for the Mister Smith repo.
It supersedes conflicting guidance in packet templates, implementation plans, and ad hoc decisions.

- **Amendments**: Any change to principles or stack constraints MUST be documented with rationale
  and reflected in a version increment.
- **Versioning**: Constitution versions follow semantic versioning. MAJOR for principle removal or
  redefinition, MINOR for new principles or material expansion, PATCH for clarifications.
- **Compliance review**: Every implementation PR or direct-to-main bounded slice SHOULD be
  reviewable against these principles.
- **Conflict resolution**: When packet files conflict with current repo authority, the repo
  authority docs win. When repo authority conflicts with this constitution, the constitution wins.

**Version**: 1.1.0 | **Ratified**: 2026-03-04 | **Last Amended**: 2026-03-26

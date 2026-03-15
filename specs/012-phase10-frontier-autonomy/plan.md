# Implementation Plan: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

**Branch**: `012-phase10-frontier-autonomy` | **Date**: 2026-03-10 | **Spec**: [spec.md](spec.md)

## Summary

Phase 10 extends the completed Phase 9 and Phase 9.1 substrate into a new roadmap layer centered
on **frontier autonomy**: topology-aware execution, managed memory/context, predictive supervision,
operator-visible autonomy control, and bounded delegation/provenance.

The implementation should not treat this phase as "more security work" or "more provider work."
Instead, it layers structural control-plane intelligence on top of the existing provider-neutral
LLM, persistence, security, and operations foundations:

- `mister-smith-agents` gains explicit execution-graph and topology-compilation capabilities
- `mister-smith-persistence` gains a memory-management layer above current storage primitives
- `mister-smith-llm` contributes step-level stream signals to the Guard layer rather than new
  provider-specific features
- `mister-smith-app`, observability, and deployment assets expose the live autonomy pulse to
  operators
- Phase 9.1 delegation-chain work is completed into enforceable provenance and bounded delegation

## Implementation Status Snapshot (2026-03-15)

- **Implemented and validated in repo**: foundational autonomy contracts/events (10.0),
  execution-graph/topology compilation (10.1), branch checkpointing and resilience-aware routing
  (10.2), managed memory/context assembly (10.3), Guard/Advisor supervision plus stream monitors
  (10.4), operator autonomy views plus deploy scaffolding (10.5), and bounded
  delegation/provenance enforcement (10.6).
- **Validation evidence captured in this gate pass**:
  - `cargo test -p mister-smith-agents`
  - `cargo test -p mister-smith-persistence`
  - `cargo test -p mister-smith-security`
  - `cargo test -p mister-smith-llm`
  - `cargo test -p mister-smith-core`
  - `cargo test -p mister-smith-app`
  - `python3 scripts/validate_deploy_assets.py deploy/dashboards deploy/alerts`
  - `cargo build --workspace`
- **Operational follow-up stays separate**: current Symphony, Linear, and smith MCP queue
  governance work remains active operational work, not incomplete Phase 10 framework scope.

## Technical Context

- **Language/Version**: Rust, MSRV 1.88.0
- **Primary Dependencies**: existing workspace crates, `petgraph` for DAG/topology algorithms,
  `async-nats` JetStream KV for checkpoints and control-plane state, existing `tracing` /
  OpenTelemetry stack for autonomy telemetry
- **Storage**: existing PostgreSQL + JetStream KV dual-store from Phase 6; no replacement storage
  engine in this phase
- **Security Foundation**: existing Phase 9.1 message signing, Auth Callout, state validation, and
  delegation-chain substrate
- **Testing**: targeted crate tests for topology compilation, memory/context management, guard
  decisions, operator views, and delegation enforcement; `cargo build --workspace` as cross-crate
  compatibility baseline. The 2026-03-15 gate pass validated all Phase 10 subphases with the
  targeted crate suites plus deploy-asset checks.
- **Target Platform**: Linux runtime, macOS development parity
- **Performance Goals**: branch-local recovery without re-running completed work, at least 30%
  reduction in delivered role context versus full-history broadcast, operator-visible autonomy
  state for all validation scenarios
- **Constraints**: no learned routing, no guided decoding, no local inference/disaggregated
  serving, no CRDT/session-type/general consensus suite, no ML anomaly detection

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Phase 10 extends canonical types/events/contracts rather than redefining provider, transport, or security surfaces. |
| II. Spec-First Design | PASS | Scope, grounding, clarifications, FRs, and SCs are defined in `spec.md` before any implementation work. |
| III. Phase-Gated Build Order | PASS | Phase 10 explicitly builds on completed Phases 8, 9, and 9.1 and does not backfill those scopes. |
| IV. Model-Agnostic Architecture | PASS | Topology, memory, supervision, and delegation stay provider-neutral and sit above `ModelProvider`. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Guard/Advisor augments, rather than replaces, OTP-style supervision and branch isolation. |
| VI. Evidence-Based Validation | PASS | Scope is grounded in the March audit/deviation docs plus consolidated research findings. |
| VII. Explicit Dependency Management | PASS | Existing crate dependencies and phase boundaries are stated; speculative serving/model work stays deferred. |

## Project Structure

### Documentation (this feature)

```text
specs/012-phase10-frontier-autonomy/
+-- spec.md                         # Feature specification
+-- plan.md                         # This file
+-- research.md                     # Research decisions and deferred findings
+-- data-model.md                   # Phase 10 entities and relationships
+-- quickstart.md                   # Planned verification and walkthrough scenarios
+-- contracts/
|   +-- autonomy-observability.md   # Operator-facing autonomy control plane
|   +-- delegation-provenance.md    # Bounded delegation and provenance contract
|   +-- guard-advisor.md            # Predictive supervision and intervention contract
|   +-- memory-manager.md           # Managed memory/context contract
|   +-- topology-compiler.md        # Execution graph and topology compiler contract
+-- checklists/
|   +-- requirements.md             # Spec quality checklist
+-- tasks.md                        # Task breakdown
+-- analyze.md                      # Cross-artifact analysis
```

### Source Code (repository root)

```text
Cargo.toml                                  # Workspace dependencies / optional graph utility

crates/mister-smith-core/
+-- src/autonomy.rs                         # Shared autonomy types (graph IDs, topology kinds, budgets)
+-- src/error.rs                            # Topology / guard / delegation errors if promoted to core

crates/mister-smith-events/
+-- src/autonomy.rs                         # Topology, checkpoint, and intervention event types

crates/mister-smith-agents/
+-- src/
|   +-- execution_graph.rs                  # ExecutionGraph, ExecutionBranch, validation
|   +-- topology.rs                         # TopologyCompiler, topology-selection policy
|   +-- branch_checkpoint.rs                # Branch-local checkpoint coordination
|   +-- context_manager.rs                  # Role-aware context assembly and budgets
|   +-- guard.rs                            # Guard/Advisor layer and failure classification
|   +-- intervention.rs                     # Intervention policies and records
|   +-- profile.rs                          # Agent profile / telemetry snapshot integration
|   +-- roles/
|       +-- planner.rs                      # Emits execution-graph-ready plans
|       +-- critic.rs                       # Supplies intervention evidence
|       +-- executor.rs                     # Emits branch progress and checkpoint signals

crates/mister-smith-llm/
+-- src/
|   +-- model_event.rs                      # Existing event types extended as supervision inputs
|   +-- stream_monitor.rs                   # Step-boundary / degradation signal extraction

crates/mister-smith-persistence/
+-- src/
|   +-- memory/
|       +-- fragment.rs                     # MemoryFragment persistence shape
|       +-- manager.rs                      # MemoryManager over KV + PostgreSQL
|       +-- snapshot.rs                     # Checkpoint-ready memory snapshots
|       +-- consolidation.rs                # Background consolidation and summarization

crates/mister-smith-security/
+-- src/
|   +-- delegation.rs                       # DelegationCapability and provenance enforcement
|   +-- jwt/claims.rs                       # Extend/deepen delegation-chain semantics

crates/mister-smith-app/
+-- src/
|   +-- autonomy.rs                         # Operator-facing autonomy status / inspection surfaces
|   +-- main.rs                             # CLI or app wiring for autonomy views

deploy/
+-- dashboards/mister-smith-autonomy.json  # Operator-facing topology/checkpoint/intervention dashboard
+-- alerts/mister-smith-autonomy-rules.yml # Autonomy-specific alert scaffolding
```

## Design Decisions

### D1: Explicit ExecutionGraph Before Dispatch

**Decision**: Planner output must be normalized into an explicit `ExecutionGraph` before any work is
dispatched.

**Rationale**: This gives topology selection, dependency validation, branch-local checkpointing,
and operator inspection a single authoritative object. It also keeps graph correctness out of
runtime heuristics.

### D2: Heuristic/Policy Topology Compiler Before Learned Routing

**Decision**: Phase 10 uses deterministic task-shape, dependency, health, and budget signals to
select topology. Learned routing remains deferred.

**Rationale**: The research clearly supports topology-aware execution, but learned routers, kNN
selectors, and model-trained policies are explicitly deferred. The repo needs the structural
compiler first.

### D3: Managed Memory Layer Over Existing Persistence

**Decision**: Introduce a `MemoryManager` over existing JetStream KV + PostgreSQL rather than
introducing a new standalone memory store.

**Rationale**: Phase 6 already solved storage primitives. The missing piece is lifecycle-aware
memory management: paging, summaries, consolidation, role-scoped assembly, and snapshots.

### D4: Guard Layer Augments OTP Supervision

**Decision**: Predictive supervision is a **Guard/Advisor layer above** existing OTP-style
supervision, not a replacement for restart policies or failure isolation.

**Rationale**: Hard crashes still belong to OTP supervisors. The Guard layer adds failure
classification, step-level degradation handling, and targeted interventions before full restarts.

### D5: Operator View Must Be Event-Derived

**Decision**: Operator-facing autonomy status is derived from typed topology, checkpoint, memory,
and intervention events, not from scraping logs or provider traces.

**Rationale**: This keeps observability consistent with the repo's evidence-based, typed-event
architecture and supports deterministic audits.

### D6: Delegation Builds on Phase 9.1, Not a New Auth Stack

**Decision**: Bounded delegation/provenance extends the Phase 9.1 `delegation_chain`,
message-signing, and Auth Callout substrate instead of introducing a parallel authorization model.

**Rationale**: Phase 9.1 deliberately kept the delegation foundation alive for Phase 10+. Phase 10
should complete that line of work without reworking core security concepts.

## Dependency Changes

### New Dependencies

- `petgraph` — graph validation, topological sorting, and dependency traversal for execution graphs

### Existing Crates Touched

- `mister-smith-agents` — execution graph, topology compiler, context manager, guard layer
- `mister-smith-llm` — stream-monitor hooks for step/degradation signals
- `mister-smith-persistence` — managed memory layer, snapshots, consolidation
- `mister-smith-security` — bounded delegation and provenance enforcement
- `mister-smith-events` / `mister-smith-app` / deploy assets — autonomy events and operator views

## Subphase Execution Plan

### 10.1 Execution Graph + Topology Compiler

**Scope**: Normalize planner output into `ExecutionGraph`, validate dependencies, reject cycles,
and choose among sequential/parallel/pipeline/hierarchical/hybrid topologies.

**Outputs**: execution-graph contract, topology-selection policy, invalid-graph tests, topology
selection tests.

- **Depends on**: Phase 9 planner output contract, existing Phase 7 orchestration baseline
- **Crates**: `mister-smith-agents`, `mister-smith-core`

### 10.2 Branch Checkpointing + Resilience-Aware Routing

**Scope**: Persist branch progress, attach health/budget/profile signals to routing, and support
branch-local resume or reassignment without global replay.

**Outputs**: checkpoint coordinator, branch resume tests, routing rationale records.

- **Depends on**: 10.1
- **Crates**: `mister-smith-agents`, `mister-smith-persistence`, `mister-smith-events`

### 10.3 Managed Memory / Context Manager

**Scope**: Introduce `MemoryFragment`, `ContextBudget`, `MemorySnapshot`, role-aware assembly,
paging, summaries, and background consolidation.

**Outputs**: memory-manager contract, context-budget tests, snapshot/consolidation tests.

- **Depends on**: Phase 6 persistence, 10.1 graph/branch identity
- **Crates**: `mister-smith-persistence`, `mister-smith-agents`

### 10.4 Guard/Advisor Supervision

**Scope**: Add failure taxonomy, profile snapshots, step/degradation signals, and targeted
interventions such as retry, failover, context refresh, branch isolation, or escalation.

**Outputs**: guard decision contract, failure-classification tests, stream-monitor integration
tests.

- **Depends on**: 10.1, Phase 9 `ModelEvent` / stream infrastructure
- **Crates**: `mister-smith-agents`, `mister-smith-llm`, `mister-smith-events`

### 10.5 Operator Autonomy View

**Scope**: Surface topology state, checkpoint lineage, context pressure, routing rationale, and
intervention history through typed observability outputs and app/operator surfaces.

**Outputs**: autonomy observability contract, CLI/app views, dashboard and alert definitions.

- **Depends on**: 10.2, 10.3, 10.4
- **Crates**: `mister-smith-app`, `mister-smith-events`, deploy assets

### 10.6 Bounded Delegation + Provenance

**Scope**: Enforce capability chain validity, expiry, revocation, and operator-visible provenance
for privileged autonomous actions.

**Outputs**: delegation/provenance contract, invalid-chain tests, auditability validation.

- **Depends on**: Phase 9.1 delegation-chain foundation, 10.5 operator-facing inspection
- **Crates**: `mister-smith-security`, `mister-smith-agents`, `mister-smith-app`

## Blockers and Deferred Work

### Prerequisites

- Phase 9 `ModelEvent` / dual-stream surfaces must remain stable enough to serve as Guard inputs
- Phase 9.1 security substrate must remain authoritative for message authenticity and identity
- Phase 6 persistence repositories must remain the backing store for memory-manager work

### Explicit Deferred Scope

- Learned routing and kNN / RouteLLM policy selection
- Guided decoding and speculative decoding beyond step-level supervisory signals
- Local inference, disaggregated serving, shared KV cache serving
- CRDT coordination, MPST session types, and full distributed consensus protocols
- Auction-based meta-orchestration and recursive MAS generation
- eBPF/ML-based anomaly detection and distributed backdoor monitoring

## Complexity Tracking

No constitution violations are required for this phase. The main complexity increase is explicit
execution-graph and autonomy-control-plane state, which is justified because:

- it formalizes structures the research corpus says are now the dominant performance lever
- it reuses existing persistence, security, and observability substrate instead of introducing
  parallel systems
- it uses a mature graph library for graph correctness instead of custom DAG logic

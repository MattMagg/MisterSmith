# Implementation Plan: Task-Shape-Aware Orchestration and Dynamic Team Sizing

**Branch**: `014-task-shape-aware-orchestration` | **Date**: 2026-03-16 | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from `/specs/014-task-shape-aware-orchestration/spec.md`

## Summary

`main` already contains the first `MS-45` slice: task-shape classification, topology selection,
and topology rationale. This packet turns that baseline into a full feature by freezing one shared
adaptive-team contract, then implementing dynamic team sizing, operator-visible rendering, and a
repeatable evaluation harness without widening scope into a new runtime or a new roadmap phase.

## Technical Context

**Language/Version**: Rust 1.88.0  
**Primary Dependencies**: Tokio 1.49.x, DashMap, Serde, existing agents/events/app crates, current
topology and autonomy value objects  
**Storage**: Existing in-memory event bus plus durable evaluation artifacts under `docs/plans/`  
**Testing**: `cargo test -p mister-smith-agents`, `cargo test -p mister-smith-events`,
`cargo test -p mister-smith-app`, `cargo build --workspace`, plus deterministic evaluation-harness
evidence  
**Target Platform**: Local macOS development and Linux runtime parity  
**Project Type**: Rust workspace with orchestration runtime, event layer, and operator CLI  
**Performance Goals**: size active teams from task structure before dispatch, preserve current
topology behavior, and show measurable improvement or an honest neutral result on representative
workloads  
**Constraints**: preserve current one-shot runtime and autonomy surfaces, keep scope inside
`MS-45`, treat shared contract files as single-owner choke points, and avoid queue staging during
packet creation  
**Scale/Scope**: one feature packet, three bounded stories, with `MS-60` already landed and
`MS-61`/`MS-62` remaining

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | The packet treats `MS-45` as one feature and points Linear back to one governing spec path. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and `analyze.md` are written before implementation. |
| III. Phase-Gated Build Order | PASS | This is a post-`013` feature packet on the landed Phase 10 substrate, not a claim that a new numbered roadmap phase is already implemented. |
| IV. Model-Agnostic Architecture | PASS | Adaptive sizing and evaluation are provider-neutral and may use deterministic fixtures. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The design keeps the current orchestrator, scheduler, checkpoint, and supervision seams rather than replacing them. |
| VI. Evidence-Based Validation | PASS | The packet requires targeted crate tests plus a durable evaluation artifact under `docs/plans/`. |
| VII. Explicit Dependency Management | PASS | Parallel work is allowed only after a shared contract checkpoint and only across disjoint write sets. |

## Project Structure

### Documentation (this feature)

```text
specs/014-task-shape-aware-orchestration/
├── spec.md                              # Feature specification
├── plan.md                              # This file
├── research.md                          # Research decisions and repo grounding
├── data-model.md                        # Adaptive-team decision and evidence entities
├── quickstart.md                        # Validation and operator walkthrough scenarios
├── contracts/
│   └── adaptive-orchestration-surface.md
├── tasks.md                             # Execution order and parallel-lane rules
└── analyze.md                           # Cross-artifact consistency check
```

### Source Code (repository root)

```text
crates/mister-smith-core/
├── src/autonomy.rs                      # Shared adaptive-team value objects
├── src/lib.rs                           # Re-exports for shared decision types
└── tests/trait_compilation_tests.rs     # Trait/serde coverage for new shared types

crates/mister-smith-events/
├── src/autonomy.rs                      # Operator-visible sizing summaries
├── src/bus.rs                           # Adaptive decision aggregation into status views
└── tests/autonomy_event_tests.rs        # Event/status coverage

crates/mister-smith-agents/
├── src/orchestrator.rs                  # Adaptive sizing decision + status assembly choke point
├── src/team.rs                          # Adaptive team membership plan
├── src/scheduler.rs                     # Lifecycle integration for selected worker sets
├── tests/team_sizing_tests.rs           # Dynamic sizing coverage
└── tests/team_sizing_benchmark_tests.rs # Deterministic evaluation harness

crates/mister-smith-app/
├── src/autonomy.rs                      # Operator rendering of adaptive-team decisions
└── tests/autonomy_status_tests.rs       # CLI/status rendering coverage

docs/plans/
└── <dated>-ms-45-adaptive-orchestration-evaluation.md
```

**Structure Decision**: keep adaptive sizing inside the existing agents/events/app seams. The
feature extends the current execution-graph and autonomy status path; it does not justify a new
crate or a second operator subsystem.

## Design Decisions

### D1: Treat `MS-60` As Current Truth

**Decision**: the packet records task-shape classification and topology rationale as landed
baseline work rather than future implementation scope.

**Rationale**: `MS-60` is already on `main`, and re-specifying it as unfinished would make later
execution planning dishonest.

### D2: Freeze One Shared Adaptive-Team Contract Before Parallel Work Starts

**Decision**: define shared adaptive-team types and operator-visible status fields once in a serial
checkpoint across `core`, `events`, and `orchestrator` seams.

**Rationale**: this is the shared choke point for the feature; parallel work is only safe after
those fields stop moving.

### D3: Reuse Team And Scheduler Seams Instead Of Inventing A New Team Service

**Decision**: put adaptive worker-count logic into the existing team assembly and scheduler
lifecycle seams.

**Rationale**: the runtime already records team membership and worker loads, so the gap is policy
and lifecycle integration, not a missing subsystem.

### D4: Extend Workflow Autonomy Status Instead Of Creating A New Report Surface

**Decision**: render adaptive-team decisions through the current autonomy status view.

**Rationale**: operators already inspect workflow behavior through autonomy status. A second report
surface would duplicate the control plane.

### D5: Prove The Feature With Deterministic Evaluation First

**Decision**: require a deterministic evaluation harness and durable evidence note before optional
live-provider proof.

**Rationale**: the sizing contract is structural and can be validated without turning every
comparison run into an environment-dependent runtime proof.

## Minimal Implementation Slice

### Milestone 1: Adaptive-Team Contract Freeze

**Scope**: add the shared decision model and operator-visible status shape for adaptive team
sizing.

**Validation**:

- `cargo test -p mister-smith-events`
- `cargo test -p mister-smith-app`

### Milestone 2: Dynamic Team Sizing Runtime Lane (`MS-61`)

**Scope**: implement adaptive worker selection and lifecycle integration inside existing team and
scheduler seams while preserving current topology and routing behavior.

**Validation**:

- `cargo test -p mister-smith-agents`
- targeted adaptive-team tests showing different worker counts across workload shapes

### Milestone 3: Operator Status And Evaluation Lane (`MS-62`)

**Scope**: render adaptive-team decisions for operators and add the deterministic evaluation
harness plus durable evidence note.

**Validation**:

- `cargo test -p mister-smith-events`
- `cargo test -p mister-smith-app`
- deterministic evaluation artifact under `docs/plans/`

### Milestone 4: Cross-Crate Safety

**Scope**: confirm the packet integrates cleanly with the current workspace.

**Validation**:

- `cargo build --workspace`

## Parallel Symphony Staging Posture

Do **not** treat `MS-61` and `MS-62` as unrestricted parallel lanes.

Safe order:

1. serial contract-freeze checkpoint in shared files
2. bounded runtime lane for adaptive sizing
3. bounded operator-status lane
4. bounded evaluation lane

Allowed concurrency only begins after the shared contract checkpoint lands.

At that point the safe disjoint write sets are:

- runtime sizing lane:
  `crates/mister-smith-agents/src/team.rs`,
  `crates/mister-smith-agents/src/scheduler.rs`,
  `crates/mister-smith-agents/tests/team_sizing_tests.rs`
- operator-status lane:
  `crates/mister-smith-app/src/autonomy.rs`,
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- evaluation lane:
  `crates/mister-smith-agents/tests/team_sizing_benchmark_tests.rs`,
  one dated evidence note under `docs/plans/`

The following remain single-owner choke points throughout the packet:

- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- the active `docs/plans/...` evidence note
- the parent `MS-45` workpad

## Explicitly Deferred

- widening the packet into `MS-46`, `MS-47`, or `MS-48`
- inventing a new persistent team-management subsystem
- new workflow endpoints or a separate operator dashboard just for team sizing
- requiring live provider proof for every evaluation comparison

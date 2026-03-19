# Implementation Plan: Complex Multi-Agent Proof and Unified Result Surfaces

**Branch**: `015-complex-multi-agent-proof-and-unified-result-surfaces` | **Date**: 2026-03-19 |
**Spec**: [spec.md](spec.md)  
**Input**: Feature specification from
`/specs/015-complex-multi-agent-proof-and-unified-result-surfaces/spec.md`

## Summary

`main` already contains the live supervised planner and executor path, the `tool_bus` execution
boundary, strong topology and routing visibility, bounded MCP capability discovery and enforcement,
and real result plumbing across task and session storage. This packet does not create a new final
answer feature. It freezes one shared result contract, defines a three-outcome harder-workload
proof matrix, adapts the default runtime path only as needed for honest proof, and extends task,
session, and operator projections so final-result evidence is inspectable and trustworthy.

## Technical Context

**Language/Version**: Rust 1.88.0  
**Primary Dependencies**: Tokio 1.49.x, existing app, agents, events, core, and MCP crates  
**Storage**: existing task records, workflow metadata, retained session context, and durable proof
artifacts under `docs/plans/`  
**Testing**: `cargo test -p mister-smith-agents`, `cargo test -p mister-smith-events`,
`cargo test -p mister-smith-app`, `cargo build --workspace`, plus durable evaluation artifacts  
**Target Platform**: local macOS development and Linux runtime parity  
**Project Type**: Rust workspace with runtime, operator CLI, HTTP surfaces, and event projection
layers  
**Performance Goals**: honest harder-workload proof under the default live path, explicit outcome
classification, and inspectable result evidence without broad platform expansion  
**Constraints**: preserve March 19 baseline truth, keep the result contract as the first blocking
checkpoint, avoid reopening provider, KV, budget, or broad external-agent programs, and require
MCP non-regression only if new result surfaces intersect the bounded capability surface  
**Scale/Scope**: one feature packet, four bounded stories, with current live-path substrate already
landed

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | The packet treats the March 19 checkpoint as the forward authority and freezes one result contract before follow-on work. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and `analyze.md` are written before implementation. |
| III. Phase-Gated Build Order | PASS | This is the next bounded post-`014` packet layered on landed Phase 10 and March 19 runtime proof. |
| IV. Model-Agnostic Architecture | PASS | Provider-neutral routing remains deferred; the packet focuses on proof and result contract on the current live path. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The design extends the current runtime, graph, and projection seams rather than replacing them. |
| VI. Evidence-Based Validation | PASS | The packet requires targeted crate tests plus durable proof artifacts for success, collapse, and failure-visible outcomes. |
| VII. Explicit Dependency Management | PASS | Parallel work starts only after the shared result contract and proof taxonomy are frozen. |

## Project Structure

### Documentation (this feature)

```text
specs/015-complex-multi-agent-proof-and-unified-result-surfaces/
├── spec.md                              # Feature specification
├── plan.md                              # This file
├── research.md                          # Research decisions and repo grounding
├── data-model.md                        # Result contract, projections, and proof outcome entities
├── quickstart.md                        # Validation and proof walkthrough scenarios
├── contracts/
│   └── result-surface-contract.md       # Contract mapping for existing result forms
├── tasks.md                             # Execution order and parallel-lane rules
└── analyze.md                           # Cross-artifact consistency check
```

### Source Code (repository root)

```text
crates/mister-smith-core/
├── src/autonomy.rs                      # Shared result and proof-outcome value objects
├── src/lib.rs                           # Re-exports for shared contract types
└── tests/trait_compilation_tests.rs     # Shared contract coverage

crates/mister-smith-events/
├── src/autonomy.rs                      # Typed operator-facing result summaries
├── src/bus.rs                           # Result preview and proof-outcome aggregation
└── tests/autonomy_event_tests.rs        # Event/projection coverage

crates/mister-smith-app/
├── src/execution.rs                     # Canonical runtime result object and task-facing envelope
├── src/conversation.rs                  # Session-facing retained result view
├── src/autonomy.rs                      # Operator-facing result preview and provenance rendering
└── tests/autonomy_status_tests.rs       # Task and operator status rendering coverage

crates/mister-smith-agents/
├── src/orchestrator.rs                  # Harder-workload graph/result proof assembly
├── tests/gate10_tests.rs                # Runtime path guardrails
├── tests/step_routing_benchmark_tests.rs
└── tests/team_sizing_benchmark_tests.rs # Workload proof matrix and evidence hooks

docs/plans/
└── 2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md
```

**Structure Decision**: keep this feature inside the existing app, agents, core, and events seams.
The packet extends current runtime and operator surfaces; it does not justify a new crate or a
parallel operator subsystem.

## Design Decisions

### D1: Freeze March 19 Current Truth Before Designing Anything Else

**Decision**: treat the March 19 checkpoint, live-run trace, short multi-agent evaluation, stress
test, and `MS-77` note as the authoritative baseline for this packet.

**Rationale**: the packet must extend what is already true on `main`, not re-open stale direction
docs or older backlog framing.

### D2: Make The Shared Result Contract The First Blocking Checkpoint

**Decision**: freeze the relationship between `task.result`, metadata `final_result`, nested
`aggregated_result`, session `assistant_result`, and operator preview/provenance before parallel
work starts.

**Rationale**: this is the main shared choke point; later runtime and projection work should not
invent incompatible result shapes.

### D3: Define The Workload-Proof Matrix Before Runtime Tuning

**Decision**: classify proof outcomes as `graph_formed_and_completed`,
`collapsed_to_sequential`, and `failed_before_graph` before describing any runtime-path changes.

**Rationale**: the epic is about honest proof boundaries, not just “make it use more workers.”

### D4: Widen The Default Runtime Path Only As Needed For Honest Proof

**Decision**: describe runtime-path changes only as needed to produce harder-workload success,
collapse, and failure-visible evidence on the default live path.

**Rationale**: the checkpoint does not require a generic runtime rewrite or a worker-count program.

### D5: Extend Existing Surfaces Instead Of Creating New Ones

**Decision**: task, session, and autonomy surfaces remain the proof-relevant surfaces for this
packet.

**Rationale**: current code already stores and renders most of the needed structure; the missing
piece is consistent result contract and bounded projection.

### D6: Keep Broader External-Agent Work Deferred

**Decision**: treat broader post-`MS-77` external-agent work as a later bounded epic unless the
result-surface changes intersect the already-bounded MCP surface.

**Rationale**: `MS-77` already delivered the bounded discovery and enforcement surface on the MCP
boundary, and the March 19 checkpoint makes harder-workload proof the next mainline gap.

## Minimal Implementation Slice

### Milestone 1: Freeze Current Truth And The Shared Result Contract

**Scope**: add shared result and proof-outcome value objects plus the authoritative mapping between
existing result forms.

**Validation**:

- `cargo test -p mister-smith-events`
- `cargo test -p mister-smith-app`

### Milestone 2: Define The Workload-Proof Matrix

**Scope**: encode success, collapse, and failure-visible workload classes and their expected stored
evidence shapes.

**Validation**:

- `cargo test -p mister-smith-agents`
- deterministic benchmark or fixture coverage for all three outcome classes

### Milestone 3: Adapt The Default Runtime Path Only As Needed For Honest Harder-Workload Proof

**Scope**: extend current runtime graph and result assembly only where needed to prove harder
default-path behavior honestly.

**Validation**:

- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-app`
- targeted proof-path coverage in benchmark and gate tests

### Milestone 4: Project Unified Result Surfaces On Task, Session, And Operator Views

**Scope**: surface the canonical result contract consistently across task status, retained session
views, and operator preview/provenance rendering.

**Validation**:

- `cargo test -p mister-smith-events`
- `cargo test -p mister-smith-app`

### Milestone 5: Add Evaluation Artifacts And Cross-Crate Safety Checks

**Scope**: capture durable proof artifacts and confirm clean workspace integration.

**Validation**:

- durable artifact under `docs/plans/`
- `cargo build --workspace`
- MCP non-regression only if the touched surfaces intersect the bounded capability path

## Parallel Symphony Staging Posture

Do **not** treat this packet as unrestricted parallel work.

Safe order:

1. serial current-truth and result-contract freeze
2. serial proof-outcome taxonomy freeze
3. bounded runtime proof-path lane
4. bounded result projection lane
5. bounded evaluation-artifact lane

Allowed concurrency only begins after the shared result contract and proof taxonomy are frozen.

At that point the safe disjoint write sets are:

- runtime proof-path lane:
  `crates/mister-smith-app/src/execution.rs`,
  `crates/mister-smith-agents/src/orchestrator.rs`,
  `crates/mister-smith-agents/tests/gate10_tests.rs`,
  `crates/mister-smith-agents/tests/step_routing_benchmark_tests.rs`,
  `crates/mister-smith-agents/tests/team_sizing_benchmark_tests.rs`
- result projection lane:
  `crates/mister-smith-app/src/conversation.rs`,
  `crates/mister-smith-app/src/autonomy.rs`,
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- evaluation lane:
  `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`

The following remain single-owner choke points throughout the packet:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-core/src/lib.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-app/src/execution.rs` when the result contract is still moving
- `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/contracts/result-surface-contract.md`
- the active `docs/plans/...` evaluation artifact

## Explicitly Deferred

- provider-neutral routing or router-behavior expansion
- JetStream KV or budget-control follow-up
- broader external-agent interoperability or A2A-style surface expansion
- new operator endpoints or a second result-inspection subsystem
- generalized performance programs beyond the bounded harder-workload proof matrix

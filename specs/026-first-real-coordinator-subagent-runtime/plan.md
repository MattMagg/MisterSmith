# Implementation Plan: First Real Coordinator-Subagent Runtime

**Branch**: `026-first-real-coordinator-subagent-runtime` | **Date**: 2026-04-03 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/026-first-real-coordinator-subagent-runtime/spec.md`

## Summary

The repo already has packet `022` durable workflow ownership, packet `023` runtime truth and proof
projection, packet `024` agent-boundary hardening, packet `025` step-policy summaries, same-agent
session continuity, and the smallest-workflow rule on `main`.

The legitimate next slice is to freeze and implement one bounded coordinator-runtime layer on top
of those foundations. Packet `026` will add visible coordinator-owned delegation records, visible
subordinate inbox activity, stable delegated child identity, grounded delegated work evidence,
visible coordinator feedback and merge or recovery decisions, and one honest proof view projected
through existing task, autonomy, and run-detail surfaces.

This packet is implementation-ready and is the next active `/speckit.implement` packet for the
first real coordinator-subagent runtime on current `main`.

## Technical Context

**Language/Version**: Rust 1.88.0 plus existing operator-console TypeScript
**Primary Dependencies**: `mister-smith-core`, `mister-smith-agents`, `mister-smith-app`,
`mister-smith-events`, packet `022` durable workflow outputs, packet `023` runtime-truth outputs,
packet `024` boundary-hardening outputs, packet `025` step-policy outputs, and the operator-console
selected run detail
**Storage**: existing workflow metadata, autonomy-status projections, event-bus previews, and
task-result payloads only in the first slice; no new packet-owned service or store
**Testing**: targeted Rust tests in `mister-smith-core`, `mister-smith-agents`,
`mister-smith-events`, and `mister-smith-app`, operator-console test and build checks, Speckit
prerequisite validation, markdown lint, and diff hygiene
**Target Platform**: local macOS development with Linux parity for the shipped app binary
**Project Type**: Rust workspace packet with bounded operator-proof projection
**Performance Goals**: preserve the smallest-workflow rule, avoid fake fan-out overhead on
sequential work, and keep child-state plus proof summaries readable without log archaeology
**Constraints**: packet `022` through `025` keep their ownership; no federation or discovery
scope; no new endpoint; no live-proof claim without a later rerun
**Scale/Scope**: one bounded coordinator-runtime packet layered onto current graph, autonomy,
task-result, and operator-detail seams

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, `docs/direction.md`, specs `022` through `025`, the session-context report, the OpenClaude backlog, and named code seams. |
| II. Spec-First Design | PASS | Packet `026` has `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and `analyze.md` before implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | The packet is bounded to coordinator-runtime visibility on top of landed packet `022` through `025` foundations. |
| IV. Model-Agnostic Architecture | PASS | The first slice is about coordinator-runtime semantics and proof boundaries, not provider-specific logic. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Coordinator-owned child state, collapse, clarify, and sibling-abort outcomes stay explicit instead of hiding failure under graph completion. |
| VI. Evidence-Based Validation | PASS | The packet keeps deterministic implementation validation separate from any fresh live rerun claim. |
| VII. Explicit Dependency Management | PASS | Packet `022`, `023`, `024`, and `025` ownership boundaries are explicit in the packet docs and task map. |
| VIII. Clean Closure And Resumability | PASS | The packet bundle is ready for `/speckit.implement` and keeps the next work bounded to exact write seams and validation gates. |

## Project Structure

```text
specs/026-first-real-coordinator-subagent-runtime/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── coordinator-subagent-runtime-contract.md
├── checklists/
│   ├── requirements.md
│   └── scaffold.md
├── tasks.md
└── analyze.md

crates/mister-smith-core/
├── src/autonomy.rs
└── src/lib.rs

crates/mister-smith-agents/
├── src/execution_graph.rs
├── src/orchestrator.rs
├── src/roles/coordinator.rs
├── src/roles/planner.rs
├── src/roles/worker.rs
├── src/roles/critic.rs
└── tests/
    ├── execution_graph_tests.rs
    └── team_tests.rs

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
├── src/conversation.rs
└── tests/
    ├── autonomy_status_tests.rs
    └── effect_boundary_projection_tests.rs

crates/mister-smith-events/
├── src/autonomy.rs
├── src/bus.rs
└── tests/autonomy_event_tests.rs

apps/operator-console/
├── src/types.ts
├── src/views/RunsView.tsx
└── src/App.test.tsx
```

## Design Decisions

### D1: Extend landed runtime truth instead of reopening upstream packets

Packet `026` consumes packet `022` durability, packet `023` proof and run-trace, packet `024`
boundary hardening, and packet `025` step policy by reference. It does not redefine those seams.

### D2: Real coordinator-subagent runtime means more than graph completion

Packet `026` only counts as successful when a run shows coordinator-owned delegation, visible child
state, grounded delegated work, and visible coordinator feedback or merge decisions.

### D3: The smallest-workflow rule stays in force

Real coordinator-runtime behavior must still allow honest sequential collapse when the task does
not justify fan-out.

### D4: Existing read surfaces stay canonical

Task result, autonomy status, and operator selected-run detail remain the only required read
surfaces for the first slice. No new endpoint or dashboard is introduced.

### D5: The first slice uses bounded child roles and private child context

The first slice keeps child execution role-bounded, starts with explorer or planner or
verifier-style profiles, and shares only root-owned registration, cancellation, runtime-truth, and
capability-enforcement channels.

## Minimal Implementation Slice

### Milestone 0: Completed packet truth sync

Deliverables:

- packet `026` docs refreshed from scaffold wording to implementation-ready packet authority
- stale packet-prep and worktree-local paths removed
- repo router docs updated so packet `026` is the next implementation-ready packet

Validation:

- `spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`,
  `tasks.md`, `checklists/`, and `analyze.md` all match current repo truth
- `docs/current-state.md` and `docs/direction.md` agree on packet `026` as the next
  `/speckit.implement` packet

### Milestone 1: Freeze the shared coordinator-runtime contract

Deliverables:

- packet-owned delegation, subordinate inbox, child state, delegated evidence, coordinator
  decision, and proof-view value objects
- one shared payload contract for task result, autonomy status, and run detail

Validation:

- `spec.md`, `data-model.md`, `contracts/coordinator-subagent-runtime-contract.md`, and
  `tasks.md` agree on the same entities, ownership boundaries, and surface contract
- packet checklists are complete

### Milestone 2: Add visible delegation and child state

Deliverables:

- coordinator-owned delegation records on the runtime path
- ordered subordinate inbox intake
- stable delegated child identity and visible child state transitions

Validation:

- targeted runtime tests prove at least one visible delegation record and two child state
  transitions
- sequential collapse remains honest and visible

### Milestone 3: Add grounded delegated work and feedback loops

Deliverables:

- grounded delegated work evidence references
- visible clarify, reassign, stop, merge, and collapse decisions
- deterministic sibling-cancel and user-interrupt outcomes

Validation:

- targeted runtime tests prove one grounded delegated-work path
- placeholder-only delegated completion remains clearly non-grounded

### Milestone 4: Project one honest proof story through current surfaces

Deliverables:

- task result, autonomy status, and operator run detail expose one shared packet-owned proof view
- session-aware follow-up references stay stable and bounded

Validation:

- all three surfaces tell the same proof story
- final packet note keeps deterministic checks separate from live proof

## Parallel Staging Posture

Use bounded parallel work only after the shared contract freeze is complete.

- Blocking freeze before later lanes:
  - `specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`
  - `crates/mister-smith-core/src/autonomy.rs`
- Allowed disjoint lanes after the freeze:
  - graph and delegation lane: `crates/mister-smith-agents/src/execution_graph.rs` and
    `crates/mister-smith-agents/tests/execution_graph_tests.rs`
  - coordinator-runtime lane: `crates/mister-smith-agents/src/orchestrator.rs`,
    `crates/mister-smith-agents/src/roles/coordinator.rs`, and
    `crates/mister-smith-agents/src/roles/executor.rs`
  - child-role lane: `crates/mister-smith-agents/src/roles/planner.rs`,
    `crates/mister-smith-agents/src/roles/worker.rs`, `crates/mister-smith-agents/src/roles/critic.rs`,
    and `crates/mister-smith-agents/tests/team_tests.rs`
  - projection lane: `crates/mister-smith-events/src/autonomy.rs`,
    `crates/mister-smith-events/src/bus.rs`, `crates/mister-smith-app/src/autonomy.rs`, and
    `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - proof and follow-up lane: `crates/mister-smith-app/src/execution.rs`,
    `crates/mister-smith-app/src/conversation.rs`, and
    `crates/mister-smith-app/tests/effect_boundary_projection_tests.rs`
  - operator lane: `apps/operator-console/src/types.ts`,
    `apps/operator-console/src/views/RunsView.tsx`, and `apps/operator-console/src/App.test.tsx`
- Single-owner choke points:
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-agents/src/orchestrator.rs`
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-events/src/bus.rs`
  - `apps/operator-console/src/views/RunsView.tsx`

## Explicitly Deferred

- federation, capability discovery, or generic interoperability work
- secret-minimized remote worker bridges and other remote-executor work
- default fan-out or fixed multi-worker topology
- packet `022`, `023`, `024`, or `025` ownership changes
- a new endpoint, broader operator-console redesign, or generic shell parity work
- live runtime-proof claims before implementation lands and a later bounded rerun is executed

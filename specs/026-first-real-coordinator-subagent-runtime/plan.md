# Implementation Plan: First Real Coordinator-Subagent Runtime

**Branch**: `026-first-real-coordinator-subagent-runtime` | **Date**: 2026-04-01 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/026-first-real-coordinator-subagent-runtime/spec.md`

## Summary

This is a scaffold plan written before packets `022` through `025` are complete. Its job is to
lock the packet `026` shape now, not to pretend the packet is ready to implement today.

Packet `026` is the first bounded packet that should make the local runtime feel like a real
coordinator-subagent system. That means visible coordinator-owned delegation, visible subagent
state, grounded delegated work, visible feedback and merge or recovery loops, and explicit proof
text that says when those things were not actually present.

Before implementation starts, this plan must go through one explicit revision gate so the packet
matches the real landed outputs of packets `022` through `025`.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned TypeScript for the operator console
**Primary Dependencies**: `mister-smith-core`, `mister-smith-agents`, `mister-smith-app`,
`mister-smith-events`, `apps/operator-console/`, and the still-in-progress packet `022` through
`025` outputs once they land
**Storage**: existing task, autonomy, and session state plus the durable workflow and trace seams
owned by packets `022` and `023`; exact final storage wording is deferred to the revision gate
**Testing**: targeted Rust tests, task and autonomy projection tests, operator-console view tests,
markdown lint, and `git diff --check`
**Target Platform**: local macOS development with Linux runtime parity for the shipped app binary
**Project Type**: Rust workspace packet with one bounded operator-console surface
**Performance Goals**: preserve the smallest-workflow rule, avoid fake delegation overhead on
sequential work, and keep proof surfaces readable without log archaeology
**Constraints**: no implementation before the revision gate; no federation or discovery scope; do
not redefine packet `022` through `025` ownership; do not claim live proof from scaffold-only work
**Scale/Scope**: one bounded scaffold packet and one later implementation packet after upstream
reconciliation

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, the packet-prep dossiers, and the session-context report. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `tasks.md`, and `analyze.md` are authored before any implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | Packet `026` is explicitly scaffolded behind a revision gate so it cannot silently bypass packets `022` through `025`. |
| IV. Model-Agnostic Architecture | PASS | The packet focuses on coordinator-runtime semantics, not provider-specific behavior. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet keeps coordinator-owned recovery and collapse decisions explicit instead of hiding them inside opaque graph success. |
| VI. Evidence-Based Validation | PASS | The scaffold states clear proof boundaries and requires a later refresh before any implementation or live-proof claim. |
| VII. Explicit Dependency Management | PASS | Upstream ownership from packets `022` through `025` is consumed by reference and called out directly. |
| VIII. Clean Closure And Resumability | PASS | The scaffold is written to be revised later without reauthoring packet scope from scratch. |

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
└── src/roles/executor.rs

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
└── src/conversation.rs

crates/mister-smith-events/
└── src/bus.rs

apps/operator-console/
├── src/views/RunsView.tsx
└── src/types.ts
```

## Design Decisions

### D1: Scaffold now, revise before implementation

Freeze the packet goal, non-goals, proof standard, and artifact shape now, but require a revision
gate before any implementation begins.

### D2: Real coordinator-subagent runtime needs more than graph success

Packet `026` only counts as successful when a run shows coordinator-owned delegation records, real
subagent state, grounded delegated work, and visible coordinator merge or recovery decisions.

### D3: The smallest-workflow rule stays in force

Real coordinator-runtime behavior must still allow honest sequential collapse when the task does
not justify fan-out.

### D4: Packet `022` through `025` keep their ownership

Packet `026` consumes upstream packet ownership for durability, trace truth, security boundaries,
and step policy. It does not redefine those seams.

### D5: Operator evidence stays on existing surfaces

The packet stays bounded to task result, autonomy status, and operator-console run detail.

## Minimal Implementation Slice

### Milestone 0: Revision gate and upstream reconciliation

Validation:

- `spec.md`, `plan.md`, `tasks.md`, and `analyze.md` all reflect the reconciled upstream truth
- no packet `026` artifact still assumes upstream completion that did not actually land

### Milestone 1: Shared contract freeze

Validation:

- `contracts/coordinator-subagent-runtime-contract.md` is final for the implementation pass
- shared value objects and proof-boundary wording stop moving

### Milestone 2: Visible delegation and subagent state

Validation:

- bounded runtime tests show delegation records and state transitions
- sequential collapse stays honest and visible

### Milestone 3: Grounded delegated work and feedback loops

Validation:

- bounded runtime tests prove grounded delegated work
- placeholder-only delegated completion remains clearly non-grounded

### Milestone 4: Operator proof surfaces and final evidence

Validation:

- all three operator surfaces tell the same proof story
- final packet note keeps deterministic checks separate from live proof

## Parallel Staging Posture

- No implementation lane may start until Milestone 0 is complete.
- Milestone 1 is a single-owner freeze.
- After Milestone 1, bounded lanes may split if write sets are disjoint.
- Shared-write choke points:
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-agents/src/orchestrator.rs`
  - `crates/mister-smith-agents/src/execution_graph.rs`
  - `crates/mister-smith-core/src/autonomy.rs`
  - `apps/operator-console/src/views/RunsView.tsx`

## Explicitly Deferred

- federation, capability discovery, and generic interoperability work
- any new protocol baseline or capability-mapping contract
- default fan-out or fixed multi-worker shapes
- any attempt to replace packet `022` through `025` ownership
- live proof claims before implementation and the revision gate are complete

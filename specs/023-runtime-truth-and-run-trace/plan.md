# Implementation Plan: Runtime Truth And Run Trace

**Branch**: `023-runtime-truth-and-run-trace` | **Date**: 2026-04-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`specs/023-runtime-truth-and-run-trace/spec.md`

## Summary

Packet `023` makes runtime truth honest and consistent across current result surfaces. The work is
bounded to one new packet-023-owned `runtime_truth` contract, one bounded run-trace summary, one
canonical proof-boundary model, and one consistent projection path across task, session,
autonomy, and operator views. Packet `022` keeps lifecycle ownership. Packet `021` keeps
predictive-supervision ownership.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned packet docs and operator-console TypeScript
**Primary Dependencies**: `mister-smith-core`, `mister-smith-agents`, `mister-smith-events`,
`mister-smith-app`, `apps/operator-console`, and current proof notes under `docs/plans/`
**Storage**: Existing task, session, and autonomy metadata; no new durable packet-022 storage
surface in this slice
**Testing**: Targeted Rust tests, operator-console tests, smoke-harness unit tests, markdown lint,
and `git diff --check`
**Target Platform**: Current Mister Smith runtime-backed task path and current operator inspection
surfaces
**Project Type**: Rust workspace packet with bounded UI projection updates
**Performance Goals**: Honest operator understanding in one inspection pass; no broadened runtime
claim surface without corresponding proof
**Constraints**: Keep packet `022` as lifecycle/history owner; keep packet `021`
`supervision_evidence` separate; preserve packet `019` / `020` live-proof split versus packet
`021` / `022` deterministic-only proof; treat OpenTelemetry and W3C docs as taxonomy guidance
only; do not widen scope; do not widen `MessageEnvelope`
**Scale/Scope**: One bounded packet focused on naming, taxonomy, proof boundaries, and projection

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, packet `022` closure, packet `021` closure, and current runtime seams. |
| II. Spec-First Design | PASS | Packet `023` is now implementation-ready before code execution. |
| III. Phase-And-Packet-Gated Delivery | PASS | Packet `023` stays bounded and explicitly depends on packet `022` ownership instead of reopening it. |
| IV. Model-Agnostic Architecture | PASS | The packet describes proof boundaries and run-trace taxonomy, not provider-specific behavior. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | No fault-tolerance guarantees are widened here; current runtime boundaries stay explicit. |
| VI. Evidence-Based Validation | PASS | The packet preserves live-proof versus deterministic-only boundaries and does not invent a new live rerun. |
| VII. Explicit Dependency Management | PASS | Upstream dependency on packet `022` and existing packet `019` / `020` / `021` proof notes is explicit. |
| VIII. Clean Closure And Resumability | PASS | Runtime truth, docs, and projections remain cold-start readable and bounded. |

## Project Structure

### Documentation

```text
specs/023-runtime-truth-and-run-trace/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── run-trace-proof-boundary-contract.md
├── checklists/
│   ├── requirements.md
│   └── runtime-truth-proof-boundary.md
├── tasks.md
└── analyze.md
```

### Source Code

```text
crates/mister-smith-core/
└── src/autonomy.rs

crates/mister-smith-agents/
└── src/orchestrator.rs

crates/mister-smith-events/
└── src/bus.rs

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
└── src/conversation.rs

apps/operator-console/
├── src/types.ts
└── src/views/RunsView.tsx
```

**Structure Decision**: `mister-smith-core` owns the packet-023 types and canonical wording.
`mister-smith-agents` and `mister-smith-events` synthesize the shared block. `mister-smith-app`
and `apps/operator-console` project it. `mister-smith-transport` remains unchanged in the first
slice.

## Design Decisions

### D1: `runtime_truth` is a new packet-023-owned block

**Decision**: Add a new `RuntimeTruthView` instead of overloading packet-021
`supervision_evidence`.

**Rationale**: Predictive supervision and runtime truth are adjacent but different concepts.
Keeping them separate prevents packet-boundary drift and lets the operator see both stories side
by side.

### D2: Placeholder step completion is a first-class non-proof state

**Decision**: The current `workflow.execute_step` payload-echo behavior remains a placeholder
completion boundary that can prove orchestration-substrate flow but not grounded task work.

**Rationale**: Current runtime code marks `workflow.execute_step` payloads `completed` at the
`tool_bus` boundary without proving grounded execution. Packet `023` exists to keep that honest.

### D3: `workflow_id` is the run-trace root in the first slice

**Decision**: Use `workflow_id` as the canonical run anchor and reuse existing `trace_id` only as
input metadata when present. Do not widen `MessageEnvelope`.

**Rationale**: The packet needs a shared truthful summary, not a new transport schema or full
tracing platform.

### D4: Packet `019` and `020` live proof remain separate from packet `021` and `022` deterministic proof

**Decision**: Preserve the current proof split instead of collapsing all runtime evidence into one
broad “live” label.

**Rationale**: `docs/current-state.md`, packet `021` closure, and packet `022` closure make that
split explicit and current.

## Milestones

### Milestone 1: Packet authority and truth sync

**Scope**: Revise the packet docs to implementation-ready, finish the proof-boundary checklist,
and sync `docs/current-state.md` and `docs/direction.md` with current packet truth.

**Validation**:

- `npx markdownlint-cli2 "specs/023-runtime-truth-and-run-trace/**/*.md" --config .markdownlint.json`
- `git diff --check`

### Milestone 2: Core contract and synthesis

**Scope**: Add packet-023-owned runtime-truth types in `mister-smith-core`, then synthesize them
from current graph, repair, retry, and supervision state in `mister-smith-agents` and
`mister-smith-events`.

**Validation**:

- `cargo test -p mister-smith-core`
- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-events --test autonomy_event_tests`

### Milestone 3: Projection surfaces

**Scope**: Carry the new block through task, session, autonomy, and operator surfaces, with the
operator console rendering a separate Runtime truth panel.

**Validation**:

- `cargo test -p mister-smith-app --test autonomy_status_tests`
- `cargo test -p mister-smith-app workflow_step_tool_marks_payload_as_tool_bus_completed`
- `npm --prefix apps/operator-console test`
- `npm --prefix apps/operator-console run build`
- `python3 -m unittest scripts.tests.test_live_runtime_proof_smoke`

## Parallel Staging Posture

- Blocking first:
  - packet doc revision and checklist completion
  - packet-023 core type definition
- Parallel after the core types land:
  - agents/events synthesis lane
  - app/session/operator projection lane
- Shared-write choke points:
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-events/src/bus.rs`
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-app/src/autonomy.rs`
  - `docs/current-state.md`

## Explicitly Deferred

- any new `MessageEnvelope` schema field or tracing export format
- packet `022` lifecycle, event-history, compaction, or effect-boundary implementation changes
- generic observability-platform work
- UI redesign beyond a small runtime-truth projection panel
- coordinator-runtime, real subagent-runtime, or interoperability work
- any new live runtime-proof claim unless a real rerun is executed

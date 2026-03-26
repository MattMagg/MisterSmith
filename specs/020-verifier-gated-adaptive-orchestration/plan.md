# Implementation Plan: Verifier-Gated Adaptive Orchestration

**Branch**: `020-verifier-gated-adaptive-orchestration` | **Date**: 2026-03-26 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/020-verifier-gated-adaptive-orchestration/spec.md`

## Summary

This packet adds a workflow-quality control loop to the shipped runtime path. It keeps today's
planner/executor path intact while introducing verifier-gated step decisions, handoff
clarification, contextual repair, and operator-visible orchestration-quality provenance. It is
explicitly benchmark-oriented in outcome, but it does not widen into provider work, budget work,
or a benchmark harness in this packet.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned docs/spec artifacts
**Primary Dependencies**: `mister-smith-app`, `mister-smith-core`, existing task/autonomy
projection surfaces, and the current ToolBus-backed runtime path
**Storage**: existing workflow metadata, persistence, and runtime checkpoint state only; no new
budget store or provider infrastructure
**Testing**: targeted app/core tests for verifier verdicts, repair directives, and autonomy
inspection; docs lint and closure validation for this packet-freeze pass
**Target Platform**: local macOS and Linux parity for the current app binary
**Project Type**: Rust workspace packet plus repo state-router update
**Performance Goals**: improve long-horizon workflow quality by stopping weak intermediate steps
from cascading, reducing unnecessary full-task restarts, and making repair decisions observable
**Constraints**: no provider expansion, no budgeting expansion, no operator-console redesign, no
RL training stack, no workflow-evolution program
**Scale/Scope**: one bounded verifier and repair loop on the current runtime-backed task path

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in current-state, packet `019` closure, current runtime code, and repo-local research notes. |
| II. Spec-First Design | PASS | Packet `020` freezes the verifier and repair contract before implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | Follows packet `019` closure with one bounded workflow-quality slice instead of a broad research catch-up. |
| IV. Model-Agnostic Architecture | PASS | Packet targets workflow control semantics, not provider-specific behavior. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Clarification, retry, and checkpoint-based repair extend the supervision posture rather than replacing it. |
| VI. Evidence-Based Validation | PASS | Requires targeted deterministic validation and honest live-proof boundaries. |
| VII. Explicit Dependency Management | PASS | Write set is bounded to workflow state contracts, runtime execution, inspection surfaces, and docs. |
| VIII. Clean Closure And Resumability | PASS | Packet freeze lands as durable notes and task pack on `main` with router updates. |

## Project Structure

```text
specs/020-verifier-gated-adaptive-orchestration/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── analyze.md

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
├── src/agent_inspection.rs
└── tests/autonomy_status_tests.rs

crates/mister-smith-core/
├── src/autonomy.rs
├── src/enums.rs
└── src/supervision.rs

docs/
├── current-state.md
├── ms_recent_context.md
└── plans/2026-03-26-verifier-gated-adaptive-orchestration.md
```

## Design Decisions

### D1: Keep verification orchestrator-owned, not self-judged by the active executor

The active executor may generate the candidate step, but verdict ownership belongs to a separate
verifier surface so the workflow does not silently trust the same actor that produced the output.

### D2: Repair locally before restarting globally

When a bad step or handoff is detected, the first action should be bounded clarification, retry,
or re-plan from the last stable checkpoint. Full-task restart is a later escalation, not the
default response.

### D3: Start with workflow-step and handoff boundaries, not token-stream PRM infrastructure

The research points toward token-stream monitors and learned PRMs, but the first bounded slice
should operate on explicit workflow-step boundaries already visible to the runtime. This keeps the
implementation honest and contained.

## Minimal Implementation Slice

### Milestone 1: Freeze verifier and repair contract

Validation:

- packet `020` artifacts freeze step-verdict, clarification, and repair semantics
- router docs point at packet `020` as the next bounded phase

### Milestone 2: Add runtime verifier-gated execution path

Validation:

- targeted app/core tests for accept, reject, retry, clarify, re-plan, and stop outcomes
- current shipped happy path remains intact when the verifier loop is inactive

### Milestone 3: Extend provenance and evidence

Validation:

- task/autonomy inspection proves verifier and repair history is operator-visible
- proof note or deterministic transcript stays honest about benchmark and live-proof boundaries

## Parallel Staging Posture

- Blocking freeze before any parallel lanes: packet contract, data model, and router docs
- Allowed disjoint lanes after the freeze:
  - core decision types and checkpoint semantics: `crates/mister-smith-core/src/*`
  - app execution and inspection surfaces: `crates/mister-smith-app/src/*`,
    `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - state-bearing docs and proof notes: `docs/` and `specs/020-verifier-gated-adaptive-orchestration/`
- Single-owner choke points:
  - `crates/mister-smith-app/src/execution.rs`
  - `docs/current-state.md`
  - `docs/ms_recent_context.md`

## Explicitly Deferred

- provider or budget routing changes
- token-stream PRM infrastructure or RL-trained supervision policies
- workflow-evolution search, self-improvement archives, or benchmark harness construction
- any claim that this packet alone establishes a new benchmark score

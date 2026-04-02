# Implementation Plan: Step-Level Intelligence v2

**Branch**: `025-step-level-intelligence-v2` | **Date**: 2026-04-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/025-step-level-intelligence-v2/spec.md`

## Summary

Packets `022`, `023`, and `024` are landed on current `main`, and packet `025` should now treat
them as base layers instead of moving targets. The bounded next slice is to freeze one packet-owned
step-policy contract that:

- derives deterministic step difficulty from current runtime inputs
- chooses one bounded action across `keep`, `retry`, `clarify`, `downgrade`, and `escalate`
- carries budget-aware hints without reopening packet-022, packet-023, or packet-024 ownership
- projects the same summary through task, session, autonomy, and operator-facing result surfaces
- preserves packet-023 placeholder-versus-grounded proof wording on every surface

Packet `025` is now prepared for implementation on current `main`. It is no longer a
revision-later scaffold.

## Technical Context

**Language/Version**: Rust 1.88.0 plus existing operator-console TypeScript where current
result projections already exist
**Primary Dependencies**: `mister-smith-core`, `mister-smith-events`, `mister-smith-app`,
`mister-smith-persistence`, current packet `020` through packet `024` seams, and
`scripts/tests/test_live_runtime_proof_smoke.py`
**Storage**: existing workflow metadata and result projections only; no new packet-owned durable
store is required for the first slice
**Testing**: targeted Rust tests for deterministic scoring and projection, smoke-harness unit
coverage, bounded operator-console validation if UI files move, markdown lint, and diff hygiene
**Target Platform**: local macOS development with Linux runtime parity for the shipped app binary
**Project Type**: Rust workspace packet with bounded result-surface follow-on work
**Performance Goals**: produce one deterministic bounded action without regressing current fallback
behavior and expose that summary through current result surfaces
**Constraints**: packet `020` owns repair lineage, packet `022` owns durable workflow semantics,
packet `023` owns runtime truth, packet `024` owns boundary security, and the first slice stays
heuristic and deterministic
**Scale/Scope**: one bounded step-policy packet on top of current step-evaluation, routing,
budget-pressure, runtime-truth, and result-summary seams

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, `docs/direction.md`, the landed packet `022` through packet `024` specs, and the current runtime seams. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and `analyze.md` are prepared before any packet-025 implementation begins. |
| III. Phase-And-Packet-Gated Delivery | PASS | Keeps packet `025` bounded to step-policy work on top of landed packet `020` through packet `024` layers. |
| IV. Model-Agnostic Architecture | PASS | The first slice uses landed internal runtime signals and does not depend on provider-specific stream parsing or judge-heavy control loops. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The design composes with existing repair lineage, supervision evidence, durable lifecycle state, and current result-surface recovery flows. |
| VI. Evidence-Based Validation | PASS | The packet stays deterministic-only unless a later live rerun is actually produced and explicitly scoped. |
| VII. Explicit Dependency Management | PASS | The packet names the exact packet and code ownership boundaries it consumes and leaves unchanged. |
| VIII. Clean Closure And Resumability | PASS | The packet is prepared as a durable implementation-ready bundle with explicit validation and bounded follow-on lanes. |

## Project Structure

```text
specs/025-step-level-intelligence-v2/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── step-policy-contract.md
├── checklists/
│   ├── requirements.md
│   └── step-policy.md
├── tasks.md
└── analyze.md

crates/mister-smith-core/
└── src/autonomy.rs

crates/mister-smith-events/
└── src/autonomy.rs

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
└── tests/autonomy_status_tests.rs

scripts/tests/
└── test_live_runtime_proof_smoke.py

apps/operator-console/
├── src/types.ts
├── src/views/RunsView.tsx
└── src/App.test.tsx
```

## Design Decisions

### D1: Step policy derives from landed internal runtime seams

The first implementation slice should consume current `StepEvaluationRecord`,
`StepRoutingDecisionSummary`, budget or context-pressure summaries, supervision evidence, durable
lifecycle state, and packet-023 runtime truth instead of inventing a new raw event parser.

### D2: Packet `023` remains the owner of runtime truth and proof wording

Packet `025` can summarize current step posture, but it must not create a competing proof schema
or stronger wording than the landed packet-023 contract.

### D3: Packet `022` and packet `024` stay upstream ownership layers

Packet `025` may read durable lifecycle state from packet `022` and boundary decisions from packet
`024`, but it must not absorb durable-workflow semantics or security-boundary ownership.

### D4: Existing result surfaces remain canonical

`TaskResultView` and autonomy status remain the full canonical packet-owned read surfaces.
Session-retained and operator-preview surfaces remain bounded compact projections of the same
packet-owned summary.

### D5: The first slice is heuristic and deterministic

The first useful packet-025 implementation is not a PRM or benchmark program. It is one bounded
deterministic scoring and action ladder that operators and implementers can audit.

## Minimal Implementation Slice

### Milestone 1: Freeze the shared step-policy contract

Validation:

- `spec.md`, `data-model.md`, `contracts/step-policy-contract.md`, and `tasks.md` agree on the
  action vocabulary, budget summary, projection surfaces, and packet `020` through packet `024`
  ownership boundaries
- packet-025 checklists are complete and no scaffold-only wording remains

### Milestone 2: Add deterministic difficulty assessment and bounded action choice

Validation:

- targeted `mister-smith-core` and `mister-smith-app` coverage proves at least one `keep`
  decision and one non-`keep` decision from bounded deterministic inputs
- missing or inconclusive inputs preserve current fallback behavior

### Milestone 3: Project honest summaries through current result surfaces

Validation:

- task, session, autonomy, and operator result surfaces show the same packet-owned step-policy
  summary
- proof wording remains explicit that placeholder completion is not grounded task proof

### Milestone 4: Finish bounded validation and packet note sync

Validation:

- packet-owned docs stay aligned with the landed packet `022` through packet `024` truth
- deterministic test and lint coverage passes for the touched seams

## Parallel Staging Posture

Use only when the packet benefits from bounded parallel work.

- Blocking freeze before any parallel lanes:
  - shared packet-025 contract
  - packet `020` through packet `024` ownership boundaries
- Allowed disjoint lanes after the freeze:
  - core type lane: `crates/mister-smith-core/src/autonomy.rs`
  - event view lane: `crates/mister-smith-events/src/autonomy.rs`
  - runtime assembly lane: `crates/mister-smith-app/src/execution.rs`
  - result projection lane: `crates/mister-smith-app/src/autonomy.rs` and
    `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - optional operator lane: `apps/operator-console/`
- Single-owner choke points:
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-events/src/autonomy.rs`
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-app/src/autonomy.rs`
  - any packet-owned note under `docs/plans/`

## Explicitly Deferred

- new raw streaming-event parsing or provider-specific stream contracts
- packet `022` durable-workflow, packet `023` runtime-truth, or packet `024` boundary-policy
  changes
- grounded step execution beyond the current placeholder `workflow.execute_step` seam
- PRM training, benchmark programs, coordinator runtime, subagent runtime, or interoperability
  work

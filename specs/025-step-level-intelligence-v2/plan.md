# Implementation Plan: Step-Level Intelligence v2

**Branch**: `025-step-level-intelligence-v2` | **Date**: 2026-04-02 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/025-step-level-intelligence-v2/spec.md`

## Summary

The repo already has packet `019` budget-aware routing signals, packet `020` step evaluation and
repair lineage, packet `021` supervision evidence, packet `022` durable workflow ownership,
packet `023` runtime truth, and packet `024` agent-boundary hardening on `main`.

The legitimate next slice is to freeze and implement one bounded packet-owned step-policy layer on
top of those foundations. Packet `025` will define a deterministic difficulty assessment, a
bounded budget-pressure summary, and one action ladder across `keep`, `retry`, `clarify`,
`downgrade`, and `escalate`. It will project that summary through existing task inspect,
autonomy-status, and operator selected-run detail surfaces without claiming grounded task proof.

This packet is implementation-ready and is the next active `/speckit.implement` packet for
step-level policy on current `main`.

## Technical Context

**Language/Version**: Rust 1.88.0 plus existing operator-console TypeScript
**Primary Dependencies**: `mister-smith-core`, `mister-smith-app`, `mister-smith-events`,
packet `019` budget-aware routing outputs, packet `020` step-evaluation and repair lineage,
packet `021` supervision evidence, packet `023` runtime truth, and the operator-console selected
run detail
**Storage**: existing task-result metadata and autonomy-status projections only; no new
packet-owned persistence store in the first slice
**Testing**: targeted Rust tests in `mister-smith-core`, `mister-smith-app`, and
`mister-smith-events`, smoke-harness assertions, operator-console build and tests, markdown lint,
and diff hygiene
**Target Platform**: local macOS development with Linux parity for the shipped app binary
**Project Type**: Rust workspace packet with bounded result-surface projection
**Performance Goals**: deterministic step-policy assembly with no regression to current fallback
behavior and no new endpoint
**Constraints**: packet `020` keeps verifier and repair ownership; packet `023` keeps
runtime-truth and proof-boundary ownership; packet `025` stays deterministic; no new live proof
claim without a separate rerun
**Scale/Scope**: one bounded step-policy packet layered onto current step-evaluation, routing,
summary, and operator-detail seams

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, `docs/direction.md`, packet `019` through packet `024` closure notes, and the current code seams named in `spec.md`. |
| II. Spec-First Design | PASS | Packet `025` has `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and `analyze.md` before implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | The packet is bounded to stronger step policy on top of landed packet `019` through packet `024` foundations. |
| IV. Model-Agnostic Architecture | PASS | The first slice uses repo-native deterministic signals first and does not require a provider-specific training stack. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Packet `020` repair lineage and current bounded retry or clarify behavior remain the local-correction substrate. |
| VI. Evidence-Based Validation | PASS | The packet keeps deterministic validation separate from any fresh live rerun claim. |
| VII. Explicit Dependency Management | PASS | Packet `019`, `020`, `021`, `022`, `023`, and `024` ownership boundaries are explicit in the packet docs and task map. |
| VIII. Clean Closure And Resumability | PASS | The packet bundle is ready for `/speckit.implement` and keeps the next work bounded to exact write seams and validation gates. |

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

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
└── tests/autonomy_status_tests.rs

crates/mister-smith-events/
└── src/autonomy.rs

scripts/tests/
└── test_live_runtime_proof_smoke.py

apps/operator-console/
├── src/types.ts
├── src/views/RunsView.tsx
└── src/App.test.tsx
```

## Design Decisions

### D1: Use current repo signals first

Packet `025` should build on the current signals already present on `main`:
`StepEvaluationRecord`, `StepRoutingDecisionSummary`, packet-021 `supervision_evidence`,
packet-023 `runtime_truth`, and bounded budget-pressure hints already visible in routing and
autonomy state.

### D2: Keep packet ownership clean

Packet `020` keeps verifier and repair-lineage ownership. Packet `021` keeps supervision-evidence
ownership. Packet `023` keeps runtime-truth, proof-boundary, and run-trace ownership. Packet
`025` adds step policy beside those fields instead of replacing them.

### D3: The first slice is deterministic and bounded

The first useful packet is a deterministic step-policy contract, not a PRM training system or a
judge-heavy scoring stack. Follow-on learned policy work can happen later only after this bounded
contract is real.

### D4: Existing read surfaces stay canonical

Task inspect, autonomy status, and operator selected-run detail remain the read surfaces for the
first slice. No new endpoint or trace explorer is introduced.

### D5: Proof honesty stays unchanged

Packet `025` may explain the current step policy, but it may not upgrade placeholder
`workflow.execute_step` completion into grounded task proof. Packet-023 wording stays authoritative.

## Minimal Implementation Slice

### Milestone 1: Freeze the packet-owned step-policy contract

Deliverables:

- packet-owned `StepDifficultyAssessment`, `StepBudgetPressureSummary`, `StepPolicyDecision`, and
  `StepPolicySummaryView`
- one shared surface contract for task inspect, autonomy status, and operator selected-run detail
- repo router docs updated so packet `025` is no longer described as a draft scaffold

Validation:

- `spec.md`, `data-model.md`, `contracts/step-policy-contract.md`, and `tasks.md` agree on the
  same entities, ownership boundaries, and surface contract
- packet checklists are complete

### Milestone 2: Add deterministic step assessment and action selection

Deliverables:

- core packet-owned value objects and projections
- runtime assembly of deterministic step difficulty, budget summary, and chosen action
- clean fallback behavior when inputs are missing or inconclusive

Validation:

- targeted crate tests prove at least one `keep` decision and one non-`keep` decision
- packet `020`, `021`, and `023` fields remain separate and intact

### Milestone 3: Project honest summaries through existing surfaces

Deliverables:

- task inspect and autonomy status expose the packet-owned `step_policy` summary
- operator selected-run detail renders the new summary beside current runtime-truth and
  supervision-evidence panels
- smoke-harness assertions keep placeholder-versus-grounded wording honest

Validation:

- task/autonomy/operator surfaces show the same packet-owned fields
- packet-023 proof wording remains unchanged

## Parallel Staging Posture

Use bounded parallel work only after the shared contract freeze is complete.

- Blocking freeze before later lanes:
  - `specs/025-step-level-intelligence-v2/contracts/step-policy-contract.md`
  - `crates/mister-smith-core/src/autonomy.rs`
- Allowed disjoint lanes after the freeze:
  - runtime policy lane: `crates/mister-smith-app/src/execution.rs`
  - result projection lane: `crates/mister-smith-app/src/autonomy.rs` and
    `crates/mister-smith-events/src/autonomy.rs`
  - operator detail lane: `apps/operator-console/src/types.ts`,
    `apps/operator-console/src/views/RunsView.tsx`, and `apps/operator-console/src/App.test.tsx`
  - validation lane: `crates/mister-smith-app/tests/autonomy_status_tests.rs` and
    `scripts/tests/test_live_runtime_proof_smoke.py`
- Single-owner choke points:
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-app/src/autonomy.rs`
  - `crates/mister-smith-events/src/autonomy.rs`

## Explicitly Deferred

- PRM training or learned step-policy control loops
- new provider proof or provider expansion
- packet `023` runtime-truth or run-trace schema changes
- packet `020` verifier or repair-lineage redesign
- coordinator runtime, subagent runtime, interoperability, or benchmark work
- a new endpoint, trace explorer, or broader operator-console redesign

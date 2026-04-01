# Implementation Plan: Step-Level Intelligence v2

**Branch**: `025-step-level-intelligence-v2` | **Date**: 2026-04-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/025-step-level-intelligence-v2/spec.md`

## Summary

The repo already has packet `020` verifier and repair lineage, packet `021` supervision evidence,
and packet `023` truth-boundary ownership in the prep dossier. The bounded next slice is to freeze
one deterministic step-policy contract that scores the current step, chooses a bounded action
across keep, retry, clarify, downgrade, and escalate, carries budget-aware hints, and projects
that summary through current inspect surfaces without claiming grounded task proof from placeholder
execution.

This is a scaffold packet for later revision. It freezes the design surface and future
implementation map, but it does not change current router truth or claim that packet `025` is
implementation-ready while earlier packets still move.

## Technical Context

**Language/Version**: Rust 1.88.0 plus existing operator-console TypeScript if the packet-owned
summary reaches the current run-detail UI
**Primary Dependencies**: `mister-smith-core`, `mister-smith-app`, `mister-smith-events`,
existing packet `020` and packet `021` seams, `scripts/tests/test_live_runtime_proof_smoke.py`,
and current task or autonomy summary surfaces
**Storage**: existing runtime metadata and summary surfaces only; no new packet-owned persistence
store is required for the first scaffolded slice
**Testing**: targeted Rust tests for deterministic scoring and summary projection, smoke-harness
summary assertions, optional operator-console checks if UI files move, markdown lint, and diff
hygiene
**Target Platform**: local macOS development with Linux runtime parity for the shipped app binary
**Project Type**: Rust workspace packet scaffold with bounded summary-surface follow-on work
**Performance Goals**: choose a deterministic bounded action without regressing current fallback
behavior and expose that summary without raw log archaeology
**Constraints**: packet `023` owns run-trace taxonomy and proof-boundary schema; packet `020`
owns verifier and repair lineage; the first slice stays heuristic and deterministic; no new
endpoint; no grounded-proof overclaim from `workflow.execute_step`
**Scale/Scope**: one bounded step-policy packet on top of current step-evaluation, result-summary,
and smoke-harness seams

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, packet `025` and packet `023` prep dossiers, and the current runtime code seams. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and `analyze.md` are authored before any implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | Keeps packet `025` as a bounded scaffold layered on landed packet `020` and packet `021` seams without reopening deeper packets. |
| IV. Model-Agnostic Architecture | PASS | The packet defines provider-neutral step policy and uses OpenAI Responses docs only as event-taxonomy guidance, not as a provider lock-in. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The design prefers local corrective actions before broader escalation and preserves packet `020` repair lineage as the existing recovery owner. |
| VI. Evidence-Based Validation | PASS | The packet keeps current proof deterministic-only unless a later live rerun is actually produced, and it preserves placeholder-versus-grounded honesty. |
| VII. Explicit Dependency Management | PASS | The packet names the exact repo seams it consumes and keeps packet `023` and packet `020` ownership boundaries explicit. |
| VIII. Clean Closure And Resumability | PASS | The scaffold lands as a durable packet bundle in a separate worktree with explicit revision-later notes and no mutation of current router truth. |

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

### D1: Step policy layers on top of current step-evaluation seams

Packet `025` should build on `StepEvaluationRecord`, current routing summaries, supervision
evidence, and existing inspect surfaces instead of inventing a second runtime truth model.

### D2: Packet `023` remains the owner of proof-boundary language

Packet `025` can consume packet-023-owned proof or grounding references, but it should not create
its own competing trace taxonomy or proof-boundary schema.

### D3: The first slice is heuristic and deterministic

The first useful packet is not a training stack. It is one bounded deterministic scoring and
action ladder that operators and future implementers can audit.

### D4: Existing inspect surfaces remain canonical

Task inspect and autonomy status stay the canonical read surfaces. Any packet-owned operator
summary must remain a bounded projection of those same fields rather than a new endpoint or a new
trace owner.

## Minimal Implementation Slice

### Milestone 1: Freeze the shared step-policy contract

Validation:

- `spec.md`, `data-model.md`, and `contracts/step-policy-contract.md` agree on the action
  vocabulary, score summary, budget summary, and packet `023` ownership boundary
- packet `025` continues to describe itself as scaffold-only and revision-later

### Milestone 2: Add deterministic scoring and action selection

Validation:

- targeted `mister-smith-core` and `mister-smith-app` coverage can prove at least one `keep`
  decision and one non-`keep` decision from bounded deterministic inputs
- missing or inconclusive inputs preserve current fallback behavior

### Milestone 3: Project honest summaries onto existing inspect surfaces

Validation:

- task and autonomy summaries can show score, action, and budget-aware summary fields without raw
  log archaeology
- proof wording remains explicit that placeholder completion is not grounded task proof

## Parallel Staging Posture

Use only when the packet benefits from bounded parallel work.

- Blocking freeze before any parallel lanes: the shared step-policy contract and packet `023`
  ownership boundary
- Allowed disjoint lanes after the freeze:
  - core value-object lane: `crates/mister-smith-core/src/autonomy.rs`
  - runtime policy lane: `crates/mister-smith-app/src/execution.rs`
  - summary projection lane: `crates/mister-smith-app/src/autonomy.rs`,
    `crates/mister-smith-events/src/autonomy.rs`, and
    `crates/mister-smith-app/tests/autonomy_status_tests.rs`
  - optional operator-summary lane: `apps/operator-console/`
- Single-owner choke points:
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-app/src/autonomy.rs`
  - `crates/mister-smith-core/src/autonomy.rs`
  - any active proof or packet note under `docs/plans/`

## Explicitly Deferred

- grounded step execution beyond the current placeholder `workflow.execute_step` seam
- packet `023` run-trace taxonomy or proof-boundary schema changes
- PRM training, benchmark work, coordinator runtime, subagent runtime, or interoperability work

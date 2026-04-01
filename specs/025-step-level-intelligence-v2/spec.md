# Feature Specification: Step-Level Intelligence v2

**Feature Branch**: `025-step-level-intelligence-v2`
**Created**: 2026-04-01
**Status**: Draft scaffold
**Input**: `docs/direction.md`, `docs/current-state.md`, `docs/packet-prep/README.md`,
`docs/packet-prep/025-step-level-intelligence-v2.md`,
`docs/packet-prep/023-runtime-truth-and-run-trace.md`,
`docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`,
`docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`,
`docs/plans/2026-03-29-packet-021-live-evaluation.md`,
`docs/2026-03-28-session-context-report.md`, and the current step-policy seams in
`crates/mister-smith-core/src/autonomy.rs`, `crates/mister-smith-app/src/execution.rs`,
`crates/mister-smith-app/src/autonomy.rs`, `crates/mister-smith-events/src/autonomy.rs`,
`crates/mister-smith-app/tests/autonomy_status_tests.rs`, and
`scripts/tests/test_live_runtime_proof_smoke.py`

## Current Truth & Scope

Scaffold note:

- this packet is scaffolding only
- it is based on current repo truth as of 2026-04-01
- it will be revised before implementation after earlier packet work settles
- it must not be treated as the new forward authority in `docs/current-state.md`

Current repo truth already includes:

- packet `020` verifier-gated step decisions, explicit clarification, and repair lineage on the
  runtime-backed task path
- packet `021` supervision evidence on task inspect, autonomy status, and operator surfaces, but
  the newer packet-021 proof remains deterministic-only unless fresher live proof is produced
- typed step evaluation, clarification, routing, and orchestration-quality value objects on the
  current runtime path
- supported task and autonomy result surfaces that already project proof-related summaries

The remaining gap is narrower than a broad new program:

- there is no single deterministic step-policy contract for scoring step difficulty and choosing a
  bounded action across keep, retry, clarify, downgrade, and escalate
- budget pressure, routing posture, and supervision hints are present in the runtime, but they are
  not yet frozen into one bounded step-policy summary
- the current `workflow.execute_step` boundary can still make placeholder orchestration completion
  look semantically stronger than the repo can honestly prove

This packet therefore freezes one bounded scaffold slice:

1. define deterministic step scoring on top of the current step-evaluation seam
2. define one deterministic step-policy action ladder across keep, retry, clarify, downgrade, and
   escalate, including budget-aware policy hints
3. define operator-visible step-policy summaries that consume packet `023` proof language without
   taking over packet `023` run-trace or proof-boundary ownership

This is not:

- a packet-023 follow-on that redefines run-trace taxonomy or proof-boundary schema
- a claim that `workflow.execute_step` now proves grounded task execution
- PRM training, benchmark work, coordinator runtime, subagent runtime, or interoperability scope

## Clarifications

### Session 2026-04-01

- Q: Does packet `025` own run-trace taxonomy or proof-boundary schema? → A: No. Packet `023`
  remains the owner, and packet `025` only consumes those fields.
- Q: Where does budget data live for the first slice? → A: In packet-owned step-policy metadata
  and summaries, not in a new packet-023 trace schema.
- Q: What proof wording applies when `workflow.execute_step` reports completion? → A: Packet `025`
  inherits packet-023 proof wording and must treat that outcome as placeholder orchestration proof
  unless grounded task evidence exists.
- Q: Are new read endpoints in scope for this packet? → A: No. Existing task inspect and
  autonomy surfaces remain the canonical read surfaces.
- Q: What proof posture applies to packet-021 supervision evidence while writing this scaffold?
  → A: Deterministic-only unless a fresher live proof is actually found later.

## User Scenarios & Testing

Use independently testable stories. For Mister Smith packets, prefer a small number of bounded
stories over a long backlog of loosely related asks.

### User Story 1 - Score a step deterministically (Priority: P1)

An operator or future packet implementer can rely on one deterministic step score that classifies
the current step using the existing step-evaluation, routing, supervision, and budget context.

**Independent Test**: targeted deterministic fixtures or tests can prove that the same step
context always produces the same score and grounding summary, while steps with missing policy
signals preserve the current fallback behavior.

**Acceptance Scenarios**:

1. **Given** a step with accepted verifier output, stable routing, and no budget pressure,
   **When** the runtime builds a step-policy summary, **Then** it produces a deterministic
   low-risk score and a `keep` action.
2. **Given** a step with weak evidence, degraded supervision signals, or repeated instability,
   **When** the runtime builds a step-policy summary, **Then** it produces a deterministic
   non-keep score that can drive a bounded corrective action.

### User Story 2 - Choose a bounded step action under budget pressure (Priority: P1)

An operator or future runtime implementation can apply one deterministic policy ladder so budget
pressure and step difficulty influence whether the runtime keeps, retries, clarifies, downgrades,
or escalates a step.

**Independent Test**: targeted deterministic inputs can prove at least one retry or clarify path
and at least one downgrade or escalate path without inventing a new packet-023 trace schema or
regressing current happy-path fallback.

**Acceptance Scenarios**:

1. **Given** a recoverable step with high uncertainty but bounded budget pressure, **When** the
   step policy is evaluated, **Then** the chosen action stays inside the bounded ladder and prefers
   local correction before broader escalation.
2. **Given** a step with severe difficulty or budget pressure that exceeds the bounded local
   correction policy, **When** the step policy is evaluated, **Then** the chosen action is a
   deterministic downgrade or escalate decision with explicit budget-aware reasoning.

### User Story 3 - Inspect step policy without overstating proof (Priority: P2)

An operator can inspect current task, autonomy, or operator-facing summaries and see the latest
step score, chosen action, budget hint, and honest proof wording without mistaking placeholder
completion for grounded task proof.

**Independent Test**: task and autonomy summaries plus any operator-facing packet-owned summary can
show the same step-policy fields and explicit proof wording without requiring raw log archaeology.

**Acceptance Scenarios**:

1. **Given** a step-policy decision was produced, **When** an operator inspects the existing
   result surfaces, **Then** they can see the current score, chosen action, and budget-aware
   summary.
2. **Given** a step completed only through the current placeholder execution boundary, **When**
   that step policy is displayed, **Then** the surface states that the outcome is orchestration
   proof or placeholder completion rather than grounded task proof.

## Edge Cases

- step policy runs before graph or branch context exists and must still produce a deterministic
  outcome without claiming packet-023 ownership
- budget pressure suggests downgrade while step difficulty suggests clarify or retry
- packet `020` repair lineage and packet `021` supervision evidence both apply to the same step
- the OpenAI Responses streaming-events reference path changes again before implementation freeze,
  requiring a re-check of the current official reference page
- a step returns `completed` through `workflow.execute_step` but provides no grounded task
  evidence

## Requirements

### Functional Requirements

- **FR-001**: System MUST define one deterministic `StepDifficultyAssessment` on top of the
  current step-evaluation, routing, supervision, and budget context.
- **FR-002**: System MUST define one bounded step-policy action vocabulary across `keep`,
  `retry`, `clarify`, `downgrade`, and `escalate`.
- **FR-003**: System MUST preserve packet `020` ownership of verifier and repair lineage, with
  packet `025` only layering step policy on top of that existing seam.
- **FR-004**: System MUST preserve packet `023` ownership of run-trace taxonomy and proof-boundary
  schema, with packet `025` only consuming those fields and not redefining them.
- **FR-005**: System MUST carry budget pressure or budget hint information in step-policy metadata
  and summaries without inventing a new packet-023 trace schema.
- **FR-006**: System MUST surface packet-owned step-policy summaries through the existing task
  inspect, autonomy, and packet-owned operator-facing summary surfaces rather than a new endpoint.
- **FR-007**: System MUST state explicitly when a step result is placeholder orchestration proof
  rather than grounded task proof.
- **FR-008**: System MUST preserve current fallback behavior when policy inputs are absent,
  inconclusive, or outside the bounded first slice.
- **FR-009**: System MUST treat the OpenAI Responses event taxonomy as the canonical streaming
  event input for packet-025 terms, while re-confirming the current official reference page before
  final implementation freeze.
- **FR-010**: System MUST keep the first slice heuristic and deterministic; it MUST NOT require a
  judge-heavy or training-heavy policy to be useful.
- **FR-011**: System MUST NOT infer grounded task proof from placeholder step completion alone.
- **FR-012**: System MUST keep the future implementation scope bounded to current step-evaluation,
  result-summary, and smoke-harness seams rather than widening into coordinator runtime,
  subagent runtime, benchmark work, or interoperability work.

### Key Entities

- **StepDifficultyAssessment**: deterministic packet-owned summary of current step difficulty,
  confidence, and why the current step belongs in a bounded policy bucket
- **StepPolicyDecision**: packet-owned action summary that chooses one bounded action from keep,
  retry, clarify, downgrade, or escalate
- **StepBudgetPressureSummary**: packet-owned summary of budget pressure or budget hint that can
  influence the chosen action without taking over packet-023 trace ownership
- **StepGroundingStatusRef**: packet-023-owned proof-boundary reference that packet `025` consumes
  so the step-policy surface can distinguish placeholder completion from grounded proof

## Success Criteria

- **SC-001**: targeted deterministic validation can show at least one `keep` decision and at least
  one non-`keep` decision from the bounded step-policy ladder without regressing current fallback
  behavior
- **SC-002**: existing inspect surfaces can show step score, action, and budget-aware summary
  without requiring raw log archaeology
- **SC-003**: every packet-025 artifact repeats the same honest proof boundary and does not claim
  that `workflow.execute_step` is grounded task proof
- **SC-004**: packet `023` ownership of run-trace and proof-boundary schema remains unchanged
  across the spec, plan, contract, tasks, and analysis artifacts

## Assumptions

- packets `022` through `024` are still moving, so this scaffold will be revised before any
  implementation freeze
- packet `021` supervision evidence remains deterministic-only unless a fresher live proof is
  found during a later revision pass
- the current task inspect and autonomy surfaces remain the canonical read surfaces for any
  packet-owned summary in the first slice
- the current official OpenAI Responses streaming reference may move, so the final implementation
  freeze will re-confirm the exact current page before event names are treated as final

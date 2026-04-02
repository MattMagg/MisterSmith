# Feature Specification: Step-Level Intelligence v2

**Feature Branch**: `025-step-level-intelligence-v2`
**Created**: 2026-04-01
**Status**: Implementation-ready
**Input**: `docs/current-state.md`, `docs/direction.md`,
`docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`,
`docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`,
`docs/plans/2026-03-27-runtime-planning-simplification.md`,
`docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`,
`docs/plans/2026-04-01-packet-022-durable-workflow-core.md`,
`specs/020-verifier-gated-adaptive-orchestration/`,
`specs/022-durable-workflow-core/`,
`specs/023-runtime-truth-and-run-trace/`,
`specs/024-agent-boundary-security-hardening/`, and the current runtime seams in
`crates/mister-smith-core/src/autonomy.rs`,
`crates/mister-smith-app/src/execution.rs`,
`crates/mister-smith-app/src/autonomy.rs`,
`crates/mister-smith-events/src/autonomy.rs`,
`crates/mister-smith-app/tests/autonomy_status_tests.rs`,
`apps/operator-console/src/types.ts`,
`apps/operator-console/src/views/RunsView.tsx`, and
`scripts/tests/test_live_runtime_proof_smoke.py`

## Current Truth And Scope

Packet `025` is now the frozen implementation packet for stronger step-level intelligence on
current `main`.

It is ready for `/speckit.implement`. The open choices called out below are bounded coding choices,
not revision gates.

Current repo truth already includes the foundations this packet must build on rather than reopen:

- packet `019` already owns budget-aware routing profiles and bounded budget-pressure hints that
  packet `025` can consume without redefining routing ownership
- packet `020` already owns verifier-gated step decisions, clarification requests, repair
  directives, failure-context checkpoints, and `orchestration_quality`
- packet `021` already owns `supervision_evidence` on task, autonomy, and operator surfaces, with
  deterministic validation only unless a newer live rerun is produced
- packet `022` already owns durable workflow lifecycle, event-history, compaction, and
  effect-boundary semantics
- packet `023` already owns `runtime_truth`, `proof_boundary`, and bounded `run_trace` projection
  across task, session, autonomy, and operator surfaces
- packet `024` already owns least-privilege capability enforcement, quarantine reporting, and
  auth-callout fallback ceilings across ToolBus, MCP, and security seams
- the current runtime path already exposes `routing_history`, `step_routing_history`,
  `orchestration_quality`, `supervision_evidence`, `runtime_truth`, and budget-pressure hints on
  current result or status surfaces

The remaining gap is narrower than a broad new orchestration program:

- there is still no single deterministic step-policy contract that combines the current
  step-evaluation, routing, supervision, runtime-truth, and budget-hint inputs into one auditable
  score and action
- current surfaces can show adjacent packet-020, packet-021, and packet-023 signals, but they do
  not yet expose one packet-owned summary that explains why the runtime should keep, retry,
  clarify, downgrade, or escalate a step
- placeholder `workflow.execute_step` completion can still look stronger than it is unless the new
  step-policy surface repeats the packet-023 proof boundary honestly

Packet `025` therefore owns exactly three stories:

1. deterministic step difficulty scoring from current runtime inputs
2. one bounded action ladder across `keep`, `retry`, `clarify`, `downgrade`, and `escalate`
3. one packet-owned step-policy summary projected through existing inspect surfaces without
   redefining proof, lifecycle, or boundary ownership

Packet `025` does not own:

- packet `020` verifier verdict or repair-lineage semantics
- packet `022` lifecycle, event-history, compaction, or effect-boundary semantics
- packet `023` runtime-truth, proof-boundary, or run-trace ownership
- packet `024` capability-boundary, quarantine, or auth-callout policy
- a new endpoint, trace platform, benchmark harness, training stack, coordinator runtime,
  subagent runtime, or interoperability program
- a new live runtime-proof claim unless a real rerun is explicitly executed later

## Clarifications

### Session 2026-04-02

- Q: Is packet `025` implementation-ready now? → A: Yes. It is the active packet for stronger
  step-level policy on current `main`.
- Q: Does packet `025` own verifier verdicts, repair directives, or clarification semantics?
  → A: No. Packet `020` remains the owner, and packet `025` only consumes those signals.
- Q: Does packet `025` own lifecycle or durable event-history semantics? → A: No. Packet `022`
  remains the owner, and packet `025` only reads those fields when needed for policy context.
- Q: Does packet `025` own runtime truth, proof-boundary language, or run-trace schema? → A: No.
  Packet `023` remains the owner, and packet `025` only consumes those fields.
- Q: Does packet `025` change ToolBus, MCP, quarantine, or auth-callout policy? → A: No. Packet
  `024` remains the current boundary-hardening baseline.
- Q: Are new read endpoints or a new live rerun part of this packet? → A: No. Existing task,
  autonomy, and operator surfaces remain canonical, and proof stays deterministic-only unless a
  later live rerun is actually executed.

## User Scenarios And Testing

Use independently testable stories. Keep the packet bounded to one deterministic policy layer on
top of already-landed runtime signals.

### User Story 1 - Score a step deterministically (Priority: P1)

An operator or implementer can rely on one deterministic step score that classifies the current
step using the existing step-evaluation, routing, supervision, runtime-truth, and budget context.

**Independent Test**: targeted deterministic fixtures or tests can prove that the same step
context always produces the same score and explanation, while missing policy inputs preserve the
current fallback behavior.

**Acceptance Scenarios**:

1. **Given** a step with accepted verifier output, stable routing, clear runtime truth, and no
   budget pressure, **When** the runtime builds a step-policy summary, **Then** it produces a
   deterministic low-risk score and a `keep` action.
2. **Given** a step with weak evidence, degraded supervision signals, unstable routing, or
   repeated local repair pressure, **When** the runtime builds a step-policy summary, **Then** it
   produces a deterministic non-`keep` score with explicit reasons.

### User Story 2 - Choose a bounded action under evidence and budget pressure (Priority: P1)

An operator or implementer can apply one deterministic action ladder so current evidence and
budget pressure influence whether the runtime keeps, retries, clarifies, downgrades, or escalates
the step.

**Independent Test**: targeted deterministic inputs can prove at least one retry or clarify path
and at least one downgrade or escalate path without inventing a new packet-022 or packet-023
schema.

**Acceptance Scenarios**:

1. **Given** a recoverable step with high uncertainty but bounded budget pressure, **When** the
   step policy is evaluated, **Then** the chosen action stays inside the bounded ladder and prefers
   local correction before wider escalation.
2. **Given** a step with severe difficulty or budget pressure that exceeds the bounded local
   correction policy, **When** the step policy is evaluated, **Then** the chosen action is a
   deterministic `downgrade` or `escalate` decision with explicit budget-aware reasoning.

### User Story 3 - Inspect step policy without overstating proof or ownership (Priority: P2)

An operator can inspect current task, autonomy, or operator-facing summaries and see the latest
step score, chosen action, budget hint, and honest proof wording without mistaking placeholder
completion for grounded task proof or packet `025` for the owner of adjacent packet surfaces.

**Independent Test**: task and autonomy summaries plus the bounded operator projection can show the
same step-policy fields and explicit proof wording without requiring raw log archaeology or a new
endpoint.

**Acceptance Scenarios**:

1. **Given** a step-policy decision was produced, **When** an operator inspects the existing
   result surfaces, **Then** they can see the current score, chosen action, and budget-aware
   summary on those existing surfaces.
2. **Given** a step completed only through the current placeholder execution boundary, **When**
   that step policy is displayed, **Then** the surface states that the outcome is orchestration
   proof only and not grounded task proof.

## Edge Cases

- step policy runs before graph or branch context exists and must still produce a deterministic
  bounded outcome
- budget pressure suggests `downgrade` while step difficulty suggests `clarify` or `retry`
- packet `020` repair lineage and packet `021` supervision evidence both apply to the same step
- packet `022` lifecycle state says the workflow is `paused`, `cancelling`, or terminal while the
  latest packet-025 summary is still inspectable
- packet `023` runtime truth reports placeholder completion and no grounded evidence exists
- packet `024` boundary decisions exist for the run, but packet `025` must not turn those records
  into new execution authority

## Requirements

### Functional Requirements

- **FR-001**: System MUST define one deterministic `StepDifficultyAssessment` from the current
  `StepEvaluationRecord`, routing history, step-routing history, supervision evidence,
  runtime-truth, and budget-hint inputs already present on current `main`.
- **FR-002**: System MUST define one bounded step-policy action vocabulary across `keep`, `retry`,
  `clarify`, `downgrade`, and `escalate`.
- **FR-003**: System MUST preserve packet `020` ownership of verifier verdicts, repair
  directives, clarification requests, failure-context checkpoints, and `orchestration_quality`,
  with packet `025` only layering step policy on top of those existing seams.
- **FR-004**: System MUST preserve packet `022` ownership of durable lifecycle, event-history,
  compaction, and effect-boundary semantics, with packet `025` only consuming those fields as
  context where needed.
- **FR-005**: System MUST preserve packet `023` ownership of `runtime_truth`, `proof_boundary`,
  and `run_trace`, with packet `025` only consuming those fields and not redefining them.
- **FR-006**: System MUST preserve packet `024` ownership of capability-boundary, quarantine, and
  auth-callout policy, and MUST NOT create new execution authority or security policy as part of
  step scoring.
- **FR-007**: System MUST carry budget pressure or budget hints in packet-owned step-policy
  metadata and summaries without inventing a new trace schema or budget control loop.
- **FR-008**: System MUST surface packet-owned step-policy summaries through the existing task
  inspect, autonomy, and bounded operator-facing summary surfaces rather than a new endpoint.
- **FR-009**: System MUST state explicitly when a step result is placeholder orchestration proof
  rather than grounded task proof, using packet-023-owned proof wording.
- **FR-010**: System MUST preserve current fallback behavior when policy inputs are absent,
  inconclusive, or outside the bounded first slice.
- **FR-011**: System MUST keep the first slice heuristic and deterministic; it MUST NOT require a
  judge-heavy or training-heavy policy to be useful.
- **FR-012**: System MUST NOT infer grounded task proof from placeholder step completion alone.
- **FR-013**: System MUST keep the future implementation scope bounded to current
  `mister-smith-core`, `mister-smith-app`, `mister-smith-events`, smoke-harness, and
  operator-surface seams rather than widening into coordinator runtime, subagent runtime,
  benchmark work, or interoperability work.
- **FR-014**: System MUST tie major packet claims to the exact repo anchors named in this packet,
  its contract, and its task plan.

### Key Entities

- **StepDifficultyAssessment**: deterministic packet-owned summary of current step difficulty,
  confidence, and why the step belongs in a bounded policy bucket
- **StepBudgetPressureSummary**: packet-owned summary of budget pressure or policy hint that can
  influence the chosen action without taking over packet-022 or packet-023 ownership
- **StepPolicyDecision**: packet-owned action summary that chooses one bounded action from `keep`,
  `retry`, `clarify`, `downgrade`, or `escalate`
- **StepPolicySummaryView**: packet-owned inspect projection that packages the latest difficulty
  assessment, budget summary, decision, and display note for existing task, autonomy, and
  operator surfaces
- **StepPolicyInputRefs**: packet-owned references to packet-020 repair lineage, packet-021
  supervision evidence, packet-022 lifecycle context, packet-023 proof boundary, and any relevant
  packet-024 boundary evidence already emitted by the runtime

## Success Criteria

- **SC-001**: targeted deterministic validation can show at least one `keep` decision and at
  least one non-`keep` decision from the bounded step-policy ladder without regressing current
  fallback behavior
- **SC-002**: existing task, autonomy, and bounded operator surfaces can show step score, action,
  budget-aware summary, and honest proof wording without requiring raw log archaeology
- **SC-003**: every packet-025 artifact repeats the same honest proof boundary and does not claim
  that `workflow.execute_step` is grounded task proof
- **SC-004**: packet `020`, packet `022`, packet `023`, and packet `024` ownership boundaries
  remain unchanged across the spec, plan, contract, tasks, and analysis artifacts
- **SC-005**: packet `025` remains bounded to deterministic step policy and summary projection,
  with no new endpoint, security policy, benchmark, training, coordinator-runtime, or
  interoperability work pulled in

## Assumptions

- packet `020`, packet `021`, packet `022`, packet `023`, and packet `024` are already landed on
  current `main` and remain the adjacent authorities packet `025` must consume
- packet `021` supervision evidence remains deterministically validated unless a newer live rerun
  is explicitly executed later
- existing task inspect, autonomy status, and operator run-detail surfaces remain the canonical
  read surfaces for the first slice
- the first implementation slice can use the existing runtime metadata and result-projection
  surfaces without introducing a new durable packet-owned store

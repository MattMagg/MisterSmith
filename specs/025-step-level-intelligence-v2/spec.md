# Feature Specification: Step-Level Intelligence v2

**Feature Branch**: `025-step-level-intelligence-v2`
**Created**: 2026-04-01
**Status**: Implementation-ready
**Input**: `docs/current-state.md`, `docs/direction.md`,
`docs/2026-03-28-session-context-report.md`,
`docs/research-prompts/09-step-level-intelligence.md`,
`docs/research-output/research/targeted-step-level-intelligence-R6.md`,
`docs/plans/2026-03-18-ms-72-step-routing-visibility-evaluation.md`,
`docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md`,
`docs/plans/2026-04-01-packet-022-durable-workflow-core.md`,
`specs/023-runtime-truth-and-run-trace/`, `specs/024-agent-boundary-security-hardening/`, and
the current step-policy seams in `crates/mister-smith-core/src/autonomy.rs`,
`crates/mister-smith-events/src/autonomy.rs`, `crates/mister-smith-app/src/execution.rs`,
`crates/mister-smith-app/src/autonomy.rs`,
`crates/mister-smith-app/tests/autonomy_status_tests.rs`, and
`scripts/tests/test_live_runtime_proof_smoke.py`

## Current Truth & Scope

Current repo truth already includes the layers packet `025` must build on rather than reopen:

- packet `020` landed verifier-gated step decisions, repair lineage, and clarification handling on
  the runtime-backed task path
- packet `021` landed bounded supervision evidence on task, autonomy, and operator-facing surfaces
- packet `022` landed durable workflow lifecycle, history, compaction, and effect-boundary
  ownership on current `main`
- packet `023` landed one shared `runtime_truth` contract, one bounded run-trace summary, and one
  explicit proof-boundary view across task, session, autonomy, and operator surfaces
- packet `024` landed action-bound external capability enforcement, clearer quarantine reasons,
  and auth-callout fallback clamping on current `main`
- current runtime seams already expose the raw inputs packet `025` needs:
  `StepEvaluationRecord`, `StepRoutingDecisionSummary`, `ContextPressureSummary`,
  `TeamSizingDecision`, `RuntimeTruthView`, `SupervisionEvidenceView`, `TaskResultView`,
  `SessionRetainedResultView`, `OperatorResultPreview`, and `AutonomyStatusView`

The remaining gap is narrower than the original scaffold implied:

- there is no single packet-owned step-policy contract that consumes current step evaluation,
  routing, budget or context pressure, supervision, and runtime-truth inputs into one deterministic
  difficulty assessment and one bounded next action
- there is no shared packet-owned summary view that projects that same step-policy story through
  current task, session, autonomy, and operator surfaces
- `workflow.execute_step` still represents placeholder orchestration completion unless grounded
  task evidence exists, so packet `025` must preserve packet-023 proof wording rather than invent
  a stronger proof claim

Packet `025` therefore owns one bounded implementation slice:

1. define deterministic `StepDifficultyAssessment` on top of current runtime signals
2. define one bounded `StepPolicyDecision` ladder across `keep`, `retry`, `clarify`, `downgrade`,
   and `escalate`
3. define one packet-owned `StepPolicySummaryView` that projects through current task, session,
   autonomy, and operator surfaces while consuming packet-023 runtime-truth fields

Packet `025` does not own:

- packet `022` durable workflow semantics, lifecycle meaning, history, compaction, or
  effect-boundary rules
- packet `023` runtime-truth wording, run-trace taxonomy, or proof-boundary schema
- packet `024` capability boundary, quarantine, sandbox, auth-callout, or least-privilege policy
- PRM training, judge-heavy scoring, benchmark programs, coordinator runtime, subagent runtime, or
  interoperability work

## Clarifications

### Session 2026-04-02

- Q: Does packet `025` own runtime-truth wording or proof-boundary schema? → A: No. Packet `023`
  remains the owner, and packet `025` only consumes those fields.
- Q: Does packet `025` change durable workflow lifecycle or history behavior? → A: No. Packet
  `022` remains the owner, and packet `025` only reads those states when they affect policy
  presentation.
- Q: Does the first packet-025 slice need raw streaming-event parsing or a new Responses event
  parser? → A: No. The first slice should consume landed internal step, routing, supervision,
  budget, and runtime-truth surfaces.
- Q: Which result surfaces are canonical for packet `025`? → A: `task.result` and autonomy status
  remain the full canonical surfaces, while session-retained and operator-preview surfaces remain
  compact projections of the same packet-owned summary.
- Q: What proof wording applies when `workflow.execute_step` reports completion? → A: Packet-023
  placeholder wording stays canonical, and packet `025` must not strengthen it into grounded task
  proof.

## User Scenarios & Testing

Use independently testable stories. For Mister Smith packets, prefer a small number of bounded
stories over a long backlog of loosely related asks.

### User Story 1 - Score a step deterministically from landed runtime signals (Priority: P1)

An operator or future packet implementer can rely on one deterministic step score that classifies
the current step using the landed step-evaluation, routing, supervision, budget-pressure, and
runtime-truth inputs already available on current `main`.

**Independent Test**: targeted deterministic fixtures prove the same input signals always produce
the same difficulty bucket and explanation, while missing inputs preserve current fallback
behavior.

**Acceptance Scenarios**:

1. **Given** a step with accepted verifier output, stable routing, normal budget posture, and no
   proof-boundary concern, **When** the runtime builds a step-policy summary, **Then** it produces
   a deterministic low-risk assessment and a `keep` action.
2. **Given** a step with weak evidence, repeated instability, or degraded supervision context,
   **When** the runtime builds a step-policy summary, **Then** it produces a deterministic
   non-`keep` assessment with explicit reasons.

### User Story 2 - Choose one bounded step action under budget and repair pressure (Priority: P1)

An operator or future runtime implementation can apply one deterministic policy ladder so current
repair lineage, routing posture, and budget pressure influence whether the runtime keeps, retries,
clarifies, downgrades, or escalates a step.

**Independent Test**: targeted deterministic inputs prove at least one `retry` or `clarify` path
and at least one `downgrade` or `escalate` path without inventing a new runtime-truth, durable
workflow, or security contract.

**Acceptance Scenarios**:

1. **Given** a recoverable step with high uncertainty but bounded local repair options, **When**
   the policy is evaluated, **Then** the chosen action stays inside the bounded ladder and prefers
   local correction before broader escalation.
2. **Given** a step with severe difficulty, exhausted repair posture, or strong budget pressure,
   **When** the policy is evaluated, **Then** the chosen action becomes a deterministic
   `downgrade` or `escalate` decision with explicit reasoning.

### User Story 3 - Inspect honest step-policy summaries on current surfaces (Priority: P2)

An operator can inspect task, session, autonomy, and operator-facing summaries and see the latest
step score, chosen action, budget hint, and honest proof wording without mistaking placeholder
completion for grounded task proof.

**Independent Test**: the current task, session, autonomy, and operator-preview surfaces all show
the same packet-owned step-policy fields and explicit proof wording without requiring raw log
archaeology.

**Acceptance Scenarios**:

1. **Given** a step-policy decision was produced, **When** an operator inspects the current result
   surfaces, **Then** they can see the current difficulty, chosen action, and budget-aware
   summary.
2. **Given** a step completed only through the current placeholder execution boundary, **When**
   that step policy is displayed, **Then** the surface states that the result is orchestration
   proof rather than grounded task proof.

## Edge Cases

- step policy runs before graph or branch context exists and still must produce a deterministic
  bounded result
- repair lineage suggests `retry` while budget pressure suggests `downgrade`
- supervision evidence raises concern while verifier output is still accepted
- task metadata carries runtime truth but no step-routing history yet
- the latest durable lifecycle state is `paused`, `cancelled`, or `terminated`, so packet `025`
  must not imply further forward progress
- a step returns `completed` through `workflow.execute_step` but provides no grounded task
  evidence

## Requirements

### Functional Requirements

- **FR-001**: System MUST define one deterministic `StepDifficultyAssessment` on top of current
  `StepEvaluationRecord`, the latest available `StepRoutingDecisionSummary`, available budget or
  context-pressure signals, supervision evidence, and packet-023 runtime-truth inputs.
- **FR-002**: System MUST define one bounded step-policy action vocabulary across `keep`,
  `retry`, `clarify`, `downgrade`, and `escalate`.
- **FR-003**: System MUST preserve packet `020` ownership of verifier verdicts, clarification, and
  repair lineage, with packet `025` only layering step policy on top of those existing fields.
- **FR-004**: System MUST preserve packet `022` ownership of durable workflow lifecycle, history,
  compaction, and effect-boundary semantics, with packet `025` only consuming those states when
  they affect step-policy presentation.
- **FR-005**: System MUST preserve packet `023` ownership of runtime-truth wording, proof-boundary
  schema, and run-trace taxonomy, with packet `025` only consuming those fields and not redefining
  them.
- **FR-006**: System MUST preserve packet `024` ownership of capability boundary, quarantine,
  sandbox, and auth-callout policy, and MUST NOT let step-policy output act as boundary authority.
- **FR-007**: System MUST carry budget or context-pressure hints in packet-owned step-policy
  metadata without inventing a new runtime-truth, durable-workflow, or security schema.
- **FR-008**: System MUST project packet-owned `StepPolicySummaryView` through existing task,
  session, autonomy, and operator-facing result surfaces rather than a new endpoint.
- **FR-009**: System MUST state explicitly when a step result remains placeholder orchestration
  proof rather than grounded task proof.
- **FR-010**: System MUST preserve current fallback behavior when policy inputs are absent,
  inconclusive, or outside the bounded first slice.
- **FR-011**: System MUST keep the first packet-025 slice heuristic and deterministic; it MUST NOT
  require judge-heavy or training-heavy scoring to be useful.
- **FR-012**: System MUST keep implementation scope bounded to current step-evaluation,
  step-routing, budget-pressure, result-summary, and smoke-harness seams rather than widening into
  coordinator runtime, subagent runtime, benchmark work, or interoperability work.
- **FR-013**: System MUST respect durable lifecycle state when rendering step-policy output and
  MUST NOT imply forward step action once the workflow is paused, cancelled, or terminated.

### Key Entities

- **StepDifficultyAssessment**: packet-owned deterministic summary of current step difficulty,
  confidence, and why the step belongs in a bounded policy bucket
- **StepBudgetPressureSummary**: packet-owned summary of budget or context pressure that can shape
  action choice without taking over packet-022, packet-023, or packet-024 ownership
- **StepPolicyDecision**: packet-owned action summary that chooses one bounded action from
  `keep`, `retry`, `clarify`, `downgrade`, or `escalate`
- **StepPolicySummaryView**: packet-owned view that projects the latest assessment, pressure, and
  decision through current task, session, autonomy, and operator-facing surfaces

## Success Criteria

- **SC-001**: Targeted deterministic validation shows at least one `keep` decision and at least
  one non-`keep` decision from the bounded step-policy ladder without regressing current fallback
  behavior.
- **SC-002**: Task, session, autonomy, and operator-facing result surfaces expose the same latest
  step-policy summary for the same workflow in deterministic validation.
- **SC-003**: Every packet-025 artifact repeats the same honest proof boundary and does not claim
  that `workflow.execute_step` is grounded task proof.
- **SC-004**: Packet `022`, packet `023`, and packet `024` ownership boundaries remain unchanged
  across the spec, plan, contract, tasks, and analysis artifacts.
- **SC-005**: Packet `025` remains implementable inside the existing step-evaluation, summary, and
  smoke-harness seams without requiring a new endpoint or external research dependency.

## Assumptions

- packets `022`, `023`, and `024` are landed and should be treated as current implementation
  authority while packet `025` is prepared for implementation
- the current task and autonomy surfaces remain the full canonical read surfaces for packet-owned
  step-policy output, with session and operator surfaces acting as compact projections
- the first packet-025 slice derives from landed internal runtime signals rather than a new raw
  streaming-event parser
- any later stream-native, PRM-backed, or training-heavy step intelligence remains follow-on work
  after this bounded deterministic slice

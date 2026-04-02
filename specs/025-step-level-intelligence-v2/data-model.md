# Data Model: Step-Level Intelligence v2

## Step policy entities

### `StepDifficultyAssessment`

- `step_id`: stable step identifier from the current runtime seam
- `difficulty_bucket`: bounded deterministic category such as `low`, `moderate`, `high`, or
  `critical`
- `confidence_label`: bounded confidence summary for why the assessment is stable enough to act on
- `reason_codes`: ordered short reasons explaining why the current difficulty bucket was chosen
- `step_evaluation_ref`: reference to the driving `StepEvaluationRecord`
- `routing_ref`: optional reference to the latest `StepRoutingDecisionSummary`
- `supervision_ref`: optional reference to the latest `SupervisionEvidenceView`
- `runtime_truth_ref`: packet-023-owned reference used to preserve placeholder-versus-grounded
  honesty

### `StepBudgetPressureSummary`

- `pressure_source`: bounded source label such as `context_pressure`, `team_sizing`, or `none`
- `pressure_level`: bounded summary such as `none`, `watch`, `softcap`, or `hard_stop`
- `budget_ref`: optional reference to `ContextPressureSummary` when available
- `team_sizing_ref`: optional reference to `TeamSizingDecision` when available
- `policy_hint`: deterministic hint that can shape action choice without becoming a new packet-022,
  packet-023, or packet-024 contract
- `display_note`: short human-readable note for result surfaces

### `StepPolicyDecision`

- `step_id`: stable step identifier
- `chosen_action`: one value from `keep`, `retry`, `clarify`, `downgrade`, or `escalate`
- `action_reason`: concise explanation for why that action was chosen
- `difficulty_ref`: reference to the driving `StepDifficultyAssessment`
- `budget_reason`: optional explanation of how budget pressure affected the chosen action
- `repair_lineage_ref`: optional packet-020-owned reference when the chosen action intersects with
  existing repair behavior
- `lifecycle_gate_ref`: optional packet-022-owned reference when durable lifecycle state narrows
  what action can be honestly shown

## Result-surface projection

### `StepPolicySummaryView`

- `step_id`: latest step covered by the summary
- `difficulty_assessment`: latest bounded difficulty assessment
- `budget_pressure`: latest bounded pressure summary
- `policy_decision`: latest bounded action summary
- `runtime_truth_ref`: packet-023-owned proof or grounding reference
- `display_note`: short text explaining whether the current result is placeholder orchestration
  proof or grounded task proof

Projected locations:

- `TaskResultView.step_policy`
- `SessionRetainedResultView.step_policy`
- `OperatorResultPreview.step_policy`
- `AutonomyStatusView.step_policy`

## Invariants

- packet `020` owns verifier, clarification, and repair lineage; packet `025` only references
  those inputs
- packet `022` owns durable workflow lifecycle, history, compaction, and effect-boundary meaning;
  packet `025` only reads lifecycle state when it affects presentation
- packet `023` owns runtime-truth wording, proof-boundary schema, and run-trace taxonomy; packet
  `025` only stores references to those fields
- packet `024` owns boundary security, capability enforcement, quarantine, sandbox, and
  auth-callout posture; packet `025` does not replace or widen those decisions
- action selection stays inside the bounded vocabulary of `keep`, `retry`, `clarify`, `downgrade`,
  and `escalate`
- placeholder completion can inform a step-policy summary but cannot become grounded task proof by
  itself
- missing or inconclusive policy inputs fall back to current runtime behavior rather than forcing a
  synthetic packet-owned decision

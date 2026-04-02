# Data Model: Step-Level Intelligence v2

## Packet-owned entities

### `StepDifficultyAssessment`

- `workflow_id`: stable workflow identifier from the current runtime seam
- `step_id`: stable step identifier from the current step-evaluation or step-routing seam
- `difficulty_bucket`: bounded deterministic category such as `low`, `moderate`, `high`, or
  `critical`
- `confidence_label`: bounded summary for how stable the assessment is
- `reason_codes`: ordered short reasons explaining why the current difficulty bucket was chosen
- `verifier_ref`: bounded reference to the latest packet-020 `StepEvaluationRecord`
- `routing_ref`: bounded reference to the latest step-routing entry that influenced the assessment
- `supervision_ref`: optional bounded reference to packet-021 `supervision_evidence`
- `grounding_status_ref`: bounded reference to the packet-023 `runtime_truth` evidence class and
  proof-boundary posture

### `StepBudgetPressureSummary`

- `workflow_id`: stable workflow identifier
- `step_id`: stable step identifier when the pressure summary is step-specific
- `pressure_level`: bounded summary such as `none`, `watch`, `softcap`, or `hard_stop`
- `pressure_source`: short label for which current seam provided the pressure signal
- `policy_hint`: deterministic hint that can shape action choice without becoming a new proof
  contract
- `budget_root`: current budget root when available from the runtime path
- `note`: short human-readable explanation for inspect surfaces

### `StepPolicyDecision`

- `workflow_id`: stable workflow identifier
- `step_id`: stable step identifier
- `chosen_action`: one bounded value from `keep`, `retry`, `clarify`, `downgrade`, or `escalate`
- `action_reason`: concise explanation for why that action was chosen
- `difficulty_ref`: reference to the driving `StepDifficultyAssessment`
- `budget_ref`: optional reference to the driving `StepBudgetPressureSummary`
- `repair_lineage_ref`: optional link to packet-020-owned repair lineage when the chosen action
  intersects with current repair behavior
- `requires_operator_attention`: bounded flag for operator-visible escalation cases

### `StepPolicyInputRefs`

- `latest_step_evaluation`: latest packet-020 step-evaluation reference used as input
- `latest_step_routing`: latest step-routing reference used as input
- `supervision_evidence`: latest packet-021 supervision-evidence reference when used
- `runtime_truth`: latest packet-023 runtime-truth reference when used

### `StepPolicySummaryView`

- `difficulty_assessment`: latest packet-owned difficulty summary
- `budget_pressure`: latest packet-owned budget summary when available
- `policy_decision`: latest packet-owned chosen action
- `input_refs`: bounded references to the adjacent packet-owned seams that shaped the decision
- `display_note`: short explanation for human readers
- `proof_boundary_ref`: packet-023-owned proof-boundary reference carried through unchanged

## Surface placement

### Task inspect

- `task.result.step_policy`: canonical task-facing packet-025 summary

### Autonomy status

- `AutonomyStatusView.step_policy`: operator-facing status projection of the same summary

### Operator selected-run detail

- existing task inspect payload projected in the operator console
- no new endpoint or separate storage surface

## Invariants

- packet `020` owns verifier verdicts, clarification requests, repair directives, and repair
  lineage; packet `025` only stores bounded references to those facts
- packet `021` owns predictive-supervision evidence; packet `025` may reference it but does not
  replace it
- packet `023` owns runtime truth, proof-boundary wording, and run-trace schema; packet `025`
  stores references and adjacent summary only
- packet `025` action selection stays inside the bounded vocabulary of `keep`, `retry`,
  `clarify`, `downgrade`, and `escalate`
- placeholder completion can inform packet-owned step policy but cannot become grounded task proof
  by itself
- missing or inconclusive packet-025 inputs fall back to current runtime behavior instead of
  forcing a synthetic stronger decision
- the first slice adds summary fields to existing result surfaces only; it does not create a new
  endpoint or packet-owned persistence subsystem

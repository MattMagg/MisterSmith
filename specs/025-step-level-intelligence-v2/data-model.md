# Data Model: Step-Level Intelligence v2

## Step policy entities

### `StepDifficultyAssessment`

- `step_id`: stable step identifier from the current runtime seam
- `difficulty_bucket`: bounded deterministic category such as `low`, `moderate`, `high`, or
  `critical`
- `confidence_label`: bounded confidence summary for why the score is stable enough to act on
- `reason_codes`: ordered short reasons explaining why the current difficulty bucket was chosen
- `supervision_ref`: optional reference to the latest packet-021 supervision evidence used as an
  input signal
- `budget_pressure_ref`: optional reference to the current budget-pressure summary
- `grounding_status_ref`: packet-023-owned reference used to preserve placeholder-vs-grounded
  honesty

### `StepBudgetPressureSummary`

- `budget_root`: current budget ownership root if the runtime already exposes it
- `pressure_level`: bounded summary such as `none`, `watch`, `softcap`, or `hard-stop`
- `policy_hint`: deterministic hint that can shape action choice without becoming a new proof
  schema
- `remaining_headroom_note`: short human-readable note for existing inspect surfaces

### `StepPolicyDecision`

- `step_id`: stable step identifier
- `chosen_action`: one value from `keep`, `retry`, `clarify`, `downgrade`, or `escalate`
- `action_reason`: concise explanation for why that action was chosen
- `difficulty_ref`: reference to the driving `StepDifficultyAssessment`
- `budget_reason`: optional explanation of how budget pressure affected the chosen action
- `repair_lineage_ref`: optional link to packet-020-owned repair lineage when the chosen action
  intersects with existing repair behavior

## Operator evidence projection

### `StepPolicySummaryView`

- `difficulty_assessment`: latest bounded score summary
- `budget_pressure`: latest budget hint summary
- `policy_decision`: latest bounded action summary
- `proof_boundary_ref`: packet-023-owned proof or grounding reference
- `display_note`: short text that explains whether the current result is placeholder orchestration
  proof or grounded task proof

## Invariants

- packet `023` owns run-trace taxonomy and proof-boundary schema; packet `025` only stores a
  reference to those fields
- packet `020` remains the owner of verifier and repair lineage
- packet `025` action selection stays inside the bounded vocabulary of keep, retry, clarify,
  downgrade, and escalate
- placeholder completion can inform a step-policy summary but cannot become grounded task proof by
  itself
- missing or inconclusive policy inputs fall back to current runtime behavior rather than forcing a
  synthetic policy decision

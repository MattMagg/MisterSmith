# Data Model: Verifier-Gated Adaptive Orchestration

## Workflow decision entities

### `StepEvaluationRecord`

- `workflow_id`: owning workflow identifier
- `step_id`: stable identifier for the evaluated workflow step or handoff
- `verdict`: `accepted`, `retry_step`, `clarify_handoff`, `replan_from_checkpoint`, or `stop`
- `confidence`: bounded verifier confidence or certainty indicator
- `reason`: human-readable explanation for the verdict
- `failure_code`: optional structured failure or deficiency label
- `checkpoint_ref`: optional reference to the last stable checkpoint used for repair

### `HandoffClarificationRequest`

- `source_step_id`: the step that produced the incomplete handoff
- `target_step_id`: the blocked downstream step
- `missing_constraints`: explicit requirements or assumptions that must be clarified
- `attempt_count`: bounded clarification counter
- `expires_at`: guardrail for abandoning stale clarification loops

### `RepairDirective`

- `action`: `retry_step`, `clarify_handoff`, `replan_from_checkpoint`, or `stop`
- `issued_by`: verifier or supervisory surface that owns the repair decision
- `failure_context_ref`: reference to the rejection diagnostics
- `retry_budget_remaining`: bounded local retry budget

## Preserved workflow state

### `FailureContextCheckpoint`

- last accepted stable step
- preserved inputs and constraints needed to resume safely
- rejection diagnostics attached to the failed step
- lineage needed to explain why the repair branch exists

### `OrchestrationQualityView`

- latest verifier verdict
- latest repair directive
- clarification count
- last stable checkpoint
- final workflow outcome relative to the repair loop

## Invariants

- the verifier owns verdicts, but it does not silently mutate accepted step output
- local repair must be bounded; clarification and retry loops cannot run forever
- rejection of one step should prefer local repair from a stable checkpoint over full-task restart
- task and autonomy views must remain consistent with the actual verifier and repair history
- omitting or disabling the verifier-gated loop must preserve today's shipped happy path

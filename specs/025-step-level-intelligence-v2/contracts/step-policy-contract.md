# Contract: Step Policy Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one shared step-policy contract that scores the current step, summarizes budget pressure,
chooses one bounded next action, and projects that summary through existing inspect surfaces
without competing with packet `020`, `021`, or `023` ownership.

Packet `025` does **not** create a new proof-boundary schema. Packet `023` remains the owner of
`runtime_truth`, proof-boundary wording, and run-trace taxonomy.

## Canonical Mapping

The contract for this packet is:

- `StepDifficultyAssessment` scores the current step using deterministic current-state inputs
- `StepBudgetPressureSummary` carries bounded pressure hints that can shape action choice
- `StepPolicyDecision` chooses one bounded action from `keep`, `retry`, `clarify`, `downgrade`,
  and `escalate`
- `StepPolicySummaryView` projects those packet-owned summaries onto existing inspect surfaces
- `proof_boundary_ref` and grounding posture remain packet-023-owned references that packet `025`
  consumes but does not redefine

No packet-025 surface may become a competing runtime-truth or proof-boundary contract.

## Canonical Surface Placement

### Task inspect

- `task.result.step_policy` is the canonical task-facing summary

### Autonomy status

- `AutonomyStatusView.step_policy` is the operator-facing status projection

### Operator selected-run detail

- the operator console reads `task.result.step_policy` from the existing inspect payload
- no new endpoint is introduced

### Explicitly deferred in the first slice

- session projection
- new trace explorer or step-policy endpoint

## Canonical Payload Shape

Example authoritative payload shape:

```json
{
  "step_policy": {
    "difficulty_assessment": {
      "workflow_id": "workflow-1",
      "step_id": "planner.step.2",
      "difficulty_bucket": "high",
      "confidence_label": "deterministic",
      "reason_codes": [
        "weak_current_evidence",
        "unstable_recent_step_history"
      ]
    },
    "budget_pressure": {
      "workflow_id": "workflow-1",
      "step_id": "planner.step.2",
      "pressure_level": "softcap",
      "pressure_source": "runtime.task_path",
      "policy_hint": "prefer_local_correction_before_escalation",
      "budget_root": "runtime.task_path"
    },
    "policy_decision": {
      "workflow_id": "workflow-1",
      "step_id": "planner.step.2",
      "chosen_action": "downgrade",
      "action_reason": "high_difficulty_plus_softcap_budget_pressure",
      "repair_lineage_ref": "packet-020:last-stable-checkpoint"
    },
    "input_refs": {
      "latest_step_evaluation": "packet-020:planner.step.2",
      "latest_step_routing": "step-routing:planner.step.2",
      "supervision_evidence": "packet-021:planner.step.2",
      "runtime_truth": "packet-023:placeholder_or_simulated_step_completion"
    },
    "proof_boundary_ref": {
      "owner_packet": "023",
      "task_proof": "result is orchestration proof, not substantive task proof"
    },
    "display_note": "placeholder orchestration proof only; local correction preferred before escalation"
  }
}
```

Behavior:

- the only bounded action values are `keep`, `retry`, `clarify`, `downgrade`, and `escalate`
- packet `020` repair lineage may be referenced, but packet `025` does not replace that contract
- packet `021` supervision evidence may be referenced, but packet `025` does not replace that
  contract
- packet `023` proof-boundary wording and runtime-truth fields stay authoritative
- packet `025` can explain current policy posture, but it cannot turn policy confidence into
  grounded task proof

## Fallback Contract

Expected first-slice fallback behavior:

- if packet-025 inputs are missing or inconclusive, the runtime preserves current behavior
- the packet may omit `step_policy` or emit a bounded low-confidence summary, but it must not
  fabricate a stronger decision than current repo truth supports
- existing `runtime_truth`, `supervision_evidence`, and `orchestration_quality` surfaces remain
  valid even before packet-025 fields are present everywhere

## Relationship To Existing Surfaces

The following existing surfaces remain authoritative baseline inputs:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `scripts/tests/test_live_runtime_proof_smoke.py`
- `apps/operator-console/src/types.ts`
- `apps/operator-console/src/views/RunsView.tsx`

This packet only extends them with:

- one explicit deterministic step-difficulty assessment
- one bounded budget-pressure summary
- one bounded action-decision vocabulary
- one coherent projection through current inspect surfaces

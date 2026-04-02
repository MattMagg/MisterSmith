# Contract: Step Policy Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one shared step-policy contract that scores the current step, chooses one bounded action,
and projects that summary through current result surfaces without competing with packet `020`,
packet `022`, packet `023`, or packet `024` ownership.

This packet does **not** create a new durable-workflow, runtime-truth, or security schema.

## Canonical Inputs

The first packet-025 slice consumes the landed internal runtime seams:

- `StepEvaluationRecord`
- latest available `StepRoutingDecisionSummary`
- `ContextPressureSummary` when available
- `TeamSizingDecision` when available
- `SupervisionEvidenceView` when available
- `RuntimeTruthView`
- durable lifecycle state when available

The first slice does **not** require a new raw streaming-event parser.

## Canonical Mapping

The contract for this packet is:

- `StepDifficultyAssessment` scores the current step using deterministic current-state inputs
- `StepBudgetPressureSummary` carries bounded pressure hints that can influence action choice
- `StepPolicyDecision` chooses one bounded action from `keep`, `retry`, `clarify`, `downgrade`,
  and `escalate`
- `StepPolicySummaryView` projects those packet-owned summaries onto current result surfaces
- packet-020 repair lineage, packet-022 lifecycle state, packet-023 runtime truth, and packet-024
  boundary policy remain upstream inputs or scope boundaries, not packet-025-owned outputs

No packet-025 surface may become a competing proof-boundary, lifecycle, or boundary-security
contract.

## Canonical Evidence Shape

Example authoritative payload shape:

```json
{
  "step_id": "planner.step.2",
  "difficulty_assessment": {
    "difficulty_bucket": "high",
    "confidence_label": "deterministic",
    "reason_codes": [
      "weak_current_evidence",
      "budget_softcap_active"
    ],
    "runtime_truth_ref": {
      "owner_packet": "023",
      "evidence_class": "placeholder_or_simulated_step_completion"
    }
  },
  "budget_pressure": {
    "pressure_source": "team_sizing",
    "pressure_level": "softcap",
    "policy_hint": "prefer_downgrade_before_escalate",
    "display_note": "budget pressure capped the active team"
  },
  "policy_decision": {
    "chosen_action": "downgrade",
    "action_reason": "high_difficulty_plus_softcap_budget_pressure",
    "repair_lineage_ref": "packet-020:last-stable-checkpoint"
  },
  "display_note": "placeholder orchestration proof only"
}
```

Behavior:

- the only bounded action values are `keep`, `retry`, `clarify`, `downgrade`, and `escalate`
- budget hints may influence action choice but do not create a second proof or lifecycle schema
- packet-023 proof wording remains canonical
- packet-020 repair lineage may be linked when the chosen action overlaps with the current repair
  seam
- durable lifecycle state may narrow or suppress forward-action wording when the workflow is
  paused, cancelled, or terminated

## Result Surface Contract

Current result surfaces remain authoritative:

- `TaskResultView`
- `SessionRetainedResultView`
- `AutonomyStatusView`
- `OperatorResultPreview`

Expected behavior:

- each surface can expose the latest packet-owned step-policy summary
- task and autonomy remain the full canonical surfaces
- session and operator projections remain compact summaries of the same packet-owned data
- no new endpoint is introduced

## Proof-Honesty Contract

Packet `025` must preserve packet-023 proof wording exactly where placeholder completion is the
best available evidence.

Expected behavior:

- a step-policy summary can say that the runtime completed a step boundary
- a step-policy summary cannot say that placeholder completion proved grounded task execution
- packet-025 display wording may summarize packet-023 proof posture, but it may not strengthen it

## Relationship To Existing Surfaces

The following existing surfaces remain authoritative baseline inputs:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `scripts/tests/test_live_runtime_proof_smoke.py`

This packet only extends them with:

- one explicit deterministic step-difficulty surface
- one bounded action-decision vocabulary
- one bounded budget-pressure summary
- one coherent projection through current result surfaces

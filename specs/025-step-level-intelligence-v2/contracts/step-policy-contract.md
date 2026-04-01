# Contract: Step Policy Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one shared step-policy contract that scores the current step, chooses one bounded action,
and projects that summary through existing inspect surfaces without competing with packet `023`
truth ownership or packet `020` repair ownership.

This packet does **not** create a new proof-boundary schema. Packet `023` remains the owner of
run-trace taxonomy and proof-boundary language.

## Canonical Mapping

The contract for this packet is:

- `StepDifficultyAssessment` scores the current step using deterministic current-state inputs
- `StepBudgetPressureSummary` carries bounded budget hints that can influence action choice
- `StepPolicyDecision` chooses one bounded action from `keep`, `retry`, `clarify`, `downgrade`,
  and `escalate`
- `StepPolicySummaryView` projects those packet-owned summaries onto existing inspect surfaces
- `proof_boundary_ref` and any grounding status remain packet-023-owned references that packet
  `025` consumes but does not redefine

No packet-025 surface may become a competing run-trace or proof-boundary contract.

## Canonical Evidence Shape

Example authoritative payload shape:

```json
{
  "step_id": "result_track",
  "difficulty_assessment": {
    "difficulty_bucket": "high",
    "confidence_label": "deterministic",
    "reason_codes": [
      "weak_current_evidence",
      "unstable_recent_step_history"
    ],
    "grounding_status_ref": {
      "owner_packet": "023",
      "grounding_status": "placeholder_completion"
    }
  },
  "budget_pressure": {
    "budget_root": "runtime.task_path",
    "pressure_level": "softcap",
    "policy_hint": "prefer_downgrade_before_escalate"
  },
  "policy_decision": {
    "chosen_action": "downgrade",
    "action_reason": "high_difficulty_plus_softcap_budget_pressure",
    "repair_lineage_ref": "packet-020:last-stable-checkpoint"
  },
  "proof_boundary_ref": {
    "owner_packet": "023",
    "proof_boundary": "supported task path only"
  },
  "display_note": "placeholder orchestration proof only"
}
```

Behavior:

- the only bounded action values are `keep`, `retry`, `clarify`, `downgrade`, and `escalate`
- budget hints may influence action choice but do not create a second proof schema
- `grounding_status_ref` and `proof_boundary_ref` are packet-023-owned references
- packet `020` repair lineage may be linked when the chosen action overlaps with the current repair
  seam, but packet `025` does not replace that lineage

## Event Taxonomy Guidance

Packet `025` treats the OpenAI Responses event taxonomy as the canonical input baseline for
streamed step-policy terminology.

Expected posture:

- use the current Responses semantic event model as the source for streamed event naming guidance
- re-confirm the exact official streaming reference page before final implementation freeze
- do not let event-naming alignment turn packet `025` into the owner of packet-023 trace schema

## Task Surface Contract

`task.result` remains the task-facing authoritative inspection surface for the latest step-policy
summary.

Expected behavior:

- task inspection exposes the latest bounded score, action, and budget summary
- task inspection carries packet-023-owned proof references without redefining them
- the task surface can state explicitly that a result is placeholder orchestration proof

## Autonomy Surface Contract

`AutonomyStatusView` remains the operator-facing status surface for packet-owned summaries.

Expected behavior:

- autonomy status can show the latest bounded step-policy summary
- autonomy status remains a summary surface, not a competing trace owner
- autonomy status points back to task inspection for the canonical full view when needed

## Operator Summary Contract

Any packet-owned operator-facing summary must remain a bounded projection of current inspect
surfaces.

Expected behavior:

- no new endpoint is introduced
- the operator summary shows score, action, budget hint, and explicit placeholder-vs-grounded
  wording if UI work is in scope
- the summary does not widen into a new dashboard or a new run-trace owner

## Relationship To Existing Surfaces

The following existing surfaces remain authoritative baseline inputs:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `scripts/tests/test_live_runtime_proof_smoke.py`

This packet only extends them with:

- one explicit deterministic step-scoring surface
- one bounded action-decision vocabulary
- one bounded budget-aware summary
- one coherent projection through current inspect surfaces

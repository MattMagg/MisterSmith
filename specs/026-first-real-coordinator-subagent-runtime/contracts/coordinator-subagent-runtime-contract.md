# Contract: Coordinator-Subagent Runtime Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Scaffold note

This is a scaffold contract written before packets `022` through `025` are complete. It defines
the packet `026` contract shape now, but it must be revised before implementation starts.

## Design goal

Freeze one bounded contract for the first honest local coordinator-subagent runtime so later
implementation does not have to rediscover what "real coordinator-subagent runtime" means.

This packet does **not**:

- redefine packet `022` durability ownership
- redefine packet `023` trace or proof-boundary ownership
- redefine packet `024` boundary-hardening ownership
- redefine packet `025` step-policy ownership
- widen into federation or capability discovery

## Canonical mapping

The contract for packet `026` is:

- `CoordinatorDelegationRecord` is the required visible record that the coordinator delegated one
  bounded job to one subagent
- `SubagentStateRecord` is the required visible state surface for that delegated job
- `DelegatedWorkEvidenceRef` is the required proof reference for what the delegated job actually
  produced
- `CoordinatorMergeDecision` is the required visible record of merge, clarify, reassign, stop, or
  collapse decisions owned by the coordinator
- `CoordinatorRuntimeProofView` is the required joined view on task result, autonomy status, and
  run detail

No other packet `026` surface may claim real coordinator-subagent runtime success without these
records.

## Packet success rule

Packet `026` success requires:

- at least one visible coordinator-owned delegation record
- visible delegated subagent state
- grounded delegated work evidence for at least one delegated job
- visible coordinator merge or recovery decisions when the run requires them
- explicit proof text when delegated work remains placeholder-only

Packet `026` does **not** require fan-out on every task. Honest sequential collapse remains valid
when the smallest-workflow rule says branching is unnecessary.

## Canonical evidence shape

Example authoritative payload shape:

```json
{
  "workflow_id": "11111111-1111-1111-1111-111111111111",
  "session_id": "22222222-2222-2222-2222-222222222222",
  "coordinator_agent_id": "agent-coordinator-1",
  "delegations": [
    {
      "delegation_id": "delegation-1",
      "subagent_id": "agent-worker-1",
      "delegated_job_label": "audit backend boundaries",
      "status": "running"
    }
  ],
  "subagent_states": [
    {
      "delegation_id": "delegation-1",
      "current_state": "running",
      "state_reason": "grounded repo inspection in progress"
    }
  ],
  "delegated_work_evidence": [
    {
      "delegation_id": "delegation-1",
      "evidence_kind": "grounded",
      "evidence_summary": "bounded evidence captured for delegated backend audit"
    }
  ],
  "coordinator_decisions": [
    {
      "decision_id": "decision-1",
      "decision_kind": "merge",
      "decision_outcome": "accepted"
    }
  ],
  "proof_boundary": "real coordinator-subagent runtime satisfied"
}
```

Behavior:

- placeholder-only delegated completion must remain explicitly non-grounded
- mixed runs may include both grounded and non-grounded delegated outcomes
- coordinator decisions remain separate from subagent state, but must be linkable to it
- sequential collapse must not fabricate delegation records

## Task surface contract

`task.result` remains one authoritative operator-facing inspection surface.

Expected behavior:

- task result exposes the latest joined `CoordinatorRuntimeProofView`
- task result can explain why a run did or did not satisfy the packet `026` proof standard
- task result can distinguish graph success from real coordinator-subagent success

## Autonomy surface contract

`AutonomyStatusView` remains the main runtime status surface.

Expected behavior:

- autonomy status exposes delegation, subagent state, and proof-boundary summary data
- autonomy status shows collapse honestly when the run stayed sequential
- autonomy status stays consistent with task result and does not invent extra coordination claims

## Operator run-detail contract

The operator-console run detail remains a bounded summary surface.

Expected behavior:

- run detail renders delegation, state, evidence, and decision summaries as first-class content
- run detail does not require raw payload digging to understand the proof boundary
- run detail stays bounded and does not widen into a new dashboard or observability redesign

## Relationship to upstream packets

Packet `026` depends on upstream ownership from packets `022` through `025`:

- packet `022`: durable lifecycle and effect-boundary semantics
- packet `023`: run-trace and proof-boundary semantics
- packet `024`: security-boundary and delegated-authority semantics
- packet `025`: step-policy and escalation semantics

Before implementation starts, this contract must be revised to match what those packets actually
landed.

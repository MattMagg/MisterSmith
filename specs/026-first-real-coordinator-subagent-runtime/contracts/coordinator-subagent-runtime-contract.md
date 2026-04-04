# Contract: Coordinator-Subagent Runtime Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design goal

Freeze one bounded contract for the first honest local coordinator-subagent runtime so later
implementation does not have to rediscover what "real coordinator-subagent runtime" means.

This packet does **not**:

- redefine packet `022` durability ownership
- redefine packet `023` trace or proof-boundary ownership
- redefine packet `024` boundary-hardening ownership
- redefine packet `025` step-policy ownership
- widen into federation or capability discovery
- widen into a new endpoint or dashboard

## Canonical mapping

The contract for packet `026` is:

- `CoordinatorDelegationRecord` is the required visible record that the coordinator delegated one
  bounded job to one child
- `CoordinatorSubordinateInboxRecord` is the required visible intake record for child completion,
  blocked, clarify, cancel, sibling-abort, and user-interrupt events
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
- visible delegated child state
- visible subordinate inbox activity for delegated child work
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
  "coordinator_agent_id": "agent-coordinator-1",
  "delegation_records": [
    {
      "delegation_id": "delegation-1",
      "session_id": "22222222-2222-2222-2222-222222222222",
      "child_role": "explorer",
      "subagent_id": "agent-worker-1",
      "delegated_job_label": "audit backend boundaries",
      "allowed_follow_up_actions": ["clarify", "resume", "stop", "inspect"],
      "status": "running"
    }
  ],
  "subordinate_inbox": [
    {
      "delegation_id": "delegation-1",
      "event_id": "event-1",
      "event_sequence": 1,
      "event_kind": "blocked"
    }
  ],
  "subagent_states": [
    {
      "delegation_id": "delegation-1",
      "current_state": "blocked",
      "state_reason": "grounded repo inspection needs clarification"
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
      "decision_kind": "clarify",
      "decision_outcome": "accepted"
    }
  ],
  "proof_boundary": "real coordinator-subagent runtime satisfied",
  "session_follow_up_note": "preserve session_id, coordinator_agent_id, delegated child identity, and evidence refs only; do not assume transcript replay"
}
```

Behavior:

- placeholder-only delegated completion must remain explicitly non-grounded
- mixed runs may include both grounded and non-grounded delegated outcomes
- coordinator decisions remain separate from child state, but must be linkable to it
- subordinate inbox events stay ordered within one delegated child stream
- child scratch context stays private by default
- only root-owned shared channels may carry registration, cancellation, runtime-truth projection,
  and capability-enforcement data
- sequential collapse must not fabricate delegation records

## Ordered parallel batch rule

When the runtime uses bounded parallel child work:

- each child stream must retain deterministic event ordering
- sibling cancellation must project explicit abort outcomes for affected children
- user interrupts must surface as explicit child outcomes, not silent disappearance
- merge and collapse decisions remain coordinator-owned even when children terminate early

## Child role rule

The first slice uses bounded child-role profiles instead of prompt-only specialization:

- `explorer`
- `planner`
- `verifier`

The implementation may map those profiles onto current repo-native role seams, but the packet
contract must stay operator-visible and role-bounded.

## Task surface contract

`task.result` remains one authoritative operator-facing inspection surface.

Expected behavior:

- task result exposes the latest joined `CoordinatorRuntimeProofView`
- task result explains why a run did or did not satisfy the packet `026` proof standard
- task result distinguishes graph success from real coordinator-subagent success
- retained session assistant results may project a bounded `coordinator_runtime_follow_up` object,
  but that follow-up view must stay limited to stable identifiers, proof text, and evidence refs

## Autonomy surface contract

`AutonomyStatusView` remains the main runtime status surface.

Expected behavior:

- autonomy status exposes delegation, subordinate inbox, child state, and proof-boundary summary
  data
- autonomy status shows collapse honestly when the run stayed sequential
- autonomy status stays consistent with task result and does not invent extra coordination claims

## Operator run-detail contract

The operator-console run detail remains a bounded summary surface.

Expected behavior:

- run detail renders delegation, subordinate inbox, state, evidence, and coordinator decision
  summaries as first-class content
- run detail does not require raw payload digging to understand the proof boundary
- run detail stays bounded and does not widen into a new dashboard or observability redesign

## Relationship to upstream packets

Packet `026` depends on upstream ownership from packets `022` through `025`:

- packet `022`: durable lifecycle and effect-boundary semantics
- packet `023`: run-trace and proof-boundary semantics
- packet `024`: security-boundary and delegated-authority semantics
- packet `025`: step-policy and escalation semantics

Packet `026` consumes those seams by reference and does not redefine them.

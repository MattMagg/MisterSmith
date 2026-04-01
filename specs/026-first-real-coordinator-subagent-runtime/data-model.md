# Data Model: First Real Coordinator-Subagent Runtime

## Scaffold note

These entities are frozen now as scaffold names and intent. Final field wording must be revised
before implementation after packet `022` through `025` land.

## Coordinator-runtime entities

### `CoordinatorDelegationRecord`

- `delegation_id`: stable identifier for one coordinator-owned delegation event
- `workflow_id`: stable workflow identifier for the run
- `session_id`: optional stable session identifier for session-aware follow-up
- `coordinator_agent_id`: stable coordinator identity for the run
- `subagent_id`: delegated subagent identity
- `delegated_job_label`: short plain-English label for the bounded delegated job
- `delegated_scope_ref`: link to the branch, node, or bounded work unit the subagent owns
- `delegation_reason`: plain-English rationale for why the job was delegated
- `created_at`: delegation creation time
- `status`: current high-level delegation state

### `SubagentStateRecord`

- `delegation_id`: stable reference back to the owning delegation record
- `subagent_id`: delegated subagent identity
- `current_state`: one of `queued`, `delegated`, `running`, `blocked`, `clarified`,
  `reassigned`, `merged`, `completed`, `failed`, or `collapsed`
- `previous_state`: last visible state, if any
- `state_reason`: short explanation for the current state
- `state_updated_at`: last state-change time
- `coordinator_action_ref`: optional reference to the latest coordinator decision tied to this
  state

### `DelegatedWorkEvidenceRef`

- `delegation_id`: stable reference to the delegated job
- `evidence_kind`: grounded, placeholder-only, or mixed
- `evidence_summary`: short summary of what the delegated job actually produced
- `artifact_refs`: references to result artifacts, task output, or other bounded evidence
- `proof_boundary_note`: explicit note explaining what this evidence does and does not prove
- `recorded_at`: time the evidence reference was written

### `CoordinatorMergeDecision`

- `decision_id`: stable identifier for one coordinator-owned merge or recovery decision
- `workflow_id`: stable workflow identifier
- `decision_kind`: merge, clarify, reassign, stop, or collapse
- `input_refs`: delegated job or state references the coordinator used
- `decision_reason`: short explanation for the decision
- `decision_outcome`: accepted, deferred, blocked, or terminal
- `decided_at`: time the decision was made

### `CoordinatorRuntimeProofView`

- `workflow_id`: stable workflow identifier
- `coordinator_agent_id`: stable coordinator identity
- `delegation_refs`: ordered set of delegation records included in the proof view
- `subagent_state_refs`: ordered set of visible subagent states
- `evidence_refs`: ordered set of delegated work evidence references
- `decision_refs`: ordered set of coordinator merge or recovery decisions
- `proof_boundary`: explicit statement of whether the run satisfied the packet `026` proof
  standard
- `session_follow_up_note`: note describing what session context can legitimately carry forward

## Invariants

- no run may satisfy packet `026` proof without at least one `CoordinatorDelegationRecord`
- no run may satisfy packet `026` proof without visible `SubagentStateRecord` data
- `DelegatedWorkEvidenceRef.evidence_kind = placeholder-only` cannot satisfy packet success
- `CoordinatorMergeDecision` remains coordinator-owned even when the final outcome is collapse
- sequential collapse is a valid honest outcome when fan-out is not justified
- session-aware follow-up may preserve identifiers and evidence references, but must not imply
  unlimited transcript reuse
- all final wording and field names must pass through the pre-implementation revision gate

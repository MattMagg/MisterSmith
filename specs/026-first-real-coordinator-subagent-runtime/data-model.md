# Data Model: First Real Coordinator-Subagent Runtime

## Coordinator-runtime entities

### `CoordinatorDelegationRecord`

- `delegation_id`: stable identifier for one coordinator-owned delegation event
- `workflow_id`: stable workflow identifier for the run
- `session_id`: optional stable session identifier for session-aware follow-up
- `coordinator_agent_id`: stable coordinator identity for the run
- `child_role`: bounded child profile such as `explorer`, `planner`, or `verifier`
- `subagent_id`: stable delegated child identity
- `delegated_job_label`: short plain-English label for the bounded delegated job
- `delegated_scope_ref`: link to the branch, node, or bounded work unit the child owns
- `delegation_reason`: short rationale for why the job was delegated
- `allowed_follow_up_actions`: bounded list across `clarify`, `resume`, `stop`, and `inspect`
- `created_at`: delegation creation time
- `status`: current high-level delegation state

### `CoordinatorSubordinateInboxRecord`

- `delegation_id`: stable reference back to the owning delegation record
- `event_id`: stable identifier for one subordinate inbox event
- `event_sequence`: monotonic ordering value within one delegated child stream
- `event_kind`: `completed`, `blocked`, `clarify_requested`, `cancelled`, `sibling_cancelled`, or
  `user_interrupted`
- `event_payload_ref`: reference to the child event payload or evidence
- `recorded_at`: intake time for the event
- `visible_to`: packet-owned visibility rule for coordinator and operator surfaces

### `SubagentStateRecord`

- `delegation_id`: stable reference back to the owning delegation record
- `subagent_id`: delegated child identity
- `current_state`: one of `queued`, `delegated`, `running`, `blocked`, `clarified`,
  `reassigned`, `merged`, `completed`, `failed`, or `collapsed`
- `previous_state`: last visible state, if any
- `state_reason`: short explanation for the current state
- `state_updated_at`: last state-change time
- `coordinator_action_ref`: optional reference to the latest coordinator decision tied to this
  state

### `DelegatedWorkEvidenceRef`

- `delegation_id`: stable reference to the delegated job
- `evidence_kind`: `grounded`, `placeholder-only`, or `mixed`
- `evidence_summary`: short summary of what the delegated job actually produced
- `artifact_refs`: references to result artifacts, task output, or other bounded evidence
- `proof_boundary_note`: explicit note explaining what this evidence does and does not prove
- `recorded_at`: time the evidence reference was written

### `CoordinatorMergeDecision`

- `decision_id`: stable identifier for one coordinator-owned merge or recovery decision
- `workflow_id`: stable workflow identifier
- `decision_kind`: `merge`, `clarify`, `reassign`, `stop`, or `collapse`
- `input_refs`: delegated job or state references the coordinator used
- `decision_reason`: short explanation for the decision
- `decision_outcome`: `accepted`, `deferred`, `blocked`, or `terminal`
- `decided_at`: time the decision was made

### `CoordinatorRuntimeProofView`

- `workflow_id`: stable workflow identifier
- `coordinator_agent_id`: stable coordinator identity
- `delegation_records`: ordered set of delegation records included in the proof view
- `subordinate_inbox`: ordered set of subordinate inbox events included in the proof view
- `subagent_states`: ordered set of visible child states
- `delegated_work_evidence`: ordered set of delegated work evidence references
- `coordinator_decisions`: ordered set of coordinator merge or recovery decisions
- `proof_boundary`: explicit statement of whether the run satisfied the packet `026` proof
  standard
- `session_follow_up_note`: note describing what session context can legitimately carry forward

### `coordinator_runtime_follow_up` assistant-result projection

- `session_id`: stable session identifier when the delegated run was session-backed
- `coordinator_agent_id`: stable coordinator identity that may be reused on follow-up
- `proof_boundary`: packet `026` proof statement copied from the runtime proof view
- `session_follow_up_note`: explicit bounded carry-forward rule
- `delegation_ids`: stable delegation identifiers that may be referenced again
- `delegated_child_ids`: stable child identities that may be clarified, resumed, stopped, or
  inspected again
- `decision_ids`: stable coordinator decision identifiers retained for follow-up context
- `evidence_refs`: stable artifact or task references from delegated work evidence

## Invariants

- no run may satisfy packet `026` proof without at least one `CoordinatorDelegationRecord`
- no run may satisfy packet `026` proof without visible `SubagentStateRecord` data
- no run may satisfy packet `026` proof without grounded `DelegatedWorkEvidenceRef` data for at
  least one delegated job
- `DelegatedWorkEvidenceRef.evidence_kind = placeholder-only` cannot satisfy packet success
- subordinate inbox events must remain ordered within one delegated child stream
- `CoordinatorMergeDecision` remains coordinator-owned even when the final outcome is collapse
- child scratch context stays private by default, and shared channels are limited to registration,
  cancellation, runtime-truth projection, and capability enforcement
- sequential collapse is a valid honest outcome when fan-out is not justified
- session-aware follow-up may preserve identifiers and evidence references, but must not imply
  unlimited transcript reuse

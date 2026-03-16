# Data Model: Multi-Turn Same-Agent Conversations

**Date**: 2026-03-16  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Source Map

| Source | Data-model impact |
| ------ | ----------------- |
| `crates/mister-smith-app/src/execution.rs` | Root `workflow_id` remains the existing root `task_id` for each accepted turn |
| `crates/mister-smith-http/src/server.rs` | Current runtime submission contract shows where session-aware wrappers must attach |
| `crates/mister-smith-app/src/autonomy.rs` | Workflow autonomy stays the deep inspect surface and needs session linkage |
| `crates/mister-smith-events/src/autonomy.rs` | Session linkage must fit inside existing workflow autonomy status views |
| `crates/mister-smith-persistence/migrations/00001_initial_schema.sql` | Session storage should extend the existing `tasks.*` persistence boundary |
| `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` | Clarifies the one-shot baseline that the new session model wraps |

## Entities

### ConversationSession

Durable envelope for one retained same-agent conversation.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| session_id | `SessionId` | Required, stable | Conversation identifier minted once at session creation |
| coordinator_agent_id | `AgentId` | Required, stable | Session-scoped coordinator identity reused across turns |
| status | `SessionStatus` | Required | `active` or `ended` in the initial slice |
| provider_kind | `String` | Required | Provider used for the accepted turns in this session |
| model_id | `String` | Required | Model used for the accepted turns in this session |
| active_workflow_id | `Option<TaskId>` | Optional | Root workflow currently handling the active turn |
| last_completed_workflow_id | `Option<TaskId>` | Optional | Most recent terminal workflow for the session |
| turn_count | `u32` | Required, monotonic | Number of accepted turns |
| retained_context | `serde_json::Value` | Required | Persisted conversation context used to reconstruct the next turn |
| created_at | `DateTime<Utc>` | Required | Creation timestamp |
| updated_at | `DateTime<Utc>` | Required | Last session mutation time |
| ended_at | `Option<DateTime<Utc>>` | Optional | Logical close time |

**Invariant**: `coordinator_agent_id` is stable for the life of the session.

**Invariant**: a session with `status = ended` cannot accept new turns.

**Invariant**: `active_workflow_id` is `None` when the session is idle.

---

### SessionTurn

Ordered record of one user turn inside a session.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| turn_id | `Uuid` | Required, stable | Durable turn identifier |
| session_id | `SessionId` | Required | Owning conversation session |
| turn_index | `u32` | Required, monotonic per session | 1-based accepted turn order |
| workflow_id | `TaskId` | Required, unique | Root workflow created for this turn |
| user_message | `String` | Required | Operator-supplied turn text |
| result_summary | `Option<serde_json::Value>` | Optional | Terminal assistant result or summary snapshot |
| status | `TurnStatus` | Required | Mirrors root workflow lifecycle for the turn |
| created_at | `DateTime<Utc>` | Required | Turn creation time |
| completed_at | `Option<DateTime<Utc>>` | Optional | Terminal completion time |

**Invariant**: each accepted turn creates exactly one new root `workflow_id`.

**Invariant**: `turn_index` is unique within one session and increases by one for each accepted
turn.

---

### WorkflowSessionLink

Minimal linkage carried in root workflow persistence and autonomy status.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| session_id | `SessionId` | Required | Owning conversation session |
| workflow_id | `TaskId` | Required | Root workflow for the turn |
| turn_index | `u32` | Required | Turn order within the session |
| coordinator_agent_id | `AgentId` | Required | Session coordinator identity |

**Invariant**: the link is written exactly once for the root turn workflow and remains stable for
the life of that workflow.

---

### SessionStatusView

Operator-facing inspect response for a session.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| session_id | `SessionId` | Required | Conversation identifier |
| status | `SessionStatus` | Required | Current lifecycle state |
| coordinator_agent_id | `AgentId` | Required | Stable same-agent identifier |
| provider_kind | `String` | Required | Current provider attribution |
| model_id | `String` | Required | Current model attribution |
| active_workflow_id | `Option<TaskId>` | Optional | Active turn root workflow when busy |
| last_completed_workflow_id | `Option<TaskId>` | Optional | Most recent completed turn root |
| turn_count | `u32` | Required | Accepted turn count |
| turns | `Vec<SessionTurnSummary>` | Required | Ordered turn summaries |
| ended_at | `Option<DateTime<Utc>>` | Optional | Logical close time |

**Invariant**: `turns` are returned in ascending `turn_index` order.

---

### RetainedSessionContext

Persisted context snapshot used to reconstruct the next coordinator turn.

| Field | Type | Constraints | Description |
| ----- | ---- | ----------- | ----------- |
| last_user_message | `Option<String>` | Optional | Most recent user turn |
| last_assistant_result | `Option<serde_json::Value>` | Optional | Most recent assistant result |
| transcript_summary | `Vec<serde_json::Value>` | Required | Ordered summaries of prior turns |
| latest_workflow_id | `Option<TaskId>` | Optional | Last workflow reflected in this snapshot |

**Invariant**: retained context is sufficient to reconstruct the next turn without relying on a
live in-memory planner state object.

## Relationships

```text
ConversationSession 1 --- N SessionTurn
SessionTurn 1 --- 1 root workflow record in tasks.records
root workflow record 1 --- 1 WorkflowSessionLink
WorkflowSessionLink -> workflow autonomy view (optional session linkage fields)
```

## Lifecycle Rules

### Session lifecycle

`active` -> `ended`

Notes:

- busy versus idle is derived from whether `active_workflow_id` is present
- the first slice does not reopen ended sessions

### Turn lifecycle

`queued` -> `running` -> (`completed` | `failed`)

Notes:

- turn lifecycle mirrors the root workflow lifecycle
- a failed turn does not force the whole session to end

## Identifier Guarantees

- `session_id` never changes after create
- `coordinator_agent_id` never changes after create
- `workflow_id` changes on every accepted turn
- `workflow_id == task_id` for the root workflow record used by existing one-shot inspection

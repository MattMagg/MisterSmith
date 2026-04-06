# Data Model: Chat-First CLI Loop

## Packet-owned entities

### `CliSessionLoopView`

- `session_id`: stable durable session identifier
- `title`: compact user-facing session title
- `coordinator_agent_id`: stable coordinator identity already attached to the session
- `provider_kind`: runtime-attributed provider for the session
- `model_id`: runtime-attributed model for the session
- `active_workflow_id`: current active turn workflow, when one exists
- `last_completed_workflow_id`: most recent completed or failed turn workflow
- `loop_state`: current loop posture such as `ready`, `turn_pending`, `turn_running`, `blocked`,
  `degraded`, or `ended`
- `current_turn_state`: optional inline state summary for the turn currently in focus
- `transcript_entries`: ordered retained conversation entries shown in the loop
- `control_state`: in-session steering posture for model, permissions, config, status, and MCP
- `truth_notices`: ordered inline notices describing degraded, busy, ended, or proof-limited
  state

### `InlineTurnState`

- `workflow_id`: workflow identifier for the current or most recent turn
- `turn_index`: accepted turn number within the session
- `turn_status`: user-visible state such as `accepted`, `running`, `completed`, `failed`, or
  `blocked`
- `lifecycle_state`: underlying lifecycle projection retained from current session truth
- `result_preview`: bounded preview for the most recent available result
- `proof_boundary_note`: explicit summary of what the current turn result does and does not prove
- `state_source`: whether the state is derived from the live runtime path or retained durable
  state

### `TranscriptEntry`

- `turn_index`: ordered position of the turn inside the session
- `workflow_id`: stable workflow identifier for that turn
- `user_message`: original user turn text
- `assistant_result_preview`: compact retained result summary, when available
- `lifecycle_state`: retained lifecycle view for the turn
- `resume_provenance`: restart or resume lineage, when available
- `entry_kind`: `retained_turn`, `current_turn`, or `system_notice`

### `LoopControlState`

- `selected_provider_kind`: user-selected provider override, when set
- `selected_model_id`: user-selected model override, when set
- `permission_mode`: current permission posture
- `config_posture`: current config posture
- `status_view`: current status rendering mode
- `mcp_posture`: current MCP posture

### `LoopTruthNotice`

- `notice_kind`: stable machine-readable notice type
- `severity`: relative severity for CLI display
- `summary`: user-facing truth statement
- `support_surface`: related support command or surface, when one exists
- `blocks_live_turn`: whether the current notice prevents the next live turn from proceeding

## State transitions

### Loop posture

- `ready` -> `turn_pending` when a new follow-up turn is accepted
- `turn_pending` -> `turn_running` when the active workflow is live
- `turn_running` -> `ready` when the turn completes and the session remains active
- `turn_running` -> `blocked` when the runtime or session state prevents normal completion
- `ready` -> `degraded` when the runtime becomes unavailable but retained context still exists
- any active posture -> `ended` when the session is logically ended

### Truth posture

- live runtime state may move to retained-only state, but the loop must preserve context and mark
  `state_source` honestly
- busy and ended states remain exclusive with accepting a new live turn
- proof-boundary notes may change per turn, but they must remain visible whenever a result preview
  is shown

## Invariants

- packet `031` must not create a second session identity or a second retained history model
- exactly one session remains in focus inside the live loop at a time
- resumed sessions preserve stored `LoopControlState` instead of resetting to a generic default
- retained-only or degraded views must never be presented as if they were current live runtime
  state
- `LoopTruthNotice` must stay inline and visible without replacing the transcript and next input
- the current turn may be pending, running, blocked, completed, or failed, but the loop must
  preserve continuity across all of those states

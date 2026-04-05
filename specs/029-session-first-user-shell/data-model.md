# Data Model: Session-First User Shell

## Core product entities

### `Session`

- `session_id`: stable retained session identifier shared by CLI and GUI
- `coordinator_agent_id`: stable coordinator identity reused across accepted turns
- `status`: current session lifecycle state
- `active_workflow_id`: optional active work owned by the session
- `last_completed_workflow_id`: optional latest completed or failed work owned by the session
- `turn_count`: number of accepted turns
- `provider_kind`: current provider attribution visible to the user
- `model_id`: current model attribution visible to the user
- `updated_at`: most recent session update time
- `ended_at`: logical close time when the session ends

### `SessionSummaryCard`

- `session_id`: stable identifier for the recent-session entry
- `title`: short recognizable user-facing label for the session
- `status`: session lifecycle state shown in recent-session browsing
- `last_preview`: compact preview of the latest retained assistant result or session state
- `model_id`: visible model attribution for the session
- `provider_kind`: visible provider attribution for the session
- `updated_at`: last activity time used for ordering recent sessions
- `resume_target`: direct action that reopens this session from CLI or GUI

### `SessionTranscriptProjection`

- `session_id`: owning session identifier
- `ordered_turns`: retained turn summaries shown after resume
- `last_user_message`: latest retained user prompt
- `last_assistant_result`: latest retained assistant-facing result summary
- `resume_provenance`: restart-resume lineage when present

### `SessionControlState`

- `session_id`: owning session identifier
- `model_selection`: currently chosen model or profile
- `permission_mode`: current approval or permission posture
- `config_view`: current searchable configuration state exposed in-session
- `status_view`: current runtime and support-state summary visible in-session
- `mcp_posture`: current MCP availability and health summary

### `StartupHomeView`

- `recent_sessions`: ordered list of `SessionSummaryCard` items
- `resume_last_target`: direct recent-session target when a prior session exists
- `new_session_target`: primary action for starting fresh work
- `startup_warnings`: ordered support-state notices shown inline at entry
- `config_target`: direct path to configuration from the home view

### `SupportStatusNotice`

- `notice_kind`: warning class such as runtime unavailable, auth degraded, doctor warning, or MCP
  degraded
- `severity`: informational, warning, or blocking
- `summary`: plain-language explanation visible at startup or in-session
- `affected_actions`: any session actions that are limited by the notice
- `recovery_target`: the support surface the user may open to recover

### `SharedSessionProtocolMessage`

- `message_kind`: home snapshot, session open, session resume, session continue, or control update
- `session_id`: optional owning session identifier
- `payload_ref`: packet-owned reference to the shared session or control payload
- `issued_by`: CLI or GUI front end that emitted the message
- `created_at`: time the message was issued

## Invariants

- there is exactly one canonical retained session identity for a given session across CLI and GUI
- recent-session browsing and resume-last must resolve against the same underlying retained session
  data
- the GUI must not invent a second transcript or summary model for a session already known to the
  CLI
- core live-session controls must resolve against one shared `SessionControlState`
- startup warnings may limit actions honestly, but they must not hide recent-session visibility or
  the primary start-new action unless the action is truly blocked
- support surfaces remain reachable, but they do not become the default home or the first-level
  navigation model

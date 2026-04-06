# Data Model: Session-First CLI Shell

## Core Entities

### Session

- **Purpose**: The durable unit of CLI user work.
- **Key fields**:
  - `session_id`
  - `coordinator_agent_id`
  - `status`
  - `provider_kind`
  - `model_id`
  - `active_workflow_id`
  - `last_completed_workflow_id`
  - `turn_count`
  - `created_at`
  - `updated_at`
- **Rules**:
  - one retained session identity remains canonical for each CLI session
  - resume and browse flows resolve against this same durable session identity

### SessionSummaryRow

- **Purpose**: Compact recent-session data shown in startup and browse flows.
- **Key fields**:
  - `session_id`
  - `title`
  - `status`
  - `last_preview`
  - `provider_kind`
  - `model_id`
  - `updated_at`
- **Rules**:
  - summary rows must give enough context for a user to choose the right prior session
  - summary ordering stays consistent between startup and broader history views

### SessionTranscriptProjection

- **Purpose**: The retained history or summary history shown when the user resumes a CLI session.
- **Key fields**:
  - `session_id`
  - ordered turn summaries
  - retained assistant result projections
  - restart-resume provenance when present
- **Rules**:
  - the retained transcript remains tied to one durable session identity
  - degraded support state does not erase retained history

### SessionControlState

- **Purpose**: The in-session state a user can steer from the live CLI shell.
- **Key fields**:
  - `session_id`
  - model selection
  - permission mode
  - config posture
  - status view
  - MCP posture
- **Rules**:
  - control changes stay associated with the live session
  - support warnings may limit an action, but they do not replace the session-first shell

### StartupHomeView

- **Purpose**: The recent-first CLI entry surface.
- **Key fields**:
  - recent session rows
  - resume-last target
  - start-new target
  - startup warnings
  - config target
- **Rules**:
  - the startup home remains useful when no retained sessions exist
  - warnings stay visible without burying the main session actions

### SupportStatusNotice

- **Purpose**: The warning or degraded-state message shown at startup or during live session use.
- **Key fields**:
  - notice kind
  - severity
  - user-facing summary
  - linked support surface when needed
- **Rules**:
  - notices explain what is blocked and what remains available
  - notices do not replace the main session flow

## Invariants

- the CLI uses one canonical retained-session model already present in the repo
- quick resume and broader recent-session browsing resolve against the same session identities
- startup home and broader recent-session browsing use the same recent-first ordering rules
- live-session controls operate on one shared session control state
- in-session control changes do not mint a second `session_id` or a second retained transcript
- session-shell control preferences stay durable and inspectable even when the current runtime path
  is unchanged in this packet
- startup warnings may limit actions honestly, but they must not hide recent sessions or start-new
- GUI parity and cross-surface continuity remain outside this packet

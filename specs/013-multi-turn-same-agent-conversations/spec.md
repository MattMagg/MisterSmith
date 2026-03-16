# Feature Specification: Multi-Turn Same-Agent Conversations

**Feature Branch**: `013-multi-turn-same-agent-conversations`  
**Created**: 2026-03-16  
**Status**: Draft  
**Input**: `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`,
`docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`, current runtime-backed
`mister-smith run` plus HTTP/CLI/autonomy surfaces

## Current Truth & Scope

The current honest runtime-backed operator path is:

- `mister-smith run`
- `POST /api/v1/tasks`
- `GET /api/v1/tasks/{task_id}`
- `mister-smith autonomy list`
- `mister-smith autonomy status --workflow-id <id>`

That path executes real persisted workflows, but it is still a one-shot submission surface. The
runtime currently reuses a process-global coordinator `agent_id`, yet it creates fresh planner
state per submitted workflow. That means it can expose a stable agent identifier without providing
an actual retained same-agent conversation contract.

This feature adds the smallest honest usability layer on top of that proven runtime:

- create a conversation session
- continue the same session with another user turn
- inspect the session and its turn-to-workflow lineage
- end the session without deleting its history

The initial slice is intentionally narrow:

- one human operator per session
- one session-scoped coordinator identity per session
- one active turn at a time per session
- workflow-scoped autonomy remains the deep inspection surface
- no shared sessions, no branchable histories, no force-cancel, no UI work

### Honest Same-Agent Guarantee

For this feature, "same agent" means:

- the same stable `coordinator_agent_id` for the life of the session
- the same persisted session context reconstructed for each new turn

It does **not** mean:

- one immortal in-memory actor object that must survive process restart
- fixed worker identities across turns

### Stable Identifier Contract

- `session_id`: stable conversation identifier created once and reused for every turn in that
  conversation
- `workflow_id`: stable root workflow identifier created once per accepted user turn
- `agent_id`: stable session-scoped coordinator identifier; worker IDs may vary and are not part
  of the same-agent contract

Relationship rules:

- one `session_id` owns many ordered `workflow_id` values
- each accepted turn creates exactly one new root `workflow_id`
- the root `workflow_id` remains the existing root `task_id` for backward compatibility with
  `GET /api/v1/tasks/{task_id}` and workflow autonomy inspection

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create And Continue One Conversation (Priority: P1)

An operator starts a conversation, gets back a stable `session_id`, then sends a follow-up turn
that is handled by the same retained coordinator identity instead of starting a brand-new
conversation.

**Why this priority**: Without create plus continue, the feature is still just one-shot workflow
submission and does not close the core usability gap.

**Independent Test**: Start one session, wait for the first workflow to reach a terminal state,
append one new turn to that same session, and verify the second turn reuses the same
`coordinator_agent_id` while producing a new `workflow_id`.

**Acceptance Scenarios**:

1. **Given** no existing session, **When** the operator creates a session with an initial message,
   **Then** the system returns a new `session_id`, a new root `workflow_id`, a
   `coordinator_agent_id`, and an accepted session state.
2. **Given** an existing open idle session, **When** the operator continues it with another user
   turn, **Then** the system reuses the same `session_id` and `coordinator_agent_id`, creates a
   new `workflow_id`, and records the new turn in order.
3. **Given** an existing session with an active turn already running, **When** the operator
   submits another turn, **Then** the system rejects the request with a conflict response instead
   of silently queueing or overwriting work.
4. **Given** a completed follow-up turn, **When** the operator compares both turn roots, **Then**
   each turn has its own root `workflow_id` and both remain linked to the same `session_id`.

---

### User Story 2 - Inspect Session Lineage And Current State (Priority: P1)

An operator can inspect a conversation session without digging through raw logs or direct database
queries. They can see the stable coordinator identity, ordered turns, active or last workflow, and
the mapping from session state to workflow autonomy state.

**Why this priority**: A conversation surface that cannot be inspected or resumed honestly is not
operator-usable.

**Independent Test**: Create a session with at least two turns, inspect it over HTTP or CLI, and
verify the response includes the session identifier, same coordinator identity, ordered turn
history, and the workflow IDs needed to inspect autonomy state.

**Acceptance Scenarios**:

1. **Given** a session with one or more accepted turns, **When** the operator inspects the session,
   **Then** they can see `session_id`, `coordinator_agent_id`, session lifecycle state,
   `active_workflow_id`, `last_completed_workflow_id`, and ordered turn summaries.
2. **Given** a workflow autonomy view for a session-owned turn, **When** the operator inspects the
   workflow, **Then** the autonomy output includes enough session linkage to correlate that
   workflow back to its owning `session_id` and turn index.
3. **Given** the runtime restarts after a session already exists, **When** the operator inspects
   that idle session, **Then** the session record and prior turns are still available without
   manual repair.

---

### User Story 3 - End A Session Cleanly (Priority: P2)

An operator can explicitly end a conversation session once they are done, while keeping its turn
history inspectable and preventing accidental follow-up turns on a closed conversation.

**Why this priority**: Explicit lifecycle closure keeps the operator contract honest and prevents
stale sessions from looking resumable when they are not.

**Independent Test**: End an idle session, confirm it remains inspectable, and verify that a new
turn cannot be appended afterward.

**Acceptance Scenarios**:

1. **Given** an idle open session, **When** the operator ends it, **Then** the system marks the
   session ended and retains its history for later inspection.
2. **Given** an ended session, **When** the operator tries to continue it, **Then** the system
   rejects the request with a clear ended-session error instead of reopening it implicitly.
3. **Given** an active running session, **When** the operator tries to end it in this initial
   slice, **Then** the system rejects the request with a conflict response rather than silently
   cancelling in-flight work.

### Edge Cases

- Continue requested for an unknown `session_id`
- Continue requested while the session already has an active `workflow_id`
- End requested for an already-ended session
- Inspect requested after the latest turn failed but the session is still open for a retry turn
- Runtime restart between turns, where the next turn must rebuild retained context from persisted
  session state rather than in-memory planner state

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a session-creation operation that accepts an initial user message
  and starts the first root workflow turn.
- **FR-002**: System MUST mint a new stable `session_id` exactly once per created conversation.
- **FR-003**: System MUST mint a new stable root `workflow_id` for every accepted user turn.
- **FR-004**: System MUST preserve one stable session-scoped `coordinator_agent_id` across every
  turn in the same session.
- **FR-005**: System MUST define the root `workflow_id` as the canonical root run identifier for a
  turn, and that identifier MUST remain usable anywhere the current root `task_id` is accepted.
- **FR-006**: System MUST persist enough session state and ordered turn history to inspect or
  continue an idle session after runtime restart.
- **FR-007**: System MUST define the same-agent guarantee as stable coordinator identity plus
  retained session context, not as a requirement for one immortal in-memory actor instance.
- **FR-008**: System MUST allow at most one active turn per session in this slice.
- **FR-009**: System MUST reject concurrent continue requests against a busy session with a clear
  conflict response.
- **FR-010**: System MUST provide inspect surfaces over HTTP and CLI for session state, ordered
  turns, active workflow linkage, and last completed workflow linkage.
- **FR-011**: System MUST allow an operator to end an idle session without deleting its history.
- **FR-012**: System MUST reject new turn submission for an ended session.
- **FR-013**: System MUST carry `session_id` and turn linkage into persisted root workflow metadata
  and workflow-scoped autonomy inspection.
- **FR-014**: System MUST keep `POST /api/v1/tasks` and `GET /api/v1/tasks/{task_id}` as valid
  one-shot compatibility surfaces that do not require session semantics.
- **FR-015**: System MUST keep the exact `provider_kind` and `model_id` used for each turn visible
  in persisted records and operator inspection output.

### Key Entities *(include if feature involves data)*

- **ConversationSession**: The durable session envelope that owns a stable `session_id`, a stable
  `coordinator_agent_id`, retained session context, lifecycle state, and the active or last
  workflow linkage.
- **SessionTurn**: One ordered user turn inside a session. It links a user message, turn index,
  root `workflow_id`, turn status, and terminal result summary.
- **SessionStatusView**: The operator-facing inspect view that combines durable session data with
  the current or last workflow linkage.
- **WorkflowSessionLink**: The persisted mapping carried in root workflow metadata and autonomy
  views so each workflow can be traced back to its owning session and turn index.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can create one session and append at least one follow-up turn where both
  turns share the same `session_id` and `coordinator_agent_id`, while each turn has a distinct
  root `workflow_id`.
- **SC-002**: Inspecting a two-turn session returns ordered turn summaries plus the active or last
  workflow linkage without requiring direct database access.
- **SC-003**: Submitting a new turn to a busy or ended session is rejected deterministically with a
  conflict-class response and no duplicate workflow creation.
- **SC-004**: After runtime restart, an existing idle session can still be inspected and continued
  without manual repair of persistence state.
- **SC-005**: Workflow-scoped autonomy inspection exposes enough session linkage to correlate a
  turn workflow back to its owning session and turn index.

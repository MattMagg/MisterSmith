# Feature Specification: Chat-First CLI Loop

**Feature Branch**: `031-chat-first-cli-loop`
**Created**: 2026-04-06
**Status**: Draft
**Input**: `docs/current-state.md`, `docs/direction.md`,
`docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`,
`docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`,
`specs/029-session-first-user-shell/spec.md`, `specs/030-session-first-cli-shell/spec.md`, the
current CLI session-shell seams in `crates/mister-smith-app/src/main.rs` and
`crates/mister-smith-app/src/conversation.rs`, the current session-facing views in
`crates/mister-smith-http/src/server.rs`, and the user directive to define the next CLI-first
frontier slice without widening into implementation, task generation, or repo workflow execution

## Current Truth & Scope

Current repo truth already includes product foundations this feature must extend rather than
replace:

- Mister Smith already has a recent-first CLI startup home, recent-session browsing, direct
  resume flows, durable session identity, retained history, stored session control posture, and
  in-session steering controls on `main`.
- The current CLI can already open or resume a durable session and accept follow-up turns while
  preserving the same session identity and runtime-truth support notices.
- Runtime-truth, proof-boundary, step-policy, and coordinator-runtime evidence are already part of
  current product truth and must remain honestly surfaced when the session loop becomes more live
  and conversation-shaped.

The remaining gap is narrower than a broad new platform program:

- The active CLI session still reads more like task submission plus session inspection than like a
  live coding-agent conversation that the user naturally stays inside.
- Follow-up turns, busy states, completion states, and failure states do not yet feel like one
  continuous chat-first loop with inline conversation feedback.

This packet or feature therefore freezes one bounded slice:

1. turn the active CLI session into one live conversation loop for both new and resumed work
2. keep follow-up turns, inline turn-state feedback, and retained continuity inside that same
   loop instead of bouncing the user into detached inspection behavior
3. preserve in-session steering, durable session truth, runtime-truth honesty, and supervised
   autonomy while keeping runtime and admin machinery as support surfaces

This is not:

- GUI parity or a shared CLI and GUI shell contract
- Linear, Symphony, Ralph, SpecKit, or any other repo-workflow integration
- broad runtime redesign, new coordination architecture, or generic imitation of existing agent
  tools

## Assumptions & Defaults

- This packet is CLI-only. It may influence later GUI work, but it does not define GUI behavior or
  parity requirements.
- The existing durable session model remains the one source of truth for session identity,
  retained history, active or last-work linkage, and stored control posture.
- The packet begins after the user has entered or resumed a session. Startup-home behavior from
  packet `030` remains foundational context, not the main scope of this slice.
- Inline conversation feedback means the user can understand accepted, active, completed, failed,
  and blocked turn states from inside the live session without depending on detached inspection
  output.
- Runtime, proof, auth, config, and support commands remain reachable, but they stay secondary to
  the live conversation loop.

## User Scenarios & Testing

Use independently testable stories. For Mister Smith packets, prefer a small number of bounded
stories over a long backlog of loosely related asks.

### User Story 1 - Stay Inside One Live Conversation (Priority: P1)

A user starts or resumes a CLI session and can keep talking naturally inside one live coding-agent
conversation instead of feeling like each prompt is being submitted into a detached workflow view.

**Independent Test**: Open or resume a CLI session, send multiple follow-up turns, and confirm the
user remains inside the same active conversation loop while turn state changes are surfaced inline.

**Acceptance Scenarios**:

1. **Given** an active CLI session is open, **When** the user sends a follow-up turn, **Then** the
   shell keeps the user inside the same session conversation and shows that turn's accepted,
   active, completed, or failed state inline.
2. **Given** a turn is still in progress, **When** the user remains in the live session,
   **Then** the session view explains what is happening without requiring a separate inspection
   path just to understand the current turn state.
3. **Given** a turn fails or stops early, **When** the shell reports the outcome, **Then** the
   failure appears inline with honest next-step guidance while preserving the same conversation
   context for follow-up.

### User Story 2 - Resume Retained Work Back Into The Loop (Priority: P1)

A returning user can reopen retained work and land directly back inside the same conversation
context instead of reopening prior work as a static inspection artifact.

**Independent Test**: Create retained sessions, resume the most recent session and a selected prior
session, and confirm each one reopens as a usable live conversation with preserved continuity.

**Acceptance Scenarios**:

1. **Given** a retained session exists, **When** the user resumes the last session or opens a
   specific session, **Then** the shell re-enters that session as a live conversation that is ready
   for follow-up.
2. **Given** a resumed session already has stored control posture or support notices, **When** the
   session opens, **Then** that context remains visible inline without breaking conversation
   continuity.
3. **Given** the runtime is unavailable during resume, **When** the session opens from durable
   storage, **Then** the shell still shows retained context and honest limitation wording instead
   of hiding history or pretending the session is currently live.

### User Story 3 - Steer And Supervise The Session In Place (Priority: P1)

A user can steer model, permissions, config, status, and MCP posture from inside the live session
while still understanding busy, degraded, and proof-limited states without being pushed into an
admin-first workflow.

**Independent Test**: Open an active session, adjust the in-session controls, and confirm the same
session identity, retained context, and support truth remain visible inside the live conversation.

**Acceptance Scenarios**:

1. **Given** an active CLI session, **When** the user changes model, permissions, config, status
   view, or MCP posture, **Then** the change is reflected inside the same live conversation loop
   without forcing the user into a separate primary workflow.
2. **Given** the session is busy, degraded, or blocked, **When** the user tries another action,
   **Then** the shell explains the current session state inline and preserves the conversation and
   control context.
3. **Given** runtime-truth or proof-boundary limits apply, **When** the shell presents session
   state, **Then** it keeps those limits visible in user language without overstating live proof or
   hiding the supporting surfaces.

## Edge Cases

- a first-time user starts a brand-new session and should enter the live loop immediately even
  without prior retained history
- a resumed session is already busy with active work when the user reopens it
- the runtime is unavailable during resume or follow-up, but retained context is still readable
- the user attempts another follow-up turn while the previous turn is still active
- inline warnings and notices must stay visible without burying the conversation or next-step
  actions
- support-state degradation must not create a second session identity or hide retained control
  posture
- deterministic versus live-proof boundaries must stay explicit whenever the session view reflects
  retained or degraded state

## Requirements

### Functional Requirements

- **FR-001**: Mister Smith MUST treat an opened or resumed CLI session as one continuous
  conversation loop rather than a submit-and-inspect handoff.
- **FR-002**: The CLI MUST keep the user inside the same active session context when they send
  follow-up turns and MUST surface accepted, active, completed, failed, or blocked turn state
  inline in that session experience.
- **FR-003**: The feature MUST preserve the current durable session identity, retained history,
  stored control posture, and session continuity already landed on `main`.
- **FR-004**: The CLI MUST reopen retained sessions directly into a conversation-first view that is
  ready for follow-up without requiring a separate inspection path to regain context.
- **FR-005**: The feature MUST keep steering controls for model, permissions, config, status, and
  MCP inside the live session loop.
- **FR-006**: The CLI MUST fail explicitly and honestly when a session cannot continue live work
  because the runtime is unavailable, the session is busy, or the session has ended, while still
  exposing retained context whenever that context is available.
- **FR-007**: The feature MUST keep the write set bounded to the CLI session loop and MUST NOT
  widen into GUI parity, repo-workflow tooling, or broad runtime redesign.
- **FR-008**: The feature MUST record deterministic versus live-proof boundaries honestly whenever
  the session loop shows retained state, degraded runtime availability, or proof-limited outcomes.
- **FR-009**: The feature MUST keep runtime, proof, auth, config, and other support surfaces
  secondary to the live conversation instead of making them the main way to understand an active
  session.
- **FR-010**: The feature MUST use user-facing language centered on session continuity, status,
  steering, and next action rather than internal workflow-control jargon.

### Key Entities

- **Live Conversation Loop**: The active CLI session experience where the user stays in one
  conversation while sending follow-up turns and reading inline turn state.
- **Inline Turn State**: The user-visible accepted, active, completed, failed, or blocked status
  for the current turn inside the live conversation.
- **Retained Session Context**: The durable session identity, retained history, and stored control
  posture that make resumed work feel like continuation rather than re-entry from scratch.
- **Session Steering Controls**: The in-session controls for model, permissions, config, status,
  and MCP posture that remain part of the conversation loop.
- **Session Truth Notice**: The inline explanation of degraded runtime availability, busy state, or
  proof-boundary limits that keeps the product honest without collapsing the conversation.

## Success Criteria

- **SC-001**: A user can send at least two successive follow-up turns in one CLI session without
  leaving the active conversation loop or depending on a separate inspection command to keep
  context.
- **SC-002**: A returning user can reopen the most recent session or a selected retained session
  and regain usable conversation context in no more than 1 action after choosing that session.
- **SC-003**: When a turn is accepted, active, completed, failed, or blocked, the user can
  understand the current state from the live session view without raw log archaeology.
- **SC-004**: A user can adjust model, permissions, config, status, and MCP posture from inside an
  active session while preserving the same session identity and retained context.
- **SC-005**: Runtime-unavailable or proof-limited states remain visible and honest while retained
  history stays accessible.
- **SC-006**: The resulting spec remains clearly bounded to the CLI session loop and does not
  introduce GUI parity, repo workflow, or broad runtime redesign as part of the slice.

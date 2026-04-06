# Feature Specification: Chat-First CLI Loop

**Feature Branch**: `031-chat-first-cli-loop`
**Created**: 2026-04-06
**Status**: Frozen
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

## Observable Loop Contract

The packet is implementation-ready only if the CLI behavior is observable in user terms rather than
described as a vague "chat-first" aspiration.

- The first render for a new or resumed session shows one bounded live-session surface: retained
  transcript context or a bounded summary, the current control posture, any current truth notices,
  and the next allowed action from the same session identity.
- Sending a follow-up turn does not bounce the user into a detached inspection workflow. The same
  session loop acknowledges the turn inline, keeps prior context visible, and updates the current
  turn state in place until the turn finishes, fails, or is blocked.
- Detached inspection or status-heavy commands may still exist as secondary support surfaces, but
  the user must not need them just to understand what the active session is doing right now.
- When live execution is unavailable, the loop still renders retained context and stored controls
  inline, but it uses plain user-facing wording that says live work cannot continue yet.

## User-Visible State Distinctions

### Inline Turn States

| State | User-visible meaning | Required inline behavior |
| ----- | -------------------- | ------------------------ |
| `accepted` | The loop accepted the new turn and kept it inside the current session. | Show the turn in context immediately and identify it as the active turn in progress. |
| `running` | Live work is currently happening for the accepted turn. | Keep the same session open, show that work is underway, and avoid requiring a detached inspect path. |
| `completed` | The turn finished and produced a bounded result or summary. | Show the outcome inline, preserve the transcript, and keep the loop ready for follow-up. |
| `failed` | The turn ended unsuccessfully but the session context still exists. | Show the failure inline with honest next-step guidance and preserve follow-up continuity. |
| `blocked` | The turn could not continue under the current session or runtime posture. | Explain the blocking condition inline and keep the next allowed action visible from the same loop. |

### Session Truth Notices

| Notice | Meaning | Required distinction |
| ------ | ------- | -------------------- |
| `busy` | Another live turn is already active for the session. | Do not imply a second turn was accepted; explain that the user is waiting on current work. |
| `degraded` | Retained session context is readable, but live runtime work is not currently available. | Keep transcript and controls visible while saying live work cannot continue yet. |
| `ended` | The session is logically closed and cannot accept another live turn. | Keep retained context visible, but direct the user toward starting or resuming a different session. |
| `proof_limited` | The loop can show bounded state, not a new live-proof claim. | Keep proof boundaries explicit in user language whenever previews or retained state are shown. |

## User Scenarios & Testing

Use independently testable stories. For Mister Smith packets, prefer a small number of bounded
stories over a long backlog of loosely related asks.

### User Story 1 - Stay Inside One Live Conversation (Priority: P1)

A user starts or resumes a CLI session and can keep talking naturally inside one live coding-agent
conversation instead of feeling like each prompt is being submitted into a detached workflow view.

**Independent Test**: Start a new session or resume an existing one, send multiple follow-up
turns, and confirm the user remains inside the same active conversation loop while turn state
changes are surfaced inline without using a detached inspect-only command.

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
4. **Given** a user starts a brand-new session, **When** they send the first follow-up turn,
   **Then** the CLI enters the same live conversation loop shape used for resumed work instead of
   falling back to a submit-and-inspect path.

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
   continuity, and the stored controls remain readable before the user sends another live turn.
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
   **Then** the shell explains whether the state is busy, degraded, blocked, or ended with
   distinct inline guidance and preserves the conversation and control context.
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
- busy, degraded, ended, and proof-limited states must remain distinct instead of collapsing into
  one generic fallback notice
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
- **FR-011**: The first render of a new or resumed live session MUST show retained conversation
  context, current control posture, any active truth notice, and the next allowed action from the
  same session identity.
- **FR-012**: The CLI MUST define inline turn-state behavior for `accepted`, `running`,
  `completed`, `failed`, and `blocked` turns in user-visible terms, and each state MUST be
  understandable from inside the loop itself.
- **FR-013**: `busy`, `degraded`, `ended`, and `proof-limited` conditions MUST remain distinct
  user-visible session states with different inline guidance rather than one generic notice flow.
- **FR-014**: Detached inspection views or status-heavy commands MAY remain available, but the user
  MUST NOT depend on them to understand the active session's current turn state or next action.
- **FR-015**: When a retained session is reopened while live runtime work is unavailable, the loop
  MUST still show retained transcript context and stored control posture before any new live action
  is attempted.

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
  leaving the active conversation loop and without using a detached inspect-only command between
  those turns to understand current progress.
- **SC-002**: A returning user can reopen the most recent session or a selected retained session
  and regain usable conversation context, retained controls, and the next allowed action in no
  more than 1 action after choosing that session.
- **SC-003**: When a turn is accepted, active, completed, failed, or blocked, the user can
  understand the current state from the live session view itself without raw log archaeology or a
  separate inspect-only flow.
- **SC-004**: A user can adjust model, permissions, config, status, and MCP posture from inside an
  active session while preserving the same session identity and retained context.
- **SC-005**: Runtime-unavailable or proof-limited states remain visible and honest while retained
  history stays accessible.
- **SC-006**: Busy, degraded, ended, and proof-limited states are distinguishable from one another
  by their inline wording and next-step guidance.
- **SC-007**: The resulting spec remains clearly bounded to the CLI session loop and does not
  introduce GUI parity, repo workflow, or broad runtime redesign as part of the slice.

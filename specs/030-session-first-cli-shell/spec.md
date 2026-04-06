# Feature Specification: Session-First CLI Shell

**Feature Branch**: `030-session-first-cli-shell`
**Created**: 2026-04-05
**Status**: Frozen
**Input**: `docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`,
`docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`, `docs/current-state.md`, the
current CLI seams in `crates/mister-smith-app/src/main.rs` and
`crates/mister-smith-app/src/conversation.rs`, the current session API seams in
`crates/mister-smith-http/src/server.rs`, and the user directive to create a CLI-only packet
while leaving the existing shared shell packet unchanged

## Current Truth & Scope

Current repo truth already includes product seams this feature must reshape rather than replace:

- `mister-smith` currently defaults to the runtime boot path when launched with no subcommand, so
  the main CLI entry still feels runtime-first instead of session-first.
- Durable session behavior already exists. The product can already create, continue, inspect, end,
  and list retained sessions with stable `session_id`, stable `coordinator_agent_id`, ordered turn
  history, provider and model attribution, and recent-session preview data.
- The current HTTP session views already expose the session summary and detail data the CLI can
  build on for recent-session and resume behavior.
- Runtime, auth, autonomy, doctor, proof, and MCP-adjacent surfaces already exist as support
  capabilities, but they still sit too close to the center of the CLI story.

The remaining gap is narrower than a broad new platform program:

- Mister Smith does not yet open into one clear session-first CLI shell that teaches users to
  start, resume, browse, and steer work in place.
- Resume and recent-session behavior are not yet first-class top-level CLI flows.
- Live session controls are not yet organized around one simple in-shell experience.

This feature therefore freezes one bounded slice:

1. make opening Mister Smith lead to a recent-first CLI shell home instead of a runtime-first
   entry
2. make start, resume, browse recent sessions, and live-session steering the primary CLI product
   path
3. keep the current durable session system as the one CLI source of truth while leaving support
   commands available but secondary

This is not:

- a shared CLI and GUI contract
- cross-surface continuity or desktop parity work
- Linear, Symphony, Ralph, SpecKit, or any other repo-workflow integration
- a generic admin console, plugin marketplace, or broad runtime redesign

## Assumptions & Defaults

- The startup home is recent-first. It shows recent sessions, resume-last, start-new, warnings,
  and open-config. It does not expand into pinned sessions, recent workspaces, or quick-start
  prompts in this slice.
- `session` is the primary user-facing noun. Existing internal terms such as `conversation` or
  `autonomy` may remain in technical seams, but they are not first-level CLI navigation.
- The CLI exposes live controls through slash commands or another clearly in-session command flow
  rather than sending users back to runtime-first maintenance commands.
- `status` and `config` take effect in the CLI shell immediately for the open session.
- `model`, `permissions`, and `MCP` changes are retained with the session shell in this packet so
  users can inspect and resume the same control posture honestly, even when runtime execution
  still follows the active runtime path.
- Shared session truth builds on the durable session seams already landed on current `main`; this
  feature does not introduce a second session store or a second retained history model.
- Runtime, doctor, auth, proof, config, and MCP administration remain support surfaces beside the
  main CLI session flow rather than the default path. This packet does not redesign their command
  tables.

## User Scenarios & Testing

### User Story 1 - Open The CLI And Start Work (Priority: P1)

A user opens Mister Smith in the terminal and lands in a recent-first CLI shell that makes it
obvious how to start a new session immediately without thinking about runtime boot commands.

**Independent Test**: Launch Mister Smith with no arguments and confirm the user sees a recent-first
CLI home, can start a new session from that home, and can begin typing without using runtime-first
commands.

**Acceptance Scenarios**:

1. **Given** a user opens Mister Smith with no arguments, **When** the CLI shell starts,
   **Then** the user sees a startup home that centers recent sessions, start-new, resume-last,
   warnings, and open-config instead of runtime-first navigation.
2. **Given** a user has no prior sessions, **When** the startup home opens, **Then** the user can
   still start a new session immediately and is not blocked by the absence of session history.
3. **Given** startup warnings exist, **When** the user opens the CLI shell, **Then** the warnings
   are visible inline without preventing the user from starting or browsing sessions.

### User Story 2 - Resume And Browse Recent CLI Sessions (Priority: P1)

A returning user can resume the last session, resume a specific session, or browse recent sessions
from the CLI without losing the simple session-first product model.

**Independent Test**: Create several retained sessions, reopen Mister Smith in the terminal, and
confirm the user can resume the last session, choose a specific recent session, and understand the
difference between quick resume and broader session browsing.

**Acceptance Scenarios**:

1. **Given** a user has a recent session, **When** they choose resume-last, **Then** Mister Smith
   reopens that session directly without sending the user through runtime or admin flows first.
2. **Given** a user wants a different prior session, **When** they browse recent sessions,
   **Then** they can identify and reopen the right session from a recent-session view that shows
   enough summary information to make the choice.
3. **Given** the backing runtime is unavailable, **When** the user opens the CLI shell,
   **Then** they can still see recent sessions and the reason live work cannot continue instead of
   losing access to retained session history.

### User Story 3 - Steer A Live CLI Session In Place (Priority: P1)

A user can stay inside one live CLI session and change core session controls in place without
leaving the session for a separate admin-first workflow.

**Independent Test**: Start a session in the CLI, adjust the core controls while it is live, and
confirm the same session identity, retained history, and control state remain intact.

**Acceptance Scenarios**:

1. **Given** a live session is open, **When** the user changes model, permissions, config, status
   view, or MCP posture in-session, **Then** the change happens without forcing the user to leave
   that session for a separate primary workflow.
2. **Given** a selected session is already active or busy, **When** the user tries to reopen or
   continue it from the CLI, **Then** the shell handles that state honestly instead of showing a
   conflicting or misleading session state.
3. **Given** support state is degraded, **When** the user stays in the live CLI session,
   **Then** the shell explains the limitation while preserving the session identity and retained
   history.

## Edge Cases

- a first-time user has no retained sessions, but the startup home still needs to make the new
  session path obvious
- the runtime is unavailable at startup, but recent sessions and support warnings still need to be
  visible
- a selected session is already busy with active work when the user tries to reopen or continue it
- startup warnings must remain visible without burying the main session actions
- support commands remain reachable, but they must not take over the default product navigation
- degraded support state may block a live action, but it must not erase visible recent-session
  history or hide the reason for the block

## Requirements

### Functional Requirements

- **FR-001**: Mister Smith MUST treat opening the CLI shell as the primary product entry and MUST
  make session start or resume the first thing the user can do.
- **FR-002**: Launching `mister-smith` with no arguments MUST open a recent-first CLI shell home
  instead of defaulting to a runtime-first path.
- **FR-003**: The CLI startup home MUST show recent sessions, a clear start-new action, a clear
  resume-last action, visible startup warnings, and a direct path to config.
- **FR-004**: The product MUST support first-class CLI resume behavior for the most recent session
  and for a user-selected prior session.
- **FR-005**: The product MUST provide a CLI recent-session browsing flow that is distinct from
  quick resume and makes it clear the user is choosing from retained session history.
- **FR-006**: The CLI MUST use the current durable session identity, session storage, and retained
  transcript or summary history as the one source of truth for retained sessions.
- **FR-007**: The CLI MUST preserve the same stable session identity and the same retained
  transcript or summary history when a user resumes or reopens prior work.
- **FR-008**: The product MUST allow core live-session steering in place for model, permissions,
  config, status, and MCP without forcing the user into a separate admin-first workflow. In this
  packet, `status` and `config` act immediately in the CLI shell, while `model`, `permissions`,
  and `MCP` persist as session-shell preferences and remain visible with honest warnings.
- **FR-009**: The core in-session controls in this slice MUST be available through the CLI shell
  itself, including slash commands or another clearly in-session command flow for the same control
  set.
- **FR-010**: The product MUST present startup status and warnings honestly, including runtime
  unavailability, without hiding recent sessions or the main session actions.
- **FR-011**: The recent-session and resume flows MUST work even when support surfaces report
  degraded state, unless the user is attempting an action that truly cannot proceed.
- **FR-012**: The product MUST keep runtime, doctor, auth, proof, config, and MCP administration
  as support surfaces beside the main session flow rather than the main default CLI path, without
  requiring packet 030 to redesign those support command tables.
- **FR-013**: The product MUST use simple user-facing CLI language centered on session, resume,
  config, model, runtime, and MCP rather than leading with internal backend terms.
- **FR-014**: The feature MUST NOT widen into GUI parity, cross-surface continuity, repo-workflow
  tooling, generic admin-console positioning, or broad runtime redesign.

### Key Entities

- **Session**: The durable unit of user work that can be started, resumed, inspected, and
  continued through the CLI shell.
- **Session Summary**: The compact recent-session record shown in startup and browse flows so a
  user can recognize and reopen prior work.
- **Session Transcript**: The retained history or summary history that gives a resumed session its
  continuity in the CLI shell.
- **Session Control State**: The current in-session settings and status the user can steer in
  place, including model, permissions, config posture, status view, and MCP posture.
- **Startup Home View**: The recent-first CLI entry that introduces the product through start,
  resume, recent sessions, warnings, and config instead of runtime-first controls.
- **Support Status Notice**: The inline warning or degraded-state message that keeps support truth
  visible without replacing the session-first CLI path.

## Success Criteria

- **SC-001**: A new user can open Mister Smith and begin a new session from the CLI startup home
  in no more than 2 actions.
- **SC-002**: A returning user can reopen the most recent session in no more than 1 selection from
  the CLI startup home.
- **SC-003**: A user can browse recent sessions and reopen a specific prior session in no more than
  3 actions without needing runtime-first or admin-first commands.
- **SC-004**: A user can change model, permissions, config, status view, and MCP posture from
  inside an active CLI session without leaving that session for a separate primary workflow.
- **SC-005**: Startup warnings remain visible whenever support state is degraded, while the user
  can still reach recent-session and start-new actions unless the attempted action is truly
  blocked.
- **SC-006**: The resulting spec remains clearly bounded to the session-first CLI shell and does
  not introduce GUI parity, repo-workflow, or admin-console scope as part of the main product
  path.

# Feature Specification: Session-First User Shell

**Feature Branch**: `029-session-first-user-shell`
**Created**: 2026-04-05
**Status**: Draft
**Input**: `docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`,
`docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`, `docs/current-state.md`,
the current CLI seams in `crates/mister-smith-app/src/main.rs` and
`crates/mister-smith-app/src/conversation.rs`, the current session API seams in
`crates/mister-smith-http/src/server.rs`, and the current desktop surface under
`apps/operator-console/`

## Current Truth & Scope

Current repo truth already includes product seams this feature must reshape rather than replace:

- `mister-smith` currently defaults to the runtime boot path when launched with no subcommand, so
  the main entry still feels runtime-first instead of session-first.
- Durable session behavior already exists. The product can already create, continue, inspect, end,
  and list retained sessions with stable `session_id`, stable `coordinator_agent_id`, ordered turn
  history, provider and model attribution, and recent-session preview data.
- The current HTTP session views already expose the session summary and detail data a user shell
  can build on, including active or last workflow linkage and restart-resume lineage.
- The current desktop app already proves there is a local GUI surface for session and runtime
  state, but it is still shaped as an operator console rather than the target user shell.

The remaining gap is narrower than a broad new platform program:

- Mister Smith does not yet open into one clear session-first shell that teaches users to start,
  resume, and steer work in place.
- CLI and GUI do not yet present one shared user-facing session model with matching entry,
  resume, and control behavior.
- Runtime, doctor, auth, proof, and MCP administration are still too close to the center of the
  product story.

This feature therefore freezes one bounded slice:

1. make opening Mister Smith lead to a recent-first session home instead of a runtime-first entry
2. make start, resume, browse recent sessions, and live-session steering the primary product path
   in both CLI and GUI
3. define one shared session system, one shared app protocol, and one shared control model across
   both front ends

This is not:

- a redesign of the underlying runtime architecture
- a generic admin console or operator dashboard expansion
- Linear, Symphony, Ralph, SpecKit, or any other repo-workflow integration
- plugin marketplace, IDE bridge, voice-first UX, or unrelated framework parity work

## Assumptions & Defaults

- The startup home is recent-first. It shows recent sessions, resume-last, start-new, warnings,
  and open-config. It does not expand into pinned sessions, recent workspaces, or quick-start
  prompts in this slice.
- `session` is the primary user-facing noun. Existing internal terms such as `conversation` or
  `autonomy` may remain in technical seams, but they are not first-level product navigation.
- CLI slash commands and GUI in-session controls must reach feature parity for the core controls in
  this slice. The GUI may use a command palette or another equivalent in-session control surface.
- Shared session truth builds on the durable session seams already landed on current `main`; this
  feature does not introduce a second session model or a second source of truth.
- Runtime, doctor, auth, proof, and MCP administration remain available, but they are support
  surfaces beside the main session flow rather than the main product path.

## User Scenarios & Testing

### User Story 1 - Open The Shell And Start Work (Priority: P1)

A user opens Mister Smith and lands in a recent-first shell home that makes it obvious how to
start a new session immediately without thinking about runtime boot commands.

**Independent Test**: Launch Mister Smith with no arguments in either front end and confirm the
user sees a recent-first home, can start a new session from that home, and can begin typing
without using runtime-first commands.

**Acceptance Scenarios**:

1. **Given** a user opens Mister Smith with no arguments, **When** the shell starts, **Then** the
   user sees a startup home that centers recent sessions, start-new, resume-last, warnings, and
   open-config instead of runtime-first navigation.
2. **Given** a user has no prior sessions, **When** the startup home opens, **Then** the user can
   still start a new session immediately and is not blocked by the absence of session history.
3. **Given** startup warnings exist, **When** the user opens the shell, **Then** the warnings are
   visible inline without preventing the user from starting or browsing sessions.

### User Story 2 - Resume And Browse Recent Sessions (Priority: P1)

A returning user can resume the last session, resume a specific session, or browse recent sessions
from either front end without losing the simple session-first product model.

**Independent Test**: Create several retained sessions, reopen Mister Smith in both front ends,
and confirm the user can resume the last session, choose a specific recent session, and understand
the difference between quick resume and broader session browsing.

**Acceptance Scenarios**:

1. **Given** a user has a recent session, **When** they choose resume-last, **Then** Mister Smith
   reopens that session directly without sending the user through runtime or admin flows first.
2. **Given** a user wants a different prior session, **When** they browse recent sessions,
   **Then** they can identify and reopen the right session from a recent-session view that shows
   enough summary information to make the choice.
3. **Given** the backing runtime is unavailable, **When** the user opens the shell, **Then** they
   can still see recent sessions and the reason work cannot continue, instead of losing access to
   the product's session history.

### User Story 3 - Steer A Live Session Across CLI And GUI (Priority: P1)

A user can stay inside one live session, change core session controls in place, and move between
CLI and GUI without losing the same session identity or history.

**Independent Test**: Start a session in one front end, adjust core controls while it is live,
open the same session in the other front end, and confirm the same session identity, transcript,
and control state are preserved.

**Acceptance Scenarios**:

1. **Given** a live session is open, **When** the user changes model, permissions, config, status
   view, or MCP posture in-session, **Then** the change happens without forcing the user to leave
   that session for a separate admin surface.
2. **Given** a session started in the CLI, **When** the user opens the same session in the GUI,
   **Then** the GUI continues the same session rather than creating a second copy or a second
   session model.
3. **Given** a session is already active in one front end, **When** the user opens it in the
   other front end, **Then** the product preserves one shared session state and handles the live
   session honestly instead of showing conflicting state.

## Edge Cases

- a first-time user has no retained sessions, but the startup home still needs to make the new
  session path obvious
- the runtime is unavailable at startup, but recent sessions and support warnings still need to be
  visible
- a selected session is already busy with active work when the user tries to reopen or continue it
- startup warnings must remain visible without burying the main session actions
- CLI and GUI open the same session close together and must still reflect one shared session state
- a session can be browsed from history even when support surfaces such as doctor or MCP admin
  report warnings
- support commands remain reachable, but they must not take over the default product navigation

## Requirements

### Functional Requirements

- **FR-001**: Mister Smith MUST treat opening the shell as the primary product entry and MUST make
  session start or resume the first thing the user can do.
- **FR-002**: Launching `mister-smith` with no arguments MUST open a recent-first session home
  instead of defaulting to a runtime-first path.
- **FR-003**: The startup home MUST show recent sessions, a clear start-new action, a clear
  resume-last action, visible startup warnings, and a direct path to config.
- **FR-004**: The product MUST support first-class resume behavior for the most recent session and
  for a user-selected prior session.
- **FR-005**: The product MUST provide a recent-session browsing flow that is distinct from quick
  resume and makes it clear the user is choosing from retained session history.
- **FR-006**: CLI and GUI MUST use one shared session identity model, one shared session storage
  model, and one shared transcript model for the same retained sessions.
- **FR-007**: CLI and GUI MUST use one shared app protocol between the front ends and the backing
  session or runtime system so the same session can move between front ends without losing state.
- **FR-008**: The product MUST preserve the same stable session identity and the same retained
  transcript or summary history when a user moves between CLI and GUI.
- **FR-009**: The product MUST allow core live-session steering in place for model, permissions,
  config, status, and MCP without forcing the user into a separate admin-first workflow.
- **FR-010**: The core in-session controls in this slice MUST be available in both front ends,
  with CLI slash commands and GUI command-palette or equivalent parity for the same control set.
- **FR-011**: The product MUST present startup status and warnings honestly, including runtime
  unavailability, without hiding recent sessions or the main session actions.
- **FR-012**: The recent-session and resume flows MUST work even when support surfaces report
  degraded state, unless the user is attempting an action that truly cannot proceed.
- **FR-013**: The product MUST keep runtime, doctor, auth, proof, config, and MCP administration
  as support surfaces beside the main session flow rather than the main default navigation path.
- **FR-014**: The product MUST preserve the currently landed durable session seams as the source of
  truth for session identity, history, and active or last-work linkage rather than inventing a
  second session store.
- **FR-015**: The product MUST use simple user-facing navigation language centered on session,
  resume, config, model, runtime, and MCP rather than leading with internal backend terms.
- **FR-016**: The feature MUST NOT widen into repo-workflow tooling, generic admin-console
  positioning, broad runtime redesign, or unrelated product programs outside the shared user shell.

### Key Entities

- **Session**: The durable unit of user work that can be started, resumed, inspected, and
  continued across both front ends.
- **Session Summary**: The compact recent-session record shown in startup and browse flows so a
  user can recognize and reopen prior work.
- **Session Transcript**: The retained history or summary history that gives a resumed session its
  continuity across CLI and GUI.
- **Session Control State**: The current in-session settings and status the user can steer in
  place, including model, permissions, config posture, status view, and MCP posture.
- **Startup Home View**: The recent-first entry screen that introduces the product through start,
  resume, recent sessions, warnings, and config instead of runtime-first controls.
- **Shared App Protocol**: The common request and state contract that lets both front ends operate
  on the same session system without diverging into separate products.

## Success Criteria

- **SC-001**: A new user can open Mister Smith and begin a new session from the startup home in no
  more than 2 actions.
- **SC-002**: A returning user can reopen the most recent session in no more than 1 selection from
  the startup home in either front end.
- **SC-003**: A user can browse recent sessions and reopen a specific prior session in no more than
  3 actions without needing runtime-first or admin-first commands.
- **SC-004**: A user can change model, permissions, config, status view, and MCP posture from
  inside an active session without leaving that session for a separate primary workflow.
- **SC-005**: A session started in one front end can be reopened in the other front end while
  preserving the same session identity and retained history.
- **SC-006**: Startup warnings remain visible whenever support state is degraded, while the user
  can still reach recent-session and start-new actions unless the attempted action is truly blocked.
- **SC-007**: The resulting spec remains clearly bounded to the shared session-first user shell and
  does not introduce repo-workflow or admin-console scope as part of the main product path.

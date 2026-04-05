# Contract: Session Shell Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design goal

Freeze one bounded contract for the user-facing Mister Smith shell so later implementation does
not rediscover what "session-first" means.

This packet does **not**:

- redefine the runtime substrate
- redefine session persistence ownership
- widen into repo-workflow tooling
- widen into a generic admin console or observability dashboard

## Main shell entries

The shell must prioritize these user-facing entries:

```text
mister-smith [prompt]
mister-smith resume [session_id|--last] [prompt]
mister-smith sessions list|open
mister-smith app
```

Behavior:

- `mister-smith` with no arguments opens the recent-first shell home
- `mister-smith <prompt>` starts a new interactive session directly
- `mister-smith resume --last` reopens the most recent session directly
- `mister-smith resume <session_id>` reopens a specific retained session directly
- `mister-smith sessions ...` is the broader retained-session management surface rather than the
  quick-resume path
- `mister-smith app` opens the desktop front end that uses the same shared session system
- delete and export session-management behaviors remain outside this packet's must-have command
  surface

## Startup home contract

The startup home is the default shell entry and must include:

- recent sessions
- a start-new action
- a resume-last action when recent history exists
- inline startup warnings
- a direct config action

The startup home does **not** need to include pinned sessions, recent workspaces, or quick-start
prompts in this slice.

## In-session control contract

The core in-session controls for this packet are:

- model
- permissions
- config
- status
- MCP

Behavior:

- the CLI exposes these through slash commands
- the GUI exposes the same control set through a command palette or equivalent in-session surface
- control changes happen in the live session instead of redirecting the user to an admin-first
  workflow

## Support surface contract

These support surfaces remain available:

```text
mister-smith runtime ...
mister-smith doctor
mister-smith auth ...
mister-smith proof ...
mister-smith config ...
mister-smith mcp ...
```

Behavior:

- they remain part of the product
- they must not become the default product entry
- they must not replace the startup home as the first thing users see
- their warnings remain visible in the shell when relevant

## User-facing language contract

First-level navigation language stays centered on:

- session
- resume
- config
- model
- runtime
- MCP

The shell must avoid leading with internal backend nouns when simpler product language is
available.

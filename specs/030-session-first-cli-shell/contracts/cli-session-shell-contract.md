# Contract: CLI Session Shell Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design goal

Freeze one bounded contract for the user-facing Mister Smith CLI shell so later implementation does
not rediscover what "session-first" means in the terminal.

This packet does **not**:

- define a shared CLI and GUI contract
- redefine the runtime substrate
- redefine session persistence ownership
- widen into repo-workflow tooling or a generic admin console

## Main CLI entries

The CLI must prioritize these user-facing entries:

```text
mister-smith [prompt]
mister-smith resume [session_id|--last] [prompt]
mister-smith sessions list|open
```

Behavior:

- `mister-smith` with no arguments opens the recent-first CLI shell home
- `mister-smith <prompt>` starts a new interactive session directly
- `mister-smith resume --last` reopens the most recent session directly
- `mister-smith resume <session_id>` reopens a specific retained session directly
- `mister-smith sessions ...` is the broader retained-session management surface rather than the
  quick-resume path
- delete and export session-management behaviors remain outside this packet's must-have command
  surface

## Startup home contract

The startup home is the default CLI entry and must include:

- recent sessions
- a start-new action
- a resume-last action when recent history exists
- inline startup warnings
- a direct config action

The startup home does **not** need to include pinned sessions, recent workspaces, quick-start
prompts, or GUI-launch actions in this slice.

## In-session control contract

The core in-session controls for this packet are:

- model
- permissions
- config
- status
- MCP

Behavior:

- the CLI exposes these through slash commands or another clearly in-session command flow
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
- they must not become the default CLI entry
- they must not replace the startup home as the first thing users see
- their warnings remain visible in the shell when relevant

## User-facing language contract

First-level CLI language stays centered on:

- session
- resume
- config
- model
- runtime
- MCP

The CLI must avoid leading with internal backend nouns when simpler product language is available.

# CLI Session Loop Contract

## Purpose

Define the packet-owned behavior contract for Mister Smith's chat-first CLI session loop. This
contract is limited to the CLI experience on top of the current durable session model.

## Entry surfaces

The packet depends on these existing entry surfaces and refines what happens after they land in a
session:

- `mister-smith`
- `mister-smith resume --last`
- `mister-smith resume <session_id>`
- `mister-smith sessions open <session_id>`
- plain-text follow-up turns inside the active session loop
- in-session commands such as `/new`, `/resume`, `/model`, `/permissions`, `/status`, `/config`,
  and `/mcp`

## Required loop outputs

An active CLI session loop must expose all of these together:

1. **Conversation context**
   - stable session identity
   - retained transcript entries or bounded conversation summaries
   - the current turn in context with prior turns
2. **Inline turn state**
   - accepted
   - running
   - completed
   - failed
   - blocked
3. **Session steering**
   - visible current model and provider posture
   - visible permission, config, status, and MCP posture
   - in-session control flow for changing those values
4. **Truth notices**
   - runtime unavailable
   - session busy
   - session ended
   - retained-only or proof-limited state

## State contract

| Loop State | Meaning | Allowed Next Actions |
| ---------- | ------- | -------------------- |
| `ready` | Session is open and can accept the next turn | plain-text follow-up, steering commands, `/resume`, `/new` |
| `turn_pending` | A turn was accepted but has not completed yet | read inline state, steering commands that do not break session truth |
| `turn_running` | The active workflow is in progress | read inline state, limited steering, no misleading second live turn |
| `blocked` | The turn cannot proceed normally because of busy or degraded conditions | inspect inline truth notice, steer support posture, retry only when honest |
| `degraded` | Retained session context is readable but the runtime is not currently available | resume later, inspect retained context, adjust support posture |
| `ended` | The session is logically closed | inspect retained context, start or resume a different session |

## Truth rules

- retained-only views must say so explicitly
- proof-boundary wording must remain visible whenever a retained result or current result preview
  is shown
- busy-session behavior must not imply that a second live turn was accepted if it was not
- steering commands must preserve session identity and continuity instead of kicking the user out
  to a primary admin workflow

## Deferred by this packet

- GUI parity or shared CLI and GUI shell contract work
- new runtime endpoints or a new session store
- repo workflow or external control-plane integration

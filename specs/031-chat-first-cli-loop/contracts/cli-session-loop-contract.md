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

## First loop render contract

The first visible state for a new or resumed session must keep the user inside one session loop.
That initial render must include all of the following together:

1. session identity and bounded conversation context
2. current control posture for model, permissions, config, status, and MCP
3. any active truth notice that changes what the user can do next
4. the next allowed action from the same session identity

Detached inspection output may still exist, but it is secondary support context rather than the
main active-session view.

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

## Inline turn-state contract

| Turn State | Visible meaning | Loop requirement |
| ---------- | --------------- | ---------------- |
| `accepted` | The session accepted the turn and kept it in the current conversation. | Show the new turn inline immediately and identify it as the turn now in focus. |
| `running` | Live work is happening now. | Keep the loop open, keep prior context visible, and avoid requiring a detached inspect-only path. |
| `completed` | The turn finished and produced a bounded result. | Show the outcome inline and leave the loop ready for follow-up. |
| `failed` | The turn stopped unsuccessfully. | Keep the failure inline, preserve context, and expose the next honest action. |
| `blocked` | The turn cannot continue under the current session or runtime posture. | Keep the blocking reason inline and explain what the user can do next from the same loop. |

## Truth-notice contract

| Notice Kind | Visible meaning | Required distinction |
| ----------- | --------------- | -------------------- |
| `busy` | A live turn is already active. | Do not imply a second turn was accepted. |
| `degraded` | Retained context is readable, but live runtime work is unavailable. | Keep retained context visible while saying live work cannot continue yet. |
| `ended` | The session is closed. | Preserve retained context, but route the user toward another session. |
| `proof_limited` | The loop is showing bounded or retained state, not a new live-proof claim. | Keep proof wording explicit in user language whenever previews are shown. |

## Truth rules

- retained-only views must say so explicitly
- proof-boundary wording must remain visible whenever a retained result or current result preview
  is shown
- busy-session behavior must not imply that a second live turn was accepted if it was not
- steering commands must preserve session identity and continuity instead of kicking the user out
  to a primary admin workflow
- the first render after resume must keep stored control posture and retained context visible
- detached inspection or status-heavy commands remain optional support surfaces, not required loop
  comprehension paths

## Deferred by this packet

- GUI parity or shared CLI and GUI shell contract work
- new runtime endpoints or a new session store
- repo workflow or external control-plane integration

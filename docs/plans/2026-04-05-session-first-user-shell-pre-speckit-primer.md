# 2026-04-05 Session-First User Shell Pre-SpecKit Primer

Status: active pre-SpecKit primer

## Purpose

This note expands the session-first CLI and GUI proposal into a pre-SpecKit planning artifact.

It is not a full packet spec.

It exists to freeze the product direction, the bounded problem, the likely future spec shape, and
the exact questions that the later SpecKit packet must answer.

Use this before creating a full SpecKit packet.

Related design note:

- `docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`

## What This Note Is

This note is:

- a product primer
- a packet-prep style decision note
- a bridge from idea to future SpecKit packet

This note is not:

- a final packet contract
- an implementation-ready task pack
- a claim that the feature is already frozen enough to code

## Bottom Line

Mister Smith should have one session-first user shell with two front ends:

- terminal UI
- desktop GUI

The center of the product should be running and resuming sessions.

Runtime boot, maintenance, proof, auth, and MCP management should still exist, but they should
support the session flow instead of defining the product.

## Why This Needs A Pre-SpecKit Primer

The earlier proposal answers "what direction should this go?"

That is useful, but it is still too light to jump straight into full SpecKit packet authoring.

Before SpecKit, we need one deeper note that freezes:

- the exact user problem
- the current repo truth
- the bounded feature surface
- the future packet boundary
- the questions that must be resolved during full spec work

Without this step, a future SpecKit packet is likely to drift into one of two bad shapes:

- an admin console packet with some session language added late
- a generic coding-agent shell packet that forgets Mister Smith's own runtime and operator truth

## Product Thesis

The core Mister Smith user experience should be:

- open the shell
- start or resume a session
- steer the session in place
- inspect recent sessions and current state
- move between terminal and desktop without losing the session

The main object is the session.

Not:

- the server
- the process
- the config file
- the MCP server list
- the proof script

Those are support systems around the session.

## Current Repo Truth

Current `main` already gives us important pieces of the product:

- `crates/mister-smith-app/src/main.rs`
  - current binary entry point
  - current CLI groups: `run`, `conversation`, `autonomy`, `auth`
- `crates/mister-smith-app/src/conversation.rs`
  - session creation, continuation, inspection, ending, and list support
- `crates/mister-smith-app/src/autonomy.rs`
  - workflow status and operator-facing runtime truth projection
- `crates/mister-smith-app/src/bootstrap.rs`
  - health, metrics, websocket, and API boot surface
- `crates/mister-smith-http/src/server.rs`
  - task and session request types that already define the operator-facing API seams
- `apps/operator-console/`
  - current local desktop operator surface

Repo-wide router truth also says:

- the OS runtime and operator surfaces are part of the product
- Linear, Symphony, Ralph, and SpecKit are not part of the shipped product
- the next forward product work is still around coordinator-runtime truth, stronger runtime
  surfaces, and operator clarity

That means this feature should stay product-side.

It should not become repo-workflow glue.

## Exact Problem To Solve

The current shell shape is upside down for a user-facing product.

Today the binary feels more like:

- start the runtime
- call a few support subcommands

But current market-leading agent tools teach users the opposite flow:

- open the session shell
- type immediately
- resume old work easily
- change config, model, permissions, and MCP inside the session

The residual gap for Mister Smith is:

- the product does not yet present one clear session-first shell across CLI and desktop

This gap is about product shape, not just extra commands.

## Product Decision

Freeze these decisions before SpecKit:

### D1: Interactive session shell is the main entry

`mister-smith` with no arguments should open the interactive shell.

### D2: Resume is first-class

Resuming work should be as central as starting work.

### D3: CLI and GUI share the same session system

The desktop app and terminal shell should use one shared session engine and one shared storage
model.

### D4: Session steering should happen inside the shell

Model, permissions, config, status, and MCP inspection should be available in-session through slash
commands or an in-session command palette.

### D5: Maintenance is secondary

Runtime, doctor, proof, auth, and MCP admin commands remain important, but they sit beside the
main product path instead of being the main path.

### D6: User-facing nouns should stay simple

Use simple top-level language:

- session
- resume
- config
- model
- runtime
- mcp

Do not make internal vocabulary the main navigation language when simpler user language is better.

## Proposed Future Feature Shape

This note does not lock a final packet number.

It does freeze one likely future feature theme:

- working feature name: `session-first-user-shell`

That future feature would likely cover:

- default interactive shell entry
- recent sessions and resume flows
- shared CLI/GUI session storage
- in-session slash commands or command palette
- startup home view
- bottom status rail
- support surfaces for runtime, doctor, auth, proof, and MCP

## Future User Flows That The Packet Must Cover

The later SpecKit packet should cover at least these flows.

### Flow 1: start a new session

The user opens `mister-smith` and types immediately.

### Flow 2: resume the last session

The user opens `mister-smith resume --last` or uses an in-shell picker.

### Flow 3: browse recent sessions

The user sees recent sessions before typing and can jump back into one.

### Flow 4: steer the live session

The user changes model, permission mode, config, or MCP posture without leaving the shell.

### Flow 5: move between CLI and GUI

The user starts in the terminal and later opens the desktop app to continue the same session
history.

### Flow 6: inspect system state without leaving the product

The user can see status, warnings, config, and runtime health from the live shell.

## Proposed Future Command Shape

This remains provisional until SpecKit, but this primer freezes the intended direction:

```text
mister-smith [prompt]
mister-smith resume [session_id|--last] [prompt]
mister-smith sessions list|open|delete|export
mister-smith app

mister-smith auth ...
mister-smith mcp ...
mister-smith config ...
mister-smith doctor
mister-smith runtime ...
mister-smith proof ...
```

The important point is not the exact flag spelling yet.

The important point is priority:

1. session entry
2. session resume and management
3. in-session steering
4. maintenance and support commands

## Proposed Future In-Session Controls

This note freezes the need for in-session controls, not the final command set.

Minimum expected controls:

- `/new`
- `/resume`
- `/sessions`
- `/model`
- `/permissions`
- `/status`
- `/config`
- `/mcp`
- `/doctor`
- `/quit`

The later SpecKit packet should decide:

- slash commands only
- slash commands plus command palette
- terminal and GUI parity rules

## Boundaries

The future packet should stay inside these boundaries.

### In scope

- CLI shell UX
- desktop GUI alignment
- shared session storage and identity
- startup and resume behavior
- in-session steering controls
- status and warning presentation
- support command placement and naming

### Out of scope

- Linear or Symphony workflow control
- Ralph or SpecKit command embedding inside the user shell
- plugin marketplace work
- IDE bridge work
- voice-first UX
- generic framework parity work that is not needed for the session-first shell
- broad runtime redesign outside the user-shell seams

## Current Risks

These are the main ways the later packet could go wrong.

### Risk 1: admin-tool drift

The packet could over-focus on runtime control and bury the session flow again.

### Risk 2: generic coding-agent drift

The packet could copy Codex, Claude Code, or Gemini too literally and stop being grounded in
Mister Smith's own runtime and operator seams.

### Risk 3: split-brain CLI and GUI

The terminal shell and desktop app could end up with different session models or different state
stores.

### Risk 4: naming drift

The packet could expose too much internal naming at the top level instead of using simple user
language.

### Risk 5: scope creep

The packet could widen into runtime architecture redesign, external interoperability work, or
repo-workflow surfaces.

## Questions The Full Spec Must Answer

These questions should be answered in the later SpecKit packet.

### Product questions

- what is the exact launch behavior with no arguments?
- what should the startup home show before the first prompt?
- what is the exact difference between `resume`, `sessions open`, and in-shell `/resume`?

### Session-model questions

- what is the stable session identifier shape across CLI and GUI?
- what is the minimal session metadata needed for recent-session cards?
- how should session titles be created and updated?

### Storage questions

- where should shared session state live?
- what is the authoritative transcript format?
- what should be cached versus reconstructed?

### Interaction questions

- which controls must be slash commands?
- which controls deserve a full-screen in-shell panel?
- what status data belongs in the bottom rail versus the main transcript?

### Runtime questions

- what runtime warnings should appear inline on startup?
- how should runtime-unavailable behavior degrade in the shell?
- which `runtime` commands are truly needed versus redundant with the GUI and doctor flow?

### GUI questions

- what must the desktop app share exactly with the CLI?
- what can the GUI add without creating a second product model?

## Suggested Future SpecKit Artifact Set

When the user asks for the full packet, the likely artifact set should include:

- `spec.md`
- `plan.md`
- `research.md`
- `data-model.md`
- `quickstart.md`
- `tasks.md`
- `analyze.md`
- one contract for session-shell behavior
- one contract for shared CLI/GUI session state if needed

The full packet should probably also point at current repo seams such as:

- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/src/bootstrap.rs`
- `crates/mister-smith-http/src/server.rs`
- `apps/operator-console/`

## Recommended Sequence

1. keep the existing proposal note as the lighter design summary
2. use this primer as the pre-SpecKit source of truth
3. when ready, author a full SpecKit packet from this primer
4. only after the packet exists, decide the implementation slice

## Stop Conditions For Future Packet Authoring

Stop before full SpecKit work if any of these are still unresolved:

- the product is still being framed as runtime-first instead of session-first
- CLI and GUI ownership are still split across incompatible session models
- the future packet is starting to include repo-workflow tooling
- the packet is trying to redesign unrelated runtime internals instead of the user shell

## Clear Recommendation

Treat the session-first user shell as its own product feature and do one real pre-SpecKit step
before full packet authoring.

The earlier proposal says where to go.

This primer says what the full SpecKit packet must preserve, what it must not widen into, and what
questions it must answer.

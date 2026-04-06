# Research Notes: Chat-First CLI Loop

## Current repo truth

- packet `030` already gives Mister Smith a recent-first CLI startup home, first-class resume, and
  in-session slash commands on current `main`
- durable session identity, retained history, stored control posture, runtime-unavailable fallback,
  and honest session inspection are already grounded in `mister-smith-app` and
  `mister-smith-http`
- packet `023` runtime-truth projections, packet `025` step-policy summaries, and packet `026`
  coordinator-runtime follow-up truth already define what the CLI must not overclaim
- the current live loop still sends a turn, prints accepted identifiers, re-renders a full session
  dump, and relies on detached inspection-style output to make progress understandable

## Why packet 030 is not the end state

Packet `030` solved the entry problem. The product can now open into a session-first CLI shell,
resume retained work, and steer controls in place.

That is necessary but not sufficient for the target product feel. The live experience still
teaches the user to submit work and then inspect session state, instead of staying inside one live
coding-agent conversation.

The next slice therefore needs to change the active session loop, not the startup home.

## Decision 1: Keep the current durable session model

- **Decision**: Reuse the current session identity, retained history, control posture, and
  session-facing views as the one source of truth.
- **Rationale**: The repo already has stable session seams and honest degraded-state fallback.
  Packet `031` should improve the conversation feel on top of those seams instead of creating a
  second loop model.
- **Alternatives considered**:
  - create a new chat-first session store: rejected because it would reopen landed continuity work
  - treat the loop as transient UI state only: rejected because resumed continuity would drift

## Decision 2: Make turn state inline instead of inspection-first

- **Decision**: The live session loop should surface accepted, active, completed, failed, and
  blocked turn states inline.
- **Rationale**: This is the smallest bounded change that makes the CLI feel like a real
  conversation without widening into a new runtime architecture.
- **Alternatives considered**:
  - keep accepted identifiers plus full session dumps: rejected because it preserves the current
    detached feel
  - add new admin or log commands: rejected because they move the user away from the loop

## Decision 3: Resume must land back in conversation

- **Decision**: Resumed work should reopen directly into a usable live loop with retained context
  and stored controls visible.
- **Rationale**: Resume is already first-class in packet `030`; the missing piece is conversation
  continuity after resume, not more resume entry points.
- **Alternatives considered**:
  - keep resume as a read-mostly inspect view: rejected because it breaks the desired product feel
  - reopen resumed work through a separate history browser: rejected because the loop should remain
    central

## Decision 4: Preserve truth boundaries inside the chat-first surface

- **Decision**: Busy, degraded, ended-session, and proof-limited states must stay explicit in the
  loop.
- **Rationale**: Mister Smith differentiates through supervised autonomy and honest proof claims,
  so a chat-first surface cannot hide runtime truth or imply stronger proof than the environment
  actually provides.
- **Alternatives considered**:
  - hide degraded states until failure: rejected because it weakens trust
  - collapse proof-boundary wording into generic chat language: rejected because it would overstate
    claims

## Bounded conclusion

The highest-leverage next slice is not broader shell parity or new workflow tooling. It is one
CLI-only packet that makes the active session feel like a live coding-agent conversation while
preserving the durable session and runtime-truth seams already landed on `main`.

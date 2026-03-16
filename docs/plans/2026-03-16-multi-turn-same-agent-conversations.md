# Multi-Turn Same-Agent Conversations — Ralph Loop And Symphony Handoff

Date: March 16, 2026

## Objective

Turn the March 16 runtime proof into a bounded usability feature packet for a real back-and-forth
conversation with the same retained agent across multiple turns.

This note is now the durable handoff companion to the SpecKit packet in:

- `specs/013-multi-turn-same-agent-conversations/spec.md`
- `specs/013-multi-turn-same-agent-conversations/plan.md`
- `specs/013-multi-turn-same-agent-conversations/research.md`
- `specs/013-multi-turn-same-agent-conversations/data-model.md`
- `specs/013-multi-turn-same-agent-conversations/contracts/session-surface.md`
- `specs/013-multi-turn-same-agent-conversations/quickstart.md`
- `specs/013-multi-turn-same-agent-conversations/tasks.md`

## Current Truth To Preserve

- the real runtime-backed operator path exists today through `mister-smith run`,
  `POST /api/v1/tasks`, `GET /api/v1/tasks/{task_id}`, and the workflow autonomy inspection
  surfaces
- that path is one-shot workflow submission, not a persistent conversation contract
- the current runtime reuses a process-global coordinator `agent_id`, but it rebuilds fresh
  planner state for each workflow, so current `agent_id` reuse is not yet an honest same-agent
  conversation
- the current root workflow identifier is also the current root `task_id`; the packet keeps that
  compatibility contract instead of replacing it

## Minimum Honest Contract

The packet defines exactly four operator actions:

1. **Create**
   - create a session and submit the first turn
   - return `session_id`, `workflow_id`, and `coordinator_agent_id`
2. **Continue**
   - append one new turn to an existing session
   - keep the same `session_id` and `coordinator_agent_id`
   - mint a new root `workflow_id`
3. **Inspect**
   - inspect session state, ordered turns, and workflow linkage
   - keep deep autonomy inspection keyed by `workflow_id`
4. **End**
   - logically close an idle session
   - preserve history and reject later turns

Same-agent guarantee for slice 1:

- same stable session-scoped `coordinator_agent_id`
- same persisted retained session context reconstructed on each turn

Not promised in slice 1:

- one immortal in-memory actor object
- fixed worker identities across turns
- concurrent queued turns

## Stable Identifier Relationship

- `session_id`: one durable conversation envelope
- `coordinator_agent_id`: one stable coordinator identity for that session
- `workflow_id`: one root workflow per accepted turn
- `workflow_id == root task_id` for compatibility with the current task-inspection surface

Relationship rule:

- one session owns many ordered workflow turns

## Recommended First Symphony Slice

Keep the slice bounded to these changes only:

1. session identifier types plus explicit PostgreSQL session and turn persistence
2. one session-aware service that wraps `RuntimeTaskService`
3. HTTP and CLI session surfaces for create, continue, inspect, and end
4. workflow autonomy linkage back to `session_id` and turn index

Explicitly out of scope:

- shared sessions
- multi-user collaboration
- queued concurrent turns
- force-end or force-cancel
- session branching
- worker-identity stability guarantees
- a parallel session-specific autonomy subsystem

## Suggested Ralph Loop Inputs

Use these as the only required input set for the first loop:

- `specs/013-multi-turn-same-agent-conversations/spec.md`
- `specs/013-multi-turn-same-agent-conversations/plan.md`
- `specs/013-multi-turn-same-agent-conversations/tasks.md`
- `specs/013-multi-turn-same-agent-conversations/contracts/session-surface.md`
- `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`

Ralph Loop objective:

- implement the bounded session contract without replacing the current one-shot runtime path

Ralph Loop stop conditions:

- stop if the implementation starts drifting into queued turns, forced cancellation, multi-user
  semantics, or a brand-new autonomy subsystem
- stop if `workflow_id` is no longer the root `task_id` alias for session-owned turns
- stop if the same-agent guarantee depends on one process-pinned actor instead of persisted session
  state

## Suggested Symphony Staging Posture

Smith classification for this packet:

- legitimacy: `legitimate_but_unstaged`
- project: `MisterSmith Validated Backlog`
- state: `Backlog`
- labels: `Validated`, `Symphony Candidate`

Recommended issue shape if staged:

- title: `Multi-turn same-agent conversations`
- type label: `Feature`
- primary crate label: `crate:app`
- note secondary crates in description: `crate:http`, `crate:persistence`, `crate:events`

Reason for backlog posture:

- the slice is now repo-validated and implementation-scoped, but it should enter the watched queue
  only when explicitly staged as runnable work

## Acceptance Criteria

- create one session and complete at least two accepted turns on that same session
- both turns share the same `session_id` and `coordinator_agent_id`
- both turns have distinct root `workflow_id` values
- session inspect returns ordered turn history plus active or last workflow linkage
- workflow autonomy inspection exposes enough session linkage to correlate a workflow back to its
  owning session and turn index
- ended sessions remain inspectable and reject new turns
- idle sessions survive runtime restart and can be inspected or continued afterward

## Open Questions

- Should slice 1 introduce a first-class `SessionId` newtype in `mister-smith-core`, or keep it as
  a plain UUID at the transport and persistence boundary?
- Should the session coordinator identity also be recorded into `agents.registry` in slice 1, or
  stay as a session-local `AgentId` until later lifecycle work?
- Should inspect return only ordered turn summaries in slice 1, or include full transcript bodies
  behind pagination later?
- Should the first live proof require only one real provider/model pair, or both a deterministic
  test path and a real provider-backed smoke path?

## Recommended Next Step

Land this docs/spec packet on `main`, then port the bounded session implementation from
`codex/recovery-20260316` onto `main` only after the remaining mixed runtime files are validated
as a safe session slice.

## Implementation Snapshot

Recovered implementation snapshot on `codex/recovery-20260316` included the bounded slice:

- `mister-smith-core`: added `SessionId` and `SessionStatus`
- `mister-smith-persistence`: added migration `00006_conversation_sessions.sql`, durable
  `tasks.sessions` and `tasks.session_turns`, plus repository/query helpers
- `mister-smith-app`: added `ConversationRuntimeService`, session-aware workflow submission,
  conversation CLI commands, and autonomy session-linkage enrichment from workflow metadata
- `mister-smith-http`: added session service contracts plus
  `POST /api/v1/sessions`, `POST /api/v1/sessions/{session_id}/turns`,
  `GET /api/v1/sessions/{session_id}`, and `POST /api/v1/sessions/{session_id}/end`
- `mister-smith-events`: extended `AutonomyStatusView` with optional `session_id`, `turn_index`,
  and `coordinator_agent_id`

Validation recorded on the recovered snapshot:

- `cargo build --workspace`
- `cargo test -p mister-smith-persistence`
- `cargo test -p mister-smith-events`
- `cargo test -p mister-smith-http`
- `cargo test -p mister-smith-app`

Still pending:

- port the bounded session implementation onto `main` in a clean recovery slice
- a real runtime smoke proof that creates one session, completes two turns, inspects the session,
  and verifies autonomy linkage on a live provider-backed run
- feature-specific HTTP and runtime integration tests for the new session endpoints beyond the
  current unit coverage

# Research: Multi-Turn Same-Agent Conversations

**Date**: 2026-03-16  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

The March 16 runtime proof already established the hard part that this feature should preserve:
Mister Smith can run a real persisted workflow through the current runtime and expose the result
through HTTP and autonomy inspection surfaces.

The missing capability is not "make runtime execution real." That part exists. The missing
capability is "make repeated turns belong to one honest same-agent conversation."

The strongest repo-local conclusion is therefore:

- do not invent a parallel chat system
- do not redefine workflow execution
- do add a durable session layer that wraps the proven root workflow path

## Current Repo Findings That Shape The Design

### R1: The Current Runtime Is Workflow-Centered, Not Session-Centered

**Sources**:

- `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
- `crates/mister-smith-http/src/server.rs`
- `crates/mister-smith-app/src/execution.rs`

**Evidence**:

- `POST /api/v1/tasks` creates one root workflow/task record
- `GET /api/v1/tasks/{task_id}` reads one root workflow/task record
- autonomy inspection is keyed by `workflow_id`
- `RuntimeTaskService::submit_task()` currently returns one root `task_id`, which is also the root
  `workflow_id`

**Decision**: preserve one root workflow per accepted turn and wrap it in a session layer instead
of replacing it.

**Alternatives considered**:

- Replace the workflow engine with a chat-only executor: rejected because it would discard the
  current real runtime proof.
- Treat conversation turns as child tasks only: rejected because the existing operator surface is
  already rooted at the top-level workflow record.

### R2: Stable `agent_id` Alone Does Not Prove A Retained Conversation

**Sources**:

- `crates/mister-smith-app/src/execution.rs`

**Evidence**:

- the runtime creates `coordinator_id: AgentId::new()` once at bootstrap
- the runtime reuses that coordinator ID across workflows
- the runtime also creates `PlannerState::default()` fresh for each workflow run

**Decision**: same-agent must be defined as stable session coordinator identity plus retained
session context, not as "the runtime happened to reuse one agent ID."

**Alternatives considered**:

- Define same-agent as reused `agent_id` only: rejected because it would overclaim continuity that
  the current runtime does not actually provide.
- Require one always-live in-memory actor for the session: rejected because it would fail restart
  and would widen the slice unnecessarily.

### R3: Explicit Session Persistence Is Cheaper And More Honest Than Metadata-Only State

**Sources**:

- `crates/mister-smith-persistence/migrations/00001_initial_schema.sql`
- `crates/mister-smith-persistence/src/postgres/queries.rs`

**Evidence**:

- `tasks.records` already stores root workflow state, correlation IDs, parent links, and metadata
- no durable session or ordered turn table exists today
- busy-session conflict detection, session end, and restart resume all require explicit session
  state

**Decision**: add explicit session and turn persistence rather than hiding the whole feature inside
  `tasks.records.metadata`.

**Alternatives considered**:

- store everything inside root workflow metadata only: rejected because inspection and resume would
  become brittle and query-hostile
- create a new independent persistence subsystem: rejected because the existing PostgreSQL task
  store already has the right operational boundary

### R4: Workflow Autonomy Should Stay The Deep Inspection Surface

**Sources**:

- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`

**Evidence**:

- autonomy status is accumulated per workflow ID
- the event bus already assembles an operator view from typed workflow-scoped events
- no session-scoped autonomy accumulator exists today

**Decision**: keep autonomy inspection workflow-scoped and add optional session linkage fields so a
workflow autonomy view can be traced back to its owning session.

**Alternatives considered**:

- invent a full parallel session-autonomy subsystem: rejected because it duplicates the current
  control plane
- leave autonomy unchanged and force operators to correlate manually: rejected because it would
  leave the same-agent contract opaque

### R5: Initial Session Lifecycle Must Be Conservative

**Sources**:

- `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`
- current root workflow task/autonomy lifecycle

**Evidence**:

- the current runtime already has explicit queued/running/completed/failed workflow states
- adding turn queueing, force-cancel, or shared conversations would multiply lifecycle cases

**Decision**: one active turn per session, reject concurrent continue and active-session end in the
first slice.

**Alternatives considered**:

- queue multiple turns per session: rejected because it adds hidden ordering rules and wider
  persistence logic
- allow end to force-cancel active work: rejected because cancellation semantics are not part of
  the current honest proof

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` | Defines the current honest runtime-backed operator surface and its one-shot boundary |
| `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md` | Defines the usability gap that this packet needs to close |
| `crates/mister-smith-app/src/execution.rs` | Shows the current root workflow execution contract, root record shape, fixed coordinator ID, and fresh planner state per run |
| `crates/mister-smith-http/src/server.rs` | Shows the current task submission and task lookup service boundary |
| `crates/mister-smith-app/src/autonomy.rs` | Shows the current workflow-scoped operator autonomy inspection surface |
| `crates/mister-smith-events/src/autonomy.rs` | Shows the current workflow-scoped typed autonomy status shape |
| `crates/mister-smith-events/src/bus.rs` | Shows autonomy accumulation keyed by workflow ID |
| `crates/mister-smith-persistence/migrations/00001_initial_schema.sql` | Shows the current task record substrate that the session layer should extend |
| `crates/mister-smith-persistence/src/postgres/queries.rs` | Shows existing task lookup and correlation seams that can host session linkage |

## Explicitly Deferred Questions

- whether session-scoped coordinator identities should also be registered in `agents.registry`
- whether the initial inspect response should expose full transcript bodies or only turn summaries
- whether future slices should support queued turns, force-end, or branchable session histories

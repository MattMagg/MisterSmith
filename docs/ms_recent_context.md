# Mister Smith Recent Context

Date: March 21, 2026
Status: Current

## Current State

- `main` is the only durable development branch.
- Packet 015 is fully landed on `main` through `MS-94`, and its parent epic `MS-78` is closed.
- Packet 016 is fully landed on `main` through `MS-97` through `MS-100`, and its parent epic
  `MS-96` is closed.
- The runtime-backed task path is live and locally proven on `openai_chatgpt` with `gpt-5.4`.
- The bounded same-agent session slice is live on `main`.
- Harder-workload graph proof, unified result projection, bounded operator preview/provenance, and
  persisted proof-outcome visibility are now landed on `main`.
- Smith now exposes workflow-family tools for issue and workpad mutation, backlog slicing, watched
  queue staging, lifecycle resolution, Ralph packet flows, and SpecKit task translation.
- Symphony's watched queue is empty again after packet-016 closure; there are no honest refill
  candidates in the current frontier family.

## Durable Sources To Read First

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md`
4. `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
5. `WORKFLOW.md`
6. `docs/linear/LINEAR.md`
7. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
8. `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`

## What Just Landed

- packet 016 closure through `MS-97` through `MS-100` and parent epic `MS-96`
- harder-workload graph proof on the default path
- shared result contract and proof-outcome taxonomy across task, session, and operator surfaces
- bounded operator preview/provenance and persisted proof-outcome visibility
- delegated HTTP task-ingress continuity through workflow metadata and workflow-level autonomy
  inspection

## Current Direction

The current development-workflow program is still Smith-first, but the March 16 notes are now
supporting history rather than the primary direction router:

- `docs/current-state.md` is the current repo-wide router
- `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md` is the current
  forward-development checkpoint
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  records the completed packet-016 closure

The next frontier planning action is no longer “create packet 016.” Packet 016 is closed. The next
bounded planning move is to start from the new checkpoint and identify one fresh repo-grounded gap
before creating another packet or staging new watched-queue work.

Closed frontier packet and recovery issues now include:

- `MS-43` first live multi-agent runtime proof
- `MS-44` multi-turn same-agent conversations
- `MS-78` packet-015 parent epic
- `MS-96` packet-016 parent epic
- `MS-89` through `MS-94` packet-015 execution slices
- `MS-97` through `MS-100` packet-016 execution slices

## Queue Posture

- keep future bounded work in `MisterSmith Validated Backlog`
- do not move anything into `Todo` until a new execution cycle stages an honest runnable slice
- when staging begins, split the primary epic into parallelizable sub-slices with non-overlapping
  write sets

## Resume Checklist

- confirm repo state and current `main`
- confirm the watched queue is still empty before creating new runnable work
- start with `route_workflow_request`, `get_control_plane_snapshot`, and
  `get_issue_execution_snapshot` before falling back to raw Linear or ad hoc workflow glue
- use `docs/current-state.md` and
  `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md` as the first direction read
- use `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md` as the source of truth for the
  currently implemented Smith workflow-family surface
- use `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  when packet-016 closure evidence matters
- update Linear and repo docs together when the direction changes

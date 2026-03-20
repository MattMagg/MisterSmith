# Mister Smith Recent Context

Date: March 20, 2026
Status: Current

## Current State

- `main` is the only durable development branch.
- Packet 015 is fully landed on `main` through `MS-94`, and its parent epic `MS-78` is closed.
- The runtime-backed task path is live and locally proven on `openai_chatgpt` with `gpt-5.4`.
- The bounded same-agent session slice is live on `main`.
- Harder-workload graph proof, unified result projection, bounded operator preview/provenance, and
  persisted proof-outcome visibility are now landed on `main`.
- Smith now exposes workflow-family tools for issue and workpad mutation, backlog slicing, watched
  queue staging, lifecycle resolution, Ralph packet flows, and SpecKit task translation.
- Symphony's watched queue is currently empty again after packet-015 closure; the next refill must
  come from an honest new staging pass rather than historical residue.

## Durable Sources To Read First

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/plans/2026-03-19-central-development-checkpoint.md`
4. `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`
5. `WORKFLOW.md`
6. `docs/linear/LINEAR.md`
7. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
8. `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`

## What Just Landed

- packet 015 closure through `MS-94` and parent epic `MS-78`
- harder-workload graph proof on the default path
- shared result contract and proof-outcome taxonomy across task, session, and operator surfaces
- bounded operator preview/provenance and persisted proof-outcome visibility

## Current Direction

The current development-workflow program is still Smith-first, but the March 16 notes are now
supporting history rather than the primary direction router:

- `docs/current-state.md` is the current repo-wide router
- `docs/plans/2026-03-19-central-development-checkpoint.md` is the current forward checkpoint
- `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`
  records the completed packet-015 closure

The next frontier planning action is no longer “write packet 015.” Packet 015 is closed. The next
bounded planning move is to refresh the forward-development checkpoint before starting another
frontier implementation lane.

If the same line of product work continues, the remaining bounded follow-on is:

- the remaining post-`MS-77` external-agent interoperability closure on a bounded surface

Closed frontier packet and recovery issues now include:

- `MS-43` first live multi-agent runtime proof
- `MS-44` multi-turn same-agent conversations
- `MS-78` packet-015 parent epic
- `MS-89` through `MS-94` packet-015 execution slices

## Queue Posture

- keep future bounded work in `MisterSmith Validated Backlog`
- do not move anything into `Todo` until the next execution cycle stages an honest runnable slice
- when staging begins, split the primary epic into parallelizable sub-slices with non-overlapping
  write sets

## Resume Checklist

- confirm repo state and current `main`
- confirm the watched queue is still empty before creating new runnable work
- start with `route_workflow_request`, `get_control_plane_snapshot`, and
  `get_issue_execution_snapshot` before falling back to raw Linear or ad hoc workflow glue
- use `docs/current-state.md` and `docs/plans/2026-03-19-central-development-checkpoint.md` as
  the first direction read
- use `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md` as the source of truth for the
  currently implemented Smith workflow-family surface
- use `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`
  when packet-015 closure evidence matters
- update Linear and repo docs together when the direction changes

# Mister Smith Recent Context

Date: March 16, 2026
Status: Current

## Current State

- `main` is the only durable development branch.
- The March 16 recovery work is reconciled onto `main`.
- The runtime-backed task path is live and locally proven on `openai_chatgpt` with `gpt-5.4`.
- The bounded same-agent session slice is live on `main`.
- Smith now exposes workflow-family tools for issue and workpad mutation, backlog slicing, watched
  queue staging, lifecycle resolution, Ralph packet flows, and SpecKit task translation.
- Symphony's watched queue is intentionally empty because no new work has been staged into `Todo`.

## Durable Sources To Read First

1. `AGENTS.md`
2. `WORKFLOW.md`
3. `docs/linear/LINEAR.md`
4. `docs/plans/2026-03-16-smith-first-development-system.md`
5. `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md`
6. `docs/plans/2026-03-16-winyear-frontier-direction.md`
7. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
8. `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`

## What Just Landed

- runtime-backed `POST /api/v1/tasks` and operator-visible autonomy inspection
- real provider-backed local runtime proof
- bounded same-agent session create, continue, inspect, and end surfaces
- recovery reconciliation directly onto `main`

## Current Direction

The current development-workflow program is Smith-first:

- `docs/plans/2026-03-16-smith-first-development-system.md` is the active note for making Smith
  the default development control plane across planning, coding, validation, review, Ralph, and
  SpecKit flows.

The next program is `WinYear`: make Mister Smith clearly behave like an orchestration operating
system rather than a generic framework.

Recommended primary epic:

- `MS-45` task-shape-aware orchestration and dynamic team sizing

Additional validated backlog epics:

- `MS-46` session restart-resume and distributed operating state
- `MS-47` step-level intelligence and model routing control loop
- `MS-48` capability kernel and external-agent interoperability

Closed March 16 recovery issues:

- `MS-43` first live multi-agent runtime proof
- `MS-44` multi-turn same-agent conversations

## Queue Posture

- keep the next epics in `MisterSmith Validated Backlog`
- do not move them into `Todo` until the next execution cycle explicitly stages bounded slices
- when staging begins, split the primary epic into parallelizable sub-slices with non-overlapping
  write sets

## Resume Checklist

- confirm repo state and current `main`
- confirm the watched queue is still empty before creating new runnable work
- start with `route_workflow_request`, `get_control_plane_snapshot`, and
  `get_issue_execution_snapshot` before falling back to raw Linear or ad hoc workflow glue
- use `docs/plans/2026-03-16-smith-first-development-system.md` as the source of truth for the
  development-workflow model
- use `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md` as the source of truth for the
  currently implemented Smith workflow-family surface
- use `docs/plans/2026-03-16-winyear-frontier-direction.md` as the source of truth for "what next"
- update Linear and repo docs together when the direction changes

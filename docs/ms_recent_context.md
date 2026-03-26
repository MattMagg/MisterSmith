# Mister Smith Recent Context

Date: March 26, 2026
Status: Current

## Current State

- `main` is the only durable development branch.
- Packet 015 is fully landed on `main` through `MS-94`, and its parent epic `MS-78` is closed.
- Packet 016 is fully landed on `main` through `MS-97` through `MS-100`, and its parent epic
  `MS-96` is closed.
- Packet 017 is landed on `main` as bounded runtime provider selection.
- The runtime-backed task path is live and locally proven on `openai_chatgpt` with `gpt-5.4`.
- The runtime-backed task path now reads `provider_kind` and `model_id` from framework config for
  the supported shipped providers `openai_chatgpt`, `claude_subscription`, and `mock`; only the
  `openai_chatgpt` / `gpt-5.4` path has live proof so far.
- Packet 019 is now in progress on `main`: the runtime-backed task path accepts a typed
  `runtime_routing_profile`, can boot a bounded multi-provider cascade with JetStream-backed
  budget enforcement when configured, and keeps the no-profile single-provider fallback intact.
- Task and autonomy provenance now surface runtime routing policy, budget root, and the latest
  accepted step tier/checkpoint evidence from the runtime task path.
- Packet 018 is the in-review smoke-harness lane and is not yet landed on `main`.
- The bounded same-agent session slice is live on `main`.
- Harder-workload graph proof, unified result projection, bounded operator preview/provenance, and
  persisted proof-outcome visibility are now landed on `main`.
- Smith now exposes workflow-family tools for issue and workpad mutation, backlog slicing, watched
  queue staging, lifecycle resolution, Ralph packet flows, and SpecKit task translation.
- The next scope-frozen packet after packet 017 is packet 019, the budget-backed runtime routing
  control-loop lane.

## Durable Sources To Read First

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md`
4. `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md`
5. `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
6. `WORKFLOW.md`
7. `docs/linear/LINEAR.md`
8. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`

## What Just Landed

- packet 016 closure through `MS-97` through `MS-100` and parent epic `MS-96`
- packet 019 runtime routing slices for typed profile config, bounded multi-provider bootstrap,
  JetStream-backed budget-store wiring, and routing-evidence surfacing
- harder-workload graph proof on the default path
- shared result contract and proof-outcome taxonomy across task, session, and operator surfaces
- bounded operator preview/provenance and persisted proof-outcome visibility
- delegated HTTP task-ingress continuity through workflow metadata and workflow-level autonomy
  inspection

## Current Direction

The current development-workflow program is still Smith-first, but the March 16 notes are now
supporting history rather than the primary direction router:

- `docs/current-state.md` is the current repo-wide router
- `docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md` is the current
  next-phase scope freeze
- `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md` remains the checkpoint that
  required this fresh bounded packet
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  records the completed packet-016 closure

The next frontier planning action is no longer “identify a fresh gap.” That guardrail has been
satisfied. The next bounded planning lane is packet `019`, which freezes the budget-backed runtime
routing control loop as the next development phase. The config/bootstrap/budget/evidence slices are
now landed on `main`; the remaining bounded gap is honest proof guidance or equivalent repeatable
runtime evidence for the configured budget-aware path.

Closed frontier packet and recovery issues now include:

- `MS-43` first live multi-agent runtime proof
- `MS-44` multi-turn same-agent conversations
- `MS-78` packet-015 parent epic
- `MS-96` packet-016 parent epic
- `MS-89` through `MS-94` packet-015 execution slices
- `MS-97` through `MS-100` packet-016 execution slices
- packet `017` bounded runtime provider selection

## Queue Posture

- keep future bounded work in `MisterSmith Validated Backlog`
- do not move anything into `Todo` until a new execution cycle stages an honest runnable slice
- when staging begins, split the primary epic into parallelizable sub-slices with non-overlapping
  write sets

## Resume Checklist

- confirm repo state and current `main`
- confirm whether packet `018` has landed before implementation work depends on its proof harness
- start with `route_workflow_request`, `get_control_plane_snapshot`, and
  `get_issue_execution_snapshot` before falling back to raw Linear or ad hoc workflow glue
- use `docs/current-state.md` and
  `docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md` as the first direction
  read
- use `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md` as the source of truth for the
  currently implemented Smith workflow-family surface
- use `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  when packet-016 closure evidence matters
- update Linear and repo docs together when the direction changes

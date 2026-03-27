# Mister Smith Recent Context

Date: March 27, 2026
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
- Packet 019 is now complete on `main`: the runtime-backed task path accepts a typed
  `runtime_routing_profile`, can boot a bounded multi-provider cascade with JetStream-backed
  budget enforcement when configured, and keeps the no-profile single-provider fallback intact.
- Packet 020 is now landed on `main` through `MS-104` through `MS-107`: verifier-gated
  workflow-step decisions, first-class handoff clarification, preserved failure-context repair
  lineage, and operator-visible orchestration-quality provenance are now part of the runtime task
  path.
- Task and autonomy provenance now surface runtime routing policy, budget root, the latest
  accepted step tier/checkpoint evidence, and verifier/repair lineage from the runtime task path.
- one bounded packet-019 live proof now exists for the
  `budget_softcap_openai_mock` profile; the accepted provider-backed tier remained
  `openai_chatgpt` / `gpt-5.4`, while the live step-routing outcome recorded
  `tier=primary`, `action=downgrade`, and `triggered_checkpoints=["budget_policy"]`
- Packet 018 is the in-review smoke-harness lane and is not yet landed on `main`.
- The bounded same-agent session slice is live on `main`.
- Harder-workload graph proof, unified result projection, bounded operator preview/provenance, and
  persisted proof-outcome visibility are now landed on `main`.
- Smith now exposes workflow-family tools for issue and workpad mutation, backlog slicing, watched
  queue staging, lifecycle resolution, Ralph packet flows, and SpecKit task translation.
- No newer post-packet-020 bounded phase is frozen yet.

## Durable Sources To Read First

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
4. `docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md`
5. `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
6. `WORKFLOW.md`
7. `docs/linear/LINEAR.md`
8. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`

## What Just Landed

- packet 016 closure through `MS-97` through `MS-100` and parent epic `MS-96`
- packet 019 runtime routing slices for typed profile config, bounded multi-provider bootstrap,
  JetStream-backed budget-store wiring, routing-evidence surfacing, and bounded live-proof closure
- harder-workload graph proof on the default path
- shared result contract and proof-outcome taxonomy across task, session, and operator surfaces
- bounded operator preview/provenance and persisted proof-outcome visibility
- delegated HTTP task-ingress continuity through workflow metadata and workflow-level autonomy
  inspection

## Current Direction

The current development-workflow program is still Smith-first, but the March 16 notes are now
supporting history rather than the primary direction router:

- `docs/current-state.md` is the current repo-wide router
- `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md` is the packet-020 scope and
  closure note for the most recently landed frontier slice
- `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` is the bounded closure note for
  the completed packet-019 proof lane
- `docs/plans/2026-03-26-budget-backed-runtime-routing-control-loop.md` is the packet-019 scope
  freeze and closure router
- `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md` is now historical support
  context that led into packets `019` and `020`; it is no longer the active router
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  records the completed packet-016 closure

The next frontier planning action is no longer "close packet 020." Packet `020` is now landed on
`main`; the next bounded post-packet-020 phase has not yet been frozen.

Closed frontier packet and recovery issues now include:

- `MS-43` first live multi-agent runtime proof
- `MS-44` multi-turn same-agent conversations
- `MS-78` packet-015 parent epic
- `MS-96` packet-016 parent epic
- packet `019` bounded runtime routing control loop
- `MS-103` packet-020 parent epic
- `MS-89` through `MS-94` packet-015 execution slices
- `MS-97` through `MS-100` packet-016 execution slices
- `MS-104` through `MS-107` packet-020 execution slices
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
  `docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md` as the first packet-020
  closure/context read
- use `docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md` as the source of truth for the
  currently implemented Smith workflow-family surface
- use `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  when packet-016 closure evidence matters
- update Linear and repo docs together when the direction changes

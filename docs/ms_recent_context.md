# Mister Smith Recent Context

Date: April 5, 2026
Status: Current

## Current State

- `main` is the only durable development branch and is currently synced at `d30e627`.
- packet `015` is fully landed on `main` through `MS-94`, and its parent epic `MS-78` is closed
- packet `016` is fully landed on `main` through `MS-97` through `MS-100`, and its parent epic
  `MS-96` is closed
- packet `017` is landed on `main` as bounded runtime provider selection
- the runtime-backed task path is live and locally proven on `openai_chatgpt` with `gpt-5.4`
- the runtime-backed task path reads `provider_kind` and `model_id` from framework config for the
  shipped providers `openai_chatgpt`, `claude_subscription`, and `mock`; only the
  `openai_chatgpt` / `gpt-5.4` path carries the explicit live-proof baseline so far
- packet `019` is complete on `main`: the runtime-backed task path accepts a typed
  `runtime_routing_profile`, can boot a bounded multi-provider cascade with JetStream-backed
  budget enforcement when configured, and keeps the no-profile single-provider fallback intact
- packet `020` is landed on `main` through `MS-104` through `MS-107`: verifier-gated
  workflow-step decisions, first-class handoff clarification, preserved failure-context repair
  lineage, and operator-visible orchestration-quality provenance are part of the runtime task path
- packet `021` is landed on `main`: predictive-supervision evidence and explicit proof-boundary
  wording are projected on current read surfaces with deterministic validation
- packet `022` is landed on `main`: durable workflow lifecycle, event-history, compaction, and
  effect-boundary ownership are part of current repo truth with deterministic validation
- packet `023` is landed on `main`: shared `runtime_truth`, proof-boundary, and bounded run-trace
  projections are now part of task, session, autonomy, and operator surfaces with deterministic
  validation
- packet `024` is landed on `main`: action-bound capability enforcement, clearer quarantine
  reasons, and clamped auth-callout fallback ceilings are now part of current boundary truth with
  deterministic validation
- packet `025` is landed on `main`: deterministic step-policy summaries now project difficulty,
  bounded action choice, and honest proof wording through task, autonomy, and operator surfaces
- packet `026` is the next implementation-ready packet for first real coordinator-subagent runtime
- packets `027` and `028` remain later scaffolds
- the bounded same-agent session slice is live on `main`
- GitHub Actions are intentionally disabled in this repository; local validation plus CodeRabbit
  and operator review are the current review posture
- legacy Symphony watched-queue state is a development-workflow surface and can remain empty
  without implying a product problem

## Durable Sources To Read First

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/direction.md`
4. `specs/023-runtime-truth-and-run-trace/`
5. `specs/024-agent-boundary-security-hardening/`
6. `specs/025-step-level-intelligence-v2/`
7. `specs/026-first-real-coordinator-subagent-runtime/`
8. `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`
9. `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
10. `WORKFLOW.md`
11. `docs/linear/LINEAR.md`

## What Just Landed

- packet `023` runtime-truth, proof-boundary, and bounded run-trace projections
- packet `024` agent-boundary security hardening across ToolBus, MCP, quarantine, and
  auth-callout fallback behavior
- packet `025` deterministic step-policy summaries across task, autonomy, and operator surfaces

## Current Direction

The current development-workflow program is still Smith-first, but the repo-wide router and next
packet story have moved forward:

- `docs/current-state.md` is the current repo-wide router
- `docs/direction.md` is the current strategic priority source
- `specs/023-runtime-truth-and-run-trace/` and
  `specs/024-agent-boundary-security-hardening/` are the latest landed packet authorities
- `specs/025-step-level-intelligence-v2/` is the latest landed step-policy packet authority
- `specs/026-first-real-coordinator-subagent-runtime/` is the next implementation-ready packet
- `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md` remains the packet-021
  deterministic proof-boundary note
- `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` remains the last explicit
  bounded live-proof note on the supported path

The next frontier planning action is no longer "close packet 020" or "land packet 023/024/025."
Those packets are already landed on `main`. The next bounded planning move is to stage packet
`026` as one honest runnable slice when direct execution is intentionally selected.

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
- packets `021`, `022`, `023`, `024`, and `025`

## Queue Posture

- keep future bounded work in `MisterSmith Validated Backlog`
- do not move packet `026` into `Todo` until one honest runnable slice is deliberately selected
- keep packets `027` and `028` out of active execution until they are deliberately promoted

## Resume Checklist

- confirm repo state and current `main`
- confirm that packets `023`, `024`, and `025` are landed on `main` and packet `026` is the next
  implementation-ready slice
- start with `route_workflow_request`, `get_control_plane_snapshot`, and
  `get_issue_execution_snapshot` before falling back to raw Linear or ad hoc workflow glue
- use `docs/current-state.md`, `docs/direction.md`, and
  `specs/026-first-real-coordinator-subagent-runtime/` as the first read for the next packet move
- use `specs/023-runtime-truth-and-run-trace/` and
  `specs/024-agent-boundary-security-hardening/` and
  `specs/025-step-level-intelligence-v2/` when landed implementation truth matters
- use `docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md` as the source of truth for
  the current Smith workflow-family surface
- update Linear and repo docs together when the direction changes

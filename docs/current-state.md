# Mister Smith Current State

Date: April 9, 2026
Status: Current

## Purpose

This is the stable repo-wide overview, live-truth summary, and document router for Mister Smith.

Use `docs/direction.md` when you need the merged strategic answer for where Mister Smith is going
and what should be built next.

Use this file when you need one honest answer to:

- what is currently true on `main`
- what is part of the Mister Smith operating system
- what exists in the repo but is not yet the default runtime path
- which document to read next

## Document Roles

| Need | Primary document | Role |
| ---- | ---------------- | ---- |
| Overall direction | `docs/direction.md` | Single authoritative direction source that merges repo truth and strategic priority |
| Whole-repo overview | `docs/current-state.md` | Current repo and OS state, plus document routing |
| Latest landed runtime-truth packet | `specs/023-runtime-truth-and-run-trace/` | Latest landed runtime-truth and run-trace packet on `main` |
| Latest landed security packet | `specs/024-agent-boundary-security-hardening/` | Latest landed least-privilege agent-boundary packet on `main` |
| Latest landed step-policy packet | `specs/025-step-level-intelligence-v2/` | Latest landed step-policy packet on `main` |
| Latest landed coordinator-runtime packet | `specs/026-first-real-coordinator-subagent-runtime/` | Latest landed coordinator-runtime packet on `main` |
| Latest landed session-first CLI packet | `specs/030-session-first-cli-shell/` | Landed session-first CLI shell packet on `main` |
| Latest landed chat-first CLI packet | `specs/031-chat-first-cli-loop/` | Landed chat-first CLI loop packet on `main` |
| Remaining unpromoted packet material | `specs/027-*` through `specs/029-*` | Draft or pre-spec packet material still requires deliberate promotion |
| Packet 022 implementation authority | `specs/022-durable-workflow-core/` | Latest durable-workflow implementation authority on current `main` |
| Current bounded live-proof note | `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md` | Latest bounded smoke-harness note and artifact lane |
| Packet 021 closure evidence | `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md` | Deterministic packet-021 proof-boundary and supervision-evidence note |
| Packet 019 closure evidence | `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` | Historical bounded live-proof and proof-boundary note for packet `019` |
| Packet 016 closure evidence | `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md` | Durable proof and final validation artifact for the completed packet-016 epic |
| Packet 015 closure evidence | `docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md` | Historical live-proof and validation artifact for the completed packet-015 epic |
| Smith-first dev control plane | `AGENTS.md` and `docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md` | Active Smith-first contract for request routing and gate-aware execution prep |
| Historical workflow automation background | `WORKFLOW.md` plus local-only `docs/linear/` notes when present | Legacy Linear and Symphony operating context; not the active Smith MCP route |
| Architectural build map | `ROADMAP.md` | Phase dependency map and build order |
| Broad repo orientation | `README.md` | High-level repo surface and operator entry points |
| Operator-oriented repo orientation | `CLAUDE.md` | Concise working overview for contributors and agents |
| Runtime proof details | `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md` | Historical proof packet and repeatable live-run context |
| Session slice details | `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md` | Bounded session packet and handoff |
| Canonical system contract | `spec/` | Architecture truth |
| Phase implementation packets | `specs/` | Build instructions and task packs |

## Product Boundary

Mister Smith OS is the Rust workspace and its runtime surfaces:

- the runtime boot and supervision substrate
- NATS and JetStream transport
- PostgreSQL-backed persistence
- HTTP and CLI operator surfaces
- autonomy, task, and session state
- the agent, routing, and execution crates in this repo

External development workflow systems are not part of the Mister Smith OS itself:

- Linear
- Symphony
- Ralph
- SpecKit

Those are development and control-plane tools used to build, stage, review, and operate work on
the repo. They are important to development flow, but they are not runtime subsystems of the OS.

The shipped `mister-smith-mcp` crate belongs to this workspace and this product boundary. It also
serves as a repo control-plane surface, but it should not be collapsed into the same bucket as
external workflow services such as Linear or Symphony.

## Current Repo-Wide State

- `main` is the durable development branch and is currently synced at
  `aae62e5f932a99b96efce5686e8c1d9c0a4635a1`.
- The workspace contains 20 crates: 18 library crates, 1 binary crate, and 1 integration-test
  crate.
- Phases 1 through 10 are landed in the repo as implemented substrate and validation artifacts.
- GitHub Actions are intentionally disabled in this repository; current review posture is local
  validation plus CodeRabbit and operator review.
- The current live operator surfaces are:
  - the chat-first session shell through `mister-smith`, plus `mister-smith resume` and
    `mister-smith sessions`
  - one-shot workflow execution through `mister-smith run` and `POST /api/v1/tasks`
  - autonomy inspection through `mister-smith autonomy list` and `mister-smith autonomy status`
  - bounded same-agent session handling through `POST /api/v1/sessions` and related session routes
  - the local macOS Tauri operator cockpit under `apps/operator-console/`, including managed
    runtime bootstrap, list/detail inspection, task/session actions, and websocket timeline
- the default runtime path uses supervised planner and executor lifecycles, a Tokio workflow
  runner, and a ToolBus-backed execution boundary
- a real local provider-backed runtime proof has been completed on the supported
  `openai_chatgpt` / `gpt-5.4` path, and the repo ships a repeatable smoke harness for that
  bounded proof surface under `scripts/live_runtime_proof_smoke.py`
- the current bounded live-proof note and artifact lane live under
  `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md` and its sibling artifact tree, while
  `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` remains historical budget-aware
  proof provenance
- the default runtime path reads `provider_kind` and `model_id` from framework configuration for
  the shipped providers `openai_chatgpt`, `claude_subscription`, and `mock`
- packet `019` is landed on `main`: when `llm.runtime_routing_profile` is configured, the runtime
  can boot a bounded multi-provider cascade plus JetStream-backed budget enforcement; when no
  profile is configured, the current single-provider `RoundRobin` fallback remains intact
- packet `020` is landed on `main` through `MS-104` through `MS-107`: the runtime-backed task path
  supports verifier-gated step decisions, first-class handoff clarification, preserved
  failure-context plus last stable checkpoint repair lineage, and operator-visible
  orchestration-quality provenance
- packet `021` is landed on `main`: the runtime-backed task path projects bounded
  predictive-supervision evidence plus explicit proof-boundary text on task inspect, autonomy
  status, and operator-console run detail, with deterministic validation
- packet `022` is landed on `main`: durable workflow lifecycle, event-history, bounded
  compaction, and effect-boundary ownership are part of current repo truth with deterministic
  validation
- packet `023` is landed on `main`: one shared `runtime_truth` contract, one bounded run-trace
  summary, and one explicit proof-boundary view are projected across task, session, autonomy, and
  operator surfaces, with deterministic validation
- packet `024` is landed on `main`: action-bound external capability enforcement, clearer
  quarantine reasons, and auth-callout fallback clamping are part of current ToolBus, MCP, and
  security truth with deterministic validation
- packet `025` is landed on `main`: step-policy summaries now project deterministic difficulty,
  bounded action choice, and honest proof wording through task, autonomy, and operator run-detail
  surfaces
- packet `026` is landed on `main`: coordinator-owned delegation, child-state projection,
  delegated-work evidence, coordinator decisions, and bounded session follow-up refs are part of
  current repo truth with deterministic validation
- packet `030` is landed on `main`: the session-first CLI shell now opens into a recent-first
  retained-session home with resume, browse, and in-session control surfaces, backed by
  deterministic validation
- packet `031` is landed on `main`: the active CLI session now stays inside a chat-first loop with
  inline turn-state, resumed continuity, and truth-notice behavior, backed by deterministic
  validation
- packets `027` and `028` remain draft scaffolds, and packet `029` remains draft pre-spec
  planning; no later packet is currently promoted as the next implementation-ready slice
- the watched Symphony queue can be empty without implying a product problem; that queue is part
  of the development workflow, not the OS runtime

## Important Distinction

Phase and packet completion in this repo means the relevant substrate, crates, tests, and artifact
sets are landed. It does not automatically mean every advanced seam is already covered by a fresh
live rerun on the default runtime path.

Read the current state in three layers:

1. **Landed in repo**
   - implemented crates, contracts, tests, and validation artifacts exist
2. **Live end-to-end proof baseline**
   - a real bounded rerun exists on the supported runtime path
3. **Deterministically validated on current `main`**
   - the seam is landed and tested, but this repo does not claim a newer live rerun yet

## What Is Live And Proven Now

- runtime boot, health probes, and shutdown behavior
- NATS/JetStream plus PostgreSQL local runtime prerequisites
- repo-native local stack bootstrap for `postgres`, `nats`, and the bundled `mister-smith`
  runtime from the macOS operator cockpit
- repo-owned repeatable smoke proof for the current `openai_chatgpt` / `gpt-5.4` task path,
  including Docker-backed prerequisite checks, internal NATS `varz` verification, real task
  submission, autonomy inspection, and predictable proof artifacts
- real workflow submission and terminal completion tracking
- autonomy inspection surfaces keyed by `workflow_id`
- bounded same-agent sessions with stable `session_id` and `coordinator_agent_id`
- recent-first CLI startup, resume, retained-session browsing, and chat-first active-session loop
  flows through the shipped CLI shell
- supervised planner and executor lifecycles on the default runtime path
- ToolBus-backed workflow step execution on the default runtime path
- bounded runtime provider/model selection through framework config for `openai_chatgpt`,
  `claude_subscription`, and `mock`, with the live proof baseline still recorded on
  `openai_chatgpt` and `gpt-5.4`
- one config-gated budget-aware runtime profile with `routing_policy=cascade`,
  `registered_provider_count=2`, `budget_root=runtime.task_path`, and a live
  `latest step routing tier=primary action=downgrade checkpoints=budget_policy` proof surface,
  recorded in `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
- smallest-workflow live planning as the default runtime posture:
  - stay sequential unless the task clearly benefits from parallel work
  - create a coordinator merge step only when multiple branches actually exist
  - preserve the requested output shape instead of forcing a canned memo pattern

The current bounded live runtime-proof baseline remains the supported `openai_chatgpt` / `gpt-5.4`
smoke-harness lane. Later landed packets extend the runtime path on `main`, but shipped code,
deterministic validation, and fresh live reruns should still be described separately.

## What Is Landed On Main With Deterministic Validation

- packet-021 predictive-supervision evidence on task inspect, autonomy status, and operator run
  detail surfaces
- packet-022 durable workflow ownership for lifecycle, history, compaction, and effect boundaries
- packet-023 runtime-truth, proof-boundary, and bounded run-trace summaries across task, session,
  autonomy, and operator surfaces
- packet-024 least-privilege agent-boundary enforcement across ToolBus, MCP metadata and execute
  paths, quarantine reporting, and auth-callout fallback ceilings
- packet-025 deterministic step-policy summaries across task, autonomy, and operator run-detail
  surfaces
- packet-026 coordinator-runtime delegation, child-state, delegated-work, decision, and bounded
  follow-up projections across task, autonomy, session, and operator surfaces

## What Exists In The Repo But Is Not Yet The Default Runtime Path

These capabilities are real in the codebase, but the default runtime path does not yet use all of
them end to end:

- a config-gated bounded multi-provider runtime routing profile with JetStream-backed budget
  enforcement that is live-proven for one bounded profile but is still opt-in rather than the
  no-profile default
- additive external-agent interoperability surfaces and capability discovery adapters
- remaining unpromoted packet material for interoperability and stronger coordination under
  `specs/027-capability-discovery-and-interoperability/`,
  `specs/028-selective-strong-coordination/`,
  `specs/029-session-first-user-shell/`

Current default runtime limitations to keep in mind:

- the live runtime-proof baseline is still `openai_chatgpt` with `gpt-5.4`; alternate supported
  provider selections need explicit runtime proof before they carry the same claim
- when no runtime routing profile is configured, the fallback runtime router path remains plain
  round-robin
- the config-gated budget-backed control-loop path is landed on `main` and has one bounded
  repeatable live-proof profile, but it is not yet the unqualified no-profile runtime baseline
- the bounded MCP discovery and enforcement surface from `MS-77` is already landed on `main`
- the external-agent follow-on from packet `016` is closed on `main`: accepted delegated HTTP task
  ingress through `POST /api/v1/tasks` is carried through persisted workflow metadata and projected
  onto workflow-level autonomy status as an operator-visible boundary decision with preserved
  provenance and policy continuity

## What Is Planned Next

The completed frontier epics are:

- `MS-45`: task-shape-aware orchestration and dynamic team sizing
- `MS-46`: session restart-resume and distributed operating state
- `MS-47`: step-level intelligence and model routing control loop
- packet `015`: complex multi-agent proof and unified result surfaces (`MS-78` through `MS-94`)
- packet `016`: external-agent boundary continuity and runtime proof (`MS-97` through `MS-100`,
  parent `MS-96`)

Packets `019`, `020`, `021`, `022`, `023`, `024`, `025`, `026`, `030`, and `031` are all landed
on `main`.

No later frontier packet is currently promoted as the next implementation-ready slice. The later
packet material under `specs/027-capability-discovery-and-interoperability/`,
`specs/028-selective-strong-coordination/`, and `specs/029-session-first-user-shell/` remains
draft or pre-spec material and should be deliberately promoted before direct execution.

## Practical Reading Order

If you need to understand the repo quickly:

1. read `docs/current-state.md`
2. read `docs/direction.md`
3. read `README.md`
4. read `specs/023-runtime-truth-and-run-trace/`,
   `specs/024-agent-boundary-security-hardening/`,
   `specs/025-step-level-intelligence-v2/`, and
   `specs/026-first-real-coordinator-subagent-runtime/`,
   `specs/030-session-first-cli-shell/`, and `specs/031-chat-first-cli-loop/`
5. read the remaining unpromoted packet material under
   `specs/027-capability-discovery-and-interoperability/`,
   `specs/028-selective-strong-coordination/`, and `specs/029-session-first-user-shell/` only
   when selecting the next frontier slice
6. read `WORKFLOW.md` and any local-only `docs/linear/` notes only if you are working on the development
   control plane
7. read `ROADMAP.md`, `spec/`, and the rest of `specs/` when you need architectural or
   phase-level detail

## Source Of Truth Rules

- Use this file for the repo's current broad state.
- Use `docs/direction.md` for strategic priority and sequencing.
- Use `specs/023-runtime-truth-and-run-trace/` for the landed packet-023 runtime-truth contract.
- Use `specs/024-agent-boundary-security-hardening/` for the landed packet-024 agent-boundary
  hardening slice.
- Use `specs/025-step-level-intelligence-v2/` for the landed packet-025 step-policy contract.
- Use `specs/026-first-real-coordinator-subagent-runtime/` for the landed packet-026
  coordinator-runtime contract.
- Use `specs/030-session-first-cli-shell/` and `specs/031-chat-first-cli-loop/` for the landed
  CLI shell and chat-loop packet truth on `main`.
- Use `specs/027-capability-discovery-and-interoperability/`,
  `specs/028-selective-strong-coordination/`, and `specs/029-session-first-user-shell/` as
  remaining draft or pre-spec packet material rather than current implementation-ready truth.
- Use `specs/022-durable-workflow-core/` for packet-022 durable workflow ownership.
- Use `scripts/live_runtime_proof_smoke.py` and
  `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md` for the current bounded live-proof
  harness and latest bounded note.
- Use `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md` for the landed
  packet-021 closure and deterministic proof-boundary context.
- Use `docs/plans/2026-03-27-runtime-planning-simplification.md` for the March 27 follow-up that
  simplified live runtime planning and surfaced runtime-owned repair telemetry.
- Use `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` for historical packet-019
  budget-aware live-proof provenance.
- Use `WORKFLOW.md` and any local-only `docs/linear/` notes for development workflow rules.
- Use `spec/` for architecture truth.
- Use `specs/` for implementation packet truth.

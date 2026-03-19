# Mister Smith Current State

Date: March 19, 2026
Status: Active

## Purpose

This is the stable repo-wide overview and document router for Mister Smith.

Use this file when you need one honest answer to:

- what is currently true on `main`
- what is part of the Mister Smith operating system
- what exists in the repo but is not yet the default runtime path
- which document to read next

## Document Roles

| Need | Primary document | Role |
| ---- | ---------------- | ---- |
| Whole-repo overview | `docs/current-state.md` | Current repo and OS state, plus document routing |
| What should happen next | `docs/plans/2026-03-19-central-development-checkpoint.md` | Current forward-development authority, epic order, and scope guardrails |
| Development workflow and watched queue | `WORKFLOW.md`, `docs/linear/LINEAR.md` | Development control plane contract |
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

- `main` is the durable development branch.
- The workspace contains 20 crates: 18 library crates, 1 binary crate, and 1 integration-test
  crate.
- Phases 1 through 10 are landed in the repo as implemented substrate and validation artifacts.
- The current live, honest operator surfaces are:
  - one-shot workflow execution through `mister-smith run` and `POST /api/v1/tasks`
  - autonomy inspection through `mister-smith autonomy list` and `mister-smith autonomy status`
  - bounded same-agent session handling through `POST /api/v1/sessions` and related session routes
- the default runtime path now uses supervised planner and executor lifecycles, a Tokio workflow
  runner, and a ToolBus-backed execution boundary
- A real local provider-backed runtime proof has been completed on the current runtime path using
  `openai_chatgpt` with `gpt-5.4`.
- The watched Symphony queue can be empty without implying a product problem; that queue is part of
  the development workflow, not the OS runtime.

## Important Distinction

Phase completion in this repo means the relevant substrate, crates, tests, and artifact sets are
landed. It does not automatically mean every advanced seam is already wired into the default live
runtime path.

Read the current state in three layers:

1. **Landed in repo**
   - implemented crates, contracts, tests, and validation artifacts exist
2. **Wired into the default runtime path**
   - exercised by the current `mister-smith-app` execution path
3. **Planned next**
   - accepted direction or backlog, but not yet landed or not yet fully wired

## What Is Live And Proven Now

- runtime boot, health probes, and shutdown behavior
- NATS/JetStream plus PostgreSQL local runtime prerequisites
- real workflow submission and terminal completion tracking
- autonomy inspection surfaces keyed by `workflow_id`
- bounded same-agent sessions with stable `session_id` and `coordinator_agent_id`
- supervised planner and executor lifecycles on the default runtime path
- ToolBus-backed workflow step execution on the default runtime path

This is the current OS path that has real end-to-end proof.

## What Exists In The Repo But Is Not Yet The Default Runtime Path

These capabilities are real in the codebase, but the default runtime path does not yet use all of
them end to end:

- provider-neutral `ModelRouter` substrate
- deterministic `MockProvider`
- budget abstractions and router budget hooks
- JetStream KV-backed budget and distributed state control
- additive external-agent interoperability surfaces and capability discovery adapters

Current default runtime limitations to keep in mind:

- the live runtime path is currently fixed to `openai_chatgpt` and `gpt-5.4`
- the default runtime router path is currently a plain round-robin router, not the full
  budget-backed control-loop path
- external delegation metadata, provenance, and operator-visible decisions are landed, but there is
  not yet a first-class external-agent interoperability surface on `main`

## What Is Planned Next

The completed frontier epics are:

- `MS-45`: task-shape-aware orchestration and dynamic team sizing
- `MS-46`: session restart-resume and distributed operating state
- `MS-47`: step-level intelligence and model routing control loop

The next repo-wide planning action is:

- maintain this checkpoint as the forward authority
- write one bounded next SpecKit packet for the remaining differentiation gap between landed
  substrate and proven runtime behavior

The remaining product gap after the March 19 proof cycle is still centered on:

- reliable complex multi-agent execution proof under harder workloads
- final result visibility on operator surfaces
- the exact remaining bounded scope after `MS-77` for external-agent interoperability

This direction is tracked in `docs/plans/2026-03-19-central-development-checkpoint.md`.

## Practical Reading Order

If you need to understand the repo quickly:

1. read `docs/current-state.md`
2. read `README.md`
3. read `docs/plans/2026-03-19-central-development-checkpoint.md`
4. read `WORKFLOW.md` and `docs/linear/LINEAR.md` only if you are working on the development
   control plane
5. read `ROADMAP.md`, `spec/`, and `specs/` when you need the architectural or phase-level detail

## Source Of Truth Rules

- Use this file for the repo's current broad state.
- Use `docs/plans/2026-03-19-central-development-checkpoint.md` for next-step direction.
- Use `WORKFLOW.md` and `docs/linear/LINEAR.md` for development workflow rules.
- Use `spec/` for architecture truth.
- Use `specs/` for implementation packet truth.

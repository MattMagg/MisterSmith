# Implementation Plan: Multi-Turn Same-Agent Conversations

**Branch**: `013-multi-turn-same-agent-conversations` | **Date**: 2026-03-16 | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from
`/specs/013-multi-turn-same-agent-conversations/spec.md`

## Summary

The March 16 runtime proof established a real runtime-backed one-shot workflow path. This feature
adds the minimum honest session layer needed for a retained same-agent back-and-forth conversation:
session creation, session continuation, session inspection, and session end.

The bounded design keeps the existing workflow engine and workflow-scoped autonomy surfaces intact.
Each accepted user turn still becomes one root workflow. The new session layer wraps that workflow
path with a stable `session_id`, a stable session-scoped `coordinator_agent_id`, ordered turn
history, and enough persisted context to resume the conversation after runtime restart.

## Technical Context

**Language/Version**: Rust 1.88.0  
**Primary Dependencies**: Axum 0.8.x, Clap 4.x, sqlx 0.8.x, Tokio 1.49.x, existing EventBus and
TaskRepository seams  
**Storage**: PostgreSQL `tasks.*` tables plus existing root workflow records in `tasks.records`  
**Testing**: `cargo test -p mister-smith-http`, `cargo test -p mister-smith-app`,
`cargo test -p mister-smith-persistence`, `cargo test -p mister-smith-events`,
`cargo build --workspace`, plus a real runtime smoke proof  
**Target Platform**: Local macOS development, Linux runtime parity  
**Project Type**: Rust workspace with binary, HTTP service, persistence layer, and operator CLI  
**Performance Goals**: no duplicate workflow creation for busy or ended sessions, one active turn
per session, inspectable session state without raw database queries  
**Constraints**: preserve the current one-shot workflow path, keep scope to one bounded slice, no
new UI, no multi-user sessions, no concurrent turns, no force-cancel semantics  
**Scale/Scope**: one operator per session, many workflow turns per session, current local runtime
proof remains the base path

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | The packet introduces one explicit session contract rather than redefining workflow/task/autonomy semantics in multiple places. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, and `tasks.md` are written before implementation. |
| III. Phase-Gated Build Order | PASS | This is a post-Phase-10 feature slice on top of completed runtime/autonomy surfaces, not a claim that a new roadmap phase exists. |
| IV. Model-Agnostic Architecture | PASS | The session contract is provider-neutral even though the current proof artifact names `openai_chatgpt` / `gpt-5.4`. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The design adds a session wrapper around existing workflow execution and restart behavior; it does not replace supervision boundaries. |
| VI. Evidence-Based Validation | PASS | Scope is grounded in the current runtime proof, current HTTP/CLI contracts, current persistence schema, and current autonomy projection shape. |
| VII. Explicit Dependency Management | PASS | The plan keeps changes inside existing `core`, `persistence`, `events`, `http`, and `app` seams with concrete file targets. |

## Project Structure

### Documentation (this feature)

```text
specs/013-multi-turn-same-agent-conversations/
├── spec.md                     # Feature specification
├── plan.md                     # This file
├── research.md                 # Research decisions and anti-drift choices
├── data-model.md               # Session, turn, and workflow-link entities
├── quickstart.md               # Validation and operator walkthrough scenarios
├── contracts/
│   └── session-surface.md      # HTTP, CLI, and autonomy linkage contract
└── tasks.md                    # Bounded implementation slice and execution order
```

### Source Code (repository root)

```text
crates/mister-smith-core/
├── src/ids.rs                  # SessionId newtype if promoted to core
├── src/enums.rs                # Session lifecycle enum if promoted to core
└── src/lib.rs                  # Re-exports for shared session identifiers

crates/mister-smith-persistence/
├── migrations/00006_conversation_sessions.sql
├── src/postgres/queries.rs     # Session and turn query helpers
├── src/repository/session.rs   # Session repository facade
└── tests/session_repository_tests.rs

crates/mister-smith-events/
├── src/autonomy.rs             # Optional session linkage on workflow autonomy views
├── src/bus.rs                  # Preserve workflow-scoped accumulation with session linkage
└── tests/autonomy_event_tests.rs

crates/mister-smith-http/
├── src/server.rs               # Session request/response service contracts
├── src/handlers.rs             # create/continue/inspect/end handlers
├── src/routes.rs               # `/api/v1/sessions/...` routes
└── tests/session_http_tests.rs

crates/mister-smith-app/
├── src/conversation.rs         # Session service that wraps RuntimeTaskService
├── src/execution.rs            # Optional session linkage injected into root workflow metadata
├── src/autonomy.rs             # Session-aware rendering helpers
├── src/bootstrap.rs            # Register session routes and service wiring
├── src/main.rs                 # `mister-smith conversation ...` CLI
└── tests/conversation_runtime_tests.rs
```

**Structure Decision**: extend the existing `app` + `http` + `persistence` seams rather than
introducing a new crate or a second runtime path. The session feature is a thin usability layer on
top of the proven workflow engine, not a parallel orchestration stack.

## Design Decisions

### D1: Same-Agent Means Session Envelope, Not Immortal Process Object

**Decision**: define the same-agent guarantee as a stable session-scoped coordinator identity plus
retained persisted context reconstructed for each turn.

**Rationale**: the current runtime already recreates planner state per workflow. A process-pinned
actor contract would be brittle and would fail the restart-resume requirement.

### D2: Each Turn Still Produces One Root Workflow

**Decision**: keep one root `workflow_id` per accepted turn and reuse the current root `task_id`
compatibility surface.

**Rationale**: the proven runtime, task inspection path, and autonomy view are already
workflow-centered. The session layer should wrap them, not replace them.

### D3: Add Explicit Session Tables Instead Of Hiding State In JSON Metadata

**Decision**: add durable session and session-turn persistence instead of overloading
`tasks.records.metadata` as the only source of truth.

**Rationale**: inspection, resume, busy-session conflict detection, and logical end semantics all
need explicit queryable state.

### D4: One Active Turn Per Session In Slice 1

**Decision**: reject concurrent turn submission or session end while a session has an active root
workflow.

**Rationale**: this keeps the usability contract honest and avoids building turn queueing,
reordering, or cancellation semantics into the first slice.

### D5: Keep Autonomy Workflow-Scoped And Add Session Linkage

**Decision**: keep deep autonomy inspection keyed by `workflow_id` and add optional `session_id`,
turn index, and coordinator linkage into workflow autonomy status.

**Rationale**: that is the smallest change that lets operators correlate session state with the
existing autonomy control plane.

### D6: End Means Logical Close, Not Delete

**Decision**: ending a session marks it closed and preserves its history. It does not delete rows
or force-cancel running workflows in the first slice.

**Rationale**: retaining inspectable history is required for operator trust, while force-cancel is
separate lifecycle work.

## Minimal Implementation Slice

### Milestone 1: Session Types And Persistence

**Scope**: add stable session identifiers and explicit PostgreSQL storage for sessions and ordered
turns.

**Validation**:

- `cargo test -p mister-smith-persistence`
- repository tests for create, append, inspect, end, and busy-session guards

### Milestone 2: Session Runtime Wrapper

**Scope**: wrap `RuntimeTaskService` with a session-aware service that materializes retained
context, preserves a stable coordinator identity, and finalizes turn records from workflow
completion.

**Validation**:

- `cargo test -p mister-smith-app`
- runtime integration tests for create plus continue on one session

### Milestone 3: HTTP And CLI Operator Surfaces

**Scope**: add `create`, `continue`, `inspect`, and `end` session surfaces while preserving the
legacy one-shot task endpoints.

**Validation**:

- `cargo test -p mister-smith-http`
- CLI tests for `mister-smith conversation start|continue|inspect|end`

### Milestone 4: Workflow Autonomy Linkage And Live Proof

**Scope**: add session linkage to workflow autonomy status and capture one real runtime smoke proof
of a two-turn same-session conversation.

**Validation**:

- `cargo test -p mister-smith-events`
- `cargo build --workspace`
- live runtime smoke proof with one session, two turns, one stable `coordinator_agent_id`, and two
  distinct `workflow_id` values

## Explicitly Deferred

- shared or multi-user sessions
- concurrent queued turns within one session
- force-end or force-cancel for active sessions
- session branching, transcript editing, or transcript pagination UX
- worker-identity stability as part of the same-agent guarantee
- a brand-new session-scoped autonomy subsystem separate from workflow autonomy

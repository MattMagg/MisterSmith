# Implementation Plan: External-Agent Boundary Continuity And Runtime Proof

**Branch**: `016-external-agent-boundary-continuity-and-runtime-proof` | **Date**: 2026-03-20 |
**Spec**: [spec.md](spec.md)  
**Input**: Feature specification from
`/specs/016-external-agent-boundary-continuity-and-runtime-proof/spec.md`

## Summary

`main` already contains the bounded MCP discovery and enforcement surface, persisted raw delegated
ingress context, workflow-level autonomy inspection, CLI parity, and packet `015` plus `MS-95`
closure for result-surface and failure-visible operator proof. Packet `016` does not invent a new
external-agent program. It freezes one narrower continuity lane: accepted delegated HTTP task
ingress via `POST /api/v1/tasks`, persisted workflow-metadata continuity, compatibility with
retained session continuity rules, workflow-level autonomy inspection at
`GET /api/v1/autonomy/status/{workflow_id}`, and CLI parity through
`mister-smith autonomy status --workflow-id ...`.

## Technical Context

**Language/Version**: Rust 1.88.0  
**Primary Dependencies**: Tokio 1.49.x, existing http, app, agents, events, and security crates  
**Storage**: existing workflow task records, workflow metadata, retained session context, and
durable proof artifacts under `docs/plans/`  
**Testing**: targeted http, app, agents, and event tests; `cargo build --workspace`; durable live
proof note for one accepted ingress run  
**Target Platform**: local macOS development and Linux runtime parity  
**Project Type**: Rust workspace with HTTP ingress, runtime execution, workflow metadata, and
operator inspection surfaces  
**Performance Goals**: honest accepted-ingress continuity proof without widening runtime scope  
**Constraints**: keep packet `015` closed, keep `MS-77` baseline truth, prefer reusing
`external_capability_decisions`, do not widen into all delegated HTTP ingress, and keep live
rejection proof out of scope unless a workflow-backed reject surface already exists  
**Scale/Scope**: one bounded packet on top of landed surfaces

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | The packet is grounded in the `MS-96` decision note, the refreshed checkpoint, current-state, and current code truth. |
| II. Spec-First Design | PASS | `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, `tasks.md`, and `analyze.md` are written before implementation. |
| III. Phase-Gated Build Order | PASS | This is the next bounded post-`015` packet layered on landed packet `015`, `MS-95`, and `MS-77` truth. |
| IV. Model-Agnostic Architecture | PASS | Provider, router, budget, and JetStream KV work stay deferred. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet extends existing runtime and inspection seams rather than replacing them. |
| VI. Evidence-Based Validation | PASS | The packet requires deterministic rejection tests plus one accepted live ingress proof. |
| VII. Explicit Dependency Management | PASS | Shared decision-surface and no-fabrication rules are frozen before parallel implementation lanes start. |

## Project Structure

### Documentation (this feature)

```text
specs/016-external-agent-boundary-continuity-and-runtime-proof/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── analyze.md
```

### Source Code (repository root)

```text
crates/mister-smith-http/
├── src/handlers.rs                      # Delegated POST /api/v1/tasks forwarding and tests
└── src/server.rs                        # Deterministic delegated-ingress rejection coverage

crates/mister-smith-app/
├── src/execution.rs                     # Persisted workflow metadata continuity and recovery rules
├── src/autonomy.rs                      # Workflow-level autonomy projection and CLI parity
├── src/conversation.rs                  # Retained session continuity compatibility rules
├── src/bootstrap.rs                     # Workflow-level autonomy route wiring
└── tests/autonomy_status_tests.rs       # Workflow-level status and CLI parity coverage

crates/mister-smith-events/
├── src/autonomy.rs                      # Operator-visible decision summary typing
├── src/bus.rs                           # Projection rules if event aggregation needs updates
└── tests/autonomy_event_tests.rs        # Projection coverage

docs/plans/
└── 2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md
```

**Structure Decision**: keep packet `016` inside the existing HTTP, app, event, and inspection
seams. No new crate or new operator subsystem is justified.

## Design Decisions

### D1: Freeze The Gap Around Accepted Delegated Task Ingress Only

**Decision**: freeze packet `016` around `POST /api/v1/tasks` rather than silently widening to all
delegated HTTP ingress.

**Rationale**: current repo truth already proves delegated forwarding here, and it is the narrowest
route that matches the unresolved continuity gap.

### D2: Use The Workflow-Level Status Route As The Proof Surface

**Decision**: use `GET /api/v1/autonomy/status/{workflow_id}` plus CLI parity as the runtime proof
surface.

**Rationale**: `MS-95` already establishes this as the supported operator path on `main`.

### D3: Prefer The Existing Operator Decision Surface

**Decision**: prefer reusing `external_capability_decisions`.

**Rationale**: the repo already has this operator-visible summary for bounded MCP and ToolBus
decisions. Packet `016` should not invent a second decision surface without proof that reuse is
insufficient.

### D4: Preserve The No-Fabrication Invariant

**Decision**: keep the rule that raw stored `external_delegation` context does not fabricate an
allowed or rejected operator-visible decision.

**Rationale**: the packet must prove real accepted continuity, not infer it from raw stored input.

### D5: Keep Rejection Proof Deterministic Unless Runtime Truth Justifies More

**Decision**: keep deterministic rejection tests in scope and keep live rejection proof out of
scope unless a workflow-backed reject surface is proven to exist.

**Rationale**: current repo truth clearly supports deterministic rejection coverage but does not
yet prove the same live proof shape for rejected ingress.

### D6: Keep Packet 015 And MS-77 Closed

**Decision**: treat packet `015`, `MS-95`, and `MS-77` as fixed baseline truth.

**Rationale**: packet `016` is continuity-and-proof work on top of landed surfaces, not a reopen
lane.

## Minimal Implementation Slice

### Milestone 1: Freeze Current Truth And Decision-Surface Rules

**Scope**: confirm the no-fabrication invariant, the preferred reuse of
`external_capability_decisions`, and the exact task-ingress/status-route proof shape.

**Validation**:

- targeted app and event tests
- deterministic delegated-ingress forwarding and rejection coverage

### Milestone 2: Carry Accepted Ingress Continuity Through Workflow Metadata And Inspection

**Scope**: extend workflow metadata and workflow-level autonomy projection only as needed to expose
one first-class accepted ingress decision with provenance and policy continuity.

**Validation**:

- targeted app and event tests
- CLI parity checks

### Milestone 3: Capture One Accepted Live Proof Run

**Scope**: run one accepted delegated `POST /api/v1/tasks` request, capture the returned
`workflow_id`, inspect the active status route and CLI output, and record the durable proof note.

**Validation**:

- durable evaluation artifact under `docs/plans/`
- `cargo build --workspace`

## Parallel Symphony Staging Posture

Do **not** stage queue work from this packet.

Allowed future implementation concurrency starts only after the shared packet rules are frozen.

Safe disjoint lanes after the initial freeze:

- delegated-ingress lane:
  `crates/mister-smith-http/src/handlers.rs`,
  `crates/mister-smith-http/src/server.rs`
- workflow-metadata and inspection lane:
  `crates/mister-smith-app/src/execution.rs`,
  `crates/mister-smith-app/src/autonomy.rs`,
  `crates/mister-smith-app/src/bootstrap.rs`,
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- event and proof lane:
  `crates/mister-smith-events/src/autonomy.rs`,
  `crates/mister-smith-events/src/bus.rs`,
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`,
  `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`

Single-owner choke points:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- the active packet-016 evaluation note

## Explicitly Deferred

- packet `015` reopening
- all delegated HTTP ingress beyond `POST /api/v1/tasks`
- live rejection proof unless a workflow-backed reject surface is confirmed
- session-ingress proof as a primary packet surface
- provider-neutral routing, budget, JetStream KV, A2A, mesh, CRDT, or MPST work

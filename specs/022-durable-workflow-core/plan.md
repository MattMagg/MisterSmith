# Implementation Plan: Durable Workflow Core

**Branch**: `022-durable-workflow-core` | **Date**: 2026-04-01 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/022-durable-workflow-core/spec.md`

## Summary

The repo already has live workflow execution, session continuity, restart-resume proof, branch
checkpoint persistence, and KV-plus-SQL durability helpers. What it still lacks is one frozen
durable workflow contract for event history, replay-safe state transitions, lifecycle verbs, and
idempotent effect boundaries. Packet `022` is the implementation packet for that contract on
current `main`.

## Technical Context

**Language/Version**: Rust 1.88.0 plus packet docs and plan artifacts
**Primary Dependencies**: `mister-smith-core`, `mister-smith-agents`,
`mister-smith-persistence`, `mister-smith-app`, `mister-smith-events`, and the existing proof
notes plus the March 28 durable-workflows transfer brief
**Storage**: JetStream KV, PostgreSQL, and existing hybrid durability helpers
**Testing**: markdown lint for packet docs now; later targeted Rust tests around branch
checkpointing, durable replay, lifecycle projections, and the existing restart-resume proof lane
**Target Platform**: local macOS authoring now, Linux runtime parity later
**Project Type**: Rust workspace implementation packet for durable workflow core
**Performance Goals**: deterministic replay from durable history, no duplicate accepted
transitions, bounded replay cost for long-running workflows, and no regression to current session
continuity
**Constraints**: preserve current session continuity; do not widen into
coordinator runtime, interoperability, or strong coordination; treat Temporal and Azure Durable
Functions as comparator semantics only
**Scale/Scope**: one bounded packet for event-history semantics, lifecycle control, effect
boundaries, and minimal compaction plus replay-governance rules

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, the durable-workflows transfer brief, the restart-resume proof note, and the named repo seams. |
| II. Spec-First Design | PASS | `spec.md`, `plan.md`, `design.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, and `tasks.md` are created before any implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | Treats packet `022` as the active bounded implementation packet for durable workflow core and keeps adjacent packets out of scope. |
| IV. Model-Agnostic Architecture | PASS | Defines durable workflow semantics without binding the product to one provider or framework clone. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Extends fault-tolerant recovery semantics over existing restart-resume and checkpoint surfaces rather than replacing them. |
| VI. Evidence-Based Validation | PASS | Keeps deterministic validation, live proof, and open first-slice choices explicitly separate. |
| VII. Explicit Dependency Management | PASS | Names the current durability seams and keeps the candidate future write set bounded. |
| VIII. Clean Closure And Resumability | PASS | Leaves a runnable packet with clear kickoff checks, bounded tasks, and explicit proof boundaries. |

## Implementation Kickoff Check

Run this quick check at the start of implementation. It is a narrowing pass, not a packet rewrite
gate.

1. Re-read `docs/current-state.md`, `docs/direction.md`,
   `docs/research-output/analysis/2026-03-28-durable-workflows-transfer-brief.md`, and
   `docs/plans/2026-03-19-session-restart-resume-live-proof.md`.
2. Reconfirm whether earlier in-flight packet work changed any of these seams materially:
   - `crates/mister-smith-agents/src/branch_checkpoint.rs`
   - `crates/mister-smith-persistence/src/kv/state.rs`
   - `crates/mister-smith-persistence/src/hybrid/manager.rs`
   - `crates/mister-smith-persistence/src/repository/task.rs`
   - `crates/mister-smith-app/src/conversation.rs`
   - `crates/mister-smith-app/src/execution.rs`
   - `crates/mister-smith-events/src/autonomy.rs`
3. If current `main` materially contradicts packet `022`, update the packet docs before code.
4. Then choose the first actual implementation slice and its validation set.

## Project Structure

```text
specs/022-durable-workflow-core/
├── spec.md
├── plan.md
├── design.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── durable-workflow-contract.md
├── checklists/
│   ├── requirements.md
│   └── durability.md
└── tasks.md

Candidate future write set after refresh:
crates/mister-smith-agents/
├── src/branch_checkpoint.rs
└── src/orchestrator.rs

crates/mister-smith-persistence/
├── src/kv/state.rs
├── src/hybrid/manager.rs
└── src/repository/task.rs

crates/mister-smith-app/
├── src/conversation.rs
└── src/execution.rs

crates/mister-smith-events/
└── src/autonomy.rs

crates/mister-smith-http/
└── src/handlers.rs
```

**Structure Decision**: Keep this packet self-contained under
`specs/022-durable-workflow-core/` and treat the Rust files above as the expected initial write
seams for implementation.

## Design Decisions

### D1: Event history is the semantic source of truth

**Decision**: The future packet should freeze one repo-native workflow history contract and treat
durable projections as derived state from that accepted history.

**Rationale**: The durable-workflows transfer brief and the current checkpoint seams both point to
event history plus replay as the missing substrate contract. The packet should copy the semantic
strength of Temporal-style replay, not the product shape.

### D2: Effect correctness stays separate from state-transition correctness

**Decision**: The future packet should define explicit effect intent and effect completion
boundaries and keep those guarantees separate from accepted workflow state transitions.

**Rationale**: Current repo truth already has persistence, transport, and dedup-related pieces, but
the brief is explicit that broker dedup alone is not effect correctness.

### D3: Current session continuity is preserved, not redesigned

**Decision**: The future packet should layer durable workflow semantics under the existing session
and restart-resume surfaces rather than replacing them.

**Rationale**: The March 19 live proof already shows preserved `session_id`,
`coordinator_agent_id`, and resumed lineage. Packet `022` should not reopen that baseline.

### D4: Open first-slice choices stay explicit until resolved

**Decision**: Any packet choice that is not yet exact stays explicit and gets resolved in the
first bounded implementation slice.

**Rationale**: The packet should stay honest about what is still open without creating a fake stop
condition.

### D5: Cross-crate tradeoffs live in a dedicated design note

**Decision**: Keep one `design.md` in the packet so the first coding session can review write
seams, invariants, and refresh-sensitive tradeoffs in one place.

**Rationale**: This packet crosses agents, persistence, app, events, and HTTP surfaces, so the
architecture note should be explicit before implementation starts.

## Minimal Implementation Slice

### Milestone 0: Implementation Kickoff

**Scope**: Reconfirm current truth, touched seams, and the first-slice choices that must be fixed
before code branches widen.

**Validation**:

- kickoff check completed and recorded before write lanes widen
- any real contradiction with current `main` fixed before code starts

### Milestone 1: Freeze the durable history and lifecycle contract

**Scope**: Define one frozen durable history model, one lifecycle vocabulary, and one consistent
projection rule across task, session, and autonomy surfaces.

**Validation**:

- contract and data model agree on durable entities and lifecycle meanings
- first-slice lifecycle behavior stays honest about `applied`, `noop`, and `deferred` outcomes and
  does not overclaim live runner control
- spec, plan, and tasks stay inside packet `022` scope without drifting into adjacent packets

### Milestone 2: Freeze effect boundaries and recovery posture

**Scope**: Define effect intent and completion boundaries, replay-safe recovery posture, and the
first bounded operator-facing meaning of repeated effects or repeated lifecycle commands.

**Validation**:

- effect-boundary contract stays distinct from exactly-once state-transition claims
- repeated lifecycle commands keep one durable outcome vocabulary without pretending packet `022`
  already stops or resumes a live runner
- replay and retry scenarios remain consistent with the existing restart-resume baseline

### Milestone 3: Freeze compaction and replay-governance posture

**Scope**: Define the first bounded compaction rule, version-safe replay expectations, and the
replay-regression gate posture for future implementation work.

**Validation**:

- the packet defines bounded replay growth without widening into a storage-platform redesign
- proof-boundary language stays honest about current validation and future live-proof work

## Parallel Staging Posture

Use only after the pre-implementation refresh is complete and the first coding slice is selected.

- Blocking freeze before any parallel lanes: durable history model, lifecycle vocabulary, and
  effect-boundary contract
- Allowed disjoint lanes after the freeze:
  - persistence and history-model lane: `mister-smith-persistence`
  - orchestration and checkpoint lane: `mister-smith-agents`
  - projection lane: `mister-smith-app` and `mister-smith-events`
- Single-owner choke points:
  - `crates/mister-smith-agents/src/branch_checkpoint.rs`
  - `crates/mister-smith-agents/src/orchestrator.rs`
  - `crates/mister-smith-persistence/src/kv/state.rs`
  - `crates/mister-smith-persistence/src/hybrid/manager.rs`
  - `crates/mister-smith-app/src/conversation.rs`
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-http/src/handlers.rs`

## Explicitly Deferred

- coordinator-runtime or real subagent-runtime expansion
- interoperability or capability-discovery work
- strong coordination, consensus, CRDT, or MPST work
- any new live-default truth claim until implementation and proof are actually completed

# Implementation Plan: Profile-Aware Predictive Runtime Supervision

**Branch**: `021-profile-aware-predictive-runtime-supervision` | **Date**: 2026-03-27 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/021-profile-aware-predictive-runtime-supervision/spec.md`

## Summary

`main` already contains the Phase 10 Guard/Advisor substrate, packet `020` verifier and repair
lineage, and the March 27 runtime-owned repair telemetry pass. This packet freezes the next
bounded frontier phase: make the supported runtime ingress truly profile-aware and predictive by
moving beyond provider-only supervision targets, adding bounded performance fingerprints, and
surfacing supervisory evidence as first-class operator state.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned docs and operator-console TypeScript
**Primary Dependencies**: `mister-smith-core`, `mister-smith-agents`, `mister-smith-app`,
`mister-smith-events`, existing stream-monitor and supervision surfaces, and bounded JetStream KV
support for profile fingerprints through `mister-smith-persistence`
**Storage**: existing task/autonomy metadata plus one bounded fingerprint store over current
JetStream KV primitives; structured summaries only, no duplicated raw-transcript store, and no
new benchmark or training infrastructure
**Testing**: targeted Rust tests for supervision wiring and fingerprint behavior, current
operator-console checks when UI files move, markdown lint, and diff hygiene
**Target Platform**: local macOS development and Linux runtime parity for the shipped app binary
**Project Type**: Rust workspace packet with bounded UI and router-doc sync
**Performance Goals**: intervene locally before graph-wide restart, keep current happy-path
latency stable when no supervision fires, and make supervisory evidence inspectable without log
archaeology
**Constraints**: no topology rewrite, no no-profile routing defaultization, no CKM or PPO
training, no CRDT coordination, no queue staging, and no new benchmark claims
**Scale/Scope**: one bounded runtime packet focused on supported ingress supervision plus
operator-facing evidence

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, packet `020` closure, March 27 runtime proof notes, and current code surfaces. |
| II. Spec-First Design | PASS | Packet `021` freezes the next bounded phase before any implementation slice is staged. |
| III. Phase-And-Packet-Gated Delivery | PASS | Chooses one frontier seam after packet `020` rather than mixing routing, topology, and coordination futures. |
| IV. Model-Agnostic Architecture | PASS | Targets runtime supervision semantics, not provider-specific behavior or model lock-in. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Extends local intervention and profile-aware guidance above the existing restart substrate. |
| VI. Evidence-Based Validation | PASS | Requires targeted tests, explicit proof boundaries, and replayable runtime evidence for fingerprints. |
| VII. Explicit Dependency Management | PASS | Keeps the write set bounded to existing runtime, persistence, event, and UI seams. |
| VIII. Clean Closure And Resumability | PASS | Packet freeze lands as durable spec artifacts plus a current-state sync on `main`. |

## Project Structure

```text
specs/021-profile-aware-predictive-runtime-supervision/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── analyze.md

crates/mister-smith-core/
├── src/autonomy.rs
└── src/lib.rs

crates/mister-smith-agents/
├── src/orchestrator.rs
├── src/profile.rs
├── src/guard.rs
├── src/intervention.rs
└── tests/

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
└── tests/

crates/mister-smith-events/
├── src/autonomy.rs
├── src/bus.rs
└── tests/

crates/mister-smith-persistence/
├── src/kv/
├── src/repository/
└── tests/

apps/operator-console/
├── src/views/RunsView.tsx
└── src/types.ts
```

## Design Decisions

### D1: Supervision scope must track runtime structure, not stay provider-only

**Decision**: once graph, branch, or node context exists on the supported ingress, predictive
supervision should target that runtime structure instead of remaining locked to
`GuardTarget::Provider(...)`.

**Rationale**: provider-local supervision is too coarse to express the branch-local recovery and
operator evidence already supported by the rest of the runtime.

### D2: Profile fingerprints are bounded advisory memory, not a learned control plane

**Decision**: add a bounded fingerprint surface backed by replayable runtime evidence and explicit
expiry rather than introducing an opaque training or inference loop.

**Rationale**: AWorld-style profiling is the strongest frontier signal, but this packet should use
it as advisory reinforcement for existing Guard decisions, not as a second orchestration engine.
Fingerprints are stored as structured summaries in JetStream KV, with source references instead of
duplicated raw transcripts.

### D3: Packet `020` repair lineage stays canonical for verifier-driven repair

**Decision**: packet `021` augments the runtime with predictive supervision and fingerprints, but
it does not replace verifier-gated repair semantics or create a conflicting result contract.

**Rationale**: packet `020` is already landed on `main`; the new packet must compose with it,
not reopen it.

### D4: Supervisory evidence belongs in current task, autonomy, and run-detail surfaces

**Decision**: render profile, guard, and intervention evidence through the existing result and run
inspection surfaces rather than inventing a new dashboard mode.

**Rationale**: the operator already looks at task result, autonomy status, and the run detail.
Adding a new surface would widen scope without adding new runtime truth.

## Minimal Implementation Slice

### Milestone 1: Freeze the shared supervision contract

**Scope**: define packet `021` entities, update the repo router, and freeze the operator-visible
evidence shape.

**Validation**:

- packet `021` artifacts are complete
- `docs/current-state.md` no longer claims that no post-packet-020 packet is frozen

### Milestone 2: Wire predictive supervision onto the supported ingress

**Scope**: move from provider-only runtime supervision to branch- and node-aware predictive
supervision where graph context exists.

**Validation**:

- targeted `mister-smith-agents` and `mister-smith-app` tests for profile snapshots, guard
  decisions, local interventions, and happy-path fallback

### Milestone 3: Add bounded fingerprints and operator evidence

**Scope**: seed advisory fingerprints from replayable evidence, surface them in result views, and
render the latest supervisory evidence in the operator console.

**Validation**:

- deterministic fingerprint fixtures prove one reinforced intervention path
- task, autonomy, and operator-console surfaces remain consistent

## Parallel Staging Posture

- Blocking freeze before any parallel lanes: shared value objects, result-view contract, and
  router-doc sync
- Allowed disjoint lanes after the freeze:
  - fingerprint and Guard policy lane: `mister-smith-core`, `mister-smith-agents`,
    `mister-smith-persistence`
  - runtime projection lane: `mister-smith-app`, `mister-smith-events`
  - operator-console lane: `apps/operator-console/`
- Single-owner choke points:
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-agents/src/orchestrator.rs`
  - `crates/mister-smith-core/src/autonomy.rs`
  - `docs/current-state.md`
  - any active proof note under `docs/plans/`

## Explicitly Deferred

- default-runtime activation of packet `019` routing when no profile is configured
- reopening `MS-110` adaptive-topology work without new evidence
- CKM-based cognitive coordination, consensus-free debate, or RL-trained intervention policies
- CRDT coordination, MPST protocol verification, or event-triggered consensus
- broad operator-console redesign beyond bounded run-detail evidence

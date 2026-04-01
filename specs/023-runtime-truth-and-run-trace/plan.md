# Implementation Plan: Runtime Truth And Run Trace

**Branch**: `023-runtime-truth-and-run-trace` | **Date**: 2026-04-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`specs/023-runtime-truth-and-run-trace/spec.md`

## Scaffold Status

This is a scaffold plan written ahead of upstream packet completion.

- It is meant to speed later packet work.
- It does not authorize immediate implementation.
- It must be revised against then-current repo truth before any future `/speckit.implement`.

## Summary

Packet `023` freezes one bounded contract for honest runtime truth and run traces: one trace root
per workflow run, one shared proof-boundary model, one placeholder-step honesty rule, and one
consistent projection story across task, session, autonomy, and operator surfaces. It does not
reopen packet `022` lifecycle ownership, and it does not widen into UI polish, generic
observability-platform work, or coordinator-runtime scope.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned packet docs
**Primary Dependencies**: `mister-smith-core`, `mister-smith-events`, `mister-smith-app`,
`mister-smith-transport`, current result and autonomy surfaces, and current proof notes under
`docs/plans/`
**Storage**: Existing task/autonomy metadata and packet-owned docs only for this scaffold; future
implementation may consume packet `022` durable-history surfaces once they are frozen
**Testing**: Packet-writing checks now, then targeted Rust and smoke-harness checks later once
implementation is revalidated
**Target Platform**: Current Mister Smith runtime-backed task path and its existing operator
inspection surfaces
**Project Type**: Rust workspace packet with state-bearing docs and future projection-contract work
**Performance Goals**: Honest operator understanding in one inspection pass; no broadened runtime
claim surface without corresponding proof
**Constraints**: Keep packet `022` as lifecycle/history owner; preserve packet `019` and `020`
live-proof split versus packet `021` deterministic-only proof; treat OpenTelemetry and W3C docs as
taxonomy guidance only; do not widen scope
**Scale/Scope**: One scaffold packet focused on naming, taxonomy, proof boundaries, and revision
gates

## Constitution Check

*GATE: Must pass before any future implementation begins. Re-check after upstream packet work lands.*

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, the packet `023` dossier, and current runtime seams. |
| II. Spec-First Design | PASS | This scaffold exists to define the packet before any code work. |
| III. Phase-And-Packet-Gated Delivery | PASS | Packet `023` stays bounded and explicitly depends on packet `022` ownership instead of reopening it. |
| IV. Model-Agnostic Architecture | PASS | The packet describes proof boundaries and trace taxonomy, not provider-specific behavior. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | No fault-tolerance guarantees are widened here; current runtime boundaries stay explicit. |
| VI. Evidence-Based Validation | PASS | The scaffold preserves live-proof versus deterministic-only boundaries and requires later revalidation. |
| VII. Explicit Dependency Management | PASS | Upstream dependency on packet `022` and existing packet `019` / `020` / `021` proof notes is explicit. |
| VIII. Clean Closure And Resumability | PASS | The artifact set is designed for later cold-start reuse and revision. |

## Project Structure

### Documentation (this feature)

```text
specs/023-runtime-truth-and-run-trace/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── run-trace-proof-boundary-contract.md
├── checklists/
│   ├── requirements.md
│   └── runtime-truth-proof-boundary.md
├── tasks.md
└── analyze.md
```

### Source Code (future implementation targets only)

```text
crates/mister-smith-transport/
└── src/envelope.rs

crates/mister-smith-core/
└── src/autonomy.rs

crates/mister-smith-events/
└── src/bus.rs

crates/mister-smith-app/
├── src/execution.rs
├── src/autonomy.rs
└── src/conversation.rs

apps/operator-console/
└── src/views/
```

**Structure Decision**: This scaffold writes only packet artifacts now. The runtime and projection
paths above are listed as future implementation targets so a later session does not need to
rediscover the likely write set, including the existing session-facing conversation surface.

## Design Decisions

### D1: Packet `023` owns naming and proof-boundary projection, not lifecycle semantics

**Decision**: Packet `023` freezes truthful naming, trace taxonomy, and proof-boundary projection
only. Packet `022` remains the owner of durable lifecycle, event-history, compaction, and effect
boundaries.

**Rationale**: The dossier and packet dependency map explicitly place packet `023` after packet
`022` and keep it narrower than durable-workflow semantics.

### D2: Placeholder step completion is a first-class non-proof state

**Decision**: The scaffold treats the current `workflow.execute_step` payload-echo behavior as a
placeholder completion boundary that can prove orchestration-substrate flow but not grounded task
work.

**Rationale**: Current runtime code marks `workflow.execute_step` payloads `completed` at the
`tool_bus` boundary without proving grounded execution. The packet must freeze this honesty rule.

### D3: Packet `019` and `020` live proof remain separate from packet `021` deterministic proof

**Decision**: The scaffold preserves the current proof split instead of collapsing all runtime
evidence into one broad “live” label.

**Rationale**: `docs/current-state.md` and the packet `021` closure and live-evaluation notes make
that split explicit and current.

### D4: External tracing standards are taxonomy references, not current emitted-truth claims

**Decision**: Use OpenTelemetry traces, context propagation, and W3C Trace Context to shape the
taxonomy, but do not claim the repo already emits a complete span model.

**Rationale**: The dossier makes this a hard boundary and prevents fake observability claims.

## Minimal Scaffold Slice

### Milestone 1: Freeze the scaffold contract

**Scope**: Write `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/run-trace-proof-boundary-contract.md`, and both checklists.

**Validation**:

- packet `023` artifact set exists under `specs/023-runtime-truth-and-run-trace/`
- every artifact repeats the scaffold-only and revalidation-required posture

### Milestone 2: Freeze the future implementation posture

**Scope**: Write `tasks.md` as a future-only scaffold and create one durable analysis artifact that
highlights top gaps before later implementation.

**Validation**:

- `tasks.md` starts with a blocking revalidation task before any code work
- analysis output is preserved in a reusable artifact

## Parallel Staging Posture

- Blocking first: spec contract, revalidation gate, and proof-boundary wording
- Allowed later as separate future lanes once the packet is revised for implementation:
  - transport and core taxonomy lane
  - events and app projection lane
  - operator projection lane
- Shared-write choke points for future implementation:
  - `crates/mister-smith-app/src/execution.rs`
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-events/src/bus.rs`
  - `docs/current-state.md`

## Deferred Until Revalidation

- any code changes in runtime or operator surfaces
- any attempt to rewrite repo router docs as if packet `023` were already the active
  implementation-ready post-packet-021 packet
- any decision that depends on the final packet `022` lifecycle and history contract language
- any broader observability, tracing export, or runtime-coordinator work

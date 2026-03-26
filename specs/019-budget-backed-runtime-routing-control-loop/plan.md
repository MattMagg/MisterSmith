# Implementation Plan: Budget-Backed Runtime Routing Control Loop

**Branch**: `019-budget-backed-runtime-routing-control-loop` | **Date**: 2026-03-26 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/019-budget-backed-runtime-routing-control-loop/spec.md`

## Summary

This packet activates the already-landed router and budget substrate on the runtime-backed task
path. It keeps today's single-provider boot path as the explicit fallback while adding a typed
runtime routing profile, bounded multi-provider bootstrap, one JetStream-backed budget store, and
budget-aware routing evidence.

## Technical Context

**Language/Version**: Rust 1.88.0 plus current runtime shell/proof support
**Primary Dependencies**: `mister-smith-config`, `mister-smith-app`, `mister-smith-llm`, current
JetStream transport/runtime bootstrap, and any existing packet-018 proof harness that lands first
**Testing**: targeted config/app/router tests, `cargo clippy --workspace -- -D warnings`,
`cargo build --workspace`, and proof-harness/evidence updates only if the environment can prove the
new path honestly
**Target Platform**: local macOS and Linux parity for the existing app binary
**Constraints**: no new providers, no external-agent widening, no operator-console redesign

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in current-state, the March 21 checkpoint, packet `017`, and the current router/budget code. |
| II. Spec-First Design | PASS | This packet freezes the multi-provider runtime profile before implementation. |
| III. Phase-Gated Build Order | PASS | Builds directly on landed routing/budget substrate instead of inventing a replacement. |
| IV. Model-Agnostic Architecture | PASS | Uses shipped-provider tiers and existing router policies rather than a new provider silo. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Preserves supervision/provenance and explicit fallback behavior. |
| VI. Evidence-Based Validation | PASS | Requires targeted runtime tests and honest proof-boundary reporting. |
| VII. Explicit Dependency Management | PASS | Write set stays bounded to config, runtime bootstrap, router/budget integration, and state docs. |

## Project Structure

```text
specs/019-budget-backed-runtime-routing-control-loop/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── analyze.md

crates/mister-smith-config/
├── src/types.rs
├── src/loader.rs
└── src/validation.rs

crates/mister-smith-app/
└── src/execution.rs

crates/mister-smith-llm/
├── src/budget.rs
├── src/router.rs
└── tests/
```

## Design Decisions

### D1: Treat runtime routing profile as config, not app-local ad hoc logic

The next phase must not bury tier labels, budget roots, or routing policy branches directly inside
`execution.rs`.

### D2: Preserve one-provider fallback while activating the new path

The budget-backed control loop is the new bounded activation surface, but the current single
provider boot path remains the safe default until proof and operator evidence catch up.

### D3: Reuse the existing router and budget primitives

`ModelRouter`, `CascadePolicy`, and `BudgetEnforcer` are already the right substrate. The packet
should wire them into the runtime instead of inventing a second routing engine.

## Minimal Implementation Slice

### Milestone 1: Config shape and runtime-profile parsing

Validation:

- config tests for defaults, parsing, validation, and fallback behavior

### Milestone 2: Runtime bootstrap and budget-store wiring

Validation:

- targeted app tests for single-provider fallback and bounded multi-provider boot
- router/budget tests for the production store adapter

### Milestone 3: Routing evidence and proof-boundary refresh

Validation:

- targeted task/autonomy evidence checks
- proof-harness or durable evidence update only if the new path can be exercised honestly

## Explicitly Deferred

- new provider implementations
- broad multi-tenant budget governance
- dynamic control-plane mutation of routing policy after boot
- operator-console redesign
- any claim that the fully budget-backed path is universally live-proven before evidence exists

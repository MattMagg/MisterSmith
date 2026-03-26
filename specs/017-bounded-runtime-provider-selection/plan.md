# Implementation Plan: Bounded Runtime Provider Selection

**Branch**: `017-bounded-runtime-provider-selection` | **Date**: 2026-03-26 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/017-bounded-runtime-provider-selection/spec.md`

## Summary

This packet removes one current-state limitation from the shipped runtime path without widening into
budget control or external-agent follow-on work. The runtime-backed task path will read
`provider_kind` and `model_id` from framework configuration, default to today's
`openai_chatgpt` / `gpt-5.4` path, support the providers the current app binary actually ships,
and preserve task/session/autonomy provenance.

## Technical Context

**Language/Version**: Rust 1.88.0
**Primary Dependencies**: existing `mister-smith-config`, `mister-smith-app`, and
`mister-smith-llm` crates
**Testing**: targeted config and app tests plus `cargo clippy --workspace -- -D warnings` and
`cargo build --workspace`
**Target Platform**: local macOS and Linux parity for the existing app binary
**Constraints**: no new routing-policy program, no queue staging, no external-agent widening

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in current-state, the March 21 checkpoint, and current app/config code. |
| II. Spec-First Design | PASS | This packet and the planning note freeze scope before implementation. |
| III. Phase-Gated Build Order | PASS | The slice builds on landed Phase 9 and packet-015/016 truth. |
| IV. Model-Agnostic Architecture | PASS | The packet promotes existing provider-neutral substrate without widening into new routing programs. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Supervision and provenance remain unchanged. |
| VI. Evidence-Based Validation | PASS | Targeted config/app tests plus workspace build. |
| VII. Explicit Dependency Management | PASS | Write set stays bounded to config, app runtime wiring, and state-bearing docs. |

## Project Structure

```text
specs/017-bounded-runtime-provider-selection/
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
├── src/bootstrap.rs
├── src/execution.rs
├── src/conversation.rs
└── src/auth.rs

docs/
└── current-state.md
```

## Design Decisions

### D1: Add typed runtime `llm` config instead of ad hoc env reads

This keeps provider/model selection in the normal framework-config path and avoids creating a
parallel config seam inside `execution.rs`.

### D2: Support only providers the current binary ships

This keeps the slice honest. `openai_chatgpt`, `claude_subscription`, and `mock` are in scope.
`openai` and `anthropic` remain explicit unsupported selections in this binary.

### D3: Preserve current default and metadata semantics

The packet changes configurability, not the meaning of operator-visible metadata.

## Minimal Implementation Slice

### Milestone 1: Config shape and selection helpers

Validation:

- config unit tests for defaults and env overlay

### Milestone 2: Runtime bootstrap and metadata wiring

Validation:

- targeted `mister-smith-app` tests for runtime selection behavior

### Milestone 3: State-bearing doc refresh

Validation:

- `cargo build --workspace`

## Explicitly Deferred

- multi-provider fan-out
- routing-policy selection
- budget enforcer activation
- JetStream KV budget store wiring
- API-key provider support in the current app binary

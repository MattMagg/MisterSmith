# Implementation Plan: Live Runtime Proof Smoke Harness

**Branch**: `018-live-runtime-proof-smoke-harness` | **Date**: 2026-03-26 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/018-live-runtime-proof-smoke-harness/spec.md`

## Summary

This packet turns the current manual default-path live proof into a repeatable repo-owned smoke
harness. The harness will validate the local prerequisites honestly, run one real task through
`POST /api/v1/tasks`, assert the current runtime/autonomy proof markers, and persist artifacts in a
predictable location.

## Technical Context

**Language/Version**: Python 3 plus current repo runtime shell surfaces
**Primary Dependencies**: existing `scripts/` support plus current runtime HTTP/CLI surfaces
**Testing**: script helper tests, one honest smoke run if environment permits, and Rust validation
only if the slice touches Rust code
**Target Platform**: local macOS and Linux parity for the existing app binary
**Constraints**: no alternate-provider proof, no new runtime features, no queue-stage widening

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in current-state, the March 21 checkpoint, and the March 19 live proof note. |
| II. Spec-First Design | PASS | This packet freezes scope before implementation. |
| III. Phase-Gated Build Order | PASS | Builds on landed runtime/task/session/autonomy surfaces already on `main`. |
| IV. Model-Agnostic Architecture | PASS | Exercises the current shipped default path without widening routing policy. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Focuses on proof repeatability, not feature redesign. |
| VI. Evidence-Based Validation | PASS | Uses the March 19 live evidence plus deterministic helper tests and a real smoke run when available. |
| VII. Explicit Dependency Management | PASS | Initial write set stays in scripts, docs, and existing runtime proof surfaces. |

## Project Structure

```text
specs/018-live-runtime-proof-smoke-harness/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── analyze.md

scripts/
├── live_runtime_proof_smoke.py
└── tests/test_live_runtime_proof_smoke.py

docs/plans/
└── artifacts/<smoke-run>/
```

## Design Decisions

### D1: Script the already-proven default path instead of inventing a new proof flow

The harness should mirror the documented March 19 proof steps closely so the new automation stays
honest.

### D2: Treat NATS/JetStream verification as an evidence question, not a fixed endpoint question

If `8222/healthz` is flaky, the script should use a supported and truthful surface such as Docker
health, logs, or runtime stream initialization rather than pretending the probe is authoritative.

### D3: Keep artifact output predictable

The value of the harness is repeatability. It should write task, autonomy, and health evidence to
one predictable artifact directory.

## Minimal Implementation Slice

### Milestone 1: Harness scaffolding and helper tests

Validation:

- helper/unit tests for request building, response assertions, or artifact-path handling

### Milestone 2: End-to-end smoke path

Validation:

- one local smoke run if the environment is available

### Milestone 3: State/doc refresh

Validation:

- update docs only where repo truth changes from manual-only proof to repeatable harness support

## Explicitly Deferred

- alternate-provider proof
- session-route proof expansion
- budget-state integration
- runtime feature changes unrelated to proof repeatability

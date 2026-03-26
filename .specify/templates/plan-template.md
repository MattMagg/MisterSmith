# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

## Summary

[Summarize the bounded gap, the chosen approach, and the preserved baseline]

## Technical Context

**Language/Version**: [e.g., Rust 1.88.0]
**Primary Dependencies**: [workspace crates, scripts, or external libraries touched]
**Storage**: [JetStream, PostgreSQL, files, N/A]
**Testing**: [targeted validation stack, broader build checks, proof guidance]
**Target Platform**: [e.g., local macOS and Linux parity]
**Project Type**: [e.g., Rust workspace packet, workflow packet, docs-only slice]
**Performance Goals**: [latency, reliability, benchmark, or operator-facing target]
**Constraints**: [explicit no-go areas, invariants, environment constraints]
**Scale/Scope**: [bounded scope statement]

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | [grounded source docs and code truth] |
| II. Spec-First Design | PASS | [packet artifacts precede implementation] |
| III. Phase-And-Packet-Gated Delivery | PASS | [current repo truth and active checkpoint respected] |
| IV. Model-Agnostic Architecture | PASS | [or N/A, with reason] |
| V. Erlang/OTP-Style Fault Tolerance | PASS | [or N/A, with reason] |
| VI. Evidence-Based Validation | PASS | [validation and proof boundary] |
| VII. Explicit Dependency Management | PASS | [bounded write set and dependency mapping] |
| VIII. Clean Closure And Resumability | PASS | [closure path and durable notes] |

## Project Structure

```text
specs/[###-feature-name]/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── analyze.md

[repo write-set tree rooted at real files and directories]
```

## Design Decisions

### D1: [decision title]

[decision rationale grounded in current repo truth]

### D2: [decision title]

[decision rationale grounded in current repo truth]

### D3: [decision title]

[decision rationale grounded in current repo truth]

## Minimal Implementation Slice

### Milestone 1: [freeze or bootstrap milestone]

Validation:

- [targeted proof or test]
- [targeted proof or test]

### Milestone 2: [primary implementation milestone]

Validation:

- [targeted proof or test]
- [targeted proof or test]

### Milestone 3: [evidence or follow-on milestone]

Validation:

- [targeted proof or test]
- [targeted proof or test]

## Parallel Staging Posture

Use only when the packet benefits from bounded parallel work.

- Blocking freeze before any parallel lanes: [task or milestone]
- Allowed disjoint lanes after the freeze: [lane and write set]
- Single-owner choke points: [files, docs, or control-plane surfaces]

## Explicitly Deferred

- [explicitly deferred scope]
- [explicitly deferred scope]
- [claim or proof boundary that must remain deferred]

# Implementation Plan: Chat-First CLI Loop

**Branch**: `031-chat-first-cli-loop` | **Date**: 2026-04-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/031-chat-first-cli-loop/spec.md`

## Summary

Packet `030` already moved Mister Smith toward a session-first CLI shell: recent-first startup,
resume flows, durable session continuity, stored control posture, and in-session slash commands
are grounded on current `main`.

The bounded gap is no longer entry or resume. The active session still behaves like task
submission followed by detached inspection. Packet `031` freezes the next honest slice: keep the
user inside one live CLI conversation loop, surface turn-state inline, reopen retained work back
into that loop, and preserve runtime-truth, proof-boundary, and supervised-autonomy honesty while
keeping runtime and admin machinery secondary.

## Technical Context

**Language/Version**: Rust 1.88.0
**Primary Dependencies**: `mister-smith-app`, `mister-smith-http`, current durable session seams,
packet `023` runtime-truth projections, packet `025` step-policy summaries, and packet `030`
session-first CLI shell behavior
**Storage**: Existing PostgreSQL-backed retained session store plus the current runtime session
service; no new packet-owned store
**Testing**: Targeted Rust coverage in `mister-smith-app` and `mister-smith-http`, packet-doc
validation, markdown lint, and diff hygiene; no new live-proof claim in this workflow
**Target Platform**: Local macOS development with Linux parity for the shipped CLI
**Project Type**: Rust workspace packet for a bounded CLI session-loop experience
**Performance Goals**: Preserve session continuity, show turn-state inline without raw log
archaeology, and keep degraded or busy states readable without forcing the user out of the loop
**Constraints**: CLI-only, preserve packet `030` foundations, preserve the current durable session
model, do not widen into GUI parity or repo workflow tooling, and keep live-proof claims explicit
**Scale/Scope**: One bounded packet on the existing CLI session shell and session-facing runtime
surfaces

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, `docs/direction.md`, packet `030`, and the current CLI/session code seams. |
| II. Spec-First Design | PASS | Packet `031` now has `spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `tasks.md`, and `analyze.md` before implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | The slice extends landed session-shell foundations instead of reopening earlier packets or widening into a broader shell program. |
| IV. Model-Agnostic Architecture | PASS | The packet changes the CLI session experience, not provider-specific runtime contracts. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Busy, degraded, and ended-session states remain explicit instead of being hidden by a chat-first surface. |
| VI. Evidence-Based Validation | PASS | The packet keeps deterministic artifact validation and future runtime proof clearly separated. |
| VII. Explicit Dependency Management | PASS | The plan names the current CLI, session, and runtime-truth seams and keeps the write set bounded to them. |
| VIII. Clean Closure And Resumability | PASS | The packet bundle is complete and resumable before implementation work begins. |

## Project Structure

```text
specs/031-chat-first-cli-loop/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── analyze.md
├── contracts/
│   └── cli-session-loop-contract.md
├── checklists/
│   ├── requirements.md
│   └── cli-conversation.md
└── tasks.md

crates/mister-smith-app/
├── src/main.rs
└── src/conversation.rs

crates/mister-smith-http/
└── src/server.rs
```

## Design Decisions

### D1: Extend packet 030 instead of replacing it

Packet `031` starts after entry and resume are already session-first. It owns the live in-session
loop, not another startup-home redesign.

### D2: One active session must read as one continuous conversation

The user should stay inside one session while turns are accepted, in progress, completed, failed,
or blocked. The loop should not require detached inspection output just to understand current
state.

### D3: Resumed work must reopen as conversation, not as archive

Resume flows should land directly in a usable live session that preserves retained history and
stored control posture instead of reopening prior work as a static artifact.

### D4: Steering and truth notices stay inside the loop

Model, permissions, config, status, and MCP posture remain in-session controls, while busy,
degraded, and proof-limited states stay visible in user language inside the same loop.

### D5: CLI-only is the packet boundary

GUI parity, shared-shell contract work, and broader runtime or workflow programs remain explicitly
deferred.

## Minimal Implementation Slice

### Milestone 1: Freeze the loop contract

Validation:

- `spec.md`, `plan.md`, `research.md`, `data-model.md`, and
  `contracts/cli-session-loop-contract.md` agree on the same session-loop states, truth rules,
  and deferrals
- packet checklists pass without unresolved clarification markers

## Implementation-Readiness Gate

Packet `031` is implementation-ready only when all of the following are true:

- `spec.md` defines observable live-loop behavior in user-facing terms rather than product-shape
  slogans
- `data-model.md` and `contracts/cli-session-loop-contract.md` distinguish accepted, running,
  completed, failed, blocked, busy, degraded, ended, and proof-limited states clearly enough to
  drive implementation and review
- `quickstart.md`, `analyze.md`, and `tasks.md` agree that this step closes the packet freeze only
  and does not claim code completion or a fresh live-proof rerun
- both packet checklists pass

Repo-wide strategic promotion is outside this packet-freeze step. This plan only makes the packet
internally ready for bounded implementation.

### Milestone 2: Make the live loop readable in place

Validation:

- targeted CLI coverage proves follow-up turns stay inside one active loop
- accepted, active, completed, failed, and blocked states are readable inline without detached
  inspection-only flow

### Milestone 3: Preserve resumed continuity and in-session steering truth

Validation:

- targeted coverage proves resumed sessions reopen with retained context and stored controls
- degraded runtime, busy session, ended session, and proof-limited states remain explicit and
  honest inside the loop

## Parallel Staging Posture

Use bounded parallel work only after the shared session-loop contract is frozen.

- Blocking freeze before any parallel lanes:
  - `specs/031-chat-first-cli-loop/spec.md`
  - `specs/031-chat-first-cli-loop/plan.md`
  - `specs/031-chat-first-cli-loop/contracts/cli-session-loop-contract.md`
- Allowed disjoint lanes after the freeze:
  - CLI loop lane: `crates/mister-smith-app/src/main.rs`
  - session rendering lane: `crates/mister-smith-app/src/conversation.rs`
  - session-view contract lane: `crates/mister-smith-http/src/server.rs`
- Single-owner choke points:
  - `crates/mister-smith-app/src/main.rs`
  - `crates/mister-smith-app/src/conversation.rs`

## Explicitly Deferred

- GUI parity or shared CLI and GUI shell work
- repo workflow tooling, external control-plane automation, or product framing that collapses into
  Linear, Symphony, Ralph, or SpecKit
- broad runtime redesign, new session stores, or new live-proof claims beyond the existing
  supported runtime-proof baseline

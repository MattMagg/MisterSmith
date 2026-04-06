# Implementation Plan: Session-First CLI Shell

**Branch**: `030-session-first-cli-shell` | **Date**: 2026-04-05 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/030-session-first-cli-shell/spec.md`

## Summary

Current `main` already has durable retained sessions, stable session identity, retained turn
history, session list and inspect surfaces, and CLI entry seams. What it does not yet have is one
clear session-first CLI shell that teaches users to start, resume, browse, and steer sessions as
the main experience.

This packet freezes the next bounded implementation slice for that gap. The work centers on making
`mister-smith` open into a recent-first CLI shell home, making resume and recent-session flows
first-class, keeping the current durable session system as the CLI source of truth, and keeping
runtime, doctor, auth, proof, config, and MCP administration as support surfaces rather than the
main product identity.

## Technical Context

**Language/Version**: Rust 1.88.0 for the app and runtime seams
**Primary Dependencies**: `mister-smith-app`, `mister-smith-http`, existing session persistence
and retained-context seams, `clap`, `reqwest`, `axum`, and the current runtime-backed session
service
**Storage**: existing durable session records, retained turn history, and current session summary
or inspect projections only; no second packet-owned session store
**Testing**: targeted Rust tests for CLI and session behavior, targeted HTTP or session-route
coverage, packet markdown lint, and diff hygiene
**Target Platform**: local terminal use on current supported repo platforms
**Project Type**: Rust workspace packet for one CLI-first product shell
**Performance Goals**: startup and resume flows stay direct, recent sessions remain visible
without log archaeology, and degraded support state does not bury the main CLI actions
**Constraints**: keep one durable session model, preserve the current session seams as the source
of truth, avoid GUI parity scope, avoid repo-workflow scope, avoid broad runtime redesign, and
keep support surfaces secondary to the CLI session path
**Scale/Scope**: one bounded packet across shell entry behavior, recent-session home, CLI resume
flows, and in-session control posture

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, the April 5 shell notes, and the current CLI, HTTP, and retained-session seams. |
| II. Spec-First Design | PASS | The feature has `spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, and packet contracts before implementation tasks are generated. |
| III. Phase-And-Packet-Gated Delivery | PASS | The packet is a bounded product-side CLI slice layered on already-landed session and runtime seams, not a new open-ended program. |
| IV. Model-Agnostic Architecture | PASS | The packet changes CLI shell posture and session steering, not provider-specific runtime contracts. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet preserves the current runtime fault model and keeps degraded-state warnings explicit instead of hiding them behind the shell. |
| VI. Evidence-Based Validation | PASS | The plan separates deterministic CLI validation work from any broader runtime-proof claims. |
| VII. Explicit Dependency Management | PASS | The plan names the current CLI, HTTP, and retained-session dependencies and keeps session truth anchored to existing seams. |
| VIII. Clean Closure And Resumability | PASS | The packet artifacts define one bounded write set, one durable session model, and a clear validation path for later implementation. |

## Project Structure

```text
specs/030-session-first-cli-shell/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli-session-shell-contract.md
│   └── cli-session-state-contract.md
├── tasks.md
└── analyze.md

crates/mister-smith-app/
├── src/main.rs
├── src/conversation.rs
├── src/autonomy.rs
└── tests/

crates/mister-smith-http/
└── src/server.rs
```

## Design Decisions

### D1: Reuse the current durable session seams as the one CLI source of truth

The packet extends the current session model already present in the CLI and HTTP surfaces. The
CLI shell must use that same retained-session truth rather than inventing a new CLI-only history
store.

### D2: The default CLI entry becomes a recent-first shell home

The shell must teach the main product path immediately: recent sessions, start new, resume last,
startup warnings, and config. Runtime boot and maintenance remain available but no longer define
the primary CLI entry.

### D3: Resume and recent-session browsing are distinct CLI behaviors

Quick resume should stay direct, while browsing retained sessions should expose enough summary
state to let a user intentionally reopen the right prior session.

### D4: Core live-session controls stay inside the CLI shell

The CLI uses slash commands or another clearly in-session command flow for model, permissions,
config, status, and MCP so users do not need to leave the live session for a separate admin-first
path.

### D5: GUI parity is deferred from this packet

The packet deliberately narrows to the CLI shell so the product can improve the main terminal path
without paying the added scope cost of a shared multi-surface contract in this slice.

## Minimal Implementation Slice

### Milestone 1: Freeze the CLI product contract

Deliverables:

- canonical CLI session-shell and CLI session-state contracts
- one grounded plan for default entry behavior, recent-session home, and support-surface posture

Validation:

- `spec.md`, `plan.md`, `research.md`, `data-model.md`, and both contract files tell the same
  CLI-only product story
- no contract language widens into GUI parity, repo-workflow scope, or generic admin-console scope

### Milestone 2: Define the startup home and resume flows

Deliverables:

- default `mister-smith` CLI shell-home behavior
- recent-session browsing and resume-last rules
- degraded-state warning rules that preserve recent-session visibility

Validation:

- user story 1 and user story 2 map cleanly to the packet tasks and quickstart flows
- resume-last and browse-reopen remain distinct in the packet artifacts

### Milestone 3: Define live-session steering and support-surface boundaries

Deliverables:

- one shared CLI control-state model for model, permissions, config, status, and MCP
- one rule set for keeping support surfaces secondary during live-session steering
- one explicit proof-boundary note for deterministic CLI validation versus broader runtime proof

Validation:

- user story 3 maps cleanly to packet tasks and contracts
- the packet keeps live-session steering inside the CLI shell while leaving support commands
  reachable but secondary

## Parallel Staging Posture

Use bounded parallel work only after the shared CLI contract freeze is complete.

- Blocking freeze before any parallel lanes:
  - `specs/030-session-first-cli-shell/spec.md`
  - `specs/030-session-first-cli-shell/contracts/cli-session-shell-contract.md`
  - `specs/030-session-first-cli-shell/contracts/cli-session-state-contract.md`
- Allowed disjoint lanes after the freeze:
  - CLI entry lane: `crates/mister-smith-app/src/main.rs` and any packet-owned CLI tests
  - retained-session lane: `crates/mister-smith-app/src/conversation.rs` and
    `crates/mister-smith-http/src/server.rs`
- Single-owner choke points:
  - `crates/mister-smith-app/src/main.rs`
  - `crates/mister-smith-app/src/conversation.rs`
  - `crates/mister-smith-http/src/server.rs`

## Explicitly Deferred

- GUI parity, cross-surface continuity, and desktop app changes
- repo-workflow tooling, queue orchestration, or tracker integration
- broad runtime redesign, provider-routing redesign, or new persistence ownership

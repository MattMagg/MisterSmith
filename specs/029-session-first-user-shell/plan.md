# Implementation Plan: Session-First User Shell

**Branch**: `029-session-first-user-shell` | **Date**: 2026-04-05 | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from `/specs/029-session-first-user-shell/spec.md`

## Summary

Current `main` already has durable retained sessions, stable session identity, retained turn
history, session list and inspect surfaces, restart-resume lineage, and a local desktop surface.
What it does not yet have is one clear product shell that teaches users to start, resume, browse,
and steer sessions as the main experience.

This packet freezes the next bounded implementation slice for that gap. The work centers on making
`mister-smith` open into a recent-first session home, making resume and recent-session flows
first-class, defining one shared session model and app protocol for CLI and GUI, and keeping
runtime, doctor, auth, proof, config, and MCP administration as support surfaces rather than the
main product identity.

## Technical Context

**Language/Version**: Rust 1.88.0 for the app and runtime seams, plus the existing TypeScript
desktop surface under `apps/operator-console/`
**Primary Dependencies**: `mister-smith-app`, `mister-smith-http`, existing session persistence
and retained-context seams, `apps/operator-console`, `clap`, `axum`, and the current operator
console data-fetching layer
**Storage**: existing durable session records, retained turn history, and current session summary
or inspect projections only; no second packet-owned session store
**Testing**: targeted Rust tests for CLI and session behavior, targeted HTTP or session-route
coverage, desktop app test and build checks, packet markdown lint, and diff hygiene
**Target Platform**: local macOS desktop plus existing CLI surfaces, while preserving current
workspace portability expectations
**Project Type**: mixed Rust workspace and local desktop packet for one shared product shell
**Performance Goals**: startup and resume flows stay direct, recent sessions remain visible
without log archaeology, and degraded support state does not bury the main product actions
**Constraints**: keep one shared session model, preserve the current durable session seams as the
source of truth, avoid repo-workflow scope, avoid broad runtime redesign, and keep support
surfaces secondary to the session path
**Scale/Scope**: one bounded packet across shell entry behavior, recent-session home, shared
CLI/GUI session continuity, and in-session control posture

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, the April 5 shell notes, and the current CLI, HTTP, and desktop session seams. |
| II. Spec-First Design | PASS | The feature has `spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, and packet contracts before implementation tasks are generated. |
| III. Phase-And-Packet-Gated Delivery | PASS | The packet is a bounded product-side slice layered on already-landed session and operator seams, not a new open-ended program. |
| IV. Model-Agnostic Architecture | PASS | The packet changes shell posture and session steering, not provider-specific runtime contracts. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet preserves the current runtime fault model and keeps degraded-state warnings explicit instead of hiding them behind the shell. |
| VI. Evidence-Based Validation | PASS | The plan separates deterministic shell-validation work from any broader runtime-proof claims. |
| VII. Explicit Dependency Management | PASS | The plan names the current CLI, HTTP, retained-session, and desktop dependencies and keeps session truth anchored to existing seams. |
| VIII. Clean Closure And Resumability | PASS | The packet artifacts define one bounded write set, one shared session model, and a clear validation path for later implementation. |

## Project Structure

```text
specs/029-session-first-user-shell/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── session-shell-contract.md
│   └── shared-session-protocol-contract.md
├── tasks.md
└── checklists/
    ├── requirements.md
    └── session-shell.md

crates/mister-smith-app/
├── src/main.rs
├── src/conversation.rs
├── src/autonomy.rs
└── tests/

crates/mister-smith-http/
└── src/server.rs

apps/operator-console/
├── src/App.tsx
├── src/services.ts
├── src/types.ts
└── src/views/
```

## Design Decisions

### D1: Reuse the current durable session seams as the one source of truth

The packet extends the current session model already present in the CLI and HTTP surfaces. The GUI
must consume that same session truth rather than inventing a desktop-only session layer.

### D2: The default entry becomes a recent-first home, not a runtime-first command surface

The shell must teach the main product path immediately: recent sessions, start new, resume last,
startup warnings, and config. Runtime boot and maintenance remain available but no longer define
the primary entry.

### D3: Resume and recent-session browsing are distinct product behaviors

Quick resume should stay direct, while browsing retained sessions should expose enough summary
state to let a user intentionally reopen the right prior session.

### D4: CLI and GUI must expose the same core in-session controls

The CLI uses slash commands and the GUI uses a command palette or equivalent in-session surface,
but both must operate on the same model, permissions, config, status, and MCP control set.

### D5: Support surfaces remain secondary

`runtime`, `doctor`, `auth`, `proof`, `config`, and MCP administration must stay reachable, but
they cannot take over the product's main navigation language or startup path.

## Minimal Implementation Slice

### Milestone 1: Freeze the product contract for one shared shell

Deliverables:

- canonical session-shell and shared-session protocol contracts
- one grounded plan for default entry behavior, recent-session home, and support-surface posture

Validation:

- `spec.md`, `plan.md`, `research.md`, `data-model.md`, and both contract files tell the same
  product story
- no contract language widens into repo-workflow or generic admin-console scope

### Milestone 2: Define the startup home and resume flows

Deliverables:

- default `mister-smith` shell-home behavior
- recent-session browsing and resume-last rules
- degraded-state warning rules that preserve recent-session visibility

Validation:

- user story 1 and user story 2 map cleanly to the packet tasks and quickstart flows
- resume-last and browse-reopen remain distinct in the packet artifacts

### Milestone 3: Define live-session steering and cross-surface continuity

Deliverables:

- one shared control-state model for model, permissions, config, status, and MCP
- one continuity rule for moving a live session between CLI and GUI
- one rule for keeping support surfaces secondary during live-session steering

Validation:

- user story 3 maps cleanly to packet tasks and contracts
- the packet preserves one shared session identity and transcript story across both front ends

## Parallel Staging Posture

Use bounded parallel work only after the shared contract freeze is complete.

- Blocking freeze before later lanes:
  - `specs/029-session-first-user-shell/spec.md`
  - `specs/029-session-first-user-shell/contracts/session-shell-contract.md`
  - `specs/029-session-first-user-shell/contracts/shared-session-protocol-contract.md`
- Allowed disjoint lanes after the freeze:
  - CLI entry lane: `crates/mister-smith-app/src/main.rs` and any packet-owned CLI tests
  - retained-session lane: `crates/mister-smith-app/src/conversation.rs` and
    `crates/mister-smith-http/src/server.rs`
  - desktop shell lane: `apps/operator-console/src/App.tsx`, `apps/operator-console/src/services.ts`,
    `apps/operator-console/src/types.ts`, and related desktop tests
- Single-owner choke points:
  - `crates/mister-smith-app/src/main.rs`
  - `crates/mister-smith-app/src/conversation.rs`
  - `crates/mister-smith-http/src/server.rs`
  - `apps/operator-console/src/App.tsx`

## Explicitly Deferred

- repo-workflow tooling, queue orchestration, or tracker integration
- plugin marketplace, IDE bridge, voice UX, or unrelated shell expansion
- broad runtime redesign, provider-routing redesign, or new persistence ownership
- fresh live runtime-proof claims beyond the current bounded runtime-proof baseline

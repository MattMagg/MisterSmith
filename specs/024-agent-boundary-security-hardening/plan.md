# Implementation Plan: Agent-Boundary Security Hardening

**Branch**: `024-agent-boundary-security-hardening` | **Date**: 2026-04-01 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/024-agent-boundary-security-hardening/spec.md`

## Summary

Current `main` already has the core boundary pieces that packet `024` needs: separate discover and
execute actions in ToolBus, bounded MCP discovery, action-bound delegation validation, auth
callout, quarantine inspection, state validation, sandbox isolation, shared-state mediation, and
packet `016` continuity. Packet `024` does not redesign those seams. It hardens the remaining
gaps:

- remove legacy descriptorless execute authorization
- publish both discover and execute actions in MCP capability metadata
- make quarantine reasons deterministic for sanitized and monitored outcomes
- clamp auth-callout fallback to the quarantined permission ceiling

The clean packet-024 worktree is based on `origin/main`, but packet authority also uses the newer
packet-022 current-state and implementation docs from the primary checkout because those docs
already record the landed durable-workflow baseline and are intentionally outside this packet-024
write set.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned markdown artifacts
**Primary Dependencies**: `mister-smith-security`, `mister-smith-agents`,
`mister-smith-mcp`, `mister-smith-persistence`, packet `016` continuity notes,
`specs/022-durable-workflow-core/`, and the Phase 9.1 security-hardening contracts
**Storage**: existing delegation metadata, audit streams, and shared-state storage through current
PostgreSQL and JetStream-backed seams; no new storage technology is introduced by this packet
**Testing**: targeted Rust tests in `mister-smith-security`, `mister-smith-agents`,
`mister-smith-mcp`, and `mister-smith-persistence`, plus markdown and diff hygiene
**Target Platform**: local macOS development and Linux runtime parity for the shipped Rust
workspace
**Project Type**: Rust workspace packet plus bounded packet documentation
**Performance Goals**: keep least-privilege checks on the current runtime path, keep quarantine and
validation deterministic, and avoid widening hot paths into a generic policy engine
**Constraints**: no generic IAM rollout, no broader interop design, no new live reject surface, no
compliance expansion, and no silent drift away from MCP `2025-11-25` protocol pages
**Scale/Scope**: one bounded packet that freezes boundary rules across ToolBus, MCP, quarantine,
auth callout, sandboxing, and shared-state mediation

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/current-state.md`, `docs/direction.md`, packet `016`, `MS-77`, packet `022`, and current code seams. |
| II. Spec-First Design | PASS | Packet docs, contracts, checklist, and tasks are revised before code changes. |
| III. Phase-And-Packet-Gated Delivery | PASS | Checklist completion and packet authority are explicit blockers before code work. |
| IV. Model-Agnostic Architecture | PASS | The packet hardens boundary mechanics without provider-specific logic. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet builds on existing sandbox, quarantine, and auth-callout isolation rather than weakening them. |
| VI. Evidence-Based Validation | PASS | Validation remains deterministic and explicitly avoids inventing live rejection proof. |
| VII. Explicit Dependency Management | PASS | The write set and contract reuse are explicit and tied to named repo anchors. |
| VIII. Clean Closure And Resumability | PASS | The packet ends with checklist completion, deterministic validation, and explicit proof boundaries. |

## Project Structure

```text
specs/024-agent-boundary-security-hardening/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── capability-boundary-contract.md
│   ├── quarantine-and-schema-enforcement.md
│   └── identity-and-sandbox-boundary.md
├── checklists/
│   └── requirements.md
├── tasks.md
└── analyze.md

crates/mister-smith-security/
├── src/delegation.rs
├── src/auth_callout.rs
├── src/quarantine.rs
├── src/state_validator.rs
├── src/sandbox.rs
└── tests/

crates/mister-smith-agents/
├── src/tool_bus.rs
├── src/sandbox.rs
└── tests/

crates/mister-smith-mcp/
├── src/client.rs
├── src/bridge.rs
├── src/server.rs
├── src/compatibility.rs
└── tests embedded in src modules

crates/mister-smith-persistence/
├── src/repository/agent.rs
└── tests/
```

## Design Decisions

### D1: Checklist completion is the authority gate

**Decision**: packet `024` is not allowed to move into code until the packet docs are updated to
current truth and the requirements checklist is fully complete.

**Rationale**: the packet must be brought onto current `main` truth before code changes can be
trusted.

### D2: Discover and execute remain separate everywhere

**Decision**: packet `024` preserves separate discover and execute actions across ToolBus, MCP
metadata, MCP discovery, and MCP execution.

**Rationale**: MS-77 already proved this is the clean least-privilege baseline.

### D3: Execute paths must be descriptor-bound

**Decision**: action-bound execute checks will no longer accept legacy descriptorless capabilities.

**Rationale**: packet `024` is specifically about least-privilege boundary hardening, so legacy
descriptorless compatibility is now the wrong default on execute paths.

### D4: Quarantine reasons must be explicit

**Decision**: sanitized and monitored pass-through outcomes get deterministic human-readable
reasons in the quarantine inspection and audit surface.

**Rationale**: packet `024` owns boundary evidence, not just pass/fail behavior.

### D5: Current JWT/auth-callout/delegation posture stays the identity baseline

**Decision**: packet `024` keeps the current JWT, auth-callout, and delegation-envelope posture as
the implementation baseline and leaves SPIFFE as comparator guidance only.

**Rationale**: this packet hardens the existing shipped seams rather than opening a second identity
program.

### D6: Packet 016 continuity and no-fabrication stay intact

**Decision**: packet `024` composes with packet `016` accepted-ingress truth and does not invent a
workflow-backed live reject surface.

**Rationale**: packet `016` already closed that scope honestly, and packet `024` is not allowed to
reopen it.

## Milestones

### Phase 0: Checklist completion and packet authority

**Scope**:

- revise packet docs to current `main`
- replace dead references
- align tasks, contracts, research, quickstart, and analysis
- complete the packet checklist

**Validation**:

- `checklists/requirements.md` is `16/16`
- markdownlint passes on the packet docs

### Phase 1: Freeze the shared boundary contracts

**Scope**:

- capability-boundary contract
- quarantine and schema contract
- identity and sandbox contract

**Validation**:

- packet docs describe the current repo seams and the exact hardening changes
- packet authority wording is implementation-ready and matches the completed checklist

### Phase 2: Capability-boundary hardening

**Scope**:

- ToolBus execute enforcement
- MCP descriptor shape
- MCP metadata and catalog propagation

**Validation**:

- ToolBus tests reject descriptorless legacy execute authority
- MCP tests publish and preserve both discover and execute actions

### Phase 3: Quarantine and boundary-evidence hardening

**Scope**:

- deterministic reasons for sanitized and monitored outcomes
- shared-state mediation remains unchanged except for stronger evidence

**Validation**:

- security, agents, and persistence tests cover reasons and taint behavior

### Phase 4: Identity and fallback hardening

**Scope**:

- auth-callout fallback ceiling
- delegation continuity remains intact

**Validation**:

- auth-callout tests prove fallback cannot exceed quarantined access
- packet `016` continuity assumptions remain unchanged

## Parallel Staging Posture

- doc authority work stays parent-owned and runs first
- after docs are settled, code can split into two disjoint lanes:
  - ToolBus plus MCP capability metadata and enforcement
  - security crate plus shared-state and auth-callout hardening
- final validation runs serially after both lanes merge

## Explicitly Deferred

- generic IAM or enterprise identity policy work
- SPIFFE rollout work
- broader interoperability protocol design
- compliance, legal, or audit-program expansion beyond current runtime evidence
- new live rejection proof for delegated HTTP task ingress
- broader operator-console or observability redesign work

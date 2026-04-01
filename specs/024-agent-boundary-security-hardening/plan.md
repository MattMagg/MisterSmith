# Implementation Plan: Agent-Boundary Security Hardening

**Branch**: `024-agent-boundary-security-hardening` | **Date**: 2026-04-01 |
**Spec**: [spec.md](spec.md)
**Input**: Feature specification from
`/specs/024-agent-boundary-security-hardening/spec.md`

## Draft Status And Revision Gate

This is draft scaffolding, not final implementation authority.

- packet `024` is being scaffolded before earlier packets are fully complete
- claims are based on current repo truth and current dossiers
- before implementation, this plan MUST be revised against the then-current
  `docs/current-state.md`, `docs/direction.md`, and any newly landed earlier packet artifacts
- if earlier packet work changes reused contracts, packet `024` wins no authority over those
  contracts until revised

## Summary

Current repo truth already includes the major security building blocks needed for packet `024`:
ToolBus discover-versus-execute separation, MCP descriptor-and-action-bound enforcement, bounded
delegation validation, auth callout, quarantine inspection, state validation, sandbox isolation,
and packet `016` accepted-ingress continuity. This packet freezes those seams into one bounded
least-privilege contract before later packets widen delegation or interoperability surfaces.

## Technical Context

**Language/Version**: Rust 1.88.0 plus repo-owned markdown artifacts
**Primary Dependencies**: `mister-smith-security`, `mister-smith-agents`,
`mister-smith-mcp`, `mister-smith-persistence`, packet `016` continuity notes, and the Phase 9.1
security-hardening contracts
**Storage**: existing delegation metadata, audit streams, and shared-state storage through current
PostgreSQL and JetStream-backed seams; no new storage technology is introduced by this packet
**Testing**: targeted Rust tests in `mister-smith-security`, `mister-smith-agents`,
`mister-smith-mcp`, and `mister-smith-persistence`, plus markdown and diff hygiene
**Target Platform**: local macOS development and Linux runtime parity for the shipped Rust
workspace
**Project Type**: Rust workspace packet plus bounded packet documentation
**Performance Goals**: keep least-privilege checks on the current runtime path, keep quarantine and
validation deterministic, and avoid widening current hot paths into a generic policy engine
**Constraints**: no generic IAM rollout, no broader interop design, no new live reject surface, no
compliance expansion, and no silent drift away from MCP `2025-11-25` protocol pages
**Scale/Scope**: one bounded packet that freezes boundary rules and defers implementation until a
later refresh pass

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounded in `docs/direction.md`, `docs/current-state.md`, packet `016`, `MS-77`, and current code seams. |
| II. Spec-First Design | PASS | This packet is being scaffolded entirely through spec artifacts before implementation. |
| III. Phase-And-Packet-Gated Delivery | PASS | Packet `024` is bounded and includes a mandatory pre-implementation refresh gate because earlier packets are still moving. |
| IV. Model-Agnostic Architecture | PASS | The packet hardens boundaries and identity posture without introducing provider-specific logic. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet builds on existing sandbox, quarantine, and auth-callout isolation rather than weakening them. |
| VI. Evidence-Based Validation | PASS | Validation remains deterministic and explicitly avoids inventing live rejection proof. |
| VII. Explicit Dependency Management | PASS | The write set and contract reuse are explicit and tied to named repo anchors. |
| VIII. Clean Closure And Resumability | PASS | The scaffold includes draft-status notes, a refresh gate, and an analysis report for cold-start reuse. |

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
├── src/server.rs
├── src/compatibility.rs
└── tests/

crates/mister-smith-persistence/
├── src/repository/agent.rs
└── tests/
```

## Design Decisions

### D1: This scaffold is real packet prep, but not final implementation authority

**Decision**: packet `024` is authored now for speed, but implementation is blocked on a later
refresh pass against newly landed earlier packet work.

**Rationale**: this gives the repo a reusable packet shape now without pretending that moving
upstream truth is already final.

### D2: Discover and execute remain separate everywhere

**Decision**: the packet freezes discover and execute as separate permissions and separate action
bindings across ToolBus, MCP discovery, and MCP invocation.

**Rationale**: MS-77 and the current ToolBus/MCP seams already prove this split is the safest
baseline to preserve.

### D3: Quarantine and schema enforcement happen before agent consumption

**Decision**: cross-boundary content and shared-state reads remain mediated by size checks, schema
validation, malicious-pattern inspection, and quarantine outcomes before agent context sees them.

**Rationale**: the research and the Phase 9.1 contracts both point to deterministic mediation, not
prompt-only handling, as the boundary rule to freeze.

### D4: Current JWT/auth-callout/delegation posture stays the identity baseline

**Decision**: packet `024` keeps the current JWT, auth-callout, and delegation-envelope posture as
the implementation baseline and leaves SPIFFE as comparator guidance only.

**Rationale**: this packet hardens the existing shipped seams rather than opening a second identity
program.

### D5: Persistent and ephemeral separation is a boundary rule, not a broader redesign

**Decision**: packet `024` freezes persistent-versus-ephemeral separation for credentials, subject
reach, and shared-state mediation without widening into a larger IAM or role-system rewrite.

**Rationale**: the repo already has the right sandbox direction; the missing piece is one coherent
boundary contract.

### D6: Packet `016` continuity and no-fabrication stay intact

**Decision**: packet `024` must compose with packet `016` accepted-ingress truth and must not
invent a workflow-backed live reject surface.

**Rationale**: packet `016` already closed that scope honestly, and this packet is not allowed to
reopen it casually.

## Minimal Implementation Slice

### Milestone 0: Mandatory refresh gate before implementation

**Scope**: re-read `docs/current-state.md`, `docs/direction.md`,
`docs/packet-prep/024-agent-boundary-security-hardening.md`, and any earlier packet artifacts that
land before packet `024` code starts.

**Validation**:

- packet `024` spec, plan, contracts, and tasks are revised if reused contracts drifted
- no implementation begins until the refresh note is complete

### Milestone 1: Freeze the capability-boundary contract

**Scope**: preserve discover-versus-execute separation, exact descriptor/action binding, and
bounded MCP capability discovery without widening authority.

**Validation**:

- targeted ToolBus and MCP tests prove the action-bound boundary still holds
- packet contracts and tasks keep discover separate from execute everywhere

### Milestone 2: Freeze quarantine and schema-enforcement behavior

**Scope**: preserve deterministic size, schema, malicious-pattern, and quarantine behavior across
cross-boundary payloads and shared-state reads.

**Validation**:

- targeted validator, quarantine, sandbox, and persistence tests cover clean, sanitized,
  suspicious, rejected, and quarantined outcomes

### Milestone 3: Freeze identity, sandbox, and continuity behavior

**Scope**: preserve least-privilege auth-callout and sandbox credential posture, revocation,
packet `016` continuity, and boundary evidence without widening into general IAM.

**Validation**:

- auth-callout, delegation, sandbox, and continuity tests remain green
- packet `016` no-fabrication and no-live-reject rules remain intact

## Parallel Staging Posture

- blocking refresh checkpoint before any implementation lane begins:
  - packet refresh and contract reconciliation
- allowed disjoint lanes after the refresh checkpoint:
  - ToolBus and MCP boundary lane
  - quarantine and validator lane
  - auth-callout and sandbox lane
- single-owner choke points:
  - `crates/mister-smith-agents/src/tool_bus.rs`
  - `crates/mister-smith-mcp/src/server.rs`
  - `crates/mister-smith-mcp/src/compatibility.rs`
  - `crates/mister-smith-security/src/delegation.rs`
  - `crates/mister-smith-security/src/auth_callout.rs`
  - `crates/mister-smith-security/src/quarantine.rs`
  - `crates/mister-smith-security/src/state_validator.rs`
  - `crates/mister-smith-security/src/sandbox.rs`
  - `crates/mister-smith-persistence/src/repository/agent.rs`

## Explicitly Deferred

- generic IAM or enterprise identity policy work
- SPIFFE rollout work
- broader interoperability protocol design
- compliance, legal, or audit-program expansion beyond current runtime evidence
- new live rejection proof for delegated HTTP task ingress
- broader operator-console or observability redesign work

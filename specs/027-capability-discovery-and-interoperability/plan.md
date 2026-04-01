# Implementation Plan: Capability Discovery And Interoperability

**Branch**: `027-capability-discovery-and-interoperability` | **Date**: 2026-04-01 | **Spec**:
[spec.md](spec.md)
**Input**: Feature specification from
`specs/027-capability-discovery-and-interoperability/spec.md`

## Summary

Packet `027` freezes one bounded interop scaffold only. It does not authorize immediate
implementation. The packet uses the current finished packet outputs for packets `022`, `023`, and
`024` as upstream inputs, then defines one shared capability normalization seam and one first
remote lifecycle seam for A2A `v0.3.0`. MCP remains pinned to `2025-11-25` as an input and policy
boundary, not as the first new interop slice.

## Technical Context

**Language/Version**: Markdown packet artifacts for a Rust 1.88.0 workspace  
**Primary Dependencies**: `docs/direction.md`, `docs/current-state.md`,
`specs/022-durable-workflow-core/spec.md`,
`specs/023-runtime-truth-and-run-trace/spec.md`,
`specs/024-agent-boundary-security-hardening/spec.md`, packet `016` and `MS-77` proof notes,
`crates/mister-smith-agents`,
`crates/mister-smith-mcp`, `crates/mister-smith-core`, and the pinned MCP/A2A primary sources  
**Storage**: packet-local markdown only for this scaffold; future target surfaces remain the
existing workflow, autonomy, and result metadata seams  
**Testing**: `git diff --check`, targeted `npx markdownlint-cli2` on the packet directory, and
read-only cross-artifact consistency analysis after `tasks.md` exists  
**Target Platform**: repo-local SpecKit packet authoring on macOS with future implementation
targeting the existing Rust runtime and operator-facing status surfaces  
**Project Type**: docs-first SpecKit packet scaffold for future runtime and interoperability work  
**Performance Goals**: keep one bounded slice, preserve proof-boundary clarity, and avoid protocol
drift or scope creep before implementation starts  
**Constraints**: packet `027` must be revised before implementation, must not redefine packet
`022`, `023`, or `024`, must keep discovery separate from execution permission, must keep MCP pinned
to `2025-11-25`, must keep A2A pinned to `v0.3.0`, and must not widen into generic federation  
**Scale/Scope**: one normalized capability contract, one A2A lifecycle mapping contract, one
operator-visible provenance shape, and one blocking refresh gate before any future implementation

## Constitution Check

| Principle | Status | Evidence |
| --------- | ------ | -------- |
| I. Canonical Single Source | PASS | Grounds packet truth in `docs/direction.md`, `docs/current-state.md`, packet outputs `022` to `024`, pinned protocol docs, and current repo seams. |
| II. Spec-First Design | PASS | `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `tasks.md`, and checklist artifacts are authored before any implementation work. |
| III. Phase-And-Packet-Gated Delivery | PASS | The packet is scaffold-only and includes a blocking refresh gate before implementation so it does not silently outrun packets `022`, `023`, and `024`. |
| IV. Model-Agnostic Architecture | PASS | The packet defines protocol and lifecycle mapping surfaces without binding Mister Smith to a provider-specific model API. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | The packet reuses current workflow and autonomy surfaces, keeps failure isolation intact, and treats remote lifecycle mapping as additive and bounded. |
| VI. Evidence-Based Validation | PASS | Validation is explicit, deterministic, and separated from future live runtime proof claims. |
| VII. Explicit Dependency Management | PASS | Upstream packet dependencies, protocol pins, repo anchors, and deferred follow-on work are all called out directly. |
| VIII. Clean Closure And Resumability | PASS | The packet leaves a resumable scaffold with a mandatory refresh gate and durable contract artifacts for a later implementation pass. |

## Project Structure

```text
specs/027-capability-discovery-and-interoperability/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── checklists/
│   ├── requirements.md
│   └── interop.md
└── contracts/
    ├── capability-normalization-contract.md
    └── a2a-lifecycle-mapping-contract.md

Future repo write set after the refresh gate:
crates/mister-smith-agents/src/tool_bus.rs
crates/mister-smith-mcp/src/client.rs
crates/mister-smith-mcp/src/server.rs
crates/mister-smith-mcp/src/compatibility.rs
crates/mister-smith-core/src/autonomy.rs
crates/mister-smith-events/src/bus.rs
crates/mister-smith-app/src/autonomy.rs
```

## Design Decisions

### D1: A2A is the first new interop slice, MCP is the pinned baseline input

**Decision**: freeze A2A `v0.3.0` agent discovery and A2A task lifecycle mapping as the first new
interop slice, while keeping MCP `2025-11-25` as the pinned normalization and policy-boundary
baseline.

**Rationale**: the repo already has bounded MCP discovery and delegated boundary enforcement from
`MS-77`. The missing new seam is one remote discovery and lifecycle mapping path, not another MCP
discovery packet.

### D2: Normalized discovery must preserve source and permission separation

**Decision**: the shared capability descriptor will carry source identity, schema hints, lifecycle
signals, and a separate permission reference, but the descriptor itself will never stand in for
execution authority.

**Rationale**: packet `024` and `MS-77` already anchor discover-vs-execute separation, and packet
`027` must preserve that rule across local ToolBus, MCP, and A2A discovery inputs.

### D3: Remote lifecycle mapping reuses earlier packet language provisionally

**Decision**: packet `027` will reuse packet `022` lifecycle and identifier language, packet `023`
proof-boundary language, and packet `024` security language as provisional inherited inputs.

**Rationale**: this packet owns capability normalization and one remote lifecycle bridge. It does
not own durable workflow semantics, truth taxonomy, or security policy definitions.

### D4: Implementation is blocked until upstream packets are refreshed

**Decision**: the first future implementation milestone is a refresh gate that revises packet `027`
against the completed packet `022`, `023`, and `024` outputs before any code work begins.

**Rationale**: this packet is being written now for speed, but the repo would drift if a later
implementation treated provisional dossier language as final contract truth.

## Minimal Implementation Slice

### Milestone 1: Upstream Refresh Gate (Blocking)

**Scope**: reconcile packet `027` with the finished packet `022`, `023`, and `024` outputs, then
refresh packet artifacts before implementation.

**Validation**:

- `spec.md`, `plan.md`, `research.md`, and `tasks.md` are updated against the final upstream
  packet outputs
- protocol pins remain MCP `2025-11-25` and A2A `v0.3.0`
- the refresh gate remains the first executable step in `tasks.md`

### Milestone 2: Shared Descriptor Freeze

**Scope**: implement the normalized capability descriptor and source mapping across local ToolBus,
MCP, and one A2A discovery adapter.

**Validation**:

- targeted Rust tests prove normalized descriptor parity across local ToolBus, MCP, and A2A inputs
- discovery metadata remains distinct from execution permission in the shared descriptor

### Milestone 3: A2A Lifecycle Projection And Provenance

**Scope**: implement the A2A task lifecycle bridge into Mister Smith workflow, result, and
autonomy surfaces, plus operator-visible provenance.

**Validation**:

- targeted Rust tests prove the A2A lifecycle binding into workflow and status surfaces
- operator-facing result or autonomy views preserve discovery-versus-execute boundaries and packet
  `016` continuity language

## Parallel Staging Posture

- Blocking freeze before any parallel lanes: upstream refresh gate plus final packet contract
  refresh
- Allowed disjoint lanes after the freeze:
  - descriptor normalization lane: `crates/mister-smith-agents/src/tool_bus.rs`,
    `crates/mister-smith-mcp/src/client.rs`, `crates/mister-smith-mcp/src/server.rs`
  - lifecycle projection lane: `crates/mister-smith-core/src/autonomy.rs`,
    `crates/mister-smith-events/src/bus.rs`, `crates/mister-smith-app/src/autonomy.rs`
  - compatibility adapter lane: `crates/mister-smith-mcp/src/compatibility.rs` plus packet `027`
    proof note updates
- Single-owner choke points:
  - `specs/027-capability-discovery-and-interoperability/spec.md`
  - `specs/027-capability-discovery-and-interoperability/contracts/a2a-lifecycle-mapping-contract.md`
  - `crates/mister-smith-core/src/autonomy.rs`
  - `crates/mister-smith-mcp/src/compatibility.rs`

## Explicitly Deferred

- generic federation or mesh design
- remote execution permission expansion
- other remote protocols beyond MCP baseline input and one A2A slice
- live multi-remote runtime proof claims
- packet `028` strong-coordination policy or state-taxonomy work
- any change that turns packet `016` into a broader remote lifecycle proof claim

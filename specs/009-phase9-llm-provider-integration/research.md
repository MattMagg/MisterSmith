# Research: Phase 9 — LLM Provider Integration

**Date**: 2026-03-06
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

Phase 9 does not need new external research to move into planning. The governing decisions are
already present in `ROADMAP.md`, the approved LLM provider integration design, the architectural
grounding audit, the implementation deviation report, the Phase 7 baseline artifacts, and the
canonical `spec/` architecture sources. The planning job is to turn those sources into a grounded,
non-drifting artifact set.

---

## R1: One New Crate, Not Provider-Specific Crate Sprawl

**Decision**: Introduce a single `mister-smith-llm` workspace crate with feature-gated Anthropic and
OpenAI providers plus an always-available `MockProvider`.

**Rationale**: The approved design already chooses this approach, and the repo's existing
crate-per-domain pattern supports it. Splitting providers into separate crates would create extra
workspace management without solving a current problem.

**Alternatives considered**:

- **Provider-specific crates**: rejected as premature fragmentation.
- **Put provider logic in `mister-smith-agents`**: rejected because it would make orchestration
  depend directly on vendor HTTP clients.

---

## R2: Shared Error Placement Must Follow The Existing Core Pattern

**Decision**: Add `LlmError` to `crates/mister-smith-core/src/error.rs` and re-export it from
`crates/mister-smith-core/src/lib.rs`.

**Rationale**: The core crate is already the canonical home for top-level domain errors such as
`ToolError`, `SecurityError`, and `PersistenceError`. Creating a second public error hierarchy in
`mister-smith-llm` would break the "single source of truth" rule called out in the constitution and
supported by `spec/core-architecture/type-definitions.md`.

**Alternatives considered**:

- **`mister-smith-llm` owns its own top-level error type**: rejected because downstream crates would
  now need special error handling rules not used elsewhere in the workspace.
- **Provider-specific public errors only**: rejected because the spec requires provider-neutral
  behavior.

---

## R3: Capability Normalization Beats Artificial Feature Flattening

**Decision**: Express provider parity through unified request and response types plus
`ModelCapabilities`; unsupported behavior returns typed errors instead of weakening the whole public
API to the least-capable backend.

**Rationale**: Anthropic and OpenAI do not expose identical native payloads or capability sets, but
the clarified Phase 9 spec only requires parity for the shared workflow. Typed
`UnsupportedCapability` behavior is the cleanest way to preserve both model-agnostic architecture
and honest provider differences.

**Alternatives considered**:

- **Require all providers to implement all capabilities**: rejected because it would force fake or
  misleading behavior.
- **Expose raw provider payloads publicly**: rejected because it would leak vendor coupling outside
  provider modules.

---

## R4: The Agent Bridge Must Extend Existing Role Seams, Not Replace Them

**Decision**: The `mister-smith-agents` crate gains an optional `llm` feature that wires a selected
`ModelProvider` into Planner, Critic, and Executor behavior while leaving the Orchestrator and
team/scheduler flow provider-neutral.

**Rationale**: The current role implementations in
`crates/mister-smith-agents/src/roles/{planner,critic,executor}.rs` are intentionally thin, and
`crates/mister-smith-agents/src/orchestrator.rs` already owns the decompose -> assign -> aggregate
workflow. Phase 9 should enrich these seams, not invent a second orchestration pipeline.

**Alternatives considered**:

- **New provider-aware orchestrator crate**: rejected because it would fork the existing execution
  path.
- **LLM-enable every agent role in one phase**: rejected as scope creep beyond the approved design.

---

## R5: Tool Calls Must Flow Through ToolBus, Not Around It

**Decision**: Implement `ToolBus::to_tool_definitions()` and `ToolBus::execute_tool_call()` as the
only sanctioned bridge for model-initiated tool use.

**Rationale**: The approved Phase 9 design and the current ToolBus contract both require permission
checks, timeouts, metrics, and audit or error handling at the ToolBus boundary. A provider-specific
tool-dispatch path would bypass those guarantees and recreate the exact drift this planning pass is
trying to prevent.

**Alternatives considered**:

- **Provider-specific function-calling adapter that dispatches directly to agents**: rejected
  because it bypasses ToolBus security and timeout semantics.
- **MCP-only tool bridge**: rejected because native agent-backed tools are already first-class in
  Phase 7.

---

## R6: Validation Needs Three Tiers, Not One

**Decision**: Use three validation tiers:

1. deterministic unit and serialization tests around the shared types and `MockProvider`
2. env-gated Anthropic and OpenAI integration tests
3. Gate 9 orchestration validation through the existing agent runtime

**Rationale**: Mock tests prove the contract, provider tests prove vendor wiring, and Gate 9 proves
the framework-level workflow. None of the three replaces the others.

**Alternatives considered**:

- **Only mock tests**: rejected because they cannot prove real provider compatibility.
- **Only live-provider tests**: rejected because they are too slow and brittle to carry all contract
  validation.

---

## R7: Phase 7.5 Hardening Is Blocker State, Not Backlog Laundry

**Decision**: Keep the six Phase 7.5 hardening items visible in Phase 9 plan artifacts as
prerequisites or blockers. Do not redefine them as Phase 9 deliverables.

**Rationale**: The implementation deviation report explicitly places these items before Phase 9, and
the clarified spec repeats that instruction. Planning must preserve that posture so `/speckit.tasks`
and `/speckit.analyze` can report blockers honestly.

## Source Map

| Source | Why it matters |
| -------- | ---------------- |
| `ROADMAP.md:586-660` | Canonical Phase 9 scope, subphases, and Gate 9 |
| `docs/plans/2026-03-05-llm-provider-integration-design.md:23-306` | Approved crate, type, and bridge design |
| `docs/2026-03-05-architectural-grounding-audit.md:112-170` | Traceability gap to fix forward |
| `docs/2026-03-05-implementation-deviation-report.md:296-318` | Phase 9 posture plus Phase 7.5 blocker list |
| `spec/data-management/agent-orchestration.md:2467-2665` | LLM coordination reference and ToolBus boundary context |
| `spec/data-management/message-schemas.md:1069-1265` | Deferred hook-event schemas that must stay out of scope |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md:453-500` | Deferred Neural/AI Operations scope |
| `spec/core-architecture/async-patterns.md:1939-2315` | Agent-as-tool and ToolBus patterns to preserve |

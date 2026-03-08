# Research: Phase 9 — LLM Provider Integration

**Date**: 2026-03-07 (revised)
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

Phase 9 research is grounded in a completed 7-round research phase covering 2,000+ papers and
500+ industry references, synthesized into 9 consolidated documents at
`docs/research-output/consolidated/`. The authoritative findings document is
`docs/research-output/consolidated/00-MASTER-FINDINGS.md`.

Three findings are directly incorporated into Phase 9:

| Finding | Source | Phase 9 Impact |
| ------- | ------ | -------------- |
| **#8** | `consolidated/01-model-routing-and-cost-optimization.md` | Two-plane router, health-aware circuit breakers, hierarchical budget enforcement via JetStream KV CAS |
| **#9** | `consolidated/01-model-routing-and-cost-optimization.md` | SLM-default / LLM-fallback routing policy, cascade configuration |
| **#13** | `consolidated/06-streaming-architecture.md` | Dual-stream formalization (lossless semantic + best-effort UI), `ModelEvent` enum, backpressure policy matrix |

## Pre-Research Decisions (Original R1-R7)

The following decisions were made before the 7-round research phase and remain valid. The research
confirmed rather than contradicted them.

### R1: One New Crate, Not Provider-Specific Crate Sprawl

**Decision**: Introduce a single `mister-smith-llm` workspace crate with feature-gated providers
plus an always-available `MockProvider`.

**Confirmed by research**: No contrary evidence found. The one-crate pattern is consistent with
the workspace's one-domain-per-crate convention and does not conflict with any research finding.

### R2: Shared Error Placement Must Follow The Existing Core Pattern

**Decision**: Add `LlmError` to `crates/mister-smith-core/src/error.rs` and re-export.

**Confirmed by research**: Consistent with the project's canonical single-source error pattern.

### R3: Capability Normalization Beats Artificial Feature Flattening

**Decision**: Express provider parity through unified types plus `ModelCapabilities`; unsupported
behavior returns typed errors.

**Confirmed by research**: Finding #9 (SLM-default) reinforces this — different models have
fundamentally different capability sets, and the routing layer handles the mismatch.

### R4: The Agent Bridge Must Extend Existing Role Seams, Not Replace Them

**Decision**: The `mister-smith-agents` crate gains an optional `llm` feature that wires a
selected `ModelProvider` into Planner, Critic, and Executor behavior.

**Confirmed by research**: Finding #1 (dynamic topology) would eventually replace static role
assignments, but that is Phase 11 scope. The agent bridge pattern remains correct for Phase 9.

### R5: Tool Calls Must Flow Through ToolBus, Not Around It

**Decision**: `ToolBus::to_tool_definitions()` and `ToolBus::execute_tool_call()` are the only
sanctioned bridge for model-initiated tool use.

**Confirmed by research**: Finding #6 (security) strengthens this — all tool invocations must
pass through a centralized permission and audit boundary. The ToolBus is that boundary.

### R6: Validation Needs Three Tiers, Not One

**Decision**: Use deterministic mock tests, env-gated provider tests, and Gate 9 orchestration
validation.

**Confirmed by research**: No contrary evidence. Three-tier validation is consistent with the
research emphasis on defense-in-depth.

### R7: Phase 7.5 Hardening Is Blocker State, Not Backlog Laundry

**Decision**: Keep Phase 7.5 hardening items visible as prerequisites or blockers.

**Modified by research**: The Phase 9.1 Security Hardening spec (at
`specs/011-phase9.1-security-hardening/`) now addresses several security gaps that were
previously treated as generic Phase 7.5 blockers. The specific security items are now properly
scoped rather than left as generic deferred work.

---

## Research-Driven Additions

These are new architectural decisions driven by the 7-round research phase that did not exist in
the original Phase 9 specification.

### R8: Two-Plane Router Architecture (Finding #8)

**Decision**: Separate a microsecond-latency data plane (NATS request-reply, ~50us) from a
control plane (JetStream KV watches) for model configuration, health telemetry, and budget state.

**Rationale**: Converged across all three R3 industry reports, validated by academic surveys
(Varangot-Reille 2025, Behera 2025), and reinforced by production gateways (Bifrost 11us
overhead at 5,000 RPS, Kong, Vercel, AWS).

**Key evidence**: NATS request-reply achieves ~50us average latency. Rust-based gateways achieve
11us overhead. LiteLLM (Python/FastAPI) achieves <500 RPS. This infrastructure advantage is
structural and cannot be matched by Python-based competitors.

**Alternatives considered**:
- Single-plane RPC (status quo in LangGraph, CrewAI, AutoGen): rejected because it cannot
  support real-time configuration updates without service restarts.

### R9: SLM-Default / LLM-Fallback Routing (Finding #9)

**Decision**: Default routing policy starts with the cheapest capable model (1-12B parameters)
and escalates to larger models based on configurable confidence thresholds.

**Rationale**: Sharma & Mehta (2025) comprehensive review; Liu (2025, 106 citations) showing
0.5B outperforms GPT-4o with compute-optimal scaling; Yang (2025, 81 citations) on optimal
CoT length. Cost reduction of 10-100x for structured tasks.

**Scope boundaries**: Guided decoding (XGrammar/Outlines) and local model inference are Phase 10+.
Phase 9 implements the cascade policy and confidence-based escalation, not the decoding
enforcement.

### R10: Dual-Stream Formalization (Finding #13)

**Decision**: Emit two parallel streams from the same canonical event log — a lossless semantic
stream (JetStream) for orchestration correctness and a best-effort UI stream (NATS Core) for
real-time rendering.

**Rationale**: All three R3 source reports independently conclude that streaming must be modeled
as a typed event pipeline. The dual-stream design decouples correctness from presentation,
enabling per-event-class backpressure policies.

**Key design**: `StreamChunk`/`ChunkDelta` (4 variants) remain the raw provider boundary.
`ModelEvent` (28 variants) is the canonical internal event type. These are two layers, not a
replacement.

### R11: Budget Enforcement via JetStream KV CAS (Finding #8)

**Decision**: Hierarchical budget tracking (org -> team -> user -> request tag) using JetStream
KV atomic compare-and-swap operations with a reserve-before-send / reconcile-after-completion
pattern.

**Rationale**: All three R3 reports converge on budget enforcement belonging in the router, not
in application code. CAS-based enforcement demonstrates <1% overrun rate vs potentially
unbounded overruns with naive check-then-spend.

## Source Map

| Source | Why it matters |
| -------- | ---------------- |
| `docs/research-output/consolidated/00-MASTER-FINDINGS.md` | Authoritative ranked findings from all 7 research rounds |
| `docs/research-output/consolidated/01-model-routing-and-cost-optimization.md` | Two-plane router, budget enforcement, SLM-default, cascade routing — core Phase 9 architecture |
| `docs/research-output/consolidated/06-streaming-architecture.md` | Dual-stream, backpressure, ModelEvent, stream actors — Phase 9 streaming contract |
| `docs/research-output/consolidated/04-security-and-trust.md` | Security findings driving Phase 9.1 (separate spec) |
| `docs/RESEARCH_CHECKPOINT.md` | Confidence tiers, evidence gaps, what's not pursued |
| `ROADMAP.md:586-660` | Canonical Phase 9 scope, subphases, and Gate 9 |
| `docs/plans/2026-03-05-llm-provider-integration-design.md:23-306` | Approved crate, type, and bridge design |
| `spec/data-management/agent-orchestration.md:2467-2665` | LLM coordination reference and ToolBus boundary context |
| `spec/core-architecture/async-patterns.md:1939-2315` | Agent-as-tool and ToolBus patterns to preserve |

## Explicitly Deferred Findings

| Finding | Phase | Reason |
| ------- | ----- | ------ |
| #1 Dynamic Topology / MaAS | 11 | Depends on Phase 9 agent-LLM bridge |
| #2 Step-Level Intelligence / PRMs | 10 | Depends on Phase 9 streaming infrastructure |
| #3 CRDT Coordination | 13 | Independent of LLM integration |
| #4 Predictive Supervision | 12 | Independent of LLM integration |
| #5 MPST Session Types | 13 | Independent of LLM integration |
| #6 Defense-in-Depth Security | 9.1 | Separate spec at `specs/011-phase9.1-security-hardening/` |
| #7 Infectious Jailbreak Defense | 9.1 | Separate spec at `specs/011-phase9.1-security-hardening/` |
| #10 Neuromorphic Fault Tolerance | 12+ | Requires predictive supervision foundation |
| #12 Persistent KV Cache | 10 | Depends on streaming infrastructure |
| #14 Decentralized Agent Discovery | 11 | Depends on capability model |

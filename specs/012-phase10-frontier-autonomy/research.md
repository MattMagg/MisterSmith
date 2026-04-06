# Research: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

**Date**: 2026-03-10
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

Phase 10 is grounded in the completed research corpus already stored in this repository. The
highest-signal conclusion is that Mister Smith should advance from a static, fixed-topology,
restart-oriented framework into a **supervised autonomy control plane** built on the existing
provider, persistence, and zero-trust substrate.

This phase is not a generic continuation of Phase 9 or 9.1. It is the first roadmap extension
that intentionally combines:

- topology-aware orchestration
- managed memory/context
- predictive supervision
- operator-visible autonomy state
- bounded delegation/provenance

## Implementation Alignment Note (2026-03-15)

The current repository state still matches the research ordering captured here:

- the repo now has real topology, checkpoint, managed-memory, Guard, operator-view, and bounded
  delegation surfaces, which is consistent with the research priority ordering behind R1-R5
- the repo still keeps learned routing, speculative decoding, local inference, consensus, and ML
  anomaly detection outside Phase 10, which preserves the design note's anti-drift boundary

No new evidence in this review pass justified pulling deferred serving or model-selection work back
into Phase 10 scope.

## Key Research Findings

### R1: Topology Dominates Static Team Structure

**Sources**:

- `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
- `docs/research-output/consolidated/02-orchestration-and-self-organization.md`

**Evidence**: Dynamic topology selection and meta-orchestration outperform static teams across the
research corpus. AdaptOrch, MaAS, and MAS^2 are all cited as evidence that execution structure is
now the main performance lever when models are held constant.

**Action**: Introduce an explicit `ExecutionGraph` and deterministic topology compiler as Phase 10
baseline architecture.

### R2: Step-Level Intelligence Should Enter Through Supervision First

**Sources**:

- `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
- `docs/research-output/consolidated/06-streaming-architecture.md`
- `docs/research-output/consolidated/03-supervision-and-resilience.md`

**Evidence**: Streaming monitors, step-boundary detection, and targeted intervention strategies are
high-value next steps, but full speculative decoding and guided-decoding pipelines remain more
expensive extensions.

**Action**: Phase 10 consumes step-level and stream-level degradation signals as Guard inputs. It
does **not** pull in the full speculative-decoding or guided-decoding roadmap.

### R3: Memory Must Be Managed as an OS Resource

**Sources**:

- `docs/research-output/consolidated/07-memory-and-context.md`

**Evidence**: The memory research corpus converges on tiered memory, role-aware context routing,
background consolidation, and checkpoint-ready context snapshots. The repo already has the storage
primitives; what is missing is the management layer.

**Action**: Add a `MemoryManager` over existing JetStream KV + PostgreSQL with `MemoryFragment`,
`ContextBudget`, summaries, consolidation, and snapshots.

### R4: Predictive Supervision Must Augment OTP, Not Replace It

**Sources**:

- `docs/research-output/consolidated/03-supervision-and-resilience.md`
- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`

**Evidence**: The strongest resilience guidance combines existing OTP-style supervision for hard
failures with Guard/Advisor layers for semantic degradation, profile-aware routing, and targeted
interventions.

**Action**: Phase 10 introduces a Guard/Advisor layer with failure taxonomy and intervention
selection while keeping branch isolation and restart semantics under the existing supervision model.

### R5: Zero-Trust Is Execution Substrate, Not the Whole Phase

**Sources**:

- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`
- `docs/research-output/consolidated/04-security-and-trust.md`
- `specs/011-phase9.1-security-hardening/spec.md`

**Evidence**: The March 9 design note explicitly rejects letting the roadmap collapse into
security-only work. At the same time, Phase 9.1 already defers Macaroon-compatible delegation and
provenance to Phase 10+.

**Action**: Phase 10 completes bounded delegation/provenance as an enabling autonomy capability but
does not redefine the Phase 9.1 security substrate.

## Design Decisions

### D1: Use a Mature Graph Library for DAG/Topology Operations

**Decision**: Use `petgraph` rather than hand-built graph traversal and validation logic.

**Rationale**: Dependency validation, cycle detection, and topological ordering are solved
problems. The repository should not custom-build graph algorithms when a mature Rust library exists.

### D2: Keep Topology Selection Deterministic in Phase 10

**Decision**: Use policy- and signal-driven topology selection before introducing learned routers.

**Rationale**: The structural compiler is the prerequisite. Learned routing and kNN/RouteLLM-style
selection remain valuable, but they are downstream of an explicit execution-graph and evented
control plane.

### D3: Memory Management Extends, Not Replaces, Phase 6 Storage

**Decision**: Preserve JetStream KV + PostgreSQL as the Phase 10 memory backing stores.

**Rationale**: The research's strongest implication for this repo is that it already has the
storage substrate. Replacing it would be unnecessary reinvention.

### D4: Operator Visibility Must Use Typed Events and Snapshots

**Decision**: Operator-facing autonomy views derive from typed topology/checkpoint/memory/guard
events and snapshots.

**Rationale**: This keeps the phase aligned with the repo's event-driven architecture and avoids
log-scraping or provider-specific introspection paths.

### D5: Delegation Proceeds from Existing Claims and Auth Callout Work

**Decision**: Extend the Phase 9.1 delegation-chain and capability foundation instead of swapping
to a new token system in the same phase.

**Rationale**: This keeps Phase 10 tractable and maintains continuity with already-specified work.
Token-format replacement or federation-specific capability systems can remain later decisions.

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `docs/research-output/consolidated/00-MASTER-FINDINGS.md` | Ranked summary confirming topology, step intelligence, predictive supervision, and managed memory as the next leverage points |
| `docs/research-output/consolidated/02-orchestration-and-self-organization.md` | Grounds topology selection, DAG execution, and dynamic team structure |
| `docs/research-output/consolidated/03-supervision-and-resilience.md` | Grounds Guard/Advisor design, failure taxonomy, and checkpoint-aware recovery |
| `docs/research-output/consolidated/07-memory-and-context.md` | Grounds managed memory, role-aware context routing, and background consolidation |
| `docs/research-output/consolidated/04-security-and-trust.md` | Grounds bounded delegation/provenance as enabling substrate |
| `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` | Provides the phase's strategic framing and anti-drift guardrail |
| `docs/audits/2026-03-05-implementation-deviation-report.md` | Identifies "Phase 10: Advanced Agent Patterns" as the next roadmap extension |
| `spec/core-architecture/supervision-and-events.md` | Grounds typed autonomy event propagation and operator-visible event assembly |
| `spec/data-management/agent-orchestration.md` | Provides the existing planner/router/memory boundaries and context-management hook |
| `spec/data-management/message-schemas.md` | Defines workflow coordination and hook-event boundaries that Phase 10 can extend |

## Explicitly Deferred Findings

| Finding | Phase | Reason |
| ------- | ----- | ------ |
| Learned routing via RouteLLM / kNN / ONNX | 11+ | Requires the structural compiler and event substrate first |
| Guided decoding / speculative decoding pipelines | 11+ | Step-level supervision enters first; decoding strategy remains a later optimization |
| Local inference / disaggregated serving / shared KV serving | 11+ | Depends on later serving and infrastructure decisions |
| CRDT coordination and MPST session types | 13+ | Independent coordination research stream |
| Auction-based meta-orchestration / MAS^2 generation | 12+ | Structural compiler and guard layer come first |
| eBPF or ML-based anomaly detection | 12+ | Requires richer runtime data collection and policy calibration |

## Open Questions Deferred to Later Planning

- Whether `AutonomyError` and related types should live in `mister-smith-core` or stay scoped to
  domain crates
- Whether operator-facing autonomy views are CLI-first, dashboard-first, or both
- Whether delegation enforcement stays fully JWT/claim-based in Phase 10 or adds an internal
  attenuation token format in the same phase

# Research: Phase 2 Runtime and Async Infrastructure Contracts

## Decision 1: Gate 2 evidence scope for this phase

- Decision: Use documentation consistency evidence commands for Gate 2 in this feature; defer crate compile gates until runtime implementation crates exist.
- Rationale: This phase is contract planning only and intentionally excludes runtime implementation.
- Alternatives considered: Require runtime compile checks now (rejected: not feasible for doc-only scope).

## Decision 2: Canonical Phase 2 source anchors

- Decision: Treat `tokio-runtime.md`, `monitoring-and-health.md`, `supervision-and-events.md`, `async-patterns.md`, `connection-management.md`, and `observability-monitoring-framework.md` as primary anchors.
- Rationale: These files cover all Phase 2 contract domains and are already referenced by roadmap/deep-dive docs.
- Alternatives considered: Include broad cross-domain docs as equal authority (rejected: introduces ambiguity/noise).

## Decision 3: Consistency strictness across active vs legacy references

- Decision: Apply strict terminology consistency in active Phase 2 references; permit legacy illustrative snippets only with canonical-link notes.
- Rationale: Maintains practical governance while reducing false-positive drift alarms.
- Alternatives considered: Ignore legacy snippets entirely (rejected: hides potential confusion).

## Decision 4: Async performance contract framing

- Decision: Express performance expectations as bounded resources, explicit backpressure, and retry/timeout/circuit-breaker policy clarity.
- Rationale: Aligns with constitution performance principle without requiring implementation-specific benchmarks.
- Alternatives considered: Add numeric latency/throughput targets in this phase (rejected: premature without runtime implementations).

## Decision 5: Contract artifact format

- Decision: Publish a phase-local markdown contract baseline under `contracts/phase2-runtime-async-contracts.md`.
- Rationale: Phase 2 defines architecture contracts rather than executable public APIs.
- Alternatives considered: OpenAPI/proto contracts (rejected: not representative of internal runtime contracts).

## Decision 6: Scope boundary enforcement

- Decision: Explicitly keep actor protocol semantics, external transport behavior, security policy enforcement, and persistence internals out of scope.
- Rationale: Protects phase boundaries and prevents implementation leakage.
- Alternatives considered: Partial inclusion of actor/external behavior (rejected: violates roadmap layering).

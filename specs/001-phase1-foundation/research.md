# Research: Phase 1 Foundation Contracts

## Decision 1: Canonical source authority

- Decision: Treat `ROADMAP.md`, `plans/roadmap-phases/phase-1-foundation.md`, and designated canonical files in `spec/core-architecture/` + `spec/operations/` as the authoritative source set.
- Rationale: Prevents drift from historical implementation planning docs that may contain outdated snippets.
- Alternatives considered: Use batch planning docs as equal authority (rejected: conflicts with validated canonical baseline).

## Decision 2: Configuration domain enforcement model

- Decision: Enforce runtime/agent/transport/security configuration domains without requiring exact struct names across all docs.
- Rationale: Canonical docs mix structural and operational views; domain-level consistency is stable and testable.
- Alternatives considered: Hard-require exact struct names in every file (rejected: brittle and not aligned with clarified requirement).

## Decision 3: Cross-document consistency strictness

- Decision: Apply strict consistency to active Phase 1 references; allow legacy illustrative snippets only with explicit canonical-reference note.
- Rationale: Balances immediate correctness with practical migration of older examples.
- Alternatives considered: Full hard fail for any mismatch including legacy snippets (rejected: high maintenance noise).

## Decision 4: Required evidence set

- Decision: Use Gate 1 compile checks plus four `rg` consistency checks as mandatory evidence.
- Rationale: Measurable, reproducible, and directly tied to spec acceptance criteria.
- Alternatives considered: Narrative review only (rejected: not deterministic).

## Decision 5: Contract artifact format

- Decision: Publish a feature-local contract baseline in markdown under `contracts/phase1-contract-baseline.md`.
- Rationale: This feature defines conceptual contracts, not executable APIs; markdown preserves traceability.
- Alternatives considered: OpenAPI/Proto schemas (rejected: not representative of Phase 1 scope).

## Decision 6: Scope protection for performance and runtime

- Decision: Explicitly prohibit runtime behavior additions and treat compile-time contract clarity as the performance safeguard for this phase.
- Rationale: Keeps feature aligned with Phase 1 dependency boundaries.
- Alternatives considered: Include runtime prototypes for validation (rejected: violates out-of-scope constraints).

# Mister Smith Constitution

## Core Principles

### I. Code Quality Is a Delivery Requirement

Every change must improve or preserve long-term maintainability.

- Code and specs must be clear, locally understandable, and aligned with established repository patterns.
- Duplication must be removed or intentionally justified when constraints require it.
- Public contracts (APIs, schemas, message formats, interfaces) must be explicit, versioned, and documented.
- Error handling must be explicit; silent failure paths and hidden fallbacks are not acceptable.

### II. Testing Is the Source of Truth

Behavior is only accepted when validated by tests at the right level.

- New behavior requires tests that prove expected outcomes and edge cases.
- Bug fixes require regression tests that fail before the fix and pass after it.
- Contract, integration, and unit tests must be chosen based on risk surface, not convenience.
- CI must remain green for all touched domains before merge.

### III. User Experience Must Be Consistent

User-facing behavior must be predictable across interfaces and releases.

- Similar workflows must use consistent terminology, response shapes, and error semantics.
- Changes to user-visible behavior require clear migration notes and updated documentation.
- Defaults must be safe, sensible, and aligned with prior user expectations.
- Accessibility, readability, and task clarity are required for user-facing documentation and UI surfaces.

### IV. Performance Is a Non-Negotiable Constraint

Performance and resource efficiency are first-class requirements, not afterthoughts.

- Features must define measurable performance goals before implementation (latency, throughput, memory, startup time, or equivalent).
- Changes that affect hot paths require benchmark evidence or production-like profiling.
- Regressions beyond agreed budgets require explicit approval and a mitigation plan.
- Backpressure, bounded resource usage, and graceful degradation are required for distributed workloads.

### V. Governance Through Enforceable Quality Gates

Principles are binding only when enforced in workflow and review.

- Pull requests must include evidence for quality, test coverage, UX impact, and performance impact.
- Reviewers must block merges that violate any core principle without an approved exception.
- Exceptions must be time-boxed, documented, and tracked to closure.
- Planning artifacts must include constitution checks before implementation begins.

## Standards and Quality Gates

### Required Evidence Per Change

- **Code Quality**: Lint/format checks pass; architectural consistency with existing modules is demonstrated.
- **Testing**: Appropriate unit/integration/contract coverage is added or updated for changed behavior.
- **UX Consistency**: User-visible impacts are documented, and consistency with existing patterns is verified.
- **Performance**: Performance-sensitive changes include benchmark/profiling evidence against defined budgets.

### Merge Blocking Conditions

- Failing tests, broken builds, or unresolved lint/format violations.
- Unexplained user-facing behavior changes or inconsistent interface semantics.
- Missing performance evidence for performance-sensitive changes.
- Missing migration guidance when contracts or user workflows are changed.

## Development Workflow and Review Process

- Plan work with explicit constitution checks before implementation.
- Implement using existing repository patterns and reuse existing abstractions before introducing new ones.
- Validate with automated checks relevant to touched areas.
- Document trade-offs and residual risk in the change description.
- Merge only after principle compliance is confirmed in review.

## Governance

This constitution supersedes conflicting local practices for planning, implementation, and review.

Amendments require:

- A written proposal describing the change and rationale.
- Explicit impact analysis on existing specs, plans, and workflows.
- Approval from repository maintainers.
- A migration/update plan for any affected documentation or templates.

Compliance is verified during planning, code review, and release readiness checks.

**Version**: 1.0.0 | **Ratified**: 2026-03-04 | **Last Amended**: 2026-03-04

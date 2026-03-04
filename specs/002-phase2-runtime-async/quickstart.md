# Quickstart: Validate Phase 2 Runtime and Async Contracts

## Prerequisites

- Run from repository root: `/Users/matthewmaggio/Mister-Smith`
- Active branch: `002-phase2-runtime-async`
- `rg` (ripgrep) available
- Node.js tooling with `npx` available

## 1. Runtime Lifecycle Contract Checks

```bash
rg -n "RuntimeManager|graceful shutdown|shutdown" spec/core-architecture/tokio-runtime.md spec/core-architecture/runtime-and-errors.md
```

Expected outcome:

- Runtime lifecycle and shutdown semantics are explicitly represented.

## 2. Monitoring and Event Contract Checks

```bash
rg -n "HealthMonitor|Metrics|health check|metrics" spec/core-architecture/monitoring-and-health.md spec/operations/observability-monitoring-framework.md
rg -n "EventBus|SystemEvent|SupervisionEvent|event" spec/core-architecture/supervision-and-events.md spec/core-architecture/monitoring-and-health.md
```

Expected outcome:

- Monitoring and event contracts are discoverable and aligned.

## 3. Async Utility and Resource Contract Checks

```bash
rg -n "TaskExecutor|CircuitBreaker|timeout|retry|backpressure" spec/core-architecture/async-patterns.md spec/core-architecture/module-organization-type-system.md
rg -n "ConnectionPool|ResourceManager|health" spec/data-management/connection-management.md spec/core-architecture/component-architecture.md
```

Expected outcome:

- Async control and resource lifecycle contracts are represented with bounded-behavior semantics.

## 4. Artifact Quality Check

```bash
npx markdownlint-cli2 "specs/002-phase2-runtime-async/*.md" --config .markdownlint.json
npx markdownlint-cli2 "specs/002-phase2-runtime-async/contracts/*.md" --config .markdownlint.json
npx markdownlint-cli2 "specs/002-phase2-runtime-async/checklists/*.md" --config .markdownlint.json
```

Expected outcome:

- Zero markdown lint errors.

## 5. Interpretation Rules

- This phase validates documentation contracts, not runtime implementation behavior.
- Any unresolved terminology drift across active references blocks progression to later planning.

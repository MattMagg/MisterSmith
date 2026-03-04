# Phase 2 Runtime and Async Contract Baseline

## 1. Runtime Lifecycle Contracts

Required contract domains:

- Runtime startup lifecycle
- Runtime steady-state behavior
- Graceful shutdown coordination
- Shutdown failure handling boundaries

Contract rules:

- Runtime lifecycle semantics must be explicit and phase-bounded.
- Actor protocol and external transport execution behavior are excluded from this phase.

## 2. Monitoring and Event Contracts

Required contract domains:

- Health check registration and evaluation
- Metrics collection and reporting interfaces
- In-process event bus and lifecycle events

Contract rules:

- Monitoring and event terminology must remain consistent across core and operations references.
- Critical lifecycle transitions must have observable contract coverage.

## 3. Async Utility Contracts

Required contract domains:

- Task execution coordination
- Timeout and retry policy semantics
- Circuit-breaker behavior
- Backpressure signaling and handling

Contract rules:

- Async utility contracts must emphasize bounded resource behavior.
- Failure and degraded-operation semantics must be explicit.

## 4. Resource Lifecycle Contracts

Required contract domains:

- Resource acquisition and release
- Pool lifecycle behavior
- Resource health check semantics

Contract rules:

- Resource contracts must support downstream transport and persistence reuse.
- Outage/degraded acquisition behavior must be documented.

## 5. Governance and Evidence

Required evidence command families:

- Runtime lifecycle consistency checks
- Monitoring and event contract checks
- Async utility and pooling contract checks
- Markdown quality checks for phase artifacts

Legacy snippet policy:

- Active references are strict.
- Legacy illustrative references are acceptable only when explicitly linked to canonical definitions.

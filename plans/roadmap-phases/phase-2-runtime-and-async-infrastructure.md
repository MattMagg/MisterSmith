# Phase 2: Runtime and Async Infrastructure

## Purpose and Scope

Stand up the async execution substrate used by the full framework: runtime lifecycle, health and
metrics plumbing, in-process event bus, reusable async primitives, and resource management.

### In Scope

- Tokio runtime manager and shutdown control
- Health monitoring and metrics registration
- Internal event system for component observability
- Async execution utility patterns (timeouts, retries, circuit breakers)
- Generic resource and connection lifecycle abstractions

### Out of Scope

- Actor protocol semantics
- External transport protocol handlers
- Security policy enforcement

## Inputs and Dependencies

### Upstream Dependencies

- Phase 1 (types, traits, configuration)

### Key Source Inputs

- `ROADMAP.md` Phase 2 and Gate 2
- `VERSION_REFERENCE.md` runtime/library baseline (Tokio 1.49.0, tracing stack)
- `VALIDATION_REPORT.md` for async-pattern and supervision reconciliations

### Required Specification Anchors

- `spec/core-architecture/tokio-runtime.md`
- `spec/core-architecture/monitoring-and-health.md`
- `spec/core-architecture/supervision-and-events.md`
- `spec/core-architecture/async-patterns.md`
- `spec/data-management/connection-management.md`
- `spec/core-architecture/component-architecture.md`
- `spec/operations/observability-monitoring-framework.md`

## Outputs and Downstream Consumers

### Produces

- Runtime lifecycle contract (`RuntimeManager`) and shutdown sequence
- System-wide health and metrics integration points
- In-process event bus model for supervision and diagnostics
- Shared async control patterns for task execution and backpressure
- Resource pooling contract reused by transport and persistence layers

### Consumed By

- Phase 3 actor spawning and supervision eventing
- Phase 4 transport connection handling and telemetry
- Phase 5 security audit logging and certificate health monitoring
- Phase 6 persistence pools and health checks
- Phase 8 observability and process lifecycle orchestration

## Gate Criteria and Validation

### Gate Criteria

- Runtime lifecycle supports startup, steady state, and graceful shutdown
- Monitoring surfaces align with operations-level observability specs
- Event types and emission paths cover critical lifecycle transitions
- Async guidance matches current Tokio/tracing idioms in repo specs
- Resource management guidance is reusable across transport and persistence

### How to validate

- `rg -n "pub async fn start_system|pub async fn graceful_shutdown|DEFAULT_SHUTDOWN_TIMEOUT" spec/core-architecture/tokio-runtime.md`
- `rg -n "pub struct HealthMonitor|pub trait HealthCheck|pub struct MetricsCollector" spec/core-architecture/monitoring-and-health.md`
- `rg -n "HealthMonitor|MetricsCollector|HealthCheck" spec/operations/observability-monitoring-framework.md` (cross-domain alignment)
- `rg -n "pub enum SystemEventType|pub async fn publish|subscribe_broadcast" spec/core-architecture/supervision-and-events.md`
- `rg -n "Started|Stopping|Stopped|HealthCheckPassed|HealthCheckFailed" spec/core-architecture/supervision-and-events.md` (lifecycle transition coverage)
- `rg -n "#\\[async_trait\\]|tracing::|#\\[instrument\\]" spec/core-architecture/tokio-runtime.md spec/core-architecture/async-patterns.md` (current idioms)
- `rg -n "spawned_tasks_count|injection_queue_depth" spec/core-architecture/tokio-runtime.md spec/core-architecture/async-patterns.md` (expect zero — no deprecated APIs)
- `rg -n "ConnectionPool<R:" spec/core-architecture/component-architecture.md` (generic resource pool)
- `rg -n "calculate_optimal_pool_size|PoolSizeRecommendation" spec/data-management/connection-management.md` (pool sizing)

### Validation Evidence

- `RuntimeManager` struct referenced in both `tokio-runtime.md` and `component-architecture.md`
- `EventBus` and `HealthMonitor` wired together via `MonitoringSystem` in `monitoring-and-health.md`
- `SystemEventType` variants (Started, Stopping, Stopped) match Phase 3 consumption patterns in ROADMAP.md 3.1–3.2
- Generic `ConnectionPool<R: Resource>` in `component-architecture.md` is parameterized (not hardcoded to a specific backend)

## Official-Doc Best Practices

- Prefer a single Tokio runtime boundary and explicit shutdown coordination ([Tokio runtime](https://docs.rs/tokio/1.49.0/tokio/runtime/) and [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown)).
- Use bounded channels and explicit backpressure behavior for internal event fanout ([Tokio broadcast](https://docs.rs/tokio/1.49.0/tokio/sync/broadcast/)).
- Keep observability structured and async-safe with `tracing` spans/events, not ad hoc logging ([tracing crate](https://docs.rs/tracing/0.1.44/tracing/)).

## Known Risks / Unknowns

### Risks

- Runtime shutdown semantics can diverge from process-management expectations
- Metrics and event taxonomies can drift between core and operations docs
- Async utility patterns can become inconsistent across specifications
- `connection-management.md` uses CLASS/FUNCTION pseudocode while all other Phase 2 specs use concrete Rust — the generic `ConnectionPool<R: Resource>` contract is actually in `component-architecture.md`

### Required Follow-ups

- Keep shutdown semantics synchronized with `process-management-specifications.md`
- Revalidate tracing/metrics assumptions whenever dependency specs are updated
- When implementing resource management, use `component-architecture.md` for the Rust trait/struct contracts and `connection-management.md` for the domain-specific pool sizing algorithms

## Authoritative Spec Files

- `spec/core-architecture/tokio-runtime.md` — RuntimeManager, shutdown, task spawning
- `spec/core-architecture/monitoring-and-health.md` — HealthMonitor, MetricsCollector, health checks
- `spec/core-architecture/supervision-and-events.md` — EventBus, SystemEvent, event handlers
- `spec/core-architecture/async-patterns.md` — TaskExecutor, TaskGuard, circuit breaker, backpressure
- `spec/core-architecture/component-architecture.md` — generic `ConnectionPool<R: Resource>` (canonical resource pool contract)
- `spec/data-management/connection-management.md` — pool sizing, transaction management (pseudocode; Rust contract is in `component-architecture.md`)
- `spec/operations/observability-monitoring-framework.md` — OpenTelemetry, Prometheus, OTLP

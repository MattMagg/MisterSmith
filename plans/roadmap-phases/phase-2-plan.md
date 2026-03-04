# Phase 2 Implementation Plan: Runtime and Async Infrastructure

**Date**: 2026-03-04 | **Upstream**: Phase 1 (Foundation) | **Downstream**: Phases 3-8

## Summary

Phase 2 stands up the async execution substrate: Tokio runtime lifecycle, health and metrics plumbing, in-process event bus, reusable async primitives, and generic resource management. This layer is consumed by every subsequent phase -- actors, transport, security, persistence, agents, and operations all depend on the runtime, monitoring, and async patterns produced here.

## Technical Context

**Language/Version**: Rust, MSRV 1.88.0 (binding constraint: async-nats 0.46.0)
**Runtime**: Tokio 1.49.0 (`full` feature set: rt-multi-thread, io, net, time, sync, fs, process, signal)
**Observability**: tracing 0.1.44, tracing-subscriber 0.3.22, metrics 0.24.3, metrics-exporter-prometheus 0.18.1
**Testing**: `cargo test`, `#[tokio::test]`, criterion 0.5 (async benchmarks)
**Target Platform**: Linux server (Kubernetes), macOS development
**Project Type**: Rust workspace library crates
**Constraints**: Single Tokio runtime boundary per process; bounded channels for all internal fanout; no blocking I/O on worker threads

### Phase 2 Crate Dependencies

```toml
[workspace.dependencies]
# Phase 2 direct dependencies
tokio = { version = "1.49.0", features = ["full"] }
futures = "0.3.32"
async-trait = "0.1.83"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "1.0.69"
uuid = { version = "1.11.0", features = ["v4", "serde"] }
dashmap = "6.1.0"
num_cpus = "1.0"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.22", features = ["env-filter"] }
metrics = "0.24.3"
metrics-exporter-prometheus = "0.18.1"
crossbeam = "0.8"
parking_lot = "0.12"
```

## Constitution Check

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Canonical Single Source of Truth | Pass | All types reference `type-definitions.md`; event types from `supervision-and-events.md`; pool contract from `component-architecture.md` |
| II. Spec-First Design | Pass | 6 authoritative spec files define all public API surfaces for Phase 2 |
| III. Phase-Gated Build Order | Pass | Phase 2 depends only on Phase 1 (types, traits, config); Gate 1 must pass before Phase 2 begins |
| IV. Model-Agnostic Architecture | Pass | Phase 2 contains no LLM-specific code -- pure runtime infrastructure |
| V. Erlang/OTP-Style Fault Tolerance | Pass | Event system and failure detector lay groundwork for Phase 3 supervision trees |
| VI. Evidence-Based Validation | Pass | Gate 2 criteria are executable (runtime start/stop, health probe, event flow, metrics collection) |
| VII. Explicit Dependency Management | Pass | All crate versions from `VERSION_REFERENCE.md`; workspace dependencies documented above |

## Subphase-to-Crate Map

| Subphase | Crate | Primary Spec | Depends On |
|----------|-------|-------------|------------|
| 2.1 | `mister-smith-runtime` | `spec/core-architecture/tokio-runtime.md` | 1.1 (types), 1.3 (config) |
| 2.2 | `mister-smith-monitoring` | `spec/core-architecture/monitoring-and-health.md` | 2.1 (async health checks need runtime) |
| 2.3 | `mister-smith-events` | `spec/core-architecture/supervision-and-events.md` | 2.1 (runtime), 2.2 (events emit metrics) |
| 2.4 | `mister-smith-async` | `spec/core-architecture/async-patterns.md` | 2.1 (runtime), 2.2 (monitoring) |
| 2.5 | `mister-smith-resources` | `spec/core-architecture/component-architecture.md` + `spec/data-management/connection-management.md` | 2.2 (health), 1.3 (pool config) |

### Cross-Reference: Observability Layer

`spec/operations/observability-monitoring-framework.md` defines the OpenTelemetry/Prometheus integration that Phase 2 monitoring prepares surface area for. The actual OTLP exporter wiring is Phase 8.1, but the `MetricsRegistry`, `HealthMonitor`, and `MetricsBackend` trait are Phase 2 outputs that Phase 8.1 will consume.

### Cross-Reference: Batch 1 Plans

`plans/batch1-core-architecture/agent01-system-architecture-implementation.md` covers the system architecture implementation. Its crate dependency matrix and workspace member list (line 36-43) are outdated (Tokio 1.45.1, metrics 0.23, MSRV 1.75). Phase 2 implementation must use the updated versions from `VERSION_REFERENCE.md`.

## Types and Structs Per Crate

### 2.1 `mister-smith-runtime`

Source: `spec/core-architecture/tokio-runtime.md`

**Structs**:
- `RuntimeConfig` -- Serde-serializable runtime configuration (worker_threads, max_blocking_threads, thread_keep_alive, thread_stack_size, feature enables)
- `RuntimeManager` -- Holds Arc<Runtime>, shutdown signal, health monitor, metrics collector, supervision tree, event bus, task handles
- `RuntimePerformanceMonitor` -- Wraps `tokio::runtime::Handle` for Tokio runtime metrics collection (workers, tasks, queues, budget)
- `RuntimeTuning` -- Static methods for workload-specific configurations (websocket, data pipeline, agent system)
- `TaskScheduler` -- Concurrency-limited task scheduling with priority queue
- `PrioritizedTask` -- Priority + boxed task closure
- `RuntimeBestPractices` -- Static utility for optimal worker thread calculation

**Enums**:
- `RuntimeError` -- BuildFailed, StartupFailed, ShutdownFailed, ConfigurationInvalid
- `ErrorSeverity` -- Low, Medium, High, Critical
- `RecoveryStrategy` -- Retry, Restart, Escalate, Reload, CircuitBreaker, Failover, Ignore
- `WorkloadType` -- CpuBound, IoBound, Mixed, LatencySensitive

**Constants**:
- `DEFAULT_WORKER_THREADS`, `DEFAULT_MAX_BLOCKING_THREADS`, `DEFAULT_THREAD_KEEP_ALIVE`, `DEFAULT_THREAD_STACK_SIZE`
- `DEFAULT_SHUTDOWN_TIMEOUT` (30s)
- `HIGH_THROUGHPUT_WORKERS`, `CPU_BOUND_WORKERS`, `IO_BOUND_WORKERS`

**Key Methods**:
- `RuntimeConfig::build_runtime()` -> `Result<Runtime, RuntimeError>`
- `RuntimeConfig::cpu_bound()`, `io_bound()`, `high_throughput()` -- preset constructors
- `RuntimeManager::initialize(config)` -> `Result<Self, RuntimeError>`
- `RuntimeManager::start_system()` -- spawns health, metrics, supervision, signal handler tasks
- `RuntimeManager::graceful_shutdown()` -- signal, drain supervision, flush metrics, join tasks, shutdown_timeout
- `RuntimePerformanceMonitor::collect_metrics()` -- populates metrics 0.24 gauges/counters

### 2.2 `mister-smith-monitoring`

Source: `spec/core-architecture/monitoring-and-health.md`

**Structs**:
- `HealthStatus` -- component_id, status, last_check (SystemTime), message, metadata
- `HealthMonitor` -- check_interval, health_checks registry (RwLock<Vec>), status cache, optional event bus
- `RuntimeHealthCheck` -- health check impl for runtime responsiveness
- `DatabaseHealthCheck` -- health check impl for DB connectivity (placeholder; Phase 6 provides real impl)
- `AgentSystemHealthCheck` -- health check impl for actor system (placeholder; Phase 3 provides real impl)
- `MetricsCollector` -- internal metrics buffer (HashMap<String, Vec<Metric>>) with flush interval
- `Metric` -- name, value, timestamp, tags
- `MonitoringSystem` -- coordinator wiring HealthMonitor + MetricsCollector + EventBus
- `SystemComponents` -- Optional references to RuntimeManager, ActorSystem, DatabasePool, SupervisionTree
- `PrometheusBackend` -- MetricsBackend impl for Prometheus push gateway

**Enums**:
- `Status` -- Healthy, Degraded, Unhealthy, Unknown
- `MetricValue` -- Counter(u64), Gauge(f64), Histogram(Vec<f64>), Summary { sum, count }

**Traits**:
- `HealthCheck` -- `async fn check()`, `fn component_id()`, `fn check_interval()`
- `MetricsBackend` -- `async fn send_metrics(Vec<Metric>)`

**Newtypes**:
- `ComponentId(String)` -- used throughout events and monitoring

### 2.3 `mister-smith-events`

Source: `spec/core-architecture/supervision-and-events.md`

**Structs**:
- `SystemEvent` -- id (Uuid), timestamp (SystemTime), source (ComponentId), event_type, payload (serde_json::Value), correlation_id, causation_id
- `EventFilter` -- event_types, sources, correlation_ids (all Option<Vec>)
- `EventBus` -- subscribers (TypeId -> Vec<Arc<dyn EventHandler>>), event_queue (VecDeque), broadcast_sender, optional event_store, metrics_collector
- `EventBuilder` -- builder pattern for SystemEvent construction with payload serialization
- `InMemoryEventStore` -- Vec<SystemEvent> behind RwLock for testing

**Enums**:
- `EventType` -- System(SystemEventType), Agent(AgentEventType), Tool(ToolEventType), Custom(String)
- `SystemEventType` -- Started, Stopping, Stopped, HealthCheckPassed, HealthCheckFailed, ConfigurationChanged, ResourcePoolExhausted, CircuitBreakerOpen, CircuitBreakerClosed
- `AgentEventType` -- Created, Started, Stopped, Failed, MessageReceived, MessageProcessed, StateChanged
- `ToolEventType` -- Registered, Unregistered, ExecutionStarted, ExecutionCompleted, ExecutionFailed, PermissionDenied
- `EventError` -- PublishFailed, Timeout, SubscriptionFailed, ValidationFailed(String), CorrelationFailed(String), StoreFailed(String), SerializationFailed(String)

**Traits**:
- `EventHandler` -- `async fn handle_event(SystemEvent)`, `fn event_filter() -> Option<EventFilter>`
- `EventStore` -- `async fn append()`, `async fn query()`, `async fn get_by_id()`, `async fn get_by_correlation()`

**Key Design Decisions**:
- Broadcast channel capacity: 10,000 (from spec)
- Uses `tokio::sync::broadcast` for real-time fanout + `VecDeque` for async processing
- Dead letter handling: events that fail to reach any subscriber are logged and routed to `DeadLetterQueue` (referenced in `component-architecture.md` but needs explicit implementation)
- SystemTime (not Instant) for timestamps -- serializable, cross-process comparable

### 2.4 `mister-smith-async`

Source: `spec/core-architecture/async-patterns.md` (Sections 1-2, 4-5)

**Structs**:
- `TaskExecutor` -- task queue, worker handles, semaphore, metrics, shutdown broadcast, circuit breaker, task pool, error strategy
- `TaskHandle<T>` -- task_id, oneshot::Receiver, JoinHandle for tracking/cancellation
- `TaskMetrics` -- AtomicU64 counters: total_submitted, completed, failed, currently_running, panics_recovered, circuit_breaker_trips
- `TaskPool<T>` -- object pool for task reuse (factory + max_size)
- `CircuitBreaker` -- failure_count, last_failure_time, state, failure_threshold, recovery_timeout, half_open_max_calls
- `RetryPolicy` -- max_attempts, base_delay, max_delay, backoff_multiplier
- `StreamProcessor<T>` -- input stream, processor chain, output sink, backpressure config, buffer, metrics
- `StreamMetrics` -- AtomicU64: items_processed, items_dropped, backpressure_events, processing_errors
- `BackpressureConfig` -- strategy, wait_duration, buffer_size, threshold
- `TaskGuard` -- RAII guard with JoinHandle abort + cleanup closure on Drop
- `DeadlockPreventingMutex<T>` -- Tokio mutex with acquisition ordering + timeout
- `AsyncBarrier` -- wrapper around tokio::sync::Barrier
- `CountdownLatch` -- AtomicUsize count + tokio::sync::Notify for multi-task coordination
- `MpmcChannel<T>` -- crossbeam bounded/unbounded channel wrapper

**Enums**:
- `TaskError` -- ExecutorShutdown, TaskCancelled, ExecutionFailed(String), TimedOut, PoisonedLock, PanicOccurred(String), CircuitBreakerOpen
- `ErrorStrategy` -- StopOnError, LogAndContinue, RetryWithBackoff, CircuitBreaker
- `TaskPriority` -- Low(0), Normal(1), High(2), Critical(3)
- `BackpressureStrategy` -- Wait, Drop, Buffer, Block
- `CircuitState` -- Closed, Open, HalfOpen

**Traits**:
- `AsyncTask` -- `async fn execute()`, `fn priority()`, `fn timeout()`, `fn retry_policy()`, `fn task_id()`
- `Processor<T>` -- `async fn process(T) -> Result<T>`, `fn name()`

**Key Design Decisions**:
- CircuitBreaker uses `std::sync` primitives (not tokio) for lock-free fast-path checking
- TaskExecutor limits concurrency via Semaphore
- Retry uses exponential backoff with jitter to prevent thundering herd
- Stream processing supports chained processors with configurable backpressure

### 2.5 `mister-smith-resources`

Source: `spec/core-architecture/component-architecture.md` (Rust contracts) + `spec/data-management/connection-management.md` (sizing algorithms)

**Structs**:
- `ConnectionPool<R: Resource>` -- pool (Arc<Mutex<VecDeque<R>>>), max_size, min_size, acquire_timeout, idle_timeout, health_check_interval, tls_config
- `PooledResource<R>` -- RAII wrapper that returns resource to pool on drop
- `ResourceManager` -- connection_pools (HashMap<PoolType, ConnectionPool>), memory_manager, file_handles, thread_pools, security_context
- `PoolSizeRecommendation` -- recommended_size, min_connections, max_connections, reasoning
- `ConnectionPoolSizer` -- static methods for Little's Law-based pool sizing
- `PoolSizeTemplate` -- environment-specific sizing (dev/staging/production)

**Enums**:
- `PoolType` -- (to be defined per resource kind)
- `EnvironmentType` -- Development, Staging, Production

**Traits** (from Phase 1.2, consumed here):
- `Resource` -- `async fn acquire()`, `async fn release()`, `fn is_healthy()`, `async fn secure_handshake()`

**Key Design Decisions**:
- The generic `ConnectionPool<R: Resource>` contract lives in `component-architecture.md` and is the canonical Rust impl
- `connection-management.md` provides domain-specific pool sizing algorithms (Little's Law) and transaction management in pseudocode
- Pool health checks run at configurable intervals; unhealthy resources are evicted and replaced
- Idle timeout evicts unused connections to prevent stale resource accumulation

## Key Implementation Decisions

### 1. Runtime Configuration Strategy

The spec defines `RuntimeConfig` with preset constructors (cpu_bound, io_bound, high_throughput, agent_system). The implementation should:
- Load base config from `mister-smith-config` (Phase 1.3)
- Allow preset selection via config file or environment variable
- Validate thread counts against system capabilities at startup
- Log effective configuration at info level on startup

### 2. Event Bus Channel Sizing

The spec hardcodes `broadcast::channel(10000)`. This should be:
- Configurable via `EventBusConfig` struct
- Defaulting to 10,000
- Documented that `broadcast::channel` will drop oldest messages for lagged receivers (Tokio semantics)
- Dead letter handling must be implemented for events that fail subscriber delivery

### 3. Metrics Backend Choice

Phase 2 implements:
- Internal `MetricsCollector` (buffered, flushed periodically) for the framework's own metric types
- `MetricsBackend` trait for pluggable export (Prometheus push gateway as reference impl)
- Integration with `metrics` 0.24 crate macros (`counter!`, `gauge!`, `histogram!`) for Tokio runtime metrics

Phase 8.1 adds:
- OpenTelemetry SDK initialization (OTLP exporter)
- `tracing-opentelemetry` bridge
- Full Prometheus scrape endpoint via HTTP (Phase 4.4 Axum)

### 4. Phi Accrual Failure Detector

Referenced in `supervision-and-events.md`, `supervision-trees.md`, and `type-definitions.md`. Phase 2 provides the `FailureDetector` struct with phi accrual support. The full `PhiAccrualFailureDetector` algorithm should:
- Track heartbeat arrival times per node
- Compute phi values using exponential distribution
- Use configurable threshold (spec references `CONFIGURABLE_THRESHOLD`)
- Emit `SystemEventType::HealthCheckFailed` events when phi exceeds threshold

### 5. Shutdown Coordination

`RuntimeManager::graceful_shutdown()` follows this sequence:
1. Set `AtomicBool` shutdown signal
2. Shutdown supervision tree (drain children first)
3. Flush metrics collector
4. Join all spawned task handles
5. `runtime.shutdown_timeout(DEFAULT_SHUTDOWN_TIMEOUT)` (30s)

This sequence must coordinate with `spec/operations/process-management-specifications.md` signal handling (SIGTERM, SIGINT).

## Gate 2 Criteria

From `ROADMAP.md`:

> The runtime starts, shuts down gracefully, and reports health. Events flow through the bus. Metrics are collected. You can write `#[tokio::test]` tests that exercise the async patterns. No actors, no agents, no external I/O yet.

### Concrete Validation Commands

```bash
# Runtime lifecycle
cargo test -p mister-smith-runtime -- --test runtime_starts_and_shuts_down
cargo test -p mister-smith-runtime -- --test graceful_shutdown_with_timeout

# Health monitoring
cargo test -p mister-smith-monitoring -- --test health_check_registration_and_execution
cargo test -p mister-smith-monitoring -- --test system_health_aggregation

# Event bus
cargo test -p mister-smith-events -- --test publish_and_subscribe_roundtrip
cargo test -p mister-smith-events -- --test broadcast_subscriber_receives_events
cargo test -p mister-smith-events -- --test event_filter_matches_correctly
cargo test -p mister-smith-events -- --test dead_letter_handling

# Async patterns
cargo test -p mister-smith-async -- --test task_executor_concurrent_execution
cargo test -p mister-smith-async -- --test circuit_breaker_state_transitions
cargo test -p mister-smith-async -- --test retry_with_backoff
cargo test -p mister-smith-async -- --test countdown_latch_synchronization
cargo test -p mister-smith-async -- --test task_guard_cleanup_on_drop

# Resource management
cargo test -p mister-smith-resources -- --test connection_pool_acquire_release
cargo test -p mister-smith-resources -- --test pool_health_check_eviction
cargo test -p mister-smith-resources -- --test pool_size_recommendation

# Cross-crate integration
cargo test --workspace
cargo build --workspace
cargo clippy --workspace -- -D warnings
```

## Known Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| RuntimeManager holds Arc references to Phase 3 types (SupervisionTree) | Circular dependency between crates | Phase 2 RuntimeManager takes generic/trait-based references; concrete SupervisionTree wiring deferred to Phase 3 |
| metrics 0.24 API differs from 0.23 used in batch1 agent01 plan | API breakage in existing plans | Use 0.24 exclusively per VERSION_REFERENCE.md; batch1 plans are advisory, not binding |
| EventBus broadcast channel drops messages for lagged receivers | Silent data loss | Implement dead letter queue; document lag behavior; add `event.dropped` counter metric |
| PhiAccrualFailureDetector has no established Rust crate | Must implement from scratch | Algorithm is well-documented (Hayashibara et al.); implement with exponential distribution arrival model |
| Shutdown sequence must coordinate with process management (Phase 8) | Potential conflicts | Keep shutdown semantics synchronized; Phase 8.2 wraps Phase 2 RuntimeManager, does not replace it |

## Required Follow-ups

- When implementing 2.1, strip `SupervisionTree` and `EventBus` from `RuntimeManager` fields -- those are Phase 2.3/3.2 outputs. Use trait-based injection or builder pattern to wire them later.
- Revalidate metrics 0.24 API against `RuntimePerformanceMonitor::collect_metrics()` spec code (may need API adjustments for 0.24 macro signatures).
- `connection-management.md` pool sizing algorithms are pseudocode -- translate to Rust with property-based tests (proptest) to validate Little's Law math.
- Dead letter queue is referenced in `component-architecture.md` `EventBus` struct but not fully implemented in any spec. Implement as bounded `VecDeque` with configurable retention.

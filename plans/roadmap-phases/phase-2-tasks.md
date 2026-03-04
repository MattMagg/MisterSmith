# Tasks: Phase 2 -- Runtime and Async Infrastructure

**Input**: `plans/roadmap-phases/phase-2-plan.md`, `plans/roadmap-phases/phase-2-runtime-and-async-infrastructure.md`
**Prerequisites**: Phase 1 gate passed (core types compile, traits compile, config loads)

## Format: `[ID] [P?] Description with file path`

- **[P]**: Can run in parallel (different files, no dependencies on other tasks in the same phase)
- Include exact file paths in descriptions
- Tasks within a phase are ordered by dependency; [P] tasks at the same level can be parallelized

---

## Phase 1: Setup -- Workspace and Crate Scaffolding

**Purpose**: Create the 5 new crates, wire workspace dependencies, establish shared test utilities.

**Checkpoint**: `cargo build --workspace` compiles with all 5 new empty crates.

- [ ] T001 Add 5 new crate members to root `Cargo.toml` workspace: `mister-smith-runtime`, `mister-smith-monitoring`, `mister-smith-events`, `mister-smith-async`, `mister-smith-resources`
- [ ] T002 [P] Scaffold `mister-smith-runtime/Cargo.toml` with dependencies: tokio (full), serde, serde_json, thiserror, uuid, num_cpus, tracing, metrics, mister-smith-core, mister-smith-config
- [ ] T003 [P] Scaffold `mister-smith-monitoring/Cargo.toml` with dependencies: tokio, async-trait, serde, serde_json, thiserror, tracing, metrics, metrics-exporter-prometheus, mister-smith-core, mister-smith-runtime
- [ ] T004 [P] Scaffold `mister-smith-events/Cargo.toml` with dependencies: tokio (sync, time), async-trait, serde, serde_json, thiserror, uuid, tracing, mister-smith-core, mister-smith-monitoring
- [ ] T005 [P] Scaffold `mister-smith-async/Cargo.toml` with dependencies: tokio (sync, time, task), futures, async-trait, serde, serde_json, thiserror, uuid, tracing, crossbeam, parking_lot, mister-smith-core, mister-smith-runtime, mister-smith-monitoring
- [ ] T006 [P] Scaffold `mister-smith-resources/Cargo.toml` with dependencies: tokio (sync, time), async-trait, serde, thiserror, tracing, mister-smith-core, mister-smith-config, mister-smith-monitoring
- [ ] T007 [P] Create `mister-smith-runtime/src/lib.rs` with module declarations (runtime, metrics, tuning, scheduling, error)
- [ ] T008 [P] Create `mister-smith-monitoring/src/lib.rs` with module declarations (health, metrics, backend, system)
- [ ] T009 [P] Create `mister-smith-events/src/lib.rs` with module declarations (bus, types, handler, store, builder, error)
- [ ] T010 [P] Create `mister-smith-async/src/lib.rs` with module declarations (executor, task, circuit_breaker, retry, stream, sync, guard)
- [ ] T011 [P] Create `mister-smith-resources/src/lib.rs` with module declarations (pool, manager, sizing, health)
- [ ] T012 [P] Create shared test utilities crate or module: `tests/common/mod.rs` or `mister-smith-test-utils/` with mock HealthCheck, mock Resource, test RuntimeConfig factory
- [ ] T013 Verify `cargo build --workspace` compiles cleanly with all new crates (empty lib.rs stubs)

---

## Phase 2: Runtime Manager (`mister-smith-runtime`)

**Purpose**: Tokio runtime lifecycle -- builder pattern, configuration presets, shutdown coordination, task spawning, performance monitoring.

**Spec**: `spec/core-architecture/tokio-runtime.md`

**Checkpoint**: Runtime starts, runs, and shuts down gracefully. `RuntimePerformanceMonitor` collects Tokio metrics. `#[tokio::test]` tests pass.

### Error Types

- [ ] T014 Implement `RuntimeError` enum in `mister-smith-runtime/src/error.rs`: BuildFailed(io::Error), StartupFailed(String), ShutdownFailed(String), ConfigurationInvalid(String) -- per `tokio-runtime.md` line 112-122
- [ ] T015 Implement `ErrorSeverity` enum (Low, Medium, High, Critical) and `RecoveryStrategy` enum (Retry, Restart, Escalate, Reload, CircuitBreaker, Failover, Ignore) in `mister-smith-runtime/src/error.rs` -- per `tokio-runtime.md` line 125-142

### Runtime Configuration

- [ ] T016 Implement `RuntimeConfig` struct in `mister-smith-runtime/src/config.rs` with Serialize/Deserialize: worker_threads, max_blocking_threads, thread_keep_alive, thread_stack_size, enable_all, enable_time, enable_io -- per `tokio-runtime.md` line 203-226
- [ ] T017 Implement `RuntimeConfig::default()` with DEFAULT_WORKER_THREADS, DEFAULT_MAX_BLOCKING_THREADS (512), DEFAULT_THREAD_KEEP_ALIVE (60s), DEFAULT_THREAD_STACK_SIZE (2MB) -- per `tokio-runtime.md` line 193-197, 214-226
- [ ] T018 [P] Implement preset constructors `RuntimeConfig::cpu_bound()`, `io_bound()`, `high_throughput()` -- per `tokio-runtime.md` line 229-260
- [ ] T019 Implement `RuntimeConfig::build_runtime()` -> `Result<Runtime, RuntimeError>` using `tokio::runtime::Builder::new_multi_thread()` -- per `tokio-runtime.md` line 262-289
- [ ] T020 Implement `WorkloadType` enum and `RuntimeBestPractices::optimal_worker_threads()` in `mister-smith-runtime/src/tuning.rs` -- per `tokio-runtime.md` line 819-825, 869-875
- [ ] T021 [P] Implement `RuntimeTuning` with websocket_server_config(), data_pipeline_config(), agent_system_config() in `mister-smith-runtime/src/tuning.rs` -- per `tokio-runtime.md` line 508-546

### Runtime Manager Core

- [ ] T022 Implement `RuntimeManager` struct in `mister-smith-runtime/src/manager.rs` -- Arc<Runtime>, shutdown_signal (AtomicBool), task handles (Vec<JoinHandle<()>>). NOTE: Do NOT include HealthMonitor, MetricsCollector, SupervisionTree, EventBus as direct fields -- those are injected later via builder pattern or setter methods to avoid circular crate dependencies. Per `tokio-runtime.md` line 308-316, adapted for Phase 2 layering.
- [ ] T023 Implement `RuntimeManager::builder()` pattern returning a `RuntimeManagerBuilder` that accepts optional health_monitor, metrics_collector handles via trait objects or Arc<dyn Trait>
- [ ] T024 Implement `RuntimeManager::initialize(config: RuntimeConfig)` -> `Result<Self, RuntimeError>` -- builds Tokio runtime, creates shutdown signal -- per `tokio-runtime.md` line 319-336
- [ ] T025 Implement `RuntimeManager::start_system()` -- spawns signal handler task, logs startup. Health/metrics/supervision task spawning deferred to wiring phase -- per `tokio-runtime.md` line 338-382
- [ ] T026 Implement `RuntimeManager::graceful_shutdown()` -- sets shutdown signal, joins task handles, calls `runtime.shutdown_timeout(DEFAULT_SHUTDOWN_TIMEOUT)` -- per `tokio-runtime.md` line 384-411
- [ ] T027 Implement signal handler (SIGTERM, SIGINT) as async fn using `tokio::signal::unix` -- per `tokio-runtime.md` line 414-430
- [ ] T028 Implement `RuntimeManager::runtime()` accessor returning `&Arc<Runtime>` -- per `tokio-runtime.md` line 432-434
- [ ] T029 Implement `RuntimeManager::spawn_task()` and `RuntimeManager::spawn_blocking_task()` convenience methods that track JoinHandles internally

### Runtime Performance Monitor

- [ ] T030 Implement `RuntimePerformanceMonitor` struct in `mister-smith-runtime/src/metrics.rs` -- wraps `tokio::runtime::Handle` -- per `tokio-runtime.md` line 450-453
- [ ] T031 Implement `RuntimePerformanceMonitor::collect_metrics()` using metrics 0.24 macros (gauge!, counter!) for: workers count, blocking threads, idle threads, alive tasks, global queue depth, per-worker local queue depth, park/noop/steal counts, blocking queue depth, budget forced yield -- per `tokio-runtime.md` line 459-493

### Task Scheduling

- [ ] T032 Implement `TaskScheduler` struct in `mister-smith-runtime/src/scheduling.rs` with Semaphore-based concurrency limiting and mpsc priority queue -- per `tokio-runtime.md` line 688-721
- [ ] T033 [P] Implement `TaskScheduler::batch_processing_pattern()` and `fanout_fanin_pattern()` static utility methods -- per `tokio-runtime.md` line 724-804

### Tests

- [ ] T034 [P] Test: RuntimeConfig::default() produces valid configuration and builds runtime -- `mister-smith-runtime/tests/config_tests.rs`
- [ ] T035 [P] Test: RuntimeConfig preset constructors (cpu_bound, io_bound, high_throughput) build valid runtimes -- `mister-smith-runtime/tests/config_tests.rs`
- [ ] T036 [P] Test: RuntimeManager::initialize() succeeds with default config -- `mister-smith-runtime/tests/manager_tests.rs`
- [ ] T037 Test: RuntimeManager start_system() and graceful_shutdown() complete without error -- `mister-smith-runtime/tests/manager_tests.rs`
- [ ] T038 [P] Test: RuntimePerformanceMonitor::collect_metrics() runs without panic on active runtime -- `mister-smith-runtime/tests/metrics_tests.rs`
- [ ] T039 [P] Test: Signal handler sets shutdown signal on simulated SIGINT -- `mister-smith-runtime/tests/signal_tests.rs`

---

## Phase 3: Monitoring (`mister-smith-monitoring`)

**Purpose**: Health check registration and execution, metrics collection, failure detection, probe endpoints.

**Spec**: `spec/core-architecture/monitoring-and-health.md`, `spec/operations/observability-monitoring-framework.md`

**Checkpoint**: Health checks register, execute on interval, aggregate system health. Metrics collect and flush. PhiAccrualFailureDetector computes phi values.

### Health Types

- [ ] T040 Implement `ComponentId(String)` newtype with Clone, Hash, Eq, Serialize, Deserialize in `mister-smith-monitoring/src/types.rs` -- per `supervision-and-events.md` line 536
- [ ] T041 Implement `Status` enum (Healthy, Degraded, Unhealthy, Unknown) with PartialEq, Serialize, Deserialize in `mister-smith-monitoring/src/types.rs` -- per `monitoring-and-health.md` line 76-81
- [ ] T042 Implement `HealthStatus` struct (component_id, status, last_check: SystemTime, message: Option<String>, metadata: HashMap<String, serde_json::Value>) in `mister-smith-monitoring/src/types.rs` -- per `monitoring-and-health.md` line 66-73

### Health Check Trait and Monitor

- [ ] T043 Implement `HealthCheck` async trait in `mister-smith-monitoring/src/health.rs`: check(), component_id(), check_interval() -- per `monitoring-and-health.md` line 83-88
- [ ] T044 Implement `HealthMonitor` struct in `mister-smith-monitoring/src/health.rs`: check_interval, health_checks (RwLock<Vec<Box<dyn HealthCheck>>>), status_cache (RwLock<HashMap<ComponentId, HealthStatus>>), event_bus (Option<Arc<...>>) -- per `monitoring-and-health.md` line 90-105
- [ ] T045 Implement `HealthMonitor::new()`, `with_event_bus()`, `register_check()` -- per `monitoring-and-health.md` line 98-115
- [ ] T046 Implement `HealthMonitor::run()` loop: check interval, perform_health_checks(), respect shutdown signal -- per `monitoring-and-health.md` line 117-122
- [ ] T047 Implement `HealthMonitor::perform_health_checks()` -- iterates checks, updates status cache, publishes health events to event bus if status changed -- per `monitoring-and-health.md` line 124-166
- [ ] T048 Implement `HealthMonitor::get_status()`, `get_all_statuses()`, `is_system_healthy()` -- per `monitoring-and-health.md` line 174-188
- [ ] T049 Implement `RuntimeHealthCheck` struct (responsiveness check via tokio yield with timeout) -- per `monitoring-and-health.md` line 195-247

### Phi Accrual Failure Detector

- [ ] T050 Implement `PhiAccrualFailureDetector` struct in `mister-smith-monitoring/src/failure_detector.rs`: tracks heartbeat arrival times per NodeId, computes phi value using exponential distribution model -- per `supervision-and-events.md` line 430-464, `supervision-trees.md` line 243-255
- [ ] T051 Implement `PhiAccrualFailureDetector::phi(node_id, timestamp)` -> f64: compute suspicion level from inter-arrival time distribution -- per `supervision-and-events.md` line 442
- [ ] T052 Implement `PhiAccrualFailureDetector::record_heartbeat(node_id, timestamp)` for updating arrival time history
- [ ] T053 Implement configurable phi threshold (default: 8.0, configurable per node) for failure detection decisions

### Metrics Collection

- [ ] T054 Implement `MetricValue` enum (Counter, Gauge, Histogram, Summary) in `mister-smith-monitoring/src/metrics.rs` -- per `monitoring-and-health.md` line 414-419
- [ ] T055 Implement `Metric` struct (name, value, timestamp, tags) in `mister-smith-monitoring/src/metrics.rs` -- per `monitoring-and-health.md` line 407-412
- [ ] T056 Implement `MetricsCollector` struct with internal buffer (RwLock<HashMap<String, Vec<Metric>>>) and flush_interval -- per `monitoring-and-health.md` line 422-433
- [ ] T057 Implement `MetricsCollector::record_event_published()`, `record_handler_error()`, `increment_counter()`, `set_gauge()`, `record_histogram()`, `record_summary()` -- per `monitoring-and-health.md` line 435-541
- [ ] T058 Implement `MetricsCollector::run()` (periodic flush loop) and `flush()` (write metrics to backend or log) -- per `monitoring-and-health.md` line 456-474
- [ ] T059 Implement `MetricsBackend` async trait with `send_metrics(Vec<Metric>)` -- per `monitoring-and-health.md` line 546-549
- [ ] T060 [P] Implement `PrometheusBackend` struct as reference MetricsBackend impl (Prometheus push gateway via reqwest or metrics-exporter-prometheus) -- per `monitoring-and-health.md` line 553-581

### MetricsRegistry (from component-architecture.md)

- [ ] T061 Implement `MetricsRegistry` struct in `mister-smith-monitoring/src/registry.rs` with DashMap-based counters, gauges, histograms, exporters, overhead monitor -- per `component-architecture.md` line 112-154
- [ ] T062 Implement `OverheadMonitor` struct (max_collection_time, sampling_rate, batch_size) for adaptive metrics overhead control -- per `component-architecture.md` line 121-125

### Monitoring System Coordinator

- [ ] T063 Implement `MonitoringSystem` struct in `mister-smith-monitoring/src/system.rs` wiring HealthMonitor + MetricsCollector + EventBus reference -- per `monitoring-and-health.md` line 587-603
- [ ] T064 Implement `SystemComponents` struct and `MonitoringSystem::register_component_health_checks()` -- per `monitoring-and-health.md` line 608-626, 659-664
- [ ] T065 Implement `MonitoringSystem::start()` spawning health and metrics tasks -- per `monitoring-and-health.md` line 628-647

### Tests

- [ ] T066 [P] Test: HealthMonitor registers checks and executes them on interval -- `mister-smith-monitoring/tests/health_tests.rs`
- [ ] T067 [P] Test: HealthMonitor caches status and returns via get_status(), get_all_statuses() -- `mister-smith-monitoring/tests/health_tests.rs`
- [ ] T068 [P] Test: is_system_healthy() returns false when any component is Unhealthy -- `mister-smith-monitoring/tests/health_tests.rs`
- [ ] T069 [P] Test: RuntimeHealthCheck returns Healthy when runtime is responsive, Unhealthy on timeout -- `mister-smith-monitoring/tests/health_tests.rs`
- [ ] T070 [P] Test: PhiAccrualFailureDetector computes reasonable phi values from regular heartbeats vs. missed heartbeats -- `mister-smith-monitoring/tests/failure_detector_tests.rs`
- [ ] T071 [P] Test: MetricsCollector records and flushes metrics correctly -- `mister-smith-monitoring/tests/metrics_tests.rs`
- [ ] T072 [P] Test: MetricsRegistry increments counters, sets gauges, respects sampling rate -- `mister-smith-monitoring/tests/registry_tests.rs`

---

## Phase 4: Event System (`mister-smith-events`)

**Purpose**: In-process pub/sub for system events. Typed publish/subscribe, event filtering, dead letter handling, event store abstraction.

**Spec**: `spec/core-architecture/supervision-and-events.md` (Event System Implementation section)

**Checkpoint**: Events publish and reach subscribers. Filters match correctly. Broadcast channel delivers real-time events. Dead letter queue catches undelivered events. InMemoryEventStore persists and queries events.

### Event Types

- [ ] T073 Implement `SystemEventType` enum in `mister-smith-events/src/types.rs`: Started, Stopping, Stopped, HealthCheckPassed, HealthCheckFailed, ConfigurationChanged, ResourcePoolExhausted, CircuitBreakerOpen, CircuitBreakerClosed -- per `supervision-and-events.md` line 547-557
- [ ] T074 [P] Implement `AgentEventType` enum: Created, Started, Stopped, Failed, MessageReceived, MessageProcessed, StateChanged -- per `supervision-and-events.md` line 559-567
- [ ] T075 [P] Implement `ToolEventType` enum: Registered, Unregistered, ExecutionStarted, ExecutionCompleted, ExecutionFailed, PermissionDenied -- per `supervision-and-events.md` line 569-577
- [ ] T076 Implement `EventType` enum: System(SystemEventType), Agent(AgentEventType), Tool(ToolEventType), Custom(String) -- per `supervision-and-events.md` line 538-544
- [ ] T077 Implement `SystemEvent` struct: id (Uuid), timestamp (SystemTime), source (ComponentId), event_type (EventType), payload (serde_json::Value), correlation_id (Option<Uuid>), causation_id (Option<Uuid>) -- per `supervision-and-events.md` line 524-533

### Event Error

- [ ] T078 Implement `EventError` enum in `mister-smith-events/src/error.rs`: PublishFailed, Timeout, SubscriptionFailed, ValidationFailed(String), CorrelationFailed(String), StoreFailed(String), SerializationFailed(String) -- per `integration-patterns.md` line 919-930, `supervision-and-events.md` usage

### Event Filter

- [ ] T079 Implement `EventFilter` struct: event_types (Option<Vec<EventType>>), sources (Option<Vec<ComponentId>>), correlation_ids (Option<Vec<Uuid>>) -- per `supervision-and-events.md` line 589-594

### Event Handler Trait

- [ ] T080 Implement `EventHandler` async trait: handle_event(SystemEvent) -> Result<(), EventError>, event_filter() -> Option<EventFilter> (default None) -- per `supervision-and-events.md` line 581-587

### Event Bus

- [ ] T081 Implement `EventBus` struct in `mister-smith-events/src/bus.rs`: subscribers (RwLock<HashMap<TypeId, Vec<Arc<dyn EventHandler>>>>), event_queue (Mutex<VecDeque<SystemEvent>>), broadcast_sender (broadcast::Sender<SystemEvent>), event_store (Option<Arc<dyn EventStore>>), metrics_collector -- per `supervision-and-events.md` line 597-603
- [ ] T082 Implement `EventBus::new()` with configurable broadcast channel capacity (default 10,000) -- per `supervision-and-events.md` line 606-616
- [ ] T083 Implement `EventBus::with_event_store()` builder method -- per `supervision-and-events.md` line 618-621
- [ ] T084 Implement `EventBus::publish()` -- record metrics, persist to store, enqueue, broadcast, process subscribers -- per `supervision-and-events.md` line 623-648
- [ ] T085 Implement `EventBus::process_event()` -- iterate subscribers, apply filters, call handlers, log errors -- per `supervision-and-events.md` line 650-673
- [ ] T086 Implement `EventBus::matches_filter()` -- check event_types, sources, correlation_ids against filter -- per `supervision-and-events.md` line 675-701
- [ ] T087 Implement `EventBus::subscribe()` -- register handler in subscribers map -- per `supervision-and-events.md` line 704-712
- [ ] T088 Implement `EventBus::subscribe_broadcast()` -- return broadcast::Receiver<SystemEvent> -- per `supervision-and-events.md` line 715-717
- [ ] T089 Implement `EventBus::replay_events()` -- query event store with time range and filter -- per `supervision-and-events.md` line 719-739

### Dead Letter Queue

- [ ] T090 Implement `DeadLetterQueue` struct in `mister-smith-events/src/dead_letter.rs`: bounded VecDeque<SystemEvent> with configurable max_size, retention policy, and accessor methods (enqueue, drain, len) -- per `component-architecture.md` line 214, referenced but not fully defined in specs
- [ ] T091 Wire dead letter queue into EventBus::process_event() -- events that fail all handlers are routed to dead letter queue

### Event Builder

- [ ] T092 Implement `EventBuilder` struct in `mister-smith-events/src/builder.rs`: new(source, event_type), with_payload<T: Serialize>(), with_correlation_id(), with_causation_id(), build() -> SystemEvent -- per `supervision-and-events.md` line 801-838

### Event Store

- [ ] T093 Implement `EventStore` async trait in `mister-smith-events/src/store.rs`: append(), query(from, to), get_by_id(), get_by_correlation() -- per `supervision-and-events.md` line 748-753
- [ ] T094 Implement `InMemoryEventStore` for testing: Vec<SystemEvent> behind RwLock -- per `supervision-and-events.md` line 756-798

### Tests

- [ ] T095 [P] Test: EventBus publish and subscribe roundtrip -- handler receives published event -- `mister-smith-events/tests/bus_tests.rs`
- [ ] T096 [P] Test: EventBus broadcast subscriber receives events via subscribe_broadcast() -- `mister-smith-events/tests/bus_tests.rs`
- [ ] T097 [P] Test: EventFilter correctly matches/rejects events by type, source, correlation_id -- `mister-smith-events/tests/filter_tests.rs`
- [ ] T098 [P] Test: EventBuilder constructs valid SystemEvent with payload serialization -- `mister-smith-events/tests/builder_tests.rs`
- [ ] T099 [P] Test: InMemoryEventStore append, query by time range, get_by_id, get_by_correlation -- `mister-smith-events/tests/store_tests.rs`
- [ ] T100 [P] Test: DeadLetterQueue enqueues failed events and respects max_size bounds -- `mister-smith-events/tests/dead_letter_tests.rs`
- [ ] T101 Test: EventBus replay_events returns filtered results from event store -- `mister-smith-events/tests/bus_tests.rs`
- [ ] T102 [P] Test: Multiple concurrent publishers and subscribers do not cause data races -- `mister-smith-events/tests/concurrency_tests.rs`

---

## Phase 5: Async Patterns (`mister-smith-async`)

**Purpose**: Reusable async building blocks -- task executor, RAII guards, circuit breaker, retry/timeout combinators, stream processing, synchronization primitives.

**Spec**: `spec/core-architecture/async-patterns.md` (Sections 1-2, 4-5)

**Checkpoint**: TaskExecutor runs concurrent tasks with retry, circuit breaker trips and recovers, stream processor handles backpressure, synchronization primitives coordinate tasks correctly.

### Task Types

- [ ] T103 Implement `TaskId(Uuid)` newtype, `TaskPriority` enum (Low=0, Normal=1, High=2, Critical=3), `TaskError` enum in `mister-smith-async/src/task.rs` -- per `async-patterns.md` line 103-119, 130-145
- [ ] T104 Implement `ErrorStrategy` enum (StopOnError, LogAndContinue, RetryWithBackoff, CircuitBreaker) -- per `async-patterns.md` line 122-128
- [ ] T105 Implement `RetryPolicy` struct (max_attempts, base_delay, max_delay, backoff_multiplier) with Default, for_database(), for_network() presets -- per `async-patterns.md` line 148-186
- [ ] T106 Implement `AsyncTask` async trait: execute(), priority(), timeout(), retry_policy(), task_id() -- per `async-patterns.md` line 233-252

### Task Handle and Metrics

- [ ] T107 Implement `TaskHandle<T>` struct (task_id, oneshot::Receiver, JoinHandle) with await_result(), abort(), task_id() -- per `async-patterns.md` line 282-307
- [ ] T108 Implement `TaskMetrics` struct with AtomicU64 counters: total_submitted, completed, failed, currently_running, panics_recovered, circuit_breaker_trips -- per `async-patterns.md` line 310-330
- [ ] T109 [P] Implement `TaskPool<T>` object pool: factory, max_size, acquire(), release() -- per `async-patterns.md` line 356-386

### Circuit Breaker

- [ ] T110 Implement `CircuitState` enum (Closed, Open, HalfOpen) -- referenced in `async-patterns.md` line 424, defined in type-definitions.md
- [ ] T111 Implement `CircuitBreaker` struct in `mister-smith-async/src/circuit_breaker.rs`: failure_count (AtomicU32), last_failure_time (std::sync::Mutex), state (std::sync::RwLock<CircuitState>), failure_threshold, recovery_timeout, half_open_max_calls -- per `async-patterns.md` line 421-428
- [ ] T112 Implement `CircuitBreaker::new()`, `can_proceed()`, `record_success()`, `record_failure()` with state transitions -- per `async-patterns.md` line 431-483

### Task Executor

- [ ] T113 Implement `TaskExecutor` struct in `mister-smith-async/src/executor.rs`: task_queue, worker_handles, semaphore, metrics, shutdown broadcast, circuit_breaker, task_pool, error_strategy -- per `async-patterns.md` line 523-533
- [ ] T114 Implement `TaskExecutor::new(max_concurrent)` and `with_config(max_concurrent, ErrorStrategy)` -- per `async-patterns.md` line 536-559
- [ ] T115 Implement `TaskExecutor::submit<T: AsyncTask>()` -> `Result<TaskHandle<T::Output>, TaskError>`: acquire semaphore, spawn task with metrics tracking, return handle -- per `async-patterns.md` line 562-603
- [ ] T116 Implement `TaskExecutor::execute_with_retry()` -- exponential backoff with jitter, timeout protection, max_attempts enforcement -- per `async-patterns.md` line 608-656
- [ ] T117 Implement `TaskExecutor::shutdown()` -- broadcast shutdown, join worker handles -- per `async-patterns.md` line 658-674
- [ ] T118 Implement `TaskExecutor::metrics()` accessor -- per `async-patterns.md` line 671-673

### Stream Processing

- [ ] T119 Implement `BackpressureStrategy` enum (Wait, Drop, Buffer, Block) and `BackpressureConfig` struct in `mister-smith-async/src/stream.rs` -- per `async-patterns.md` line 708-732
- [ ] T120 Implement `Processor<T>` async trait: process(T) -> Result<T>, name() -> &str -- per `async-patterns.md` line 763-772
- [ ] T121 Implement `StreamMetrics` struct (AtomicU64: items_processed, items_dropped, backpressure_events, processing_errors) -- per `async-patterns.md` line 819-825
- [ ] T122 Implement `StreamProcessor<T>` struct: input_stream, processors vec, output_sink, backpressure_config, buffer, metrics -- per `async-patterns.md` line 809-817
- [ ] T123 Implement `StreamProcessor::process_stream()`: iterate input, apply processors, send with backpressure handling, flush buffer -- per `async-patterns.md` line 847-866
- [ ] T124 Implement `StreamProcessor` backpressure handlers for all 4 strategies (Wait, Drop, Buffer, Block) -- per `async-patterns.md` line 876-912
- [ ] T125 [P] Implement `create_buffered_stream()` and `create_rate_limited_stream()` utility functions -- per `async-patterns.md` line 957-986

### Synchronization Primitives

- [ ] T126 Implement `DeadlockPreventingMutex<T>` in `mister-smith-async/src/sync.rs`: Tokio mutex with acquisition_order and timeout, lock_with_timeout() -- per `async-patterns.md` line 1606-1631
- [ ] T127 Implement `AsyncBarrier` wrapper around tokio::sync::Barrier -- per `async-patterns.md` line 1634-1649
- [ ] T128 Implement `CountdownLatch` in `mister-smith-async/src/sync.rs`: AtomicUsize count + tokio::sync::Notify, count_down(), wait() -- per `async-patterns.md` line 1675-1702
- [ ] T129 [P] Implement `MpmcChannel<T>` wrapper around crossbeam bounded/unbounded channels with async recv_async() -- per `async-patterns.md` line 1711-1739

### Task Guard

- [ ] T130 Implement `TaskGuard` in `mister-smith-async/src/guard.rs`: holds JoinHandle<()> + optional cleanup closure, aborts handle and runs cleanup on Drop -- per `async-patterns.md` line 2058-2079

### Tests

- [ ] T131 [P] Test: TaskExecutor submits and completes tasks concurrently up to max_concurrent limit -- `mister-smith-async/tests/executor_tests.rs`
- [ ] T132 [P] Test: TaskExecutor retry with backoff retries on failure, respects max_attempts -- `mister-smith-async/tests/executor_tests.rs`
- [ ] T133 [P] Test: TaskExecutor timeout kills long-running tasks -- `mister-smith-async/tests/executor_tests.rs`
- [ ] T134 [P] Test: CircuitBreaker transitions Closed -> Open after failure_threshold failures -- `mister-smith-async/tests/circuit_breaker_tests.rs`
- [ ] T135 [P] Test: CircuitBreaker transitions Open -> HalfOpen after recovery_timeout -- `mister-smith-async/tests/circuit_breaker_tests.rs`
- [ ] T136 [P] Test: CircuitBreaker transitions HalfOpen -> Closed on success -- `mister-smith-async/tests/circuit_breaker_tests.rs`
- [ ] T137 [P] Test: StreamProcessor processes items through chained processors -- `mister-smith-async/tests/stream_tests.rs`
- [ ] T138 [P] Test: StreamProcessor applies backpressure strategy (Drop: items_dropped increments; Buffer: items buffered up to limit) -- `mister-smith-async/tests/stream_tests.rs`
- [ ] T139 [P] Test: CountdownLatch blocks until all count_down() calls complete -- `mister-smith-async/tests/sync_tests.rs`
- [ ] T140 [P] Test: DeadlockPreventingMutex times out and returns error instead of deadlocking -- `mister-smith-async/tests/sync_tests.rs`
- [ ] T141 [P] Test: TaskGuard aborts handle and runs cleanup on drop -- `mister-smith-async/tests/guard_tests.rs`
- [ ] T142 [P] Test: TaskPool reuses objects up to max_size, drops excess -- `mister-smith-async/tests/pool_tests.rs`

---

## Phase 6: Resource Management (`mister-smith-resources`)

**Purpose**: Generic connection pooling, pool sizing, health checks, eviction, resource lifecycle management.

**Spec**: `spec/core-architecture/component-architecture.md` (Resource Management section), `spec/data-management/connection-management.md` (sizing algorithms)

**Checkpoint**: ConnectionPool<R: Resource> acquires, releases, and health-checks generic resources. Pool sizing computes reasonable recommendations. Idle resources are evicted.

### Resource Trait (re-export from Phase 1)

- [ ] T143 Re-export `Resource` trait from `mister-smith-core` in `mister-smith-resources/src/lib.rs` for ergonomic access. The trait (acquire, release, is_healthy, secure_handshake) is defined in Phase 1.2 per `component-architecture.md` line 281-288.

### Connection Pool

- [ ] T144 Implement `ConnectionPool<R: Resource>` struct in `mister-smith-resources/src/pool.rs`: pool (Arc<Mutex<VecDeque<R>>>), max_size, min_size, acquire_timeout, idle_timeout, health_check_interval, tls_config -- per `component-architecture.md` line 290-298
- [ ] T145 Implement `PooledResource<R>` RAII wrapper that returns resource to pool on Drop -- per `component-architecture.md` line 301 (PooledResource::new)
- [ ] T146 Implement `ConnectionPool::acquire()` with timeout, health check on acquired resource, create-new fallback -- per `component-architecture.md` line 300-322
- [ ] T147 Implement `ConnectionPool::return_resource()` -- health check before return, drop if unhealthy or pool full -- per `component-architecture.md` line 324-333
- [ ] T148 Implement `ConnectionPool::health_check_sweep()` -- periodic background task that checks all idle resources, evicts unhealthy ones, maintains min_size
- [ ] T149 Implement `ConnectionPool::idle_eviction()` -- evict resources that exceed idle_timeout from the pool

### Pool Sizing

- [ ] T150 Implement `PoolSizeRecommendation` struct (recommended_size, min_connections, max_connections, reasoning: String) in `mister-smith-resources/src/sizing.rs` -- per `connection-management.md` line 113-118
- [ ] T151 Implement `ConnectionPoolSizer::calculate_optimal_pool_size()` using Little's Law: pool_size = (ops/sec * avg_duration) / target_utilization, with agent concurrency factor adjustment and min/max bounds -- per `connection-management.md` line 92-119
- [ ] T152 Implement `ConnectionPoolSizer::calculate_agent_concurrency_factor()` -- 1.0 for <=5 agents, 0.8 for <=20, 0.6 for >20 -- per `connection-management.md` line 121-130
- [ ] T153 Implement `PoolSizeTemplate` and `get_environment_template()` for Development/Staging/Production presets -- per `connection-management.md` line 133-158

### Resource Manager

- [ ] T154 Implement `ResourceManager` struct in `mister-smith-resources/src/manager.rs`: HashMap<String, Box<dyn Any>> for heterogeneous pool storage, registration, and lookup -- per `component-architecture.md` line 273-278
- [ ] T155 Implement `ResourceManager::register_pool<R: Resource>()` and `ResourceManager::get_pool<R: Resource>()` for type-safe pool access
- [ ] T156 Implement `ResourceManager::shutdown()` -- drain and close all managed pools

### Tests

- [ ] T157 [P] Test: ConnectionPool acquire returns healthy resource, release returns it to pool -- `mister-smith-resources/tests/pool_tests.rs`
- [ ] T158 [P] Test: ConnectionPool acquire times out when pool exhausted and at max_size -- `mister-smith-resources/tests/pool_tests.rs`
- [ ] T159 [P] Test: ConnectionPool evicts unhealthy resources on acquire -- `mister-smith-resources/tests/pool_tests.rs`
- [ ] T160 [P] Test: PooledResource returns to pool on drop (RAII) -- `mister-smith-resources/tests/pool_tests.rs`
- [ ] T161 [P] Test: calculate_optimal_pool_size returns reasonable values for various inputs (use proptest for property-based testing) -- `mister-smith-resources/tests/sizing_tests.rs`
- [ ] T162 [P] Test: get_environment_template returns correct presets for dev/staging/production -- `mister-smith-resources/tests/sizing_tests.rs`
- [ ] T163 [P] Test: ResourceManager registers and retrieves typed pools -- `mister-smith-resources/tests/manager_tests.rs`
- [ ] T164 Test: ConnectionPool health_check_sweep evicts unhealthy idle resources -- `mister-smith-resources/tests/pool_tests.rs`

---

## Phase 7: Integration and Validation

**Purpose**: Cross-crate integration tests, Gate 2 validation, documentation.

**Checkpoint**: Gate 2 passes -- runtime starts/stops, health reports, events flow, metrics collected, async patterns work, resource pools function. `cargo test --workspace` and `cargo clippy --workspace` pass clean.

### Cross-Crate Integration

- [ ] T165 Integration test: RuntimeManager starts, MonitoringSystem registers RuntimeHealthCheck, health check executes and returns Healthy -- `tests/integration/runtime_monitoring.rs`
- [ ] T166 Integration test: EventBus publishes SystemEventType::Started, subscriber receives it, MetricsCollector records event.published metric -- `tests/integration/events_monitoring.rs`
- [ ] T167 Integration test: TaskExecutor submits tasks, CircuitBreaker trips after failures, emits CircuitBreakerOpen event to EventBus -- `tests/integration/async_events.rs`
- [ ] T168 Integration test: ConnectionPool with mock Resource acquires/releases through ResourceManager, HealthMonitor checks pool health -- `tests/integration/resources_monitoring.rs`
- [ ] T169 Integration test: Full Phase 2 startup sequence -- RuntimeManager -> MonitoringSystem -> EventBus -> shutdown. Verify no resource leaks, clean shutdown. -- `tests/integration/full_lifecycle.rs`

### Gate 2 Validation

- [ ] T170 Run `cargo build --workspace` -- all crates compile without errors
- [ ] T171 Run `cargo test --workspace` -- all unit and integration tests pass
- [ ] T172 Run `cargo clippy --workspace -- -D warnings` -- no warnings
- [ ] T173 Verify runtime lifecycle: startup and graceful shutdown complete without errors (T037)
- [ ] T174 Verify health monitoring: health checks register, execute, aggregate system health (T066-T069)
- [ ] T175 Verify event flow: publish/subscribe roundtrip works, broadcast works, filters work (T095-T102)
- [ ] T176 Verify metrics collection: MetricsCollector records and flushes, RuntimePerformanceMonitor collects Tokio metrics (T038, T071)
- [ ] T177 Verify async patterns: TaskExecutor concurrent execution, CircuitBreaker state machine, retry backoff, synchronization primitives (T131-T142)
- [ ] T178 Verify resource management: ConnectionPool acquire/release, sizing calculations, idle eviction (T157-T164)

---

## Dependencies and Execution Order

### Phase Dependencies

```
Phase 1 (Setup)     -- no dependencies, start immediately
Phase 2 (Runtime)   -- depends on Phase 1 completion
Phase 3 (Monitor)   -- depends on Phase 2 (runtime must exist for async health checks)
Phase 4 (Events)    -- depends on Phase 2 (runtime) and Phase 3 (events emit metrics)
Phase 5 (Async)     -- depends on Phase 2 (runtime) and Phase 3 (monitoring)
Phase 6 (Resources) -- depends on Phase 3 (health integration) and Phase 1 config
Phase 7 (Integ.)    -- depends on all above phases
```

### Parallel Opportunities

Within each phase, tasks marked [P] can run in parallel:

- **Phase 1**: All T002-T012 can run in parallel (different crate directories)
- **Phase 2**: T018/T021 (presets), T034-T039 (tests) can run in parallel
- **Phase 3**: T066-T072 (tests) can run in parallel
- **Phase 4**: T074-T075 (agent/tool event types), T095-T102 (tests) can run in parallel
- **Phase 5**: T125 (stream utils), T129 (MpmcChannel), T131-T142 (tests) can run in parallel
- **Phase 6**: T157-T164 (tests) can run in parallel

### Critical Path

```
T001 (workspace) -> T002 (runtime crate) -> T014 (errors) -> T016 (config) -> T019 (build_runtime)
  -> T022 (RuntimeManager) -> T024 (initialize) -> T025 (start) -> T026 (shutdown)
    -> T030 (perf monitor) -> T037 (lifecycle test)
      -> T043 (HealthCheck) -> T044 (HealthMonitor) -> T046 (run loop)
        -> T081 (EventBus) -> T084 (publish) -> T087 (subscribe)
          -> T113 (TaskExecutor) -> T115 (submit)
            -> T144 (ConnectionPool) -> T146 (acquire)
              -> T165-T169 (integration tests) -> T170-T178 (gate validation)
```

Estimated minimum serial path: ~50 tasks. With parallelization: significantly less.

---

## Task Count Summary

| Phase | Tasks | Parallelizable |
|-------|-------|---------------|
| 1. Setup | T001-T013 (13) | 11 |
| 2. Runtime | T014-T039 (26) | 10 |
| 3. Monitoring | T040-T072 (33) | 11 |
| 4. Events | T073-T102 (30) | 14 |
| 5. Async Patterns | T103-T142 (40) | 17 |
| 6. Resources | T143-T164 (22) | 9 |
| 7. Integration | T165-T178 (14) | 0 |
| **Total** | **178** | **72** |

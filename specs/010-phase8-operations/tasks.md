# Tasks: Phase 8 — Operations & Production Readiness

**Input**: Design documents from `/specs/010-phase8-operations/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the binary crate, add workspace dependencies, establish project structure

- [x] T001 Create `crates/mister-smith-app/` binary crate directory with `Cargo.toml` and `src/main.rs` stub; add to workspace members in `Cargo.toml`
- [x] T002 Add Phase 8 workspace dependencies to root `Cargo.toml`: `opentelemetry = "0.31.0"`, `opentelemetry_sdk = "0.31.0"`, `opentelemetry-otlp = "0.31.0"`, `tracing-opentelemetry = "0.32.1"`, `clap = { version = "4", features = ["derive"] }`
- [x] T003 [P] Add `ProcessLifecycle` enum (`Starting`, `Ready`, `Draining`, `Stopped`, `Failed`) and `ShutdownReason` enum to `crates/mister-smith-core/src/enums.rs`
- [x] T004 [P] Add `ObservabilityConfig` struct to `crates/mister-smith-config/src/types.rs` with fields: `otlp_endpoint`, `trace_sampling_ratio`, `metrics_export_interval`, `log_format`, `log_level`, `buffer_size`, `prometheus_enabled`; add to `FrameworkConfig`
- [x] T005 [P] Create `deploy/` directory structure: `deploy/kubernetes/`, `deploy/dashboards/`, `deploy/alerts/`
- [x] T006 Verify workspace builds cleanly with `cargo build --workspace` after adding new crate and dependencies

**Checkpoint**: Binary crate exists in workspace, all new types compile, workspace builds clean

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story implementation

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Implement CLI argument parsing in `crates/mister-smith-app/src/main.rs` using clap: `--config`, `--log-level`, `--log-format`, `--version`
- [x] T008 Implement configuration loading pipeline in `crates/mister-smith-app/src/config.rs`: parse CLI args → load config file → apply env overlay → validate; return `FrameworkConfig` with `ObservabilityConfig`
- [x] T009 Implement signal handling in `crates/mister-smith-app/src/shutdown.rs`: register SIGTERM and SIGINT handlers via `tokio::signal::unix::signal(SignalKind::terminate())` and `tokio::signal::ctrl_c()`; support first-signal (graceful) and second-signal (forced) shutdown paths
- [x] T010 Implement `ProcessStateTracker` in `crates/mister-smith-app/src/main.rs` (or a shared module): thread-safe state machine tracking `ProcessLifecycle` transitions with `Arc<AtomicU8>` for lock-free reads from health probes

**Checkpoint**: Foundation ready — binary parses args, loads config, handles signals, tracks process state

---

## Phase 3: User Story 1 — Application Bootstrap and Process Lifecycle (Priority: P1) MVP

**Goal**: Framework operator starts the process, system initializes deterministically, shuts down gracefully on signal

**Independent Test**: Start the process with valid config, verify health endpoints respond, send SIGTERM, verify clean exit with code 0

### Implementation for User Story 1

- [x] T011 [US1] Implement startup bootstrap sequence in `crates/mister-smith-app/src/bootstrap.rs`: deterministic initialization order — config validation → observability init → RuntimeManager start → NATS connection (with timeout) → PostgreSQL connection (with timeout) → supervision tree init → agent spawning → HTTP/gRPC server start → set state to Ready
- [x] T012 [US1] Implement external service connectivity validation in `crates/mister-smith-app/src/bootstrap.rs`: check NATS reachability with `async_nats::connect()`, check PostgreSQL with `sqlx::PgPool::connect()`; fail-fast with specific error message and non-zero exit on unreachable
- [x] T013 [US1] Implement configurable startup timeout in `crates/mister-smith-app/src/bootstrap.rs`: wrap the full startup sequence in `tokio::time::timeout(startup_timeout)`; if exceeded, log specific step that timed out, set state to Failed, exit with code 1
- [x] T014 [US1] Implement graceful shutdown sequence in `crates/mister-smith-app/src/shutdown.rs`: on first signal — set state to Draining → stop accepting new messages → wait for in-flight drain (with timeout) → stop agents in reverse start order → flush AuditPersister → flush MetricsCollector → flush OTel providers → close NATS → close PostgreSQL pool → stop HTTP/gRPC → set state to Stopped → exit 0
- [x] T015 [US1] Implement forced shutdown path in `crates/mister-smith-app/src/shutdown.rs`: on second signal during graceful shutdown — skip message drain and agent stop, proceed directly to connection closure and exit with code 2
- [x] T016 [US1] Wire the full lifecycle in `crates/mister-smith-app/src/main.rs`: `#[tokio::main]` entry point calls bootstrap → spawns shutdown listener → awaits termination signal → runs shutdown sequence; log startup duration and shutdown duration
- [x] T017 [US1] Add integration test in `crates/mister-smith-integration-tests/tests/phase8_lifecycle.rs`: test that binary starts (mock or skip external services), reaches Ready, responds to health, exits cleanly on SIGTERM

**Checkpoint**: Binary starts, initializes deterministically, shuts down gracefully. US1 acceptance scenarios 1-4 verifiable.

---

## Phase 4: User Story 2 — Observability Pipeline (Priority: P1)

**Goal**: SRE sees distributed traces, scrapes metrics, reads structured logs with trace correlation

**Independent Test**: Start system, trigger agent operations, verify traces appear in collector, metrics are scrapable, logs include trace IDs

### Implementation for User Story 2

- [x] T018 [US2] Implement OpenTelemetry initialization in `crates/mister-smith-app/src/observability.rs`: create `init_observability(config: &ObservabilityConfig)` that sets up TracerProvider (OTLP batch exporter via tonic), MeterProvider (OTLP periodic exporter), and installs global propagator (W3C TraceContext)
- [x] T019 [US2] Implement tracing subscriber setup in `crates/mister-smith-app/src/observability.rs`: build a `tracing_subscriber::Registry` with layers — `tracing_opentelemetry::layer()` for OTel bridge, `tracing_subscriber::fmt::layer()` with JSON or pretty format, `EnvFilter` for log level; install as global default
- [x] T020 [US2] Implement graceful OTel shutdown in `crates/mister-smith-app/src/observability.rs`: `shutdown_observability()` function that calls `TracerProvider::shutdown()` and `MeterProvider::shutdown()` with timeout to flush remaining telemetry; called during shutdown sequence (T014)
- [x] T021 [US2] Implement telemetry buffering for collector unavailability in `crates/mister-smith-app/src/observability.rs`: configure OTLP exporter with `BatchConfig` using max queue size from `ObservabilityConfig.buffer_size`; this ensures FR-010 (telemetry buffers locally, does not block)
- [x] T022 [P] [US2] Implement W3C TraceContext injection in `crates/mister-smith-transport/src/envelope.rs` (or appropriate transport file): add `inject_trace_context(envelope: &mut MessageEnvelope)` that extracts current span context and writes `traceparent`/`tracestate` headers into the envelope's header map
- [x] T023 [P] [US2] Implement W3C TraceContext extraction in `crates/mister-smith-transport/src/envelope.rs`: add `extract_trace_context(envelope: &MessageEnvelope) -> Option<Context>` that reads `traceparent`/`tracestate` from headers and returns an OpenTelemetry Context for span parenting
- [x] T024 [US2] Wire trace context propagation into NATS publish/subscribe in `crates/mister-smith-nats/src/client.rs`: call `inject_trace_context()` before publishing and `extract_trace_context()` on message receipt; create child spans linked to extracted parent context
- [x] T025 [P] [US2] Add `#[instrument]` spans to key agent operations in `crates/mister-smith-agents/src/agent.rs`: instrument `AgentRuntime::start()`, `AgentRuntime::stop()`, `AgentRuntime::handle_message()` with span attributes `agent.id`, `agent.type`, `agent.state`
- [x] T026 [P] [US2] Add `#[instrument]` spans to orchestration operations in `crates/mister-smith-agents/src/orchestrator.rs`: instrument task decomposition and result aggregation with span attributes `task.id`, `task.type`
- [x] T027 [US2] Implement Prometheus `MetricsBackend` in `crates/mister-smith-monitoring/src/prometheus.rs`: implement `MetricsBackend` trait using `metrics-exporter-prometheus` crate; register standard metrics from data-model.md (counters: messages sent/received, tasks completed/failed, agent restarts; gauges: active agents, queue depth; histograms: task duration, message latency, health check duration)
- [x] T028 [US2] Add `/metrics` Prometheus endpoint to `crates/mister-smith-http/src/routes.rs`: new route `GET /metrics` that returns Prometheus text exposition format from the metrics registry; response time must be <500ms (SC-004)
- [x] T029 [US2] Add structured log format configuration in `crates/mister-smith-app/src/observability.rs`: JSON format includes `timestamp`, `level`, `target`, `message`, `span.trace_id`, `span.span_id`, and structured `fields` per contracts/observability.md
- [x] T030 [US2] Add integration test in `crates/mister-smith-integration-tests/tests/phase8_observability.rs`: verify that agent operations produce spans with correct parent-child relationships, metrics endpoint returns expected counters, and log entries include trace IDs

**Checkpoint**: Traces flow through collector, metrics scrapable at /metrics, logs have trace correlation. US2 acceptance scenarios 1-4 verifiable.

---

## Phase 5: User Story 3 — Cross-Phase Integration Wiring (Priority: P2)

**Goal**: Wire the three cross-phase gaps: audit persistence, security enforcement, monitoring-to-supervision

**Independent Test**: Generate audit events → verify persisted to DB. Spawn agent without credentials → verify rejected. Stop heartbeats → verify failure detected.

### Implementation for User Story 3

- [x] T031 [US3] Wire AuditLogger → AuditPersister in `crates/mister-smith-app/src/bridges.rs`: during bootstrap, create `AuditPersister` with reference to `AuditLogger` (from Phase 5); start background flush loop under supervision; verify zero event loss by draining on shutdown
- [x] T032 [US3] Wire AgentRuntime → PolicyEngine in `crates/mister-smith-app/src/bridges.rs`: during bootstrap, pass `PolicyEngine` and `JwtManager` references to agent spawning; agent operations must call `PolicyEngine::check_permission()` before execution; unauthorized operations return error and record audit event
- [x] T033 [US3] Wire HeartbeatEmitter → PhiAccrualFailureDetector in `crates/mister-smith-app/src/bridges.rs`: during bootstrap, route agent heartbeat events to `PhiAccrualFailureDetector::record_heartbeat()`; spawn background monitor that checks `phi()` values periodically; when phi exceeds threshold (agent suspected failed), notify the appropriate supervisor via the supervision tree
- [x] T034 [US3] Implement supervision event recording in `crates/mister-smith-app/src/bridges.rs`: when supervisor restarts an agent, record the event in both the metrics pipeline (increment `mistersmith_agent_restarts_total` counter) and the audit log (via `AuditLogger::record()`)
- [x] T035 [US3] Implement fresh credential issuance on agent restart in `crates/mister-smith-app/src/bridges.rs`: when a supervisor restarts a failed agent, generate a new JWT token via `JwtManager` and inject into the new agent instance
- [x] T036 [US3] Add integration test in `crates/mister-smith-integration-tests/tests/phase8_integration.rs`: test audit event persistence (create events, verify in repository), test unauthorized operation rejection, test heartbeat absence detection and supervision notification

**Checkpoint**: All three cross-phase gaps wired. US3 acceptance scenarios 1-4 verifiable.

---

## Phase 6: User Story 4 — Containerization and Deployment Artifacts (Priority: P3)

**Goal**: DevOps engineer builds container image, deploys to Kubernetes, health probes work

**Independent Test**: Build image, run container, verify probes respond, verify env var config works

### Implementation for User Story 4

- [x] T037 [P] [US4] Implement liveness probe endpoint `GET /health/live` in `crates/mister-smith-http/src/routes.rs`: returns 200 if HTTP server is responding (no dependency checks); response body per contracts/health-probes.md; must be <10ms
- [x] T038 [P] [US4] Implement readiness probe endpoint `GET /health/ready` in `crates/mister-smith-http/src/routes.rs`: checks NATS connectivity, PostgreSQL connectivity, and agent count; returns 200 only when all checks pass; returns 503 during startup (before Ready) and during shutdown (Draining); response body per contracts/health-probes.md
- [x] T039 [US4] Create multi-stage Dockerfile at `deploy/Dockerfile`: build stage uses `rust:1.88-slim` with cargo-chef for layer caching; runtime stage uses `debian:bookworm-slim` with only binary + CA certs; target image <100MB compressed (SC-006); expose port 8080
- [x] T040 [P] [US4] Create `deploy/docker-compose.yml` for local development: services for NATS (latest), PostgreSQL (15), OTel Collector (contrib), Grafana, and mister-smith app with proper networking and health checks
- [x] T041 [P] [US4] Create Kubernetes Deployment manifest at `deploy/kubernetes/deployment.yaml`: single-replica deployment with liveness/readiness/startup probes per contracts/health-probes.md, resource requests/limits, env var injection from ConfigMap and Secret
- [x] T042 [P] [US4] Create Kubernetes Service manifest at `deploy/kubernetes/service.yaml`: ClusterIP service exposing ports 8080 (HTTP), 50051 (gRPC)
- [x] T043 [P] [US4] Create Kubernetes ConfigMap manifest at `deploy/kubernetes/configmap.yaml`: environment variables for NATS URL, PostgreSQL URL, OTLP endpoint, log level, log format per contracts/process-lifecycle.md env var section
- [x] T044 [P] [US4] Create Kubernetes namespace manifest at `deploy/kubernetes/namespace.yaml`: `mister-smith` namespace with standard labels
- [x] T045 [US4] Verify container builds, starts in <5s (SC-006), probes respond, env var configuration overrides file config

**Checkpoint**: Container image builds and deploys. US4 acceptance scenarios 1-4 verifiable.

---

## Phase 7: User Story 5 — Health Dashboard and Alerting (Priority: P3)

**Goal**: Operator imports dashboard, sees live data, alerts fire on threshold breach

**Independent Test**: Import dashboard JSON into Grafana, connect to metrics endpoint, verify all panels render with live data

### Implementation for User Story 5

- [x] T046 [P] [US5] Create Grafana dashboard JSON at `deploy/dashboards/mister-smith-overview.json`: panels for active agents by type (stat), message throughput (timeseries), task completion rates (timeseries), error rates (timeseries), supervision tree status (table), agent restart timeline (timeseries), resource utilization (gauge); use metric names from data-model.md
- [x] T047 [P] [US5] Create Prometheus alert rules at `deploy/alerts/mister-smith-rules.yml`: rules for agent failure rate spike (`rate(mistersmith_agent_restarts_total[5m]) > 1`), message queue depth growth (`mistersmith_message_queue_depth > 100` for 2m), heartbeat loss (`increase(mistersmith_agent_restarts_total{reason="heartbeat_loss"}[5m]) > 0`), high task error rate (`rate(mistersmith_tasks_failed_total[5m]) / rate(mistersmith_tasks_completed_total[5m]) > 0.1` for 5m)
- [x] T048 [US5] Validate dashboard by verifying all panel queries reference metrics that exist in the Prometheus backend (T027); verify alert rule PromQL expressions are syntactically valid

**Checkpoint**: Dashboard and alerts ready for import. US5 acceptance scenarios 1-3 verifiable.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Gate 8 validation, final testing, documentation

- [x] T049 Run all workspace tests: `cargo test --workspace` — verify existing 950+ tests still pass (no regressions)
- [x] T050 Run clippy: `cargo clippy --workspace -- -D warnings` — verify clean lint
- [x] T051 [P] Verify all 10 success criteria from spec.md:
  - SC-001: Cold start <10s
  - SC-002: Graceful shutdown <30s
  - SC-003: 100% trace coverage for inter-agent messages
  - SC-004: Metrics endpoint <500ms
  - SC-005: Zero audit event loss
  - SC-006: Container <100MB, starts <5s
  - SC-007: Dashboard panels render with live data
  - SC-008: Unauthorized ops rejected 100%
  - SC-009: Heartbeat absence detected within 2x interval
  - SC-010: Normal operation when collector unreachable
- [x] T052 [P] Run Gate 8 validation from ROADMAP.md: containerized service runs, health probes respond, traces appear in collector, metrics scraped, graceful shutdown without message loss
- [x] T053 [P] Run quickstart.md validation: execute all steps in `specs/010-phase8-operations/quickstart.md` and verify they work
- [x] T054 Update `CLAUDE.md` implementation status table: Phase 8 Operations → Complete with `mister-smith-app` crate
- [x] T055 Update `ROADMAP.md` if needed: mark Phase 8 gate criteria as validated

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — process lifecycle is the foundation for everything
- **US2 (Phase 4)**: Depends on US1 (needs bootstrap to exist for OTel init and HTTP server for /metrics)
- **US3 (Phase 5)**: Depends on US1 (needs bootstrap to wire bridges)
- **US4 (Phase 6)**: Depends on US1 (needs binary to containerize) + US2 (needs /metrics endpoint)
- **US5 (Phase 7)**: Depends on US2 (needs metrics to exist for dashboard queries)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Foundation — no other story dependencies. Must complete first.
- **US2 (P1)**: Depends on US1 (binary bootstrap). Can partially parallel with US3.
- **US3 (P2)**: Depends on US1 (bootstrap wiring). Can partially parallel with US2.
- **US4 (P3)**: Depends on US1 + US2. Health probes (T037-T038) can start after US1.
- **US5 (P3)**: Depends on US2 (metrics). Dashboard/alert files (T046-T047) are independent.

### Within Each User Story

- Types/config before implementation
- Bootstrap before wiring
- Core logic before integration
- Implementation before integration tests

### Parallel Opportunities

- **Phase 1**: T003, T004, T005 can run in parallel (different crates/directories)
- **Phase 4 (US2)**: T022 + T023 (inject/extract) parallel; T025 + T026 (agent/orchestrator spans) parallel; T027 independent of trace work
- **Phase 5 (US3)**: T031, T032, T033 partially parallel (different bridges, same file — serialize writes)
- **Phase 6 (US4)**: T037 + T038 parallel; T040 + T041 + T042 + T043 + T044 all parallel (different files)
- **Phase 7 (US5)**: T046 + T047 parallel (different files)
- **Phase 8**: T049, T050, T051, T052, T053 all parallel

---

## Parallel Example: User Story 2

```bash
# Trace context injection and extraction can run in parallel:
Task T022: "Implement W3C TraceContext injection in crates/mister-smith-transport/src/envelope.rs"
Task T023: "Implement W3C TraceContext extraction in crates/mister-smith-transport/src/envelope.rs"

# Agent and orchestrator instrumentation can run in parallel:
Task T025: "Add #[instrument] spans to agent operations in crates/mister-smith-agents/src/agent.rs"
Task T026: "Add #[instrument] spans to orchestration operations in crates/mister-smith-agents/src/orchestrator.rs"
```

## Parallel Example: User Story 4

```bash
# All Kubernetes manifests can be created in parallel:
Task T041: "Create Kubernetes Deployment at deploy/kubernetes/deployment.yaml"
Task T042: "Create Kubernetes Service at deploy/kubernetes/service.yaml"
Task T043: "Create Kubernetes ConfigMap at deploy/kubernetes/configmap.yaml"
Task T044: "Create Kubernetes namespace at deploy/kubernetes/namespace.yaml"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T010)
3. Complete Phase 3: User Story 1 (T011-T017)
4. **STOP and VALIDATE**: Binary starts, responds to health, shuts down cleanly
5. This is the minimum viable operational system

### Incremental Delivery

1. Setup + Foundational → Framework compiles with new crate
2. US1 → Binary starts and stops (MVP!)
3. US2 → Observable system (traces, metrics, logs)
4. US3 → Fully integrated system (cross-phase wiring)
5. US4 → Deployable system (container, K8s)
6. US5 → Monitorable system (dashboards, alerts)
7. Each story adds production readiness without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- T022/T023 target the same file but modify different functions — serialize if in same session
- Total: 55 tasks across 8 phases (6 setup, 4 foundational, 7 US1, 13 US2, 6 US3, 9 US4, 3 US5, 7 polish)

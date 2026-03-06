# Research: Phase 8 — Operations & Production Readiness

**Date**: 2026-03-06
**Branch**: `010-phase8-operations`

## Decision 1: New Crate vs Extending Existing Crates

**Decision**: Create one new crate `mister-smith-app` for the main binary entry point and process lifecycle. Extend existing crates for observability and health probes rather than creating a standalone observability crate.

**Rationale**: The framework already has `mister-smith-monitoring` (metrics, health, failure detector), `mister-smith-http` (Axum endpoints), and `tracing` throughout. OpenTelemetry integration is a bridge layer — it configures providers and attaches tracing-opentelemetry as a subscriber layer. This belongs in the binary crate that owns the process lifecycle, not in a library crate. Health probe endpoints (`/live`, `/ready`) extend the existing HTTP crate.

**Alternatives considered**:
- Separate `mister-smith-observability` crate: Rejected — the OTel setup is process-level configuration (TracerProvider, MeterProvider), not reusable library logic. Putting it in the binary avoids circular dependencies.
- Extending `mister-smith-runtime`: Rejected — RuntimeManager owns the Tokio runtime, not the full application lifecycle. The app binary orchestrates runtime + transport + supervision + agents.

## Decision 2: OpenTelemetry Integration Pattern

**Decision**: Use `tracing-opentelemetry 0.32.x` as the bridge layer between the existing `tracing` instrumentation and OpenTelemetry OTLP export. Initialize TracerProvider, MeterProvider, and LoggerProvider in the binary's startup sequence.

**Rationale**: The codebase already uses `tracing` (0.1.44) extensively across all 18 crates. Adding `#[instrument]` attributes and span creation is incremental. The bridge layer (`tracing-opentelemetry`) converts tracing spans to OTel spans automatically, requiring zero changes to existing library code for basic trace export. OTLP over gRPC (via Tonic, already a dependency) is the standard export path.

**Alternatives considered**:
- Direct OpenTelemetry API in library crates: Rejected — would require modifying all 18 crates to use OTel context API instead of tracing. The bridge approach requires zero library changes.
- Jaeger exporter: Rejected — Jaeger adopted OTLP natively (Jaeger 1.35+). OTLP is the universal standard.

## Decision 3: Distributed Trace Context Propagation

**Decision**: Inject W3C TraceContext headers into NATS message headers and propagate through MessageEnvelope metadata. For HTTP/gRPC, use standard middleware extractors.

**Rationale**: The `MessageEnvelope` in `mister-smith-transport` already carries headers as a `HashMap<String, String>`. W3C TraceContext (`traceparent`, `tracestate`) fits naturally into this header map. NATS messages support headers natively (async-nats 0.46). The existing `Transport::publish()` path can inject context; `Transport::subscribe()` handlers can extract it.

**Alternatives considered**:
- B3 propagation (Zipkin format): Rejected — W3C TraceContext is the OTEL default and the industry standard.
- Embedding trace context in message payload: Rejected — headers are the correct layer; payload is application data.

## Decision 4: Health Probe Endpoint Design

**Decision**: Add `/health/live` (liveness) and `/health/ready` (readiness) endpoints to the existing `mister-smith-http` Axum router. Keep the existing `/api/v1/health` as a detailed health report.

**Rationale**: Kubernetes probes need fast, focused responses. Liveness: "is the process alive and not deadlocked?" (always 200 unless stuck). Readiness: "are all external dependencies connected?" (checks NATS, PostgreSQL connectivity). The existing `/api/v1/health` returns component details — too heavy for a probe. Separate lightweight endpoints follow Kubernetes best practices.

**Alternatives considered**:
- gRPC health check protocol (grpc.health.v1): Considered for future addition but HTTP probes are simpler and work in all environments. The existing gRPC crate already has `tonic-health` as a dependency for this.
- Single `/health` with query parameters: Rejected — Kubernetes probes don't support query parameters in HTTP GET checks.

## Decision 5: Process Lifecycle and Startup Sequencing

**Decision**: The `mister-smith-app` binary implements deterministic startup:
1. Load and validate configuration
2. Initialize tracing/observability pipeline
3. Start Tokio runtime (RuntimeManager)
4. Connect to external services (NATS, PostgreSQL) with timeout
5. Initialize supervision tree
6. Spawn initial agents from configuration
7. Start HTTP/gRPC servers with health endpoints
8. Signal "ready" state

Shutdown is the reverse, triggered by SIGTERM/SIGINT via `tokio::signal`.

**Rationale**: This follows the ROADMAP Phase 8.2 specification and mirrors Erlang/OTP application startup. External service connectivity must be validated before agent spawning (agents depend on transport). Health endpoints activate last so probes only succeed when the system is fully initialized.

**Alternatives considered**:
- Lazy initialization (connect on first use): Rejected — fails-fast-at-startup is safer for production. Constitution Principle V mandates deterministic lifecycle.
- Parallel startup of independent components: Considered but deferred — sequential startup is simpler to debug and the 10-second cold start target (SC-001) is achievable sequentially.

## Decision 6: Metrics Export Strategy

**Decision**: Use the existing `MetricsCollector` + `MetricsBackend` trait from `mister-smith-monitoring` with a new Prometheus backend that exposes metrics via the HTTP endpoint. Additionally, wire OpenTelemetry MeterProvider for OTLP metric export.

**Rationale**: The `MetricsBackend` trait (`monitoring/metrics.rs:54`) is already designed for pluggable backends. A Prometheus backend writes to a `prometheus` registry; the HTTP server serves `/metrics` from that registry. The `metrics-exporter-prometheus 0.18.1` crate is already in workspace dependencies. OTLP metric export provides a second path for environments using Grafana Cloud / Datadog.

**Alternatives considered**:
- StatsD export: Rejected — Prometheus pull model is the Kubernetes standard.
- Only OTLP (no Prometheus): Rejected — Prometheus scraping is ubiquitous and simpler to set up. Both paths should be available.

## Decision 7: Audit Event Bridge Wiring

**Decision**: Wire the existing `AuditPersister` (Phase 6) to the existing `AuditLogger` (Phase 5) in the application bootstrap sequence. The `AuditPersister` already implements the drain pattern — Phase 8 just needs to instantiate it with the correct AuditLogger reference and start its background flush loop.

**Rationale**: Both sides of the bridge exist. `AuditLogger::drain_events()` was added in commit `eb1177f`. `AuditPersister` in `mister-smith-persistence` implements the drain-to-PostgreSQL pattern. Phase 8's job is composition — creating both, passing the logger to the persister, and starting the flush loop in the supervisor tree.

**Alternatives considered**:
- Event-bus-based bridge: Rejected — the ring buffer drain pattern is simpler and already implemented. EventBus adds unnecessary indirection for this use case.

## Decision 8: Container Image Strategy

**Decision**: Multi-stage Dockerfile using `rust:1.88-slim` for build and `debian:bookworm-slim` for runtime. Static linking not required (sqlx needs OpenSSL/rustls at runtime). Target image size under 100MB compressed.

**Rationale**: The spec requires <100MB compressed (SC-006). A slim Debian base with only the compiled binary and CA certificates achieves this. Alpine is avoided because musl libc has known issues with async-nats DNS resolution. The build stage uses cargo-chef for layer caching.

**Alternatives considered**:
- Alpine-based (`rust:1.88-alpine`): Rejected — musl DNS issues with async-nats. Also, sqlx compile times are worse on musl.
- Distroless: Considered — would be even smaller. But debugging production issues requires a shell. Can be offered as an alternative build target.
- Static musl binary: Rejected — OpenSSL/rustls runtime requirements and musl DNS issues.

## Decision 9: Dashboard and Alert Rule Format

**Decision**: Grafana JSON dashboard definitions and Prometheus alerting rules in YAML. Both stored as files in the repository under `deploy/dashboards/` and `deploy/alerts/`.

**Rationale**: Grafana is the de facto standard for Prometheus visualization. The spec (FR-019, FR-020) requires "exportable dashboard definitions" and "configurable alert rule definitions." Grafana JSON and Prometheus YAML are the most widely adopted formats. They can be imported into any Grafana instance or Prometheus/Alertmanager deployment.

**Alternatives considered**:
- Datadog dashboard JSON: Rejected — vendor-specific. Grafana is open-source and widely adopted.
- Custom dashboard format: Rejected — no ecosystem tooling would support it.

## Decision 10: Horizontal Scaling Pattern

**Decision**: Multiple replicas connect to the same NATS cluster using NATS queue groups for work distribution. JetStream consumers use durable consumer names with the `deliver_group` option. PostgreSQL connections use the shared pool per replica.

**Rationale**: NATS queue groups are the native mechanism for load-balanced message delivery. When multiple subscribers join the same queue group, NATS delivers each message to exactly one subscriber in the group — preventing duplication (FR-018). JetStream durable consumers with deliver groups extend this to durable messaging. This is already how `mister-smith-nats` works — Phase 8 just needs to ensure the queue group name is configurable.

**Alternatives considered**:
- Application-level work distribution: Rejected — NATS queue groups handle this natively and more efficiently.
- Leader election: Not needed for the base case. All replicas are equal workers. Leader election could be added later for singleton tasks.

## Technology Additions for Phase 8

| Dependency | Version | Purpose |
|-----------|---------|---------|
| opentelemetry | 0.31.0 | Core OTel API |
| opentelemetry_sdk | 0.31.0 | OTel SDK (TracerProvider, MeterProvider) |
| opentelemetry-otlp | 0.31.0 | OTLP gRPC exporter |
| tracing-opentelemetry | 0.32.1 | Bridge: tracing → OpenTelemetry |
| clap | 4.x | CLI argument parsing for the binary |
| tokio (signal feature) | 1.49.0 | Already included — SIGTERM/SIGINT handling |
| metrics-exporter-prometheus | 0.18.1 | Already in workspace — Prometheus backend |

## Existing Infrastructure Inventory

### Already Built (No Changes Needed)
- `tracing` instrumentation across all 18 crates
- `MetricsCollector` + `MetricsBackend` trait (monitoring)
- `MetricsRegistry` with lock-free atomics (monitoring)
- `HealthMonitor` + `HealthCheck` trait (monitoring)
- `PhiAccrualFailureDetector` (monitoring)
- `AuditLogger` with `drain_events()` (security)
- `AuditPersister` drain-to-PostgreSQL (persistence)
- `HeartbeatEmitter` (agents)
- `AgentRuntime` with lifecycle management (agents)
- Configuration loading with env overlay (config)
- HTTP API with `/api/v1/health` (http)
- NATS transport with JetStream (nats)
- `MonitoringSystem` coordinator (monitoring)

### Needs Extension
- HTTP crate: add `/health/live`, `/health/ready` probe endpoints
- Transport layer: W3C TraceContext injection/extraction in NATS headers
- Agent crate: `#[instrument]` spans on key operations

### Needs Building
- `mister-smith-app` binary crate: process lifecycle, signal handling, startup sequencing
- OpenTelemetry initialization and OTLP export configuration
- Prometheus metrics endpoint (`/metrics`)
- Dockerfile (multi-stage)
- Kubernetes manifests (Deployment, Service, ConfigMap)
- Grafana dashboard JSON definitions
- Prometheus alert rule definitions

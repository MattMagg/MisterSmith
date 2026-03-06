# Implementation Plan: Phase 8 — Operations & Production Readiness

**Branch**: `010-phase8-operations` | **Date**: 2026-03-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/010-phase8-operations/spec.md`

## Summary

Phase 8 transforms the Mister Smith framework from a collection of 18 library crates into a deployable, observable, production-grade system. The core deliverable is a new `mister-smith-app` binary crate that orchestrates process lifecycle (deterministic startup, graceful shutdown, signal handling), wires cross-phase integrations (audit bridge, security enforcement, monitoring-to-supervision), initializes the OpenTelemetry observability pipeline (distributed tracing via tracing-opentelemetry bridge, Prometheus metrics, structured JSON logging), and exposes Kubernetes-compatible health probes. Deployment artifacts include a multi-stage Dockerfile, Kubernetes manifests, Grafana dashboards, and Prometheus alert rules.

## Technical Context

**Language/Version**: Rust 1.88.0 (MSRV, driven by async-nats 0.46.0)
**Primary Dependencies**: tokio 1.49.0, async-nats 0.46.0, axum 0.8.8, tonic 0.14, sqlx 0.8, tracing 0.1.44, opentelemetry 0.31.0, tracing-opentelemetry 0.32.1, clap 4.x
**Storage**: PostgreSQL 15+ (via sqlx), JetStream KV (via async-nats)
**Testing**: `cargo test --workspace` (950+ tests as of Phase 7)
**Target Platform**: Linux containers (Kubernetes), local development on macOS/Linux
**Project Type**: Binary crate (`mister-smith-app`) + library crate extensions
**Performance Goals**: Cold start <10s (SC-001), graceful shutdown <30s (SC-002), metrics endpoint <500ms (SC-004), container image <100MB compressed (SC-006)
**Constraints**: Must not break existing 950+ tests, must wire existing components (no reimplementation), Constitution Principles VIII (integration completeness) and IX (observable by default) are mandatory
**Scale/Scope**: Single binary, horizontally scalable via NATS queue groups, 9 agent types

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Canonical Single Source of Truth | PASS | New types defined in `mister-smith-core` (ProcessLifecycle, ShutdownReason), referenced from app crate |
| II. Spec-First Design | PASS | Full spec at `specs/010-phase8-operations/spec.md` with 20 FRs, 10 SCs |
| III. Phase-Gated Build Order | PASS | Phase 8 depends on Phases 1-7 (all complete). Gate 7 passed. |
| IV. Model-Agnostic Architecture | PASS | No LLM dependencies. Phase 9 is separate. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Process lifecycle follows OTP application pattern. Supervision tree wired. |
| VI. Evidence-Based Validation | PASS | All SCs are measurable. Gate 8 criteria defined in ROADMAP.md. |
| VII. Explicit Dependency Management | PASS | New dependencies (opentelemetry, clap) documented in research.md. |
| VIII. Cross-Phase Integration Completeness | PASS | US3 explicitly wires all three identified gaps (audit, security, monitoring). |
| IX. Observable by Default | PASS | US2 implements full observability pipeline as constitutional requirement. |

**Post-design re-check**: All principles remain satisfied. The design extends existing components rather than reimplementing, preserving single-source-of-truth. Cross-phase bridges compose existing traits and types.

## Project Structure

### Documentation (this feature)

```text
specs/010-phase8-operations/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 research decisions
├── data-model.md        # Phase 1 data model
├── quickstart.md        # Phase 1 quickstart guide
├── contracts/           # Phase 1 interface contracts
│   ├── health-probes.md
│   ├── process-lifecycle.md
│   └── observability.md
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 task breakdown (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── mister-smith-app/              # NEW — Binary crate (Phase 8 primary deliverable)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                # Entry point, CLI parsing, startup/shutdown orchestration
│       ├── bootstrap.rs           # Deterministic startup sequence
│       ├── shutdown.rs            # Graceful + forced shutdown, signal handling
│       ├── observability.rs       # OTel initialization (TracerProvider, MeterProvider, subscriber)
│       ├── bridges.rs             # Cross-phase integration wiring (audit, security, monitoring)
│       └── config.rs              # App-level config extensions (ObservabilityConfig)
│
├── mister-smith-core/src/         # EXTEND — Add ProcessLifecycle, ShutdownReason types
├── mister-smith-config/src/       # EXTEND — Add ObservabilityConfig to FrameworkConfig
├── mister-smith-http/src/         # EXTEND — Add /health/live, /health/ready, /metrics endpoints
├── mister-smith-transport/src/    # EXTEND — W3C TraceContext injection/extraction in headers
├── mister-smith-agents/src/       # EXTEND — #[instrument] spans on agent operations
├── mister-smith-monitoring/src/   # EXTEND — Prometheus MetricsBackend implementation
└── mister-smith-integration-tests/ # EXTEND — Phase 8 integration tests

deploy/                            # NEW — Deployment artifacts
├── Dockerfile                     # Multi-stage build
├── docker-compose.yml             # Local development stack
├── kubernetes/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   └── namespace.yaml
├── dashboards/
│   └── mister-smith-overview.json # Grafana dashboard definition
└── alerts/
    └── mister-smith-rules.yml     # Prometheus alert rules
```

**Structure Decision**: One new binary crate (`mister-smith-app`) as the process entry point. Existing library crates are extended with targeted additions (health probes, trace context, metrics backend). Deployment artifacts go in a new top-level `deploy/` directory, separate from source code. This preserves the workspace crate structure while adding the operational layer.

## Complexity Tracking

No constitution violations to justify. The design composes existing components — no new abstractions, patterns, or architectural departures.

## Implementation Phases

### Phase A: Foundation (P1 — must complete first)

**Binary Crate + Process Lifecycle + Core Types**

1. Create `mister-smith-app` binary crate with workspace membership
2. Add `ProcessLifecycle` and `ShutdownReason` enums to `mister-smith-core`
3. Add `ObservabilityConfig` to `mister-smith-config`
4. Implement CLI argument parsing with `clap`
5. Implement startup sequence: config load → service connection → supervision init → agent spawn → health endpoints
6. Implement signal handling (SIGTERM/SIGINT via `tokio::signal`)
7. Implement graceful shutdown (reverse startup order) with timeout
8. Implement forced shutdown (second signal)

**Dependencies**: Phases 1-7 complete (all existing crates)
**Validation**: Binary starts, reaches "ready" state, exits cleanly on SIGTERM

### Phase B: Observability (P1 — parallel with Phase A once bootstrap exists)

**OpenTelemetry + Distributed Tracing + Metrics + Structured Logging**

1. Add workspace dependencies: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`
2. Implement OTel initialization in `mister-smith-app/src/observability.rs`:
   - TracerProvider with OTLP batch exporter
   - MeterProvider with OTLP periodic exporter
   - tracing-subscriber with tracing-opentelemetry layer
   - JSON/pretty log formatting
3. Implement W3C TraceContext propagation:
   - Inject traceparent/tracestate into NATS message headers (transport crate)
   - Extract on receive and set as current span context
4. Add `#[instrument]` spans to key agent operations (agents crate):
   - `agent.start`, `agent.stop`, `agent.handle_message`
   - `task.execute`, `task.decompose`, `task.aggregate`
5. Implement Prometheus `MetricsBackend` (monitoring crate):
   - Register standard metrics (counters, gauges, histograms per data-model.md)
   - Wire into existing `MetricsCollector`
6. Add `/metrics` Prometheus endpoint to HTTP server

**Dependencies**: Phase A (binary crate exists for OTel init)
**Validation**: Traces appear in collector, metrics scrapable, logs have trace IDs

### Phase C: Cross-Phase Integration (P2 — after Phase A)

**Audit Bridge + Security Enforcement + Monitoring Wiring**

1. Wire AuditLogger → AuditPersister in bootstrap sequence:
   - Create AuditPersister with reference to AuditLogger
   - Start background flush loop under supervision
   - Verify zero event loss (SC-005)
2. Wire AgentRuntime → PolicyEngine:
   - Agent operations require valid JWT token
   - Unauthorized operations rejected with audit event
3. Wire HeartbeatEmitter → PhiAccrualFailureDetector → Supervision:
   - Route agent heartbeats to failure detector
   - Failure detector suspicion triggers supervisor notification
4. Record supervision events in both metrics and audit log (FR-014)

**Dependencies**: Phase A (bootstrap wires components)
**Validation**: Audit events persist, unauthorized ops rejected, heartbeat loss detected

### Phase D: Health Probes (P2 — after Phase A)

1. Add `/health/live` endpoint to `mister-smith-http` (fast, no deps check)
2. Add `/health/ready` endpoint (checks NATS, PostgreSQL, agent count)
3. Extend existing `/api/v1/health` with new subsystem components
4. Implement readiness state tracking (503 during startup and draining)

**Dependencies**: Phase A (HTTP server running)
**Validation**: Probes respond correctly in all states (starting, ready, draining)

### Phase E: Deployment Artifacts (P3 — after Phases A-D)

1. Create multi-stage Dockerfile (rust:1.88-slim build, debian:bookworm-slim runtime)
2. Create docker-compose.yml for local development (NATS + PostgreSQL + app)
3. Create Kubernetes manifests (Deployment, Service, ConfigMap, namespace)
4. Verify container image <100MB compressed, starts <5 seconds

**Dependencies**: Phase A (binary must exist to containerize)
**Validation**: Container builds, starts, probes respond, config via env vars works

### Phase F: Dashboards and Alerting (P3 — after Phase B)

1. Create Grafana dashboard JSON:
   - Agent overview panel (active by type)
   - Message throughput panel (sent/received rates)
   - Task completion panel (success/failure rates)
   - Error rate panel
   - Supervision tree status
   - Resource utilization
2. Create Prometheus alert rules:
   - Agent failure rate spike (>5 restarts in 5min)
   - Message queue depth growth (>100 pending for >2min)
   - Heartbeat loss detected (phi > threshold)
   - High error rate (>10% task failure for >5min)

**Dependencies**: Phase B (metrics must exist for dashboards)
**Validation**: Dashboard imports into Grafana, panels render, alerts fire on threshold breach

### Phase G: Integration Testing and Gate Validation

1. End-to-end test: start → operate → observe → shutdown
2. Verify all 10 success criteria (SC-001 through SC-010)
3. Run Gate 8 validation from ROADMAP.md
4. Performance validation against targets

**Dependencies**: All previous phases
**Validation**: Gate 8 passes

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| OpenTelemetry 0.31 API instability | Low | Medium | Pin exact versions, use only stable APIs |
| Startup timeout exceeded | Low | High | Sequential startup is simple; 10s target achievable |
| Cross-phase integration bugs | Medium | High | Each bridge is independently testable |
| Container image size exceeds 100MB | Low | Low | Multi-stage build with minimal deps; static analysis of binary |
| Trace context lost in NATS routing | Medium | Medium | Unit test trace propagation at transport layer |

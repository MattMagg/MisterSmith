# Data Model: Phase 8 — Operations & Production Readiness

**Date**: 2026-03-06
**Branch**: `010-phase8-operations`

## Entities

### ProcessState

The top-level application lifecycle state machine.

```
Fields:
  - state: ProcessLifecycle (Starting | Ready | Draining | Stopped | Failed)
  - started_at: Option<Instant>
  - ready_at: Option<Instant>
  - config: FrameworkConfig
  - agents: Vec<AgentId>
  - shutdown_reason: Option<ShutdownReason>

State Transitions:
  Starting → Ready        (all services connected, agents spawned, health endpoints active)
  Starting → Failed       (startup timeout exceeded or service unreachable)
  Ready → Draining        (SIGTERM/SIGINT received)
  Draining → Stopped      (all agents stopped, state flushed, messages drained)
  Draining → Stopped      (forced: second signal received during drain)
  Any → Failed            (unrecoverable error)

Validation:
  - startup_timeout must be > 0 and < 300s
  - shutdown_timeout must be > 0 and < 120s
  - At least one external service (NATS) must be configured
```

### ShutdownReason

```
Variants:
  - Signal(SignalKind)     — SIGTERM, SIGINT
  - Error(String)          — Unrecoverable error description
  - StartupTimeout         — Ready state not reached in time
  - Forced                 — Second signal during graceful shutdown
```

### ObservabilityConfig

Configuration for the telemetry pipeline.

```
Fields:
  - otlp_endpoint: Option<String>          — OTLP collector endpoint (e.g., "http://localhost:4317")
  - otlp_protocol: OtlpProtocol            — Grpc | Http (default: Grpc)
  - service_name: String                   — Service name for resource attributes (default: "mister-smith")
  - service_version: String                — From build metadata
  - environment: String                    — From MS_ENVIRONMENT env var
  - trace_sampling_ratio: f64              — 0.0 to 1.0 (default: 1.0 in dev, 0.1 in prod)
  - metrics_export_interval: Duration      — How often to push metrics via OTLP (default: 60s)
  - log_format: LogFormat                  — Json | Pretty (default: Json in prod, Pretty in dev)
  - log_level: String                      — tracing filter directive (default: "info")
  - buffer_size: usize                     — Local telemetry buffer when collector unreachable (default: 8192)
  - prometheus_enabled: bool               — Expose /metrics endpoint (default: true)

Validation:
  - trace_sampling_ratio must be in [0.0, 1.0]
  - metrics_export_interval must be >= 5s
  - buffer_size must be in [1024, 65536]
```

### HealthProbeResponse

Lightweight response for Kubernetes probes.

```
Fields:
  - status: ProbeStatus (Ok | Unavailable)
  - timestamp: DateTime<Utc>

Serialization: JSON
  { "status": "ok", "timestamp": "2026-03-06T12:00:00Z" }
  or
  HTTP 503 with { "status": "unavailable", "timestamp": "..." }
```

### ReadinessDetail

Extended response for the readiness probe.

```
Fields:
  - status: ProbeStatus
  - checks: HashMap<String, CheckResult>

CheckResult:
  - name: String       — "nats", "postgresql", "agents"
  - ok: bool
  - message: Option<String>
  - latency_ms: Option<u64>
```

### TraceContext

W3C TraceContext fields carried in message headers.

```
Fields:
  - traceparent: String    — "00-{trace_id}-{span_id}-{flags}" (W3C format)
  - tracestate: Option<String>  — Vendor-specific key-value pairs

Header Names (in MessageEnvelope / NATS headers):
  - "traceparent"
  - "tracestate"
```

### DashboardDefinition

Grafana-compatible dashboard JSON structure (stored as files, not Rust types).

```
Fields:
  - uid: String
  - title: String
  - panels: Vec<Panel>
  - templating: Vec<Variable>
  - time: TimeRange

Panel Types:
  - TimeSeries (for rates, throughput)
  - Stat (for current values like agent count)
  - Table (for agent listing)
  - Heatmap (for latency distribution)
```

### AlertRule

Prometheus alerting rule (stored as YAML, not Rust types).

```
Fields:
  - alert: String              — Rule name
  - expr: String               — PromQL expression
  - for: Duration              — How long condition must hold
  - labels: HashMap<String, String>  — severity, team
  - annotations: HashMap<String, String>  — summary, description
```

## Relationships

```
ProcessState
  ├── owns → FrameworkConfig (loaded at startup)
  ├── owns → ObservabilityConfig (subset of FrameworkConfig)
  ├── manages → Vec<AgentId> (spawned agents)
  ├── wires → AuditLogger → AuditPersister (Phase 5→6 bridge)
  ├── wires → HeartbeatEmitter → PhiAccrualFailureDetector (Phase 7→2 bridge)
  └── wires → AgentRuntime → PolicyEngine (Phase 7→5 bridge)

HealthMonitor
  ├── checks → NatsHealthCheck
  ├── checks → PostgresHealthCheck
  ├── checks → RuntimeHealthCheck
  └── publishes → HealthProbeResponse (via HTTP endpoints)

ObservabilityPipeline
  ├── TracerProvider → OTLP Exporter
  ├── MeterProvider → OTLP Exporter + Prometheus Registry
  ├── tracing-subscriber → tracing-opentelemetry layer
  └── TraceContext ←→ MessageEnvelope headers (propagation)
```

## Metric Names (Phase 8 Standard)

```
Counters:
  mistersmith_messages_sent_total{agent_type, agent_id}
  mistersmith_messages_received_total{agent_type, agent_id}
  mistersmith_tasks_completed_total{agent_type, status}
  mistersmith_tasks_failed_total{agent_type, error_type}
  mistersmith_agent_restarts_total{agent_type, agent_id}
  mistersmith_audit_events_total{event_type}
  mistersmith_audit_events_persisted_total{}

Gauges:
  mistersmith_agents_active{agent_type}
  mistersmith_message_queue_depth{subject}
  mistersmith_process_state{state}

Histograms:
  mistersmith_task_duration_seconds{agent_type}
  mistersmith_message_latency_seconds{agent_type}
  mistersmith_health_check_duration_seconds{check_name}
```

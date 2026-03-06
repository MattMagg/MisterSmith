# Contract: Process Lifecycle

## Binary Interface

```
mister-smith [OPTIONS]

OPTIONS:
  -c, --config <PATH>     Configuration file path (default: auto-discover)
  --log-level <LEVEL>     Override log level (trace, debug, info, warn, error)
  --log-format <FORMAT>   Override log format (json, pretty)
  --version               Print version and exit
  --help                  Print help and exit
```

## Environment Variables

All environment variables use the `MISTER_SMITH_` prefix with `__` as the nested separator:

```
MISTER_SMITH_TRANSPORT__NATS_URL=nats://nats:4222
MISTER_SMITH_TRANSPORT__HTTP_PORT=8080
MISTER_SMITH_TRANSPORT__GRPC_PORT=50051
MISTER_SMITH_SECURITY__ENABLED=true
MISTER_SMITH_PERSISTENCE__DATABASE_URL=postgres://...
MISTER_SMITH_OBSERVABILITY__OTLP_ENDPOINT=http://otel-collector:4317
MISTER_SMITH_OBSERVABILITY__TRACE_SAMPLING_RATIO=0.1
MISTER_SMITH_OBSERVABILITY__LOG_LEVEL=info
MS_ENVIRONMENT=production
```

## Startup Sequence

```
1. Parse CLI arguments
2. Load configuration (file + env overlay)
3. Validate configuration completeness
4. Initialize observability pipeline (tracing subscriber + OTel providers)
5. Start Tokio runtime via RuntimeManager
6. Connect to NATS (with timeout, fail-fast on unreachable)
7. Connect to PostgreSQL (with timeout, fail-fast on unreachable)
8. Initialize supervision tree
9. Wire cross-phase bridges:
   a. AuditLogger → AuditPersister (start flush loop)
   b. HeartbeatEmitter → PhiAccrualFailureDetector
   c. AgentRuntime → PolicyEngine (JWT auth)
10. Spawn initial agents from configuration
11. Start HTTP server (health probes + API + metrics)
12. Start gRPC server (if configured)
13. Set process state to Ready
14. Log: "Mister Smith ready" with startup duration
```

**Timeout**: If step 13 not reached within `startup_timeout` (default: 30s), log error and exit with code 1.

## Shutdown Sequence

Triggered by SIGTERM or SIGINT:

```
1. Set process state to Draining
2. Readiness probe returns 503 (removed from load balancer)
3. Stop accepting new messages from external sources
4. Wait for in-flight messages to complete (with drain_timeout)
5. Stop agents in reverse start order
6. Flush AuditPersister (final drain)
7. Flush MetricsCollector
8. Flush OpenTelemetry providers (TracerProvider::shutdown, MeterProvider::shutdown)
9. Close NATS connection
10. Close PostgreSQL pool
11. Stop HTTP/gRPC servers
12. Set process state to Stopped
13. Log: "Mister Smith stopped" with shutdown duration
14. Exit with code 0
```

**Forced shutdown**: If a second SIGTERM/SIGINT arrives during graceful shutdown, skip steps 3-8 and proceed directly to step 9.

**Timeout**: If shutdown not complete within `shutdown_timeout` (default: 30s), log warning and exit with code 1.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0    | Clean shutdown |
| 1    | Startup failure (config invalid, service unreachable, timeout) |
| 2    | Forced shutdown (second signal) |
| 130  | SIGINT (Ctrl+C) without graceful shutdown completion |
| 143  | SIGTERM without graceful shutdown completion |

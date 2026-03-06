# Quickstart: Phase 8 — Operations & Production Readiness

## Prerequisites

- Rust 1.88.0+ (MSRV)
- Docker (for container builds and local services)
- NATS server (Docker: `docker run -d --name nats -p 4222:4222 -p 8222:8222 nats:latest`)
- PostgreSQL 15+ (Docker: `docker run -d --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=dev postgres:15`)
- Optional: OpenTelemetry Collector (for trace/metric export)

## Local Development

### 1. Build the binary

```bash
cargo build -p mister-smith-app
```

### 2. Run with defaults

```bash
# Minimal — connects to local NATS and PostgreSQL
MISTER_SMITH_TRANSPORT__NATS_URL=nats://localhost:4222 \
MISTER_SMITH_PERSISTENCE__DATABASE_URL=postgres://postgres:dev@localhost:5432/mistersmith \
  cargo run -p mister-smith-app
```

### 3. Verify health

```bash
# Liveness
curl http://localhost:8080/health/live

# Readiness
curl http://localhost:8080/health/ready

# Detailed health
curl http://localhost:8080/api/v1/health

# Metrics
curl http://localhost:8080/metrics
```

### 4. Run with observability

```bash
# Start OTel collector (Jaeger all-in-one for local dev)
docker run -d --name jaeger \
  -p 4317:4317 \
  -p 16686:16686 \
  jaegertracing/all-in-one:latest

# Run with OTLP export
MISTER_SMITH_TRANSPORT__NATS_URL=nats://localhost:4222 \
MISTER_SMITH_PERSISTENCE__DATABASE_URL=postgres://postgres:dev@localhost:5432/mistersmith \
MISTER_SMITH_OBSERVABILITY__OTLP_ENDPOINT=http://localhost:4317 \
MISTER_SMITH_OBSERVABILITY__LOG_FORMAT=pretty \
  cargo run -p mister-smith-app

# View traces at http://localhost:16686
```

## Container Build

```bash
# Build the image
docker build -t mister-smith:latest .

# Run
docker run -d \
  -p 8080:8080 \
  -e MISTER_SMITH_TRANSPORT__NATS_URL=nats://nats:4222 \
  -e MISTER_SMITH_PERSISTENCE__DATABASE_URL=postgres://postgres:dev@postgres:5432/mistersmith \
  mister-smith:latest

# Verify
docker exec <container> curl -s localhost:8080/health/live
```

## Kubernetes Deployment

```bash
# Apply manifests
kubectl apply -f deploy/kubernetes/

# Verify
kubectl get pods -l app=mister-smith
kubectl logs -f deployment/mister-smith
```

## Testing Phase 8

```bash
# All workspace tests
cargo test --workspace

# Integration tests (requires NATS + PostgreSQL)
NATS_URL=nats://localhost:4222 \
DATABASE_URL=postgres://postgres:dev@localhost:5432/mistersmith \
  cargo test -p mister-smith-integration-tests

# Verify graceful shutdown
cargo run -p mister-smith-app &
PID=$!
sleep 5
kill -TERM $PID  # Should exit 0 after draining
```

## Validation Checklist

- [ ] `curl /health/live` returns 200
- [ ] `curl /health/ready` returns 200 (all deps connected)
- [ ] `curl /metrics` returns Prometheus format with expected metrics
- [ ] Traces appear in collector (if configured)
- [ ] `kill -TERM <pid>` results in clean exit code 0
- [ ] Second `kill -TERM` during shutdown forces immediate exit
- [ ] Container image builds and starts in <5 seconds
- [ ] Dashboard JSON imports into Grafana successfully

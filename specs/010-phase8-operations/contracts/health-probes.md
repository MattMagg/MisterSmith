# Contract: Health Probe Endpoints

## Liveness Probe

```
GET /health/live

Response 200 (process alive):
{
  "status": "ok",
  "timestamp": "2026-03-06T12:00:00Z"
}

Response 503 (process unhealthy — deadlocked, unresponsive):
{
  "status": "unavailable",
  "timestamp": "2026-03-06T12:00:00Z"
}
```

**Behavior**: Returns 200 if the HTTP server is responding. Only returns 503 if the process is genuinely stuck (e.g., Tokio runtime not scheduling tasks). This probe should be fast (<10ms) and never check external dependencies.

**Kubernetes configuration**:
```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3
```

## Readiness Probe

```
GET /health/ready

Response 200 (ready to accept work):
{
  "status": "ok",
  "checks": {
    "nats": { "ok": true, "latency_ms": 2 },
    "postgresql": { "ok": true, "latency_ms": 5 },
    "agents": { "ok": true, "message": "4 agents running" }
  },
  "timestamp": "2026-03-06T12:00:00Z"
}

Response 503 (not ready):
{
  "status": "unavailable",
  "checks": {
    "nats": { "ok": false, "message": "connection refused" },
    "postgresql": { "ok": true, "latency_ms": 5 },
    "agents": { "ok": false, "message": "0 of 4 agents running" }
  },
  "timestamp": "2026-03-06T12:00:00Z"
}
```

**Behavior**: Returns 200 only when ALL external dependencies are connected and agents are running. During startup, returns 503 until the system reaches "ready" state. During shutdown/draining, returns 503 to remove the pod from the service.

**Kubernetes configuration**:
```yaml
readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 5
  failureThreshold: 2
```

## Metrics Endpoint

```
GET /metrics

Response 200 (Prometheus text format):
# HELP mistersmith_messages_sent_total Total messages sent
# TYPE mistersmith_messages_sent_total counter
mistersmith_messages_sent_total{agent_type="worker",agent_id="abc123"} 42
...
```

**Behavior**: Returns all registered metrics in Prometheus exposition format. Response time should be <500ms (SC-004).

## Detailed Health (Existing)

```
GET /api/v1/health

Response 200:
{
  "status": "healthy",
  "components": [
    { "name": "http_server", "status": "healthy" },
    { "name": "nats_transport", "status": "healthy" },
    { "name": "postgresql", "status": "healthy" },
    { "name": "supervision_tree", "status": "healthy", "message": "4 agents supervised" },
    { "name": "audit_persister", "status": "healthy", "message": "0 events pending" }
  ]
}
```

**Behavior**: Existing endpoint from Phase 4. Extended in Phase 8 to include all subsystem components.

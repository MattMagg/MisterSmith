# HTTP API Contract: Transport & Messaging

Spec reference: FR-011, FR-012, FR-013, FR-022, FR-023, FR-024, US4

## Base Configuration

- Default bind: `0.0.0.0:8080`
- Content-Type: `application/json`
- All responses include `X-Request-Id` header (UUID v4)

## REST Endpoints

### GET /api/v1/health

System health status. Kubernetes-compatible.

**Response 200**:
```json
{
  "status": "healthy | degraded | unhealthy",
  "components": {
    "nats": { "status": "connected", "latency_ms": 2 },
    "jetstream": { "status": "connected" },
    "actors": { "status": "healthy", "count": 42 }
  },
  "uptime_secs": 3600
}
```

### GET /api/v1/agents

List all active agents.

**Query Parameters**:
- `availability` (optional): Filter by `idle`, `busy`, `offline`
- `type` (optional): Filter by agent type

**Response 200**:
```json
{
  "agents": [
    {
      "agent_id": "agent-001",
      "agent_type": "worker",
      "availability": "idle",
      "active_tasks": 0,
      "load": 0.0,
      "started_at": "2026-03-04T12:00:00Z"
    }
  ],
  "total": 1
}
```

### GET /api/v1/agents/{agent_id}

Get a specific agent's details.

**Response 200**: Single agent object (same shape as list item).
**Response 404**: `{ "error": "agent_not_found", "message": "Agent 'agent-xyz' not found" }`

### POST /api/v1/tasks

Submit a task for execution.

**Request Body**:
```json
{
  "task_type": "analysis",
  "payload": { "document_id": "doc-123" },
  "priority": "normal",
  "deadline": "2026-03-04T13:00:00Z",
  "assigned_agent": "agent-001",
  "metadata": { "source": "api" }
}
```

**Response 202**:
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "assigned_agent_id": "agent-001",
  "status": "assigned"
}
```

**Response 400**: `{ "error": "invalid_request", "message": "..." }`

### GET /api/v1/tasks/{task_id}

Get task status and result.

**Response 200**:
```json
{
  "task_id": "550e8400-...",
  "status": "success | failure | partial | pending | running",
  "result": { ... },
  "duration_ms": 1234,
  "agent_id": "agent-001"
}
```

**Response 404**: `{ "error": "task_not_found", "message": "..." }`

### GET /api/v1/config

Get system configuration.

**Query Parameters**:
- `component` (optional): Filter by component name

**Response 200**:
```json
{
  "config": {
    "transport.nats.server_urls": "[\"nats://localhost:4222\"]",
    "transport.http.rate_limit_rps": "100"
  }
}
```

## WebSocket Endpoint

### GET /api/v1/events/ws

WebSocket upgrade endpoint for real-time event streaming.

**Connection**: Standard WebSocket upgrade via `Connection: Upgrade` + `Upgrade: websocket`.

**Query Parameters**:
- `filter` (optional): Comma-separated event categories: `agent_status`, `task_progress`, `system_events`. Default: all categories.

**Server → Client Messages**:
```json
{
  "type": "agent_status",
  "data": {
    "agent_id": "agent-001",
    "availability": "busy",
    "timestamp": "2026-03-04T12:00:01Z"
  }
}
```

```json
{
  "type": "task_progress",
  "data": {
    "task_id": "550e8400-...",
    "status": "running",
    "progress_pct": 75
  }
}
```

```json
{
  "type": "system_event",
  "data": {
    "event_type": "agent_spawned",
    "source": "supervision",
    "severity": "info",
    "message": "Agent agent-002 spawned"
  }
}
```

**Keepalive**: Server sends WebSocket ping every 30 seconds (configurable). Client must respond with pong. Connections not responding within 10 seconds of a ping are terminated.

**Client → Server Messages**:
```json
{
  "action": "subscribe",
  "categories": ["agent_status", "task_progress"]
}
```

```json
{
  "action": "unsubscribe",
  "categories": ["system_events"]
}
```

## Middleware

### Request ID Tracking (FR-012)

Every request receives a `X-Request-Id` header (UUID v4). If the client sends `X-Request-Id`, the server preserves it. Logged with every request for tracing.

### Rate Limiting (FR-012)

Default: 100 requests/second per client IP. Exceeded requests receive `429 Too Many Requests` with `Retry-After` header.

### Security Hooks (FR-013)

Middleware points for authentication and authorization. In Phase 4, these hooks exist but do not enforce — all requests pass through. Phase 5 populates them with JWT validation, mTLS, and RBAC checks.

## Error Format

All errors follow a consistent structure:

```json
{
  "error": "error_code",
  "message": "Human-readable description",
  "request_id": "uuid"
}
```

Standard HTTP status codes: 200 (OK), 202 (Accepted), 400 (Bad Request), 404 (Not Found), 429 (Rate Limited), 500 (Internal Error).

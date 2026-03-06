# Contract: Observability Pipeline

## Trace Context Propagation

### NATS Messages

Trace context injected into NATS message headers:

```
Header: traceparent = "00-{32-hex-trace-id}-{16-hex-span-id}-{2-hex-flags}"
Header: tracestate = "mistersmith=agent_type:worker,agent_id:abc123"
```

### HTTP Requests

Standard W3C TraceContext headers (handled by tracing-opentelemetry middleware):

```
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
tracestate: mistersmith=...
```

### gRPC Metadata

Same W3C TraceContext via tonic metadata:

```
traceparent: 00-...
tracestate: ...
```

## Span Naming Conventions

```
Agent operations:
  agent.start          — Agent initialization
  agent.stop           — Agent shutdown
  agent.handle_message — Processing a received message
  agent.heartbeat      — Heartbeat emission

Task operations:
  task.execute         — Task execution lifecycle
  task.decompose       — Coordinator decomposing a task
  task.aggregate       — Coordinator aggregating results

Messaging:
  transport.publish    — Publishing a message
  transport.subscribe  — Receiving a message
  transport.request    — Request-reply cycle

Supervision:
  supervision.restart  — Restarting a failed agent
  supervision.escalate — Escalating failure to parent

Persistence:
  audit.flush          — Flushing audit events to PostgreSQL
  persistence.query    — Database query execution
```

## Span Attributes

```
Required on all spans:
  service.name         = "mister-smith"
  service.version      = build version
  deployment.environment = from MS_ENVIRONMENT

Agent spans:
  agent.id             = AgentId (UUID)
  agent.type           = AgentType variant name
  agent.state          = Current AgentState

Task spans:
  task.id              = TaskId (UUID)
  task.type            = Task type name
  task.status          = completion status

Message spans:
  messaging.system     = "nats"
  messaging.destination = NATS subject
  messaging.message_id = MessageId (UUID)
  messaging.priority   = MessagePriority variant
```

## Metric Exposition

### Prometheus Format (GET /metrics)

```
# HELP mistersmith_messages_sent_total Total messages sent by agents
# TYPE mistersmith_messages_sent_total counter
mistersmith_messages_sent_total{agent_type="worker",agent_id="abc"} 142

# HELP mistersmith_messages_received_total Total messages received by agents
# TYPE mistersmith_messages_received_total counter
mistersmith_messages_received_total{agent_type="coordinator",agent_id="def"} 89

# HELP mistersmith_tasks_completed_total Tasks completed
# TYPE mistersmith_tasks_completed_total counter
mistersmith_tasks_completed_total{agent_type="worker",status="success"} 120
mistersmith_tasks_completed_total{agent_type="worker",status="failure"} 2

# HELP mistersmith_agent_restarts_total Agent restart count
# TYPE mistersmith_agent_restarts_total counter
mistersmith_agent_restarts_total{agent_type="worker",agent_id="abc"} 1

# HELP mistersmith_agents_active Currently active agents
# TYPE mistersmith_agents_active gauge
mistersmith_agents_active{agent_type="worker"} 3
mistersmith_agents_active{agent_type="coordinator"} 1

# HELP mistersmith_message_queue_depth Current message queue depth
# TYPE mistersmith_message_queue_depth gauge
mistersmith_message_queue_depth{subject="tasks.assignment"} 5

# HELP mistersmith_task_duration_seconds Task execution duration
# TYPE mistersmith_task_duration_seconds histogram
mistersmith_task_duration_seconds_bucket{agent_type="worker",le="0.1"} 80
mistersmith_task_duration_seconds_bucket{agent_type="worker",le="0.5"} 110
mistersmith_task_duration_seconds_bucket{agent_type="worker",le="1.0"} 118
mistersmith_task_duration_seconds_bucket{agent_type="worker",le="+Inf"} 120
mistersmith_task_duration_seconds_sum{agent_type="worker"} 45.2
mistersmith_task_duration_seconds_count{agent_type="worker"} 120

# HELP mistersmith_health_check_duration_seconds Health check execution time
# TYPE mistersmith_health_check_duration_seconds histogram
mistersmith_health_check_duration_seconds_bucket{check_name="nats",le="0.01"} 95
mistersmith_health_check_duration_seconds_bucket{check_name="nats",le="+Inf"} 100
```

## Structured Log Format

JSON format (production):

```json
{
  "timestamp": "2026-03-06T12:00:00.123Z",
  "level": "INFO",
  "target": "mister_smith_agents::runtime",
  "message": "Agent started",
  "span": {
    "name": "agent.start",
    "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
    "span_id": "00f067aa0ba902b7"
  },
  "fields": {
    "agent.id": "550e8400-e29b-41d4-a716-446655440000",
    "agent.type": "Worker",
    "startup_duration_ms": 42
  }
}
```

Pretty format (development):

```
2026-03-06T12:00:00.123Z  INFO agent.start{agent.id=550e8400 agent.type=Worker}: Agent started (42ms)
```

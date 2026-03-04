# Data Model: Transport & Messaging

## Core Transport Entities

### MessageEnvelope

Universal message wrapper for all framework communication.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| message_id | UUID (v4) | Yes | Unique message identifier |
| timestamp | DateTime (RFC 3339) | Yes | Message creation time |
| schema_version | String (semver) | Yes | Envelope schema version (default "1.0.0") |
| message_type | String | Yes | Discriminator for routing and deserialization |
| correlation_id | UUID (v4) | No | Links request to response |
| trace_id | UUID (v4) | No | Distributed tracing identifier |
| source_agent_id | AgentId | No | Sending agent |
| target_agent_id | AgentId | No | Intended recipient |
| priority | MessagePriority | Yes | Message priority level (default Normal) |
| payload | Bytes | Yes | Serialized message content (MessagePack) |
| headers | HashMap<String, String> | No | Transport-level metadata |

**Validation rules**:
- message_id MUST be unique per message (UUID v4)
- schema_version MUST be valid semver
- message_type MUST be non-empty
- payload size MUST NOT exceed transport max payload (configurable, default 1MB)

**Serialization**: Envelope itself serialized with `rmp_serde::to_vec_named` for wire. JSON via `serde_json` for HTTP endpoints and debugging.

### AgentAvailability

Transport-level presence status, distinct from lifecycle `AgentState`.

| Variant | Description |
|---------|-------------|
| Idle | Agent is available and waiting for work |
| Busy | Agent is processing a task, may accept queued messages |
| Offline | Agent is not reachable on the transport layer |

**Transition rules**:
- Idle → Busy: Agent begins processing a task
- Busy → Idle: Agent completes task processing
- Any → Offline: Transport connection lost or agent shutdown
- Offline → Idle: Agent reconnects to transport

### SubjectTaxonomy

Hierarchical NATS subject naming scheme.

| Pattern | Category | Description |
|---------|----------|-------------|
| `agents.{agent_id}.commands.{type}` | Agent | Commands directed to a specific agent |
| `agents.{agent_id}.status` | Agent | Agent availability status updates |
| `agents.{agent_id}.heartbeat` | Agent | Periodic heartbeat |
| `agents.{agent_id}.events.{type}` | Agent | Agent lifecycle events |
| `tasks.{task_type}.assignment` | Task | Task assignment (queue group eligible) |
| `tasks.{task_type}.queue.{priority}` | Task | Priority-based task queues |
| `tasks.{task_id}.progress` | Task | Task progress updates |
| `tasks.{task_id}.result` | Task | Task completion results |
| `system.events.{type}` | System | System-wide events |
| `system.config.{component}` | System | Configuration updates |
| `system.health` | System | Health check signals |
| `workflow.{workflow_id}.start` | Workflow | Workflow initiation |
| `workflow.{workflow_id}.step.{step_id}` | Workflow | Step completion |
| `workflow.{workflow_id}.result` | Workflow | Workflow result |

**Wildcard patterns**: `agents.>` (all agent subjects), `tasks.*.assignment` (all task assignments).

## Message Types

### TaskAssignment

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| task_id | UUID | Yes | Unique task identifier |
| task_type | String | Yes | Task category for routing |
| payload | Value | Yes | Task-specific parameters |
| priority | MessagePriority | Yes | Execution priority |
| deadline | DateTime | No | Task deadline |
| assigned_agent | AgentId | No | Specific agent target |
| requester_id | AgentId | Yes | Requesting agent |
| metadata | HashMap<String, String> | No | Arbitrary key-value metadata |

### TaskResult

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| task_id | UUID | Yes | Matching TaskAssignment task_id |
| status | TaskStatus | Yes | Success / Failure / Partial |
| result | Value | No | Task output data |
| error | String | No | Error description if failed |
| duration_ms | u64 | Yes | Execution time in milliseconds |
| agent_id | AgentId | Yes | Agent that processed the task |

### AgentHeartbeat

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| agent_id | AgentId | Yes | Heartbeat source |
| availability | AgentAvailability | Yes | Current transport status |
| load | f64 | No | Current load factor (0.0-1.0) |
| active_tasks | u32 | No | Number of in-progress tasks |
| uptime_secs | u64 | Yes | Agent uptime |

### SystemEvent

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| event_type | String | Yes | Event category |
| source | String | Yes | Component that generated event |
| severity | Severity | Yes | Info / Warning / Error / Critical |
| message | String | Yes | Human-readable description |
| data | Value | No | Structured event data |

### WorkflowStart / StepComplete / WorkflowResult

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| workflow_id | UUID | Yes | Workflow identifier |
| step_id | String | Conditional | Step identifier (StepComplete only) |
| status | WorkflowStatus | Yes | Active / Completed / Failed |
| output | Value | No | Step/workflow output |
| next_steps | Vec<String> | No | Remaining steps (WorkflowStart) |

### AgentSpawn / AgentTerminate / ConfigUpdate

| Entity | Key Fields |
|--------|------------|
| AgentSpawn | agent_id, agent_type, config |
| AgentTerminate | agent_id, reason, graceful (bool) |
| ConfigUpdate | component, key, value, previous_value |

## Transport Configuration Entities

### NatsTransportConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| server_urls | Vec<String> | ["nats://localhost:4222"] | NATS server addresses |
| name | String | "mister-smith" | Connection name |
| max_reconnects | Option<usize> | None (unlimited) | Max reconnection attempts |
| connection_timeout | Duration | 5s | Initial connection timeout |
| request_timeout | Duration | 10s | Default request-reply timeout |
| client_capacity | usize | 2048 | Internal send buffer size |
| subscription_capacity | usize | 65536 | Per-subscriber buffer |
| tls_required | bool | false | Require TLS |
| tls_cert | Option<PathBuf> | None | Client certificate path |
| tls_key | Option<PathBuf> | None | Client key path |
| tls_ca | Option<PathBuf> | None | CA certificate path |

### JetStreamConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | true | Enable JetStream |
| domain | Option<String> | None | JetStream domain |
| max_ack_inflight | usize | 5000 | Max pending publish acks |
| ack_timeout | Duration | 30s | Publish ack timeout |

### HttpTransportConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| bind_address | String | "0.0.0.0:8080" | HTTP listen address |
| websocket_enabled | bool | true | Enable WebSocket endpoint |
| ws_keepalive_interval | Duration | 30s | WebSocket ping interval |
| max_ws_connections | usize | 1000 | Max concurrent WebSocket connections |
| rate_limit_rps | u32 | 100 | Requests per second limit |

### GrpcTransportConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| bind_address | String | "0.0.0.0:50051" | gRPC listen address |
| max_message_size | usize | 4MB | Max gRPC message size |

### McpConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | false | Enable MCP integration |
| clients | Vec<McpClientConfig> | [] | External MCP servers |
| servers | Vec<McpServerConfig> | [] | MCP server endpoints |
| nats_bridge_enabled | bool | false | Enable NATS-MCP bridge |
| nats_bridge_prefix | String | "ms.mcp" | NATS subject prefix for MCP |

## Entity Relationships

```
MessageEnvelope
  ├── contains → Payload (TaskAssignment | TaskResult | AgentHeartbeat | SystemEvent | ...)
  ├── routes via → SubjectTaxonomy
  └── serialized by → rmp-serde (wire) | serde_json (HTTP)

Transport (trait)
  ├── NatsTransport → async-nats Client → NATS Server
  │     └── JetStreamContext → Streams, Consumers
  ├── HttpTransport → Axum Router → REST + WebSocket
  ├── GrpcTransport → Tonic Server → Protobuf Services
  └── McpClient/McpServer → rmcp → stdio | streamable-HTTP

AgentAvailability (transport status)
  └── distinct from → AgentState (lifecycle status, Phase 3)
```

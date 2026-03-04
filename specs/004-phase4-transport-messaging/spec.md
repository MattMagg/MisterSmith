# Feature Specification: Transport & Messaging

**Feature Branch**: `004-phase4-transport-messaging`
**Created**: 2026-03-04
**Status**: Draft
**Input**: User description: "Phase 4: Transport & Messaging — NATS transport, HTTP/gRPC endpoints, message envelope and serialization"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Transport Abstraction & Message Envelope (Priority: P1)

As a framework developer, I can send structured messages between actors through a protocol-agnostic transport layer so that my application logic is decoupled from the underlying messaging protocol.

The transport abstraction defines a common `Transport` trait with publish, subscribe, request, and reply operations. All messages are wrapped in a standardized envelope containing a unique message ID, timestamp, schema version, message type discriminator, correlation ID for request-reply patterns, source and target agent IDs, priority level, and a serialized payload. The envelope format is consistent regardless of whether the message travels over NATS, HTTP, or gRPC.

**Why this priority**: Everything else in the transport layer depends on this abstraction. Without a unified envelope and trait contract, each transport protocol would define its own incompatible message format, making cross-protocol communication impossible.

**Independent Test**: Can be fully tested with an in-memory transport implementation that validates envelope construction, serialization round-trips, and trait compliance without any external dependencies.

**Acceptance Scenarios**:

1. **Given** a message payload and routing metadata, **When** the framework wraps it in an envelope, **Then** the envelope contains a valid UUID message ID, ISO 8601 timestamp, schema version, message type, and serialized payload.
2. **Given** a serialized envelope, **When** it is deserialized, **Then** all envelope fields are recovered exactly and the payload is intact.
3. **Given** a request message with a correlation ID, **When** the response is sent, **Then** the response envelope carries the same correlation ID.
4. **Given** a custom payload type that implements serialization, **When** it is wrapped in an envelope, **Then** the envelope can carry any conforming payload without modification.
5. **Given** a transport trait implementation, **When** the framework publishes a message, **Then** the trait handles serialization, routing, and delivery confirmation uniformly.

---

### User Story 2 - NATS Inter-Agent Communication (Priority: P2)

As a framework developer, I can have actors communicate over NATS using publish/subscribe, request/reply, and queue group patterns so that agents can coordinate work across distributed processes.

NATS is the primary inter-agent transport. The framework connects to a NATS server, maps actor addresses to NATS subjects using a hierarchical subject taxonomy (e.g., `agents.<id>.commands.<type>`), and supports queue groups for load-balanced message delivery. Request-reply operations include configurable timeouts. The NATS transport handles connection lifecycle including automatic reconnection.

**Why this priority**: NATS is the backbone of the multi-agent framework — all inter-agent communication flows through it. This is the most critical transport after the abstraction layer.

**Independent Test**: Can be tested against a running NATS server (Docker container) by publishing messages on subjects, subscribing with queue groups, and verifying request-reply round-trips.

**Acceptance Scenarios**:

1. **Given** two actors with known IDs, **When** actor A publishes a message to actor B's subject, **Then** actor B receives the message with correct envelope metadata.
2. **Given** a NATS subject with multiple subscribers in a queue group, **When** a message is published, **Then** exactly one subscriber in the group receives the message.
3. **Given** actor A sends a request to actor B, **When** actor B replies within the timeout, **Then** actor A receives the response with matching correlation ID.
4. **Given** actor A sends a request to actor B, **When** actor B does not reply within the timeout, **Then** actor A receives a timeout error.
5. **Given** the NATS server becomes temporarily unreachable, **When** the connection is restored, **Then** the transport automatically reconnects and resumes message delivery.
6. **Given** a publish operation, **When** the internal send buffer is full, **Then** the operation applies backpressure (awaits capacity) rather than dropping the message.

---

### User Story 3 - Durable Messaging via JetStream (Priority: P3)

As a framework developer, I can persist messages in NATS JetStream streams so that messages survive agent restarts and network partitions, enabling reliable at-least-once delivery.

JetStream provides durable message storage with configurable retention policies. The framework can create streams for specific subject patterns, publish durable messages, and consume them via pull or push consumers with explicit acknowledgment. This ensures that critical messages (task assignments, workflow events) are not lost when agents restart.

**Why this priority**: Durability is essential for production reliability. Without persistent messaging, agent restarts cause message loss, breaking workflow continuity.

**Independent Test**: Can be tested by publishing messages to a JetStream stream, stopping and restarting a consumer, and verifying all messages are delivered after recovery.

**Acceptance Scenarios**:

1. **Given** a JetStream stream configured for task-related subjects, **When** a task assignment message is published, **Then** the message is stored persistently in the stream.
2. **Given** a consumer subscribed to a stream, **When** the consumer disconnects and reconnects, **Then** it receives all unacknowledged messages from where it left off.
3. **Given** a consumer that explicitly acknowledges a message, **When** the consumer is restarted, **Then** the acknowledged message is not redelivered.
4. **Given** a message that has not been acknowledged within the configured timeout, **When** the timeout expires, **Then** the message is redelivered to an available consumer.
5. **Given** a stream with a retention policy, **When** messages exceed the configured limits (count, size, or age), **Then** the oldest messages are purged according to the policy.

---

### User Story 4 - HTTP API & WebSocket Endpoints (Priority: P4)

As a framework developer, I can expose my agent system through REST API endpoints and WebSocket connections so that external clients can interact with the system over standard HTTP and receive real-time streaming updates.

The HTTP transport provides REST endpoints for system management (agent listing, health status, configuration), task submission, and monitoring. WebSocket connections enable real-time streaming of agent status changes, task progress events, and system events without polling. It includes middleware hooks for request ID tracking and rate limiting. Security enforcement middleware points are provided but not enforced until Phase 5.

**Why this priority**: HTTP access is necessary for management UIs, monitoring dashboards, and external integrations, but is not required for core agent-to-agent communication. WebSocket streaming is essential for real-time operational dashboards.

**Independent Test**: Can be tested by starting the HTTP server, sending requests to management endpoints, opening a WebSocket connection, and verifying both JSON responses and streaming event delivery.

**Acceptance Scenarios**:

1. **Given** a running agent system with the HTTP transport enabled, **When** a client sends a GET request to the agents endpoint, **Then** it receives a JSON list of active agents with their current status.
2. **Given** a running agent system, **When** a client sends a GET request to the health endpoint, **Then** it receives the system health status (healthy, degraded, or unhealthy) with component details.
3. **Given** a task submission endpoint, **When** a client sends a POST request with a valid task payload, **Then** the task is assigned to an available agent and the client receives a task ID.
4. **Given** a request to the API, **When** the middleware processes it, **Then** a unique request ID is assigned and included in the response headers.
5. **Given** the HTTP transport configuration, **When** security middleware hooks are defined, **Then** they are executed in the request pipeline but do not enforce authentication (deferred to Phase 5).
6. **Given** a client opens a WebSocket connection to the events endpoint, **When** an agent status changes or a task completes, **Then** the client receives the event as a JSON message in real-time.
7. **Given** a WebSocket connection, **When** the client subscribes to a specific event filter (e.g., agent status only), **Then** it receives only events matching the filter.
8. **Given** a WebSocket connection that is idle, **When** no events occur for 30 seconds, **Then** the server sends a keepalive ping to maintain the connection.

---

### User Story 5 - gRPC Service Layer (Priority: P5)

As a framework developer, I can connect agents via high-performance gRPC services for type-safe, streaming inter-service communication in service mesh deployments.

The gRPC transport defines protobuf service definitions for agent communication, task management, and system operations. It supports unary RPCs for request-reply and streaming RPCs for continuous agent-to-agent data flows. Error mapping converts transport-level gRPC status codes to framework error types.

**Why this priority**: gRPC is an alternative transport for high-throughput service mesh environments. While important for production deployments, the framework can operate fully with NATS and HTTP alone.

**Independent Test**: Can be tested by starting a gRPC server, calling service methods with a gRPC client, and verifying correct protobuf response encoding and error mapping.

**Acceptance Scenarios**:

1. **Given** a gRPC agent service, **When** a client sends a unary RPC to submit a task, **Then** the server processes it and returns a typed protobuf response.
2. **Given** a streaming RPC for agent status updates, **When** a client opens a stream, **Then** it receives a continuous flow of status updates until the stream is closed.
3. **Given** a gRPC request that triggers a framework error, **When** the server maps the error, **Then** the client receives an appropriate gRPC status code with error details.
4. **Given** a gRPC health check service, **When** a client queries health, **Then** it receives the standard gRPC health check response compatible with Kubernetes probes.

---

### User Story 6 - MCP Integration (Priority: P6)

As a framework developer, I can connect my agent system to external tools via the Model Context Protocol (MCP) as both a client and server, so that agents can consume external MCP tool servers and expose their own capabilities to external MCP clients.

The MCP integration operates in two modes. As an **MCP client**, the framework connects to external MCP servers (filesystem, database, API tools) using stdio or streamable-HTTP transports, auto-discovers available tools via `tools/list`, and registers them into the agent tool system. As an **MCP server**, the framework exposes agent tools to external MCP clients through a streamable-HTTP endpoint with namespace-scoped views and tool filtering. The integration includes tool caching with `list_changed` invalidation, on-demand session management (lazy connect on first tool call), and a two-layer auth model (transport-level auth plus tool-level permission checks). As a unique differentiator, MCP tool calls can be bridged over NATS for distributed tool discovery and remote execution across nodes.

**Why this priority**: MCP integration connects agents to the real world (external tools, databases, APIs) and allows the framework to participate in the broader MCP ecosystem. However, the core transport layer (NATS, HTTP, gRPC) must be established first since MCP transports build on top of them.

**Independent Test**: MCP client can be tested by connecting to a mock MCP server over stdio, discovering tools, and invoking them. MCP server can be tested by starting the streamable-HTTP endpoint and using an MCP client to call `tools/list` and `tools/call`.

**Acceptance Scenarios**:

1. **Given** an external MCP server configured in the framework, **When** the framework starts, **Then** it connects (lazily on first tool call or eagerly based on config), calls `tools/list`, and registers discovered tools into the agent tool system.
2. **Given** a registered MCP tool, **When** an agent invokes the tool, **Then** the framework routes the call to the external MCP server and returns the result.
3. **Given** tool caching is enabled, **When** an MCP server sends a `notifications/tools/list_changed` notification, **Then** the cached tool list is invalidated and refreshed on next access.
4. **Given** a tool filter configured for an MCP client connection (e.g., `read_*`), **When** tools are discovered, **Then** only tools matching the filter pattern are registered.
5. **Given** the MCP server endpoint is running, **When** an external MCP client calls `tools/list`, **Then** it receives a list of agent tools scoped to the client's namespace view.
6. **Given** an external MCP client calls `tools/call` for an agent tool, **When** the tool-level permission check passes, **Then** the call is routed to the appropriate agent and the result is returned.
7. **Given** multiple MCP servers configured with namespace isolation, **When** tools are registered, **Then** tools from each server are namespaced to avoid collisions (e.g., `filesystem.read_file` vs `github.read_file`).
8. **Given** NATS bridge is enabled, **When** an MCP tool call targets an agent on a different node, **Then** the call is routed over NATS to the owning node and the result is returned to the caller.
9. **Given** an MCP server connection fails, **When** the framework detects the failure, **Then** it reconnects with backoff and the MCP session manager cleans up stale sessions.

---

### Edge Cases

- What happens when a message envelope exceeds the maximum allowed payload size? The transport MUST reject oversized messages with a clear error rather than silently truncating.
- How does the system handle a NATS server that is unreachable at startup? The transport MUST retry connection with exponential backoff and report health as degraded until connected.
- What happens when a subscriber's message processing is slower than the publish rate? The transport MUST apply backpressure to publishers rather than dropping messages.
- How does the system handle deserialization of an envelope with an unknown message type? The transport MUST reject the message with a descriptive error and optionally route it to a dead letter queue.
- What happens when a JetStream stream reaches its storage limit? Messages MUST be handled according to the configured retention policy (discard old or reject new).
- How does the system handle concurrent subscriptions to the same subject from the same process? Each subscription MUST receive messages independently (fan-out) unless grouped in a queue group.
- What happens when a gRPC stream is interrupted mid-transfer? The client MUST receive a connection error and be able to re-establish the stream.
- How does the HTTP transport handle malformed JSON in a POST body? The server MUST return a 400 status with a descriptive error message.
- What happens when a WebSocket client disconnects without a close frame? The server MUST detect the broken connection via ping/pong timeout and clean up resources.
- What happens when an MCP server goes offline while a tool call is in-flight? The framework MUST return a timeout error to the calling agent and mark the MCP server as degraded.
- How does the system handle tool name collisions across multiple MCP servers? Each server's tools MUST be namespaced by server name to prevent collisions.
- What happens when an MCP tool call exceeds the configured timeout? The framework MUST cancel the pending call and return a timeout error without leaking sessions.
- How does the NATS-MCP bridge handle tool calls when the target node is unreachable? The bridge MUST timeout and return an error rather than blocking indefinitely.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST define a `Transport` trait with publish, subscribe, and request-reply operations that all transport implementations conform to. Reply semantics are protocol-specific: received messages carry a reply context (e.g., reply subject in NATS) enabling responders to reply via `publish` to the reply address. The trait does not expose a separate `reply` method because reply mechanisms differ fundamentally across protocols (NATS reply subject, HTTP response, gRPC return value).
- **FR-002**: System MUST wrap all messages in a standardized envelope containing message ID (UUID), timestamp, schema version, message type, optional correlation ID, source agent ID, target agent ID, priority, and serialized payload.
- **FR-003**: System MUST serialize message envelopes using a binary format (MessagePack) for wire transport and support JSON for debugging and HTTP endpoints.
- **FR-004**: System MUST implement a NATS transport that connects to a NATS server, maps agent addresses to subjects, and supports publish/subscribe, request/reply, and queue group patterns.
- **FR-005**: System MUST implement subject-based routing using a hierarchical taxonomy (e.g., `agents.<id>.commands.<type>`, `tasks.<type>.assignment`) for organized message delivery.
- **FR-006**: System MUST support NATS request-reply with configurable timeouts, returning a timeout error if no response is received within the deadline.
- **FR-007**: System MUST handle NATS connection lifecycle including automatic reconnection with configurable retry policy when the server becomes unreachable.
- **FR-008**: System MUST support NATS JetStream for durable message persistence, including stream creation, message publishing, and consumer-based retrieval with explicit acknowledgment.
- **FR-009**: System MUST support JetStream pull and push consumers with configurable delivery policies (all, last, new, by sequence, by time).
- **FR-010**: System MUST redeliver unacknowledged JetStream messages after a configurable timeout.
- **FR-011**: System MUST implement an HTTP transport using a router with REST endpoints for agent listing, health status, task submission, and system configuration.
- **FR-012**: System MUST include HTTP middleware for request ID tracking (adding unique IDs to all requests/responses) and rate limiting.
- **FR-013**: System MUST provide HTTP security middleware hooks that can be populated in Phase 5 without modifying endpoint handlers.
- **FR-014**: System MUST implement a gRPC transport with protobuf service definitions for agent communication, task management, and system health checking.
- **FR-015**: System MUST support gRPC streaming RPCs for continuous data flows (server-streaming for status updates, bidirectional for agent communication).
- **FR-016**: System MUST map gRPC status codes to framework error types bidirectionally.
- **FR-017**: System MUST define transport-level error types covering connection failures, serialization errors, timeout errors, subject routing errors, and protocol-specific errors.
- **FR-018**: System MUST support publish backpressure — when the internal send buffer is full, publish operations await capacity rather than dropping messages.
- **FR-019**: System MUST report transport health status (connected, reconnecting, disconnected) through the existing health monitoring infrastructure.
- **FR-020**: System MUST use `AgentAvailability` semantics (idle/busy/offline) for transport-level status channels, distinct from lifecycle `AgentState`.
- **FR-021**: System MUST define concrete message types as typed structs: TaskAssignment, TaskResult, AgentHeartbeat, SystemEvent, WorkflowStart, StepComplete, WorkflowResult, AgentSpawn, AgentTerminate, and ConfigUpdate.
- **FR-022**: System MUST support WebSocket connections on the HTTP transport for real-time streaming of agent status changes, task events, and system events to external clients.
- **FR-023**: System MUST support WebSocket event filtering, allowing clients to subscribe to specific event categories (agent status, task progress, system events) rather than receiving all events.
- **FR-024**: System MUST send WebSocket keepalive pings to maintain idle connections and detect broken connections via ping/pong timeout.
- **FR-025**: System MUST implement an MCP client that connects to external MCP servers via stdio and streamable-HTTP transports, auto-discovers tools via `tools/list`, and registers them into the agent tool system.
- **FR-026**: System MUST implement an MCP server that exposes agent tools to external MCP clients via a streamable-HTTP endpoint, supporting `tools/list` and `tools/call` operations.
- **FR-027**: System MUST support MCP tool caching — cache `tools/list` responses per server and invalidate on `notifications/tools/list_changed`.
- **FR-028**: System MUST support MCP tool filtering — configurable patterns (e.g., `read_*`) that limit which discovered tools are registered from each external server.
- **FR-029**: System MUST support MCP namespace isolation — tools from each server are prefixed with the server name to prevent collisions across multiple servers.
- **FR-030**: System MUST implement on-demand MCP session management — connections are established lazily on first tool call and cleaned up on shutdown or failure.
- **FR-031**: System MUST implement a two-layer MCP auth model: transport-level auth (OAuth2/API key/mTLS) for connection security, and tool-level permission checks for call authorization.
- **FR-032**: System MUST support namespace-scoped MCP server views — different clients see different tool subsets based on their namespace configuration.
- **FR-033**: System MUST support NATS-MCP bridging — MCP tool calls targeting agents on remote nodes are routed over NATS for distributed tool discovery and execution.
- **FR-034**: System MUST expose agent knowledge bases and configuration snapshots as MCP Resources (read-only) via `resources/list` and `resources/read` handlers. Runtime state (mailbox depth, supervision status) MUST NOT be exposed as MCP Resources.
- **FR-035**: System MUST guarantee per-subject message ordering — messages published to the same subject by the same publisher are delivered to subscribers in publish order. Cross-subject ordering is best-effort and not guaranteed.

### Key Entities

- **MessageEnvelope**: The universal wrapper for all framework messages. Contains routing metadata (message ID, correlation ID, source/target agent, message type, priority), temporal metadata (timestamp, schema version), and the serialized payload. Protocol-agnostic — the same envelope is used regardless of transport.
- **Transport**: The protocol-agnostic communication contract. Defines operations for publishing messages to subjects, subscribing to subjects (with optional queue groups), sending requests with timeout, and replying to requests. Each protocol (NATS, HTTP, gRPC) provides its own implementation.
- **NatsTransport**: The primary transport implementation. Manages a NATS client connection, maps agent addresses to NATS subjects, handles reconnection lifecycle, and provides access to JetStream for durable messaging.
- **JetStreamContext**: Durable messaging layer. Manages streams (creation, configuration, deletion), consumers (pull/push with delivery policies), and message acknowledgment. Provides at-least-once delivery guarantees for critical messages.
- **HttpTransport**: REST API layer. Manages HTTP routes, middleware pipeline (request ID, rate limiting, security hooks), and JSON serialization for external client access.
- **GrpcTransport**: High-performance RPC layer. Manages protobuf service definitions, streaming connections, and gRPC-to-framework error mapping.
- **SubjectTaxonomy**: The hierarchical naming scheme for NATS subjects. Organizes communication channels by category (agents, tasks, system, workflows) with parameterized segments for agent IDs, task types, and command types.
- **AgentAvailability**: Transport-level presence status (idle, busy, offline) distinct from lifecycle AgentState. Used for load balancing and routing decisions across transport channels.
- **McpClient**: Connects to external MCP servers, discovers tools, caches tool lists, and routes tool calls from agents to the external server. Manages session lifecycle with lazy connect and automatic reconnection.
- **McpServer**: Exposes agent tools to external MCP clients via streamable-HTTP. Supports namespace-scoped views, tool filtering, and two-layer auth. Runs as a supervised actor with restart policies.
- **McpSessionManager**: Manages on-demand MCP sessions — creates connections lazily on first tool call, handles reconnection on failure, and cleans up sessions on shutdown.
- **McpNatsBridge**: Routes MCP tool calls across distributed nodes over NATS. Enables federated tool discovery where `tools/list` aggregates tools from all NATS-connected nodes.
- **ResourceRegistry**: Registers agent knowledge bases and configuration as MCP Resources, providing read-only access via `resources/list` and `resources/read` handlers.

## Clarifications

### Session 2026-03-04

- Q: Should WebSocket streaming be included in Phase 4 HTTP transport scope? → A: Yes — include WebSocket streaming for real-time status updates and event streaming to external clients (ROADMAP 4.4 alignment).
- Q: Should MCP integration analysis (spec/mcp_integration_analysis.md) be included in Phase 4 scope? → A: Yes — include full MCP integration: bidirectional (client + server), NATS bridge, namespace isolation, tool caching, on-demand sessions, two-layer auth.
- Q: What message ordering guarantees does the transport layer provide? → A: Per-subject ordering guaranteed — messages on the same subject from the same publisher arrive in order; cross-subject ordering is best-effort.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Agents can exchange messages through the transport layer with correct envelope metadata in under 10 milliseconds for local NATS communication.
- **SC-002**: A message serialized and deserialized through the envelope system preserves all fields exactly, with zero data loss across 10,000 round-trip cycles.
- **SC-003**: The NATS transport successfully reconnects after a server outage within 30 seconds and resumes message delivery without manual intervention.
- **SC-004**: JetStream consumers receive all unacknowledged messages after a restart, achieving at-least-once delivery with zero message loss.
- **SC-005**: The HTTP API responds to health check and agent listing requests in under 50 milliseconds under normal load.
- **SC-006**: The gRPC service correctly maps all framework error types to gRPC status codes and back, with no mapping gaps.
- **SC-007**: Queue group message delivery distributes messages evenly across consumers (within 10% variance) over 1,000 messages.
- **SC-008**: A complete integration test sends a TaskAssignment through NATS and receives a corresponding TaskResult back, validating the full publish-process-reply pipeline.
- **SC-009**: Transport health status accurately reflects connection state changes within 5 seconds of a connectivity event.
- **SC-010**: All transport operations handle backpressure correctly — no messages are dropped when operating at capacity.
- **SC-011**: WebSocket clients receive agent status change events within 1 second of the change occurring.
- **SC-012**: WebSocket event filtering correctly delivers only subscribed event categories, with zero cross-category leakage over 1,000 events.
- **SC-013**: The MCP client successfully discovers and registers tools from an external MCP server, and an agent can invoke a discovered tool and receive a correct result.
- **SC-014**: The MCP server correctly responds to `tools/list` and `tools/call` from an external MCP client, with namespace-scoped views returning only the tools configured for that namespace.
- **SC-015**: MCP tool caching reduces redundant `tools/list` calls — after initial discovery, subsequent tool lookups are served from cache until invalidated.
- **SC-016**: NATS-MCP bridge routes a tool call from Node A to an agent on Node B and returns the result, validating distributed tool execution end-to-end.

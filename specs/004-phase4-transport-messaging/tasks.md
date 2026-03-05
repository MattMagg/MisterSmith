# Tasks: Phase 4 — Transport & Messaging

**Input**: Design documents from `/specs/004-phase4-transport-messaging/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Test tasks are included inline within each user story phase as unit and integration tests.

**Organization**: Tasks are grouped by user story (P1–P6) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- All source code lives under `crates/` in the workspace root
- 5 new crates: `mister-smith-transport`, `mister-smith-nats`, `mister-smith-http`, `mister-smith-grpc`, `mister-smith-mcp`
- Integration tests in `crates/mister-smith-integration-tests/`
- Proto source files in `specs/004-phase4-transport-messaging/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Scaffold all 5 new workspace crates with Cargo.toml, lib.rs stubs, and shared workspace dependency declarations.

- [X] T001 Create crate directory structure for all 5 transport crates per plan.md project structure: `crates/mister-smith-transport/`, `crates/mister-smith-nats/`, `crates/mister-smith-http/`, `crates/mister-smith-grpc/`, `crates/mister-smith-mcp/` — each with `Cargo.toml` and `src/lib.rs` stub
- [X] T002 Add workspace dependencies to root `Cargo.toml`: async-nats 0.46.0, rmp-serde 1.3.1, axum 0.8.8, tonic 0.14, prost 0.14, tonic-build 0.14, tonic-health, rmcp 1.1.0, bytes, uuid, chrono, serde_bytes — and add all 5 new crates to `[workspace.members]`
- [X] T003 Configure proto compilation: copy proto files from `specs/004-phase4-transport-messaging/contracts/*.proto` to `crates/mister-smith-transport/proto/` (4 files: `common.proto`, `agent_service.proto`, `system_service.proto`, `health_service.proto`), create `crates/mister-smith-transport/build.rs` with `tonic_build::compile_protos` for all 4 proto files (common.proto must be in the include path for imports), add `tonic-build` and `prost-build` as build-dependencies

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Cross-cutting types and helpers used by ALL transport implementations. MUST complete before any user story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Implement `TransportError` enum in `crates/mister-smith-transport/src/errors.rs` — variants: ConnectionFailed, SerializationError, DeserializationError, Timeout, SubjectInvalid, PayloadTooLarge, SubscriptionError, PublishError, ProtocolError(String), with `From` conversions to/from `mister-smith-core` error types and `std::fmt::Display`/`thiserror::Error` derives
- [X] T005 [P] Implement `MessagePriority` enum in `crates/mister-smith-transport/src/priority.rs` — variants: Low, Normal, High, Critical with Default=Normal, Ord implementation for priority-based ordering, serde Serialize/Deserialize derives
- [X] T006 [P] Implement `AgentAvailability` enum in `crates/mister-smith-transport/src/availability.rs` — variants: Idle, Busy, Offline with transition validation (Offline→Busy is invalid), serde derives, and unit tests for valid/invalid transitions per data-model.md
- [X] T007 [P] Implement serialization helpers in `crates/mister-smith-transport/src/serialization.rs` — `to_msgpack<T: Serialize>(val: &T) -> Result<Vec<u8>>` using `rmp_serde::to_vec_named`, `from_msgpack<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>`, `to_json<T: Serialize>(val: &T) -> Result<String>`, `from_json<T: DeserializeOwned>(s: &str) -> Result<T>` — with unit tests for round-trip correctness and `#[serde(with = "serde_bytes")]` annotation guidance
- [X] T008 Wire up `crates/mister-smith-transport/src/lib.rs` with public re-exports of errors, priority, availability, and serialization modules; verify `cargo build -p mister-smith-transport` compiles

**Checkpoint**: Foundation ready — all transport crates have scaffolding, shared types compile, serialization works.

---

## Phase 3: User Story 1 — Transport Abstraction & Message Envelope (Priority: P1) 🎯 MVP

**Goal**: Define the protocol-agnostic `Transport` trait and `MessageEnvelope` so all transport implementations share a unified message format and communication contract.

**Independent Test**: Build and test entirely with `InMemoryTransport` — no external dependencies. Verify envelope construction, serialization round-trips (10,000 cycles), and Transport trait compliance.

### Implementation for User Story 1

- [X] T009 [US1] Implement `MessageEnvelope` struct and builder in `crates/mister-smith-transport/src/envelope.rs` — fields per data-model.md (message_id UUID, timestamp, schema_version, message_type, correlation_id, trace_id, source_agent_id, target_agent_id, priority, payload Bytes, headers HashMap), builder pattern with `payload_msgpack<T>()` and `payload_json<T>()` methods, validation (non-empty message_type, payload size ≤ max), `Serialize`/`Deserialize` derives, and unit tests for: builder construction, validation rejection of oversized payloads, serde round-trip
- [X] T010 [US1] Define `Transport` trait, `Subscription`, and `ReceivedMessage` types in `crates/mister-smith-transport/src/transport.rs` — async trait with `publish`, `subscribe`, `queue_subscribe`, `request` methods per design decision D1; `ReceivedMessage` struct containing `MessageEnvelope` + `reply_subject: Option<String>` (populated by NATS from incoming message reply field, None for HTTP/gRPC); `Subscription` wrapping `Pin<Box<dyn Stream<Item = ReceivedMessage> + Send>>` with `next()` method; reply is performed via `transport.publish(received.reply_subject.unwrap(), response)` — no separate `reply` method on the trait
- [X] T011 [P] [US1] Implement `SubjectTaxonomy` in `crates/mister-smith-transport/src/subject.rs` — builder methods for all 14 subject patterns from data-model.md (`agent_command(id, type)`, `agent_status(id)`, `task_assignment(type)`, `task_result(id)`, `workflow_start(id)`, `system_event(type)`, etc.), validation that segments are non-empty and contain no wildcards (except in wildcard-specific constructors), and unit tests for all patterns
- [X] T012 [P] [US1] Implement typed message structs in `crates/mister-smith-transport/src/messages.rs` — all 10 message types from data-model.md: `TaskAssignment`, `TaskResult`, `AgentHeartbeat`, `SystemEvent`, `WorkflowStart`, `StepComplete`, `WorkflowResult`, `AgentSpawn`, `AgentTerminate`, `ConfigUpdate` — each with serde derives, and unit tests for MessagePack serialization round-trip of each type
- [X] T013 [US1] Implement `InMemoryTransport` in `crates/mister-smith-transport/src/inmemory.rs` — full `Transport` trait impl backed by `tokio::sync::broadcast` channels per design decision D3, supporting publish/subscribe/queue_subscribe (round-robin delivery)/request-reply (with timeout), and unit tests: publish-subscribe delivery, queue group single-delivery, request-reply with correlation ID, request timeout error
- [X] T014 [US1] Write comprehensive envelope serialization tests in `crates/mister-smith-transport/src/envelope.rs` (or `tests/` module) — 10,000 MessagePack round-trip cycles with zero data loss (SC-002), JSON serialization for HTTP compatibility, envelope with all optional fields set, envelope with minimal fields
- [X] T015 [US1] Update `crates/mister-smith-transport/src/lib.rs` re-exports to include envelope, transport, subject, messages, inmemory modules; verify `cargo test -p mister-smith-transport` passes all tests

**Checkpoint**: `mister-smith-transport` crate is self-contained and fully testable. Transport trait, envelope, subjects, and 10 message types work with InMemoryTransport. This is the MVP.

---

## Phase 4: User Story 2 — NATS Inter-Agent Communication (Priority: P2)

**Goal**: Implement the primary inter-agent transport over NATS with pub/sub, request-reply, queue groups, backpressure, and automatic reconnection.

**Independent Test**: Requires running NATS Docker container (`docker start NATS`). Test pub/sub delivery, queue group load balancing, request-reply with timeout, and reconnection recovery.

**Depends on**: US1 (Transport trait, MessageEnvelope)

### Implementation for User Story 2

- [X] T016 [P] [US2] Implement `NatsTransportConfig` and `JetStreamConfig` in `crates/mister-smith-nats/src/config.rs` — all fields from data-model.md NatsTransportConfig (server_urls, name, max_reconnects, connection_timeout, request_timeout, client_capacity, subscription_capacity, TLS fields) and JetStreamConfig (enabled, domain, max_ack_inflight, ack_timeout), with `Default` impl, serde derives, and `RuntimeConfigExt` integration following Phase 2 patterns
- [X] T017 [P] [US2] Implement NATS-specific error types in `crates/mister-smith-nats/src/errors.rs` — `NatsError` enum wrapping `async_nats` errors with variants for connection, subscription, publish, JetStream, and subject routing errors; bidirectional `From` conversions to `TransportError`
- [X] T018 [P] [US2] Implement subject routing helpers in `crates/mister-smith-nats/src/subjects.rs` — functions to map `SubjectTaxonomy` patterns to NATS subject strings, validate NATS subject constraints (no spaces, max length), and wildcard subject construction for subscriptions (`agents.>`, `tasks.*.assignment`)
- [X] T019 [US2] Implement `NatsTransport` struct in `crates/mister-smith-nats/src/client.rs` — `connect(config)` constructor using `async_nats::ConnectOptions` with event_callback for connection lifecycle (Connected/Disconnected/LameDuckMode/SlowConsumer per R1), `Transport` trait impl with: `publish` (async with backpressure via bounded channel), `subscribe` (wrapping async-nats `Subscriber` stream into `Subscription`), `queue_subscribe` (NATS queue groups), `request` (with configurable timeout, correlation ID matching per FR-006); connection state tracking via `client.connection_state()`
- [X] T020 [US2] Implement NATS `HealthCheck` in `crates/mister-smith-nats/src/health.rs` — impl `mister_smith_monitoring::HealthCheck` for `NatsTransport`, reporting Healthy when Connected, Degraded when reconnecting, Unhealthy when Disconnected (FR-019, SC-009)
- [X] T021 [US2] Wire up `crates/mister-smith-nats/src/lib.rs` with public re-exports; verify `cargo build -p mister-smith-nats` compiles
- [X] T022 [US2] Write NATS integration tests in `crates/mister-smith-nats/tests/` (gated behind `#[cfg(feature = "integration")]` or requires running NATS) — tests: pub/sub message delivery with correct envelope, queue group single-delivery across 3 subscribers (SC-007: ±10% variance over 1,000 messages), request-reply with correlation ID, request timeout error, backpressure behavior when buffer full (SC-010), connection state after disconnect/reconnect (SC-003: within 30 seconds)

**Checkpoint**: Agents can communicate over NATS with pub/sub, queue groups, and request-reply. Connection lifecycle is monitored through HealthCheck.

---

## Phase 5: User Story 3 — Durable Messaging via JetStream (Priority: P3)

**Goal**: Add JetStream durable messaging for at-least-once delivery with stream management, consumer configuration, explicit acknowledgment, and message redelivery.

**Independent Test**: Publish messages to a JetStream stream, stop and restart a consumer, verify all unacknowledged messages are redelivered. Requires NATS Docker with JetStream enabled.

**Depends on**: US2 (NatsTransport with NATS client connection)

### Implementation for User Story 3

- [X] T023 [US3] Implement JetStream stream and consumer management in `crates/mister-smith-nats/src/jetstream.rs` — `JetStreamManager` struct wrapping `async_nats::jetstream::Context` (obtained via `jetstream::new(client)`), methods for: `create_stream(name, subjects, retention_policy)`, `delete_stream(name)`, `create_pull_consumer(stream, config)`, `create_push_consumer(stream, config)` with configurable delivery policies (All, Last, New, BySequence, ByTime per FR-009), `publish(subject, envelope)` returning `PublishAckFuture` (double-await pattern per R1)
- [X] T024 [US3] Implement durable consumer message processing in `crates/mister-smith-nats/src/jetstream.rs` — pull consumer `messages()` returning a stream, explicit `ack()`, `nak()`, `term()` methods on received messages (FR-008), configurable ack timeout with automatic redelivery (FR-010), and max delivery count configuration
- [X] T025 [US3] Implement JetStream retention and limits in `crates/mister-smith-nats/src/jetstream.rs` — stream config for max_messages, max_bytes, max_age with retention policies (Limits, Interest, WorkQueue), and handling when limits are exceeded (discard old vs reject new)
- [X] T026 [US3] Write JetStream integration tests in `crates/mister-smith-nats/tests/` — tests: publish to stream and verify persistence (SC-004), consumer disconnect/reconnect receives unacked messages, acknowledged messages not redelivered, ack timeout triggers redelivery (FR-010), retention policy purges old messages when limits exceeded

**Checkpoint**: Critical messages survive agent restarts. At-least-once delivery is validated with explicit ack and redelivery.

---

## Phase 6: User Story 4 — HTTP API & WebSocket Endpoints (Priority: P4)

**Goal**: Expose REST endpoints for system management and WebSocket connections for real-time event streaming to external clients.

**Independent Test**: Start HTTP server, send requests to all REST endpoints and verify JSON responses. Open WebSocket, trigger events, and verify streaming delivery with filtering.

**Depends on**: US1 (MessageEnvelope, typed messages)

### Implementation for User Story 4

- [X] T027 [P] [US4] Implement `HttpTransportConfig` in `crates/mister-smith-http/src/config.rs` — fields from data-model.md (bind_address, websocket_enabled, ws_keepalive_interval, max_ws_connections, rate_limit_rps), `Default` impl, serde derives
- [X] T028 [P] [US4] Implement HTTP error mapping in `crates/mister-smith-http/src/errors.rs` — `HttpError` enum with variants for NotFound, BadRequest, RateLimited, InternalError; impl `IntoResponse` for Axum to produce consistent JSON error format per http-api.md contract (`{ "error": "code", "message": "...", "request_id": "uuid" }`)
- [X] T029 [US4] Implement middleware in `crates/mister-smith-http/src/middleware.rs` — request ID tracking (generate UUID, set `X-Request-Id` header, preserve client-provided ID per FR-012), rate limiting (per-IP, configurable RPS, return 429 with `Retry-After` per FR-012), security middleware hooks (pass-through in Phase 4, placeholder for Phase 5 auth enforcement per FR-013)
- [X] T030 [US4] Implement REST route definitions and handlers in `crates/mister-smith-http/src/routes.rs` and `crates/mister-smith-http/src/handlers.rs` — endpoints per http-api.md contract: `GET /api/v1/health` (system health with component details), `GET /api/v1/agents` (list with optional availability/type filters), `GET /api/v1/agents/{agent_id}` (single agent, 404 if not found), `POST /api/v1/tasks` (submit task, return 202 with task_id), `GET /api/v1/tasks/{task_id}` (status and result), `GET /api/v1/config` (system config with optional component filter); use Axum 0.8 `{param}` path syntax (NOT `:param`)
- [X] T031 [US4] Implement WebSocket endpoint in `crates/mister-smith-http/src/websocket.rs` — `GET /api/v1/events/ws` with Axum `WebSocketUpgrade` extractor using `any()` routing per R4, event filtering via query param `?filter=agent_status,task_progress` (FR-023), broadcast channel internally (design decision D7), keepalive pings every 30s (FR-024) with 10s pong timeout, client subscribe/unsubscribe messages, `Message::Text(Utf8Bytes)` for events (Axum 0.8 API), connection cleanup on disconnect
- [X] T032 [US4] Implement `HttpTransport` server lifecycle in `crates/mister-smith-http/src/server.rs` — `start(config, app_state)` composing router with all routes + middleware + WebSocket, graceful shutdown integration, `AppState` shared state type holding references to agent registry and event broadcast channel
- [X] T033 [US4] Wire up `crates/mister-smith-http/src/lib.rs` with re-exports; write unit tests for: middleware request ID generation, rate limiting (verify 429 after exceeding limit), JSON error format, handler response shapes; verify `cargo test -p mister-smith-http` passes

**Checkpoint**: External clients can query agents, submit tasks, and check health via REST. WebSocket provides real-time event streaming with filtering and keepalive (SC-011, SC-012).

---

## Phase 7: User Story 5 — gRPC Service Layer (Priority: P5)

**Goal**: Implement gRPC services for type-safe, streaming inter-service communication with Kubernetes-compatible health checking.

**Independent Test**: Start gRPC server, call service methods with a gRPC client (tonic), verify protobuf responses and error mapping.

**Depends on**: US1 (Transport trait, proto types compiled in build.rs)

### Implementation for User Story 5

- [X] T034 [P] [US5] Implement `GrpcTransportConfig` in `crates/mister-smith-grpc/src/config.rs` — fields from data-model.md (bind_address, max_message_size), `Default` impl, serde derives
- [X] T035 [P] [US5] Implement gRPC status ↔ framework error mapping in `crates/mister-smith-grpc/src/errors.rs` — bidirectional mapping: `TransportError::Timeout` ↔ `Code::DeadlineExceeded`, `TransportError::ConnectionFailed` ↔ `Code::Unavailable`, `TransportError::SubjectInvalid` ↔ `Code::InvalidArgument`, `TransportError::PayloadTooLarge` ↔ `Code::ResourceExhausted`, etc. (FR-016, SC-006: no mapping gaps); unit tests for all mappings in both directions
- [X] T036 [US5] Implement `AgentService` gRPC server in `crates/mister-smith-grpc/src/agent_service.rs` — impl the generated `agent_service_server::AgentService` trait from proto: `list_agents`, `get_agent`, `submit_task`, `get_task_result`, `stream_agent_status` (server-streaming), `agent_channel` (bidirectional streaming per FR-015); use generated proto types from `crates/mister-smith-transport/proto/`
- [X] T037 [US5] Implement `SystemService` gRPC server in `crates/mister-smith-grpc/src/system_service.rs` — impl the generated `system_service_server::SystemService` trait: `stream_events` (server-streaming with severity filter), `get_config`, `update_config`, `get_metrics`
- [X] T038 [US5] Implement health service and server lifecycle in `crates/mister-smith-grpc/src/health.rs` and `crates/mister-smith-grpc/src/server.rs` — use `tonic-health` crate for standard gRPC health checking protocol (K8s probe compatible), compose Tonic server with AgentService + SystemService + Health, graceful shutdown, max message size from config
- [X] T039 [US5] Wire up `crates/mister-smith-grpc/src/lib.rs` with re-exports; write unit tests for error mapping completeness; verify `cargo build -p mister-smith-grpc` compiles with proto codegen

**Checkpoint**: gRPC services accept requests with protobuf encoding, stream status updates, and report health compatible with Kubernetes probes (SC-006).

---

## Phase 8: User Story 6 — MCP Integration (Priority: P6)

**Goal**: Connect agents to external tools via MCP as both client and server, with tool caching, namespace isolation, NATS bridging, and resource exposure.

**Independent Test**: MCP client connects to a mock MCP server (stdio), discovers tools, invokes them. MCP server responds to `tools/list` and `tools/call` from an external MCP client.

**Depends on**: US1 (messages, envelope), US2 (NATS transport — for bridge only)

### Implementation for User Story 6

- [X] T040 [P] [US6] Implement `McpConfig` in `crates/mister-smith-mcp/src/config.rs` — fields from data-model.md (enabled, clients vec, servers vec, nats_bridge_enabled, nats_bridge_prefix), `McpClientConfig` (name, transport type, command/url, tool_filter, namespace), `McpServerConfig` (bind_address, namespace_views), serde derives
- [X] T041 [P] [US6] Implement MCP error wrapping in `crates/mister-smith-mcp/src/errors.rs` — `McpError` enum using `thiserror 1.x` wrapping rmcp errors at crate boundary per design decision D5, variants: ConnectionFailed, ToolNotFound, ToolCallFailed, SessionError, BridgeTimeout, NamespaceConflict; `From<rmcp::Error>` impl to isolate thiserror ^2 dependency
- [X] T042 [US6] Implement `McpClient` in `crates/mister-smith-mcp/src/client.rs` — connect to external MCP servers via stdio and streamable-HTTP transports using rmcp (feature flags per R2), `tools/list` discovery with tool caching (FR-027), `notifications/tools/list_changed` invalidation, tool filtering by configurable patterns (FR-028), namespace prefixing per server (FR-029, e.g., `filesystem.read_file`), `tools/call` routing with timeout, and unit tests for cache hit/miss and namespace prefixing
- [X] T043 [US6] Implement `McpSessionManager` in `crates/mister-smith-mcp/src/session.rs` — on-demand session management: lazy connect on first tool call (FR-030), reconnection with backoff on failure, session cleanup on shutdown, session health tracking, and unit tests for lazy initialization and cleanup
- [X] T044 [US6] Implement `McpServer` in `crates/mister-smith-mcp/src/server.rs` — expose agent tools to external MCP clients via streamable-HTTP endpoint using rmcp server features, `tools/list` handler returning namespace-scoped views (FR-032), `tools/call` handler with tool-level permission checks (FR-031), and namespace filtering per client connection
- [X] T045 [US6] Implement `McpNatsBridge` in `crates/mister-smith-mcp/src/bridge.rs` — route MCP tool calls to agents on remote nodes over NATS (FR-033), distributed `tools/list` aggregation across NATS-connected nodes, configurable NATS subject prefix (default `ms.mcp`), timeout handling for unreachable nodes
- [X] T046 [US6] Implement `ResourceRegistry` in `crates/mister-smith-mcp/src/resources.rs` — register agent knowledge bases and config snapshots as MCP Resources (FR-034), `resources/list` and `resources/read` handlers (read-only), exclude runtime state (mailbox depth, supervision status); wire up `crates/mister-smith-mcp/src/lib.rs` with all re-exports; verify `cargo build -p mister-smith-mcp` compiles

**Checkpoint**: Agents consume external MCP tools and expose their own tools. Tool caching, namespace isolation, and NATS bridge work end-to-end (SC-013, SC-014, SC-015, SC-016).

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, supervision integration, Gate 4 validation, and workspace-wide quality checks.

- [X] T047 End-to-end integration test in `crates/mister-smith-integration-tests/`: send a `TaskAssignment` through NATS, have a worker process it, receive `TaskResult` back — validating the full publish-process-reply pipeline (SC-008, Gate 4 critical path)
- [X] T048 Transport supervision integration: register `NatsTransport` as a Permanent supervised actor, `HttpTransport` and `GrpcTransport` as Permanent actors, `McpClient` instances as Transient actors within the Phase 3 supervision tree (design decision D6, CHK046-049); emit transport lifecycle events to EventBus
- [X] T049 [P] Cross-transport integration tests in `crates/mister-smith-integration-tests/`: HTTP health endpoint reflects NATS connection state, WebSocket receives events triggered by NATS messages, gRPC health service reports correct serving status based on transport health
- [X] T050 [P] Run `cargo clippy --workspace -- -D warnings` across all new crates; fix all warnings
- [X] T051 [P] Add rustdoc documentation comments to all public types and trait methods across the 5 new crates; verify `cargo doc --workspace --no-deps` builds without warnings
- [X] T052 Validate quickstart.md scenarios: confirm each of the 7 integration scenarios from `specs/004-phase4-transport-messaging/quickstart.md` can be adapted into passing tests; run full `cargo test --workspace` and verify all tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — BLOCKS US2, US3, US4, US5, US6
- **US2 (Phase 4)**: Depends on US1 — BLOCKS US3
- **US3 (Phase 5)**: Depends on US2 (JetStream extends NATS client)
- **US4 (Phase 6)**: Depends on US1 only — can run in PARALLEL with US2/US3
- **US5 (Phase 7)**: Depends on US1 only — can run in PARALLEL with US2/US3/US4
- **US6 (Phase 8)**: Depends on US1 (core), US2 (NATS bridge in T045 only)
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (US1: Transport Abstraction)
                                                │
                                  ┌──────────────┼──────────────┐
                                  ▼              ▼              ▼
                            Phase 4 (US2)   Phase 6 (US4)  Phase 7 (US5)
                                  │          HTTP+WS          gRPC
                                  ▼
                            Phase 5 (US3)
                             JetStream
                                  │
                                  ▼
                            Phase 8 (US6)  ◄─── also needs US1
                               MCP
                                  │
                                  ▼
                            Phase 9 (Polish)
```

### Within Each User Story

- Config types and error types (parallelizable) before core implementation
- Core struct implementation before integration/tests
- Integration/health checks after core works
- lib.rs re-exports after all modules exist

### Parallel Opportunities

- **Phase 2**: T004, T005, T006, T007 are all independent files — run in parallel
- **Phase 3**: T011 (subjects) and T012 (messages) can run in parallel after T009 (envelope)
- **Phase 4**: T016, T017, T018 can run in parallel (config, errors, subjects)
- **Phase 6, 7**: US4 (HTTP) and US5 (gRPC) can run in parallel after US1 completes
- **Phase 8**: T040, T041 can run in parallel (config, errors)
- **Phase 9**: T049, T050, T051 can run in parallel

---

## Parallel Example: After US1 Completes

```
# These three user stories can launch in parallel (different crates, no shared state):
Agent A: US2 (mister-smith-nats) — NATS transport implementation
Agent B: US4 (mister-smith-http) — HTTP + WebSocket implementation
Agent C: US5 (mister-smith-grpc) — gRPC service implementation

# Within US2, these tasks can run in parallel:
Task: T016 "NatsTransportConfig in crates/mister-smith-nats/src/config.rs"
Task: T017 "NatsError types in crates/mister-smith-nats/src/errors.rs"
Task: T018 "Subject routing in crates/mister-smith-nats/src/subjects.rs"
```

---

## Implementation Strategy

### MVP First (US1 Only — Transport Abstraction)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T008)
3. Complete Phase 3: US1 Transport Abstraction (T009–T015)
4. **STOP and VALIDATE**: `cargo test -p mister-smith-transport` — all envelope round-trips pass, InMemoryTransport satisfies Transport trait, all message types serialize/deserialize correctly
5. This is a fully functional, testable increment

### Critical Path (NATS — The Core Differentiator)

1. MVP (above) → US2 (NATS transport) → US3 (JetStream durability)
2. **VALIDATE**: End-to-end TaskAssignment → TaskResult through NATS (Gate 4 critical test)
3. This establishes the real-time pub/sub backbone — the framework's differentiator

### Full Delivery (All Transports)

1. Critical Path (above)
2. US4 (HTTP + WebSocket) — external access and real-time dashboards
3. US5 (gRPC) — service mesh support
4. US6 (MCP) — external tool integration
5. Phase 9 (Polish) — integration tests, supervision, Gate 4 full validation

### Incremental Delivery

Each completed user story adds standalone value:
- **US1**: Framework has a transport abstraction testable without external deps
- **US2**: Agents can communicate over NATS in real-time
- **US3**: Messages survive restarts — production reliability
- **US4**: External clients can manage and monitor the system
- **US5**: Service mesh deployments with high-performance gRPC
- **US6**: Agents connect to external tools via MCP ecosystem

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- NATS integration tests require Docker: `docker start NATS` (ports 4222, 8222)
- Proto files live in `specs/004-phase4-transport-messaging/contracts/` — copied to crate at setup
- rmcp (thiserror ^2 conflict) is isolated in `mister-smith-mcp` — no leakage to other crates
- Axum 0.8: use `{param}` path syntax, `any()` for WebSocket, `Message::Text(Utf8Bytes)` not String
- async-nats 0.46: publish is async (backpressure), `PublishAckFuture` needs double-await for JetStream
- Commit after each task or logical group; run `cargo clippy` regularly

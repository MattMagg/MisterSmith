# Phase 4 Research: Transport & Messaging

## R1: async-nats 0.46.0 API

**Decision**: Use async-nats 0.46.0 with features: jetstream, kv, service, ring (default feature set).

**Rationale**: Mandated by constitution. Local source reference at `nats.rs/async-nats/` confirms API compatibility.

**Key API patterns**:
- `Client` is cheap to clone (Arc internals). All operations go through bounded mpsc channel to a single `ConnectionHandler` task.
- Publish is async with backpressure via bounded channel (default capacity 2048, configurable via `ConnectOptions::client_capacity`).
- `Subscriber` implements `futures::Stream<Item = Message>` — use `StreamExt::next()`.
- Queue subscribe: `client.queue_subscribe(subject, queue_group)`.
- Request-reply: `client.request(subject, payload)` with default 10s timeout. Custom timeout via `Request::timeout()`.
- Connection events via `ConnectOptions::event_callback(|Event| async { ... })`. Events: Connected, Disconnected, LameDuckMode, SlowConsumer(sid).
- Connection state: `client.connection_state()` returns `State::Pending | Connected | Disconnected`.
- JetStream: `jetstream::new(client)` returns `Context`. Publish returns `PublishAckFuture` (double-await pattern). Pull consumers via `consumer.messages().await?` returning `Stream`. Message ack via `message.ack().await`.
- Max payload: 1MB from server info. Publish validates before sending.

**Alternatives considered**: None — async-nats is the only maintained Rust NATS client.

## R2: MCP SDK Selection

**Decision**: Use `rmcp` 1.1.0 (official MCP SDK under `modelcontextprotocol` org).

**Rationale**: Official SDK with 4.5M downloads, just hit 1.0 (2026-03-03). Supports client + server, stdio + streamable-HTTP, tools/resources/prompts, Tower integration. Active development (407 commits, 3095 GitHub stars).

**Dependency conflict**: `rmcp` depends on `thiserror ^2`. Our workspace uses `thiserror 1.x` per constitution. Resolution: Cargo supports both versions in the same dependency tree. `mister-smith-mcp` depends on rmcp (bringing thiserror 2 transitively); all other crates continue with thiserror 1. rmcp error types are wrapped in our own types at the crate boundary — no thiserror 2 leakage.

**Feature flags for Phase A (client)**: `client`, `transport-child-process`, `transport-streamable-http-client-reqwest`
**Feature flags for Phase B (server)**: `server`, `transport-io`, `transport-streamable-http-server`

**Alternatives considered**: `rust-mcp-sdk` (community, 84K downloads, pre-1.0, "use at own risk" disclaimer). Rejected — insufficient maturity and not official.

## R3: MessagePack Serialization

**Decision**: Use `rmp-serde` 1.3.1 with `to_vec_named` for wire format.

**Rationale**: Constitution mandates MessagePack for wire format. `rmp-serde` is the standard Rust MessagePack serde adapter (63M+ downloads). Named mode (`to_vec_named`) encodes structs as maps with field name keys — tolerant of field reordering and (with `#[serde(default)]`) field addition. This enables envelope schema evolution without breaking wire compatibility.

**Key considerations**:
- Use `to_vec_named` (NOT `to_vec`) — positional encoding breaks on field additions.
- Deserializer accepts both named and positional formats transparently.
- Use `#[serde(with = "serde_bytes")]` on `Vec<u8>` fields to encode as binary, not integer arrays.
- 2x faster than serde_json, 57% smaller output.
- Default depth limit: 1024 nested levels.

**Alternatives considered**: serde_json (larger, slower, but human-readable — kept for debugging and HTTP endpoints), protobuf (rejected for envelope — too rigid for dynamic payloads, used only for gRPC service definitions).

## R4: Axum 0.8 HTTP Framework

**Decision**: Use Axum 0.8.8 for HTTP transport and WebSocket support.

**Rationale**: Constitution mandates Axum 0.8.x. Current version 0.8.8. MSRV 1.75 (compatible with our 1.88.0).

**Key API changes from 0.7**:
- Path params: `{id}` syntax (NOT `:id`). Old syntax panics at runtime.
- WebSocket: Use `any()` routing (not `get()`) for HTTP/1.1 + HTTP/2 support.
- `Message::Text(Utf8Bytes)` not `String`; `Message::Binary(Bytes)` not `Vec<u8>`.
- `WebSocket::close()` removed — send `Message::Close` explicitly.
- `#[async_trait]` removed from extractors — native async fn in traits.
- All handlers must be `Sync`.

**WebSocket API**: `WebSocketUpgrade` extractor → `on_upgrade(handle_socket)` → `socket.recv()` / `socket.send()`. Split via `StreamExt::split()` for concurrent read/write.

**Middleware**: `middleware::from_fn(handler)` for simple middleware, `middleware::from_fn_with_state(state, handler)` for stateful. Tower layers via `ServiceBuilder`.

**Alternatives considered**: None — Axum is mandated by constitution.

## R5: Tonic 0.14 gRPC

**Decision**: Use Tonic 0.14.x with prost 0.14.x for protobuf.

**Rationale**: Constitution mandates Tonic 0.14.x. Standard Rust gRPC framework, Tokio-native.

**Key patterns**: Service definitions via `.proto` files compiled by `tonic-build`. Streaming RPCs via `tonic::Streaming<T>`. Error mapping: `tonic::Status` ↔ framework errors. Health checking via `tonic-health` crate (gRPC health check protocol).

**Alternatives considered**: None — Tonic is mandated by constitution.

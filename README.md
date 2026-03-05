# Mister Smith

A multi-agent orchestration framework built in Rust with NATS messaging and Erlang-inspired supervision trees. Model-agnostic — works with any LLM backend.

## Implementation Status

| Phase | Status | Crates | Tests |
|-------|--------|--------|-------|
| 1. Foundation | Complete | `mister-smith-core`, `mister-smith-config` | 60 |
| 2. Runtime & Async | Complete | `mister-smith-runtime`, `mister-smith-monitoring`, `mister-smith-events`, `mister-smith-async`, `mister-smith-resources` | 243 |
| 3. Actor & Supervision | Complete | `mister-smith-actor`, `mister-smith-supervision` | 389 |
| 4. Transport & Messaging | Complete | `mister-smith-transport`, `mister-smith-nats`, `mister-smith-http`, `mister-smith-grpc`, `mister-smith-mcp` | 605 |
| 5. Security | In progress | — | — |
| 6. Persistence & State | Not started | — | — |
| 7. Agent System | Not started | — | — |
| 8. Operations | Not started | — | — |

**14 crates** in the workspace, **605 tests** passing, zero clippy warnings.

## Architecture

Mister Smith coordinates distributed AI agents through three core subsystems:

**Supervision Trees** — Hierarchical fault tolerance inspired by Erlang/OTP. Supervisors manage agent lifecycles with configurable restart strategies (OneForOne, OneForAll, RestForOne), failure escalation, and circuit breakers.

**NATS Messaging** — High-performance pub/sub communication layer using NATS and JetStream. Supports request-response, publish-subscribe, queue groups, and subject-based routing (`agent.<type>.<id>.<action>`).

**Agent Orchestration** — Nine specialized agent roles (Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory) with dynamic team composition based on task requirements.

## Quick Start

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
```

**Rust MSRV**: 1.88.0 (driven by async-nats 0.46.0)

## Workspace Crates

```
mister-smith-core          Foundation types, traits, error hierarchy
├── mister-smith-config    Typed TOML config with env overlay
├── mister-smith-runtime   Tokio RuntimeManager, scheduling
├── mister-smith-monitoring  HealthMonitor, phi accrual failure detector, metrics
├── mister-smith-events    EventBus pub/sub, dead letter queue
├── mister-smith-async     TaskExecutor, CircuitBreaker, RetryPolicy, streams
├── mister-smith-resources ConnectionPool, pool sizing, health reports
├── mister-smith-actor     ActorCell, ActorRef, lifecycle, mailbox
├── mister-smith-supervision  SupervisedSystem, restart strategies, health checks
├── mister-smith-transport MessageEnvelope, Transport trait, serialization, InMemoryTransport
├── mister-smith-nats      NATS pub/sub, request-reply, JetStream
├── mister-smith-http      Axum REST API, WebSocket, middleware, rate limiting
├── mister-smith-grpc      Tonic gRPC services, health, protobuf
├── mister-smith-mcp       MCP client/server, tool registry, NATS bridge
└── mister-smith-integration-tests  Cross-crate validation
```

## Technology Stack

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 1.88.0 (MSRV) | Language |
| Tokio | 1.49.0 | Async runtime |
| async-nats | 0.46.0 | Agent messaging (JetStream, KV, service) |
| Axum | 0.8.8 | HTTP/WebSocket transport |
| Tonic | 0.14 | gRPC transport |
| Prost | 0.14 | Protobuf serialization |
| serde | 1.0 | JSON/MessagePack serialization |
| thiserror | 1.0 | Error hierarchy |
| tracing | 0.1 | Structured logging |

## Documentation

| Area | Entry Point |
|------|------------|
| Build roadmap | [`ROADMAP.md`](ROADMAP.md) |
| System architecture | [`spec/core-architecture/system-architecture.md`](spec/core-architecture/system-architecture.md) |
| Agent types | [`spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) |
| Agent orchestration | [`spec/data-management/agent-orchestration.md`](spec/data-management/agent-orchestration.md) |
| Message schemas | [`spec/data-management/message-schemas.md`](spec/data-management/message-schemas.md) |
| Transport layer | [`spec/transport/transport-layer-specifications.md`](spec/transport/transport-layer-specifications.md) |
| Security framework | [`spec/security/security-framework.md`](spec/security/security-framework.md) |
| Version baseline | [`VERSION_REFERENCE.md`](VERSION_REFERENCE.md) |

## Design Principles

- **Fail-fast with graceful recovery** — Detect failures quickly, recover through supervision trees
- **Event-driven architecture** — Loose coupling via pub/sub messaging
- **Resource-bounded execution** — Memory-bounded contexts, connection pooling, backpressure handling
- **Model-agnostic** — No LLM vendor lock-in; works with any backend

## License

MIT OR Apache-2.0

# Mister Smith

Multi-agent orchestration framework — Rust + NATS + supervision trees. Model-agnostic (works with any LLM).

## Commands

```bash
cargo build --workspace                    # Build all crates
cargo test --workspace                     # Run all tests (950 as of Phase 7 complete)
cargo clippy --workspace -- -D warnings    # Lint (must pass clean)
```

## Implementation Status

| Phase | Status | Crates |
|-------|--------|--------|
| 1. Foundation | Complete | `mister-smith-core`, `mister-smith-config` |
| 2. Runtime & Async | Complete | `mister-smith-runtime`, `mister-smith-monitoring`, `mister-smith-events`, `mister-smith-async`, `mister-smith-resources`, `mister-smith-integration-tests` |
| 3. Actor System & Supervision | Complete | `mister-smith-actor`, `mister-smith-supervision` |
| 4. Transport & Messaging | Complete | `mister-smith-transport`, `mister-smith-nats`, `mister-smith-http`, `mister-smith-grpc`, `mister-smith-mcp` |
| 5. Security | Complete | `mister-smith-security` |
| 6. Persistence & State | Complete | `mister-smith-persistence` |
| 7. Agent System | Complete | `mister-smith-agents` |
| 8 | Not started | See `ROADMAP.md` |

## Workspace Crate Dependencies

```
mister-smith-core (foundation types, traits, errors)
├── mister-smith-config (typed TOML config, env overlay)
├── mister-smith-runtime (Tokio RuntimeManager, scheduling)
├── mister-smith-monitoring (HealthMonitor, phi accrual failure detector, metrics)
├── mister-smith-events (EventBus pub/sub, dead letter queue)
├── mister-smith-async (TaskExecutor, CircuitBreaker, RetryPolicy, stream processing)
├── mister-smith-resources (ConnectionPool, pool sizing, health reports)
├── mister-smith-actor (ActorCell, ActorRef, lifecycle management, mailbox)
├── mister-smith-supervision (SupervisedSystem, restart strategies, health checks)
├── mister-smith-transport (MessageEnvelope, Transport trait, serialization, InMemoryTransport)
├── mister-smith-nats (NATS pub/sub, request-reply, JetStream, health checks)
├── mister-smith-http (Axum REST API, WebSocket, middleware, rate limiting)
├── mister-smith-grpc (Tonic gRPC services, health, protobuf)
├── mister-smith-mcp (MCP client/server, tool registry, NATS bridge)
├── mister-smith-security (JWT auth, RBAC, TLS/mTLS, audit logging)
├── mister-smith-persistence (PostgreSQL + JetStream KV dual-store, repositories, audit bridge)
├── mister-smith-agents (AgentRuntime, registry, scheduler, orchestrator, team, tool bus, 9 roles)
└── mister-smith-integration-tests (cross-crate validation)
```

## Repository Structure

| Directory | Contents |
|-----------|----------|
| `crates/` | Rust workspace — 18 crates (Phase 1-7: foundation, runtime/async, actor/supervision, transport, security, persistence, agents) |
| `spec/` | Canonical architecture specifications — 65+ files across 8 domains (the system contract) |
| `specs/` | SpecKit implementation artifacts — per-phase spec, plan, and task files (the build instructions) |
| `ROADMAP.md` | 8-phase build roadmap — dependency-aware implementation order |
| `plans/` | Implementation plans — batch 1 (core architecture) 7 of 8 agents complete, batch 2 partial |
| `archive/` | Completed validation work, historical operations, and research |
| `nats.rs/` | Official NATS Rust client (cloned from nats-io/nats.rs) — reference for async-nats API |
| `.github/workflows/` | CI/CD pipelines |

> **`spec/` vs `specs/` — these are different directories.** `spec/` contains the canonical architecture specifications defining *what* the system is (types, patterns, interfaces, message schemas). `specs/` contains SpecKit-generated implementation artifacts defining *how* each phase is built (feature specs, plans, task breakdowns). The `ROADMAP.md` bridges them by referencing `spec/` docs for each phase.

## Key Entry Points

Start here when reading the framework:

1. **Build roadmap**: `ROADMAP.md` — 8-phase implementation order with gate criteria
2. **Architecture overview**: `spec/core-architecture/system-architecture.md`
3. **Component design**: `spec/core-architecture/component-architecture.md`
4. **Agent types and orchestration**: `spec/data-management/agent-orchestration.md`
5. **Message contracts**: `spec/data-management/message-schemas.md`
6. **Type system**: `spec/core-architecture/type-definitions.md`

## Architecture Domains

| Domain | Path | Files | Covers |
|--------|------|-------|--------|
| Core Architecture | `spec/core-architecture/` | 19 | System design, async patterns, supervision trees, Tokio runtime, types |
| Data Management | `spec/data-management/` | 19 | Agent orchestration, message schemas, persistence, storage |
| Transport | `spec/transport/` | 5 | NATS, gRPC, HTTP transport layers |
| Security | `spec/security/` | 7 | Auth, authorization, TLS, security patterns |
| Operations | `spec/operations/` | 7 + scripts | Deployment, monitoring, configuration, build |
| Agent Domains | `spec/agent-domains/` | 1 | Consolidated agent type analysis (9 agent types) |
| Testing | `spec/testing/` | 2 | Test framework, test schemas |
| Research | `spec/research/` | 0 | Claude CLI files archived to `archive/claude-cli-research/` |

## High-Impact Files

Changes to these files cascade across the architecture:

- `spec/core-architecture/system-architecture.md` — foundation for all specs
- `spec/core-architecture/type-definitions.md` — core types referenced everywhere
- `spec/data-management/message-schemas.md` — message formats used across system
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` — agent type definitions

## Technology Stack

| Component | Spec Version | Notes |
|-----------|-------------|-------|
| MSRV | **1.88.0** | Driven by async-nats 0.46.0 requirement |
| Runtime | Tokio 1.49.0 | Full feature set (rt-multi-thread, io, net, time, sync, fs, process, signal) |
| Messaging | async-nats 0.46.0 | jetstream, kv, object-store, service features |
| HTTP | Axum 0.8 | |
| gRPC | Tonic 0.14 | With prost 0.14 for protobuf |
| Storage | PostgreSQL + Redis | sqlx with runtime-tokio-rustls |
| Security | JWT, TLS 1.3, mTLS | ring 0.17, jsonwebtoken 10, aes-gcm 0.10 |
| Orchestration | Kubernetes | |

> See `VERSION_REFERENCE.md` for the full dependency matrix. Review `nats.rs/async-nats/` for API reference before implementing transport layer.

## Local Development Environment

**NATS server**: Docker container `NATS` running nats-server v2.12.4
- Ports: 4222 (client), 6222 (cluster), 8222 (monitoring) — container-internal only, not published to host
- Start: `docker start NATS`
- To publish ports: `docker run -d --name NATS -p 4222:4222 -p 8222:8222 nats:latest`

**NATS Rust client**: `nats.rs/` — cloned from `nats-io/nats.rs`, contains async-nats 0.46.0 source for API reference

## Available Apps (via Rube MCP)

The following apps are connected and available for use. Select the most appropriate app or tool based on the task at hand.

| App | Description |
|-----|-------------|
| **Context7 MCP** | Fetches up-to-date, version-specific documentation and code examples directly into the prompt. Use when you need accurate library/framework docs or API references. |
| **GitHub** | Code hosting and version control platform. Use for managing repositories, creating/reviewing pull requests, tracking issues, and CI/CD workflows. |
| **Tavily** | AI-optimized search and data retrieval. Use for quickly searching the web or filtering relevant information from documents and databases. Load the Tavily-best-practices skill whenever you need to use Tavily > .claude/skills/tavily-best-practices |

## Active Technologies
- Phase 3 (complete): Rust MSRV 1.88.0, Tokio 1.49.0 (mpsc, oneshot, sync, time), async-trait 0.1.83, mister-smith-core (Actor/Supervisor traits, supervision types, error types), mister-smith-events (EventBus, AgentEventType), mister-smith-monitoring (HealthCheck, HealthMonitor, MetricsCollector)
- Phase 4 (next): async-nats 0.46.0 (jetstream, kv, service), rmcp 1.1.0 (client, server, streamable-HTTP), rmp-serde 1.3.1, Axum 0.8.8, Tonic 0.14.x, prost 0.14.x, tonic-build 0.14.x, serde 1.x, serde_json 1.x, bytes 1.x, uuid 1.x
- Rust, MSRV 1.88.0 (005-phase5-security)
- In-memory (audit persistence deferred to Phase 6) (005-phase5-security)
- Rust, MSRV 1.88.0 + sqlx 0.8.6 (new), async-nats 0.46.0 (existing), tokio 1.49.0 (existing), serde 1.x (existing) (006-phase6-persistence-state)
- PostgreSQL 15+ (relational), JetStream KV (distributed ephemeral) (006-phase6-persistence-state)
- Rust, MSRV 1.88.0 + mister-smith-core (types, traits), mister-smith-actor (ActorCell, ActorRef, mailbox), mister-smith-supervision (SupervisedSystem, restart strategies), mister-smith-transport (Transport, DurableTransport, MessageEnvelope), mister-smith-nats (NatsTransport, JetStream), mister-smith-mcp (MCP client/server, tool bridge), mister-smith-security (PolicyEngine, JwtManager, AuditLogger), mister-smith-persistence (repositories, state persistence), mister-smith-events (EventBus), mister-smith-monitoring (HealthMonitor, phi accrual) (007-phase7-agent-system)
- PostgreSQL (via Phase 6 persistence layer), JetStream KV (via Phase 6 dual-store) (007-phase7-agent-system)

## Recent Changes
- 006-phase6-persistence-state: Added sqlx 0.8.6, chrono 0.4 — PR #108
- 005-phase5-security: Added Rust, MSRV 1.88.0

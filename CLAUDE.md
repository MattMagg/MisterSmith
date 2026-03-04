# Mister Smith

Multi-agent orchestration framework — Rust + NATS + supervision trees. Model-agnostic (works with any LLM).

## Commands

```bash
cargo build --workspace                    # Build all crates
cargo test --workspace                     # Run all tests (243 as of Phase 2)
cargo clippy --workspace -- -D warnings    # Lint (must pass clean)
```

## Implementation Status

| Phase | Status | Crates |
|-------|--------|--------|
| 1. Foundation | Complete | `mister-smith-core`, `mister-smith-config` |
| 2. Runtime & Async | Complete | `mister-smith-runtime`, `mister-smith-monitoring`, `mister-smith-events`, `mister-smith-async`, `mister-smith-resources`, `mister-smith-integration-tests` |
| 3. Actor System & Supervision | Next | `mister-smith-actor`, `mister-smith-supervision` (planned) |
| 4–8 | Not started | See `ROADMAP.md` |

## Workspace Crate Dependencies

```
mister-smith-core (foundation types, traits, errors)
├── mister-smith-config (typed TOML config, env overlay)
├── mister-smith-runtime (Tokio RuntimeManager, scheduling)
├── mister-smith-monitoring (HealthMonitor, phi accrual failure detector, metrics)
├── mister-smith-events (EventBus pub/sub, dead letter queue)
├── mister-smith-async (TaskExecutor, CircuitBreaker, RetryPolicy, stream processing)
├── mister-smith-resources (ConnectionPool, pool sizing, health reports)
└── mister-smith-integration-tests (cross-crate validation)
```

## Repository Structure

| Directory | Contents |
|-----------|----------|
| `crates/` | Rust workspace — 8 crates (Phase 1 foundation + Phase 2 runtime/async) |
| `specs/` | SpecKit feature directories with spec, plan, and task artifacts |
| `ROADMAP.md` | 8-phase build roadmap — dependency-aware implementation order |
| `spec/` | Framework specifications — 65+ files across 8 domains |
| `plans/` | Implementation plans — batch 1 (core architecture) 7 of 8 agents complete, batch 2 partial |
| `archive/` | Completed validation work, historical operations, and research |
| `nats.rs/` | Official NATS Rust client (cloned from nats-io/nats.rs) — reference for async-nats API |
| `.github/workflows/` | CI/CD pipelines |

## Key Entry Points

Start here when reading the framework:

1. **Build roadmap**: `ROADMAP.md` — 8-phase implementation order with gate criteria
2. **Architecture overview**: `spec/core-architecture/system-architecture.md`
3. **Component design**: `spec/core-architecture/component-architecture.md`
4. **Agent types and orchestration**: `spec/data-management/agent-orchestration.md`
5. **Message contracts**: `spec/data-management/message-schemas.md`
6. **Type system**: `spec/core-architecture/type-definitions.md`

## Spec Domains

| Domain | Path | Files | Covers |
|--------|------|-------|--------|
| Core Architecture | `spec/core-architecture/` | 21 | System design, async patterns, supervision trees, Tokio runtime, types |
| Data Management | `spec/data-management/` | 19 | Agent orchestration, message schemas, persistence, storage |
| Transport | `spec/transport/` | 5 | NATS, gRPC, HTTP transport layers |
| Security | `spec/security/` | 7 | Auth, authorization, TLS, security patterns |
| Operations | `spec/operations/` | 7 + scripts | Deployment, monitoring, configuration, build |
| Agent Domains | `spec/agent-domains/` | 1 | Consolidated agent type analysis (9 agent types) |
| Testing | `spec/testing/` | 2 | Test framework, test schemas |
| Research | `spec/research/` | 3 | Claude CLI integration analysis |

## High-Impact Files

Changes to these files cascade across the spec:

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
- Rust, MSRV 1.88.0 + okio 1.49.0 (mpsc, oneshot, sync, time), async-trait 0.1.83, mister-smith-core (Actor/Supervisor traits, supervision types, error types), mister-smith-events (EventBus, AgentEventType), mister-smith-monitoring (HealthCheck, HealthMonitor, MetricsCollector) (003-phase3-actor-supervision)
- N/A (in-memory only; no persistence in Phase 3) (003-phase3-actor-supervision)

## Recent Changes
- 003-phase3-actor-supervision: Added Rust, MSRV 1.88.0 + okio 1.49.0 (mpsc, oneshot, sync, time), async-trait 0.1.83, mister-smith-core (Actor/Supervisor traits, supervision types, error types), mister-smith-events (EventBus, AgentEventType), mister-smith-monitoring (HealthCheck, HealthMonitor, MetricsCollector)

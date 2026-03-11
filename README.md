# Mister Smith

A multi-agent orchestration framework built in Rust with NATS messaging and Erlang-inspired supervision trees. Model-agnostic — works with any LLM backend.

## Implementation Status

| Phase | Status | Crates | Tests |
|-------|--------|--------|-------|
| 1. Foundation | Complete | `mister-smith-core`, `mister-smith-config` | 60 |
| 2. Runtime & Async | Complete | `mister-smith-runtime`, `mister-smith-monitoring`, `mister-smith-events`, `mister-smith-async`, `mister-smith-resources` | 243 |
| 3. Actor & Supervision | Complete | `mister-smith-actor`, `mister-smith-supervision` | 389 |
| 4. Transport & Messaging | Complete | `mister-smith-transport`, `mister-smith-nats`, `mister-smith-http`, `mister-smith-grpc`, `mister-smith-mcp` | 605 |
| 5. Security | Complete | `mister-smith-security` | 717 |
| 6. Persistence & State | Complete | `mister-smith-persistence` | 882 |
| 7. Agent System | Complete | `mister-smith-agents` | 951 |
| 8. Operations | Complete | `mister-smith-app` | 983 |
| 9. LLM Providers | Complete | `mister-smith-llm`, `mister-smith-agents` (llm feature) | 1115 |

**20 crates** in the workspace (18 library + 1 binary + 1 integration test), **1,115 tests** passing, zero clippy warnings.

## Architecture

Mister Smith coordinates distributed AI agents through three core subsystems:

**Supervision Trees** — Hierarchical fault tolerance inspired by Erlang/OTP. Supervisors manage agent lifecycles with configurable restart strategies (OneForOne, OneForAll, RestForOne), failure escalation, and circuit breakers.

**NATS Messaging** — High-performance pub/sub communication layer using NATS and JetStream. Supports request-response, publish-subscribe, queue groups, and hierarchical subject-based routing (`agents.{id}.commands.{type}`, `tasks.{type}.assignment`, `workflow.{id}.step.{step_id}`, etc.).

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

## Vet

Pull requests now run a dedicated `Vet` GitHub Actions workflow from
`.github/workflows/vet.yml`. The workflow uses the repo-local `.vet/configs.toml`
`ci` profile, so the PR review configuration lives with the repository instead
of inside the GitHub UI.

For local Codex sessions, run:

```bash
scripts/run-vet.sh "Describe the change you want vet to review"
```

The wrapper auto-discovers the newest Codex session file for this repository,
loads the project-level Codex history exporter, and uses the repo's `codex`
profile by default. Local non-agentic runs therefore need `OPENAI_API_KEY` or a
compatible custom `vet` model configuration.

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
├── mister-smith-nats      NATS pub/sub, request-reply, JetStream, W3C TraceContext
├── mister-smith-http      Axum REST API, WebSocket, middleware, rate limiting
├── mister-smith-grpc      Tonic gRPC services, health, protobuf
├── mister-smith-mcp       MCP client/server, tool registry, NATS bridge
├── mister-smith-security  JWT auth, RBAC, TLS/mTLS, audit logging
├── mister-smith-persistence  PostgreSQL + JetStream KV dual-store, repositories, audit bridge
├── mister-smith-llm       ModelProvider trait, MockProvider, 4 LLM providers, ModelRouter, dual-stream
├── mister-smith-agents    AgentRuntime, registry, scheduler, orchestrator, team, tool bus, 9 roles, LLM bridge
├── mister-smith-app       Binary entry point, bootstrap, shutdown, observability, health probes
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
| rmcp | 1.1.0 | MCP client/server |
| sqlx | 0.8.6 | PostgreSQL async driver |
| jsonwebtoken | 10.x | JWT authentication |
| rustls | 0.23 | TLS/mTLS |
| opentelemetry | 0.31.0 | Distributed tracing |
| metrics-exporter-prometheus | 0.18.1 | Prometheus metrics |
| clap | 4.x | CLI argument parsing |
| serde | 1.0 | JSON/MessagePack serialization |
| thiserror | 1.0 | Error hierarchy |
| tracing | 0.1 | Structured logging |

## LLM Integration (Phase 9)

- **5 providers**: MockProvider (deterministic testing), AnthropicProvider, OpenAiProvider, OpenAiChatGptProvider, ClaudeSubscriptionProvider
- **Model routing**: Round-robin, cost-optimized, capability-matched, and cascade (SLM-default/LLM-fallback with confidence-based escalation)
- **Dual-stream architecture**: Semantic channel (lossless tool calls, lifecycle events) + UI channel (best-effort text with backpressure coalescing)
- **Budget enforcement**: Reserve-before-send / reconcile-after-completion with hierarchical key resolution
- **Circuit breaker**: Per-provider health tracking with sliding-window error rate and p95 latency
- **Agent bridge**: Planner, Critic, Executor roles gain LLM-powered implementations via `llm` feature flag
- **Tool calling**: Bidirectional ToolBus ↔ LLM function calling round-trip

## Production Features (Phase 8)

- **Process lifecycle**: Deterministic bootstrap sequence with configurable startup timeout
- **Health probes**: `/health/live` (always 200), `/health/ready` (200 when Ready, 503 otherwise)
- **Prometheus metrics**: `/metrics` endpoint with framework-level counters, gauges, histograms
- **Distributed tracing**: W3C TraceContext propagation through NATS message envelopes
- **Graceful shutdown**: Signal handling (SIGTERM graceful, second signal forced), connection draining
- **Deployment**: Multi-stage Dockerfile (<100MB), Kubernetes manifests, Grafana dashboards, Prometheus alert rules

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
| Phase 9 LLM design | [`specs/009-phase9-llm-provider-integration/`](specs/009-phase9-llm-provider-integration/) |
| Frontier autonomy note | [`docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`](docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md) |
| Version baseline | [`VERSION_REFERENCE.md`](VERSION_REFERENCE.md) |
| Deployment artifacts | [`deploy/`](deploy/) |

## Design Principles

- **Fail-fast with graceful recovery** — Detect failures quickly, recover through supervision trees
- **Event-driven architecture** — Loose coupling via pub/sub messaging
- **Resource-bounded execution** — Memory-bounded contexts, connection pooling, backpressure handling
- **Model-agnostic** — No LLM vendor lock-in; works with any backend
- **Observable by default** — Structured logs, distributed traces, and Prometheus metrics from day one

## License

MIT OR Apache-2.0

# Mister Smith

A frontier orchestration operating system built in Rust with NATS messaging and Erlang-inspired
supervision trees. Model-agnostic and designed for supervised, stateful, multi-agent execution.

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
| 10. Frontier Autonomy & Control Plane | Complete | `mister-smith-agents`, `mister-smith-persistence`, `mister-smith-security`, `mister-smith-app`, `mister-smith-events`, `mister-smith-llm`, deploy assets | Phase 10 gate validated on 2026-03-15 |

**20 crates** in the workspace (18 library + 1 binary + 1 integration test). The table above now
reflects completed build phases through Phase 10. Treat exact test totals and warning counts as
CI-validated state, not a static README guarantee.

## Start Here

Use [`docs/current-state.md`](docs/current-state.md) as the stable repo-wide overview and document
router.

Document roles in brief:

- `docs/current-state.md`: current repo and OS state
- `docs/plans/2026-03-19-central-development-checkpoint.md`: current forward-development checkpoint
- `docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md`:
  packet-015 closure evidence
- `WORKFLOW.md` and `docs/linear/LINEAR.md`: development control-plane contract
- `ROADMAP.md`: architectural build map
- `spec/` and `specs/`: architecture and implementation packet truth

## Current Control Plane

This section is about the development control plane around the repo. It is not the same thing as
the Mister Smith OS runtime.

- Smith-first development workflow system:
  [`docs/plans/2026-03-16-smith-first-development-system.md`](docs/plans/2026-03-16-smith-first-development-system.md)
- Current Smith workflow-family implementation note:
  [`docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md`](docs/plans/2026-03-16-smith-mcp-ms-51-ms-59-execution.md)
- Live queue contract:
  [`WORKFLOW.md`](WORKFLOW.md) and
  [`docs/linear/LINEAR.md`](docs/linear/LINEAR.md)
- Current mainline direction and packet-015 closure posture:
  [`docs/plans/2026-03-19-central-development-checkpoint.md`](docs/plans/2026-03-19-central-development-checkpoint.md),
  [`docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md`](docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md)
- Phase 10 design and gate artifacts:
  [`specs/012-phase10-frontier-autonomy/`](specs/012-phase10-frontier-autonomy/spec.md),
  [`docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`](docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md),
  [`specs/015-complex-multi-agent-proof-and-unified-result-surfaces/`](specs/015-complex-multi-agent-proof-and-unified-result-surfaces/spec.md)
- smith MCP rebuild and workflow recovery plans:
  [`docs/plans/2026-03-14-smith-mcp-rebuild.md`](docs/plans/2026-03-14-smith-mcp-rebuild.md),
  [`docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`](docs/plans/2026-03-15-smith-mcp-workflow-forensics.md),
  [`docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`](docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md)

## Smith MCP Workflow Families

Use Smith as the default control-plane entrypoint for repo development work.

- route and state discovery:
  `route_workflow_request`, `get_control_plane_snapshot`, `get_issue_execution_snapshot`,
  `resolve_issue_lifecycle`
- Linear issue and workpad mutation:
  `save_linear_issue`, `save_issue_workpad`
- backlog slicing and watched-queue control:
  `materialize_backlog_slices`, `plan_queue_stage`, `apply_queue_stage`
- Ralph and SpecKit glue:
  `prepare_ralph_packet`, `record_ralph_outcome`, `prepare_speckit_context`,
  `translate_speckit_tasks`
- Managed Ralph entrypoint:
  `./scripts/bootstrap_ralph.sh` updates the upstream checkout and `./scripts/ralph`
  is the only supported Ralph command path for Mister Smith sessions. Before
  every `./scripts/ralph run`, prepare the current packet/workpad context with
  `./scripts/ralph prompt --packet <packet.json>`; the prep marker is one-shot,
  so rerun the prompt step before every subsequent `run`.

Default operator sequence:

1. Route the request with `route_workflow_request`.
2. Snapshot current repo, issue, or queue state before mutation.
3. Use the Smith workflow-family tools before raw Linear or ad hoc shell fallbacks.
4. Stage watched-queue work only through `plan_queue_stage` and `apply_queue_stage`.

Phase 10 gate evidence on 2026-03-15:

- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-persistence`
- `cargo test -p mister-smith-security`
- `cargo test -p mister-smith-llm`
- `cargo test -p mister-smith-core`
- `cargo test -p mister-smith-app`
- `python3 scripts/validate_deploy_assets.py deploy/dashboards deploy/alerts`
- `cargo build --workspace`

## Current Operator Surfaces

These are OS runtime surfaces, not Symphony or Linear development workflow surfaces.

The repo now has real runtime-backed operator paths plus the packet-015 result-surface closure
validated against `openai_chatgpt` / `gpt-5.4` through March 20, 2026:

- one-shot task execution:
  - `mister-smith run`
  - `POST /api/v1/tasks`
  - `GET /api/v1/tasks/{task_id}`
  - `mister-smith autonomy list`
  - `mister-smith autonomy status --workflow-id <id>`
- durable same-agent conversation sessions:
  - `POST /api/v1/sessions`
  - `POST /api/v1/sessions/{session_id}/turns`
  - `GET /api/v1/sessions/{session_id}`
  - `POST /api/v1/sessions/{session_id}/end`

Current session contract for the first bounded slice:

- one stable `session_id` and `coordinator_agent_id` across accepted turns
- one active turn at a time per session
- workflow autonomy remains keyed by `workflow_id`, with session linkage included in the rendered
  status
- ended sessions stay inspectable and reject later turns

Primary notes for the current operator and result-surface baseline:

- runtime-backed task path and first real live proof:
  [`docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`](docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md)
- bounded same-agent session packet and live validation notes:
  [`docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`](docs/plans/2026-03-16-multi-turn-same-agent-conversations.md)
- forward checkpoint and packet-015 closure evidence:
  [`docs/plans/2026-03-19-central-development-checkpoint.md`](docs/plans/2026-03-19-central-development-checkpoint.md),
  [`docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md`](docs/plans/2026-03-20-packet-015-live-runtime-evaluation.md)

## Architecture

Mister Smith coordinates distributed AI agents through three core subsystems:

**Supervision Trees** — Hierarchical fault tolerance inspired by Erlang/OTP. Supervisors manage agent lifecycles with configurable restart strategies (OneForOne, OneForAll, RestForOne), failure escalation, and circuit breakers.

**NATS Messaging** — High-performance pub/sub communication layer using NATS and JetStream. Supports request-response, publish-subscribe, queue groups, and hierarchical subject-based routing (`agents.{id}.commands.{type}`, `tasks.{type}.assignment`, `workflow.{id}.step.{step_id}`, etc.).

**Agent Orchestration** — Nine specialized agent roles (Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory) with dynamic team composition based on task requirements.

## Quick Start

```bash
# Build
cargo build --workspace

# Test the affected crate
cargo test -p <crate-name>

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

This section describes landed repo substrate, not a claim that every capability below is already
wired into the default live runtime path. Use [`docs/current-state.md`](docs/current-state.md) for
that distinction.

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
- **Prometheus metrics**: `/metrics` endpoint with system-level counters, gauges, histograms
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

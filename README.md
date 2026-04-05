# Mister Smith

A multi-agent orchestration operating system built in Rust. Mister Smith coordinates AI agents
through Erlang-inspired supervision trees, NATS messaging, and a model-agnostic LLM layer —
giving you fault-tolerant, observable, budget-aware agent execution with real operator control.

## Why Mister Smith

Most agent frameworks give you a thin wrapper around LLM calls. Mister Smith is an operating
system for agents — it manages their lifecycles, supervises their failures, routes their messages,
enforces their budgets, and gives operators real-time visibility into what every agent is doing and
why.

**Supervision, not hope.** Agents crash. Models hallucinate. Networks partition. Mister Smith
handles all of it through hierarchical supervision trees borrowed from Erlang/OTP — the same
pattern that keeps telephone switches running for decades. When an agent fails, its supervisor
decides whether to restart it, restart its siblings, or escalate. No silent failures, no orphaned
tasks.

**Model-agnostic by design.** Mister Smith is not married to any LLM provider. Swap between
OpenAI, Anthropic, Claude, or your own provider without changing your agent logic. Cascade routing
lets you start with a cheaper model and escalate to a more capable one only when confidence drops —
cutting costs without sacrificing quality.

**Budget enforcement built in.** Reserve tokens before sending, reconcile after completion,
enforce hierarchical limits per workflow, branch, and step. Runaway agents don't run away your
bill.

**Observable from day one.** OpenTelemetry traces, Prometheus metrics, Grafana dashboards, and
structured JSON logs ship with the system. W3C TraceContext propagates through every NATS message
envelope so you can follow a request from CLI submission through agent coordination to final
result.

**Rust performance and safety.** No garbage collector pauses. No null pointer exceptions. The
entire system compiles to a single static binary under 100MB. Memory-bounded contexts and
backpressure handling prevent resource exhaustion under load.

## Features

### Nine Specialized Agent Roles

Mister Smith decomposes work across purpose-built agent roles that form dynamic teams based on task
requirements:

| Role | Purpose |
|------|---------|
| **Supervisor** | Manages agent lifecycles and restart strategies |
| **Coordinator** | Orchestrates multi-agent workflows and dependencies |
| **Planner** | Decomposes goals into executable subtask DAGs |
| **Executor** | Carries out atomic actions and tool calls |
| **Critic** | Validates outcomes against acceptance criteria |
| **Router** | Load-balances and routes tasks by capability |
| **Worker** | Performs computational work units |
| **Monitor** | Observes system health and collects metrics |
| **Memory** | Persistent knowledge storage and retrieval |

### Erlang-Style Supervision Trees

Hierarchical fault tolerance with four restart strategies:

- **OneForOne** — restart only the failed agent (independent workers)
- **OneForAll** — restart all siblings when one fails (tightly coupled groups)
- **RestForOne** — restart the failed agent and everything started after it (ordered pipelines)
- **Escalate** — propagate to the parent supervisor (critical failures)

Failure detection uses a phi-accrual detector that adapts to each agent's heartbeat pattern,
minimizing false positives while catching real failures fast.

### Multi-Provider LLM Integration

Five built-in providers with pluggable routing:

- **Anthropic** (Claude API)
- **OpenAI** (standard API)
- **OpenAI ChatGPT** (session-based)
- **Claude Subscription** (credential-based)
- **Mock** (deterministic testing)

**Cascade routing** starts with your default provider and escalates to a more capable (and
expensive) model when confidence drops below a configurable threshold. Combined with per-workflow
budget enforcement, you get cost control without manual intervention.

**Dual-stream architecture** separates semantic events (tool calls, lifecycle transitions) from UI
output (streaming text), so your agent coordination stays lossless while user-facing output handles
backpressure gracefully.

### Transport and Messaging

Four transport backends for different needs:

| Transport | Technology | Use Case |
|-----------|-----------|----------|
| **NATS** | async-nats 0.46 + JetStream | Agent-to-agent messaging, pub/sub, persistent queues |
| **HTTP** | Axum 0.8 | REST API, WebSocket streaming, operator interface |
| **gRPC** | Tonic 0.14 | Type-safe inter-service RPC |
| **MCP** | rmcp 1.1 | Bidirectional tool discovery and invocation |

NATS subject hierarchy provides structured routing:
```
agents.{id}.commands     # Direct agent commands
tasks.{type}.queue       # Task distribution queues
workflow.{id}.step.{n}   # Workflow step coordination
events.{type}            # System-wide event bus
```

### Security

- **JWT authentication** (RS256, 15-minute access tokens, 7-day refresh)
- **RBAC + ABAC** authorization with fine-grained permission policies
- **mTLS** between services (rustls, TLS 1.2+)
- **Message signing** (HMAC-SHA256 with nonce-based replay prevention)
- **Delegation chains** with provenance tracking across agent boundaries
- **Sandbox execution** with timeout and memory limits
- **Agent quarantine** for unhealthy agents
- **Full audit trail** persisted to PostgreSQL

### Persistence

Dual-store architecture balances consistency with speed:

- **PostgreSQL** — authoritative store for tasks, sessions, agents, audit logs
- **JetStream KV** — distributed cache for agent state, budgets, ephemeral data

A data router directs reads and writes to the appropriate backend with conflict resolution
and graceful degradation when a backend is unavailable.

### Observability

- **Health probes**: `/health/live` (liveness) and `/health/ready` (readiness)
- **Prometheus metrics**: task throughput, agent capacity, error rates, context pressure, budget usage
- **OpenTelemetry tracing**: W3C TraceContext propagation through NATS envelopes
- **Grafana dashboards**: pre-built system and autonomy dashboards in `deploy/dashboards/`
- **Alert rules**: pre-built Prometheus alerts in `deploy/alerts/`
- **WebSocket streaming**: real-time event feed at `/api/v1/events/ws`

## Getting Started

### Prerequisites

- **Rust** 1.88.0 or later
- **Docker** and Docker Compose (for PostgreSQL and NATS)
- An LLM provider API key (or use the mock provider for testing)

### Build

```bash
git clone https://github.com/MattMagg/MisterSmith.git
cd mister-smith
cargo build --workspace
```

### Start Infrastructure

```bash
docker compose -f deploy/docker-compose.yml up -d postgres nats
```

This starts PostgreSQL on port 5432 and NATS (with JetStream) on port 4222.

### Configure

Create a `config.toml` (or use environment variables):

```toml
[runtime]
worker_threads = 4

[transport]
nats_url = "nats://localhost:4222"
http_port = 8080

[monitoring]
health_check_interval = "30s"
log_level = "info"

[llm]
provider_kind = "openai_chatgpt"
model_id = "gpt-5.4"
```

Environment variable overrides:
- `MISTER_SMITH_LOG_LEVEL` — log level (trace, debug, info, warn, error)
- `MISTER_SMITH_NATS_URL` — NATS connection URL
- `MISTER_SMITH_DATABASE_URL` — PostgreSQL connection string
- `ANTHROPIC_API_KEY` — Anthropic provider credentials
- `OPENAI_API_KEY` — OpenAI provider credentials

### Run

```bash
# Start the runtime
mister-smith run --config config.toml

# Or with defaults
mister-smith run
```

## Usage

### Submit a Task

```bash
# Via HTTP API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"description": "Analyze the error logs from the last hour", "priority": "high"}'

# Check task status
curl http://localhost:8080/api/v1/tasks/{task_id}
```

### Durable Conversations

Multi-turn sessions maintain a stable agent coordinator across turns:

```bash
# Start a session
mister-smith conversation start --message "Help me debug this memory leak"

# Continue with follow-up turns
mister-smith conversation continue \
  --session-id <session_id> \
  --message "Here are the heap dumps from the last 3 runs"

# Inspect the full session history
mister-smith conversation inspect --session-id <session_id>

# End the session
mister-smith conversation end --session-id <session_id>
```

### Inspect Autonomy Status

See what every workflow is doing, its topology, branch health, and intervention history:

```bash
# List all active workflows
mister-smith autonomy list

# Detailed status for one workflow
mister-smith autonomy status --workflow-id <workflow_id>
```

### Provider Authentication

```bash
# ChatGPT browser-based login
mister-smith auth openai-chatgpt login
mister-smith auth openai-chatgpt status

# Claude subscription status
mister-smith auth claude status
```

### REST API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health/live` | Liveness probe (always 200) |
| `GET` | `/health/ready` | Readiness probe (503 during startup) |
| `GET` | `/metrics` | Prometheus metrics |
| `POST` | `/api/v1/tasks` | Submit a new task |
| `GET` | `/api/v1/tasks/{id}` | Get task status and results |
| `GET` | `/api/v1/agents` | List registered agents |
| `POST` | `/api/v1/sessions` | Start a durable session |
| `POST` | `/api/v1/sessions/{id}/turns` | Add a turn to a session |
| `GET` | `/api/v1/sessions/{id}` | Inspect session state |
| `POST` | `/api/v1/sessions/{id}/end` | End a session |
| `GET` | `/api/v1/events/ws` | WebSocket event stream |

## Deployment

### Docker

The multi-stage Dockerfile produces a distroless image under 100MB:

```bash
docker build -f deploy/Dockerfile -t mister-smith .
```

### Docker Compose (Full Stack)

```bash
docker compose -f deploy/docker-compose.yml up -d
```

This starts PostgreSQL, NATS, the Mister Smith runtime, OpenTelemetry collector, and Grafana.

### Kubernetes

Manifests for Deployment, Service, ConfigMap, and Secrets are in `deploy/kubernetes/`.

### Operator Console

A local macOS desktop app (Tauri + React) for visual operation:

- Boots the local PostgreSQL + NATS stack automatically
- Launches the Mister Smith runtime as a managed sidecar
- Real-time workflow timeline via WebSocket
- Task submission and session management
- NATS monitor integration

```bash
cd apps/operator-console
npm install && npm run tauri dev
```

## Architecture

```
                        ┌──────────────────────────────┐
                        │     CLI / HTTP API / gRPC     │
                        └──────────────┬───────────────┘
                                       │
                        ┌──────────────▼───────────────┐
                        │       Agent Orchestrator      │
                        │  (dynamic team composition)   │
                        └──────────────┬───────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
    ┌─────────▼─────────┐   ┌─────────▼─────────┐   ┌─────────▼─────────┐
    │    Supervision     │   │    LLM Router      │   │    Tool Bus       │
    │  (fault tolerance) │   │  (cascade routing)  │   │ (function calling) │
    └─────────┬─────────┘   └─────────┬─────────┘   └─────────┬─────────┘
              │                        │                        │
    ┌─────────▼─────────────────────────▼────────────────────────▼─────────┐
    │                        NATS Messaging Layer                          │
    │              (pub/sub, JetStream, request-reply)                     │
    └─────────┬──────────────────────────────────────────────┬─────────────┘
              │                                              │
    ┌─────────▼─────────┐                         ┌─────────▼─────────┐
    │    PostgreSQL      │                         │   JetStream KV    │
    │  (authoritative)   │                         │     (cache)       │
    └───────────────────┘                         └───────────────────┘
```

## Workspace

20 crates organized by architectural layer:

| Layer | Crates |
|-------|--------|
| **Foundation** | `mister-smith-core`, `mister-smith-config` |
| **Runtime** | `mister-smith-runtime`, `mister-smith-monitoring`, `mister-smith-events`, `mister-smith-async`, `mister-smith-resources` |
| **Actor System** | `mister-smith-actor`, `mister-smith-supervision` |
| **Transport** | `mister-smith-transport`, `mister-smith-nats`, `mister-smith-http`, `mister-smith-grpc`, `mister-smith-mcp` |
| **Security** | `mister-smith-security` |
| **Persistence** | `mister-smith-persistence` |
| **LLM** | `mister-smith-llm` |
| **Agents** | `mister-smith-agents` |
| **Application** | `mister-smith-app` |
| **Testing** | `mister-smith-integration-tests` |

## Technology

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | 1.88.0 MSRV |
| Async runtime | Tokio | 1.49.0 |
| Messaging | async-nats (JetStream, KV) | 0.46.0 |
| HTTP | Axum | 0.8.8 |
| gRPC | Tonic + Prost | 0.14 |
| MCP | rmcp | 1.1.0 |
| Database | sqlx (PostgreSQL) | 0.8.6 |
| Auth | jsonwebtoken + rustls | 10.x, 0.23 |
| Observability | opentelemetry + tracing | 0.31.0, 0.1.44 |
| Metrics | metrics-exporter-prometheus | 0.18.1 |
| Serialization | serde + serde_json + rmp-serde | 1.x |

## Development

```bash
# Build everything
cargo build --workspace

# Test a specific crate
cargo test -p mister-smith-agents

# Lint
cargo clippy --workspace -- -D warnings
```

## License

MIT OR Apache-2.0

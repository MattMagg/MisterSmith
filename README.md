# Mister Smith

Mister Smith is a multi-agent orchestration operating system built in Rust. It combines supervised
agent execution, NATS and JetStream messaging, PostgreSQL-backed workflow state, MCP integration,
durable sessions, and operator-facing CLI, HTTP, and desktop surfaces into one runtime-focused
system.

The current repo is centered on building a real agent runtime, not just a prompt wrapper. On
`main`, Mister Smith already ships the runtime substrate, autonomy inspection surfaces, same-agent
session continuity, bounded live runtime proof on the `openai_chatgpt` / `gpt-5.4` path, and a
local macOS operator console for managing the stack and inspecting runs.

## Why Mister Smith

Most agent frameworks stop at orchestration helpers around model calls. Mister Smith is trying to
be the operating layer behind long-running, inspectable, failure-tolerant agent work:

- **Supervised execution** rather than optimistic retries
- **Durable workflow and session state** rather than best-effort memory
- **Operator-visible autonomy** rather than hidden internal heuristics
- **Explicit runtime truth and proof boundaries** rather than vague "agent succeeded" summaries
- **Strong execution boundaries** across tools, transports, and external capabilities

This repo should be read as a runtime system with real operator surfaces, not as a development
workflow around Linear, Symphony, or other external tools.

## Current Highlights

- **Session-first CLI shell.** Running `mister-smith` with no subcommand opens the retained-session
  shell, and the binary also supports `resume`, `sessions`, `conversation`, `autonomy`, and
  provider auth helpers.
- **Durable same-agent conversations.** Sessions retain `session_id`, coordinator continuity, turn
  history, and follow-up control surfaces across CLI and HTTP.
- **Operator control plane.** Runtime submission, task inspection, autonomy inspection, and live
  event streaming are exposed through CLI, HTTP, and the local desktop operator console.
- **Runtime-backed orchestration substrate.** Supervised planner and executor lifecycles, ToolBus
  execution boundaries, NATS/JetStream transport, and PostgreSQL persistence are all part of the
  landed runtime path.
- **MCP in the product boundary.** The repo ships `mister-smith-mcp` as a real workspace crate for
  tool exposure, compatibility, and external capability mediation.
- **Recent landed runtime packets.** Packets `022` through `026` are landed on `main`, covering
  durable workflow ownership, runtime-truth and run-trace projection, agent-boundary hardening,
  deterministic step policy projection, and first bounded coordinator-runtime delegation.

## What Is Live On `main`

The current repo-wide truth lives in [docs/current-state.md](docs/current-state.md). At a high
level, the repo currently has:

- the Rust workspace substrate through Phase 10
- one-shot runtime execution through `mister-smith run` and `POST /api/v1/tasks`
- autonomy inspection through `mister-smith autonomy list`, `mister-smith autonomy status`, and
  related HTTP views
- durable session handling through CLI flows and `POST /api/v1/sessions`
- a bounded live runtime-proof baseline on `openai_chatgpt` / `gpt-5.4`
- a local macOS Tauri operator console under `apps/operator-console/`

The newest landed packet authorities are:

- `specs/023-runtime-truth-and-run-trace/`
- `specs/024-agent-boundary-security-hardening/`
- `specs/025-step-level-intelligence-v2/`
- `specs/026-first-real-coordinator-subagent-runtime/`

Draft, frozen-planning, or pre-spec forward work exists under `specs/027-*` through
`specs/031-*`, but those later packets are not yet the default runtime story.

## Community Health

- [CONTRIBUTING.md](CONTRIBUTING.md) for setup, validation, and pull request expectations
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community standards
- [SECURITY.md](SECURITY.md) for private vulnerability reporting
- [SUPPORT.md](SUPPORT.md) for where to ask questions and when to file issues

## Feature Surface

### Session-First Operator Surfaces

- CLI home shell for starting, resuming, and browsing retained sessions
- direct conversation commands for create, continue, inspect, and end
- autonomy inspection for workflow state, proof wording, and operator-facing status
- HTTP API plus WebSocket event feed for external operators and local tooling
- local Tauri operator console for stack bootstrap, run inspection, and session/task actions

### Runtime Orchestration

- Erlang-inspired supervision trees with restart strategy support
- coordinator, planner, executor, verifier, and runtime evidence projection seams
- durable workflow lifecycle, event-history, and effect-boundary ownership
- deterministic step-policy projection and coordinator-owned delegation surfaces
- explicit runtime-truth, run-trace, and proof-boundary views

### Transport, Execution, and Integration

| Surface | Technology | Role |
| ------- | ---------- | ---- |
| **NATS + JetStream** | async-nats | agent transport, event distribution, queues, and KV-backed runtime state |
| **HTTP** | Axum | task, session, autonomy, health, metrics, and websocket surfaces |
| **gRPC** | Tonic | typed service boundaries |
| **MCP** | rmcp | capability exposure, compatibility, and external tool mediation |
| **ToolBus** | repo-native execution boundary | bounded tool execution and capability control |

### Security and Observability

- JWT auth plus policy-aware authorization seams
- quarantine and external-capability enforcement surfaces
- structured runtime proof and delegated-work provenance
- Prometheus metrics, OpenTelemetry traces, Grafana dashboards, and alert rules
- health probes at `/health/live` and `/health/ready`

## Getting Started

### Prerequisites

- **Rust** 1.88.0 or later
- **Docker** and Docker Compose (for PostgreSQL and NATS)
- An LLM provider API key (or use the mock provider for testing)

### Build

```bash
git clone https://github.com/MattMagg/MisterSmith.git
cd MisterSmith
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

# Open the session-first CLI shell
mister-smith
```

## Usage

### Session-First CLI

```bash
# Open the home shell
mister-smith

# Resume the last retained session
mister-smith resume --last

# Browse retained sessions
mister-smith sessions list
```

### Durable Conversations

Multi-turn sessions maintain a stable coordinator across turns:

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

### Submit A Task

```bash
# Via HTTP API
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"description": "Analyze the error logs from the last hour", "priority": "high"}'

# Check task status
curl http://localhost:8080/api/v1/tasks/{task_id}
```

### Inspect Autonomy Status

See what the operator-facing autonomy control plane is doing:

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
| ------ | -------- | ----------- |
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
- Session and run-detail inspection for the current local stack

```bash
cd apps/operator-console
npm install && npm run tauri dev
```

## Architecture

```text
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
| ----- | ------ |
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

## Core Stack

- **Rust** runtime and workspace architecture
- **Tokio** async execution
- **NATS + JetStream** messaging and bounded state projection
- **Axum** HTTP and websocket operator surfaces
- **Tonic** gRPC boundaries
- **rmcp** MCP support
- **PostgreSQL + sqlx** durable state
- **OpenTelemetry, tracing, Prometheus, Grafana** observability

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

This repository uses dual licensing under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

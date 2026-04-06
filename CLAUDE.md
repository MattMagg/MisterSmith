# Mister Smith

Frontier orchestration operating system — Rust + NATS + supervision trees. Model-agnostic and
built for supervised multi-agent execution.

## Commands

```bash
cargo build --workspace                    # Build all crates
cargo test -p <crate>                      # Test a specific crate after changes
cargo clippy --workspace -- -D warnings    # Lint (must pass clean)
```

> **NEVER run `cargo test --workspace` unless the user explicitly asks for it.** The workspace has 20 crates and 1115+ tests — a full run takes minutes and is almost never necessary. Use `cargo test -p <crate>` for targeted testing after changes. Use `cargo build --workspace` (~8s) to check cross-crate compilation. Do not run workspace tests "just to check status" — the test count is tracked in MEMORY.md.

## Start Here

Use `docs/direction.md` for Mister Smith's overall direction and `docs/current-state.md` for the
stable repo-wide state summary and document router.

- `docs/direction.md`: single authoritative direction source for where Mister Smith is going and what should be built next
- `docs/current-state.md`: current repo and OS state
- `specs/023-runtime-truth-and-run-trace/`: latest landed runtime packet on `main`
- `specs/024-agent-boundary-security-hardening/`: latest landed security packet on `main`
- `specs/025-step-level-intelligence-v2/`: latest landed step-policy packet on `main`
- `specs/026-first-real-coordinator-subagent-runtime/`: latest landed coordinator-runtime packet on
  `main`
- `specs/027-capability-discovery-and-interoperability/`,
  `specs/028-selective-strong-coordination/`, `specs/029-session-first-user-shell/`,
  `specs/030-session-first-cli-shell/`, and `specs/031-chat-first-cli-loop/`: later draft,
  frozen-planning, or pre-spec packet material; no later packet is currently promoted as the next
  implementation-ready slice
- `specs/022-durable-workflow-core/`: packet-022 implementation authority
- `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md`: latest bounded live-proof note and
  artifact index for the current smoke-harness lane
- `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`: historical budget-aware proof
  note
- `docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md`: current direct-execution
  control-plane note
- local-only `docs/linear/LINEAR.md` when present: legacy Linear operating model for repo workflow
- `WORKFLOW.md`: legacy Symphony workflow background
- `ROADMAP.md`: architectural build map

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
| 8. Operations | Complete | `mister-smith-app` (binary entry point, bootstrap, shutdown, health probes, observability, cross-phase bridges) |
| 9. LLM Providers | Complete | `mister-smith-llm` (ModelProvider trait, MockProvider, OpenAI/Anthropic/Claude providers, ModelRouter, circuit breaker, budget enforcement, dual-stream, ModelEvent, cascade routing); `mister-smith-agents` extended with `llm` feature (Planner/Critic/Executor LLM integration, ToolBus bridge) |
| 9.1 Security Hardening | Complete | `mister-smith-transport`, `mister-smith-security`, `mister-smith-persistence`, `mister-smith-agents` |
| 10. Frontier Autonomy & Control Plane | Complete | `mister-smith-agents`, `mister-smith-persistence`, `mister-smith-security`, `mister-smith-app`, `mister-smith-events`, `mister-smith-llm`, deploy assets |

## Current Control Plane Sources

These are development workflow and control-plane sources, not the same thing as the Mister Smith OS
runtime.

- Smith-first development workflow system:
  `docs/plans/2026-03-16-smith-first-development-system.md`
- Current Smith workflow-family implementation note:
  `docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md`
- Legacy workflow background: `WORKFLOW.md`
- Local-only Linear operating model when present: `docs/linear/LINEAR.md`
- Current overall direction and repo-wide router: `docs/direction.md`, `docs/current-state.md`
- Latest landed packet authorities:
  `specs/023-runtime-truth-and-run-trace/`,
  `specs/024-agent-boundary-security-hardening/`,
  `specs/025-step-level-intelligence-v2/`,
  `specs/026-first-real-coordinator-subagent-runtime/`
- Latest bounded live-proof note and artifact index:
  `docs/plans/2026-04-05-live-runtime-eval-specs-022-026.md`
- Later packet material that is not yet promoted as the next implementation-ready slice:
  `specs/027-capability-discovery-and-interoperability/`,
  `specs/028-selective-strong-coordination/`,
  `specs/029-session-first-user-shell/`,
  `specs/030-session-first-cli-shell/`,
  `specs/031-chat-first-cli-loop/`
- Historical budget-aware live-proof note:
  `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
- Historical packet-016 closure evidence:
  `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
- Phase 10 artifact set and gate evidence: `specs/012-phase10-frontier-autonomy/`,
  `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`
- Historical workflow recovery and queue-governance background:
  `docs/plans/2026-03-14-smith-mcp-rebuild.md`,
  `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`,
  `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`

## Smith-First Operator Flow

For Mister Smith development sessions, assume Smith MCP is the first hop unless the repo proves a
real gap.

1. Route the operator request with `route_workflow_request`.
2. Pull current state with `get_control_plane_snapshot` or `get_issue_execution_snapshot`.
3. Use the Smith workflow-family tools for the actual task:
   - `save_linear_issue` and `save_issue_workpad`
   - `prepare_direct_execution` and `materialize_backlog_slices`
   - `resolve_issue_lifecycle` and `review_merge_status`
   - `prepare_ralph_packet` and `record_ralph_outcome`
   - `prepare_speckit_context` and `translate_speckit_tasks`
4. Fall back to raw Linear, shell, or one-off repo glue only when Smith does not yet model the
   operation.

When a task explicitly calls for Ralph, use `./scripts/ralph` instead of bare `ralph`. The wrapper
bootstraps the managed upstream install under `~/.local/share/mister-smith/ralph-orchestrator`.
Before every `./scripts/ralph run`, generate the current Smith packet/workpad context, run
`./scripts/ralph prompt --packet <packet.json>`, and only then invoke `./scripts/ralph run`.
Each successful `run` consumes the prep marker, so rerun `./scripts/ralph prompt` for every
subsequent run attempt.

## State Semantics

Phase completion in this repo means the relevant substrate and validation artifacts are landed. It
does not automatically mean every advanced seam is already wired into the default live runtime
path. Use `docs/current-state.md` when you need the honest distinction between "landed in repo"
and "live default runtime path". Use `docs/direction.md` when you need the strategic answer to
"what matters next and why".

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
├── mister-smith-transport (MessageEnvelope, Transport trait, serialization, InMemoryTransport, Phase 9.1 security envelope fields)
├── mister-smith-nats (NATS pub/sub, request-reply, JetStream, health checks)
├── mister-smith-http (Axum REST API, WebSocket, middleware, rate limiting)
├── mister-smith-grpc (Tonic gRPC services, health, protobuf)
├── mister-smith-mcp (MCP client/server, tool registry, NATS bridge)
├── mister-smith-security (JWT auth, RBAC, TLS/mTLS, audit logging, message signing, Auth Callout, state validation, sandbox/quarantine primitives)
├── mister-smith-persistence (PostgreSQL + JetStream KV dual-store, repositories, audit bridge, quarantined shared-state boundaries)
├── mister-smith-llm (ModelProvider trait, MockProvider, OpenAI/Anthropic/Claude providers, ModelRouter, circuit breaker, budget, dual-stream, ModelEvent)
├── mister-smith-agents (AgentRuntime, registry, scheduler, orchestrator, team, tool bus, 9 roles, optional LLM bridge, sandbox/quarantine integration)
├── mister-smith-app (binary entry point, bootstrap, shutdown, observability, health probes, cross-phase bridges)
└── mister-smith-integration-tests (cross-crate validation)
```

## Repository Structure

| Directory | Contents |
|-----------|----------|
| `crates/` | Rust workspace — 20 crates across landed phases 1-10 |
| `spec/` | Canonical architecture specifications — 65+ files across 8 domains (the system contract) |
| `specs/` | SpecKit implementation artifacts — per-phase spec, plan, and task files (the build instructions) |
| `ROADMAP.md` | 10-phase build roadmap — dependency-aware implementation order |
| `plans/` | Implementation plans — batch 1 (core architecture) 7 of 8 agents complete, batch 2 partial |
| `archive/` | Completed validation work, historical operations, and research |
| `deploy/` | Deployment artifacts — Dockerfile, docker-compose, Kubernetes manifests, Grafana dashboards, Prometheus alerts |
| `nats.rs/` | Official NATS Rust client (cloned from nats-io/nats.rs) — reference for async-nats API |
| `docs/` | Current-state routers, plans, research output, code reviews, and session analysis |
| `scripts/` | Runtime proof, Ralph, closure, and local support scripts |
| `.agents/workflows/` | Agent workflow templates (bulk PR merge, mandate) |
| `.github/` | Repo metadata, templates, labels, and archived workflow history; hosted GitHub Actions are intentionally disabled |

## Review Posture

- GitHub Actions are intentionally disabled in this repository.
- Use local validation as the merge gate.
- Treat CodeRabbit plus operator review as the active review posture.

> **`spec/` vs `specs/` — these are different directories.** `spec/` contains the canonical architecture specifications defining *what* the system is (types, patterns, interfaces, message schemas). `specs/` contains SpecKit-generated implementation artifacts defining *how* each phase is built (feature specs, plans, task breakdowns). The `ROADMAP.md` bridges them by referencing `spec/` docs for each phase.

## Key Entry Points

Start here when reading the system:

1. **Build roadmap**: `ROADMAP.md` — 10-phase implementation order with gate criteria
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

| Category | Technology | Version | Notes |
|----------|-----------|---------|-------|
| Language | Rust (MSRV) | 1.88.0 | Driven by async-nats 0.46.0 requirement |
| Runtime | Tokio | 1.49.x workspace baseline | `Cargo.toml` pins `1.49.0`; current lockfile resolves `1.50.0` |
| Messaging | async-nats (JetStream, KV, service) | 0.46.0 | jetstream, kv, object-store, service features |
| HTTP | Axum | 0.8.8 | |
| gRPC | Tonic + Prost | 0.14.x | |
| MCP | rmcp (client, server, streamable-HTTP) | 1.3.0 | `crates/mister-smith-mcp/Cargo.toml` |
| Database | sqlx (PostgreSQL, runtime-tokio-rustls) | 0.8.6 | |
| Security | jsonwebtoken, rustls | 10.x, 0.23 | JWT, TLS 1.3, mTLS |
| Observability | opentelemetry + tracing + metrics-exporter-prometheus | 0.31.0, 0.1.44, 0.18.1 | |
| CLI | clap | 4.x | |
| Serialization | serde, serde_json, rmp-serde | 1.x | |
| Errors | thiserror | 2.x | Workspace baseline is `2.0.18`; some transitive dependencies still bring `1.0.69` |
| Storage | PostgreSQL 15+ (relational), JetStream KV (distributed ephemeral) | — | |
| Orchestration | Kubernetes | — | Deploy artifacts in `deploy/` |

> See `VERSION_REFERENCE.md` for the full dependency matrix. Review `nats.rs/async-nats/` for API reference before implementing transport layer.

## Local Development Environment

**Local runtime dependencies**: repo-native Docker Compose stack under `deploy/docker-compose.yml`
- Services: `postgres` and `nats`
- Published ports: `4222` (NATS client), `8222` (NATS HTTP monitor), `5432` (PostgreSQL)
- Start: `docker compose -f deploy/docker-compose.yml up -d postgres nats`
- Recreate when the local stack drifts: `docker compose -f deploy/docker-compose.yml up -d --force-recreate postgres nats`
- The NATS container is configured with `--http_port 8222`, so local monitor endpoints such as
  `http://127.0.0.1:8222/varz` are expected to work for operator health views
- Minimum safe version: `nats-server` must be `>= v2.11.1` for CVE-2025-30215 mitigation
- Version check: `docker run --rm nats:2.12.4-alpine --version`

**NATS Rust client**: `nats.rs/` — cloned from `nats-io/nats.rs`, contains async-nats 0.46.0 source for API reference

## Available Apps (via Rube MCP)

Use Rube as the gateway whenever you need an external MCP, API, or research connection. Prefer Parallel for deeper multi-source research, and prefer Tavily for lighter search or targeted extraction.

The following apps are connected and available for use. Select the most appropriate app or tool based on the task at hand.

| App | Description |
|-----|-------------|
| **Context7 MCP** | Fetches up-to-date, version-specific documentation and code examples directly into the prompt. Use when you need accurate library/framework docs or API references. |
| **GitHub** | Code hosting and version control platform. Use for managing repositories, creating/reviewing pull requests, tracking issues, and CI/CD workflows. |
| **Linear** | Streamlined issue tracking and project planning tool. Use for managing issues, sprints, views, and project workflows with GitHub integrations. |
| **Mem0** | Self-improving memory layer for LLM applications. Use for persisting, retrieving, and managing long-term memory across agent sessions and conversations. |
| **Parallel** | Automated web research API. Use for transforming natural language queries into structured, schema-compliant research outputs at scale. |
| **Tavily** | AI-optimized search and data retrieval. Use for quickly searching the web or filtering relevant information from documents and databases. Load the Tavily-best-practices skill whenever you need to use Tavily. |

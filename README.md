# Mister Smith

A multi-agent orchestration framework for distributed AI systems, built on Rust with NATS messaging and Erlang-inspired supervision trees.

## Project Status

| Phase | Status |
|-------|--------|
| Specification | Complete — 66 files across 8 domains |
| Validation | Complete — final readiness 95/100 (Generated 2026-03-03, roadmap reconciliation addendum 2026-03-04) |
| Implementation | Not started |

## Architecture

Mister Smith coordinates distributed AI agents through three core subsystems:

**Supervision Trees** — Hierarchical fault tolerance inspired by Erlang/OTP. Supervisors manage agent lifecycles with configurable restart strategies, failure escalation, and circuit breakers.

**NATS Messaging** — High-performance pub/sub communication layer using NATS and JetStream. Supports request-response, publish-subscribe, queue groups, and blackboard coordination patterns.

**Agent Orchestration** — Nine specialized agent roles (Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory) with dynamic team composition based on task requirements.

## Technology Stack

| Component | Spec Version | Purpose |
|-----------|-------------|---------|
| Runtime | Tokio 1.49.0 (validated baseline) | Async foundation |
| Messaging | async-nats 0.46.0 | Agent communication |
| HTTP | Axum 0.8.8 | External API |
| gRPC | Tonic 0.14.5 | Internal RPC |
| Storage | PostgreSQL + Redis | Persistence + caching |
| Security | JWT, TLS 1.3, mTLS | Authentication + encryption |
| Deployment | Kubernetes | Orchestration |

## Current Residual Risks (Non-Blocking)

From `VALIDATION_REPORT.md` (Generated 2026-03-03; addendum 2026-03-04), critical and high-priority issues are resolved.
Remaining low-priority items:

1. Some links in OBSOLETE docs or planned-but-not-yet-created docs still resolve incompletely
2. A few informational `tech-framework.md` breadcrumbs remain in legacy content
3. Security-policy field naming (`priority`) can be confused with message-priority semantics
4. Minor style differences remain in selected data-integration examples

## Documentation

| Area | Entry Point |
|------|------------|
| System architecture | [`spec/core-architecture/system-architecture.md`](spec/core-architecture/system-architecture.md) |
| Agent types | [`spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) |
| Agent orchestration | [`spec/data-management/agent-orchestration.md`](spec/data-management/agent-orchestration.md) |
| Message schemas | [`spec/data-management/message-schemas.md`](spec/data-management/message-schemas.md) |
| Transport layer | [`spec/transport/transport-layer-specifications.md`](spec/transport/transport-layer-specifications.md) |
| Security framework | [`spec/security/security-framework.md`](spec/security/security-framework.md) |
| Build roadmap | [`ROADMAP.md`](ROADMAP.md) |
| Roadmap phase docs | [`plans/roadmap-phases/`](plans/roadmap-phases/) |
| Implementation plans | [`plans/IMPLEMENTATION_PLANNING_TRACKER.md`](plans/IMPLEMENTATION_PLANNING_TRACKER.md) |
| Validation report | [`VALIDATION_REPORT.md`](VALIDATION_REPORT.md) |
| Version baseline | [`VERSION_REFERENCE.md`](VERSION_REFERENCE.md) |

## Design Principles

- **Fail-fast with graceful recovery** — Detect failures quickly, recover through supervision trees
- **Event-driven architecture** — Loose coupling via pub/sub messaging
- **Resource-bounded execution** — Memory-bounded contexts, connection pooling, backpressure handling
- **Extensibility** — Middleware patterns and plugin architecture for customization

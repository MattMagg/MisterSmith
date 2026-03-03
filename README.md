# Mister Smith

A multi-agent orchestration framework for distributed AI systems, built on Rust with NATS messaging and Erlang-inspired supervision trees.

## Project Status

| Phase | Status |
|-------|--------|
| Specification | Complete — 65+ files across 8 domains |
| Validation | Complete — 60-agent assessment, 82/100 readiness score |
| Implementation | Not started |

## Architecture

Mister Smith coordinates distributed AI agents through three core subsystems:

**Supervision Trees** — Hierarchical fault tolerance inspired by Erlang/OTP. Supervisors manage agent lifecycles with configurable restart strategies, failure escalation, and circuit breakers.

**NATS Messaging** — High-performance pub/sub communication layer using NATS and JetStream. Supports request-response, publish-subscribe, queue groups, and blackboard coordination patterns.

**Agent Orchestration** — Nine specialized agent roles (Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory) with dynamic team composition based on task requirements.

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Runtime | Tokio 1.38 | Async foundation |
| Messaging | async-nats 0.34 | Agent communication |
| HTTP | Axum 0.8 | External API |
| gRPC | Tonic 0.11 | Internal RPC |
| Storage | PostgreSQL + Redis | Persistence + caching |
| Security | JWT, TLS 1.3, mTLS | Authentication + encryption |
| Deployment | Kubernetes | Orchestration |

## Known Critical Gaps

From the 60-agent validation (see `archive/validation-report.md`):

1. **Supervision trees** — Exist only as pseudocode; 0% implementation-ready
2. **Agent orchestration** — 47% readiness; coordination patterns underspecified
3. **Production safety** — 65/100; critical blockers in fault tolerance paths
4. **Kubernetes deployment** — Gaps in orchestration-specific configuration
5. **Cross-domain integration** — Compound gaps between security and transport layers
6. **Resource management** — Backpressure handling needs concrete specification

## Documentation

| Area | Entry Point |
|------|------------|
| System architecture | [`spec/core-architecture/system-architecture.md`](spec/core-architecture/system-architecture.md) |
| Agent types | [`spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) |
| Agent orchestration | [`spec/data-management/agent-orchestration.md`](spec/data-management/agent-orchestration.md) |
| Message schemas | [`spec/data-management/message-schemas.md`](spec/data-management/message-schemas.md) |
| Transport layer | [`spec/transport/transport-layer-specifications.md`](spec/transport/transport-layer-specifications.md) |
| Security framework | [`spec/security/security-framework.md`](spec/security/security-framework.md) |
| Implementation plans | [`plans/IMPLEMENTATION_PLANNING_TRACKER.md`](plans/IMPLEMENTATION_PLANNING_TRACKER.md) |
| Validation report | [`archive/validation-report.md`](archive/validation-report.md) |

## Design Principles

- **Fail-fast with graceful recovery** — Detect failures quickly, recover through supervision trees
- **Event-driven architecture** — Loose coupling via pub/sub messaging
- **Resource-bounded execution** — Memory-bounded contexts, connection pooling, backpressure handling
- **Extensibility** — Middleware patterns and plugin architecture for customization

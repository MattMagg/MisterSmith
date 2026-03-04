# Mister Smith

Multi-agent orchestration framework — Rust + NATS + supervision trees. Currently in specification form; no implementation code exists yet.

## Repository Structure

| Directory | Contents |
|-----------|----------|
| `spec/` | Framework specifications — 65+ files across 8 domains |
| `plans/` | Implementation plans — batch 1 (core architecture) complete, batch 2 partial |
| `archive/` | Completed validation work, historical operations, and research |
| `nats.rs/` | Official NATS Rust client (cloned from nats-io/nats.rs) — reference for async-nats API |
| `.github/workflows/` | CI/CD pipelines for documentation validation |
| `logs/` | Session logs |

## Key Entry Points

Start here when reading the framework:

1. **Architecture overview**: `spec/core-architecture/system-architecture.md`
2. **Component design**: `spec/core-architecture/component-architecture.md`
3. **Agent types and orchestration**: `spec/data-management/agent-orchestration.md`
4. **Message contracts**: `spec/data-management/message-schemas.md`
5. **Type system**: `spec/core-architecture/type-definitions.md`

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

| Component | Spec Version | Current Version | Notes |
|-----------|-------------|-----------------|-------|
| Runtime | Tokio 1.38 | — | Check crates.io before implementation |
| Messaging | async-nats 0.34 | **0.46.0** | Major version gap — API changes likely |
| HTTP | Axum 0.8 | — | |
| gRPC | Tonic 0.11 | — | |
| Storage | PostgreSQL + Redis | — | |
| Security | JWT, TLS 1.3, mTLS | — | |
| Orchestration | Kubernetes | — | |

> **Version drift**: Specs were written against async-nats 0.34. The current release is 0.46.0 (Rust edition 2021, min rustc 1.88.0). API surface has changed — review `nats.rs/async-nats/` before implementing transport layer specs.

## Local Development Environment

**NATS server**: Docker container `NATS` running nats-server v2.12.4
- Ports: 4222 (client), 6222 (cluster), 8222 (monitoring) — container-internal only, not published to host
- Start: `docker start NATS`
- To publish ports: `docker run -d --name NATS -p 4222:4222 -p 8222:8222 nats:latest`

**NATS Rust client**: `nats.rs/` — cloned from `nats-io/nats.rs`, contains async-nats 0.46.0 source for API reference

## Validation Summary

A 60-agent validation operation assessed the framework (see `archive/validation-report.md`):

- **Overall readiness**: 82/100
- **Documentation quality**: 97/100
- **Production readiness**: NOT APPROVED — 6 critical gaps remain
- **Critical gaps**: Supervision trees (pseudocode only), agent orchestration (47% readiness)
- **Estimated implementation effort**: 184 developer-weeks, 20-24 week timeline

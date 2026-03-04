# Mister Smith

Multi-agent orchestration framework — Rust + NATS + supervision trees. Model-agnostic (works with any LLM). Currently in specification form; no implementation code exists yet.

## Repository Structure

| Directory | Contents |
|-----------|----------|
| `ROADMAP.md` | 8-phase build roadmap — dependency-aware implementation order |
| `VALIDATION_REPORT.md` | Latest validation assessment (95/100 readiness) |
| `VERSION_REFERENCE.md` | Crate version matrix — pinned versions for all dependencies |
| `spec/` | Framework specifications — 65+ files across 8 domains |
| `plans/` | Implementation plans — batch 1 (core architecture) 7 of 8 agents complete, batch 2 partial |
| `archive/` | Completed validation work, historical operations, and research |
| `nats.rs/` | Official NATS Rust client (cloned from nats-io/nats.rs) — reference for async-nats API |
| `.github/workflows/` | CI/CD pipelines for documentation validation |
| `logs/` | Session logs |

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

## Validation Summary

A multi-agent validation operation assessed the framework (see `VALIDATION_REPORT.md`):

- **Overall readiness**: 95/100
- **Documentation quality**: 97/100
- **Critical gaps resolved**: Version alignment, type reconciliation, terminology generalization
- **Remaining risk**: Supervision trees (pseudocode → production Rust is the hardest phase)

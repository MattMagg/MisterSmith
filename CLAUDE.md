# Mister Smith

Multi-agent orchestration framework — Rust + NATS + supervision trees. Currently in specification form; no implementation code exists yet.

## Repository Structure

| Directory | Contents |
|-----------|----------|
| `spec/` | Framework specifications — 65+ files across 8 domains |
| `plans/` | Implementation plans — batch 1 (core architecture) complete, batch 2 partial |
| `archive/` | Completed validation work, historical operations, and research |
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

| Component | Technology |
|-----------|-----------|
| Runtime | Tokio 1.38 |
| Messaging | async-nats 0.34 (NATS + JetStream) |
| HTTP | Axum 0.8 |
| gRPC | Tonic 0.11 |
| Storage | PostgreSQL + Redis |
| Security | JWT, TLS 1.3, mTLS |
| Orchestration | Kubernetes |

## Validation Summary

A 60-agent validation operation assessed the framework (see `archive/validation-report.md`):

- **Overall readiness**: 82/100
- **Documentation quality**: 97/100
- **Production readiness**: NOT APPROVED — 6 critical gaps remain
- **Critical gaps**: Supervision trees (pseudocode only), agent orchestration (47% readiness)
- **Estimated implementation effort**: 184 developer-weeks, 20-24 week timeline

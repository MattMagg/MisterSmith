# Implementation Plan: Phase 5 Security

**Branch**: `005-phase5-security` | **Date**: 2026-03-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/005-phase5-security/spec.md`

## Summary

Implement JWT authentication, RBAC authorization, TLS/mTLS certificate management, transport security middleware, and security audit logging as a single `mister-smith-security` crate with feature flags (`jwt`, `rbac`, `tls`, `audit`). The security layer integrates with existing Phase 4 transport middleware (Axum, Tonic, NATS) and Phase 2 EventBus for audit event publication.

## Technical Context

**Language/Version**: Rust, MSRV 1.88.0
**Primary Dependencies**:
- `jsonwebtoken` 10.3.0 (feature: `aws_lc_rs`) — JWT encoding/decoding
- `rustls` 0.23.37 — TLS 1.3 implementation
- `tokio-rustls` 0.26.4 — Async TLS wrapper
- `rcgen` 0.14.7 (feature: `pem`) — Dev/test certificate generation
- `rustls-pki-types` 1.x — Certificate/key types (`CertificateDer`, `PrivateKeyDer`)
- `dashmap` 6.x — Concurrent hash maps for revocation lists, rate limiters
- `sha2` 0.10.x — Hash chain for audit tamper-evidence
- `arc-swap` 1.x — Zero-downtime certificate reload
- Existing workspace: `serde`, `serde_json`, `chrono`, `uuid`, `tokio`, `tracing`, `parking_lot`
**Storage**: In-memory (audit persistence deferred to Phase 6)
**Testing**: `cargo test` with `#[tokio::test]` for async tests; `rcgen` for test certificate generation
**Target Platform**: Linux server (same as framework)
**Project Type**: Library crate (workspace member)
**Performance Goals**: <1ms token validation overhead, <500µs authorization decisions (cached)
**Constraints**: All subsystems independently toggleable; master switch disables all when off
**Scale/Scope**: Single-process in-memory; distributed rate limiting out of scope

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Canonical Single Source | PASS | Security types defined in `mister-smith-security` crate only. `SecurityError` added to `SystemError` in core via `#[from]`. |
| II. Spec-First Design | PASS | 7 security spec files in `spec/security/`, feature spec in `specs/005-phase5-security/spec.md`, 5 contracts defined. |
| III. Phase-Gated Build Order | PASS | Phase 5 depends on Phases 1-4 (all complete). Gate 4 satisfied: transport endpoints accept requests with pluggable security middleware. |
| IV. Model-Agnostic | PASS | Security layer authenticates agents generically — no LLM provider references. |
| V. Erlang/OTP Fault Tolerance | PASS | Security failures don't crash actors — they produce error responses. SecurityConfig disables subsystems gracefully. |
| VI. Evidence-Based Validation | PASS | Gate 5 criteria are testable: JWT validation, auth middleware rejection, mTLS handshake, audit logging. |
| VII. Explicit Dependencies | PASS | All new dependencies documented in research.md with exact versions from VERSION_REFERENCE.md. |

**Post-Design Re-check**: No violations. Single crate with feature flags keeps the dependency graph simple.

## Project Structure

### Documentation (this feature)

```text
specs/005-phase5-security/
├── plan.md                        # This file
├── spec.md                        # Feature specification
├── research.md                    # Phase 0 research output
├── data-model.md                  # Phase 1 entity definitions
├── quickstart.md                  # Phase 1 integration scenarios
├── checklists/
│   └── requirements.md            # Spec quality checklist
├── contracts/
│   ├── jwt-manager.md             # JwtManager API contract
│   ├── policy-engine.md           # PolicyEngine API contract
│   ├── certificate-manager.md     # CertificateManager API contract
│   ├── security-middleware.md     # Middleware/interceptor contracts
│   └── audit-logger.md            # AuditLogger API contract
└── tasks.md                       # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/mister-smith-security/
├── Cargo.toml                     # Feature flags: jwt, rbac, tls, audit (all default)
├── src/
│   ├── lib.rs                     # Public API re-exports, feature-gated modules
│   ├── config.rs                  # SecurityConfig expansion (JwtConfig, RbacConfig, TlsConfig, AuditConfig)
│   ├── error.rs                   # SecurityError enum
│   ├── jwt/
│   │   ├── mod.rs                 # JwtManager
│   │   ├── claims.rs              # AgentClaims, TokenPair
│   │   └── keys.rs                # KeySource, key loading
│   ├── rbac/
│   │   ├── mod.rs                 # PolicyEngine
│   │   ├── permission.rs          # Permission, Role types
│   │   └── constraints.rs         # PolicyConstraints, TimeWindow, ABAC conditions
│   ├── tls/
│   │   ├── mod.rs                 # CertificateManager
│   │   ├── config_builder.rs      # ServerConfig/ClientConfig construction
│   │   └── dev_certs.rs           # Self-signed certificate generation (rcgen)
│   ├── middleware/
│   │   ├── mod.rs                 # SecurityLayer composition root
│   │   ├── axum.rs                # auth_middleware, AuthenticatedAgent extractor
│   │   ├── tonic.rs               # grpc_auth_interceptor
│   │   ├── nats.rs                # SecureTransport wrapper
│   │   └── rate_limiter.rs        # Token-bucket rate limiter
│   └── audit/
│       ├── mod.rs                 # AuditLogger
│       └── events.rs              # SecurityAuditEvent, AuditEventType, AuditOutcome
└── tests/
    ├── jwt_tests.rs               # JWT generation, validation, refresh, revocation
    ├── rbac_tests.rs              # Policy evaluation, role hierarchy, deny-wins
    ├── tls_tests.rs               # Certificate loading, mTLS, TLS 1.3 enforcement
    ├── middleware_tests.rs        # HTTP middleware, gRPC interceptor
    └── audit_tests.rs            # Event recording, hash chain, alerts

# Modified existing crates:
crates/mister-smith-core/src/error.rs           # Add SecurityError variant to SystemError
crates/mister-smith-config/src/types.rs          # Expand SecurityConfig with subsystem configs
crates/mister-smith-http/src/middleware.rs        # Wire auth_middleware (replace pass-through)
crates/mister-smith-grpc/src/server.rs            # Add auth interceptor to service builder
crates/mister-smith-integration-tests/            # Security integration tests
```

**Structure Decision**: Single `mister-smith-security` crate following the established workspace pattern. Feature flags (`jwt`, `rbac`, `tls`, `audit`) allow consumers to compile only needed subsystems. All features enabled by default. This adds 1 new crate to the workspace (bringing total to 16).

## Complexity Tracking

No constitution violations to justify. Single crate with feature flags is the simplest structure that satisfies the independent-toggle requirement.

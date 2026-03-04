# Phase 5: Security

## Purpose and Scope

Define identity, authorization, and encrypted transport requirements across all runtime surfaces.
Phase 5 enforces security on transport and agent operations introduced in earlier phases.

### In Scope

- Authentication token model and lifecycle
- Authorization policy model and enforcement points
- TLS/mTLS and certificate lifecycle requirements
- Security integration boundaries with transport and operations

### Out of Scope

- New transport protocol design
- Persistence schema design

## Inputs and Dependencies

### Upstream Dependencies

- Phase 1 (security-related types and configuration primitives)
- Integration dependency: Phase 4 transport surfaces for enforcement points

### Key Source Inputs

- `ROADMAP.md` Phase 5 and Gate 5
- `VERSION_REFERENCE.md` security versions (jsonwebtoken 10.3.0, rustls 0.23.37)
- `VALIDATION_REPORT.md` model-agnostic and naming-reconciliation outcomes

### Required Specification Anchors

- `spec/security/authentication-specifications.md`
- `spec/security/authentication-implementation.md`
- `spec/security/authorization-specifications.md`
- `spec/security/authorization-implementation.md`
- `spec/security/security-framework.md`
- `spec/security/security-integration.md`
- `spec/security/security-patterns.md`

## Outputs and Downstream Consumers

### Produces

- Authentication contract (JWT/API key claims model)
- Authorization model and transport enforcement points
- TLS/mTLS requirements and certificate handling model

### Consumed By

- Phase 4 transport endpoints for security enforcement completion
- Phase 6 persistence credentials/secrets handling
- Phase 7 tool and agent permission enforcement
- Phase 8 production hardening and operational controls

## Gate Criteria and Validation

### Gate Criteria

- AuthN/AuthZ specs define unambiguous claim and permission semantics
- Transport enforcement points are explicit for HTTP and gRPC
- mTLS requirements for NATS and service-to-service channels are documented
- Security naming is reconciled with domain-specific enum/role models

### Validation Approach

- Confirm security-policy docs align with transport-integration docs
- Verify JWT/TLS versions match `VERSION_REFERENCE.md`
- Check for stale model-specific terminology that contradicts current guidance

### Validation Evidence

- Auth flow consistency across authentication and integration specs
- Explicit mapping from security policy to transport enforcement boundaries

## Official-Doc Best Practices

- Use jsonwebtoken v10 with explicit crypto backend configuration and claim validation requirements ([jsonwebtoken 10.3.0](https://docs.rs/jsonwebtoken/10.3.0/jsonwebtoken/)).
- Keep TLS/mTLS guidance aligned with rustls 0.23 typed certificate/key APIs and verifier model ([rustls 0.23.37](https://docs.rs/rustls/0.23.37/rustls/)).
- Treat TLS 1.3 as baseline for service-to-service channels when documenting production posture ([RFC 8446](https://www.rfc-editor.org/rfc/rfc8446)).

## Known Risks / Unknowns

### Risks

- Divergent role/type naming can reintroduce cross-domain ambiguity
- mTLS and certificate-rotation behavior can be underspecified for incidents
- Policy field naming (`priority`) can be confused with message-priority semantics

### Required Follow-ups

- Clarify policy-order semantics if naming ambiguity impacts implementation planning
- Revalidate transport-enforcement references when security interfaces change

## Authoritative Spec Files

- `spec/security/security-framework.md`
- `spec/security/security-integration.md`
- `spec/security/security-patterns.md`
- `spec/security/authentication-specifications.md`
- `spec/security/authentication-implementation.md`
- `spec/security/authorization-specifications.md`
- `spec/security/authorization-implementation.md`

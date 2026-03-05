# Feature Specification: Security

**Feature Branch**: `005-phase5-security`
**Created**: 2026-03-04
**Status**: Draft
**Input**: User description: "Phase 5: Security — JWT authentication (jsonwebtoken 10), RBAC authorization with Axum/Tonic middleware, TLS/mTLS with rustls 0.23"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - JWT Authentication for Agents (Priority: P1)

As a framework developer, I can authenticate agents and external clients using JWT tokens so that only verified identities can publish, subscribe, and invoke operations within the framework.

The authentication system generates and validates JWT tokens containing agent-specific claims: agent ID, agent type, capabilities, permissions, session ID, and delegation chain. Tokens use configurable signing algorithms (RS256/ES256/HS256) and include standard JWT claims (iss, sub, aud, exp, nbf, iat, jti). Token generation produces access tokens with short TTL and refresh tokens with longer TTL. Validation checks signature, expiration, audience, issuer, and optionally a token revocation list. The system provides a `JwtManager` that handles key management, token lifecycle, and claim extraction.

**Why this priority**: Authentication is the foundation of the security layer. Without identity verification, authorization cannot function, TLS client certificates cannot be mapped to identities, and audit trails have no principal to attribute events to.

**Independent Test**: Can be fully tested by generating tokens with known claims, validating them with the corresponding key, verifying expiration and revocation behavior, and confirming invalid/expired tokens are rejected — all without external services.

**Acceptance Scenarios**:

1. **Given** an agent with a known ID and type, **When** the framework generates a JWT token, **Then** the token contains all standard claims (iss, sub, aud, exp, iat, jti) and agent-specific claims (agent_id, agent_type, capabilities, permissions).
2. **Given** a valid JWT token, **When** the framework validates it, **Then** the claims are extracted and the agent identity is established.
3. **Given** an expired JWT token, **When** the framework validates it, **Then** validation fails with an expiration error.
4. **Given** a token signed with the wrong key, **When** the framework validates it, **Then** validation fails with a signature error.
5. **Given** a valid access token nearing expiration and a valid refresh token, **When** the client requests a token refresh, **Then** a new access token is issued with a fresh expiration while the refresh token remains valid.
6. **Given** a token that has been revoked, **When** the framework validates it, **Then** validation fails with a revocation error.
7. **Given** a JWT configuration with RS256 algorithm, **When** the `JwtManager` is constructed, **Then** it loads the RSA key pair and can generate and validate tokens using that algorithm.

---

### User Story 2 - RBAC Authorization & Permission Checking (Priority: P2)

As a framework developer, I can enforce role-based access control on all agent operations so that agents can only access resources and perform actions that their assigned roles permit.

The authorization system implements a policy engine that evaluates permissions based on roles assigned to principals (agents or external clients). Permissions follow an `action:resource:scope` syntax (e.g., `read:agent:own`, `write:task:tenant`, `admin:system:all`). Roles form a hierarchy where higher-level roles inherit permissions from lower-level roles. The policy engine supports both RBAC (role-based) and optional ABAC (attribute-based) conditions such as time windows, IP ranges, and resource ownership. Explicit deny policies always override allow policies. All authorization decisions are logged for audit.

**Why this priority**: Authorization gates every protected operation. Once agents can authenticate (US1), they need authorization to determine what they are allowed to do. This is required before wiring security into transport middleware.

**Independent Test**: Can be tested by creating roles with specific permissions, constructing authorization requests, and verifying that the policy engine correctly allows or denies access based on role membership, resource ownership, and attribute conditions.

**Acceptance Scenarios**:

1. **Given** an agent with the "worker" role, **When** it requests to read a task it owns, **Then** the policy engine allows the request.
2. **Given** an agent with the "worker" role, **When** it requests to delete a system resource, **Then** the policy engine denies the request.
3. **Given** an agent with the "admin" role, **When** it requests any action on any resource, **Then** the policy engine allows the request.
4. **Given** an explicit deny policy for a resource, **When** an agent with an allow policy for the same resource makes a request, **Then** the deny policy takes precedence and the request is denied.
5. **Given** an agent with no matching role or permission, **When** it makes any request, **Then** the policy engine denies with a default-deny decision.
6. **Given** a role hierarchy where "developer" inherits from "contributor" which inherits from "viewer", **When** a developer makes a request, **Then** all inherited permissions from contributor and viewer are also evaluated.
7. **Given** a policy with a time-window condition restricting access to business hours, **When** an agent requests access outside those hours, **Then** the request is denied even if the role would otherwise permit it.

---

### User Story 3 - Transport Security Middleware (Priority: P3)

As a framework developer, I can enforce authentication and authorization on HTTP, gRPC, and NATS transports so that every request across all transport layers is verified before processing.

The security middleware integrates with each Phase 4 transport to enforce authentication and authorization. For HTTP (Axum), the middleware extracts Bearer tokens from the Authorization header, validates them via the `JwtManager`, and attaches the authenticated identity to the request context for downstream authorization checks. For gRPC (Tonic), an interceptor performs the same flow using gRPC metadata. For NATS, the framework validates agent identity at connection time and enforces per-subject publish/subscribe permissions. Each transport middleware applies rate limiting and logs security events (auth success, auth failure, permission denied). Middleware is composable — authentication runs first, then authorization.

**Why this priority**: This is where the security layer meets the transport layer. Without middleware enforcement, authentication and authorization are defined but not applied, leaving the system unprotected.

**Independent Test**: Can be tested by starting an HTTP server with auth middleware, sending requests with valid/invalid/missing tokens, and verifying correct accept/reject behavior. gRPC interceptor can be tested similarly. NATS permissions can be tested with subject-level access checks.

**Acceptance Scenarios**:

1. **Given** an HTTP request with a valid Bearer token, **When** the auth middleware processes it, **Then** the request is allowed and the agent identity is available in the request context.
2. **Given** an HTTP request without an Authorization header, **When** the auth middleware processes it, **Then** the request is rejected with 401 Unauthorized.
3. **Given** an authenticated HTTP request for a protected endpoint, **When** the agent lacks the required permission, **Then** the request is rejected with 403 Forbidden.
4. **Given** a gRPC call with a valid token in metadata, **When** the auth interceptor processes it, **Then** the call proceeds with the identity attached to the request context.
5. **Given** a gRPC call with an expired token, **When** the auth interceptor processes it, **Then** the call is rejected with UNAUTHENTICATED status.
6. **Given** an agent publishing to a NATS subject it is not authorized for, **When** the publish is attempted, **Then** the framework rejects the publish with a permission denied error.
7. **Given** a rapid sequence of authentication failures from one source, **When** the rate limiter threshold is exceeded, **Then** subsequent requests are rejected with 429 Too Many Requests.

---

### User Story 4 - TLS & Certificate Management (Priority: P4)

As a framework developer, I can configure TLS for all transport connections and enable mutual TLS (mTLS) for service-to-service authentication so that all communication is encrypted and endpoints can cryptographically verify each other's identity.

The TLS system provides a `CertificateManager` that loads, validates, and manages TLS certificates for server and client connections. It enforces TLS 1.3 as the minimum protocol version across all transports. For mTLS, the manager configures client certificate verification using a trusted CA certificate. The certificate manager integrates with rustls to produce `ServerConfig` and `ClientConfig` objects consumed by Axum, Tonic, and async-nats. It supports certificate generation for development/testing (via rcgen), certificate rotation with zero-downtime reload, and certificate expiration monitoring. NATS connections use mTLS for agent-to-server authentication.

**Why this priority**: TLS encryption protects data in transit. mTLS provides cryptographic identity verification that complements JWT authentication. However, the system can operate with JWT-only authentication during development, so TLS is lower priority than auth and authz.

**Independent Test**: Can be tested by generating self-signed CA and leaf certificates, configuring TLS servers and clients, establishing connections, verifying mTLS handshake success/failure, and checking certificate expiration detection — all using localhost connections.

**Acceptance Scenarios**:

1. **Given** a server certificate and private key, **When** the `CertificateManager` loads them, **Then** a valid `ServerConfig` is produced with TLS 1.3 minimum enforced.
2. **Given** a client CA certificate, **When** the `CertificateManager` configures a client, **Then** the client verifies the server certificate against the CA.
3. **Given** mTLS is enabled, **When** a client connects without a client certificate, **Then** the server rejects the connection.
4. **Given** mTLS is enabled, **When** a client presents a valid certificate signed by the trusted CA, **Then** the connection is established and the client identity is extracted from the certificate.
5. **Given** a certificate nearing expiration (within 30 days), **When** the certificate manager checks health, **Then** a warning event is emitted with the certificate subject and days until expiration.
6. **Given** new certificates are placed on disk, **When** a reload is triggered, **Then** the server begins using the new certificates for new connections without dropping existing connections.
7. **Given** a development environment, **When** the framework generates self-signed certificates, **Then** CA, server, and client certificates are created with appropriate key usage extensions.

---

### User Story 5 - Security Audit Logging (Priority: P5)

As a framework developer, I can capture all security-relevant events in a structured audit log so that authentication attempts, authorization decisions, and security incidents are recorded for compliance and forensics.

The audit system captures security events with a consistent structure: event ID, timestamp, event type (authentication, authorization, system access, suspicious activity), principal, resource, action, outcome (success, failure, blocked), and contextual details. Events are published through the Phase 2 EventBus as well as written to a structured audit trail. The audit log supports tamper-evidence via hash chaining. Critical security events (repeated auth failures, privilege escalation attempts) trigger real-time alerts.

**Why this priority**: Audit logging is essential for compliance and incident response, but it enhances rather than enables the core security functions. The system is functional without audit logs, making this the lowest priority.

**Independent Test**: Can be tested by triggering auth success/failure events, verifying audit entries are created with correct structure, checking hash chain integrity, and confirming alert thresholds trigger notifications.

**Acceptance Scenarios**:

1. **Given** a successful authentication, **When** the audit system records it, **Then** the audit entry contains event type "authentication", outcome "success", the principal identity, and a timestamp.
2. **Given** a denied authorization decision, **When** the audit system records it, **Then** the audit entry contains the denied action, target resource, applicable policy, and requesting principal.
3. **Given** a sequence of 5 failed authentication attempts from the same source within 1 minute, **When** the audit system detects this pattern, **Then** a "suspicious activity" alert event is emitted.
4. **Given** a series of audit entries, **When** the hash chain is verified, **Then** each entry's hash incorporates the previous entry's hash, and tampering with any entry is detectable.
5. **Given** audit logging is enabled, **When** any middleware rejects a request, **Then** the rejection reason, source, and timestamp are captured in the audit trail.

---

### Edge Cases

- What happens when the JWT signing key is rotated while tokens signed with the old key are still valid? The system must support validation against both old and new keys during a configurable overlap period.
- What happens when the token revocation list grows very large? The system should use a time-bounded revocation list (only track revocations for tokens that haven't yet expired naturally).
- What happens when a certificate expires mid-connection? Existing connections continue on the previously negotiated TLS session; only new connections are affected.
- What happens when the RBAC policy store is unavailable? The system defaults to deny-all for safety, and emits a health degradation event.
- What happens when authorization evaluation exceeds the time budget? The policy engine should have a maximum evaluation timeout, defaulting to deny on timeout.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST generate JWT tokens with configurable algorithms (RS256, ES256, HS256) containing both standard (RFC 7519) and agent-specific custom claims.
- **FR-002**: System MUST validate JWT tokens by checking signature, expiration, not-before, audience, and issuer claims.
- **FR-003**: System MUST support token refresh — issuing new access tokens using a valid refresh token without re-authentication.
- **FR-004**: System MUST support token revocation — maintaining a revocation list and rejecting tokens that appear on it.
- **FR-005**: System MUST implement RBAC with hierarchical roles where child roles inherit parent permissions.
- **FR-006**: System MUST evaluate permissions using `action:resource:scope` syntax with wildcard support.
- **FR-007**: System MUST enforce explicit-deny-wins policy conflict resolution — any deny policy overrides all allow policies.
- **FR-008**: System MUST provide Axum middleware that extracts Bearer tokens, validates them, and injects the authenticated identity into the request context.
- **FR-009**: System MUST provide Tonic interceptor that extracts tokens from gRPC metadata and performs the same validation flow.
- **FR-010**: System MUST enforce per-subject publish/subscribe permissions for NATS transport operations at the application level, using the RBAC policy engine to check permissions before invoking transport publish/subscribe calls.
- **FR-011**: System MUST load and manage TLS certificates using rustls, enforcing TLS 1.3 as the minimum protocol version.
- **FR-012**: System MUST support mutual TLS (mTLS) — verifying client certificates against a trusted CA and extracting client identity.
- **FR-013**: System MUST support certificate rotation with zero-downtime by allowing hot reload of certificates without dropping existing connections.
- **FR-014**: System MUST generate self-signed certificates for development/testing environments using rcgen.
- **FR-015**: System MUST record all authentication attempts (success and failure), authorization decisions, and security-relevant events in a structured audit log.
- **FR-016**: System MUST apply rate limiting to authentication endpoints to prevent brute-force attacks.
- **FR-017**: System MUST provide a `SecurityConfig` struct (extending the existing placeholder) with a top-level `enabled` master switch and independent `enabled` flags for each subsystem (auth, authz, tls, audit), allowing incremental adoption.
- **FR-018**: System MUST integrate with the existing Phase 2 EventBus to publish security events (auth failures, permission denials, certificate warnings).

### Key Entities

- **AgentClaims**: JWT claim set containing standard RFC 7519 fields plus agent-specific extensions (agent_id, agent_type, capabilities, permissions, session_id, delegation_chain).
- **JwtManager**: Service responsible for token generation, validation, refresh, and revocation. Holds encoding/decoding keys and validation parameters.
- **Permission**: Triple of action, resource pattern, and optional constraints that defines what an agent can do.
- **Role**: Named collection of permissions with optional parent role for hierarchical inheritance.
- **PolicyEngine**: Authorization decision point that collects applicable policies, evaluates conditions, resolves conflicts (deny-wins), and returns allow/deny decisions.
- **CertificateManager**: Service that loads, validates, monitors, and rotates TLS certificates. Produces rustls configurations consumed by transports.
- **SecurityAuditEvent**: Structured audit entry capturing security-relevant events with type, principal, resource, action, outcome, and contextual details.
- **SecurityMiddleware**: Transport-specific middleware/interceptors that enforce authentication and authorization on HTTP, gRPC, and NATS requests.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All authenticated agent operations complete with less than 1ms overhead from token validation (excluding network latency).
- **SC-002**: Authorization decisions are rendered in under 500 microseconds for cached policies.
- **SC-003**: 100% of transport endpoints (HTTP, gRPC, NATS) enforce authentication when security is enabled, except for endpoints explicitly configured as exempt (health endpoints default to unauthenticated but can be locked down via configuration).
- **SC-004**: The system correctly rejects 100% of expired, revoked, or malformed tokens with appropriate error codes.
- **SC-005**: TLS connections enforce TLS 1.3 minimum — connections attempting TLS 1.2 or lower are refused.
- **SC-006**: Certificate rotation completes without dropping any existing connections or causing request failures.
- **SC-007**: All security events (auth success, auth failure, permission denied, certificate warnings) appear in the audit log within 1 second of occurrence.
- **SC-008**: The security system passes all tests (unit + integration) with zero failures across the full workspace.

### Assumptions

- The existing `SecurityConfig` placeholder in `mister-smith-config` will be expanded with a top-level `enabled` master switch and independent `enabled` flags for each subsystem (auth, authz, tls, audit), maintaining backward compatibility with existing config files that omit the security section (via `#[serde(default)]`). When the master switch is off, all subsystems are disabled regardless of individual flags.
- RS256 is the default signing algorithm unless otherwise configured, as it provides strong security with broad ecosystem compatibility.
- Token TTL defaults: access tokens 15 minutes, refresh tokens 24 hours — configurable via `SecurityConfig`.
- The RBAC system starts with a default set of roles (admin, developer, operator, viewer) that can be extended via configuration.
- Rate limiting uses in-memory token bucket algorithm; distributed rate limiting across multiple instances is out of scope for Phase 5.
- Certificate generation for development uses RSA 4096-bit keys with self-signed CA; production environments provide their own certificates.
- Audit log storage is in-memory with EventBus publication; persistent audit storage (database) is deferred to Phase 6.
- ABAC (attribute-based) conditions are supported but optional — pure RBAC is sufficient for the initial implementation.
- Security is implemented as a single `mister-smith-security` crate with feature flags (`jwt`, `rbac`, `tls`, `audit`) for selective compilation. All features are enabled by default. This keeps the dependency graph simple while allowing consumers to opt out of unused subsystems.

## Clarifications

### Session 2026-03-04

- Q: Should health check endpoints be exempt from authentication? → A: Configurable per-endpoint — health endpoints default to unauthenticated but can be locked down via configuration.
- Q: Should security subsystems activate together or independently? → A: Independent toggles — each subsystem (auth, authz, tls, audit) has its own `enabled` flag, with a top-level `enabled` as a master switch.
- Q: How should the security crate(s) be structured? → A: Single `mister-smith-security` crate with feature flags (`jwt`, `rbac`, `tls`, `audit`) for selective compilation.
- Q: Where should NATS subject permissions be enforced? → A: Application-level only — framework checks RBAC permissions before NATS operations. Server-level NATS ACLs are out of scope for Phase 5.

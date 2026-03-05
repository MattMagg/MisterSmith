# Tasks: Phase 5 Security

**Input**: Design documents from `/specs/005-phase5-security/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included — security is a correctness-critical subsystem and the spec defines explicit acceptance scenarios (SC-008: "passes all tests with zero failures").

**Organization**: Tasks are grouped by user story (US1–US5) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **New crate**: `crates/mister-smith-security/src/`
- **Tests**: `crates/mister-smith-security/tests/`
- **Modified crates**: `crates/mister-smith-core/`, `crates/mister-smith-config/`, `crates/mister-smith-http/`, `crates/mister-smith-grpc/`
- **Integration tests**: `crates/mister-smith-integration-tests/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the `mister-smith-security` crate and add workspace dependencies

- [ ] T001 Add security dependencies to workspace `Cargo.toml`: `jsonwebtoken = { version = "10.3.0", features = ["aws_lc_rs"] }`, `rustls = "0.23.37"`, `tokio-rustls = "0.26.4"`, `rcgen = { version = "0.14.7", features = ["pem"] }`, `rustls-pki-types = "1"`, `sha2 = "0.10"`, `arc-swap = "1"`, `hex = "0.4"`
- [ ] T002 Create crate directory structure per plan.md: `crates/mister-smith-security/` with `src/jwt/`, `src/rbac/`, `src/tls/`, `src/middleware/`, `src/audit/`, and `tests/`
- [ ] T003 Create `crates/mister-smith-security/Cargo.toml` with feature flags `jwt` (default), `rbac` (default), `tls` (default), `audit` (default) and workspace dependency references

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core error types, config expansion, and module scaffolding that MUST be complete before any user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Add `SecurityError` enum to `crates/mister-smith-core/src/error.rs` with variants: `AuthenticationFailed(String)`, `TokenExpired`, `TokenRevoked`, `InvalidSignature`, `InvalidToken(String)`, `AuthorizationDenied(String)`, `InsufficientPermissions(String)`, `CertificateLoadFailed(String)`, `CertificateGenerationFailed(String)`, `TlsConfigFailed(String)`, `KeyLoadFailed(String)`, `TokenGenerationFailed(String)`, `RateLimited(Duration)`. Add `Security(#[from] SecurityError)` variant to `SystemError`. Export from `crates/mister-smith-core/src/lib.rs`.
- [ ] T005 Expand `SecurityConfig` in `crates/mister-smith-config/src/types.rs`: add nested `auth: AuthConfig`, `authz: AuthzConfig`, `tls: TlsSecurityConfig`, `audit: AuditSecurityConfig` structs with `#[serde(default)]` on all fields. Add master `enabled` bool and independent `enabled` bools per subsystem. Preserve backward compatibility with existing config files.
- [ ] T006 Create `crates/mister-smith-security/src/lib.rs` with `#![deny(missing_docs, unsafe_code)]`, feature-gated module declarations (`#[cfg(feature = "jwt")] pub mod jwt;`, etc.), and public re-exports
- [ ] T007 Create `crates/mister-smith-security/src/error.rs` — re-export `SecurityError` from `mister-smith-core` and add any crate-internal error conversions (e.g., `From<jsonwebtoken::errors::Error>`)
- [ ] T008 Create `crates/mister-smith-security/src/config.rs` with `JwtConfig`, `RbacConfig`, `TlsConfig`, `AuditConfig` structs as defined in data-model.md, including `KeySource` enum, `TlsVersion` enum, and `Default` impls

**Checkpoint**: Foundation ready — `cargo build -p mister-smith-security` compiles with empty feature modules, `cargo test -p mister-smith-core` passes with new SecurityError

---

## Phase 3: User Story 1 — JWT Authentication (Priority: P1) 🎯 MVP

**Goal**: Authenticate agents and external clients using JWT tokens with configurable algorithms (RS256/ES256/HS256), token refresh, and revocation

**Independent Test**: Generate tokens with known claims, validate with corresponding keys, verify expiration/revocation behavior, confirm invalid/expired tokens rejected — all without external services

### Implementation for User Story 1

- [ ] T009 [P] [US1] Create `crates/mister-smith-security/src/jwt/claims.rs` — `AgentClaims` struct (Serialize + Deserialize) with standard JWT fields (iss, sub, aud, exp, nbf, iat, jti) and agent-specific fields (agent_id, agent_type, capabilities, permissions, session_id, delegation_chain). Implement `Default`. Create `TokenPair` struct (access_token, refresh_token, token_type, expires_in).
- [ ] T010 [P] [US1] Create `crates/mister-smith-security/src/jwt/keys.rs` — `KeySource` enum (Hmac, RsaPem, EcPem, EdPem) with key loading methods. Implement `load_encoding_key(&self) -> Result<EncodingKey, SecurityError>` and `load_decoding_key(&self) -> Result<DecodingKey, SecurityError>` using jsonwebtoken 10 API (`EncodingKey::from_rsa_pem`, `from_ec_pem`, `from_secret`, etc.).
- [ ] T011 [US1] Implement `JwtManager` in `crates/mister-smith-security/src/jwt/mod.rs` — `new(config: &JwtConfig)`, `generate_token_pair(&self, claims: &AgentClaims) -> Result<TokenPair>`, `validate_token(&self, token: &str) -> Result<AgentClaims>`, `refresh_token(&self, refresh_token: &str) -> Result<TokenPair>`, `revoke_token(&self, jti: &str)`, `is_revoked(&self, jti: &str) -> bool`, `cleanup_revoked(&self)`. Use `DashMap<String, Instant>` for revocation list. Configure `jsonwebtoken::Validation` with leeway, audience, issuer checks.
- [ ] T012 [US1] Write JWT tests in `crates/mister-smith-security/tests/jwt_tests.rs` — test generate-and-validate roundtrip (US1-AS1, AS2), expired token rejection (AS3), wrong key rejection (AS4), token refresh (AS5), token revocation (AS6), RS256/ES256/HS256 algorithm support (AS7), revocation cleanup, default claims population

**Checkpoint**: JWT authentication fully functional — `cargo test -p mister-smith-security --test jwt_tests` passes

---

## Phase 4: User Story 2 — RBAC Authorization & Permission Checking (Priority: P2)

**Goal**: Enforce role-based access control with hierarchical roles, `action:resource:scope` permission syntax, deny-wins conflict resolution, and optional ABAC conditions

**Independent Test**: Create roles with specific permissions, construct authorization requests, verify policy engine correctly allows/denies based on role membership, ownership, and conditions

### Implementation for User Story 2

- [ ] T013 [P] [US2] Create `crates/mister-smith-security/src/rbac/permission.rs` — `Permission` struct with `action`, `resource`, `scope`, `constraints` fields. Implement `Permission::parse("action:resource:scope")` parser. Create `Role` struct with `name`, `description`, `permissions`, `parent`. Implement `matches(&self, action: &str, resource: &str, scope: &str) -> bool` with wildcard support.
- [ ] T014 [P] [US2] Create `crates/mister-smith-security/src/rbac/constraints.rs` — `PolicyConstraints` struct with `time_window`, `ip_ranges`, `resource_owner` fields. Create `TimeWindow` struct (start_hour, end_hour, timezone, days). Implement `evaluate(&self, context: &HashMap<String, String>) -> bool` to check ABAC conditions.
- [ ] T015 [US2] Implement `PolicyEngine` in `crates/mister-smith-security/src/rbac/mod.rs` — `new(config: &RbacConfig)` with default roles (admin, developer, operator, viewer). `evaluate(&self, request: &AuthorizationRequest) -> PolicyDecision` with deny-wins resolution. `check_permission(&self, claims: &AgentClaims, action: &str, resource: &str) -> bool`. `effective_permissions(&self, roles: &[String]) -> Vec<Permission>` with hierarchical inheritance. `add_role()`, `remove_role()`. Create `AuthorizationRequest` and `PolicyDecision` structs per data-model.md.
- [ ] T016 [US2] Write RBAC tests in `crates/mister-smith-security/tests/rbac_tests.rs` — test worker reads own tasks (US2-AS1), worker can't delete system resources (AS2), admin full access (AS3), explicit deny wins (AS4), default deny (AS5), role hierarchy inheritance (AS6), time-window constraint (AS7), wildcard permission matching, permission string parsing

**Checkpoint**: RBAC authorization functional — `cargo test -p mister-smith-security --test rbac_tests` passes

---

## Phase 5: User Story 3 — Transport Security Middleware (Priority: P3)

**Goal**: Enforce authentication and authorization on HTTP, gRPC, and NATS transports with rate limiting and composable middleware (auth first, then authz)

**Independent Test**: Start an HTTP server with auth middleware, send requests with valid/invalid/missing tokens, verify accept/reject behavior. Test gRPC interceptor and NATS permission checks similarly.

**Dependencies**: Requires US1 (JwtManager) and US2 (PolicyEngine) to be complete

### Implementation for User Story 3

- [ ] T017 [US3] Create `SecurityLayer` composition root in `crates/mister-smith-security/src/middleware/mod.rs` — struct holding `Arc<JwtManager>`, `Arc<PolicyEngine>`, `Option<Arc<CertificateManager>>`, `Arc<AuditLogger>`, `Arc<RateLimiter>`. `new(config: &SecurityConfig) -> Result<Self>`. `is_enabled() -> bool`.
- [ ] T018 [P] [US3] Create `crates/mister-smith-security/src/middleware/rate_limiter.rs` — `RateLimiter` struct with token-bucket algorithm using `DashMap<String, VecDeque<Instant>>`. `new(max_requests: u32, window: Duration)`. `check(&self, source: &str) -> Result<(), Duration>` returning retry-after on limit exceeded.
- [ ] T019 [US3] Implement Axum auth middleware in `crates/mister-smith-security/src/middleware/axum.rs` — `auth_middleware(State(security): State<Arc<SecurityLayer>>, request: Request<Body>, next: Next) -> Response` that extracts Bearer token from Authorization header, validates via JwtManager, checks rate limiter, inserts AgentClaims into request extensions, returns 401/403/429 as appropriate. Create `AuthenticatedAgent(pub AgentClaims)` extractor implementing `FromRequestParts`.
- [ ] T020 [US3] Implement Tonic auth interceptor in `crates/mister-smith-security/src/middleware/tonic.rs` — `grpc_auth_interceptor(security: Arc<SecurityLayer>) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone` that extracts token from `authorization` metadata, validates, inserts claims into extensions, returns `Status::unauthenticated`/`Status::permission_denied` as appropriate.
- [ ] T021 [US3] Implement NATS enforcement in `crates/mister-smith-security/src/middleware/nats.rs` — `SecureTransport<T: Transport>` wrapper that checks RBAC permissions (`publish:{subject}:*`, `subscribe:{subject}:*`) before delegating to inner transport. Implement `Transport` trait (publish, subscribe, request) with permission checks.
- [ ] T022 [US3] Wire auth middleware into `crates/mister-smith-http/src/middleware.rs` — replace the pass-through `security_middleware` stub with a call to the security crate's `auth_middleware`. Update `build_router` in `crates/mister-smith-http/src/server.rs` to accept `SecurityLayer` in `AppState` and pass it to the middleware. Handle the case where security is disabled (pass-through).
- [ ] T023 [US3] Wire auth interceptor into `crates/mister-smith-grpc/src/server.rs` — add `SecurityLayer` to `GrpcServer` and apply `grpc_auth_interceptor` via `with_interceptor` on service builders. Handle disabled security (no interceptor).
- [ ] T024 [US3] Write middleware tests in `crates/mister-smith-security/tests/middleware_tests.rs` — test valid Bearer token passes (US3-AS1), missing auth header returns 401 (AS2), insufficient permissions returns 403 (AS3), gRPC valid token (AS4), gRPC expired token (AS5), NATS unauthorized publish denied (AS6), rate limiter returns 429 (AS7), health endpoint exempt, AuthenticatedAgent extractor works

**Checkpoint**: All transport layers enforce auth — `cargo test -p mister-smith-security --test middleware_tests` passes

---

## Phase 6: User Story 4 — TLS & Certificate Management (Priority: P4)

**Goal**: Configure TLS 1.3 for all transports, enable mTLS for service-to-service authentication, support certificate generation for dev/test and zero-downtime rotation

**Independent Test**: Generate self-signed certificates, configure TLS server/client, verify mTLS handshake success/failure, check certificate expiration detection — all using localhost

### Implementation for User Story 4

- [ ] T025 [P] [US4] Create `crates/mister-smith-security/src/tls/dev_certs.rs` — `generate_dev_certificates(output_dir: &Path, server_sans: &[String]) -> Result<DevCertificates>` using rcgen 0.14: generate CA keypair (`KeyPair::generate()`), create CA cert (`CertificateParams::self_signed` with `IsCa::Ca`), generate server cert (`signed_by` with `ServerAuth`), generate client cert (`signed_by` with `ClientAuth`). Write PEM files to output_dir. Return `DevCertificates` struct with paths.
- [ ] T026 [P] [US4] Create `crates/mister-smith-security/src/tls/config_builder.rs` — `build_server_config(certs, key, ca, mtls) -> Result<Arc<ServerConfig>>` using `ServerConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider())).with_protocol_versions(&[&version::TLS13])`. If mTLS: use `WebPkiClientVerifier::builder_with_provider()`. `build_client_config(ca_certs, client_cert, client_key) -> Result<Arc<ClientConfig>>` similarly.
- [ ] T027 [US4] Implement `CertificateManager` in `crates/mister-smith-security/src/tls/mod.rs` — `new(config: &TlsConfig)` loads certs from disk using `PemObject` trait. `server_config()` and `client_config()` via config_builder. `check_health() -> Vec<CertificateWarning>` checks expiry. `reload()` swaps certs via `ArcSwap`. Create `CertificateWarning` and `WarningSeverity` types.
- [ ] T028 [US4] Write TLS tests in `crates/mister-smith-security/tests/tls_tests.rs` — test cert loading (US4-AS1), client CA verification (AS2), mTLS rejects unauthenticated client (AS3), mTLS accepts valid client (AS4), certificate expiry warning (AS5), certificate reload (AS6), dev certificate generation (AS7), TLS 1.3 enforcement, server and client config construction

**Checkpoint**: TLS/mTLS fully functional — `cargo test -p mister-smith-security --test tls_tests` passes

---

## Phase 7: User Story 5 — Security Audit Logging (Priority: P5)

**Goal**: Capture all security events in a structured, tamper-evident audit log with hash chaining and real-time alert thresholds

**Independent Test**: Trigger auth success/failure events, verify audit entries with correct structure, check hash chain integrity, confirm alert thresholds trigger

### Implementation for User Story 5

- [ ] T029 [P] [US5] Create `crates/mister-smith-security/src/audit/events.rs` — `SecurityAuditEvent` struct (event_id, timestamp, event_type, principal, resource, action, outcome, details, source_ip, previous_hash). `AuditEventType` enum (Authentication, Authorization, TokenLifecycle, CertificateEvent, SuspiciousActivity, SystemAccess, ConfigurationChange). `AuditOutcome` enum (Success, Failure, Blocked, Warning).
- [ ] T030 [US5] Implement `AuditLogger` in `crates/mister-smith-security/src/audit/mod.rs` — `new(config: &AuditConfig)`. `record(event)` appends to `RwLock<VecDeque<SecurityAuditEvent>>` with SHA-256 hash chain (hash of previous entry). `record_auth()` and `record_authz()` convenience methods. `recent_events(limit)`. `verify_chain() -> Result<(), usize>`. `check_alerts()` detects repeated auth failures (5/minute threshold). Enforce `max_events` capacity.
- [ ] T031 [US5] Wire audit logging into `SecurityLayer` in `crates/mister-smith-security/src/middleware/mod.rs` — call `audit.record_auth()` on auth success/failure in Axum middleware and Tonic interceptor. Call `audit.record_authz()` on permission checks. Publish critical audit events to EventBus (if available via trait).
- [ ] T032 [US5] Write audit tests in `crates/mister-smith-security/tests/audit_tests.rs` — test auth success recording (US5-AS1), authz denial recording (AS2), auth failure alert threshold (AS3), hash chain integrity and tamper detection (AS4), middleware rejection audit capture (AS5), recent_events returns limited results, max_events capacity enforced

**Checkpoint**: Audit system captures all security events — `cargo test -p mister-smith-security --test audit_tests` passes

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration validation, workspace quality, and edge case coverage

- [ ] T033 [P] Create security integration tests in `crates/mister-smith-integration-tests/tests/security_integration.rs` — end-to-end test: generate JWT → validate → check RBAC permission → verify audit event recorded. Test security disabled pass-through. Test mTLS certificate chain with rcgen-generated certs.
- [ ] T034 Add `mister-smith-security` dependency to `crates/mister-smith-integration-tests/Cargo.toml` and verify `crates/mister-smith-http/Cargo.toml` and `crates/mister-smith-grpc/Cargo.toml` depend on security crate
- [ ] T035 Run `cargo clippy --workspace -- -D warnings` and fix all warnings
- [ ] T036 Run `cargo test --workspace` and verify all tests pass (existing 605+ plus new security tests)
- [ ] T037 Run `cargo doc -p mister-smith-security --no-deps` and fix any rustdoc warnings

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 JWT Auth (Phase 3)**: Depends on Foundational — no other story dependencies
- **US2 RBAC (Phase 4)**: Depends on Foundational — no other story dependencies
- **US3 Middleware (Phase 5)**: Depends on US1 and US2 (needs JwtManager + PolicyEngine)
- **US4 TLS (Phase 6)**: Depends on Foundational — no other story dependencies
- **US5 Audit (Phase 7)**: Depends on Foundational — integrates with US3 middleware
- **Polish (Phase 8)**: Depends on all user stories complete

### User Story Dependencies

```
Phase 1: Setup
    │
Phase 2: Foundational
    │
    ├── US1: JWT Authentication (P1) ────┐
    ├── US2: RBAC Authorization (P2) ────┤
    │                                     ├── US3: Transport Middleware (P3)
    ├── US4: TLS & Certificates (P4) ────┤
    │                                     │
    └── US5: Security Audit (P5) ────────┘ (integrates with US3)
                                          │
                                    Phase 8: Polish
```

### Within Each User Story

- Types/models before service implementations
- Service implementations before tests (test-after for security — need working implementations to test against)
- Core implementation before integration with other crates

### Parallel Opportunities

- **Phase 2**: T004 and T005 are independent (different crates), T006-T008 depend on T004
- **Phase 3**: T009 and T010 are parallel (different files)
- **Phase 4**: T013 and T014 are parallel (different files)
- **Phase 5**: T018 is parallel with T017 (different files)
- **Phase 6**: T025 and T026 are parallel (different files)
- **US1 + US2 + US4**: Can run in parallel after Phase 2 (independent stories)

---

## Parallel Example: User Story 1 + User Story 2

```bash
# After Phase 2 foundational completes, launch US1 and US2 in parallel:

# US1 parallel models:
Task: T009 "Create claims.rs with AgentClaims, TokenPair"
Task: T010 "Create keys.rs with KeySource and key loading"

# US2 parallel models (same time as US1):
Task: T013 "Create permission.rs with Permission, Role types"
Task: T014 "Create constraints.rs with PolicyConstraints"

# Then sequentially within each story:
# US1: T011 (JwtManager) → T012 (JWT tests)
# US2: T015 (PolicyEngine) → T016 (RBAC tests)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T008)
3. Complete Phase 3: User Story 1 — JWT Authentication (T009–T012)
4. **STOP and VALIDATE**: `cargo test -p mister-smith-security` — JWT works end-to-end
5. Agents can now authenticate with JWT tokens

### Incremental Delivery

1. Setup + Foundational → Crate compiles
2. Add US1 (JWT) → Agents authenticate → **MVP!**
3. Add US2 (RBAC) → Agents authorized → Permission checks work
4. Add US3 (Middleware) → Transport enforces security → HTTP/gRPC/NATS protected
5. Add US4 (TLS) → All connections encrypted → mTLS available
6. Add US5 (Audit) → Security events logged → Compliance-ready
7. Each story adds value without breaking previous stories

### Key Implementation Notes

- **ReceivedMessage**: Wraps `MessageEnvelope` + `reply_subject: Option<String>` (no reply method on Transport trait)
- **Axum 0.8**: `{param}` path syntax, `any()` for WebSocket, `Message::Text(Utf8Bytes)`
- **Crypto backend**: Use `aws_lc_rs` consistently (jsonwebtoken + rustls) — requires cmake at build time
- **SecurityConfig backward compat**: All new fields use `#[serde(default)]` so existing config files without `[security]` section still parse
- **Feature flags**: All security features default-on. Consumers can disable with `default-features = false, features = ["jwt"]` etc.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each phase or logical group
- Stop at any checkpoint to validate independently
- Total: 37 tasks (3 setup + 5 foundational + 4 US1 + 4 US2 + 8 US3 + 4 US4 + 4 US5 + 5 polish)

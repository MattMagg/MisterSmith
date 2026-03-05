# Contract: Security Middleware

**Module**: `mister_smith_security::middleware`

## Public API

### HTTP (Axum) Middleware

```rust
/// Axum middleware that validates JWT Bearer tokens and injects
/// AgentClaims into request extensions.
/// Returns 401 for missing/invalid tokens, 403 for insufficient permissions.
pub async fn auth_middleware(
    State(security): State<Arc<SecurityLayer>>,
    mut request: Request<Body>,
    next: Next,
) -> Response;

/// Axum extractor for authenticated agent identity.
/// Use in route handlers: `AuthenticatedAgent(claims): AuthenticatedAgent`
pub struct AuthenticatedAgent(pub AgentClaims);

impl<S> FromRequestParts<S> for AuthenticatedAgent
where S: Send + Sync;
```

### gRPC (Tonic) Interceptor

```rust
/// Tonic interceptor that validates JWT tokens from gRPC metadata.
/// Returns UNAUTHENTICATED for missing/invalid tokens,
/// PERMISSION_DENIED for insufficient permissions.
pub fn grpc_auth_interceptor(
    security: Arc<SecurityLayer>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone;
```

### NATS Application-Level Enforcement

```rust
/// Wrapper around Transport that enforces RBAC permissions before
/// publish/subscribe operations.
pub struct SecureTransport<T: Transport> {
    inner: T,
    policy_engine: Arc<PolicyEngine>,
    agent_claims: AgentClaims,
}

impl<T: Transport> SecureTransport<T> {
    pub fn new(inner: T, policy_engine: Arc<PolicyEngine>, claims: AgentClaims) -> Self;
}

// SecureTransport checks permissions before delegating to inner Transport:
// - publish: checks "publish:{subject}:*" permission
// - subscribe: checks "subscribe:{subject}:*" permission
// - request: checks "publish:{subject}:*" permission
```

### SecurityLayer (composition root)

```rust
/// Composes JwtManager, PolicyEngine, and CertificateManager
/// into a single shareable security context.
pub struct SecurityLayer {
    pub jwt: Arc<JwtManager>,
    pub policy: Arc<PolicyEngine>,
    pub certs: Option<Arc<CertificateManager>>,
    pub audit: Arc<AuditLogger>,
    pub rate_limiter: Arc<RateLimiter>,
}

impl SecurityLayer {
    pub fn new(config: &SecurityConfig) -> Result<Self, SecurityError>;

    /// Check if security is enabled (master switch).
    pub fn is_enabled(&self) -> bool;
}
```

### RateLimiter

```rust
pub struct RateLimiter { /* private */ }

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self;

    /// Check if a request from the given source should be rate limited.
    /// Returns Ok(()) if allowed, Err with retry-after duration if limited.
    pub fn check(&self, source: &str) -> Result<(), Duration>;
}
```

### HTTP Response Codes

| Scenario | Status Code | Body |
|----------|-------------|------|
| Missing Authorization header | 401 Unauthorized | `{"error": "missing authorization header"}` |
| Invalid/expired/revoked token | 401 Unauthorized | `{"error": "<specific reason>"}` |
| Valid token, insufficient permissions | 403 Forbidden | `{"error": "insufficient permissions"}` |
| Rate limit exceeded | 429 Too Many Requests | `{"error": "rate limit exceeded", "retry_after": N}` |

### gRPC Status Codes

| Scenario | Status | Message |
|----------|--------|---------|
| Missing token | UNAUTHENTICATED | "missing authorization metadata" |
| Invalid/expired token | UNAUTHENTICATED | "<specific reason>" |
| Insufficient permissions | PERMISSION_DENIED | "insufficient permissions for <action>" |
| Rate limited | RESOURCE_EXHAUSTED | "rate limit exceeded" |

### Test Contract

```rust
// HTTP middleware
#[tokio::test] async fn valid_bearer_token_passes();
#[tokio::test] async fn missing_auth_header_returns_401();
#[tokio::test] async fn expired_token_returns_401();
#[tokio::test] async fn insufficient_permissions_returns_403();
#[tokio::test] async fn rate_limit_returns_429();
#[tokio::test] async fn health_endpoint_exempt_by_default();

// gRPC interceptor
#[tokio::test] async fn grpc_valid_token_passes();
#[tokio::test] async fn grpc_missing_token_unauthenticated();
#[tokio::test] async fn grpc_expired_token_unauthenticated();

// NATS enforcement
#[tokio::test] async fn secure_transport_allows_authorized_publish();
#[tokio::test] async fn secure_transport_denies_unauthorized_publish();
#[tokio::test] async fn secure_transport_allows_authorized_subscribe();
```

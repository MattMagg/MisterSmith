# Quickstart: Phase 5 Security Integration

**Feature Branch**: `005-phase5-security`
**Date**: 2026-03-04

## Scenario 1: JWT Authentication for an Agent

```rust
use mister_smith_security::{JwtManager, JwtConfig, AgentClaims, KeySource};
use std::time::Duration;

// Configure JWT with RS256
let jwt_config = JwtConfig {
    algorithm: Algorithm::RS256,
    access_token_ttl: Duration::from_secs(900),    // 15 min
    refresh_token_ttl: Duration::from_secs(86400),  // 24 hours
    issuer: Some("mister-smith".into()),
    audience: vec!["mister-smith-agents".into()],
    key_source: KeySource::RsaPem {
        private_pem: "keys/private.pem".into(),
        public_pem: "keys/public.pem".into(),
    },
};

// Create the JWT manager
let jwt = JwtManager::new(&jwt_config)?;

// Generate tokens for an agent
let claims = AgentClaims {
    sub: agent_id.to_string(),
    agent_id: agent_id.to_string(),
    agent_type: "Worker".into(),
    permissions: vec!["read:task:own".into(), "write:task:own".into()],
    ..Default::default()
};

let token_pair = jwt.generate_token_pair(&claims)?;

// Validate the token later
let validated_claims = jwt.validate_token(&token_pair.access_token)?;
assert_eq!(validated_claims.agent_id, agent_id.to_string());
```

## Scenario 2: RBAC Permission Check

```rust
use mister_smith_security::{PolicyEngine, RbacConfig, AuthorizationRequest};

// Load RBAC configuration with default roles
let rbac_config = RbacConfig::with_defaults(); // admin, developer, operator, viewer

let engine = PolicyEngine::new(&rbac_config);

// Check if a worker agent can read its own tasks
let allowed = engine.check_permission(
    &agent_claims,  // from JWT validation
    "read",         // action
    "task",         // resource
);

// Full evaluation with context
let decision = engine.evaluate(&AuthorizationRequest {
    principal: agent_claims,
    action: "delete".into(),
    resource: "system".into(),
    resource_id: None,
    context: HashMap::new(),
});

if !decision.allowed {
    println!("Denied: {}", decision.reason);
}
```

## Scenario 3: HTTP Server with Auth Middleware

```rust
use mister_smith_security::{SecurityLayer, SecurityConfig, auth_middleware, AuthenticatedAgent};
use axum::{Router, routing::get, middleware};

// Build security layer from config
let security = Arc::new(SecurityLayer::new(&security_config)?);

// Build router with auth middleware on protected routes
let protected = Router::new()
    .route("/api/agents", get(list_agents))
    .route("/api/tasks", get(list_tasks))
    .route_layer(middleware::from_fn_with_state(
        security.clone(),
        auth_middleware,
    ));

// Health endpoints remain unauthenticated
let public = Router::new()
    .route("/health", get(health_check));

let app = public.merge(protected);

// In a handler, extract the authenticated identity:
async fn list_agents(AuthenticatedAgent(claims): AuthenticatedAgent) -> impl IntoResponse {
    // claims.agent_id, claims.permissions, etc.
    Json(format!("Hello, agent {}", claims.agent_id))
}
```

## Scenario 4: gRPC Server with Auth Interceptor

```rust
use mister_smith_security::grpc_auth_interceptor;
use tonic::transport::Server;

let security = Arc::new(SecurityLayer::new(&security_config)?);

Server::builder()
    .add_service(
        AgentServiceServer::with_interceptor(
            agent_service,
            grpc_auth_interceptor(security.clone()),
        )
    )
    .serve(addr)
    .await?;
```

## Scenario 5: TLS with mTLS for NATS

```rust
use mister_smith_security::{CertificateManager, TlsConfig};

// For development: generate self-signed certificates
let dev_certs = CertificateManager::generate_dev_certificates(
    Path::new("./certs"),
    &["localhost".into(), "127.0.0.1".into()],
)?;

// For production: load existing certificates
let tls_config = TlsConfig {
    enabled: true,
    cert_path: Some("certs/server.crt".into()),
    key_path: Some("certs/server.key".into()),
    ca_path: Some("certs/ca.crt".into()),
    mtls_enabled: true,
    min_protocol_version: TlsVersion::TLS13,
    ..Default::default()
};

let cert_manager = CertificateManager::new(&tls_config)?;

// Get configs for transport layers
let server_tls = cert_manager.server_config()?;
let client_tls = cert_manager.client_config()?;

// Check certificate health
for warning in cert_manager.check_health() {
    tracing::warn!(
        subject = %warning.subject,
        days = warning.days_until_expiry,
        "Certificate expiring soon"
    );
}
```

## Scenario 6: Security Audit Trail

```rust
use mister_smith_security::{AuditLogger, AuditConfig, AuditOutcome};

let audit = AuditLogger::new(&AuditConfig::default());

// Record events (called automatically by middleware, shown here for clarity)
audit.record_auth("agent-123", AuditOutcome::Success, HashMap::new());
audit.record_authz("agent-123", "delete", "system", AuditOutcome::Failure);

// Verify hash chain integrity
match audit.verify_chain() {
    Ok(()) => println!("Audit chain intact"),
    Err(idx) => eprintln!("Tamper detected at entry {}", idx),
}

// Check for suspicious patterns
for alert in audit.check_alerts() {
    tracing::error!(event_type = ?alert.event_type, "Security alert");
}
```

## Configuration (TOML)

```toml
[security]
enabled = true

[security.auth]
enabled = true
algorithm = "RS256"
access_token_ttl_secs = 900
refresh_token_ttl_secs = 86400
issuer = "mister-smith"
audience = ["mister-smith-agents"]
private_key_path = "keys/private.pem"
public_key_path = "keys/public.pem"

[security.authz]
enabled = true
default_roles = ["admin", "developer", "operator", "viewer"]

[security.tls]
enabled = true
cert_path = "certs/server.crt"
key_path = "certs/server.key"
ca_path = "certs/ca.crt"
mtls_enabled = true
min_protocol_version = "TLS13"
generate_self_signed = false
expiry_warning_days = 30

[security.audit]
enabled = true
max_events = 10000
auth_failure_alert_threshold = 5
```

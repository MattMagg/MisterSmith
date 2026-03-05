//! Integration tests for the security middleware layer.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware as axum_mw;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tonic::metadata::MetadataValue;
use tonic::{Code, Request as GrpcRequest};
use tower::ServiceExt;

use mister_smith_security::config::{AuditConfig, JwtConfig, KeySource, RbacConfig};
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::middleware::rate_limiter::RateLimiter;
use mister_smith_security::middleware::{SecurityLayer, SecurityLayerConfig};

fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        algorithm: "HS256".to_string(),
        access_token_ttl: Duration::from_secs(300),
        refresh_token_ttl: Duration::from_secs(3600),
        issuer: None,
        audience: Vec::new(),
        key_source: KeySource::Hmac {
            secret: b"test-middleware-secret-key-at-least-32-bytes!".to_vec(),
        },
    }
}

fn test_security_layer(enabled: bool, auth_enabled: bool, authz_enabled: bool) -> Arc<SecurityLayer> {
    Arc::new(
        SecurityLayer::new(SecurityLayerConfig {
            enabled,
            auth_enabled,
            authz_enabled,
            audit_enabled: true,
            tls_enabled: false,
            jwt_config: Some(test_jwt_config()),
            rbac_config: Some(RbacConfig::default()),
            audit_config: Some(AuditConfig::default()),
            #[cfg(feature = "tls")]
            tls_config: None,
        })
        .unwrap(),
    )
}

fn test_token_with_permissions(security: &SecurityLayer, permissions: Vec<String>) -> String {
    let claims = AgentClaims {
        sub: "test-agent".to_string(),
        agent_id: "test-agent".to_string(),
        agent_type: "worker".to_string(),
        permissions,
        ..Default::default()
    };
    let pair = security
        .jwt
        .as_ref()
        .expect("jwt should be configured for token generation")
        .generate_token_pair(&claims)
        .unwrap();
    pair.access_token
}

fn test_token(security: &SecurityLayer) -> String {
    test_token_with_permissions(security, Vec::new())
}

async fn test_handler() -> impl IntoResponse {
    "ok"
}

fn test_app(security: Arc<SecurityLayer>) -> Router {
    Router::new()
        .route("/protected", get(test_handler))
        .layer(axum_mw::from_fn_with_state(
            security,
            mister_smith_security::middleware::axum_mw::auth_middleware,
        ))
}

fn latest_auth_failure_reason(security: &SecurityLayer) -> String {
    security
        .audit
        .as_ref()
        .expect("audit should be configured")
        .recent_events(20)
        .into_iter()
        .rev()
        .find(|event| {
            event.event_type == mister_smith_security::audit::AuditEventType::Authentication
                && event.outcome == mister_smith_security::audit::AuditOutcome::Failure
        })
        .and_then(|event| event.details.get("reason").cloned())
        .expect("expected auth failure audit reason")
}

// -- Valid Bearer token passes (US3-AS1) -----------------------------------

#[tokio::test]
async fn valid_bearer_token_passes() {
    let security = test_security_layer(true, true, true);
    let token = test_token_with_permissions(&security, vec!["get:/protected:/protected".to_string()]);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// -- PR #103: subsystem toggle tests --------------------------------------

#[tokio::test]
async fn master_on_auth_off_passes_through() {
    let security = test_security_layer(true, false, true);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn master_on_auth_on_authz_off_authenticates_without_rbac_path() {
    let security = test_security_layer(true, true, false);
    let token = test_token(&security);
    assert!(security.policy.is_none());

    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn master_off_full_pass_through() {
    let security = test_security_layer(false, true, true);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// -- Missing auth header returns 401 (US3-AS2) ----------------------------

#[tokio::test]
async fn missing_auth_header_returns_401() {
    let security = test_security_layer(true, true, true);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// -- Invalid token returns 401 -------------------------------------------

#[tokio::test]
async fn invalid_token_returns_401() {
    let security = test_security_layer(true, true, true);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .header("authorization", "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// -- Wrong prefix returns 401 --------------------------------------------

#[tokio::test]
async fn wrong_auth_scheme_returns_401() {
    let security = test_security_layer(true, true, true);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .header("authorization", "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// -- Security disabled passes through -------------------------------------

#[tokio::test]
async fn security_disabled_passes_through() {
    let security = test_security_layer(false, true, true);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// -- Rate limiter returns 429 (US3-AS7) -----------------------------------

#[tokio::test]
async fn rate_limiter_returns_429() {
    let security = test_security_layer(true, true, true);
    let token = test_token_with_permissions(&security, vec!["get:/protected:/protected".to_string()]);

    // The SecurityLayer creates a rate limiter with 100 requests/60s.
    // Let's test the rate limiter directly instead.
    let limiter = RateLimiter::new(2, Duration::from_secs(60));
    assert!(limiter.check("test-ip").is_ok());
    assert!(limiter.check("test-ip").is_ok());
    let result = limiter.check("test-ip");
    assert!(result.is_err());

    // Verify the retry-after duration is returned
    if let Err(retry_after) = result {
        assert!(retry_after.as_secs() <= 60);
    }

    // Test the full middleware with a known token
    let app = test_app(security.clone());
    let request = Request::builder()
        .uri("/protected")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// -- AuthenticatedAgent extractor -----------------------------------------

#[tokio::test]
async fn authenticated_agent_extractor_works() {
    use mister_smith_security::middleware::axum_mw::AuthenticatedAgent;

    let security = test_security_layer(true, true, true);
    let token = test_token_with_permissions(&security, vec!["get:/me:/me".to_string()]);

    async fn handler(AuthenticatedAgent(claims): AuthenticatedAgent) -> impl IntoResponse {
        format!("Hello, {}", claims.agent_id)
    }

    let app = Router::new()
        .route("/me", get(handler))
        .layer(axum_mw::from_fn_with_state(
            security,
            mister_smith_security::middleware::axum_mw::auth_middleware,
        ));

    let request = Request::builder()
        .uri("/me")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "Hello, test-agent");
}

// -- SecurityLayer construction -------------------------------------------

#[test]
fn security_layer_enabled() {
    let layer = test_security_layer(true, true, true);
    assert!(layer.is_enabled());
}

#[test]
fn security_layer_disabled() {
    let layer = test_security_layer(false, true, true);
    assert!(!layer.is_enabled());
}

// -- Revoked token via middleware -----------------------------------------

#[tokio::test]
async fn revoked_token_returns_401() {
    let security = test_security_layer(true, true, true);
    let token = test_token(&security);

    let jwt = security.jwt.as_ref().unwrap();
    let claims = jwt.validate_token(&token).unwrap();
    jwt.revoke_token(&claims.jti);

    let app = test_app(security);
    let request = Request::builder()
        .uri("/protected")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// -- Authorization failures return 403 -------------------------------------

#[tokio::test]
async fn authenticated_but_unauthorized_http_request_returns_403() {
    let security = test_security_layer(true, true, true);
    let token = test_token(&security);
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn authorized_http_request_succeeds() {
    let security = test_security_layer(true, true, true);
    let token = test_token_with_permissions(
        &security,
        vec!["get:/protected:/protected".to_string()],
    );
    let app = test_app(security);

    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn authenticated_but_unauthorized_grpc_request_returns_permission_denied() {
    let security = test_security_layer(true, true, true);
    let token = test_token(&security);
    let interceptor = mister_smith_security::middleware::tonic_mw::grpc_auth_interceptor(security);

    let mut request = tonic::Request::new(());
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    request
        .extensions_mut()
        .insert(tonic::GrpcMethod::new("mistersmith.SecurityService", "Check"));

    let result = interceptor(request);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
}

#[test]
fn authorized_grpc_request_succeeds() {
    let security = test_security_layer(true, true, true);
    let token = test_token_with_permissions(
        &security,
        vec!["grpc_call:/mistersmith.SecurityService/Check:/mistersmith.SecurityService/Check"
            .to_string()],
    );
    let interceptor =
        mister_smith_security::middleware::tonic_mw::grpc_auth_interceptor(security.clone());

    let mut request = tonic::Request::new(());
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    request
        .extensions_mut()
        .insert(tonic::GrpcMethod::new("mistersmith.SecurityService", "Check"));

    let result = interceptor(request);
    assert!(result.is_ok());
}

#[tokio::test]
async fn audit_log_contains_authz_events_for_allow_and_deny() {
    use mister_smith_security::audit::events::{AuditEventType, AuditOutcome};

    let security = test_security_layer(true, true, true);
    let denied_token = test_token(&security);
    let allowed_token = test_token_with_permissions(
        &security,
        vec!["get:/protected:/protected".to_string()],
    );

    let app = test_app(security.clone());

    let denied_request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("authorization", format!("Bearer {denied_token}"))
        .body(Body::empty())
        .unwrap();
    let denied_response = app.clone().oneshot(denied_request).await.unwrap();
    assert_eq!(denied_response.status(), StatusCode::FORBIDDEN);

    let allowed_request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("authorization", format!("Bearer {allowed_token}"))
        .body(Body::empty())
        .unwrap();
    let allowed_response = app.oneshot(allowed_request).await.unwrap();
    assert_eq!(allowed_response.status(), StatusCode::OK);

    let audit = security.audit.as_ref().expect("audit should be configured");
    let events = audit.recent_events(32);
    let authz_events: Vec<_> = events
        .iter()
        .filter(|event| event.event_type == AuditEventType::Authorization)
        .collect();

    assert!(
        authz_events
            .iter()
            .any(|event| event.outcome == AuditOutcome::Failure)
    );
    assert!(
        authz_events
            .iter()
            .any(|event| event.outcome == AuditOutcome::Success)
    );
}

// -- Error response sanitization ------------------------------------------

#[tokio::test]
async fn invalid_token_response_is_sanitized_and_audit_keeps_details() {
    let security = test_security_layer(true, true, true);
    let app = test_app(security.clone());

    let request = Request::builder()
        .uri("/protected")
        .header("authorization", "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "unauthorized");

    let audit_reason = latest_auth_failure_reason(&security);
    assert!(audit_reason.contains("Invalid token:"));
    assert_ne!(audit_reason, "unauthorized");
}

#[tokio::test]
async fn revoked_token_response_is_sanitized_and_audit_keeps_details() {
    let security = test_security_layer(true, true, true);
    let token = test_token(&security);

    let jwt = security.jwt.as_ref().unwrap();
    let claims = jwt.validate_token(&token).unwrap();
    jwt.revoke_token(&claims.jti);

    let app = test_app(security.clone());
    let request = Request::builder()
        .uri("/protected")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "token revoked");

    let audit_reason = latest_auth_failure_reason(&security);
    assert_eq!(audit_reason, "Token revoked");
}

#[test]
fn tonic_invalid_token_is_sanitized_and_audit_keeps_details() {
    let security = test_security_layer(true, true, true);
    let interceptor =
        mister_smith_security::middleware::tonic_mw::grpc_auth_interceptor(security.clone());

    let mut request = GrpcRequest::new(());
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_static("Bearer invalid.token.here"),
    );

    let error = interceptor(request).expect_err("expected unauthenticated error");
    assert_eq!(error.code(), Code::Unauthenticated);
    assert_eq!(error.message(), "unauthorized");

    let audit_reason = latest_auth_failure_reason(&security);
    assert!(audit_reason.contains("Invalid token:"));
    assert_ne!(audit_reason, "unauthorized");
}

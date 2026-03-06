//! Cross-crate security integration tests.
//!
//! End-to-end scenarios: JWT → RBAC → Audit → Middleware.

use std::sync::Arc;
use std::time::Duration;

use mister_smith_security::config::{AuditConfig, JwtConfig, KeySource, RbacConfig};
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::middleware::{SecurityLayer, SecurityLayerConfig};
use mister_smith_security::rbac::PolicyEngine;

fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        algorithm: "HS256".to_string(),
        access_token_ttl: Duration::from_secs(300),
        refresh_token_ttl: Duration::from_secs(3600),
        issuer: Some("mister-smith".to_string()),
        audience: vec!["integration-test".to_string()],
        key_source: KeySource::Hmac {
            secret: b"integration-test-secret-key-at-least-32-bytes!".to_vec(),
        },
    }
}

// -- End-to-end: JWT → validate → RBAC → audit ---------------------------

fn test_security_layer(enabled: bool) -> SecurityLayer {
    test_security_layer_with_authz(enabled, true)
}

fn test_security_layer_with_authz(enabled: bool, authz_enabled: bool) -> SecurityLayer {
    SecurityLayer::new(SecurityLayerConfig {
        enabled,
        auth_enabled: true,
        authz_enabled,
        audit_enabled: true,
        tls_enabled: false,
        jwt_config: Some(test_jwt_config()),
        rbac_config: Some(RbacConfig::default()),
        audit_config: Some(AuditConfig::default()),
        tls_config: None,
    })
    .unwrap()
}

#[test]
fn e2e_jwt_validate_rbac_audit() {
    let security = test_security_layer(true);

    // Generate a token for a viewer agent
    let claims = AgentClaims {
        sub: "agent-viewer".to_string(),
        agent_id: "agent-viewer".to_string(),
        agent_type: "viewer".to_string(),
        ..Default::default()
    };

    let pair = security
        .jwt
        .as_ref()
        .unwrap()
        .generate_token_pair(&claims)
        .unwrap();

    // Validate the token
    let validated = security
        .jwt
        .as_ref()
        .unwrap()
        .validate_token(&pair.access_token)
        .unwrap();
    assert_eq!(validated.agent_id, "agent-viewer");

    // Check RBAC permission (viewer can read)
    assert!(security
        .policy
        .as_ref()
        .unwrap()
        .check_permission(&validated, "read", "agent"));
    // Viewer cannot write
    assert!(!security
        .policy
        .as_ref()
        .unwrap()
        .check_permission(&validated, "write", "agent"));

    // Record auth success in audit log
    security.audit.as_ref().unwrap().record_auth(
        &validated.sub,
        mister_smith_security::audit::AuditOutcome::Success,
        std::collections::HashMap::new(),
    );

    let events = security.audit.as_ref().unwrap().recent_events(1);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].principal.as_deref(), Some("agent-viewer"));
}

// -- Security disabled pass-through ---------------------------------------

#[test]
fn security_disabled_layer() {
    let security = test_security_layer(false);

    assert!(!security.is_enabled());
    // With master switch off, optional subsystems are not constructed.
    assert!(security.jwt.is_none());
    assert!(security.policy.is_none());
    assert!(security.audit.is_none());
}

// -- RBAC with default role fallback --------------------------------------

#[test]
fn rbac_default_role_integration() {
    let rbac_config = RbacConfig {
        default_role: Some("viewer".to_string()),
    };
    let engine = PolicyEngine::new(&rbac_config);

    // Claims with no agent_type should fall back to viewer
    let claims = AgentClaims {
        sub: "anonymous".to_string(),
        agent_id: "anonymous".to_string(),
        ..Default::default()
    };

    assert!(engine.check_permission(&claims, "read", "agent"));
    assert!(!engine.check_permission(&claims, "write", "agent"));
}

// -- Token lifecycle: generate → validate → revoke → reject ---------------

#[test]
fn token_lifecycle_integration() {
    let security = test_security_layer(true);

    let claims = AgentClaims {
        sub: "lifecycle-agent".to_string(),
        agent_id: "lifecycle-agent".to_string(),
        agent_type: "admin".to_string(),
        ..Default::default()
    };

    // Generate
    let pair = security
        .jwt
        .as_ref()
        .unwrap()
        .generate_token_pair(&claims)
        .unwrap();

    // Validate
    let validated = security
        .jwt
        .as_ref()
        .unwrap()
        .validate_token(&pair.access_token)
        .unwrap();
    assert_eq!(validated.agent_id, "lifecycle-agent");

    // Revoke
    security.jwt.as_ref().unwrap().revoke_token(&validated.jti);

    // Reject revoked token
    let result = security
        .jwt
        .as_ref()
        .unwrap()
        .validate_token(&pair.access_token);
    assert!(result.is_err());
}

// -- TLS dev certificate generation integration ---------------------------

#[test]
fn tls_dev_cert_generation_integration() {
    let dir = tempfile::TempDir::new().unwrap();
    let certs = mister_smith_security::tls::CertificateManager::generate_dev_certificates(
        dir.path(),
        &["localhost".to_string(), "127.0.0.1".to_string()],
    )
    .unwrap();

    // Verify all files exist
    assert!(certs.ca_cert_path.exists());
    assert!(certs.server_cert_path.exists());
    assert!(certs.client_cert_path.exists());

    // Verify we can load them into a CertificateManager
    let tls_config = mister_smith_security::config::TlsConfig {
        enabled: true,
        cert_path: Some(certs.server_cert_path),
        key_path: Some(certs.server_key_path),
        ca_path: Some(certs.ca_cert_path),
        mtls_enabled: true,
        ..Default::default()
    };
    let mgr = mister_smith_security::tls::CertificateManager::new(&tls_config);
    assert!(mgr.is_ok());
}

// -- Audit hash chain integrity across operations -------------------------

#[test]
fn audit_chain_integrity_integration() {
    let security = test_security_layer(true);

    // Perform multiple operations
    security.audit.as_ref().unwrap().record_auth(
        "agent-1",
        mister_smith_security::audit::AuditOutcome::Success,
        std::collections::HashMap::new(),
    );
    security.audit.as_ref().unwrap().record_authz(
        "agent-1",
        "read",
        "config",
        mister_smith_security::audit::AuditOutcome::Success,
    );
    security.audit.as_ref().unwrap().record_auth(
        "agent-2",
        mister_smith_security::audit::AuditOutcome::Failure,
        [("reason".to_string(), "invalid_token".to_string())]
            .into_iter()
            .collect(),
    );

    // Verify chain integrity
    assert!(security.audit.as_ref().unwrap().verify_chain().is_ok());

    // Verify event count
    let events = security.audit.as_ref().unwrap().recent_events(10);
    assert_eq!(events.len(), 3);
}

// -- gRPC auth interceptor integration --------------------------------------

async fn spawn_secure_grpc_server(
    security: Arc<SecurityLayer>,
) -> (
    tokio::task::JoinHandle<Result<(), mister_smith_grpc::errors::TransportError>>,
    String,
    tokio::sync::oneshot::Sender<()>,
) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let mut server = mister_smith_grpc::server::GrpcServer::new(
        mister_smith_grpc::config::GrpcTransportConfig::new(addr.to_string()),
    )
    .with_security(security);

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
        server
            .serve(async {
                let _ = rx.await;
            })
            .await
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    (handle, format!("http://{addr}"), tx)
}

#[tokio::test]
async fn grpc_request_without_authorization_returns_unauthenticated() {
    let security = Arc::new(test_security_layer(true));

    let (handle, endpoint, shutdown_tx) = spawn_secure_grpc_server(security).await;

    let channel = tonic::transport::Endpoint::from_shared(endpoint)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = tonic_health::pb::health_client::HealthClient::new(channel);
    let err = client
        .check(tonic_health::pb::HealthCheckRequest {
            service: "".to_string(),
        })
        .await
        .unwrap_err();

    assert_eq!(err.code(), tonic::Code::Unauthenticated);

    let _ = shutdown_tx.send(());
    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn grpc_request_with_invalid_token_returns_unauthenticated() {
    let security = Arc::new(test_security_layer(true));

    let (handle, endpoint, shutdown_tx) = spawn_secure_grpc_server(security).await;

    let channel = tonic::transport::Endpoint::from_shared(endpoint)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = tonic_health::pb::health_client::HealthClient::new(channel);
    let mut request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
        service: "".to_string(),
    });
    request.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::from_static("Bearer not-a-valid-token"),
    );

    let err = client.check(request).await.unwrap_err();

    assert_eq!(err.code(), tonic::Code::Unauthenticated);

    let _ = shutdown_tx.send(());
    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn grpc_request_with_valid_token_succeeds() {
    let security = Arc::new(test_security_layer_with_authz(true, false));

    let token = security
        .jwt
        .as_ref()
        .unwrap()
        .generate_token_pair(&AgentClaims {
            sub: "grpc-int-test".to_string(),
            agent_id: "grpc-int-test".to_string(),
            ..Default::default()
        })
        .unwrap()
        .access_token;

    let (handle, endpoint, shutdown_tx) = spawn_secure_grpc_server(security).await;

    let channel = tonic::transport::Endpoint::from_shared(endpoint)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = tonic_health::pb::health_client::HealthClient::new(channel);
    let mut request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
        service: "".to_string(),
    });
    request.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from(format!("Bearer {token}")).unwrap(),
    );

    let response = client.check(request).await.unwrap();
    assert_eq!(
        response.into_inner().status,
        tonic_health::ServingStatus::Serving as i32
    );

    let _ = shutdown_tx.send(());
    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

// -- Middleware integration with Axum -------------------------------------

#[tokio::test]
async fn axum_middleware_integration() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware as axum_mw;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    let security = Arc::new(test_security_layer_with_authz(true, false));

    async fn handler() -> impl IntoResponse {
        "authorized"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(axum_mw::from_fn_with_state(
            security.clone(),
            mister_smith_security::middleware::axum_mw::auth_middleware,
        ));

    // Unauthenticated → 401
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Authenticated → 200
    let token = security
        .jwt
        .as_ref()
        .unwrap()
        .generate_token_pair(&AgentClaims {
            sub: "int-test".to_string(),
            agent_id: "int-test".to_string(),
            ..Default::default()
        })
        .unwrap()
        .access_token;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

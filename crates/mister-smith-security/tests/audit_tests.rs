//! Black-box audit integration tests for `AuditLogger` and middleware wiring.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware as axum_mw;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

use mister_smith_security::audit::{AuditEventType, AuditLogger, AuditOutcome, SecurityAuditEvent};
use mister_smith_security::config::{AuditConfig, JwtConfig, KeySource, RbacConfig};
use mister_smith_security::middleware::{SecurityLayer, SecurityLayerConfig};

fn test_audit_config() -> AuditConfig {
    AuditConfig {
        enabled: true,
        max_events: 100,
        auth_failure_alert_threshold: 5,
    }
}

fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        algorithm: "HS256".to_string(),
        access_token_ttl: Duration::from_secs(300),
        refresh_token_ttl: Duration::from_secs(3_600),
        issuer: None,
        audience: Vec::new(),
        key_source: KeySource::Hmac {
            secret: b"audit-test-secret-key-at-least-32-bytes!".to_vec(),
        },
    }
}

fn test_layer() -> Arc<SecurityLayer> {
    Arc::new(
        SecurityLayer::new(SecurityLayerConfig {
            enabled: true,
            auth_enabled: true,
            authz_enabled: false,
            audit_enabled: true,
            tls_enabled: false,
            jwt_config: Some(test_jwt_config()),
            rbac_config: Some(RbacConfig::default()),
            audit_config: Some(test_audit_config()),
            tls_config: None,
        })
        .unwrap(),
    )
}

#[test]
fn us5_as1_auth_success_recording() {
    let logger = AuditLogger::new(&test_audit_config());
    let mut details = HashMap::new();
    details.insert("method".to_string(), "jwt".to_string());

    logger.record_auth("agent-42", AuditOutcome::Success, details);

    let events = logger.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::Authentication);
    assert_eq!(events[0].outcome, AuditOutcome::Success);
    assert_eq!(events[0].principal.as_deref(), Some("agent-42"));
}

#[test]
fn us5_as2_authz_denial_recording() {
    let logger = AuditLogger::new(&test_audit_config());

    logger.record_authz("agent-1", "delete", "system", AuditOutcome::Failure);

    let events = logger.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::Authorization);
    assert_eq!(events[0].outcome, AuditOutcome::Failure);
    assert_eq!(events[0].action.as_deref(), Some("delete"));
    assert_eq!(events[0].resource.as_deref(), Some("system"));
}

#[test]
fn us5_as3_auth_failure_alert_threshold() {
    let logger = AuditLogger::new(&test_audit_config());

    for _ in 0..5 {
        logger.record(SecurityAuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authentication,
            principal: Some("unknown".to_string()),
            resource: None,
            action: Some("authenticate".to_string()),
            outcome: AuditOutcome::Failure,
            details: HashMap::new(),
            source_ip: Some("10.1.2.3".to_string()),
            previous_hash: None,
        });
    }

    let alerts = logger.check_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].event_type, AuditEventType::SuspiciousActivity);
    assert_eq!(alerts[0].source_ip.as_deref(), Some("10.1.2.3"));
}

#[test]
fn us5_as4_hash_chain_integrity() {
    let logger = AuditLogger::new(&test_audit_config());

    logger.record_auth("agent-1", AuditOutcome::Success, HashMap::new());
    logger.record_authz("agent-1", "read", "task", AuditOutcome::Success);

    assert!(logger.verify_chain().is_ok());
}

#[tokio::test]
async fn us5_as5_middleware_rejection_audit_capture() {
    async fn ok_handler() -> &'static str {
        "ok"
    }

    let security = test_layer();
    let app = Router::new()
        .route("/protected", get(ok_handler))
        .layer(axum_mw::from_fn_with_state(
            security.clone(),
            mister_smith_security::middleware::axum_mw::auth_middleware,
        ));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let events = security.audit.as_ref().expect("audit enabled").recent_events(10);
    assert!(events.iter().any(|event| {
        event.event_type == AuditEventType::Authentication
            && event.outcome == AuditOutcome::Failure
            && event
                .details
                .get("reason")
                .map(|value: &String| value.contains("missing_auth_header"))
                .unwrap_or(false)
    }));
}

#[test]
fn recent_events_limit_and_max_capacity_enforced() {
    let logger = AuditLogger::new(&AuditConfig {
        enabled: true,
        max_events: 3,
        auth_failure_alert_threshold: 5,
    });

    for _ in 0..5 {
        logger.record_auth("agent", AuditOutcome::Success, HashMap::new());
    }

    let events = logger.recent_events(10);
    assert_eq!(events.len(), 3);

    let limited = logger.recent_events(2);
    assert_eq!(limited.len(), 2);
}

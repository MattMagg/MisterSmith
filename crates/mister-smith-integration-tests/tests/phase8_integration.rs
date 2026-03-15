//! Integration test: Phase 8 cross-phase integration wiring (US3).
//!
//! Verifies:
//! - Audit event recording and drain (AuditLogger → persistence bridge)
//! - JWT token validation and rejection (SecurityBridge)
//! - RBAC permission enforcement (PolicyEngine)
//! - Heartbeat-based failure detection (PhiAccrualFailureDetector)
//! - Supervision event audit trail

use std::sync::Arc;
use std::time::Duration;

use mister_smith_monitoring::PhiAccrualFailureDetector;
use mister_smith_security::audit::AuditLogger;
use mister_smith_security::jwt::JwtManager;
use mister_smith_security::rbac::PolicyEngine;
use mister_smith_security::{AuditConfig, JwtConfig, RbacConfig};
use tokio::sync::Mutex;

#[tokio::test]
async fn audit_logger_drain_events_returns_recorded_events() {
    use mister_smith_security::audit::events::{AuditEventType, AuditOutcome, SecurityAuditEvent};

    let logger = AuditLogger::new(&AuditConfig::default());

    let event = SecurityAuditEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        event_type: AuditEventType::Authentication,
        principal: Some("test-agent".to_string()),
        resource: Some("/api/tasks".to_string()),
        action: Some("login".to_string()),
        outcome: AuditOutcome::Success,
        details: Default::default(),
        delegation: None,
        source_ip: None,
        previous_hash: None,
    };
    logger.record(event);

    // Drain should return events
    let events = logger.drain_events();
    assert!(!events.is_empty());
    assert_eq!(events[0].action.as_deref(), Some("login"));

    // After drain, ring buffer should be empty
    let remaining = logger.drain_events();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn jwt_manager_validates_and_rejects_bad_tokens() {
    use mister_smith_security::jwt::AgentClaims;

    let config = JwtConfig::default();
    let manager = JwtManager::new(&config).unwrap();

    let now = chrono::Utc::now().timestamp() as u64;
    let claims = AgentClaims {
        sub: uuid::Uuid::new_v4().to_string(),
        exp: now + 3600,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        agent_id: uuid::Uuid::new_v4().to_string(),
        agent_type: "worker".to_string(),
        token_use: "access".to_string(),
        ..Default::default()
    };

    let token_pair = manager.generate_token_pair(&claims).unwrap();

    // Valid token should validate
    let validated = manager.validate_token(&token_pair.access_token);
    assert!(validated.is_ok());

    // Invalid token should be rejected
    let invalid = manager.validate_token("not-a-valid-jwt-token");
    assert!(invalid.is_err());
}

#[tokio::test]
async fn policy_engine_enforces_role_permissions() {
    use mister_smith_security::jwt::AgentClaims;

    let config = RbacConfig::default();
    let engine = PolicyEngine::new(&config);

    let now = chrono::Utc::now().timestamp() as u64;

    // Agent with "admin" role should have broad access
    let admin_claims = AgentClaims {
        sub: uuid::Uuid::new_v4().to_string(),
        exp: now + 3600,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        agent_id: uuid::Uuid::new_v4().to_string(),
        agent_type: "orchestrator".to_string(),
        permissions: vec!["manage:agents:*".to_string()],
        token_use: "access".to_string(),
        ..Default::default()
    };
    assert!(engine.check_permission(&admin_claims, "manage", "agents"));

    // Agent with only "read" permission should not have "manage" access
    let limited_claims = AgentClaims {
        sub: uuid::Uuid::new_v4().to_string(),
        exp: now + 3600,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        agent_id: uuid::Uuid::new_v4().to_string(),
        agent_type: "worker".to_string(),
        permissions: vec!["read:tasks:own".to_string()],
        token_use: "access".to_string(),
        ..Default::default()
    };
    assert!(!engine.check_permission(&limited_claims, "manage", "agents"));
}

#[tokio::test]
async fn phi_accrual_detects_heartbeat_absence() {
    let detector = Arc::new(Mutex::new(PhiAccrualFailureDetector::new(8.0, 100)));

    // Record heartbeats with short intervals
    {
        let mut d = detector.lock().await;
        d.record_heartbeat("agent-1");
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    {
        let mut d = detector.lock().await;
        d.record_heartbeat("agent-1");
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    {
        let mut d = detector.lock().await;
        d.record_heartbeat("agent-1");
    }

    // Shortly after, agent should be available
    {
        let d = detector.lock().await;
        assert!(d.is_available("agent-1"));
    }

    // Wait much longer than heartbeat interval
    tokio::time::sleep(Duration::from_millis(500)).await;
    {
        let d = detector.lock().await;
        let phi = d.phi("agent-1");
        assert!(phi.is_some());
        assert!(
            phi.unwrap() > 8.0,
            "Expected phi > 8.0 after heartbeat absence, got {}",
            phi.unwrap()
        );
        assert!(!d.is_available("agent-1"));
    }
}

#[tokio::test]
async fn supervision_event_audit_trail() {
    use mister_smith_security::audit::events::{AuditEventType, AuditOutcome, SecurityAuditEvent};

    let logger = Arc::new(AuditLogger::new(&AuditConfig::default()));

    // Record a supervision restart event
    let event = SecurityAuditEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        event_type: AuditEventType::SuspiciousActivity,
        principal: Some("agent-42".to_string()),
        resource: Some("supervision".to_string()),
        action: Some("agent_restart".to_string()),
        outcome: AuditOutcome::Success,
        details: {
            let mut m = std::collections::HashMap::new();
            m.insert("reason".to_string(), "heartbeat_timeout".to_string());
            m
        },
        delegation: None,
        source_ip: None,
        previous_hash: None,
    };
    logger.record(event);

    let events = logger.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action.as_deref(), Some("agent_restart"));
    assert_eq!(events[0].resource.as_deref(), Some("supervision"));
}

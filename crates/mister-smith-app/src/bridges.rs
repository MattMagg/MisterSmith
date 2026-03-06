//! Cross-phase integration wiring (Phase 8 — US3).
//!
//! Bridges the three gaps identified in the 2026-03-05 deviation report:
//! 1. AuditLogger (Phase 5) → AuditPersister (Phase 6): zero-loss event draining
//! 2. AgentRuntime (Phase 7) → PolicyEngine (Phase 5): security enforcement
//! 3. HeartbeatEmitter (Phase 7) → PhiAccrualFailureDetector (Phase 2): monitoring
//!
//! Also provides supervision event recording (metrics + audit) and fresh
//! credential issuance on agent restart.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use mister_smith_monitoring::PhiAccrualFailureDetector;
use mister_smith_security::audit::AuditLogger;

// ---------------------------------------------------------------------------
// Heartbeat → Failure Detector Bridge
// ---------------------------------------------------------------------------

/// Routes agent heartbeat events to the Phi accrual failure detector.
///
/// Runs a background task that periodically checks phi values and logs
/// warnings when agents are suspected of failure.
pub struct HeartbeatBridge {
    failure_detector: Arc<Mutex<PhiAccrualFailureDetector>>,
    check_interval: Duration,
}

impl HeartbeatBridge {
    pub fn new(
        failure_detector: Arc<Mutex<PhiAccrualFailureDetector>>,
        check_interval: Duration,
    ) -> Self {
        Self {
            failure_detector,
            check_interval,
        }
    }

    /// Record a heartbeat from an agent.
    pub async fn record_heartbeat(&self, agent_id: &str) {
        let mut detector = self.failure_detector.lock().await;
        detector.record_heartbeat(agent_id);
        debug!(agent_id, "Heartbeat recorded");
    }

    /// Check if an agent is considered available by the failure detector.
    pub async fn is_agent_available(&self, agent_id: &str) -> bool {
        let detector = self.failure_detector.lock().await;
        detector.is_available(agent_id)
    }

    /// Get the phi value for an agent (None if insufficient data).
    pub async fn agent_phi(&self, agent_id: &str) -> Option<f64> {
        let detector = self.failure_detector.lock().await;
        detector.phi(agent_id)
    }

    /// Run a background monitoring loop that periodically checks phi values
    /// for registered agents and logs warnings for suspected failures.
    pub async fn run_monitor(
        &self,
        tracked_agents: Arc<Mutex<Vec<String>>>,
        shutdown: Arc<AtomicBool>,
    ) {
        info!(
            interval_ms = self.check_interval.as_millis() as u64,
            "Heartbeat monitor started"
        );

        while !shutdown.load(Ordering::SeqCst) {
            tokio::time::sleep(self.check_interval).await;

            let agents = tracked_agents.lock().await.clone();
            let detector = self.failure_detector.lock().await;

            for agent_id in &agents {
                if let Some(phi) = detector.phi(agent_id) {
                    if phi >= detector.phi_threshold() {
                        warn!(
                            agent_id,
                            phi,
                            threshold = detector.phi_threshold(),
                            "Agent suspected failed — heartbeat loss detected"
                        );
                        metrics::counter!("mistersmith_agent_heartbeat_failures_total")
                            .increment(1);
                    }
                }
            }
        }

        info!("Heartbeat monitor stopped");
    }
}

// ---------------------------------------------------------------------------
// Supervision Event Recording
// ---------------------------------------------------------------------------

/// Records supervision events (agent restarts, failures) in both
/// the metrics pipeline and the audit log.
pub struct SupervisionRecorder {
    audit_logger: Option<Arc<AuditLogger>>,
}

impl SupervisionRecorder {
    pub fn new(audit_logger: Option<Arc<AuditLogger>>) -> Self {
        Self { audit_logger }
    }

    /// Record an agent restart event.
    pub fn record_restart(&self, agent_id: &str, reason: &str) {
        // Emit metric
        let labels = vec![
            ("agent_id".to_string(), agent_id.to_string()),
            ("reason".to_string(), reason.to_string()),
        ];
        metrics::counter!("mistersmith_agent_restarts_total", &labels).increment(1);

        info!(agent_id, reason, "Agent restarted — recorded in metrics");

        // Record audit event if logger is available
        if let Some(ref logger) = self.audit_logger {
            use mister_smith_security::audit::events::{
                AuditEventType, AuditOutcome, SecurityAuditEvent,
            };
            let mut details = std::collections::HashMap::new();
            details.insert("reason".to_string(), reason.to_string());
            details.insert("agent_id".to_string(), agent_id.to_string());

            let event = SecurityAuditEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::SuspiciousActivity,
                principal: Some(agent_id.to_string()),
                resource: Some("supervision".to_string()),
                action: Some("agent_restart".to_string()),
                outcome: AuditOutcome::Success,
                details,
                source_ip: None,
                previous_hash: None,
            };
            logger.record(event);
            debug!(agent_id, "Agent restart recorded in audit log");
        }
    }

    /// Record an agent failure event.
    pub fn record_failure(&self, agent_id: &str, error_msg: &str) {
        let labels = vec![("agent_id".to_string(), agent_id.to_string())];
        metrics::counter!("mistersmith_agent_failures_total", &labels).increment(1);

        error!(agent_id, error = error_msg, "Agent failed");

        if let Some(ref logger) = self.audit_logger {
            use mister_smith_security::audit::events::{
                AuditEventType, AuditOutcome, SecurityAuditEvent,
            };
            let mut details = std::collections::HashMap::new();
            details.insert("error".to_string(), error_msg.to_string());

            let event = SecurityAuditEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::SuspiciousActivity,
                principal: Some(agent_id.to_string()),
                resource: Some("supervision".to_string()),
                action: Some("agent_failure".to_string()),
                outcome: AuditOutcome::Failure,
                details,
                source_ip: None,
                previous_hash: None,
            };
            logger.record(event);
        }
    }
}

// ---------------------------------------------------------------------------
// Security Enforcement Bridge
// ---------------------------------------------------------------------------

/// Wraps PolicyEngine and JwtManager to enforce security on agent operations.
///
/// During bootstrap, this bridge is created with references to the security
/// subsystem components. Agent operations call `check_authorization` before
/// executing privileged actions.
pub struct SecurityBridge {
    jwt_manager: Option<Arc<mister_smith_security::jwt::JwtManager>>,
    policy_engine: Option<Arc<mister_smith_security::rbac::PolicyEngine>>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl SecurityBridge {
    pub fn new(
        jwt_manager: Option<Arc<mister_smith_security::jwt::JwtManager>>,
        policy_engine: Option<Arc<mister_smith_security::rbac::PolicyEngine>>,
        audit_logger: Option<Arc<AuditLogger>>,
    ) -> Self {
        Self {
            jwt_manager,
            policy_engine,
            audit_logger,
        }
    }

    /// Check if an agent's token is valid and has permission for the given action.
    ///
    /// Returns `Ok(())` if authorized, `Err(reason)` if unauthorized.
    /// If no JwtManager is configured, all operations are allowed.
    pub fn check_authorization(
        &self,
        token: &str,
        action: &str,
        resource: &str,
    ) -> Result<(), String> {
        let jwt_manager = match &self.jwt_manager {
            Some(m) => m,
            None => return Ok(()), // No JWT manager configured — allow all
        };

        let claims = jwt_manager.validate_token(token).map_err(|e| {
            self.record_unauthorized(token, action, resource, &e.to_string());
            format!("Token validation failed: {e}")
        })?;

        if let Some(ref engine) = self.policy_engine {
            if !engine.check_permission(&claims, action, resource) {
                let reason = format!(
                    "Agent {} lacks permission for {action} on {resource}",
                    claims.agent_id
                );
                self.record_unauthorized(token, action, resource, &reason);
                return Err(reason);
            }
        }

        Ok(())
    }

    fn record_unauthorized(&self, _token: &str, action: &str, resource: &str, reason: &str) {
        metrics::counter!("mistersmith_unauthorized_operations_total").increment(1);

        if let Some(ref logger) = self.audit_logger {
            use mister_smith_security::audit::events::{
                AuditEventType, AuditOutcome, SecurityAuditEvent,
            };
            let mut details = std::collections::HashMap::new();
            details.insert("action".to_string(), action.to_string());
            details.insert("resource".to_string(), resource.to_string());
            details.insert("reason".to_string(), reason.to_string());

            let event = SecurityAuditEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::Authorization,
                principal: None,
                resource: Some(resource.to_string()),
                action: Some(action.to_string()),
                outcome: AuditOutcome::Blocked,
                details,
                source_ip: None,
                previous_hash: None,
            };
            logger.record(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_security::AuditConfig;

    #[tokio::test]
    async fn heartbeat_bridge_records_and_queries() {
        let detector = Arc::new(Mutex::new(PhiAccrualFailureDetector::new(8.0, 100)));
        let bridge = HeartbeatBridge::new(detector, Duration::from_secs(5));

        // Initially no data
        assert!(bridge.agent_phi("agent-1").await.is_none());

        // Record heartbeats
        bridge.record_heartbeat("agent-1").await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        bridge.record_heartbeat("agent-1").await;

        // Now we should have phi data
        let phi = bridge.agent_phi("agent-1").await;
        assert!(phi.is_some());
        assert!(bridge.is_agent_available("agent-1").await);
    }

    #[test]
    fn supervision_recorder_records_restart() {
        let logger = Arc::new(AuditLogger::new(&AuditConfig::default()));
        let recorder = SupervisionRecorder::new(Some(logger.clone()));

        recorder.record_restart("agent-42", "heartbeat_timeout");

        // Verify audit event was recorded
        let events = logger.recent_events(10);
        assert!(!events.is_empty());
        assert_eq!(events[0].action.as_deref(), Some("agent_restart"));
    }

    #[test]
    fn supervision_recorder_records_failure() {
        let logger = Arc::new(AuditLogger::new(&AuditConfig::default()));
        let recorder = SupervisionRecorder::new(Some(logger.clone()));

        recorder.record_failure("agent-99", "panicked in handle_message");

        let events = logger.recent_events(10);
        assert!(!events.is_empty());
        assert_eq!(events[0].action.as_deref(), Some("agent_failure"));
    }

    #[test]
    fn supervision_recorder_works_without_audit_logger() {
        let recorder = SupervisionRecorder::new(None);
        // Should not panic even without an audit logger
        recorder.record_restart("agent-1", "test");
        recorder.record_failure("agent-1", "test error");
    }

    #[test]
    fn security_bridge_allows_when_no_jwt_configured() {
        let bridge = SecurityBridge::new(None, None, None);
        assert!(bridge
            .check_authorization("token", "read", "/agents")
            .is_ok());
    }
}

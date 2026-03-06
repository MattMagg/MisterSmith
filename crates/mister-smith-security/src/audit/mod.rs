//! Tamper-evident audit logging with SHA-256 hash chaining.
//!
//! [`AuditLogger`] records security events into an in-memory ring buffer,
//! maintaining a cryptographic hash chain so that any post-hoc modification
//! is detectable via [`AuditLogger::verify_chain`].
//!
//! Persistence to durable storage is deferred to Phase 6.

pub mod events;

pub use events::{AuditEventType, AuditOutcome, SecurityAuditEvent};

use crate::config::AuditConfig;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, warn};

/// In-memory, thread-safe audit logger with hash-chained event integrity.
///
/// Events are stored in a bounded [`VecDeque`]; when the configured capacity
/// is reached the oldest events are evicted. The hash chain links each event
/// to its predecessor so [`verify_chain`](Self::verify_chain) can detect
/// tampering anywhere in the retained window.
///
/// # Thread Safety
///
/// All public methods acquire the internal [`RwLock`] and are safe to call
/// from any thread or async task.
pub struct AuditLogger {
    /// Ring buffer of recorded events.
    events: RwLock<VecDeque<SecurityAuditEvent>>,
    /// Maximum number of events to retain.
    max_events: usize,
    /// Auth-failure count per source IP per minute before an alert fires.
    alert_threshold: u32,
}

impl AuditLogger {
    /// Create a new `AuditLogger` from the given configuration.
    pub fn new(config: &AuditConfig) -> Self {
        debug!(
            max_events = config.max_events,
            alert_threshold = config.auth_failure_alert_threshold,
            "AuditLogger initialized"
        );
        Self {
            events: RwLock::new(VecDeque::with_capacity(config.max_events)),
            max_events: config.max_events,
            alert_threshold: config.auth_failure_alert_threshold,
        }
    }

    /// Record a security event into the audit log.
    ///
    /// Before appending, the SHA-256 digest of the most recent existing event
    /// is computed and written into the new event's `previous_hash` field,
    /// extending the hash chain. If the buffer is at capacity the oldest event
    /// is evicted.
    pub fn record(&self, mut event: SecurityAuditEvent) {
        let mut events = self.events.write();

        // Compute the hash of the previous event (if any) for chain integrity.
        event.previous_hash = events.back().map(|prev| {
            let serialized =
                serde_json::to_string(prev).expect("SecurityAuditEvent must be serializable");
            hex::encode(Sha256::digest(serialized.as_bytes()))
        });

        debug!(
            event_id = %event.event_id,
            event_type = ?event.event_type,
            outcome = ?event.outcome,
            "audit event recorded"
        );

        events.push_back(event);

        // Enforce capacity by removing the oldest events.
        while events.len() > self.max_events {
            events.pop_front();
        }
    }

    /// Convenience method to record an authentication event.
    pub fn record_auth(
        &self,
        principal: &str,
        outcome: AuditOutcome,
        details: HashMap<String, String>,
    ) {
        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authentication,
            principal: Some(principal.to_string()),
            resource: None,
            action: Some("authenticate".to_string()),
            outcome,
            details,
            source_ip: None,
            previous_hash: None, // Set by `record`.
        };
        self.record(event);
    }

    /// Convenience method to record an authorization event.
    pub fn record_authz(
        &self,
        principal: &str,
        action: &str,
        resource: &str,
        outcome: AuditOutcome,
    ) {
        let event = SecurityAuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authorization,
            principal: Some(principal.to_string()),
            resource: Some(resource.to_string()),
            action: Some(action.to_string()),
            outcome,
            details: HashMap::new(),
            source_ip: None,
            previous_hash: None, // Set by `record`.
        };
        self.record(event);
    }

    /// Return the most recent `limit` events (newest last).
    pub fn recent_events(&self, limit: usize) -> Vec<SecurityAuditEvent> {
        let events = self.events.read();
        let start = events.len().saturating_sub(limit);
        events.iter().skip(start).cloned().collect()
    }

    /// Verify the integrity of the hash chain.
    ///
    /// Walks the event buffer from oldest to newest and recomputes each
    /// `previous_hash`. Returns `Ok(())` if the chain is intact, or
    /// `Err(index)` indicating the zero-based position of the first event
    /// whose `previous_hash` does not match the expected digest.
    pub fn verify_chain(&self) -> Result<(), usize> {
        let events = self.events.read();

        for i in 0..events.len() {
            if i == 0 {
                // The first retained event may or may not have a previous_hash
                // (earlier events may have been evicted). We cannot verify it,
                // so we skip.
                continue;
            }

            let prev = &events[i - 1];
            let expected = {
                let serialized =
                    serde_json::to_string(prev).expect("SecurityAuditEvent must be serializable");
                hex::encode(Sha256::digest(serialized.as_bytes()))
            };

            match &events[i].previous_hash {
                Some(hash) if hash == &expected => {} // Chain intact.
                _ => return Err(i),
            }
        }

        Ok(())
    }

    /// Drain all events from the buffer, returning them for external persistence.
    ///
    /// The events are removed from the in-memory ring buffer. Callers (such as
    /// the Phase 6 audit bridge) should persist the drained events to durable
    /// storage before they are lost.
    pub fn drain_events(&self) -> Vec<SecurityAuditEvent> {
        let mut events = self.events.write();
        events.drain(..).collect()
    }

    /// Detect sources with authentication failures exceeding the alert threshold.
    ///
    /// Scans events from the last 60 seconds, groups `Authentication` failures
    /// by `source_ip`, and returns a [`SuspiciousActivity`](AuditEventType::SuspiciousActivity)
    /// alert event for each source that exceeds the configured threshold.
    pub fn check_alerts(&self) -> Vec<SecurityAuditEvent> {
        let events = self.events.read();
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(60);

        // Count auth failures per source IP in the last minute.
        let mut failure_counts: HashMap<String, u32> = HashMap::new();
        for event in events.iter() {
            if event.timestamp < cutoff {
                continue;
            }
            if event.event_type == AuditEventType::Authentication
                && event.outcome == AuditOutcome::Failure
            {
                let source = event
                    .source_ip
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                *failure_counts.entry(source).or_insert(0) += 1;
            }
        }

        // Generate alert events for sources exceeding the threshold.
        let mut alerts = Vec::new();
        for (source, count) in &failure_counts {
            if *count >= self.alert_threshold {
                warn!(
                    source_ip = %source,
                    failure_count = count,
                    threshold = self.alert_threshold,
                    "auth failure alert threshold exceeded"
                );
                let mut details = HashMap::new();
                details.insert("failure_count".to_string(), count.to_string());
                details.insert("threshold".to_string(), self.alert_threshold.to_string());
                details.insert("window_seconds".to_string(), "60".to_string());

                alerts.push(SecurityAuditEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    event_type: AuditEventType::SuspiciousActivity,
                    principal: None,
                    resource: None,
                    action: Some("auth_failure_alert".to_string()),
                    outcome: AuditOutcome::Warning,
                    details,
                    source_ip: Some(source.clone()),
                    previous_hash: None,
                });
            }
        }

        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_logger() -> AuditLogger {
        AuditLogger::new(&AuditConfig::default())
    }

    fn make_event(event_type: AuditEventType, outcome: AuditOutcome) -> SecurityAuditEvent {
        SecurityAuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type,
            principal: Some("agent-1".to_string()),
            resource: None,
            action: Some("test".to_string()),
            outcome,
            details: HashMap::new(),
            source_ip: None,
            previous_hash: None,
        }
    }

    #[test]
    fn record_and_retrieve() {
        let logger = default_logger();
        logger.record(make_event(
            AuditEventType::Authentication,
            AuditOutcome::Success,
        ));
        logger.record(make_event(
            AuditEventType::Authorization,
            AuditOutcome::Failure,
        ));

        let recent = logger.recent_events(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].event_type, AuditEventType::Authentication);
        assert_eq!(recent[1].event_type, AuditEventType::Authorization);
    }

    #[test]
    fn recent_events_respects_limit() {
        let logger = default_logger();
        for _ in 0..5 {
            logger.record(make_event(
                AuditEventType::SystemAccess,
                AuditOutcome::Success,
            ));
        }
        let recent = logger.recent_events(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn hash_chain_integrity() {
        let logger = default_logger();
        logger.record(make_event(
            AuditEventType::Authentication,
            AuditOutcome::Success,
        ));
        logger.record(make_event(
            AuditEventType::Authorization,
            AuditOutcome::Success,
        ));
        logger.record(make_event(
            AuditEventType::TokenLifecycle,
            AuditOutcome::Success,
        ));

        assert!(logger.verify_chain().is_ok());
    }

    #[test]
    fn hash_chain_detects_tampering() {
        let logger = default_logger();
        logger.record(make_event(
            AuditEventType::Authentication,
            AuditOutcome::Success,
        ));
        logger.record(make_event(
            AuditEventType::Authorization,
            AuditOutcome::Success,
        ));
        logger.record(make_event(
            AuditEventType::TokenLifecycle,
            AuditOutcome::Success,
        ));

        // Tamper with the second event.
        {
            let mut events = logger.events.write();
            events[1].action = Some("tampered".to_string());
        }

        let result = logger.verify_chain();
        assert_eq!(result, Err(2)); // Third event's hash no longer matches.
    }

    #[test]
    fn max_events_capacity_enforced() {
        let config = AuditConfig {
            enabled: true,
            max_events: 3,
            auth_failure_alert_threshold: 5,
        };
        let logger = AuditLogger::new(&config);

        for _ in 0..5 {
            logger.record(make_event(
                AuditEventType::SystemAccess,
                AuditOutcome::Success,
            ));
        }

        let events = logger.recent_events(100);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn record_auth_convenience() {
        let logger = default_logger();
        let mut details = HashMap::new();
        details.insert("method".to_string(), "jwt".to_string());
        logger.record_auth("agent-42", AuditOutcome::Success, details);

        let recent = logger.recent_events(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].event_type, AuditEventType::Authentication);
        assert_eq!(recent[0].principal.as_deref(), Some("agent-42"));
        assert_eq!(
            recent[0].details.get("method").map(String::as_str),
            Some("jwt")
        );
    }

    #[test]
    fn record_authz_convenience() {
        let logger = default_logger();
        logger.record_authz("agent-7", "read", "config://tls", AuditOutcome::Failure);

        let recent = logger.recent_events(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].event_type, AuditEventType::Authorization);
        assert_eq!(recent[0].action.as_deref(), Some("read"));
        assert_eq!(recent[0].resource.as_deref(), Some("config://tls"));
        assert_eq!(recent[0].outcome, AuditOutcome::Failure);
    }

    #[test]
    fn check_alerts_below_threshold() {
        let logger = default_logger();
        // Record 4 failures from same source (default threshold is 5).
        for _ in 0..4 {
            let mut event = make_event(AuditEventType::Authentication, AuditOutcome::Failure);
            event.source_ip = Some("10.0.0.1".to_string());
            logger.record(event);
        }
        let alerts = logger.check_alerts();
        assert!(alerts.is_empty());
    }

    #[test]
    fn check_alerts_at_threshold() {
        let logger = default_logger();
        // Record exactly 5 failures from same source.
        for _ in 0..5 {
            let mut event = make_event(AuditEventType::Authentication, AuditOutcome::Failure);
            event.source_ip = Some("10.0.0.1".to_string());
            logger.record(event);
        }
        let alerts = logger.check_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].event_type, AuditEventType::SuspiciousActivity);
        assert_eq!(alerts[0].source_ip.as_deref(), Some("10.0.0.1"));
    }

    #[test]
    fn check_alerts_multiple_sources() {
        let config = AuditConfig {
            enabled: true,
            max_events: 1000,
            auth_failure_alert_threshold: 3,
        };
        let logger = AuditLogger::new(&config);

        // 4 failures from source A, 2 from source B.
        for _ in 0..4 {
            let mut event = make_event(AuditEventType::Authentication, AuditOutcome::Failure);
            event.source_ip = Some("10.0.0.1".to_string());
            logger.record(event);
        }
        for _ in 0..2 {
            let mut event = make_event(AuditEventType::Authentication, AuditOutcome::Failure);
            event.source_ip = Some("10.0.0.2".to_string());
            logger.record(event);
        }

        let alerts = logger.check_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].source_ip.as_deref(), Some("10.0.0.1"));
    }

    #[test]
    fn verify_chain_single_event() {
        let logger = default_logger();
        logger.record(make_event(
            AuditEventType::Authentication,
            AuditOutcome::Success,
        ));
        assert!(logger.verify_chain().is_ok());
    }

    #[test]
    fn verify_chain_empty() {
        let logger = default_logger();
        assert!(logger.verify_chain().is_ok());
    }

    #[test]
    fn previous_hash_is_set_on_record() {
        let logger = default_logger();
        logger.record(make_event(
            AuditEventType::Authentication,
            AuditOutcome::Success,
        ));
        logger.record(make_event(
            AuditEventType::Authorization,
            AuditOutcome::Success,
        ));

        let events = logger.recent_events(10);
        assert!(events[0].previous_hash.is_none());
        assert!(events[1].previous_hash.is_some());
    }
}

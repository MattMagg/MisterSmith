//! Audit persistence bridge — drains Phase 5 AuditLogger ring buffer to PostgreSQL.
//!
//! `AuditPersister` periodically reads new `SecurityAuditEvent`s from the
//! in-memory ring buffer maintained by Phase 5's `AuditLogger` and batch-writes
//! them to the `audit_log` table via `AuditRepository`.
//!
//! Feature-gated behind the `security` feature flag.

#[cfg(all(feature = "security", feature = "sqlx"))]
mod inner {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::Mutex;
    use tracing::{debug, error};
    use uuid::Uuid;

    use mister_smith_core::PersistenceError;
    use mister_smith_security::audit::{AuditLogger, SecurityAuditEvent};

    use crate::postgres::queries::AuditEntry;
    use crate::repository::audit::AuditRepository;

    /// Drains Phase 5 AuditLogger events to PostgreSQL.
    ///
    /// Runs a background task that periodically reads new events from the
    /// in-memory ring buffer and batch-inserts them into the `audit_log` table.
    pub struct AuditPersister {
        logger: Arc<AuditLogger>,
        repository: AuditRepository,
        /// Tracks event IDs already persisted to avoid duplicates.
        persisted_ids: Mutex<HashSet<String>>,
        /// How often to flush events to PostgreSQL.
        flush_interval: Duration,
        /// Maximum events to read per flush cycle.
        batch_size: usize,
    }

    impl AuditPersister {
        /// Create a new AuditPersister.
        ///
        /// # Arguments
        /// - `logger`: Arc reference to the Phase 5 AuditLogger ring buffer
        /// - `repository`: AuditRepository for PostgreSQL writes
        /// - `flush_interval`: How often to drain events (e.g., 5 seconds)
        /// - `batch_size`: Max events to read per cycle (e.g., 1000)
        pub fn new(
            logger: Arc<AuditLogger>,
            repository: AuditRepository,
            flush_interval: Duration,
            batch_size: usize,
        ) -> Self {
            Self {
                logger,
                repository,
                persisted_ids: Mutex::new(HashSet::new()),
                flush_interval,
                batch_size,
            }
        }

        /// Start the background flush task.
        ///
        /// Spawns a `tokio::task` that periodically drains new events.
        /// Returns a `JoinHandle` that can be used to await or abort the task.
        pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
            let interval = self.flush_interval;
            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval);
                loop {
                    ticker.tick().await;
                    if let Err(e) = self.flush().await {
                        error!(error = %e, "Audit flush to PostgreSQL failed");
                    }
                }
            })
        }

        /// Flush new events from the ring buffer to PostgreSQL.
        ///
        /// Holds the persisted_ids lock across the entire operation (filter +
        /// write + record) to prevent TOCTOU races with concurrent flushes.
        pub async fn flush(&self) -> Result<usize, PersistenceError> {
            let events = self.logger.recent_events(self.batch_size);
            if events.is_empty() {
                return Ok(0);
            }

            // Hold lock across entire flush to prevent TOCTOU race
            let mut persisted = self.persisted_ids.lock().await;

            // Filter out already-persisted events
            let new_events: Vec<&SecurityAuditEvent> = events
                .iter()
                .filter(|e| !persisted.contains(&e.event_id))
                .collect();

            if new_events.is_empty() {
                return Ok(0);
            }

            // Convert to AuditEntry
            let entries: Vec<AuditEntry> =
                new_events.iter().map(|e| Self::convert_event(e)).collect();

            // Batch insert
            let count = self.repository.append_batch(&entries).await?;

            let new_ids: HashSet<String> = new_events
                .into_iter()
                .map(|event| event.event_id.clone())
                .collect();

            // Prevent unbounded growth — if tracking set exceeds 2x batch size,
            // only keep the newly-persisted IDs (not all ring buffer events).
            if persisted.len() + new_ids.len() > self.batch_size * 2 {
                *persisted = new_ids;
            } else {
                persisted.extend(new_ids);
            }

            debug!(
                count = count,
                total_events = events.len(),
                "Flushed audit events to PostgreSQL"
            );

            Ok(count)
        }

        /// Convert a Phase 5 `SecurityAuditEvent` to a persistence `AuditEntry`.
        pub fn convert_event(event: &SecurityAuditEvent) -> AuditEntry {
            let id = Uuid::parse_str(&event.event_id).unwrap_or_else(|_| Uuid::new_v4());

            // Map event_type enum to string
            let event_type = format!("{:?}", event.event_type);

            // Action: use the event's action field or derive from event_type
            let action = event.action.clone().unwrap_or_else(|| event_type.clone());

            // Build metadata from details + source_ip + outcome
            let mut meta = serde_json::Map::new();
            for (k, v) in &event.details {
                meta.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            if let Some(ref ip) = event.source_ip {
                meta.insert(
                    "source_ip".to_string(),
                    serde_json::Value::String(ip.clone()),
                );
            }
            meta.insert(
                "outcome".to_string(),
                serde_json::Value::String(format!("{:?}", event.outcome)),
            );
            if let Some(ref hash) = event.previous_hash {
                meta.insert(
                    "previous_hash".to_string(),
                    serde_json::Value::String(hash.clone()),
                );
            }

            // Try to parse principal as UUID for agent_id
            let agent_id = event
                .principal
                .as_ref()
                .and_then(|p| Uuid::parse_str(p).ok());

            AuditEntry {
                id,
                event_type,
                agent_id,
                resource_type: Some("security".to_string()),
                resource_id: None,
                action,
                old_values: None,
                new_values: None,
                metadata: serde_json::Value::Object(meta),
                correlation_id: None,
                created_at: event.timestamp,
            }
        }

        /// Get the number of events that have been persisted.
        pub async fn persisted_count(&self) -> usize {
            self.persisted_ids.lock().await.len()
        }
    }
}

#[cfg(all(feature = "security", feature = "sqlx"))]
pub use inner::AuditPersister;

#[cfg(test)]
#[cfg(all(feature = "security", feature = "sqlx"))]
mod tests {
    use super::*;
    use mister_smith_security::audit::events::{AuditEventType, AuditOutcome, SecurityAuditEvent};
    use std::collections::HashMap;

    fn sample_security_event() -> SecurityAuditEvent {
        SecurityAuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authentication,
            principal: Some("test-user".to_string()),
            resource: Some("/api/agents".to_string()),
            action: Some("login".to_string()),
            outcome: AuditOutcome::Success,
            details: {
                let mut m = HashMap::new();
                m.insert("method".to_string(), "jwt".to_string());
                m
            },
            source_ip: Some("192.168.1.1".to_string()),
            previous_hash: None,
        }
    }

    #[test]
    fn convert_security_event_to_audit_entry() {
        let event = sample_security_event();
        let entry = AuditPersister::convert_event(&event);

        assert_eq!(entry.event_type, "Authentication");
        assert_eq!(entry.action, "login");
        assert_eq!(entry.resource_type, Some("security".to_string()));
        assert!(entry.agent_id.is_none()); // "test-user" is not a UUID
        assert!(entry.old_values.is_none());
        assert!(entry.new_values.is_none());

        // Verify metadata includes details, source_ip, outcome
        let meta = entry.metadata.as_object().unwrap();
        assert_eq!(meta.get("method").unwrap().as_str().unwrap(), "jwt");
        assert_eq!(
            meta.get("source_ip").unwrap().as_str().unwrap(),
            "192.168.1.1"
        );
        assert_eq!(meta.get("outcome").unwrap().as_str().unwrap(), "Success");
    }

    #[test]
    fn convert_event_with_uuid_principal() {
        let agent_id = uuid::Uuid::new_v4();
        let mut event = sample_security_event();
        event.principal = Some(agent_id.to_string());

        let entry = AuditPersister::convert_event(&event);
        assert_eq!(entry.agent_id, Some(agent_id));
    }

    #[test]
    fn convert_event_without_action_uses_event_type() {
        let mut event = sample_security_event();
        event.action = None;

        let entry = AuditPersister::convert_event(&event);
        assert_eq!(entry.action, "Authentication");
    }
}

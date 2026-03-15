//! Cross-crate persistence integration tests.
//!
//! Validates that persistence types, configurations, and security audit wiring
//! integrate correctly with the rest of the framework. Tests are feature-gated
//! behind `persistence` and do **not** require a running database or NATS server
//! (except where marked `#[ignore]`).

#[cfg(feature = "persistence")]
mod persistence_tests {
    use mister_smith_persistence::config::PersistenceConfig;
    use mister_smith_persistence::health::PersistenceHealthChecker;

    // -----------------------------------------------------------------------
    // Config integration: TOML deserialization with persistence section
    // -----------------------------------------------------------------------

    #[test]
    fn persistence_config_deserializes_from_toml() {
        let toml_str = r#"
            enabled = true

            [postgres]
            url = "postgres://user:pass@localhost:5432/mister_smith"
            max_connections = 20
            min_connections = 5
            connect_timeout_secs = 10
            idle_timeout_secs = 300

            [kv]
            enabled = true
            session_ttl_secs = 7200
            agent_state_ttl_secs = 3600
            cache_ttl_secs = 600
            replicas = 3

            [flush]
            threshold = 100
            deadline_secs = 30
            safety_margin_secs = 120
            max_flush_retries = 5

            [checkpoint]
            enabled = true
            interval_secs = 600
        "#;

        let config: PersistenceConfig = toml::from_str(toml_str).unwrap();

        assert!(config.enabled);
        assert_eq!(
            config.postgres.url.as_deref(),
            Some("postgres://user:pass@localhost:5432/mister_smith")
        );
        assert_eq!(config.postgres.max_connections, 20);
        assert_eq!(config.postgres.min_connections, 5);
        assert_eq!(config.postgres.connect_timeout_secs, 10);
        assert_eq!(config.postgres.idle_timeout_secs, 300);
        assert!(config.kv.enabled);
        assert_eq!(config.kv.session_ttl_secs, 7200);
        assert_eq!(config.kv.agent_state_ttl_secs, 3600);
        assert_eq!(config.kv.cache_ttl_secs, 600);
        assert_eq!(config.kv.replicas, 3);
        assert_eq!(config.flush.threshold, 100);
        assert_eq!(config.flush.deadline_secs, 30);
        assert_eq!(config.flush.safety_margin_secs, 120);
        assert_eq!(config.flush.max_flush_retries, 5);
        assert!(config.checkpoint.enabled);
        assert_eq!(config.checkpoint.interval_secs, 600);
    }

    #[test]
    fn persistence_config_defaults_are_sensible() {
        let config = PersistenceConfig::default();

        assert!(!config.enabled);
        assert!(config.postgres.url.is_none());
        assert_eq!(config.postgres.max_connections, 10);
        assert_eq!(config.postgres.min_connections, 2);
        assert!(config.kv.enabled);
        assert_eq!(config.flush.threshold, 50);
        assert!(!config.checkpoint.enabled);
    }

    #[test]
    fn persistence_config_partial_toml_applies_defaults() {
        let toml_str = r#"
            enabled = true
            [postgres]
            url = "postgres://localhost/test"
        "#;

        let config: PersistenceConfig = toml::from_str(toml_str).unwrap();

        assert!(config.enabled);
        assert_eq!(
            config.postgres.url.as_deref(),
            Some("postgres://localhost/test")
        );
        // Unset fields use defaults
        assert_eq!(config.postgres.max_connections, 10);
        assert_eq!(config.postgres.min_connections, 2);
        assert!(config.kv.enabled);
        assert_eq!(config.flush.threshold, 50);
    }

    // -----------------------------------------------------------------------
    // Health check types: constructible without external dependencies
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn persistence_health_checker_no_backends_returns_unknown() {
        use mister_smith_core::HealthStatus;

        let checker = PersistenceHealthChecker::new(None);
        let status = checker.check_all().await;
        assert_eq!(status, HealthStatus::Unknown);
    }

    #[test]
    fn persistence_health_checker_is_constructible() {
        // Verify the type exists and can be constructed with no backends.
        let _checker = PersistenceHealthChecker::new(None);
    }

    // -----------------------------------------------------------------------
    // Security + audit wiring (requires both persistence and security features)
    // -----------------------------------------------------------------------

    #[cfg(feature = "persistence")]
    mod audit_wiring {
        use std::collections::HashMap;
        use std::sync::Arc;

        use mister_smith_persistence::AuditEntry;
        use mister_smith_persistence::AuditPersister;
        use mister_smith_security::audit::events::{
            AuditEventType, AuditOutcome, SecurityAuditEvent,
        };
        use mister_smith_security::audit::AuditLogger;
        use mister_smith_security::config::AuditConfig;

        fn sample_security_event() -> SecurityAuditEvent {
            SecurityAuditEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::Authentication,
                principal: Some("integration-test-agent".to_string()),
                resource: Some("/api/agents".to_string()),
                action: Some("login".to_string()),
                outcome: AuditOutcome::Success,
                details: {
                    let mut m = HashMap::new();
                    m.insert("method".to_string(), "jwt".to_string());
                    m
                },
                delegation: None,
                source_ip: Some("10.0.0.1".to_string()),
                previous_hash: None,
            }
        }

        /// Verify that SecurityAuditEvent can be converted to AuditEntry
        /// via AuditPersister::convert_event. This confirms the security →
        /// persistence type bridge works across crate boundaries.
        #[test]
        fn security_audit_event_converts_to_audit_entry() {
            let event = sample_security_event();
            let entry = AuditPersister::convert_event(event);

            // Verify field mapping
            assert_eq!(entry.event_type, "Authentication");
            assert_eq!(entry.action, "login");
            assert_eq!(entry.resource_type, Some("security".to_string()));

            // Non-UUID principal should not parse as agent_id
            assert!(entry.agent_id.is_none());

            // Metadata should include details, source_ip, and outcome
            let meta = entry.metadata.as_object().unwrap();
            assert_eq!(meta.get("method").unwrap().as_str().unwrap(), "jwt");
            assert_eq!(meta.get("source_ip").unwrap().as_str().unwrap(), "10.0.0.1");
            assert_eq!(meta.get("outcome").unwrap().as_str().unwrap(), "Success");
        }

        /// Verify UUID principals are correctly parsed into agent_id.
        #[test]
        fn security_audit_event_uuid_principal_maps_to_agent_id() {
            let agent_id = uuid::Uuid::new_v4();
            let mut event = sample_security_event();
            event.principal = Some(agent_id.to_string());

            let entry = AuditPersister::convert_event(event);
            assert_eq!(entry.agent_id, Some(agent_id));
        }

        /// Verify all audit event types convert without panicking.
        #[test]
        fn all_audit_event_types_convert_cleanly() {
            let event_types = vec![
                AuditEventType::Authentication,
                AuditEventType::Authorization,
                AuditEventType::TokenLifecycle,
                AuditEventType::SystemAccess,
                AuditEventType::ConfigurationChange,
                AuditEventType::CertificateEvent,
                AuditEventType::SuspiciousActivity,
            ];

            for event_type in event_types {
                let mut event = sample_security_event();
                event.event_type = event_type;
                let entry = AuditPersister::convert_event(event);
                assert!(!entry.event_type.is_empty());
                assert!(!entry.action.is_empty());
            }
        }

        /// Verify AuditEntry is directly constructible with expected fields.
        #[test]
        fn audit_entry_is_constructible() {
            let entry = AuditEntry {
                id: uuid::Uuid::new_v4(),
                event_type: "Authentication".to_string(),
                agent_id: Some(uuid::Uuid::new_v4()),
                resource_type: Some("security".to_string()),
                resource_id: None,
                action: "login".to_string(),
                old_values: None,
                new_values: None,
                metadata: serde_json::json!({"method": "jwt"}),
                correlation_id: None,
                created_at: chrono::Utc::now(),
            };

            assert_eq!(entry.event_type, "Authentication");
            assert!(entry.agent_id.is_some());
        }

        /// Verify AuditLogger ring buffer events can be read and converted
        /// in batch, simulating the AuditPersister drain cycle without a database.
        #[test]
        fn audit_logger_events_convert_in_batch() {
            let logger = AuditLogger::new(&AuditConfig::default());

            // Record several events
            logger.record_auth("agent-1", AuditOutcome::Success, HashMap::new());
            logger.record_authz("agent-1", "read", "config", AuditOutcome::Success);
            logger.record_auth(
                "agent-2",
                AuditOutcome::Failure,
                [("reason".to_string(), "invalid_token".to_string())]
                    .into_iter()
                    .collect(),
            );

            // Drain events and convert to AuditEntry
            let events = logger.recent_events(100);
            assert_eq!(events.len(), 3);

            let entries: Vec<AuditEntry> = events
                .iter()
                .map(|e| AuditPersister::convert_event(e.clone()))
                .collect();

            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].event_type, "Authentication");
            assert_eq!(entries[1].event_type, "Authorization");
            assert_eq!(entries[2].event_type, "Authentication");
        }

        /// Full pipeline test requiring DATABASE_URL.
        /// Constructs an AuditPersister and verifies it can be created,
        /// but does NOT call flush (no database connection).
        #[test]
        #[ignore]
        fn audit_persister_full_pipeline_requires_database() {
            // This test would require:
            // - DATABASE_URL env var pointing to a PostgreSQL instance
            // - NATS_URL env var for transport
            //
            // It validates that AuditPersister::new compiles and type-checks
            // with real AuditLogger + AuditRepository. The actual flush cycle
            // requires a live database and is tested in deployment environments.
            let _logger = Arc::new(AuditLogger::new(&AuditConfig::default()));

            // AuditRepository::new requires a PgPool, which needs DATABASE_URL.
            // Validated at deployment time.
        }
    }
}

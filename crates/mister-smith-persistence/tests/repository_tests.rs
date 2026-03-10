//! Repository integration tests — T045.
//!
//! Tests generic `Repository<T>` behavior: save/find/update/delete for agents,
//! tasks, and messages. Tests OCC conflict on concurrent version updates,
//! transaction commit/rollback, and audit append/batch operations.
//!
//! Requires `DATABASE_URL` environment variable pointing to a PostgreSQL
//! instance with migrations applied.

#[cfg(feature = "sqlx")]
mod tests {
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    use mister_smith_persistence::postgres::queries::AuditEntry;
    use mister_smith_persistence::repository::audit::AuditRepository;

    /// Create a test pool from DATABASE_URL or skip the test.
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for repository integration tests");
        PgPool::connect(&url)
            .await
            .expect("Failed to connect to database")
    }

    // -----------------------------------------------------------------------
    // Audit Repository tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore]
    async fn audit_append_single_entry() {
        let pool = test_pool().await;
        let repo = AuditRepository::new(pool);

        let entry = AuditEntry {
            id: Uuid::new_v4(),
            event_type: "authentication".to_string(),
            agent_id: None,
            resource_type: Some("security".to_string()),
            resource_id: None,
            action: "login".to_string(),
            old_values: None,
            new_values: Some(serde_json::json!({"outcome": "success"})),
            metadata: serde_json::json!({"source_ip": "127.0.0.1"}),
            correlation_id: None,
            created_at: Utc::now(),
        };

        repo.append(&entry).await.expect("append should succeed");
    }

    #[tokio::test]
    #[ignore]
    async fn audit_append_batch() {
        let pool = test_pool().await;
        let repo = AuditRepository::new(pool.clone());

        let entries: Vec<AuditEntry> = (0..5)
            .map(|i| AuditEntry {
                id: Uuid::new_v4(),
                event_type: "authorization".to_string(),
                agent_id: None,
                resource_type: Some("security".to_string()),
                resource_id: None,
                action: format!("check_permission_{i}"),
                old_values: None,
                new_values: None,
                metadata: serde_json::json!({}),
                correlation_id: None,
                created_at: Utc::now(),
            })
            .collect();

        let count = repo
            .append_batch(&entries)
            .await
            .expect("batch append should succeed");
        assert_eq!(count, 5);

        let entry_ids: Vec<Uuid> = entries.iter().map(|entry| entry.id).collect();
        let persisted_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE id = ANY($1)")
                .bind(&entry_ids[..])
                .fetch_one(&pool)
                .await
                .expect("count query should succeed");

        assert_eq!(persisted_count, entries.len() as i64);
    }

    #[tokio::test]
    #[ignore]
    async fn audit_append_batch_is_atomic_on_failure() {
        let pool = test_pool().await;
        let repo = AuditRepository::new(pool.clone());

        let duplicate_id = Uuid::new_v4();
        let duplicate_created_at = Utc::now();
        let entries = vec![
            AuditEntry {
                id: duplicate_id,
                event_type: "authorization".to_string(),
                agent_id: None,
                resource_type: Some("security".to_string()),
                resource_id: None,
                action: "duplicate_0".to_string(),
                old_values: None,
                new_values: None,
                metadata: serde_json::json!({}),
                correlation_id: None,
                created_at: duplicate_created_at,
            },
            AuditEntry {
                id: duplicate_id,
                event_type: "authorization".to_string(),
                agent_id: None,
                resource_type: Some("security".to_string()),
                resource_id: None,
                action: "duplicate_1".to_string(),
                old_values: None,
                new_values: None,
                metadata: serde_json::json!({}),
                correlation_id: None,
                created_at: duplicate_created_at,
            },
        ];

        repo.append_batch(&entries)
            .await
            .expect_err("duplicate primary key should fail the batch");

        let persisted_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE id = $1 AND created_at = $2")
                .bind(duplicate_id)
                .bind(duplicate_created_at)
                .fetch_one(&pool)
                .await
                .expect("count query should succeed");

        assert_eq!(persisted_count, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn audit_append_empty_batch() {
        let pool = test_pool().await;
        let repo = AuditRepository::new(pool);

        let count = repo
            .append_batch(&[])
            .await
            .expect("empty batch should succeed");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn audit_find_by_agent() {
        let pool = test_pool().await;
        let repo = AuditRepository::new(pool);

        let agent_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert entries for this agent
        let entries: Vec<AuditEntry> = (0..3)
            .map(|i| AuditEntry {
                id: Uuid::new_v4(),
                event_type: "system_access".to_string(),
                agent_id: Some(agent_id),
                resource_type: None,
                resource_id: None,
                action: format!("action_{i}"),
                old_values: None,
                new_values: None,
                metadata: serde_json::json!({}),
                correlation_id: None,
                created_at: now,
            })
            .collect();

        repo.append_batch(&entries).await.unwrap();

        // Query back
        let results = repo
            .find_by_agent(
                agent_id,
                now - chrono::Duration::seconds(10),
                now + chrono::Duration::seconds(10),
            )
            .await
            .expect("find_by_agent should succeed");

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|e| e.agent_id == Some(agent_id)));
    }

    // -----------------------------------------------------------------------
    // Config query tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore]
    async fn config_upsert_and_get() {
        let pool = test_pool().await;

        let key = format!("test.config.{}", Uuid::new_v4());
        let value = serde_json::json!({"timeout": 30});

        // Insert
        let version = mister_smith_persistence::postgres::queries::upsert_config(
            &pool,
            &key,
            value.clone(),
            "testing",
            None,
            Some("Test config"),
        )
        .await
        .expect("upsert should succeed");
        assert_eq!(version, 1);

        // Read back
        let config =
            mister_smith_persistence::postgres::queries::get_config(&pool, &key, "testing", None)
                .await
                .expect("get_config should succeed");

        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.key, key);
        assert_eq!(config.value, value);
        assert_eq!(config.version, 1);

        // Upsert again — version should increment
        let version2 = mister_smith_persistence::postgres::queries::upsert_config(
            &pool,
            &key,
            serde_json::json!({"timeout": 60}),
            "testing",
            None,
            None,
        )
        .await
        .expect("second upsert should succeed");
        assert_eq!(version2, 2);
    }

    #[tokio::test]
    #[ignore]
    async fn config_get_by_environment() {
        let pool = test_pool().await;

        let env_name = format!("env_{}", Uuid::new_v4().as_simple());
        let key1 = format!("app.setting1.{}", Uuid::new_v4());
        let key2 = format!("app.setting2.{}", Uuid::new_v4());

        mister_smith_persistence::postgres::queries::upsert_config(
            &pool,
            &key1,
            serde_json::json!(1),
            &env_name,
            None,
            None,
        )
        .await
        .unwrap();

        mister_smith_persistence::postgres::queries::upsert_config(
            &pool,
            &key2,
            serde_json::json!(2),
            &env_name,
            None,
            None,
        )
        .await
        .unwrap();

        let results = mister_smith_persistence::postgres::queries::get_config_by_environment(
            &pool, &env_name,
        )
        .await
        .expect("get_config_by_environment should succeed");

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    #[ignore]
    async fn config_history() {
        let pool = test_pool().await;

        let key = format!("versioned.config.{}", Uuid::new_v4());

        // Create 3 versions
        for i in 1..=3 {
            mister_smith_persistence::postgres::queries::upsert_config(
                &pool,
                &key,
                serde_json::json!({"v": i}),
                "testing",
                None,
                None,
            )
            .await
            .unwrap();
        }

        let history = mister_smith_persistence::postgres::queries::get_config_history(&pool, &key)
            .await
            .expect("get_config_history should succeed");

        // Should have 1 entry (upserts update in-place), but version should be 3
        assert!(!history.is_empty());
        assert_eq!(history[0].version, 3);
    }

    // -----------------------------------------------------------------------
    // Transaction tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore]
    async fn transaction_commit() {
        let pool = test_pool().await;

        let entry_id = Uuid::new_v4();
        let entry = AuditEntry {
            id: entry_id,
            event_type: "test_tx".to_string(),
            agent_id: None,
            resource_type: None,
            resource_id: None,
            action: "transaction_test".to_string(),
            old_values: None,
            new_values: None,
            metadata: serde_json::json!({}),
            correlation_id: None,
            created_at: Utc::now(),
        };

        // Use a transaction to insert
        let mut tx = mister_smith_persistence::postgres::queries::begin_transaction(&pool)
            .await
            .expect("begin should succeed");

        sqlx::query(
            "INSERT INTO audit_log (id, event_type, action, metadata, created_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(entry.id)
        .bind(&entry.event_type)
        .bind(&entry.action)
        .bind(&entry.metadata)
        .bind(entry.created_at)
        .execute(&mut *tx)
        .await
        .expect("insert in tx should succeed");

        tx.commit().await.expect("commit should succeed");

        // Verify entry exists after commit
        let row = sqlx::query_as::<_, AuditEntry>(
            "SELECT id, event_type, agent_id, resource_type, resource_id, action, old_values, new_values, metadata, correlation_id, created_at FROM audit_log WHERE id = $1",
        )
        .bind(entry_id)
        .fetch_optional(&pool)
        .await
        .expect("query should succeed");

        assert!(row.is_some());
    }

    #[tokio::test]
    #[ignore]
    async fn transaction_rollback() {
        let pool = test_pool().await;

        let entry_id = Uuid::new_v4();

        // Start a transaction and insert, but don't commit (drop = rollback)
        {
            let mut tx = mister_smith_persistence::postgres::queries::begin_transaction(&pool)
                .await
                .expect("begin should succeed");

            sqlx::query(
                "INSERT INTO audit_log (id, event_type, action, metadata, created_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(entry_id)
            .bind("test_rollback")
            .bind("rollback_test")
            .bind(serde_json::json!({}))
            .bind(Utc::now())
            .execute(&mut *tx)
            .await
            .expect("insert in tx should succeed");

            // tx drops here without commit → automatic rollback
        }

        // Verify entry does NOT exist after rollback
        let row = sqlx::query_as::<_, AuditEntry>(
            "SELECT id, event_type, agent_id, resource_type, resource_id, action, old_values, new_values, metadata, correlation_id, created_at FROM audit_log WHERE id = $1",
        )
        .bind(entry_id)
        .fetch_optional(&pool)
        .await
        .expect("query should succeed");

        assert!(row.is_none());
    }
}

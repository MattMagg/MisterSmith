//! Env-gated PostgreSQL integration tests.
//!
//! These tests require a running PostgreSQL instance and `DATABASE_URL` env var.
//! Run with: `DATABASE_URL=postgres://... cargo test -p mister-smith-persistence --test postgres_tests -- --ignored`

use chrono::Utc;
use mister_smith_persistence::postgres::queries::*;
use mister_smith_persistence::postgres::migrations::MigrationRunner;
use mister_smith_core::PersistenceError;
use uuid::Uuid;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Create a fresh PostgreSQL pool and run migrations.
async fn setup_pool() -> sqlx::PgPool {
    let url = database_url().expect("DATABASE_URL required");
    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations
    let runner = MigrationRunner::new(pool.clone());
    runner.run().await.expect("Migrations should succeed");

    pool
}

// ---------------------------------------------------------------------------
// Migration tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn migration_runs_cleanly() {
    let pool = setup_pool().await;
    let runner = MigrationRunner::new(pool.clone());

    // Verify returns true (all migrations applied)
    assert!(runner.verify().await.unwrap());

    // Current version should be set
    let version = runner.current_version().await.unwrap();
    assert!(version.is_some());
    assert!(version.unwrap() > 0);

    // Status lists all migrations
    let statuses = runner.status().await.unwrap();
    assert!(!statuses.is_empty());
    for s in &statuses {
        assert!(s.applied, "Migration {} should be applied", s.version);
    }
}

// ---------------------------------------------------------------------------
// Agent registry CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_registry_insert_and_find() {
    let pool = setup_pool().await;

    let record = AgentRecord {
        agent_id: Uuid::new_v4(),
        agent_type: "orchestrator".to_string(),
        agent_name: format!("test-agent-{}", Uuid::new_v4()),
        status: "initializing".to_string(),
        capabilities: serde_json::json!({"tools": ["search"]}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };

    // Insert
    let inserted = insert_agent(&pool, &record).await.unwrap();
    assert_eq!(inserted.agent_id, record.agent_id);
    assert_eq!(inserted.agent_type, "orchestrator");
    assert_eq!(inserted.status, "initializing");

    // Find
    let found = find_agent(&pool, record.agent_id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.agent_name, record.agent_name);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_registry_update_status() {
    let pool = setup_pool().await;

    let record = AgentRecord {
        agent_id: Uuid::new_v4(),
        agent_type: "worker".to_string(),
        agent_name: format!("test-worker-{}", Uuid::new_v4()),
        status: "initializing".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };

    insert_agent(&pool, &record).await.unwrap();

    // Update status
    update_agent_status(&pool, record.agent_id, "active").await.unwrap();

    let found = find_agent(&pool, record.agent_id).await.unwrap().unwrap();
    assert_eq!(found.status, "active");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_registry_find_by_type_and_status() {
    let pool = setup_pool().await;
    let unique_type = format!("test-type-{}", Uuid::new_v4());

    // Insert two agents of the same type
    for i in 0..2 {
        let record = AgentRecord {
            agent_id: Uuid::new_v4(),
            agent_type: unique_type.clone(),
            agent_name: format!("agent-{i}-{}", Uuid::new_v4()),
            status: "active".to_string(),
            capabilities: serde_json::json!({}),
            configuration: serde_json::json!({}),
            metadata: serde_json::json!({}),
            parent_agent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_heartbeat: None,
        };
        insert_agent(&pool, &record).await.unwrap();
    }

    let by_type = find_agents_by_type(&pool, &unique_type).await.unwrap();
    assert_eq!(by_type.len(), 2);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_registry_not_found() {
    let pool = setup_pool().await;

    let found = find_agent(&pool, Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_registry_update_nonexistent_fails() {
    let pool = setup_pool().await;

    let result = update_agent_status(&pool, Uuid::new_v4(), "active").await;
    assert!(matches!(result, Err(PersistenceError::NotFound(_))));
}

// ---------------------------------------------------------------------------
// Agent state CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_state_upsert_and_get() {
    let pool = setup_pool().await;

    // Create an agent first (FK constraint)
    let agent_id = Uuid::new_v4();
    let record = AgentRecord {
        agent_id,
        agent_type: "worker".to_string(),
        agent_name: format!("state-test-{}", Uuid::new_v4()),
        status: "active".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };
    insert_agent(&pool, &record).await.unwrap();

    // Upsert state
    let version = upsert_state(
        &pool,
        agent_id,
        "conversation.context",
        serde_json::json!({"messages": []}),
        None,
    )
    .await
    .unwrap();
    assert_eq!(version, 1);

    // Get state
    let state = get_state(&pool, agent_id, "conversation.context")
        .await
        .unwrap();
    assert!(state.is_some());
    let state = state.unwrap();
    assert_eq!(state.state_key, "conversation.context");
    assert_eq!(state.version, 1);

    // Upsert again (version should increment)
    let version2 = upsert_state(
        &pool,
        agent_id,
        "conversation.context",
        serde_json::json!({"messages": ["hello"]}),
        Some("sha256hash"),
    )
    .await
    .unwrap();
    assert_eq!(version2, 2);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn agent_state_get_all_and_delete() {
    let pool = setup_pool().await;

    let agent_id = Uuid::new_v4();
    let record = AgentRecord {
        agent_id,
        agent_type: "worker".to_string(),
        agent_name: format!("state-all-{}", Uuid::new_v4()),
        status: "active".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };
    insert_agent(&pool, &record).await.unwrap();

    // Insert multiple state keys
    upsert_state(&pool, agent_id, "key_a", serde_json::json!(1), None)
        .await
        .unwrap();
    upsert_state(&pool, agent_id, "key_b", serde_json::json!(2), None)
        .await
        .unwrap();

    // Get all
    let all = get_all_state(&pool, agent_id).await.unwrap();
    assert_eq!(all.len(), 2);

    // Delete one
    let deleted = delete_state(&pool, agent_id, "key_a").await.unwrap();
    assert!(deleted);

    // Verify only one remains
    let remaining = get_all_state(&pool, agent_id).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].state_key, "key_b");

    // Delete nonexistent
    let deleted_again = delete_state(&pool, agent_id, "key_a").await.unwrap();
    assert!(!deleted_again);
}

// ---------------------------------------------------------------------------
// Checkpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn checkpoint_insert_and_retrieve() {
    let pool = setup_pool().await;

    let agent_id = Uuid::new_v4();
    let record = AgentRecord {
        agent_id,
        agent_type: "worker".to_string(),
        agent_name: format!("ckpt-test-{}", Uuid::new_v4()),
        status: "active".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };
    insert_agent(&pool, &record).await.unwrap();

    // Insert checkpoint
    let snapshot = serde_json::json!({"key_a": 1, "key_b": 2});
    let ckpt_id = insert_checkpoint(&pool, agent_id, snapshot.clone(), Some(42))
        .await
        .unwrap();

    // Retrieve latest
    let latest = get_latest_checkpoint(&pool, agent_id).await.unwrap();
    assert!(latest.is_some());
    let latest = latest.unwrap();
    assert_eq!(latest.checkpoint_id, ckpt_id);
    assert_eq!(latest.state_snapshot, snapshot);
    assert_eq!(latest.kv_revision, Some(42));
}

// ---------------------------------------------------------------------------
// Task CRUD (T040)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn task_insert_and_find() {
    let pool = setup_pool().await;

    let record = TaskRecord {
        task_id: Uuid::new_v4(),
        task_type: "research".to_string(),
        agent_id: None,
        payload: serde_json::json!({"query": "test search"}),
        result: None,
        metadata: serde_json::json!({}),
        status: "pending".to_string(),
        priority: 2,
        correlation_id: None,
        parent_task_id: None,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        expires_at: None,
    };

    // Insert
    let inserted = insert_task(&pool, &record).await.unwrap();
    assert_eq!(inserted.task_id, record.task_id);
    assert_eq!(inserted.task_type, "research");
    assert_eq!(inserted.status, "pending");
    assert_eq!(inserted.priority, 2);

    // Find
    let found = find_task(&pool, record.task_id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.task_type, "research");
    assert_eq!(found.payload, record.payload);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn task_update_status() {
    let pool = setup_pool().await;

    let record = TaskRecord {
        task_id: Uuid::new_v4(),
        task_type: "analysis".to_string(),
        agent_id: None,
        payload: serde_json::json!({}),
        result: None,
        metadata: serde_json::json!({}),
        status: "pending".to_string(),
        priority: 1,
        correlation_id: None,
        parent_task_id: None,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        expires_at: None,
    };

    insert_task(&pool, &record).await.unwrap();

    // Update status
    update_task_status(&pool, record.task_id, "running").await.unwrap();

    let found = find_task(&pool, record.task_id).await.unwrap().unwrap();
    assert_eq!(found.status, "running");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn task_find_by_agent() {
    let pool = setup_pool().await;

    let agent_id = Uuid::new_v4();

    // Insert two tasks for the same agent
    for i in 0..2 {
        let record = TaskRecord {
            task_id: Uuid::new_v4(),
            task_type: format!("task-type-{i}"),
            agent_id: Some(agent_id),
            payload: serde_json::json!({}),
            result: None,
            metadata: serde_json::json!({}),
            status: "pending".to_string(),
            priority: i,
            correlation_id: None,
            parent_task_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            expires_at: None,
        };
        insert_task(&pool, &record).await.unwrap();
    }

    let tasks = find_tasks_by_agent(&pool, agent_id).await.unwrap();
    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().all(|t| t.agent_id == Some(agent_id)));
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn task_find_by_correlation() {
    let pool = setup_pool().await;

    let correlation_id = Uuid::new_v4();

    let record = TaskRecord {
        task_id: Uuid::new_v4(),
        task_type: "correlated".to_string(),
        agent_id: None,
        payload: serde_json::json!({}),
        result: None,
        metadata: serde_json::json!({}),
        status: "pending".to_string(),
        priority: 0,
        correlation_id: Some(correlation_id),
        parent_task_id: None,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        expires_at: None,
    };

    insert_task(&pool, &record).await.unwrap();

    let tasks = find_tasks_by_correlation(&pool, correlation_id).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, record.task_id);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn task_not_found() {
    let pool = setup_pool().await;

    let found = find_task(&pool, Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn task_update_nonexistent_fails() {
    let pool = setup_pool().await;

    let result = update_task_status(&pool, Uuid::new_v4(), "running").await;
    assert!(matches!(result, Err(PersistenceError::NotFound(_))));
}

// ---------------------------------------------------------------------------
// Message CRUD (T040)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn message_insert_and_find() {
    let pool = setup_pool().await;

    let record = MessageRecord {
        id: Uuid::new_v4(),
        from_agent_id: Some(Uuid::new_v4()),
        to_agent_id: Some(Uuid::new_v4()),
        message_type: "request".to_string(),
        subject: Some("task.execute".to_string()),
        content: serde_json::json!({"action": "run"}),
        priority: 1,
        status: "pending".to_string(),
        correlation_id: None,
        parent_message_id: None,
        retry_count: 0,
        max_retries: 3,
        created_at: Utc::now(),
        sent_at: None,
        delivered_at: None,
        processed_at: None,
        expires_at: None,
        error_message: None,
        message_id: None,
    };

    // Insert
    let inserted = insert_message(&pool, &record).await.unwrap();
    assert_eq!(inserted.id, record.id);
    assert_eq!(inserted.message_type, "request");
    assert_eq!(inserted.status, "pending");

    // Find
    let found = find_message(&pool, record.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.subject, Some("task.execute".to_string()));
    assert_eq!(found.content, record.content);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn message_update_status() {
    let pool = setup_pool().await;

    let record = MessageRecord {
        id: Uuid::new_v4(),
        from_agent_id: None,
        to_agent_id: None,
        message_type: "notification".to_string(),
        subject: None,
        content: serde_json::json!({}),
        priority: 0,
        status: "pending".to_string(),
        correlation_id: None,
        parent_message_id: None,
        retry_count: 0,
        max_retries: 0,
        created_at: Utc::now(),
        sent_at: None,
        delivered_at: None,
        processed_at: None,
        expires_at: None,
        error_message: None,
        message_id: None,
    };

    insert_message(&pool, &record).await.unwrap();

    // Update status
    update_message_status(&pool, record.id, "delivered").await.unwrap();

    let found = find_message(&pool, record.id).await.unwrap().unwrap();
    assert_eq!(found.status, "delivered");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn message_find_by_sender() {
    let pool = setup_pool().await;

    let sender_id = Uuid::new_v4();
    let now = Utc::now();

    let record = MessageRecord {
        id: Uuid::new_v4(),
        from_agent_id: Some(sender_id),
        to_agent_id: None,
        message_type: "event".to_string(),
        subject: None,
        content: serde_json::json!({}),
        priority: 0,
        status: "sent".to_string(),
        correlation_id: None,
        parent_message_id: None,
        retry_count: 0,
        max_retries: 0,
        created_at: now,
        sent_at: Some(now),
        delivered_at: None,
        processed_at: None,
        expires_at: None,
        error_message: None,
        message_id: None,
    };

    insert_message(&pool, &record).await.unwrap();

    let start = now - chrono::Duration::minutes(1);
    let end = now + chrono::Duration::minutes(1);
    let messages = find_messages_by_sender(&pool, sender_id, start, end).await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].from_agent_id, Some(sender_id));
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn message_find_by_correlation() {
    let pool = setup_pool().await;

    let correlation_id = Uuid::new_v4();

    let record = MessageRecord {
        id: Uuid::new_v4(),
        from_agent_id: None,
        to_agent_id: None,
        message_type: "response".to_string(),
        subject: None,
        content: serde_json::json!({"result": "ok"}),
        priority: 0,
        status: "processed".to_string(),
        correlation_id: Some(correlation_id),
        parent_message_id: None,
        retry_count: 0,
        max_retries: 0,
        created_at: Utc::now(),
        sent_at: None,
        delivered_at: None,
        processed_at: Some(Utc::now()),
        expires_at: None,
        error_message: None,
        message_id: None,
    };

    insert_message(&pool, &record).await.unwrap();

    let messages = find_messages_by_correlation(&pool, correlation_id).await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, record.id);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn message_not_found() {
    let pool = setup_pool().await;

    let found = find_message(&pool, Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn message_update_nonexistent_fails() {
    let pool = setup_pool().await;

    let result = update_message_status(&pool, Uuid::new_v4(), "delivered").await;
    assert!(matches!(result, Err(PersistenceError::NotFound(_))));
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn checkpoint_latest_returns_most_recent() {
    let pool = setup_pool().await;

    let agent_id = Uuid::new_v4();
    let record = AgentRecord {
        agent_id,
        agent_type: "worker".to_string(),
        agent_name: format!("ckpt-latest-{}", Uuid::new_v4()),
        status: "active".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };
    insert_agent(&pool, &record).await.unwrap();

    // Insert two checkpoints
    let _first = insert_checkpoint(
        &pool,
        agent_id,
        serde_json::json!({"version": 1}),
        Some(10),
    )
    .await
    .unwrap();

    let second = insert_checkpoint(
        &pool,
        agent_id,
        serde_json::json!({"version": 2}),
        Some(20),
    )
    .await
    .unwrap();

    // Latest should be the second
    let latest = get_latest_checkpoint(&pool, agent_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.checkpoint_id, second);
    assert_eq!(latest.kv_revision, Some(20));
}

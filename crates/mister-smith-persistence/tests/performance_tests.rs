//! Env-gated performance validation tests (T048).
//!
//! These tests require both a running PostgreSQL instance (`DATABASE_URL`) and
//! a running NATS server with JetStream enabled (`NATS_URL`).
//!
//! Run with:
//! ```text
//! DATABASE_URL=postgres://... NATS_URL=nats://localhost:4222 \
//!   cargo test -p mister-smith-persistence --test performance_tests -- --ignored
//! ```
//!
//! The final two tests in this file are lightweight local assertions for Phase 10
//! managed-memory reduction and consolidation behavior and do not require external services.

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{Duration as ChronoDuration, Utc};
use uuid::Uuid;

use mister_smith_core::{
    AgentId, AgentType, BudgetPolicy, BudgetScope, ContextBudget, ContextBudgetId,
    ExecutionBranchId, TaskId,
};
use mister_smith_persistence::config::{FlushConfig, KvConfig};
use mister_smith_persistence::hybrid::manager::HybridStateManager;
use mister_smith_persistence::kv::buckets::{KvBucketManager, AGENT_STATE};
use mister_smith_persistence::kv::state::{ConflictStrategy, StateManager};
use mister_smith_persistence::memory::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, ManagedMemoryManager,
    MemoryFragment, SnapshotScope,
};
use mister_smith_persistence::postgres::migrations::MigrationRunner;
use mister_smith_persistence::postgres::queries::*;

// ---------------------------------------------------------------------------
// Helpers (consistent with kv_tests, postgres_tests, hybrid_tests)
// ---------------------------------------------------------------------------

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn nats_url() -> Option<String> {
    std::env::var("NATS_URL").ok()
}

fn both_urls() -> Option<(String, String)> {
    Some((database_url()?, nats_url()?))
}

/// Create a fresh PostgreSQL pool and run migrations.
async fn setup_pool() -> sqlx::PgPool {
    let url = database_url().expect("DATABASE_URL required");
    let pool = sqlx::PgPool::connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL");

    let runner = MigrationRunner::new(pool.clone());
    runner.run().await.expect("Migrations should succeed");

    pool
}

/// Connect to NATS and initialize KV buckets.
async fn setup_kv() -> (
    async_nats::jetstream::Context,
    async_nats::jetstream::kv::Store,
) {
    let url = nats_url().expect("NATS_URL required");
    let client = async_nats::connect(&url)
        .await
        .expect("Failed to connect to NATS");
    let context = async_nats::jetstream::new(client);

    let config = KvConfig {
        enabled: true,
        session_ttl_secs: 3600,
        agent_state_ttl_secs: 1800,
        cache_ttl_secs: 300,
        replicas: 1,
    };
    let mut bucket_mgr = KvBucketManager::new(context.clone(), config);
    bucket_mgr.initialize_buckets().await.expect("Buckets");

    let store = bucket_mgr.bucket(AGENT_STATE).unwrap().clone();
    (context, store)
}

/// Full dual-store setup: PG pool + KV store.
async fn setup_both() -> (sqlx::PgPool, async_nats::jetstream::kv::Store) {
    let _ = both_urls().expect("DATABASE_URL and NATS_URL required");

    let pool = setup_pool().await;
    let (_context, store) = setup_kv().await;
    (pool, store)
}

fn test_flush_config() -> FlushConfig {
    FlushConfig {
        threshold: 1000, // High threshold — no auto-flush during perf tests
        deadline_secs: 600,
        safety_margin_secs: 60,
        max_flush_retries: 3,
    }
}

/// Create an agent in PG for FK constraints.
async fn create_test_agent(pool: &sqlx::PgPool) -> Uuid {
    let agent_id = Uuid::new_v4();
    let record = AgentRecord {
        agent_id,
        agent_type: "perf-test".to_string(),
        agent_name: format!("perf-test-{agent_id}"),
        status: "active".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_heartbeat: None,
    };
    insert_agent(pool, &record).await.unwrap();
    agent_id
}

fn branch_scope() -> (TaskId, ExecutionBranchId, SnapshotScope) {
    let workflow_id = TaskId::new();
    let branch_id = ExecutionBranchId::new();
    (workflow_id, branch_id, SnapshotScope::Branch(branch_id))
}

fn context_budget(max_units: u64, policy: BudgetPolicy) -> ContextBudget {
    ContextBudget {
        budget_id: ContextBudgetId::new(),
        scope: BudgetScope::Branch,
        max_units,
        reserved_units: 0,
        policy,
    }
}

fn memory_fragment(
    workflow_id: TaskId,
    branch_id: ExecutionBranchId,
    role: AgentType,
    fragment_class: FragmentClass,
    units: u64,
    allowed_roles: Vec<AgentType>,
    content: serde_json::Value,
) -> MemoryFragment {
    MemoryFragment::new(
        SnapshotScope::Branch(branch_id),
        content,
        units,
        fragment_class,
        FragmentProvenance::new(
            workflow_id,
            Some(branch_id),
            AgentId::new(),
            role,
            "performance.managed_memory",
        ),
        FragmentFreshness::ttl(Utc::now(), ChronoDuration::hours(1)),
        AccessPolicy::for_roles(allowed_roles).for_branch(branch_id),
    )
}

// ---------------------------------------------------------------------------
// SC-002: KV read latency benchmark
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires NATS_URL
async fn kv_read_latency_under_load() {
    let (_context, store) = setup_kv().await;
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let entry_count = 100;
    let mut keys = Vec::with_capacity(entry_count);

    // Phase 1: Write 100 entries
    for i in 0..entry_count {
        let key = format!("perf:read_latency:{}:{}", Uuid::new_v4(), i);
        let value = serde_json::json!({
            "index": i,
            "data": format!("payload-{i}"),
            "timestamp": Utc::now().to_rfc3339(),
        });
        state_mgr.save(&key, &value).await.unwrap();
        keys.push(key);
    }

    // Phase 2: Read all 100 entries and measure total time
    let start = Instant::now();
    for key in &keys {
        let result: Option<serde_json::Value> = state_mgr.get(key).await.unwrap();
        assert!(result.is_some(), "Expected value for key {key}");
    }
    let elapsed = start.elapsed();

    // Sanity check: 100 reads should complete within 30 seconds even on slow CI.
    // This is not a strict latency SLA — just a smoke test that reads work under load
    // and do not exhibit pathological behavior (e.g., exponential backoff loops).
    assert!(
        elapsed < Duration::from_secs(30),
        "100 KV reads took {elapsed:?}, expected < 30s"
    );

    eprintln!(
        "SC-002: {entry_count} KV reads completed in {elapsed:?} ({:.2}ms avg)",
        elapsed.as_millis() as f64 / entry_count as f64
    );
}

// ---------------------------------------------------------------------------
// SC-005: Concurrent state access via HybridStateManager
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires DATABASE_URL + NATS_URL
async fn concurrent_state_access_no_deadlocks() {
    let (pool, store) = setup_both().await;
    let kv_ttl = Duration::from_secs(1800);
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    let hybrid = Arc::new(HybridStateManager::new(
        state_mgr,
        pool.clone(),
        test_flush_config(),
        kv_ttl,
    ));

    let agent_id = create_test_agent(&pool).await;
    let concurrency = 100;
    let timeout_duration = Duration::from_secs(60);

    let mut handles = Vec::with_capacity(concurrency);

    for i in 0..concurrency {
        let hybrid = Arc::clone(&hybrid);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{i}");
            let value = serde_json::json!({
                "task": i,
                "data": format!("concurrent-write-{i}"),
            });

            // Write
            let _rev = hybrid
                .write_state(agent_id, &key, &value)
                .await
                .expect("write_state should succeed");

            // Read back
            let read = hybrid
                .read_state(agent_id, &key)
                .await
                .expect("read_state should succeed");
            assert!(read.is_some(), "read_state should return written value");

            i
        });
        handles.push(handle);
    }

    // Collect all results with a global timeout to detect deadlocks
    let all_results = tokio::time::timeout(timeout_duration, async {
        let mut completed = Vec::with_capacity(concurrency);
        for handle in handles {
            let result = handle.await.expect("Task should not panic");
            completed.push(result);
        }
        completed
    })
    .await
    .expect("Concurrent state access timed out — possible deadlock");

    assert_eq!(
        all_results.len(),
        concurrency,
        "All {concurrency} tasks should complete"
    );

    eprintln!("SC-005: {concurrency} concurrent write+read tasks completed without deadlocks");
}

// ---------------------------------------------------------------------------
// SC-008: Bulk insert and query of task records
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires DATABASE_URL
async fn bulk_task_insert_and_query() {
    let pool = setup_pool().await;

    let agent_id = Uuid::new_v4(); // No FK constraint on tasks.records.agent_id
    let correlation_id = Uuid::new_v4();
    let bulk_count = 1000;

    // Phase 1: Insert 1000 task records
    let start = Instant::now();
    for i in 0..bulk_count {
        let record = TaskRecord {
            task_id: Uuid::new_v4(),
            task_type: format!("bulk-type-{}", i % 10),
            agent_id: Some(agent_id),
            payload: serde_json::json!({
                "index": i,
                "batch": "perf-test",
            }),
            result: None,
            metadata: serde_json::json!({}),
            status: "pending".to_string(),
            priority: (i % 5) as i32,
            correlation_id: if i < 500 { Some(correlation_id) } else { None },
            parent_task_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            expires_at: None,
        };
        insert_task(&pool, &record)
            .await
            .unwrap_or_else(|e| panic!("Failed to insert task {i}: {e}"));
    }
    let insert_elapsed = start.elapsed();

    eprintln!(
        "SC-008: Inserted {bulk_count} task records in {insert_elapsed:?} ({:.2}ms avg)",
        insert_elapsed.as_millis() as f64 / bulk_count as f64,
    );

    // Phase 2: Query by agent_id — should return all 1000
    let query_start = Instant::now();
    let by_agent = find_tasks_by_agent(&pool, agent_id).await.unwrap();
    let agent_query_elapsed = query_start.elapsed();

    assert!(
        by_agent.len() >= bulk_count,
        "Expected at least {bulk_count} tasks for agent_id, got {}",
        by_agent.len()
    );
    assert!(
        by_agent.iter().all(|t| t.agent_id == Some(agent_id)),
        "All returned tasks should belong to the test agent"
    );

    eprintln!(
        "SC-008: Query by agent_id returned {} records in {agent_query_elapsed:?}",
        by_agent.len(),
    );

    // Phase 3: Query by correlation_id — should return the first 500
    let corr_start = Instant::now();
    let by_correlation = find_tasks_by_correlation(&pool, correlation_id)
        .await
        .unwrap();
    let corr_query_elapsed = corr_start.elapsed();

    assert!(
        by_correlation.len() >= 500,
        "Expected at least 500 tasks for correlation_id, got {}",
        by_correlation.len()
    );
    assert!(
        by_correlation
            .iter()
            .all(|t| t.correlation_id == Some(correlation_id)),
        "All returned tasks should have the test correlation_id"
    );

    eprintln!(
        "SC-008: Query by correlation_id returned {} records in {corr_query_elapsed:?}",
        by_correlation.len(),
    );

    // Sanity: both queries should complete in reasonable time
    assert!(
        agent_query_elapsed < Duration::from_secs(30),
        "Agent query took {agent_query_elapsed:?}, expected < 30s"
    );
    assert!(
        corr_query_elapsed < Duration::from_secs(30),
        "Correlation query took {corr_query_elapsed:?}, expected < 30s"
    );
}

// ---------------------------------------------------------------------------
// SC-202 / T021: Managed-memory reduction and consolidation coverage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn role_aware_context_reduction_cuts_delivered_volume_by_thirty_percent() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    manager.record_fragment(memory_fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Working,
        5,
        vec![AgentType::Planner],
        serde_json::json!({"kind": "brief-1"}),
    ));
    manager.record_fragment(memory_fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Episodic,
        5,
        vec![AgentType::Planner],
        serde_json::json!({"kind": "brief-2"}),
    ));
    manager.record_fragment(memory_fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        5,
        vec![AgentType::Planner],
        serde_json::json!({"kind": "brief-3"}),
    ));

    let snapshot = manager
        .assemble_snapshot(
            scope,
            AgentType::Planner,
            context_budget(8, BudgetPolicy::Summarize),
        )
        .await
        .expect("snapshot should assemble");

    assert_eq!(snapshot.total_candidate_units, 15);
    assert!(
        snapshot.summary.is_some(),
        "reduction should synthesize a summary when over budget"
    );
    assert!(
        snapshot.delivered_units * 10 <= snapshot.total_candidate_units * 7,
        "expected at least 30% reduction, delivered {} of {} units",
        snapshot.delivered_units,
        snapshot.total_candidate_units
    );
}

#[tokio::test]
async fn async_consolidation_completes_within_background_budget() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    for index in 0..64 {
        manager.record_fragment(memory_fragment(
            workflow_id,
            branch_id,
            AgentType::Memory,
            FragmentClass::Episodic,
            2,
            vec![AgentType::Planner, AgentType::Executor],
            serde_json::json!({"kind": "historic", "index": index}),
        ));
    }

    let start = Instant::now();
    let consolidated = tokio::time::timeout(
        Duration::from_millis(250),
        manager.consolidate(scope),
    )
    .await
    .expect("consolidation should remain backgroundable")
    .expect("consolidation should succeed");
    let elapsed = start.elapsed();

    assert_eq!(consolidated.len(), 1);
    assert_eq!(consolidated[0].fragment_class, FragmentClass::Summary);
    assert_eq!(consolidated[0].provenance.derived_from.len(), 64);
    assert!(
        elapsed < Duration::from_millis(250),
        "consolidation took {elapsed:?}, expected < 250ms"
    );
}

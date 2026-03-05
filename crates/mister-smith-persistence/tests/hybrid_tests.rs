//! Env-gated hybrid dual-store integration tests.
//!
//! These tests require both PostgreSQL and NATS running.
//! Run with: `DATABASE_URL=postgres://... NATS_URL=nats://localhost:4222 cargo test -p mister-smith-persistence --test hybrid_tests -- --ignored`

use std::time::Duration;

use mister_smith_persistence::config::{FlushConfig, KvConfig};
use mister_smith_persistence::hybrid::manager::HybridStateManager;
use mister_smith_persistence::kv::buckets::KvBucketManager;
use mister_smith_persistence::kv::state::{ConflictStrategy, StateManager};
use mister_smith_persistence::postgres::migrations::MigrationRunner;
use mister_smith_persistence::postgres::queries;
use uuid::Uuid;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn nats_url() -> Option<String> {
    std::env::var("NATS_URL").ok()
}

fn both_urls() -> Option<(String, String)> {
    Some((database_url()?, nats_url()?))
}

async fn setup() -> (sqlx::PgPool, async_nats::jetstream::kv::Store) {
    let (db_url, nats_url) = both_urls().expect("DATABASE_URL and NATS_URL required");

    // PG pool + migrations
    let pool = sqlx::PgPool::connect(&db_url).await.expect("PG connect");
    let runner = MigrationRunner::new(pool.clone());
    runner.run().await.expect("Migrations");

    // KV bucket
    let client = async_nats::connect(&nats_url).await.expect("NATS connect");
    let context = async_nats::jetstream::new(client);
    let config = KvConfig {
        enabled: true,
        session_ttl_secs: 3600,
        agent_state_ttl_secs: 1800,
        cache_ttl_secs: 300,
        replicas: 1,
    };
    let mut bucket_mgr = KvBucketManager::new(context, config);
    bucket_mgr.initialize_buckets().await.expect("Buckets");

    let store = bucket_mgr
        .bucket(mister_smith_persistence::kv::buckets::AGENT_STATE)
        .unwrap()
        .clone();

    (pool, store)
}

fn test_flush_config() -> FlushConfig {
    FlushConfig {
        threshold: 5,
        deadline_secs: 60,
        safety_margin_secs: 60,
        max_flush_retries: 3,
    }
}

/// Create an agent in PG for FK constraints.
async fn create_test_agent(pool: &sqlx::PgPool) -> Uuid {
    let agent_id = Uuid::new_v4();
    let record = queries::AgentRecord {
        agent_id,
        agent_type: "hybrid-test".to_string(),
        agent_name: format!("hybrid-test-{agent_id}"),
        status: "active".to_string(),
        capabilities: serde_json::json!({}),
        configuration: serde_json::json!({}),
        metadata: serde_json::json!({}),
        parent_agent_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_heartbeat: None,
    };
    queries::insert_agent(pool, &record).await.unwrap();
    agent_id
}

// ---------------------------------------------------------------------------
// Write-through and read tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires DATABASE_URL + NATS_URL
async fn write_state_goes_to_kv_and_marks_dirty() {
    let (pool, store) = setup().await;
    let kv_ttl = Duration::from_secs(1800);
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    let hybrid = HybridStateManager::new(state_mgr, pool.clone(), test_flush_config(), kv_ttl);

    let agent_id = create_test_agent(&pool).await;

    let rev = hybrid
        .write_state(agent_id, "test_key", &serde_json::json!({"value": 1}))
        .await
        .unwrap();
    assert!(rev > 0);
    assert!(hybrid.dirty_count().await > 0);
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL + NATS_URL
async fn read_state_hits_kv_cache() {
    let (pool, store) = setup().await;
    let kv_ttl = Duration::from_secs(1800);
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    let hybrid = HybridStateManager::new(state_mgr, pool.clone(), test_flush_config(), kv_ttl);

    let agent_id = create_test_agent(&pool).await;
    let value = serde_json::json!({"cached": true});

    hybrid
        .write_state(agent_id, "cached_key", &value)
        .await
        .unwrap();

    let read = hybrid
        .read_state(agent_id, "cached_key")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(read, value);
}

// ---------------------------------------------------------------------------
// Flush to SQL
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires DATABASE_URL + NATS_URL
async fn flush_writes_dirty_keys_to_sql() {
    let (pool, store) = setup().await;
    let kv_ttl = Duration::from_secs(1800);
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    // High threshold so auto-flush doesn't trigger
    let config = FlushConfig {
        threshold: 1000,
        deadline_secs: 600,
        safety_margin_secs: 60,
        max_flush_retries: 3,
    };
    let hybrid = HybridStateManager::new(state_mgr, pool.clone(), config, kv_ttl);

    let agent_id = create_test_agent(&pool).await;

    hybrid
        .write_state(agent_id, "flush_key", &serde_json::json!({"flushed": true}))
        .await
        .unwrap();

    assert!(hybrid.dirty_count().await > 0);

    let flushed = hybrid.flush_to_sql().await.unwrap();
    assert_eq!(flushed, 1);
    assert_eq!(hybrid.dirty_count().await, 0);

    // Verify in SQL
    let row = queries::get_state(&pool, agent_id, "flush_key")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.state_value, serde_json::json!({"flushed": true}));
}

#[tokio::test]
#[ignore] // Requires DATABASE_URL + NATS_URL
async fn auto_flush_triggers_on_threshold() {
    let (pool, store) = setup().await;
    let kv_ttl = Duration::from_secs(1800);
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    let config = FlushConfig {
        threshold: 3, // Low threshold for testing
        deadline_secs: 600,
        safety_margin_secs: 60,
        max_flush_retries: 3,
    };
    let hybrid = HybridStateManager::new(state_mgr, pool.clone(), config, kv_ttl);

    let agent_id = create_test_agent(&pool).await;

    // Write 3 keys (threshold = 3, should trigger auto-flush on 3rd write)
    for i in 0..3 {
        hybrid
            .write_state(agent_id, &format!("auto_key_{i}"), &serde_json::json!(i))
            .await
            .unwrap();
    }

    // After auto-flush, dirty count should be 0
    assert_eq!(hybrid.dirty_count().await, 0);
}

// ---------------------------------------------------------------------------
// SQL fallback on KV miss
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires DATABASE_URL + NATS_URL
async fn read_falls_back_to_sql_on_kv_miss() {
    let (pool, store) = setup().await;
    let kv_ttl = Duration::from_secs(1800);
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    let config = FlushConfig {
        threshold: 1000,
        deadline_secs: 600,
        safety_margin_secs: 60,
        max_flush_retries: 3,
    };
    let hybrid = HybridStateManager::new(state_mgr, pool.clone(), config, kv_ttl);

    let agent_id = create_test_agent(&pool).await;

    // Write directly to SQL (bypassing KV)
    queries::upsert_state(
        &pool,
        agent_id,
        "sql_only_key",
        serde_json::json!({"source": "sql"}),
        None,
    )
    .await
    .unwrap();

    // Read should fall back to SQL and return the value
    let value = hybrid
        .read_state(agent_id, "sql_only_key")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(value, serde_json::json!({"source": "sql"}));

    // Second read should hit KV (hydrated by first read)
    let value2 = hybrid
        .read_state(agent_id, "sql_only_key")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(value2, serde_json::json!({"source": "sql"}));
}

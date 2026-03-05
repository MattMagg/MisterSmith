//! Env-gated JetStream KV integration tests.
//!
//! These tests require a running NATS server with JetStream enabled and `NATS_URL` env var.
//! Run with: `NATS_URL=nats://localhost:4222 cargo test -p mister-smith-persistence --test kv_tests -- --ignored`

use std::time::Duration;

use mister_smith_persistence::config::KvConfig;
use mister_smith_persistence::kv::buckets::{KvBucketManager, AGENT_STATE, QUERY_CACHE, SESSION_DATA};
use mister_smith_persistence::kv::state::{ConflictStrategy, StateManager};
use mister_smith_core::HealthStatus;

fn nats_url() -> Option<String> {
    std::env::var("NATS_URL").ok()
}

/// Connect to NATS and create a JetStream context.
async fn setup_jetstream() -> async_nats::jetstream::Context {
    let url = nats_url().expect("NATS_URL required");
    let client = async_nats::connect(&url)
        .await
        .expect("Failed to connect to NATS");
    async_nats::jetstream::new(client)
}

/// Create a unique bucket config for test isolation (avoids collisions).
fn test_kv_config() -> KvConfig {
    KvConfig {
        enabled: true,
        session_ttl_secs: 60,
        agent_state_ttl_secs: 30,
        cache_ttl_secs: 10,
        replicas: 1,
    }
}

// ---------------------------------------------------------------------------
// Bucket management tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn bucket_initialization_is_idempotent() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context.clone(), config.clone());
    manager.initialize_buckets().await.unwrap();

    // Second call should succeed (idempotent)
    let mut manager2 = KvBucketManager::new(context, config);
    manager2.initialize_buckets().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn bucket_health_check_passes_after_init() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let status = manager.health_check().await.unwrap();
    assert_eq!(status, HealthStatus::Healthy);
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn bucket_access_by_name() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    // All three standard buckets should be accessible
    assert!(manager.bucket(SESSION_DATA).is_ok());
    assert!(manager.bucket(AGENT_STATE).is_ok());
    assert!(manager.bucket(QUERY_CACHE).is_ok());

    // Nonexistent bucket should fail
    assert!(manager.bucket("NONEXISTENT").is_err());
}

// ---------------------------------------------------------------------------
// StateManager tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_save_and_get() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let store = manager.bucket(AGENT_STATE).unwrap().clone();
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let key = format!("test:save_get:{}", uuid::Uuid::new_v4());
    let value = serde_json::json!({"hello": "world", "count": 42});

    // Save
    let revision = state_mgr.save(&key, &value).await.unwrap();
    assert!(revision > 0);

    // Get
    let retrieved: Option<serde_json::Value> = state_mgr.get(&key).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_cas_update_succeeds() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let store = manager.bucket(AGENT_STATE).unwrap().clone();
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let key = format!("test:cas_ok:{}", uuid::Uuid::new_v4());
    let initial = serde_json::json!({"version": 1});

    let rev1 = state_mgr.save(&key, &initial).await.unwrap();

    // CAS update with correct revision
    let updated = serde_json::json!({"version": 2});
    let rev2 = state_mgr.update(&key, &updated, rev1).await.unwrap();
    assert!(rev2 > rev1);

    // Verify update
    let retrieved: serde_json::Value = state_mgr.get(&key).await.unwrap().unwrap();
    assert_eq!(retrieved["version"], 2);
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_cas_update_fails_on_stale_revision() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let store = manager.bucket(AGENT_STATE).unwrap().clone();
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let key = format!("test:cas_stale:{}", uuid::Uuid::new_v4());
    let initial = serde_json::json!({"version": 1});

    let rev1 = state_mgr.save(&key, &initial).await.unwrap();

    // Update to rev2
    let _rev2 = state_mgr
        .save(&key, &serde_json::json!({"version": 2}))
        .await
        .unwrap();

    // CAS with stale rev1 should fail
    let result = state_mgr
        .update(&key, &serde_json::json!({"version": 3}), rev1)
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        mister_smith_core::PersistenceError::VersionConflict { key: k, .. } => {
            assert_eq!(k, key);
        }
        other => panic!("Expected VersionConflict, got: {other:?}"),
    }
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_delete_removes_value() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let store = manager.bucket(AGENT_STATE).unwrap().clone();
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let key = format!("test:delete:{}", uuid::Uuid::new_v4());
    state_mgr
        .save(&key, &serde_json::json!({"temp": true}))
        .await
        .unwrap();

    // Delete
    state_mgr.delete(&key).await.unwrap();

    // Get should return None
    let result: Option<serde_json::Value> = state_mgr.get(&key).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_reject_strategy_prevents_overwrite() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let store = manager.bucket(AGENT_STATE).unwrap().clone();
    let state_mgr = StateManager::new(store, ConflictStrategy::Reject);

    let key = format!("test:reject:{}", uuid::Uuid::new_v4());
    state_mgr
        .save(&key, &serde_json::json!({"first": true}))
        .await
        .unwrap();

    // Second save with Reject should fail
    let result = state_mgr
        .save(&key, &serde_json::json!({"second": true}))
        .await;
    assert!(matches!(
        result,
        Err(mister_smith_core::PersistenceError::DuplicateKey(_))
    ));
}

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_get_nonexistent_returns_none() {
    let context = setup_jetstream().await;
    let config = test_kv_config();

    let mut manager = KvBucketManager::new(context, config);
    manager.initialize_buckets().await.unwrap();

    let store = manager.bucket(AGENT_STATE).unwrap().clone();
    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let result: Option<serde_json::Value> = state_mgr
        .get(&format!("nonexistent:{}", uuid::Uuid::new_v4()))
        .await
        .unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// TTL expiration test
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Requires NATS: NATS_URL=nats://localhost:4222
async fn state_ttl_expiration() {
    let context = setup_jetstream().await;

    // Create a bucket with very short TTL for testing
    let short_ttl_config = async_nats::jetstream::kv::Config {
        bucket: format!("TTL_TEST_{}", uuid::Uuid::new_v4()),
        max_age: Duration::from_secs(2),
        num_replicas: 1,
        ..Default::default()
    };

    let store = context
        .create_key_value(short_ttl_config)
        .await
        .unwrap();

    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);

    let key = "ttl_test_key";
    state_mgr
        .save(&key, &serde_json::json!({"ephemeral": true}))
        .await
        .unwrap();

    // Value should exist immediately
    let exists: Option<serde_json::Value> = state_mgr.get(key).await.unwrap();
    assert!(exists.is_some());

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Value should be gone
    let expired: Option<serde_json::Value> = state_mgr.get(key).await.unwrap();
    assert!(expired.is_none());
}

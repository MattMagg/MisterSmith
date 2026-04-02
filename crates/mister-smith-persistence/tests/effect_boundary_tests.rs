use std::time::Duration;

use chrono::Utc;
use mister_smith_core::{EffectBoundaryIntentState, EffectBoundaryOutcomeState, TaskId};
use mister_smith_persistence::config::{FlushConfig, KvConfig};
use mister_smith_persistence::hybrid::manager::HybridStateManager;
use mister_smith_persistence::kv::buckets::KvBucketManager;
use mister_smith_persistence::kv::state::{ConflictStrategy, StateManager};
use mister_smith_persistence::postgres::migrations::MigrationRunner;
use mister_smith_persistence::{
    effect_boundary_records, merge_effect_boundary_metadata, EffectBoundaryRecord,
};
use serde_json::json;
use uuid::Uuid;

fn sample_effect_boundary(
    workflow_id: TaskId,
    effect_boundary_id: Uuid,
    idempotency_key: &str,
    outcome_state: EffectBoundaryOutcomeState,
    note: &str,
) -> EffectBoundaryRecord {
    EffectBoundaryRecord {
        workflow_id,
        effect_boundary_id,
        idempotency_key: idempotency_key.to_string(),
        intent_state: EffectBoundaryIntentState::Recorded,
        outcome_state,
        intent_recorded_at: Utc::now(),
        outcome_recorded_at: (outcome_state == EffectBoundaryOutcomeState::Completed)
            .then_some(Utc::now()),
        history_event_id: None,
        note: Some(note.to_string()),
    }
}

#[test]
fn effect_boundary_metadata_merges_by_idempotency_key() {
    let workflow_id = TaskId::new();
    let mut metadata = json!({});
    let idempotency_key = "workflow.step.completed:step-1";
    let first = sample_effect_boundary(
        workflow_id,
        Uuid::new_v4(),
        idempotency_key,
        EffectBoundaryOutcomeState::CompletionUnknown,
        "intent recorded",
    );
    let replacement = sample_effect_boundary(
        workflow_id,
        Uuid::new_v4(),
        idempotency_key,
        EffectBoundaryOutcomeState::Completed,
        "completion recorded",
    );

    merge_effect_boundary_metadata(&mut metadata, &[first, replacement.clone()])
        .expect("effect boundary metadata should merge");

    let effect_boundaries =
        effect_boundary_records(&metadata).expect("effect boundaries should deserialize");
    assert_eq!(effect_boundaries, vec![replacement]);
}

#[test]
fn effect_boundary_metadata_keeps_unknown_outcome_explicit() {
    let workflow_id = TaskId::new();
    let effect_boundary = sample_effect_boundary(
        workflow_id,
        Uuid::new_v4(),
        "workflow.completed:workflow",
        EffectBoundaryOutcomeState::CompletionUnknown,
        "completion still unknown after interrupted publish",
    );
    let mut metadata = json!({});

    merge_effect_boundary_metadata(&mut metadata, std::slice::from_ref(&effect_boundary))
        .expect("effect boundary metadata should merge");

    let effect_boundaries =
        effect_boundary_records(&metadata).expect("effect boundaries should deserialize");
    assert_eq!(effect_boundaries.len(), 1);
    assert_eq!(
        effect_boundaries[0].outcome_state,
        EffectBoundaryOutcomeState::CompletionUnknown
    );
    assert_eq!(
        effect_boundaries[0].note.as_deref(),
        Some("completion still unknown after interrupted publish")
    );
}

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn nats_url() -> Option<String> {
    std::env::var("NATS_URL").ok()
}

fn both_urls() -> Option<(String, String)> {
    Some((database_url()?, nats_url()?))
}

async fn setup_hybrid() -> (HybridStateManager, sqlx::PgPool) {
    let (db_url, nats_url) = both_urls().expect("DATABASE_URL and NATS_URL required");

    let pool = sqlx::PgPool::connect(&db_url).await.expect("PG connect");
    let runner = MigrationRunner::new(pool.clone());
    runner.run().await.expect("migrations");

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
    bucket_mgr.initialize_buckets().await.expect("buckets");
    let store = bucket_mgr
        .bucket(mister_smith_persistence::kv::buckets::AGENT_STATE)
        .expect("agent_state bucket")
        .clone();

    let state_mgr = StateManager::new(store, ConflictStrategy::LastWriteWins);
    let hybrid = HybridStateManager::new(
        state_mgr,
        pool.clone(),
        FlushConfig {
            threshold: 1000,
            deadline_secs: 600,
            safety_margin_secs: 60,
            max_flush_retries: 3,
        },
        Duration::from_secs(300),
    );

    (hybrid, pool)
}

#[tokio::test]
#[ignore]
async fn hybrid_effect_boundary_cache_round_trips_when_env_is_available() {
    let (hybrid, _pool) = setup_hybrid().await;
    let workflow_id = Uuid::new_v4();
    let effect_boundary_id = Uuid::new_v4();
    let effect_boundary = json!({
        "workflow_id": workflow_id,
        "effect_boundary_id": effect_boundary_id,
        "idempotency_key": "workflow.step.completed:step-1",
        "intent_state": "recorded",
        "outcome_state": "completed",
    });

    hybrid
        .write_effect_boundary(workflow_id, effect_boundary_id, &effect_boundary)
        .await
        .expect("effect boundary should write to cache");

    let cached = hybrid
        .read_effect_boundary(workflow_id, effect_boundary_id)
        .await
        .expect("effect boundary cache read should succeed")
        .expect("effect boundary should exist in cache");

    assert_eq!(cached, effect_boundary);
}

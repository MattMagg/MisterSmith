#![allow(dead_code)]

#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/autonomy.rs"]
mod autonomy;
#[path = "../src/execution.rs"]
mod execution;
#[path = "../src/observability.rs"]
mod observability;

use chrono::Utc;
use execution::{
    effect_boundary_idempotency_key, effect_boundary_projection,
    should_skip_effect_boundary_publish,
};
use mister_smith_core::{EffectBoundaryIntentState, EffectBoundaryOutcomeState, TaskId};
use mister_smith_persistence::{merge_effect_boundary_metadata, EffectBoundaryRecord};
use serde_json::json;
use uuid::Uuid;

#[test]
fn replay_completed_effect_boundary_skips_duplicate_publish() {
    let workflow_id = TaskId::new();
    let subject = "workflow.completed";
    let payload = json!({
        "workflow_id": workflow_id,
        "summary": "bounded result",
    });
    let idempotency_key = effect_boundary_idempotency_key(subject, &payload);
    let record = EffectBoundaryRecord {
        workflow_id,
        effect_boundary_id: Uuid::new_v4(),
        idempotency_key: idempotency_key.clone(),
        intent_state: EffectBoundaryIntentState::Recorded,
        outcome_state: EffectBoundaryOutcomeState::Completed,
        intent_recorded_at: Utc::now() - chrono::Duration::seconds(5),
        outcome_recorded_at: Some(Utc::now()),
        history_event_id: None,
        note: Some("workflow.completed: completed".to_string()),
    };
    let mut metadata = json!({});

    merge_effect_boundary_metadata(&mut metadata, std::slice::from_ref(&record))
        .expect("effect boundary metadata should merge");

    assert!(should_skip_effect_boundary_publish(
        &metadata, subject, &payload
    ));
    assert_eq!(
        effect_boundary_projection(&metadata, subject, &payload),
        Some(record),
    );
}

#[test]
fn incomplete_effect_boundary_stays_explicit_and_retryable() {
    let workflow_id = TaskId::new();
    let task_id = TaskId::new();
    let subject = "workflow.step.completed";
    let payload = json!({
        "workflow_id": workflow_id,
        "task_id": task_id,
        "step_id": "draft-outline",
    });
    let idempotency_key = effect_boundary_idempotency_key(subject, &payload);
    let record = EffectBoundaryRecord {
        workflow_id,
        effect_boundary_id: Uuid::new_v4(),
        idempotency_key,
        intent_state: EffectBoundaryIntentState::Recorded,
        outcome_state: EffectBoundaryOutcomeState::CompletionUnknown,
        intent_recorded_at: Utc::now(),
        outcome_recorded_at: None,
        history_event_id: None,
        note: Some("workflow.step.completed: waiting for durable completion evidence".to_string()),
    };
    let mut metadata = json!({});

    merge_effect_boundary_metadata(&mut metadata, std::slice::from_ref(&record))
        .expect("effect boundary metadata should merge");

    assert!(
        !should_skip_effect_boundary_publish(&metadata, subject, &payload),
        "replay must not silently treat unknown completion as success",
    );
    let projection = effect_boundary_projection(&metadata, subject, &payload)
        .expect("effect boundary projection should be available");
    assert_eq!(
        projection.outcome_state,
        EffectBoundaryOutcomeState::CompletionUnknown,
    );
    assert_eq!(projection.outcome_recorded_at, None);
    assert!(projection
        .note
        .as_deref()
        .unwrap_or_default()
        .contains("waiting for durable completion evidence"),);
}

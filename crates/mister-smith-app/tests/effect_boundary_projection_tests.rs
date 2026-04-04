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
use mister_smith_core::{
    AgentId, CoordinatorDelegationRecord, CoordinatorMergeDecision, CoordinatorRuntimeProofView,
    CoordinatorSubordinateInboxRecord, DelegatedWorkEvidenceRef, EffectBoundaryIntentState,
    EffectBoundaryOutcomeState, SessionId, SubagentStateRecord, TaskId,
};
use mister_smith_persistence::{merge_effect_boundary_metadata, EffectBoundaryRecord};
use serde_json::json;
use uuid::Uuid;

fn sample_packet_026_proof(
    workflow_id: TaskId,
    evidence_kind: &str,
    proof_boundary: &str,
) -> CoordinatorRuntimeProofView {
    let coordinator_agent_id = AgentId::new();
    let delegated_agent_id = AgentId::new();

    CoordinatorRuntimeProofView {
        workflow_id,
        coordinator_agent_id,
        delegation_records: vec![CoordinatorDelegationRecord {
            delegation_id: "delegation-1".to_string(),
            workflow_id,
            session_id: Some(SessionId::new()),
            coordinator_agent_id,
            child_role: "explorer".to_string(),
            subagent_id: delegated_agent_id,
            delegated_job_label: "audit backend boundaries".to_string(),
            delegated_scope_ref: "branch-7".to_string(),
            delegation_reason: "bounded repo inspection justified delegation".to_string(),
            allowed_follow_up_actions: vec![
                "clarify".to_string(),
                "resume".to_string(),
                "stop".to_string(),
                "inspect".to_string(),
            ],
            created_at: Utc::now(),
            status: if evidence_kind == "grounded" {
                "completed".to_string()
            } else {
                "blocked".to_string()
            },
        }],
        subordinate_inbox: vec![CoordinatorSubordinateInboxRecord {
            delegation_id: "delegation-1".to_string(),
            event_id: "event-1".to_string(),
            event_sequence: 1,
            event_kind: if evidence_kind == "grounded" {
                "completed".to_string()
            } else {
                "clarify_requested".to_string()
            },
            event_payload_ref: "task:child-1".to_string(),
            recorded_at: Utc::now(),
            visible_to: "coordinator_and_operator".to_string(),
        }],
        subagent_states: vec![SubagentStateRecord {
            delegation_id: "delegation-1".to_string(),
            subagent_id: delegated_agent_id,
            current_state: if evidence_kind == "grounded" {
                "completed".to_string()
            } else {
                "blocked".to_string()
            },
            previous_state: Some("running".to_string()),
            state_reason: if evidence_kind == "grounded" {
                "grounded repo inspection finished".to_string()
            } else {
                "delegated work is still waiting on clarification".to_string()
            },
            state_updated_at: Utc::now(),
            coordinator_action_ref: Some("decision-1".to_string()),
        }],
        delegated_work_evidence: vec![DelegatedWorkEvidenceRef {
            delegation_id: "delegation-1".to_string(),
            evidence_kind: evidence_kind.to_string(),
            evidence_summary: if evidence_kind == "grounded" {
                "bounded backend audit captured grounded evidence".to_string()
            } else {
                "delegated work only reached placeholder completion".to_string()
            },
            artifact_refs: vec!["task:child-1".to_string(), "evidence:backend-audit".to_string()],
            proof_boundary_note: if evidence_kind == "grounded" {
                "grounded delegated evidence exists for this child run".to_string()
            } else {
                "placeholder-only delegated completion is not enough for packet 026".to_string()
            },
            recorded_at: Utc::now(),
        }],
        coordinator_decisions: vec![CoordinatorMergeDecision {
            decision_id: "decision-1".to_string(),
            workflow_id,
            decision_kind: if evidence_kind == "grounded" {
                "merge".to_string()
            } else {
                "clarify".to_string()
            },
            input_refs: vec!["delegation-1".to_string(), "task:child-1".to_string()],
            decision_reason: if evidence_kind == "grounded" {
                "coordinator merged grounded delegated work".to_string()
            } else {
                "coordinator requested clarification before treating delegated work as real".to_string()
            },
            decision_outcome: if evidence_kind == "grounded" {
                "accepted".to_string()
            } else {
                "blocked".to_string()
            },
            decided_at: Utc::now(),
        }],
        proof_boundary: proof_boundary.to_string(),
        session_follow_up_note:
            "preserve session_id, coordinator_agent_id, delegated child identity, and evidence refs only; do not assume transcript replay"
                .to_string(),
    }
}

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

#[test]
fn packet_026_placeholder_only_proof_stays_explicitly_non_grounded() {
    let workflow_id = TaskId::new();
    let mut canonical_result = autonomy::build_canonical_result_envelope(
        autonomy::CanonicalResultEnvelopeInput {
            workflow_id,
            provider_kind: "openai_chatgpt",
            model_id: "gpt-5.4",
            description: "placeholder delegated completion",
            runtime_execution_mode: json!({
                "execution_boundary": "tool_bus",
                "workflow_runner": "tokio_task",
            }),
            planner_output: json!({ "steps": 1 }),
            execution_plan: json!({ "steps": [{ "id": "draft-outline" }] }),
            step_results: vec![],
            aggregated_result: json!({ "summary": "placeholder delegated completion" }),
            status: "completed",
        },
    );
    canonical_result.coordinator_runtime_proof = Some(sample_packet_026_proof(
        workflow_id,
        "placeholder-only",
        "delegated work remained placeholder-only or partial; packet 026 real coordinator-subagent runtime not yet satisfied",
    ));

    let summary = autonomy::build_task_result_view("completed", canonical_result, None, None);
    let proof = summary
        .coordinator_runtime_proof
        .expect("packet 026 proof should be present");

    assert_eq!(proof.delegated_work_evidence[0].evidence_kind, "placeholder-only");
    assert!(proof
        .proof_boundary
        .contains("not yet satisfied"));
}

#[test]
fn retained_assistant_result_keeps_packet_026_follow_up_bounded_to_ids_and_evidence_refs() {
    let workflow_id = TaskId::new();
    let mut canonical_result = autonomy::build_canonical_result_envelope(
        autonomy::CanonicalResultEnvelopeInput {
            workflow_id,
            provider_kind: "openai_chatgpt",
            model_id: "gpt-5.4",
            description: "grounded delegated completion",
            runtime_execution_mode: json!({
                "execution_boundary": "tool_bus",
                "workflow_runner": "tokio_task",
            }),
            planner_output: json!({ "steps": 1 }),
            execution_plan: json!({ "steps": [{ "id": "draft-outline" }] }),
            step_results: vec![],
            aggregated_result: json!({ "summary": "grounded delegated completion" }),
            status: "completed",
        },
    );
    canonical_result.coordinator_runtime_proof = Some(sample_packet_026_proof(
        workflow_id,
        "grounded",
        "real coordinator-subagent runtime satisfied for the bounded delegated slice",
    ));
    let task_result = json!({
        "workflow_id": workflow_id,
        "status": "completed",
        "proof_outcome": "graph_formed_and_completed",
        "result": serde_json::to_value(canonical_result).expect("canonical result should serialize"),
    });

    let assistant_result = autonomy::retained_assistant_result(&task_result, 2, "completed")
        .expect("retained assistant result should be built");

    assert_eq!(
        assistant_result["coordinator_runtime_follow_up"]["delegation_ids"],
        json!(["delegation-1"])
    );
    assert_eq!(
        assistant_result["coordinator_runtime_follow_up"]["evidence_refs"],
        json!(["task:child-1", "evidence:backend-audit"])
    );
    assert!(assistant_result.get("coordinator_runtime_proof").is_none());
}

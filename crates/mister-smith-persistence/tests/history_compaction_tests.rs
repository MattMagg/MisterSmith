use chrono::Utc;
use mister_smith_core::{
    DurableWorkflowEventKind, DurableWorkflowLifecycleState, HistoryCompactionMode, TaskId,
};
use mister_smith_persistence::{
    history_compaction_records, history_compaction_state_key, latest_history_compaction,
    merge_history_compaction_metadata, merge_workflow_history_metadata, workflow_history,
    workflow_history_state_key, HistoryCompactionRecord, WorkflowHistoryEventRecord,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn compaction_metadata_round_trips_with_replay_pointer_snapshot() {
    let workflow_id = TaskId::new();
    let compaction_id = Uuid::new_v4();
    let snapshot_event_id = Uuid::new_v4();
    let mut metadata = json!({});
    let compaction = HistoryCompactionRecord {
        workflow_id,
        compaction_id,
        mode: HistoryCompactionMode::ReplayPointer,
        source_replay_start: 1,
        source_replay_end: 8,
        replay_start_position: 9,
        replacement_event_id: Some(snapshot_event_id),
        preserved_lineage_note: "compacted replay positions 1-8 into snapshot 9".to_string(),
        recorded_at: Utc::now(),
    };
    let snapshot_event = WorkflowHistoryEventRecord {
        workflow_id,
        event_id: snapshot_event_id,
        replay_position: 9,
        event_kind: DurableWorkflowEventKind::HistoryCompacted,
        recorded_at: Utc::now(),
        actor_agent_id: None,
        source: Some("test".to_string()),
        branch_id: None,
        node_id: None,
        lifecycle_state: Some(DurableWorkflowLifecycleState::Active),
        effect_boundary_id: None,
        compaction_id: Some(compaction_id),
        parent_event_id: None,
        payload: json!({
            "graph_state": "running",
            "replay_start_position": 9,
            "preserved_lineage_note": "compacted replay positions 1-8 into snapshot 9",
            "branch_states": [],
            "node_states": [],
        }),
    };

    merge_history_compaction_metadata(&mut metadata, std::slice::from_ref(&compaction))
        .expect("compaction metadata should merge");
    merge_workflow_history_metadata(&mut metadata, std::slice::from_ref(&snapshot_event))
        .expect("history snapshot should merge");

    assert_eq!(
        latest_history_compaction(&metadata).expect("compaction should load"),
        Some(compaction.clone())
    );
    assert_eq!(
        history_compaction_records(&metadata).expect("compaction history should load"),
        vec![compaction]
    );
    let replay_history = workflow_history(&metadata).expect("workflow history should load");
    assert_eq!(replay_history.len(), 1);
    assert_eq!(
        replay_history[0].event_kind,
        DurableWorkflowEventKind::HistoryCompacted
    );
    assert_eq!(replay_history[0].compaction_id, Some(compaction_id));
}

#[test]
fn latest_compaction_prefers_newest_recorded_lineage() {
    let workflow_id = TaskId::new();
    let older = HistoryCompactionRecord {
        workflow_id,
        compaction_id: Uuid::new_v4(),
        mode: HistoryCompactionMode::ReplayPointer,
        source_replay_start: 1,
        source_replay_end: 4,
        replay_start_position: 5,
        replacement_event_id: Some(Uuid::new_v4()),
        preserved_lineage_note: "first bounded rollup".to_string(),
        recorded_at: Utc::now() - chrono::Duration::seconds(30),
    };
    let newer = HistoryCompactionRecord {
        workflow_id,
        compaction_id: Uuid::new_v4(),
        mode: HistoryCompactionMode::ReplayPointer,
        source_replay_start: 1,
        source_replay_end: 8,
        replay_start_position: 9,
        replacement_event_id: Some(Uuid::new_v4()),
        preserved_lineage_note: "newest bounded rollup".to_string(),
        recorded_at: Utc::now(),
    };
    let mut metadata = json!({});

    merge_history_compaction_metadata(&mut metadata, &[older.clone(), newer.clone()])
        .expect("compaction lineage should merge");

    assert_eq!(
        latest_history_compaction(&metadata).expect("latest compaction should load"),
        Some(newer)
    );
    assert_eq!(
        history_compaction_records(&metadata)
            .expect("compaction lineage should load")
            .len(),
        2
    );
}

#[test]
fn duplicate_compaction_id_replaces_existing_record_and_keeps_workflow_kv_namespace_stable() {
    let workflow_id = TaskId::new();
    let compaction_id = Uuid::new_v4();
    let original = HistoryCompactionRecord {
        workflow_id,
        compaction_id,
        mode: HistoryCompactionMode::ReplayPointer,
        source_replay_start: 1,
        source_replay_end: 4,
        replay_start_position: 5,
        replacement_event_id: Some(Uuid::new_v4()),
        preserved_lineage_note: "initial bounded replay pointer".to_string(),
        recorded_at: Utc::now() - chrono::Duration::seconds(10),
    };
    let replacement = HistoryCompactionRecord {
        workflow_id,
        compaction_id,
        mode: HistoryCompactionMode::ReplayPointer,
        source_replay_start: 1,
        source_replay_end: 8,
        replay_start_position: 9,
        replacement_event_id: Some(Uuid::new_v4()),
        preserved_lineage_note: "replacement bounded replay pointer".to_string(),
        recorded_at: Utc::now(),
    };
    let mut metadata = json!({});

    merge_history_compaction_metadata(&mut metadata, std::slice::from_ref(&original))
        .expect("initial compaction should merge");
    merge_history_compaction_metadata(&mut metadata, std::slice::from_ref(&replacement))
        .expect("replacement compaction should merge");

    assert_eq!(
        history_compaction_records(&metadata).expect("compaction lineage should load"),
        vec![replacement.clone()]
    );
    assert_eq!(
        latest_history_compaction(&metadata).expect("latest compaction should load"),
        Some(replacement)
    );
    assert!(workflow_history_state_key(workflow_id).starts_with("workflow-history:"));
    assert!(history_compaction_state_key(workflow_id).starts_with("workflow-compaction:"));
}

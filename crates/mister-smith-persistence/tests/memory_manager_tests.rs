use chrono::{Duration, Utc};
use mister_smith_core::{
    AgentId, AgentType, BudgetPolicy, BudgetScope, ContextBudget, ContextBudgetId,
    ExecutionBranchId, MemoryError, TaskId,
};
use mister_smith_persistence::memory::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, ManagedMemoryManager,
    ResumeSource, SnapshotScope,
};
use serde_json::json;

fn branch_scope() -> (TaskId, ExecutionBranchId, SnapshotScope) {
    let workflow_id = TaskId::new();
    let branch_id = ExecutionBranchId::new();
    (workflow_id, branch_id, SnapshotScope::Branch(branch_id))
}

fn budget(max_units: u64, policy: BudgetPolicy) -> ContextBudget {
    ContextBudget {
        budget_id: ContextBudgetId::new(),
        scope: BudgetScope::Branch,
        max_units,
        reserved_units: 0,
        policy,
    }
}

fn fragment(
    workflow_id: TaskId,
    branch_id: ExecutionBranchId,
    role: AgentType,
    fragment_class: FragmentClass,
    units: u64,
    allowed_roles: Vec<AgentType>,
    content: serde_json::Value,
) -> mister_smith_persistence::memory::MemoryFragment {
    mister_smith_persistence::memory::MemoryFragment::new(
        SnapshotScope::Branch(branch_id),
        content,
        units,
        fragment_class,
        FragmentProvenance::new(
            workflow_id,
            Some(branch_id),
            AgentId::new(),
            role,
            "test.fragment",
        ),
        FragmentFreshness::ttl(Utc::now(), Duration::hours(1)),
        AccessPolicy::for_roles(allowed_roles),
    )
}

#[tokio::test]
async fn assemble_snapshot_filters_by_role_and_respects_budget() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Working,
        3,
        vec![AgentType::Planner],
        json!({"kind": "planner-brief"}),
    ));
    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Executor,
        FragmentClass::Episodic,
        5,
        vec![AgentType::Executor],
        json!({"kind": "executor-trace"}),
    ));
    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        6,
        vec![AgentType::Planner],
        json!({"kind": "older-shared-context"}),
    ));

    let snapshot = manager
        .assemble_snapshot(
            scope,
            AgentType::Planner,
            budget(5, BudgetPolicy::Summarize),
        )
        .await
        .expect("planner snapshot should assemble under budget");

    assert_eq!(snapshot.target_scope, SnapshotScope::Branch(branch_id));
    assert_eq!(snapshot.role, AgentType::Planner);
    assert_eq!(snapshot.total_candidate_units, 9);
    assert!(snapshot.delivered_units <= 5);
    assert_eq!(snapshot.fragment_ids.len(), 1);
    assert!(snapshot.summary.is_some());

    let resumed = manager
        .materialize_snapshot(snapshot.snapshot_id)
        .expect("snapshot should materialize");
    assert_eq!(resumed.resume_source, ResumeSource::FragmentSelection);
    assert_eq!(resumed.fragments.len(), 1);
    assert_eq!(resumed.fragments[0].content["kind"], "planner-brief");
}

#[tokio::test]
async fn summarize_budget_includes_all_overflow_fragments_in_summary_provenance() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    let first = fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Working,
        3,
        vec![AgentType::Planner],
        json!({"kind": "planner-brief"}),
    );
    let second = fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Episodic,
        3,
        vec![AgentType::Planner],
        json!({"kind": "overflow-1"}),
    );
    let third = fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        3,
        vec![AgentType::Planner],
        json!({"kind": "overflow-2"}),
    );

    let second_id = second.fragment_id;
    let third_id = third.fragment_id;
    manager.record_fragment(first);
    manager.record_fragment(second);
    manager.record_fragment(third);

    let snapshot = manager
        .assemble_snapshot(
            scope,
            AgentType::Planner,
            budget(4, BudgetPolicy::Summarize),
        )
        .await
        .expect("planner snapshot should summarize overflow");

    assert_eq!(snapshot.fragment_ids.len(), 1);
    let mut derived_from = snapshot
        .summary
        .as_ref()
        .expect("summary should exist")
        .derived_from
        .clone();
    derived_from.sort_by_key(|fragment_id| fragment_id.to_string());

    let mut expected = vec![second_id, third_id];
    expected.sort_by_key(|fragment_id| fragment_id.to_string());

    assert_eq!(derived_from, expected);
}

#[tokio::test]
async fn consolidate_preserves_provenance_and_emits_summary_fragment() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    let first = fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Episodic,
        4,
        vec![AgentType::Planner, AgentType::Critic],
        json!({"kind": "first"}),
    );
    let second = fragment(
        workflow_id,
        branch_id,
        AgentType::Critic,
        FragmentClass::Episodic,
        4,
        vec![AgentType::Planner, AgentType::Critic],
        json!({"kind": "second"}),
    );

    let first_id = first.fragment_id;
    let second_id = second.fragment_id;
    manager.record_fragment(first);
    manager.record_fragment(second);

    let consolidated = manager
        .consolidate(scope)
        .await
        .expect("consolidation should succeed");

    assert_eq!(consolidated.len(), 1);
    let summary = &consolidated[0];
    assert_eq!(summary.fragment_class, FragmentClass::Summary);
    assert_eq!(summary.provenance.source_role, AgentType::Memory);
    assert_eq!(summary.provenance.derived_from, vec![first_id, second_id]);
    assert_eq!(
        summary.access_policy.allowed_roles,
        vec![AgentType::Planner, AgentType::Critic]
    );
}

#[tokio::test]
async fn consolidate_excludes_expired_fragments() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Episodic,
        4,
        vec![AgentType::Planner],
        json!({"kind": "fresh"}),
    ));

    let expired = mister_smith_persistence::memory::MemoryFragment::new(
        SnapshotScope::Branch(branch_id),
        json!({"kind": "expired"}),
        4,
        FragmentClass::Episodic,
        FragmentProvenance::new(
            workflow_id,
            Some(branch_id),
            AgentId::new(),
            AgentType::Planner,
            "test.fragment",
        ),
        FragmentFreshness::ttl(Utc::now() - Duration::hours(2), Duration::minutes(5)),
        AccessPolicy::for_roles(vec![AgentType::Planner]),
    );
    manager.record_fragment(expired);

    let consolidated = manager
        .consolidate(scope)
        .await
        .expect("consolidation should succeed");

    assert_eq!(consolidated.len(), 1);
    assert_eq!(consolidated[0].provenance.derived_from.len(), 1);
    assert_eq!(consolidated[0].content["fragment_count"], 1);
}

#[tokio::test]
async fn checkpoint_creates_resume_ready_snapshot_without_raw_history_replay() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Executor,
        FragmentClass::Working,
        3,
        vec![AgentType::Executor],
        json!({"step": "execute"}),
    ));
    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        2,
        vec![AgentType::Executor],
        json!({"state": "cached"}),
    ));

    let snapshot_id = manager
        .checkpoint(
            branch_id,
            AgentType::Executor,
            budget(8, BudgetPolicy::Consolidate),
        )
        .await
        .expect("checkpoint snapshot should be created");

    let snapshot = manager
        .snapshot(snapshot_id)
        .expect("checkpoint snapshot should be stored");
    assert_eq!(snapshot.target_scope, scope);
    assert!(snapshot.checkpoint_fragment_id.is_some());

    let resumed = manager
        .materialize_snapshot(snapshot_id)
        .expect("checkpoint snapshot should materialize");
    assert_eq!(resumed.resume_source, ResumeSource::Checkpoint);
    assert_eq!(resumed.snapshot.snapshot_id, snapshot_id);
    assert_eq!(resumed.fragments.len(), 2);
    assert_eq!(resumed.fragments[0].fragment_class, FragmentClass::Working);
    assert_eq!(resumed.fragments[0].content["step"], "execute");
    assert_eq!(resumed.fragments[1].fragment_class, FragmentClass::Episodic);
    assert_eq!(resumed.fragments[1].content["state"], "cached");
}

#[tokio::test]
async fn reject_policy_returns_budget_error() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ManagedMemoryManager::default();

    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Planner,
        FragmentClass::Working,
        10,
        vec![AgentType::Planner],
        json!({"kind": "oversized"}),
    ));

    let err = manager
        .assemble_snapshot(scope, AgentType::Planner, budget(4, BudgetPolicy::Reject))
        .await
        .expect_err("reject policy should refuse oversized context");

    assert!(matches!(
        err,
        MemoryError::BudgetExceeded {
            requested: 10,
            max: 4,
            ..
        }
    ));
}

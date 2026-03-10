use chrono::{Duration, Utc};
use mister_smith_agents::roles::critic::{CriticAgent, CriticState};
use mister_smith_agents::roles::executor::{ExecutorAgent, ExecutorState};
use mister_smith_agents::roles::memory::{MemoryAgent, MemoryMessage, MemoryState};
use mister_smith_agents::roles::planner::{PlannerAgent, PlannerState};
use mister_smith_agents::ContextManager;
use mister_smith_core::{
    Actor, AgentId, AgentType, BudgetPolicy, BudgetScope, ContextBudget, ContextBudgetId,
    ExecutionBranchId, ExecutionNodeId, TaskId,
};
use mister_smith_persistence::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, MemoryFragment,
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
            "context-manager.test",
        ),
        FragmentFreshness::ttl(Utc::now(), Duration::hours(1)),
        AccessPolicy::for_roles(allowed_roles).for_branch(branch_id),
    )
}

#[tokio::test]
async fn context_manager_assembles_role_specific_payloads() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ContextManager::default();

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
        FragmentClass::Working,
        4,
        vec![AgentType::Executor],
        json!({"kind": "executor-trace"}),
    ));
    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        2,
        vec![AgentType::Planner, AgentType::Critic],
        json!({"kind": "shared-brief"}),
    ));

    let planner = manager
        .assemble_role_context(scope.clone(), AgentType::Planner, budget(6, BudgetPolicy::Summarize))
        .await
        .expect("planner context should assemble");
    let executor = manager
        .assemble_role_context(scope, AgentType::Executor, budget(6, BudgetPolicy::Summarize))
        .await
        .expect("executor context should assemble");

    assert_eq!(planner.snapshot.role, AgentType::Planner);
    assert_eq!(planner.fragments.len(), 2);
    assert_eq!(planner.payload["fragments"].as_array().unwrap().len(), 2);
    assert!(
        planner.payload["fragments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["content"]["kind"] != "executor-trace")
    );

    assert_eq!(executor.snapshot.role, AgentType::Executor);
    assert_eq!(executor.fragments.len(), 1);
    assert_eq!(executor.payload["fragments"][0]["content"]["kind"], "executor-trace");
}

#[tokio::test]
async fn checkpoint_resume_materializes_snapshot_without_raw_history_replay() {
    let (workflow_id, branch_id, _) = branch_scope();
    let mut manager = ContextManager::default();
    let completed = ExecutionNodeId::new();
    let pending = ExecutionNodeId::new();

    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Executor,
        FragmentClass::Working,
        3,
        vec![AgentType::Executor],
        json!({"kind": "executor-step"}),
    ));
    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        2,
        vec![AgentType::Executor],
        json!({"kind": "cached-state"}),
    ));

    let checkpoint = manager
        .checkpoint_branch(
            branch_id,
            AgentType::Executor,
            budget(8, BudgetPolicy::Consolidate),
            vec![completed],
            vec![pending],
        )
        .await
        .expect("branch checkpoint should be created");

    assert_eq!(checkpoint.branch_id, branch_id);
    assert_eq!(checkpoint.completed_nodes, vec![completed]);
    assert_eq!(checkpoint.pending_nodes, vec![pending]);

    let resumed = manager
        .resume_from_checkpoint(&checkpoint)
        .expect("checkpoint should resume");
    assert_eq!(resumed.snapshot.snapshot_id, checkpoint.memory_snapshot_id);
    assert_eq!(resumed.resume_source, ResumeSource::Checkpoint);
    assert_eq!(resumed.fragments.len(), 1);
    assert_eq!(resumed.fragments[0].fragment_class, FragmentClass::Checkpoint);
}

#[tokio::test]
async fn role_helpers_attach_managed_context() {
    let (workflow_id, branch_id, scope) = branch_scope();
    let mut manager = ContextManager::default();

    manager.record_fragment(fragment(
        workflow_id,
        branch_id,
        AgentType::Memory,
        FragmentClass::Episodic,
        2,
        vec![
            AgentType::Planner,
            AgentType::Executor,
            AgentType::Critic,
            AgentType::Memory,
        ],
        json!({"kind": "shared-context"}),
    ));

    let mut planner = PlannerAgent::new(AgentId::new());
    let mut planner_state = PlannerState::default();
    let planner_response = planner
        .plan_goal_with_managed_context(
            "ship release".to_string(),
            json!({"env": "staging"}),
            &mut manager,
            scope.clone(),
            budget(6, BudgetPolicy::Summarize),
            &mut planner_state,
        )
        .await
        .expect("planner should attach managed context");
    assert!(planner_response["context"]["managed_context"].is_object());

    let mut executor = ExecutorAgent::new(AgentId::new());
    let mut executor_state = ExecutorState::default();
    let executor_response = executor
        .execute_plan_with_managed_context(
            json!({"steps": [{"id": "step-1"}]}),
            &mut manager,
            scope.clone(),
            budget(6, BudgetPolicy::Summarize),
            &mut executor_state,
        )
        .await
        .expect("executor should attach managed context");
    assert!(executor_response["plan"]["managed_context"].is_object());

    let mut critic = CriticAgent::new(AgentId::new());
    let mut critic_state = CriticState::default();
    let critic_response = critic
        .evaluate_with_managed_context(
            json!({"result": "ok"}),
            json!({"rubric": "correctness"}),
            &mut manager,
            scope.clone(),
            budget(6, BudgetPolicy::Summarize),
            &mut critic_state,
        )
        .await
        .expect("critic should attach managed context");
    assert!(critic_response["criteria_applied"]["managed_context"].is_object());

    let mut memory = MemoryAgent::new(AgentId::new());
    let mut memory_state = MemoryState::default();
    memory
        .handle_message(
            MemoryMessage::Store {
                key: "branch.state".to_string(),
                value: json!({"status": "warm"}),
            },
            &mut memory_state,
        )
        .await
        .expect("memory store should succeed");

    let memory_response = memory
        .retrieve_with_managed_context(
            "branch.state".to_string(),
            &mut manager,
            scope,
            budget(6, BudgetPolicy::Summarize),
            &mut memory_state,
        )
        .await
        .expect("memory retrieve should attach managed context");
    assert!(memory_response["managed_context"].is_object());
}

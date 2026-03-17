use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mister_smith_core::{
    AgentId, ExecutionGraphId, HealthState, TaskId, TaskShapeKind, TeamSizingDecision, TopologyKind,
};
use uuid::Uuid;

use crate::config::TeamPattern;

/// An ephemeral group of agents assembled by a Coordinator.
#[derive(Debug, Clone)]
pub struct Team {
    pub team_id: Uuid,
    pub coordinator_id: AgentId,
    pub supervisor_id: Option<AgentId>,
    pub pattern: TeamPattern,
    pub task_id: TaskId,
    pub members: Vec<AgentId>,
    pub created_at: DateTime<Utc>,
    pub disbanded_at: Option<DateTime<Utc>>,
}

impl Team {
    /// Create a new team.
    pub fn new(
        coordinator_id: AgentId,
        pattern: TeamPattern,
        task_id: TaskId,
        members: Vec<AgentId>,
    ) -> Self {
        Self {
            team_id: Uuid::new_v4(),
            coordinator_id,
            supervisor_id: None,
            pattern,
            task_id,
            members,
            created_at: Utc::now(),
            disbanded_at: None,
        }
    }

    /// Set the team's supervisor.
    pub fn with_supervisor(mut self, supervisor_id: AgentId) -> Self {
        self.supervisor_id = Some(supervisor_id);
        self
    }

    /// Add a member to the team.
    pub fn add_member(&mut self, agent_id: AgentId) {
        if !self.members.contains(&agent_id) {
            self.members.push(agent_id);
        }
    }

    /// Remove a member from the team.
    pub fn remove_member(&mut self, agent_id: &AgentId) {
        self.members.retain(|id| id != agent_id);
    }

    /// Mark the team as disbanded.
    pub fn disband(&mut self) {
        self.disbanded_at = Some(Utc::now());
    }

    /// Check if the team is active (not disbanded).
    pub fn is_active(&self) -> bool {
        self.disbanded_at.is_none()
    }

    /// Get team member count.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Runtime-facing worker selection for one adaptive routing pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveTeamPlan {
    pub team_id: Uuid,
    pub workflow_id: TaskId,
    pub coordinator_id: AgentId,
    pub supervisor_id: Option<AgentId>,
    pub worker_ids: Vec<AgentId>,
    pub sizing_decision: TeamSizingDecision,
    pub topology_kind: TopologyKind,
}

/// Inputs required to size and assemble the active worker set for one frontier.
#[derive(Debug, Clone)]
pub struct AdaptiveTeamSizingInputs<'a> {
    pub workflow_id: TaskId,
    pub graph_id: ExecutionGraphId,
    pub coordinator_id: AgentId,
    pub topology_kind: TopologyKind,
    pub task_shape_kind: TaskShapeKind,
    pub decision_phase: &'a str,
    pub structural_parallelism: usize,
    pub branch_frontier_width: usize,
    pub dependency_depth: usize,
    pub available_worker_ids: &'a [AgentId],
    pub worker_loads: &'a HashMap<AgentId, usize>,
    pub health_state: HealthState,
    pub budget_pressure: Option<u8>,
    pub conservative_reasons: &'a [String],
    pub existing_team_id: Option<Uuid>,
}

/// Build the active worker plan for one routing frontier.
pub fn plan_adaptive_team(inputs: AdaptiveTeamSizingInputs<'_>) -> AdaptiveTeamPlan {
    let branch_frontier_width = inputs.branch_frontier_width.max(1);
    let desired_workers = inputs
        .structural_parallelism
        .max(1)
        .min(branch_frontier_width);
    let available_workers = inputs.available_worker_ids.len().max(1);
    let conservative_mode = !inputs.conservative_reasons.is_empty()
        || matches!(
            inputs.health_state,
            HealthState::Degraded | HealthState::Unhealthy
        )
        || inputs.budget_pressure.unwrap_or(0) >= 75;

    let mut selected_workers = desired_workers.min(available_workers);
    let mut cap_notes = Vec::new();

    if selected_workers < desired_workers {
        cap_notes.push("available worker pool smaller than structural width".to_string());
    }

    let depth_cap = coordination_depth_cap(inputs.dependency_depth, desired_workers);
    if selected_workers > depth_cap {
        selected_workers = depth_cap;
        cap_notes.push(format!(
            "dependency depth {} narrows coordination width to {}",
            inputs.dependency_depth, depth_cap
        ));
    }

    if let Some(pressure) = inputs.budget_pressure {
        if pressure >= 90 && selected_workers > 1 {
            selected_workers = 1;
            cap_notes.push(format!(
                "budget pressure {pressure} requires a single-worker cap"
            ));
        } else if pressure >= 75 && selected_workers > 2 {
            selected_workers = 2;
            cap_notes.push(format!(
                "budget pressure {pressure} caps the active team at 2 workers"
            ));
        }
    }

    match inputs.health_state {
        HealthState::Unhealthy if selected_workers > 1 => {
            selected_workers = 1;
            cap_notes.push(format!(
                "{:?} frontier health requires single-worker routing",
                inputs.health_state
            ));
        }
        HealthState::Degraded
            if inputs.budget_pressure.unwrap_or(0) >= 70 && selected_workers > 1 =>
        {
            selected_workers = 1;
            cap_notes.push(
                "degraded frontier health under elevated pressure requires single-worker routing"
                    .to_string(),
            );
        }
        _ => {}
    }

    if conservative_mode && selected_workers > 1 {
        selected_workers = 1;
        cap_notes.push("conservative posture requested single-worker execution".to_string());
    }

    let worker_ids = select_active_workers(
        inputs.available_worker_ids,
        inputs.worker_loads,
        selected_workers,
    );
    let cap_reason = (selected_workers < desired_workers).then(|| {
        cap_notes
            .last()
            .cloned()
            .unwrap_or_else(|| "available worker pool smaller than structural width".to_string())
    });

    let mut rationale_lines = vec![format!(
        "task shape {} with frontier width {}",
        inputs.task_shape_kind.as_str(),
        branch_frontier_width
    )];
    if inputs.dependency_depth >= 3 && depth_cap < desired_workers {
        rationale_lines.push(format!(
            "dependency depth {} narrows coordination width to {}",
            inputs.dependency_depth, depth_cap
        ));
    } else {
        rationale_lines.push(format!(
            "dependency depth {} keeps coordination cost acceptable",
            inputs.dependency_depth
        ));
    }
    if let Some(pressure) = inputs.budget_pressure {
        rationale_lines.push(format!("frontier budget pressure {pressure}"));
    }
    rationale_lines.push(if selected_workers < desired_workers {
        format!(
            "selected {} workers because only {} workers are currently available",
            selected_workers, available_workers
        )
    } else {
        format!(
            "selected {} workers from the available pool of {}",
            selected_workers, available_workers
        )
    });
    if matches!(
        inputs.health_state,
        HealthState::Degraded | HealthState::Unhealthy | HealthState::Unknown
    ) {
        rationale_lines.push(format!(
            "frontier health {:?} influenced active width",
            inputs.health_state
        ));
    }
    rationale_lines.extend(
        inputs
            .conservative_reasons
            .iter()
            .map(|reason| format!("conservative posture: {reason}")),
    );

    AdaptiveTeamPlan {
        team_id: inputs.existing_team_id.unwrap_or_else(Uuid::new_v4),
        workflow_id: inputs.workflow_id,
        coordinator_id: inputs.coordinator_id,
        supervisor_id: None,
        worker_ids,
        sizing_decision: TeamSizingDecision {
            workflow_id: inputs.workflow_id,
            graph_id: inputs.graph_id,
            decision_phase: inputs.decision_phase.to_string(),
            desired_workers,
            selected_workers,
            available_workers,
            branch_frontier_width,
            dependency_depth: inputs.dependency_depth,
            conservative_mode,
            budget_pressure: inputs.budget_pressure,
            cap_reason,
            rationale_lines,
            decided_at: Utc::now(),
        },
        topology_kind: inputs.topology_kind,
    }
}

fn coordination_depth_cap(dependency_depth: usize, desired_workers: usize) -> usize {
    if dependency_depth >= 3 {
        desired_workers.clamp(1, 2)
    } else {
        desired_workers.max(1)
    }
}

fn select_active_workers(
    available_worker_ids: &[AgentId],
    worker_loads: &HashMap<AgentId, usize>,
    selected_workers: usize,
) -> Vec<AgentId> {
    let mut ranked_workers = available_worker_ids.to_vec();
    ranked_workers.sort_by(|left, right| {
        worker_loads
            .get(left)
            .copied()
            .unwrap_or(0)
            .cmp(&worker_loads.get(right).copied().unwrap_or(0))
            .then_with(|| left.to_string().cmp(&right.to_string()))
    });
    ranked_workers.truncate(selected_workers.max(1));
    ranked_workers
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_core::{ExecutionGraphId, TaskShapeKind, TopologyKind};

    #[test]
    fn test_team_creation() {
        let coord = AgentId::new();
        let task = TaskId::new();
        let members = vec![AgentId::new(), AgentId::new()];

        let team = Team::new(coord, TeamPattern::SupervisorWorker, task, members.clone());
        assert!(team.is_active());
        assert_eq!(team.member_count(), 2);
        assert_eq!(team.coordinator_id, coord);
    }

    #[test]
    fn test_team_disband() {
        let mut team = Team::new(AgentId::new(), TeamPattern::Pipeline, TaskId::new(), vec![]);

        assert!(team.is_active());
        team.disband();
        assert!(!team.is_active());
        assert!(team.disbanded_at.is_some());
    }

    #[test]
    fn test_team_members() {
        let mut team = Team::new(
            AgentId::new(),
            TeamPattern::Consensus,
            TaskId::new(),
            vec![],
        );

        let member = AgentId::new();
        team.add_member(member);
        assert_eq!(team.member_count(), 1);

        // Duplicate add is a no-op
        team.add_member(member);
        assert_eq!(team.member_count(), 1);

        team.remove_member(&member);
        assert_eq!(team.member_count(), 0);
    }

    #[test]
    fn adaptive_team_plan_selects_wider_team_for_parallel_frontier() {
        let workers = vec![AgentId::new(), AgentId::new(), AgentId::new()];
        let worker_loads = HashMap::from([(workers[0], 2), (workers[1], 0), (workers[2], 1)]);
        let plan = plan_adaptive_team(AdaptiveTeamSizingInputs {
            workflow_id: TaskId::new(),
            graph_id: ExecutionGraphId::new(),
            coordinator_id: AgentId::new(),
            topology_kind: TopologyKind::Parallel,
            task_shape_kind: TaskShapeKind::ParallelFanout,
            decision_phase: "initial",
            structural_parallelism: 3,
            branch_frontier_width: 3,
            dependency_depth: 1,
            available_worker_ids: &workers,
            worker_loads: &worker_loads,
            health_state: HealthState::Healthy,
            budget_pressure: Some(20),
            conservative_reasons: &[],
            existing_team_id: None,
        });

        assert_eq!(plan.sizing_decision.desired_workers, 3);
        assert_eq!(plan.sizing_decision.selected_workers, 3);
        assert_eq!(plan.worker_ids.len(), 3);
        assert_eq!(plan.worker_ids[0], workers[1]);
    }

    #[test]
    fn adaptive_team_plan_caps_team_when_conservative_signals_are_present() {
        let workers = vec![AgentId::new(), AgentId::new(), AgentId::new()];
        let conservative_reasons = vec!["control-plane state unavailable".to_string()];
        let worker_loads = HashMap::new();
        let plan = plan_adaptive_team(AdaptiveTeamSizingInputs {
            workflow_id: TaskId::new(),
            graph_id: ExecutionGraphId::new(),
            coordinator_id: AgentId::new(),
            topology_kind: TopologyKind::Parallel,
            task_shape_kind: TaskShapeKind::ParallelFanout,
            decision_phase: "frontier_rebalance",
            structural_parallelism: 3,
            branch_frontier_width: 3,
            dependency_depth: 2,
            available_worker_ids: &workers,
            worker_loads: &worker_loads,
            health_state: HealthState::Healthy,
            budget_pressure: Some(25),
            conservative_reasons: &conservative_reasons,
            existing_team_id: None,
        });

        assert_eq!(plan.sizing_decision.desired_workers, 3);
        assert_eq!(plan.sizing_decision.selected_workers, 1);
        assert!(plan.sizing_decision.cap_reason.is_some());
        assert_eq!(plan.worker_ids.len(), 1);
    }
}

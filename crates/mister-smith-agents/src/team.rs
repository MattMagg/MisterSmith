use chrono::{DateTime, Utc};
use mister_smith_core::{AgentId, TaskId};
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut team = Team::new(
            AgentId::new(),
            TeamPattern::Pipeline,
            TaskId::new(),
            vec![],
        );

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
}

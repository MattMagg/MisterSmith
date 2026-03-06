use std::sync::Arc;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use mister_smith_core::{AgentId, AgentState, AgentType};

use crate::config::HealthLevel;

/// Registry entry for an active agent.
#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub health: HealthLevel,
    pub capabilities: Vec<String>,
    pub command_subject: String,
    pub heartbeat_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub restart_count: u32,
    pub metadata: serde_json::Value,
    pub supervisor_id: Option<AgentId>,
}

/// In-memory agent registry with concurrent access via DashMap.
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentEntry>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
        }
    }

    /// Register a new agent entry.
    pub fn register(&self, entry: AgentEntry) {
        self.agents.insert(entry.agent_id, entry);
    }

    /// Remove an agent from the registry.
    pub fn deregister(&self, agent_id: &AgentId) -> Option<AgentEntry> {
        self.agents.remove(agent_id).map(|(_, e)| e)
    }

    /// Find an agent by ID.
    pub fn find_by_id(&self, agent_id: &AgentId) -> Option<AgentEntry> {
        self.agents.get(agent_id).map(|e| e.value().clone())
    }

    /// Find all agents of a given type.
    pub fn find_by_type(&self, agent_type: AgentType) -> Vec<AgentEntry> {
        self.agents
            .iter()
            .filter(|e| e.value().agent_type == agent_type)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Find agents with a specific capability.
    pub fn find_by_capability(&self, capability: &str) -> Vec<AgentEntry> {
        self.agents
            .iter()
            .filter(|e| e.value().capabilities.iter().any(|c| c == capability))
            .map(|e| e.value().clone())
            .collect()
    }

    /// Find healthy, available agents matching type and capabilities.
    pub fn find_available(
        &self,
        agent_type: AgentType,
        capabilities: &[String],
    ) -> Vec<AgentEntry> {
        self.agents
            .iter()
            .filter(|e| {
                let entry = e.value();
                entry.agent_type == agent_type
                    && entry.state == AgentState::Running
                    && entry.health == HealthLevel::Healthy
                    && capabilities
                        .iter()
                        .all(|cap| entry.capabilities.contains(cap))
            })
            .map(|e| e.value().clone())
            .collect()
    }

    /// Update an agent's health level.
    pub fn update_health(&self, agent_id: &AgentId, health: HealthLevel) {
        if let Some(mut entry) = self.agents.get_mut(agent_id) {
            entry.health = health;
        }
    }

    /// Update an agent's heartbeat timestamp.
    pub fn update_heartbeat(&self, agent_id: &AgentId, timestamp: DateTime<Utc>) {
        if let Some(mut entry) = self.agents.get_mut(agent_id) {
            entry.heartbeat_at = timestamp;
        }
    }

    /// Update an agent's state.
    pub fn update_state(&self, agent_id: &AgentId, state: AgentState) {
        if let Some(mut entry) = self.agents.get_mut(agent_id) {
            entry.state = state;
        }
    }

    /// Get the count of registered agents.
    pub fn count(&self) -> usize {
        self.agents.len()
    }

    /// Get all registered agent entries.
    pub fn all(&self) -> Vec<AgentEntry> {
        self.agents.iter().map(|e| e.value().clone()).collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_entry(agent_type: AgentType, caps: Vec<&str>) -> AgentEntry {
        AgentEntry {
            agent_id: AgentId::new(),
            agent_type,
            state: AgentState::Running,
            health: HealthLevel::Healthy,
            capabilities: caps.into_iter().map(String::from).collect(),
            command_subject: format!("agents.{}.commands", AgentId::new()),
            heartbeat_at: Utc::now(),
            started_at: Some(Utc::now()),
            restart_count: 0,
            metadata: serde_json::Value::Null,
            supervisor_id: None,
        }
    }

    #[test]
    fn test_register_and_find() {
        let registry = AgentRegistry::new();
        let entry = make_entry(AgentType::Worker, vec!["analysis"]);
        let id = entry.agent_id;
        registry.register(entry);

        assert_eq!(registry.count(), 1);
        let found = registry.find_by_id(&id).unwrap();
        assert_eq!(found.agent_type, AgentType::Worker);
    }

    #[test]
    fn test_find_by_type() {
        let registry = AgentRegistry::new();
        registry.register(make_entry(AgentType::Worker, vec![]));
        registry.register(make_entry(AgentType::Worker, vec![]));
        registry.register(make_entry(AgentType::Coordinator, vec![]));

        assert_eq!(registry.find_by_type(AgentType::Worker).len(), 2);
        assert_eq!(registry.find_by_type(AgentType::Coordinator).len(), 1);
        assert_eq!(registry.find_by_type(AgentType::Monitor).len(), 0);
    }

    #[test]
    fn test_find_available_filters() {
        let registry = AgentRegistry::new();

        // Healthy worker with capability
        registry.register(make_entry(AgentType::Worker, vec!["analysis"]));

        // Unhealthy worker
        let mut unhealthy = make_entry(AgentType::Worker, vec!["analysis"]);
        unhealthy.health = HealthLevel::Unhealthy;
        registry.register(unhealthy);

        // Stopped worker
        let mut stopped = make_entry(AgentType::Worker, vec!["analysis"]);
        stopped.state = AgentState::Terminated;
        registry.register(stopped);

        let available =
            registry.find_available(AgentType::Worker, &["analysis".to_string()]);
        assert_eq!(available.len(), 1);
    }

    #[test]
    fn test_deregister() {
        let registry = AgentRegistry::new();
        let entry = make_entry(AgentType::Worker, vec![]);
        let id = entry.agent_id;
        registry.register(entry);

        assert_eq!(registry.count(), 1);
        registry.deregister(&id);
        assert_eq!(registry.count(), 0);
        assert!(registry.find_by_id(&id).is_none());
    }

    #[test]
    fn test_update_health() {
        let registry = AgentRegistry::new();
        let entry = make_entry(AgentType::Worker, vec![]);
        let id = entry.agent_id;
        registry.register(entry);

        registry.update_health(&id, HealthLevel::Degraded);
        let found = registry.find_by_id(&id).unwrap();
        assert_eq!(found.health, HealthLevel::Degraded);
    }
}

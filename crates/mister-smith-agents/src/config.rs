use mister_smith_core::{AgentType, RestartPolicy};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Health level derived from heartbeat regularity and check results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    /// All checks passing, heartbeat regular.
    #[default]
    Healthy,
    /// Some checks failing or heartbeat irregular.
    Degraded,
    /// Multiple checks failing.
    Unhealthy,
    /// Agent unresponsive or about to be terminated.
    Critical,
}

/// Team orchestration pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamPattern {
    /// Fan-out task assignment, fan-in result aggregation.
    SupervisorWorker,
    /// Sequential handoff from one agent to the next.
    Pipeline,
    /// Parallel evaluation with voting/majority result.
    Consensus,
}

/// Task lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Submitted, awaiting assignment.
    Pending,
    /// Assigned to an agent, awaiting execution start.
    Assigned,
    /// Agent is actively executing.
    Running,
    /// Execution finished successfully.
    Completed,
    /// Execution failed with error.
    Failed,
    /// Deadline exceeded.
    TimedOut,
    /// Cancelled by Coordinator or system.
    Cancelled,
}

/// Configuration for a specific agent instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Which role this agent fulfills.
    pub agent_type: AgentType,
    /// Supervision restart strategy.
    pub restart_policy: RestartPolicy,
    /// Time between heartbeat emissions.
    #[serde(with = "humantime_serde")]
    pub heartbeat_interval: Duration,
    /// Maximum messages in mailbox.
    pub mailbox_capacity: usize,
    /// Enable priority-ordered message processing.
    pub priority_mailbox: bool,
    /// Default timeout for task execution.
    #[serde(with = "humantime_serde")]
    pub task_timeout: Duration,
    /// Granted tool permission patterns.
    pub tool_permissions: Vec<String>,
    /// Role-specific configuration.
    pub role_config: serde_json::Value,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: AgentType::Worker,
            restart_policy: RestartPolicy::OneForOne,
            heartbeat_interval: Duration::from_secs(5),
            mailbox_capacity: 1000,
            priority_mailbox: false,
            task_timeout: Duration::from_secs(60),
            tool_permissions: Vec::new(),
            role_config: serde_json::Value::Null,
        }
    }
}

impl AgentConfig {
    /// Create a config for a specific agent type with defaults.
    pub fn for_type(agent_type: AgentType) -> Self {
        let (heartbeat_interval, mailbox_capacity, task_timeout) = match agent_type {
            AgentType::Supervisor => (Duration::from_secs(3), 1000, Duration::from_secs(60)),
            AgentType::Coordinator => (Duration::from_secs(5), 2000, Duration::from_secs(300)),
            AgentType::Monitor => (Duration::from_secs(2), 1000, Duration::from_secs(60)),
            _ => (Duration::from_secs(5), 1000, Duration::from_secs(60)),
        };

        Self {
            agent_type,
            heartbeat_interval,
            mailbox_capacity,
            task_timeout,
            ..Self::default()
        }
    }
}

/// Serde helper for Duration as milliseconds.
pub(crate) mod humantime_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

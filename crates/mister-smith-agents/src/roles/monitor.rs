//! Monitor agent role — observes and reports on system state.

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`MonitorAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorMessage {
    /// Report a health update for an agent.
    HealthUpdate {
        /// The agent whose health changed.
        agent_id: AgentId,
        /// Severity or status level (e.g. "healthy", "degraded", "critical").
        level: String,
    },
    /// Set or update an alerting threshold for a named metric.
    SetThreshold {
        /// Name of the metric.
        metric: String,
        /// Threshold value.
        value: f64,
    },
    /// Query all active alerts.
    QueryAlerts,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`MonitorAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitorState {
    /// Number of alerts currently active.
    pub active_alerts: u64,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`MonitorAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    /// A monitoring operation failed.
    #[error("monitor error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Observes and reports on system state — health updates, threshold
/// management, and alert queries.
pub struct MonitorAgent {
    id: AgentId,
}

impl MonitorAgent {
    /// Create a new `MonitorAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for MonitorAgent {
    type Message = MonitorMessage;
    type State = MonitorState;
    type Error = MonitorError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            MonitorMessage::HealthUpdate { agent_id, level } => {
                if level == "critical" || level == "unhealthy" {
                    state.active_alerts += 1;
                }
                Ok(serde_json::json!({
                    "recorded": agent_id.to_string(),
                    "level": level,
                    "active_alerts": state.active_alerts,
                }))
            }
            MonitorMessage::SetThreshold { metric, value } => Ok(serde_json::json!({
                "threshold_set": metric,
                "value": value,
            })),
            MonitorMessage::QueryAlerts => Ok(serde_json::json!({
                "active_alerts": state.active_alerts,
            })),
        }
    }

    fn pre_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post_stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn actor_id(&self) -> AgentId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_update_critical_increments_alerts() {
        let mut agent = MonitorAgent::new(AgentId::new());
        let mut state = MonitorState::default();
        assert_eq!(state.active_alerts, 0);

        let target_id = AgentId::new();
        let resp = agent
            .handle_message(
                MonitorMessage::HealthUpdate {
                    agent_id: target_id,
                    level: "critical".into(),
                },
                &mut state,
            )
            .await
            .expect("handle_message should succeed");

        assert_eq!(state.active_alerts, 1);
        assert_eq!(resp["recorded"], target_id.to_string());
        assert_eq!(resp["level"], "critical");
        assert_eq!(resp["active_alerts"], 1);
    }

    #[tokio::test]
    async fn health_update_healthy_does_not_increment() {
        let mut agent = MonitorAgent::new(AgentId::new());
        let mut state = MonitorState::default();

        let resp = agent
            .handle_message(
                MonitorMessage::HealthUpdate {
                    agent_id: AgentId::new(),
                    level: "healthy".into(),
                },
                &mut state,
            )
            .await
            .expect("handle_message should succeed");

        assert_eq!(state.active_alerts, 0);
        assert_eq!(resp["active_alerts"], 0);
    }

    #[tokio::test]
    async fn query_alerts_returns_count() {
        let mut agent = MonitorAgent::new(AgentId::new());
        let mut state = MonitorState { active_alerts: 5 };

        let resp = agent
            .handle_message(MonitorMessage::QueryAlerts, &mut state)
            .await
            .expect("handle_message should succeed");

        assert_eq!(resp["active_alerts"], 5);
    }

    #[tokio::test]
    async fn set_threshold_acknowledges() {
        let mut agent = MonitorAgent::new(AgentId::new());
        let mut state = MonitorState::default();

        let resp = agent
            .handle_message(
                MonitorMessage::SetThreshold {
                    metric: "cpu_usage".into(),
                    value: 90.0,
                },
                &mut state,
            )
            .await
            .expect("handle_message should succeed");

        assert_eq!(resp["threshold_set"], "cpu_usage");
        assert_eq!(resp["value"], 90.0);
    }
}

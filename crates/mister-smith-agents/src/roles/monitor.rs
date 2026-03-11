//! Monitor agent role — observes and reports on system state.

use mister_smith_core::{Actor, AgentId, GuardDecision, InterventionRecord};
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
    /// Record a Guard decision for operator-visible supervision state.
    GuardDecisionEvaluated(GuardDecision),
    /// Record an applied intervention.
    InterventionApplied(InterventionRecord),
    /// Query Guard/intervention counts.
    QuerySupervision,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`MonitorAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitorState {
    /// Number of alerts currently active.
    pub active_alerts: u64,
    /// Guard decisions seen by this monitor.
    pub guard_decisions: Vec<GuardDecision>,
    /// Interventions seen by this monitor.
    pub interventions: Vec<InterventionRecord>,
}

impl MonitorState {
    /// Apply a monitor message directly to state for in-process supervision sinks.
    pub fn apply(&mut self, message: &MonitorMessage) -> serde_json::Value {
        match message {
            MonitorMessage::HealthUpdate { agent_id, level } => {
                if level == "critical" || level == "unhealthy" {
                    self.active_alerts += 1;
                }
                serde_json::json!({
                    "recorded": agent_id.to_string(),
                    "level": level,
                    "active_alerts": self.active_alerts,
                })
            }
            MonitorMessage::SetThreshold { metric, value } => serde_json::json!({
                "threshold_set": metric,
                "value": value,
            }),
            MonitorMessage::QueryAlerts => serde_json::json!({
                "active_alerts": self.active_alerts,
            }),
            MonitorMessage::GuardDecisionEvaluated(decision) => {
                if decision.operator_visibility {
                    self.active_alerts += 1;
                }
                self.guard_decisions.push(decision.clone());
                serde_json::json!({
                    "guard_decisions": self.guard_decisions.len(),
                    "active_alerts": self.active_alerts,
                })
            }
            MonitorMessage::InterventionApplied(record) => {
                self.interventions.push(record.clone());
                serde_json::json!({
                    "interventions": self.interventions.len(),
                    "active_alerts": self.active_alerts,
                })
            }
            MonitorMessage::QuerySupervision => serde_json::json!({
                "guard_decisions": self.guard_decisions.len(),
                "interventions": self.interventions.len(),
                "active_alerts": self.active_alerts,
            }),
        }
    }
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
        Ok(state.apply(&message))
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
        let mut state = MonitorState {
            active_alerts: 5,
            ..MonitorState::default()
        };

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

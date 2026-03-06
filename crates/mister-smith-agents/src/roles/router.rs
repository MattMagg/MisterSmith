//! Router agent role — routes messages between agents.

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`RouterAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouterMessage {
    /// Route a message payload to the appropriate destination.
    Route(serde_json::Value),
    /// Add a routing rule mapping a pattern to a destination.
    AddRule {
        /// Pattern to match against incoming messages.
        pattern: String,
        /// Destination identifier for matching messages.
        destination: String,
    },
    /// Remove a routing rule by pattern.
    RemoveRule(String),
    /// Query all active routing rules.
    QueryRules,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// A single routing rule mapping a pattern to a destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Substring pattern to match against incoming message payloads.
    pub pattern: String,
    /// Destination identifier for matching messages.
    pub destination: String,
}

/// Persistent state for the [`RouterAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouterState {
    /// Number of messages routed since startup.
    pub messages_routed: u64,
    /// Active routing rules evaluated in order.
    pub rules: Vec<RoutingRule>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`RouterAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    /// A routing operation failed.
    #[error("router error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Routes messages between agents based on configurable rules.
pub struct RouterAgent {
    id: AgentId,
}

impl RouterAgent {
    /// Create a new `RouterAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for RouterAgent {
    type Message = RouterMessage;
    type State = RouterState;
    type Error = RouterError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            RouterMessage::Route(payload) => {
                state.messages_routed += 1;
                let payload_str = payload.to_string();
                if let Some(rule) = state
                    .rules
                    .iter()
                    .find(|r| payload_str.contains(&r.pattern))
                {
                    Ok(serde_json::json!({
                        "routed": true,
                        "destination": rule.destination,
                        "messages_routed": state.messages_routed,
                    }))
                } else {
                    Ok(serde_json::json!({
                        "routed": false,
                        "reason": "no matching rule",
                    }))
                }
            }
            RouterMessage::AddRule {
                pattern,
                destination,
            } => {
                state.rules.push(RoutingRule {
                    pattern: pattern.clone(),
                    destination: destination.clone(),
                });
                Ok(serde_json::json!({
                    "rule_added": pattern,
                    "destination": destination,
                    "total_rules": state.rules.len(),
                }))
            }
            RouterMessage::RemoveRule(pattern) => {
                state.rules.retain(|r| r.pattern != pattern);
                Ok(serde_json::json!({
                    "rule_removed": pattern,
                    "total_rules": state.rules.len(),
                }))
            }
            RouterMessage::QueryRules => {
                let rules: Vec<serde_json::Value> = state
                    .rules
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "pattern": r.pattern,
                            "destination": r.destination,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "rules": rules,
                    "messages_routed": state.messages_routed,
                }))
            }
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
    async fn add_rule_route_match_query_rules() {
        let mut agent = RouterAgent::new(AgentId::new());
        let mut state = RouterState::default();

        // Add a routing rule.
        let resp = agent
            .handle_message(
                RouterMessage::AddRule {
                    pattern: "order".into(),
                    destination: "order-service".into(),
                },
                &mut state,
            )
            .await
            .expect("AddRule should succeed");

        assert_eq!(resp["rule_added"], "order");
        assert_eq!(resp["destination"], "order-service");
        assert_eq!(resp["total_rules"], 1);

        // Route a matching payload.
        let resp = agent
            .handle_message(
                RouterMessage::Route(serde_json::json!({"type": "order", "id": 42})),
                &mut state,
            )
            .await
            .expect("Route should succeed");

        assert_eq!(resp["routed"], true);
        assert_eq!(resp["destination"], "order-service");
        assert_eq!(resp["messages_routed"], 1);

        // Query rules — should show the rule and the routed count.
        let resp = agent
            .handle_message(RouterMessage::QueryRules, &mut state)
            .await
            .expect("QueryRules should succeed");

        let rules = resp["rules"].as_array().expect("rules should be an array");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["pattern"], "order");
        assert_eq!(rules[0]["destination"], "order-service");
        assert_eq!(resp["messages_routed"], 1);
    }

    #[tokio::test]
    async fn route_no_match_returns_reason() {
        let mut agent = RouterAgent::new(AgentId::new());
        let mut state = RouterState::default();

        let resp = agent
            .handle_message(
                RouterMessage::Route(serde_json::json!({"type": "unknown"})),
                &mut state,
            )
            .await
            .expect("Route should succeed");

        assert_eq!(resp["routed"], false);
        assert_eq!(resp["reason"], "no matching rule");
        // Counter still increments even when no rule matches.
        assert_eq!(state.messages_routed, 1);
    }

    #[tokio::test]
    async fn remove_rule_then_route_fails() {
        let mut agent = RouterAgent::new(AgentId::new());
        let mut state = RouterState::default();

        // Add then remove.
        agent
            .handle_message(
                RouterMessage::AddRule {
                    pattern: "temp".into(),
                    destination: "temp-dest".into(),
                },
                &mut state,
            )
            .await
            .expect("AddRule should succeed");

        let resp = agent
            .handle_message(RouterMessage::RemoveRule("temp".into()), &mut state)
            .await
            .expect("RemoveRule should succeed");

        assert_eq!(resp["rule_removed"], "temp");
        assert_eq!(resp["total_rules"], 0);

        // Routing should now find no match.
        let resp = agent
            .handle_message(
                RouterMessage::Route(serde_json::json!({"type": "temp"})),
                &mut state,
            )
            .await
            .expect("Route should succeed");

        assert_eq!(resp["routed"], false);
    }

    #[tokio::test]
    async fn first_matching_rule_wins() {
        let mut agent = RouterAgent::new(AgentId::new());
        let mut state = RouterState::default();

        // Add two rules that both match "order".
        for (pat, dest) in [("order", "first"), ("order", "second")] {
            agent
                .handle_message(
                    RouterMessage::AddRule {
                        pattern: pat.into(),
                        destination: dest.into(),
                    },
                    &mut state,
                )
                .await
                .expect("AddRule should succeed");
        }

        let resp = agent
            .handle_message(
                RouterMessage::Route(serde_json::json!({"type": "order"})),
                &mut state,
            )
            .await
            .expect("Route should succeed");

        assert_eq!(resp["destination"], "first");
    }
}

use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use mister_smith_core::AgentId;
use serde::{Deserialize, Serialize};

/// Registry entry for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub agent_id: Option<AgentId>,
    pub mcp_session: Option<String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "crate::config::humantime_serde")]
    pub timeout: Duration,
}

/// Metrics for a registered tool.
#[derive(Debug, Default)]
pub struct ToolMetrics {
    pub invocation_count: u64,
    pub error_count: u64,
    pub total_latency_ms: u64,
}

/// Central tool registry and invocation proxy.
pub struct ToolBus {
    tools: Arc<DashMap<(String, String), ToolEntry>>,
    metrics: Arc<DashMap<(String, String), ToolMetrics>>,
}

impl ToolBus {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            metrics: Arc::new(DashMap::new()),
        }
    }

    /// Register a native agent-backed tool.
    pub fn register(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        agent_id: AgentId,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) {
        let name = name.into();
        let namespace = namespace.into();
        let key = (namespace.clone(), name.clone());
        self.tools.insert(
            key,
            ToolEntry {
                name,
                namespace,
                description: description.into(),
                input_schema,
                output_schema,
                agent_id: Some(agent_id),
                mcp_session: None,
                registered_at: chrono::Utc::now(),
                timeout: Duration::from_secs(30),
            },
        );
    }

    /// Register an MCP-backed tool.
    pub fn register_mcp(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        mcp_session: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) {
        let name = name.into();
        let namespace = namespace.into();
        let key = (namespace.clone(), name.clone());
        self.tools.insert(
            key,
            ToolEntry {
                name,
                namespace,
                description: description.into(),
                input_schema,
                output_schema,
                agent_id: None,
                mcp_session: Some(mcp_session.into()),
                registered_at: chrono::Utc::now(),
                timeout: Duration::from_secs(30),
            },
        );
    }

    /// Deregister a tool.
    pub fn deregister(&self, namespace: &str, name: &str) -> bool {
        let key = (namespace.to_string(), name.to_string());
        self.tools.remove(&key).is_some()
    }

    /// Discover tools matching a filter. Returns all tools if filter is None.
    pub fn discover(&self, namespace_filter: Option<&str>) -> Vec<ToolEntry> {
        self.tools
            .iter()
            .filter(|e| {
                namespace_filter
                    .map(|ns| e.value().namespace == ns)
                    .unwrap_or(true)
            })
            .map(|e| e.value().clone())
            .collect()
    }

    /// Look up a specific tool.
    pub fn find(&self, namespace: &str, name: &str) -> Option<ToolEntry> {
        let key = (namespace.to_string(), name.to_string());
        self.tools.get(&key).map(|e| e.value().clone())
    }

    /// Record an invocation metric.
    pub fn record_invocation(&self, namespace: &str, name: &str, latency: Duration, success: bool) {
        let key = (namespace.to_string(), name.to_string());
        let mut metrics = self.metrics.entry(key).or_default();
        metrics.invocation_count += 1;
        metrics.total_latency_ms += latency.as_millis() as u64;
        if !success {
            metrics.error_count += 1;
        }
    }

    /// Get metrics for a tool.
    pub fn get_metrics(&self, namespace: &str, name: &str) -> Option<ToolMetrics> {
        let key = (namespace.to_string(), name.to_string());
        self.metrics.get(&key).map(|e| ToolMetrics {
            invocation_count: e.invocation_count,
            error_count: e.error_count,
            total_latency_ms: e.total_latency_ms,
        })
    }

    /// Get count of registered tools.
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for ToolBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_discover() {
        let bus = ToolBus::new();
        let agent_id = AgentId::new();

        bus.register(
            "analyzer",
            "data",
            agent_id,
            "Analyzes data",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        assert_eq!(bus.count(), 1);
        let tools = bus.discover(Some("data"));
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "analyzer");
    }

    #[test]
    fn test_register_mcp() {
        let bus = ToolBus::new();
        bus.register_mcp(
            "web_search",
            "search",
            "session-123",
            "Web search tool",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let tool = bus.find("search", "web_search").unwrap();
        assert!(tool.mcp_session.is_some());
        assert!(tool.agent_id.is_none());
    }

    #[test]
    fn test_deregister() {
        let bus = ToolBus::new();
        bus.register(
            "tool1",
            "ns",
            AgentId::new(),
            "desc",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        assert!(bus.deregister("ns", "tool1"));
        assert_eq!(bus.count(), 0);
        assert!(!bus.deregister("ns", "tool1")); // already removed
    }

    #[test]
    fn test_metrics() {
        let bus = ToolBus::new();
        bus.record_invocation("ns", "tool1", Duration::from_millis(50), true);
        bus.record_invocation("ns", "tool1", Duration::from_millis(100), false);

        let metrics = bus.get_metrics("ns", "tool1").unwrap();
        assert_eq!(metrics.invocation_count, 2);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.total_latency_ms, 150);
    }
}

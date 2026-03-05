//! NATS bridge for routing MCP tool calls across distributed nodes.
//!
//! Enables distributed tool discovery and invocation by routing MCP
//! requests over NATS subjects. Handles `tools/list` aggregation
//! and `tools/call` forwarding with timeout.

use std::time::Duration;

use crate::client::McpTool;
use crate::errors::McpError;

/// Default timeout for bridge requests to remote nodes.
const DEFAULT_BRIDGE_TIMEOUT: Duration = Duration::from_secs(10);

/// Routes MCP tool calls to agents on remote nodes over NATS.
pub struct McpNatsBridge {
    /// NATS subject prefix for MCP bridge messages.
    subject_prefix: String,
    /// Timeout for remote node requests.
    timeout: Duration,
    /// Whether the bridge is active.
    active: bool,
}

impl McpNatsBridge {
    /// Create a new NATS bridge with the given subject prefix.
    pub fn new(subject_prefix: &str) -> Self {
        Self {
            subject_prefix: subject_prefix.to_string(),
            timeout: DEFAULT_BRIDGE_TIMEOUT,
            active: false,
        }
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the subject prefix.
    pub fn subject_prefix(&self) -> &str {
        &self.subject_prefix
    }

    /// Get the NATS subject for tools/list requests.
    pub fn tools_list_subject(&self) -> String {
        format!("{}.tools.list", self.subject_prefix)
    }

    /// Get the NATS subject for tools/call requests.
    pub fn tools_call_subject(&self, tool_name: &str) -> String {
        format!("{}.tools.call.{tool_name}", self.subject_prefix)
    }

    /// Start the bridge (begin listening for MCP requests on NATS).
    ///
    /// Currently a placeholder — actual NATS subscription will be added
    /// when integrating with a live NATS connection.
    pub async fn start(&mut self) -> Result<(), McpError> {
        self.active = true;
        Ok(())
    }

    /// Stop the bridge.
    pub async fn stop(&mut self) -> Result<(), McpError> {
        self.active = false;
        Ok(())
    }

    /// Check if the bridge is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Aggregate tool lists from all remote nodes via NATS request-reply.
    ///
    /// Currently a placeholder — returns empty list until NATS connection
    /// is integrated.
    pub async fn discover_remote_tools(&self) -> Result<Vec<McpTool>, McpError> {
        if !self.active {
            return Err(McpError::ConnectionFailed(
                "bridge not active".into(),
            ));
        }
        // TODO: Send request on tools_list_subject(), collect responses
        // from all nodes within timeout, aggregate and deduplicate.
        Ok(Vec::new())
    }

    /// Forward a tool call to a remote node via NATS.
    ///
    /// Currently a placeholder — returns timeout error until NATS
    /// connection is integrated.
    pub async fn call_remote_tool(
        &self,
        tool_name: &str,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        if !self.active {
            return Err(McpError::ConnectionFailed(
                "bridge not active".into(),
            ));
        }
        // TODO: Publish request on tools_call_subject(tool_name),
        // await reply within timeout.
        Err(McpError::BridgeTimeout(format!(
            "remote tool call to '{tool_name}' timed out ({}s)",
            self.timeout.as_secs()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_construction() {
        let bridge = McpNatsBridge::new("ms.mcp");
        assert_eq!(bridge.tools_list_subject(), "ms.mcp.tools.list");
        assert_eq!(
            bridge.tools_call_subject("fs.read_file"),
            "ms.mcp.tools.call.fs.read_file"
        );
    }

    #[test]
    fn custom_prefix() {
        let bridge = McpNatsBridge::new("custom.prefix");
        assert_eq!(bridge.subject_prefix(), "custom.prefix");
        assert_eq!(bridge.tools_list_subject(), "custom.prefix.tools.list");
    }

    #[tokio::test]
    async fn bridge_lifecycle() {
        let mut bridge = McpNatsBridge::new("ms.mcp");
        assert!(!bridge.is_active());

        bridge.start().await.unwrap();
        assert!(bridge.is_active());

        bridge.stop().await.unwrap();
        assert!(!bridge.is_active());
    }

    #[tokio::test]
    async fn discover_requires_active() {
        let bridge = McpNatsBridge::new("ms.mcp");
        assert!(bridge.discover_remote_tools().await.is_err());
    }

    #[tokio::test]
    async fn call_requires_active() {
        let bridge = McpNatsBridge::new("ms.mcp");
        let result = bridge
            .call_remote_tool("test.tool", serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn call_returns_timeout_when_active() {
        let mut bridge = McpNatsBridge::new("ms.mcp");
        bridge.start().await.unwrap();
        let result = bridge
            .call_remote_tool("test.tool", serde_json::json!({}))
            .await;
        assert!(matches!(result, Err(McpError::BridgeTimeout(_))));
    }

    #[test]
    fn custom_timeout() {
        let bridge = McpNatsBridge::new("ms.mcp").with_timeout(Duration::from_secs(30));
        assert_eq!(bridge.timeout, Duration::from_secs(30));
    }
}

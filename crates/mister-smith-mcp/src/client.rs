//! MCP client for connecting to external MCP servers.
//!
//! Provides tool discovery, caching, namespace prefixing, and tool invocation.
//! Actual rmcp integration will be added when connecting to real MCP servers.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::McpClientConfig;
use crate::errors::McpError;

/// Represents a discovered MCP tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    /// Tool name (without namespace prefix).
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema for tool parameters.
    pub input_schema: serde_json::Value,
}

/// MCP client for connecting to and invoking tools on external MCP servers.
pub struct McpClient {
    config: McpClientConfig,
    /// Cached tool list, keyed by namespaced name.
    tool_cache: Arc<RwLock<HashMap<String, McpTool>>>,
    /// Whether the client is connected.
    connected: Arc<RwLock<bool>>,
}

impl McpClient {
    /// Create a new MCP client (not yet connected).
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            config,
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the namespace for this client's tools.
    pub fn namespace(&self) -> &str {
        &self.config.namespace
    }

    /// Get the server name.
    pub fn server_name(&self) -> &str {
        &self.config.name
    }

    /// Check if the client is connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Connect to the MCP server and discover tools.
    ///
    /// Currently a placeholder — actual rmcp connection will be added
    /// when integrating with real MCP servers.
    pub async fn connect(&self) -> Result<(), McpError> {
        // TODO: Use rmcp to establish actual connection.
        // For now, mark as connected.
        let mut connected = self.connected.write().await;
        *connected = true;
        Ok(())
    }

    /// Discover available tools from the MCP server.
    pub async fn discover_tools(&self) -> Result<Vec<McpTool>, McpError> {
        if !*self.connected.read().await {
            return Err(McpError::ConnectionFailed(
                "not connected to MCP server".into(),
            ));
        }

        // Return cached tools.
        let cache = self.tool_cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    /// Get a tool by its namespaced name.
    pub async fn get_tool(&self, namespaced_name: &str) -> Result<McpTool, McpError> {
        let cache = self.tool_cache.read().await;
        cache
            .get(namespaced_name)
            .cloned()
            .ok_or_else(|| McpError::ToolNotFound(namespaced_name.to_string()))
    }

    /// Register a tool in the cache (used during discovery).
    pub async fn register_tool(&self, tool: McpTool) {
        let namespaced = format!("{}.{}", self.config.namespace, tool.name);
        let mut cache = self.tool_cache.write().await;
        cache.insert(namespaced, tool);
    }

    /// Invoke a tool by name with the given parameters.
    ///
    /// Currently a placeholder — actual rmcp tool invocation will be added.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        if !*self.connected.read().await {
            return Err(McpError::ConnectionFailed("not connected".into()));
        }

        // Verify tool exists.
        let namespaced = format!("{}.{tool_name}", self.config.namespace);
        let _tool = self.get_tool(&namespaced).await?;

        // TODO: Use rmcp to invoke the actual tool.
        // For now, return the params as a placeholder response.
        Ok(serde_json::json!({
            "tool": tool_name,
            "params": params,
            "status": "placeholder"
        }))
    }

    /// Invalidate the tool cache (called on tools/list_changed notification).
    pub async fn invalidate_cache(&self) {
        let mut cache = self.tool_cache.write().await;
        cache.clear();
    }

    /// Disconnect from the MCP server.
    pub async fn disconnect(&self) -> Result<(), McpError> {
        let mut connected = self.connected.write().await;
        *connected = false;
        self.invalidate_cache().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::McpTransportType;

    fn test_config() -> McpClientConfig {
        McpClientConfig {
            name: "test-server".into(),
            transport: McpTransportType::Stdio,
            command: Some("echo".into()),
            url: None,
            tool_filter: Vec::new(),
            namespace: "test".into(),
        }
    }

    #[tokio::test]
    async fn client_lifecycle() {
        let client = McpClient::new(test_config());
        assert!(!client.is_connected().await);

        client.connect().await.unwrap();
        assert!(client.is_connected().await);

        client.disconnect().await.unwrap();
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn tool_cache() {
        let client = McpClient::new(test_config());
        client.connect().await.unwrap();

        let tool = McpTool {
            name: "read_file".into(),
            description: "Read a file".into(),
            input_schema: serde_json::json!({"type": "object"}),
        };

        client.register_tool(tool).await;

        let found = client.get_tool("test.read_file").await.unwrap();
        assert_eq!(found.name, "read_file");

        client.invalidate_cache().await;
        assert!(client.get_tool("test.read_file").await.is_err());
    }

    #[tokio::test]
    async fn discover_requires_connection() {
        let client = McpClient::new(test_config());
        assert!(client.discover_tools().await.is_err());
    }
}

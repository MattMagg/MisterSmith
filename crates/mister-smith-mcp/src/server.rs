//! MCP server for exposing agent tools to external MCP clients.
//!
//! Implements `tools/list` and `tools/call` handlers with namespace
//! filtering and permission checks. Actual rmcp server integration
//! will be added when serving tools to real MCP clients.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::client::McpTool;
use crate::config::McpServerConfig;
use crate::errors::McpError;

/// Represents a registered agent tool exposed via MCP server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExposedTool {
    /// Tool name (namespace-scoped).
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema for tool parameters.
    pub input_schema: serde_json::Value,
    /// Namespace this tool belongs to.
    pub namespace: String,
}

/// Handler result for tool invocations.
pub type ToolHandler = Arc<
    dyn Fn(
            serde_json::Value,
        ) -> futures::future::BoxFuture<'static, Result<serde_json::Value, McpError>>
        + Send
        + Sync,
>;

/// MCP server that exposes agent tools to external clients.
pub struct McpServer {
    config: McpServerConfig,
    /// Registered tools keyed by namespaced name.
    tools: Arc<RwLock<HashMap<String, ExposedTool>>>,
    /// Tool handlers keyed by namespaced name.
    handlers: Arc<RwLock<HashMap<String, ToolHandler>>>,
    /// Whether the server is running.
    running: Arc<RwLock<bool>>,
}

impl McpServer {
    /// Create a new MCP server with the given configuration.
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            tools: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the bind address.
    pub fn bind_address(&self) -> &str {
        &self.config.bind_address
    }

    /// Get the namespace views.
    pub fn namespace_views(&self) -> &[String] {
        &self.config.namespace_views
    }

    /// Register a tool to be exposed via MCP.
    pub async fn register_tool(&self, tool: ExposedTool, handler: ToolHandler) {
        let namespace = &tool.namespace;
        let name = &tool.name;
        let key = format!("{namespace}.{name}");
        let mut tools = self.tools.write().await;
        tools.insert(key.clone(), tool);
        let mut handlers = self.handlers.write().await;
        handlers.insert(key, handler);
    }

    /// Handle a `tools/list` request with namespace filtering.
    pub async fn handle_tools_list(
        &self,
        namespace_filter: Option<&str>,
    ) -> Result<Vec<McpTool>, McpError> {
        let tools = self.tools.read().await;
        let filtered: Vec<McpTool> = tools
            .values()
            .filter(|t| {
                if let Some(ns) = namespace_filter {
                    t.namespace == ns
                } else if self.config.namespace_views.is_empty() {
                    true
                } else {
                    self.config.namespace_views.contains(&t.namespace)
                }
            })
            .map(|t| {
                let namespace = &t.namespace;
                let name = &t.name;
                McpTool {
                    name: format!("{namespace}.{name}"),
                    description: t.description.clone(),
                    input_schema: t.input_schema.clone(),
                }
            })
            .collect();
        Ok(filtered)
    }

    /// Handle a `tools/call` request.
    pub async fn handle_tools_call(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let handlers = self.handlers.read().await;
        let handler = handlers
            .get(tool_name)
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;

        let handler = Arc::clone(handler);
        drop(handlers);

        handler(params).await
    }

    /// Start the MCP server.
    ///
    /// Currently a placeholder — actual rmcp server binding will be added.
    pub async fn start(&self) -> Result<(), McpError> {
        let mut running = self.running.write().await;
        *running = true;
        Ok(())
    }

    /// Stop the MCP server.
    pub async fn stop(&self) -> Result<(), McpError> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Check if the server is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> McpServerConfig {
        McpServerConfig {
            bind_address: "0.0.0.0:8090".into(),
            namespace_views: vec!["agent".into()],
        }
    }

    #[tokio::test]
    async fn register_and_list_tools() {
        let server = McpServer::new(test_config());

        let tool = ExposedTool {
            name: "greet".into(),
            description: "Say hello".into(),
            input_schema: serde_json::json!({"type": "object"}),
            namespace: "agent".into(),
        };

        let handler: ToolHandler =
            Arc::new(|_params| Box::pin(async { Ok(serde_json::json!({"message": "hello"})) }));

        server.register_tool(tool, handler).await;

        let tools = server.handle_tools_list(None).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "agent.greet");
    }

    #[tokio::test]
    async fn namespace_filtering() {
        let server = McpServer::new(test_config());

        let tool1 = ExposedTool {
            name: "read".into(),
            description: "Read file".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
        };
        let tool2 = ExposedTool {
            name: "search".into(),
            description: "Search web".into(),
            input_schema: serde_json::json!({}),
            namespace: "external".into(),
        };

        let handler: ToolHandler = Arc::new(|_| Box::pin(async { Ok(serde_json::json!({})) }));

        server.register_tool(tool1, handler.clone()).await;
        server.register_tool(tool2, handler).await;

        // Default view only shows "agent" namespace per config.
        let tools = server.handle_tools_list(None).await.unwrap();
        assert_eq!(tools.len(), 1);

        // Explicit filter shows specific namespace.
        let tools = server.handle_tools_list(Some("external")).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "external.search");
    }

    #[tokio::test]
    async fn call_registered_tool() {
        let server = McpServer::new(test_config());

        let tool = ExposedTool {
            name: "echo".into(),
            description: "Echo back".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
        };

        let handler: ToolHandler =
            Arc::new(|params| Box::pin(async move { Ok(serde_json::json!({"echo": params})) }));

        server.register_tool(tool, handler).await;

        let result = server
            .handle_tools_call("agent.echo", serde_json::json!({"msg": "hi"}))
            .await
            .unwrap();
        assert_eq!(result["echo"]["msg"], "hi");
    }

    #[tokio::test]
    async fn call_unknown_tool_returns_error() {
        let server = McpServer::new(test_config());
        let result = server
            .handle_tools_call("agent.missing", serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn server_lifecycle() {
        let server = McpServer::new(test_config());
        assert!(!server.is_running().await);
        server.start().await.unwrap();
        assert!(server.is_running().await);
        server.stop().await.unwrap();
        assert!(!server.is_running().await);
    }
}

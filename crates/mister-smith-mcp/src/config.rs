//! MCP integration configuration.

use serde::{Deserialize, Serialize};

/// Top-level MCP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable MCP integration.
    #[serde(default)]
    pub enabled: bool,

    /// External MCP server connections (client mode).
    #[serde(default)]
    pub clients: Vec<McpClientConfig>,

    /// MCP server endpoints (server mode).
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,

    /// Enable NATS-MCP bridge for distributed tool calls.
    #[serde(default)]
    pub nats_bridge_enabled: bool,

    /// NATS subject prefix for MCP bridge messages.
    #[serde(default = "default_bridge_prefix")]
    pub nats_bridge_prefix: String,
}

/// Configuration for connecting to an external MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// Unique name for this MCP server connection.
    pub name: String,

    /// Transport type: "stdio" or "streamable-http".
    pub transport: McpTransportType,

    /// Command to launch (for stdio transport).
    #[serde(default)]
    pub command: Option<String>,

    /// URL to connect to (for streamable-HTTP transport).
    #[serde(default)]
    pub url: Option<String>,

    /// Tool name filter patterns (glob-style).
    #[serde(default)]
    pub tool_filter: Vec<String>,

    /// Namespace prefix for tools from this server.
    pub namespace: String,
}

/// Configuration for exposing tools via MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Bind address for the MCP server.
    #[serde(default = "default_mcp_bind")]
    pub bind_address: String,

    /// Namespace views — which tool namespaces to expose to clients.
    #[serde(default)]
    pub namespace_views: Vec<String>,
}

/// MCP transport type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpTransportType {
    /// Standard I/O transport (subprocess).
    Stdio,
    /// Streamable HTTP transport.
    StreamableHttp,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            clients: Vec::new(),
            servers: Vec::new(),
            nats_bridge_enabled: false,
            nats_bridge_prefix: default_bridge_prefix(),
        }
    }
}

fn default_bridge_prefix() -> String {
    "ms.mcp".to_string()
}

fn default_mcp_bind() -> String {
    "0.0.0.0:8090".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = McpConfig::default();
        assert!(!config.enabled);
        assert!(config.clients.is_empty());
        assert_eq!(config.nats_bridge_prefix, "ms.mcp");
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = McpConfig {
            enabled: true,
            clients: vec![McpClientConfig {
                name: "filesystem".into(),
                transport: McpTransportType::Stdio,
                command: Some("mcp-server-filesystem".into()),
                url: None,
                tool_filter: vec!["read_*".into()],
                namespace: "fs".into(),
            }],
            servers: Vec::new(),
            nats_bridge_enabled: true,
            nats_bridge_prefix: "ms.mcp".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: McpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.clients.len(), 1);
        assert_eq!(decoded.clients[0].name, "filesystem");
    }
}

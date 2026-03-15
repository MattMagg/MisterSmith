//! MCP integration for bidirectional tool discovery and invocation.
//!
//! Connects agents to external MCP tool servers and exposes agent tools
//! to external MCP clients. Isolated crate to contain rmcp/thiserror ^2 dependency.

pub mod bridge;
pub mod client;
pub mod compatibility;
pub mod config;
pub mod errors;
pub mod resources;
pub mod server;
pub mod session;

pub use bridge::McpNatsBridge;
pub use client::{McpClient, McpTool};
pub use compatibility::{
    build_smith_compatibility_server, CompatibilityStatus, SmithCompatibilityOptions, ToolResponse,
};
pub use config::{McpClientConfig, McpConfig, McpServerConfig, McpTransportType};
pub use errors::McpError;
pub use resources::{McpResource, ResourceRegistry};
pub use server::{ExposedTool, McpServer};
pub use session::{McpSessionManager, SessionState};

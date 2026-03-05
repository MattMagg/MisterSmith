//! MCP-specific error types.
//!
//! Wraps rmcp errors at the crate boundary using thiserror 1.x
//! to isolate the thiserror ^2 dependency from rmcp.

use thiserror::Error;

/// Errors specific to the MCP integration.
#[derive(Debug, Error)]
pub enum McpError {
    /// Failed to connect to an MCP server.
    #[error("MCP connection failed: {0}")]
    ConnectionFailed(String),

    /// Tool not found in the registry.
    #[error("MCP tool not found: {0}")]
    ToolNotFound(String),

    /// Tool invocation failed.
    #[error("MCP tool call failed: {0}")]
    ToolCallFailed(String),

    /// Session management error.
    #[error("MCP session error: {0}")]
    SessionError(String),

    /// NATS bridge operation timed out.
    #[error("MCP bridge timeout: {0}")]
    BridgeTimeout(String),

    /// Namespace conflict between MCP servers.
    #[error("MCP namespace conflict: {0}")]
    NamespaceConflict(String),

    /// Serialization/deserialization error.
    #[error("MCP serialization error: {0}")]
    SerializationError(String),
}

impl From<McpError> for mister_smith_transport::TransportError {
    fn from(err: McpError) -> Self {
        match err {
            McpError::ConnectionFailed(msg) => {
                mister_smith_transport::TransportError::ConnectionFailed(msg)
            }
            McpError::BridgeTimeout(msg) => mister_smith_transport::TransportError::Timeout(msg),
            other => mister_smith_transport::TransportError::ProtocolError(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_error_display() {
        let err = McpError::ToolNotFound("read_file".into());
        assert_eq!(err.to_string(), "MCP tool not found: read_file");
    }

    #[test]
    fn mcp_to_transport_error() {
        let err = McpError::ConnectionFailed("refused".into());
        let transport_err: mister_smith_transport::TransportError = err.into();
        assert!(matches!(
            transport_err,
            mister_smith_transport::TransportError::ConnectionFailed(_)
        ));
    }
}

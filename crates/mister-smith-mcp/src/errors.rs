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

// rmcp 1.1.0 deprecated the `Error` struct constructors (e.g. `Error::resource_not_found`).
// These are still functional and the replacements are not yet stable. Track migration in a
// follow-up once rmcp 2.x settles the new error API.
#[allow(deprecated)]
impl From<rmcp::Error> for McpError {
    fn from(err: rmcp::Error) -> Self {
        let code = err.code.0;
        let message = err.message.to_string();
        let context = format!("rmcp error (code {code}): {message}");
        let lowered = message.to_ascii_lowercase();

        if lowered.contains("session") {
            return McpError::SessionError(context);
        }

        if lowered.contains("timeout") || lowered.contains("timed out") {
            return McpError::BridgeTimeout(context);
        }

        if err.code == rmcp::model::ErrorCode::PARSE_ERROR {
            return McpError::SerializationError(context);
        }

        if err.code == rmcp::model::ErrorCode::RESOURCE_NOT_FOUND
            || (err.code == rmcp::model::ErrorCode::METHOD_NOT_FOUND
                && lowered.contains("tool"))
            || lowered.contains("tool not found")
        {
            return McpError::ToolNotFound(context);
        }

        if lowered.contains("connect")
            || lowered.contains("connection")
            || lowered.contains("transport")
            || lowered.contains("network")
            || lowered.contains("refused")
        {
            return McpError::ConnectionFailed(context);
        }

        McpError::ToolCallFailed(context)
    }
}

impl From<rmcp::RmcpError> for McpError {
    fn from(err: rmcp::RmcpError) -> Self {
        match err {
            rmcp::RmcpError::TransportCreation { error, .. } => {
                McpError::ConnectionFailed(format!("rmcp transport creation failed: {error}"))
            }
            rmcp::RmcpError::Runtime(join_error) => {
                McpError::SessionError(format!("rmcp runtime/session task failed: {join_error}"))
            }
            rmcp::RmcpError::TaskError(message) => {
                McpError::ToolCallFailed(format!("rmcp task error during tool call: {message}"))
            }
            _ => McpError::ToolCallFailed(format!("rmcp error: {err}")),
        }
    }
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

    #[allow(deprecated)]
    #[test]
    fn rmcp_error_maps_to_tool_not_found() {
        let rmcp_err = rmcp::Error::resource_not_found("tool not found: read_file", None);
        let converted = McpError::from(rmcp_err);

        assert!(matches!(converted, McpError::ToolNotFound(msg) if msg.contains("tool not found")));
    }

    #[allow(deprecated)]
    #[test]
    fn rmcp_error_maps_to_session_error() {
        let rmcp_err = rmcp::Error::invalid_request("session expired", None);
        let converted = McpError::from(rmcp_err);

        assert!(matches!(converted, McpError::SessionError(msg) if msg.contains("session expired")));
    }

    #[allow(deprecated)]
    #[test]
    fn rmcp_error_maps_to_connection_failure() {
        let rmcp_err = rmcp::Error::internal_error("transport connection refused", None);
        let converted = McpError::from(rmcp_err);

        assert!(matches!(
            converted,
            McpError::ConnectionFailed(msg) if msg.contains("connection refused")
        ));
    }

    #[allow(deprecated)]
    #[test]
    fn rmcp_error_maps_to_serialization_error() {
        let rmcp_err = rmcp::Error::parse_error("invalid JSON", None);
        let converted = McpError::from(rmcp_err);

        assert!(matches!(converted, McpError::SerializationError(msg) if msg.contains("invalid JSON")));
    }
}

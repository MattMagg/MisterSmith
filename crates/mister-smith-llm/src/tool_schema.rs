use serde::{Deserialize, Serialize};

/// Provider-neutral description of a callable tool exported from the ToolBus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Stable tool identifier presented to the model.
    pub name: String,
    /// Human-readable purpose summary.
    pub description: String,
    /// JSON Schema object describing tool input arguments.
    pub input_schema: serde_json::Value,
}

/// Model-emitted request to invoke a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Stable identifier for matching the eventual result.
    pub call_id: String,
    /// Requested tool name.
    pub name: String,
    /// Structured input payload to dispatch through the ToolBus.
    pub input: serde_json::Value,
}

/// Structured result returned from ToolBus execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Identifier matching the originating tool call.
    pub call_id: String,
    /// Structured tool output on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Human-readable failure description on error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    /// Construct a successful tool result.
    pub fn success(call_id: impl Into<String>, output: serde_json::Value) -> Self {
        Self {
            call_id: call_id.into(),
            output: Some(output),
            error: None,
        }
    }

    /// Construct a failed tool result.
    pub fn failure(call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            output: None,
            error: Some(error.into()),
        }
    }
}

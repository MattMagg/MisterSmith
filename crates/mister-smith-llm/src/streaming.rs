use serde::{Deserialize, Serialize};

use crate::types::StopReason;

/// Normalized streaming chunk emitted by [`crate::ModelProvider::stream`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Stable ordering index for chunk assembly.
    pub index: usize,
    /// Partial content or terminal signal.
    pub delta: ChunkDelta,
}

impl StreamChunk {
    /// Create a terminal stop chunk.
    pub fn stop(index: usize, reason: StopReason) -> Self {
        Self {
            index,
            delta: ChunkDelta::Stop { reason },
        }
    }
}

/// Provider-neutral streaming delta variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChunkDelta {
    /// Incremental text output.
    Text {
        /// Newly emitted text content.
        text: String,
    },
    /// Start of a tool-use request.
    ToolUseStart {
        /// Stable tool call identifier.
        call_id: String,
        /// Requested tool name.
        name: String,
    },
    /// Structured tool input associated with a prior tool-use start.
    ToolUseInput {
        /// Stable tool call identifier.
        call_id: String,
        /// Structured tool arguments.
        input: serde_json::Value,
    },
    /// Terminal stop marker.
    Stop {
        /// Provider-neutral stop reason.
        reason: StopReason,
    },
}

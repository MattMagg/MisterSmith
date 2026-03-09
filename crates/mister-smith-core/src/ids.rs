//! Core identifier newtypes for the framework.
//!
//! Each ID is a newtype wrapper around [`Uuid`] providing type safety
//! so that agent IDs, task IDs, message IDs, and tool IDs cannot be
//! accidentally interchanged.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an agent in the framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    /// Create a new random agent ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an agent ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for AgentId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

/// Unique identifier for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    /// Create a new random task ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a task ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for TaskId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

/// Unique identifier for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    /// Create a new random message ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a message ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for MessageId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

/// Unique identifier for a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub Uuid);

impl ToolId {
    /// Create a new random tool ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a tool ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ToolId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for ToolId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

/// Unique identifier for a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

impl ResourceId {
    /// Create a new random resource ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a resource ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for ResourceId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id_uniqueness() {
        let a = AgentId::new();
        let b = AgentId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn id_display_format() {
        let uuid = Uuid::nil();
        let id = AgentId::from_uuid(uuid);
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn id_serde_roundtrip() {
        let id = TaskId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: TaskId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn id_from_uuid_conversion() {
        let uuid = Uuid::new_v4();

        let agent_id = AgentId::from_uuid(uuid);
        assert_eq!(*agent_id.as_ref(), uuid);

        let task_id = TaskId::from_uuid(uuid);
        assert_eq!(*task_id.as_ref(), uuid);

        let message_id = MessageId::from_uuid(uuid);
        assert_eq!(*message_id.as_ref(), uuid);

        let tool_id = ToolId::from_uuid(uuid);
        assert_eq!(*tool_id.as_ref(), uuid);

        let resource_id = ResourceId::from_uuid(uuid);
        assert_eq!(*resource_id.as_ref(), uuid);
    }
}

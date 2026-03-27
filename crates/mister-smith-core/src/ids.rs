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

/// Unique identifier for a conversation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    /// Create a new random session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a session ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Uuid> for SessionId {
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

macro_rules! define_uuid_id {
    ($(#[$meta:meta])* $name:ident, $new_doc:literal, $from_doc:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            #[doc = $new_doc]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[doc = $from_doc]
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }
    };
}

define_uuid_id!(
    /// Stable identifier for an execution graph.
    ExecutionGraphId,
    "Create a new random execution graph ID.",
    "Create an execution graph ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for an execution branch.
    ExecutionBranchId,
    "Create a new random execution branch ID.",
    "Create an execution branch ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for an execution node.
    ExecutionNodeId,
    "Create a new random execution node ID.",
    "Create an execution node ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a branch checkpoint.
    CheckpointId,
    "Create a new random checkpoint ID.",
    "Create a checkpoint ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a context budget.
    ContextBudgetId,
    "Create a new random context budget ID.",
    "Create a context budget ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a managed memory fragment.
    MemoryFragmentId,
    "Create a new random memory fragment ID.",
    "Create a memory fragment ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a memory snapshot.
    MemorySnapshotId,
    "Create a new random memory snapshot ID.",
    "Create a memory snapshot ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a profile snapshot.
    ProfileSnapshotId,
    "Create a new random profile snapshot ID.",
    "Create a profile snapshot ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a profile fingerprint.
    ProfileFingerprintId,
    "Create a new random profile fingerprint ID.",
    "Create a profile fingerprint ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a Guard decision.
    GuardDecisionId,
    "Create a new random Guard decision ID.",
    "Create a Guard decision ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for an intervention record.
    InterventionRecordId,
    "Create a new random intervention record ID.",
    "Create an intervention record ID from an existing UUID."
);
define_uuid_id!(
    /// Stable identifier for a delegation capability.
    CapabilityId,
    "Create a new random capability ID.",
    "Create a capability ID from an existing UUID."
);

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

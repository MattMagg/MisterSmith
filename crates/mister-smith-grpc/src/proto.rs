//! Protobuf message types for gRPC services.
//!
//! These types correspond to the proto definitions in
//! `specs/004-phase4-transport-messaging/contracts/`. They are defined as plain
//! Rust structs here because the transport crate (which will eventually compile
//! protos via prost-build) does not yet exist. Once `mister-smith-transport` is
//! available, these types should be replaced with re-exports from that crate.

use prost::Message;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Shared Enums (common.proto)
// ---------------------------------------------------------------------------

/// Message priority levels (maps to proto `MessagePriority`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum MessagePriority {
    /// Normal priority.
    #[default]
    Normal = 0,
    /// Low priority.
    Low = 1,
    /// High priority.
    High = 2,
    /// Critical priority.
    Critical = 3,
}

impl MessagePriority {
    /// Convert from `i32` wire value.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Low),
            2 => Some(Self::High),
            3 => Some(Self::Critical),
            _ => None,
        }
    }
}

impl From<i32> for MessagePriority {
    fn from(value: i32) -> Self {
        Self::from_i32(value).unwrap_or_default()
    }
}

impl From<MessagePriority> for i32 {
    fn from(value: MessagePriority) -> Self {
        value as i32
    }
}

/// Agent availability (maps to proto `AgentAvailability`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum AgentAvailability {
    /// Agent is idle.
    #[default]
    Idle = 0,
    /// Agent is busy.
    Busy = 1,
    /// Agent is offline.
    Offline = 2,
}

impl AgentAvailability {
    /// Convert from `i32` wire value.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Busy),
            2 => Some(Self::Offline),
            _ => None,
        }
    }
}

impl From<i32> for AgentAvailability {
    fn from(value: i32) -> Self {
        Self::from_i32(value).unwrap_or_default()
    }
}

impl From<AgentAvailability> for i32 {
    fn from(value: AgentAvailability) -> Self {
        value as i32
    }
}

/// Task completion status (maps to proto `TaskStatus`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum TaskStatus {
    /// Task completed successfully.
    #[default]
    Success = 0,
    /// Task failed.
    Failure = 1,
    /// Task partially completed.
    Partial = 2,
}

impl TaskStatus {
    /// Convert from `i32` wire value.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Success),
            1 => Some(Self::Failure),
            2 => Some(Self::Partial),
            _ => None,
        }
    }
}

impl From<i32> for TaskStatus {
    fn from(value: i32) -> Self {
        Self::from_i32(value).unwrap_or_default()
    }
}

impl From<TaskStatus> for i32 {
    fn from(value: TaskStatus) -> Self {
        value as i32
    }
}

/// Event severity (maps to proto `Severity`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum Severity {
    /// Informational event.
    #[default]
    Info = 0,
    /// Warning event.
    Warning = 1,
    /// Error event.
    Error = 2,
    /// Critical event.
    Critical = 3,
}

impl Severity {
    /// Convert from `i32` wire value.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Info),
            1 => Some(Self::Warning),
            2 => Some(Self::Error),
            3 => Some(Self::Critical),
            _ => None,
        }
    }
}

impl From<i32> for Severity {
    fn from(value: i32) -> Self {
        Self::from_i32(value).unwrap_or_default()
    }
}

impl From<Severity> for i32 {
    fn from(value: Severity) -> Self {
        value as i32
    }
}

// ---------------------------------------------------------------------------
// Shared Messages (common.proto)
// ---------------------------------------------------------------------------

/// Envelope wrapping all gRPC messages with metadata.
#[derive(Clone, PartialEq, Message)]
pub struct MessageEnvelope {
    /// UUID v4 message identifier.
    #[prost(string, tag = "1")]
    pub message_id: String,
    /// Creation timestamp.
    #[prost(message, optional, tag = "2")]
    pub timestamp: Option<prost_types::Timestamp>,
    /// Schema version (semver).
    #[prost(string, tag = "3")]
    pub schema_version: String,
    /// Message type discriminator.
    #[prost(string, tag = "4")]
    pub message_type: String,
    /// Optional correlation ID for request tracing.
    #[prost(string, optional, tag = "5")]
    pub correlation_id: Option<String>,
    /// Optional distributed trace ID.
    #[prost(string, optional, tag = "6")]
    pub trace_id: Option<String>,
    /// Optional source agent identifier.
    #[prost(string, optional, tag = "7")]
    pub source_agent_id: Option<String>,
    /// Optional target agent identifier.
    #[prost(string, optional, tag = "8")]
    pub target_agent_id: Option<String>,
    /// Message priority.
    #[prost(enumeration = "MessagePriority", tag = "9")]
    pub priority: i32,
    /// Serialized inner message payload.
    #[prost(bytes = "vec", tag = "10")]
    pub payload: Vec<u8>,
    /// Arbitrary key-value headers.
    #[prost(map = "string, string", tag = "11")]
    pub headers: HashMap<String, String>,
}

/// Agent information for status reporting.
#[derive(Clone, PartialEq, Message)]
pub struct AgentInfo {
    /// Agent unique identifier.
    #[prost(string, tag = "1")]
    pub agent_id: String,
    /// Agent type classification.
    #[prost(string, tag = "2")]
    pub agent_type: String,
    /// Current availability.
    #[prost(enumeration = "AgentAvailability", tag = "3")]
    pub availability: i32,
    /// Number of active tasks.
    #[prost(uint32, tag = "4")]
    pub active_tasks: u32,
    /// Agent start time.
    #[prost(message, optional, tag = "5")]
    pub started_at: Option<prost_types::Timestamp>,
    /// Current load factor (0.0-1.0).
    #[prost(double, tag = "6")]
    pub load: f64,
}

/// Task assignment request payload.
#[derive(Clone, PartialEq, Message)]
pub struct TaskAssignment {
    /// Task unique identifier.
    #[prost(string, tag = "1")]
    pub task_id: String,
    /// Task type classification.
    #[prost(string, tag = "2")]
    pub task_type: String,
    /// Task payload as a protobuf Struct.
    #[prost(message, optional, tag = "3")]
    pub payload: Option<prost_types::Struct>,
    /// Task priority.
    #[prost(enumeration = "MessagePriority", tag = "4")]
    pub priority: i32,
    /// Optional task deadline.
    #[prost(message, optional, tag = "5")]
    pub deadline: Option<prost_types::Timestamp>,
    /// Optional assigned agent.
    #[prost(string, optional, tag = "6")]
    pub assigned_agent: Option<String>,
    /// Requester agent identifier.
    #[prost(string, tag = "7")]
    pub requester_id: String,
    /// Arbitrary metadata.
    #[prost(map = "string, string", tag = "8")]
    pub metadata: HashMap<String, String>,
}

/// Task execution result.
#[derive(Clone, PartialEq, Message)]
pub struct TaskResult {
    /// Task unique identifier.
    #[prost(string, tag = "1")]
    pub task_id: String,
    /// Task completion status.
    #[prost(enumeration = "TaskStatus", tag = "2")]
    pub status: i32,
    /// Optional result payload.
    #[prost(message, optional, tag = "3")]
    pub result: Option<prost_types::Struct>,
    /// Optional error description.
    #[prost(string, optional, tag = "4")]
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    #[prost(uint64, tag = "5")]
    pub duration_ms: u64,
    /// Agent that executed the task.
    #[prost(string, tag = "6")]
    pub agent_id: String,
}

/// System event for monitoring and alerting.
#[derive(Clone, PartialEq, Message)]
pub struct SystemEvent {
    /// Event type identifier.
    #[prost(string, tag = "1")]
    pub event_type: String,
    /// Event source.
    #[prost(string, tag = "2")]
    pub source: String,
    /// Event severity.
    #[prost(enumeration = "Severity", tag = "3")]
    pub severity: i32,
    /// Human-readable event message.
    #[prost(string, tag = "4")]
    pub message: String,
    /// Optional structured event data.
    #[prost(message, optional, tag = "5")]
    pub data: Option<prost_types::Struct>,
    /// Event timestamp.
    #[prost(message, optional, tag = "6")]
    pub timestamp: Option<prost_types::Timestamp>,
}

// ---------------------------------------------------------------------------
// Agent Service Messages (agent_service.proto)
// ---------------------------------------------------------------------------

/// Request to list active agents.
#[derive(Clone, PartialEq, Message)]
pub struct ListAgentsRequest {
    /// Optional availability filter.
    #[prost(enumeration = "AgentAvailability", optional, tag = "1")]
    pub filter_availability: Option<i32>,
    /// Optional agent type filter.
    #[prost(string, optional, tag = "2")]
    pub filter_type: Option<String>,
}

/// Response containing a list of agents.
#[derive(Clone, PartialEq, Message)]
pub struct ListAgentsResponse {
    /// List of matching agents.
    #[prost(message, repeated, tag = "1")]
    pub agents: Vec<AgentInfo>,
}

/// Request to get a specific agent.
#[derive(Clone, PartialEq, Message)]
pub struct GetAgentRequest {
    /// Agent identifier to look up.
    #[prost(string, tag = "1")]
    pub agent_id: String,
}

/// Request to submit a task for execution.
#[derive(Clone, PartialEq, Message)]
pub struct SubmitTaskRequest {
    /// Task assignment details.
    #[prost(message, optional, tag = "1")]
    pub task: Option<TaskAssignment>,
}

/// Response confirming task submission.
#[derive(Clone, PartialEq, Message)]
pub struct SubmitTaskResponse {
    /// Assigned task identifier.
    #[prost(string, tag = "1")]
    pub task_id: String,
    /// Agent assigned to execute the task.
    #[prost(string, tag = "2")]
    pub assigned_agent_id: String,
}

/// Request to get a task result.
#[derive(Clone, PartialEq, Message)]
pub struct GetTaskResultRequest {
    /// Task identifier.
    #[prost(string, tag = "1")]
    pub task_id: String,
}

/// Request to stream agent status updates.
#[derive(Clone, PartialEq, Message)]
pub struct StreamAgentStatusRequest {
    /// Optional agent ID filter (empty = all agents).
    #[prost(string, optional, tag = "1")]
    pub agent_id: Option<String>,
}

/// Agent status update event.
#[derive(Clone, PartialEq, Message)]
pub struct AgentStatusUpdate {
    /// Agent identifier.
    #[prost(string, tag = "1")]
    pub agent_id: String,
    /// Current availability.
    #[prost(enumeration = "AgentAvailability", tag = "2")]
    pub availability: i32,
    /// Update timestamp.
    #[prost(message, optional, tag = "3")]
    pub timestamp: Option<prost_types::Timestamp>,
    /// Optional current load.
    #[prost(double, optional, tag = "4")]
    pub load: Option<f64>,
    /// Optional active task count.
    #[prost(uint32, optional, tag = "5")]
    pub active_tasks: Option<u32>,
}

// ---------------------------------------------------------------------------
// System Service Messages (system_service.proto)
// ---------------------------------------------------------------------------

/// Request to stream system events.
#[derive(Clone, PartialEq, Message)]
pub struct StreamEventsRequest {
    /// Filter by event types (empty = all events).
    #[prost(string, repeated, tag = "1")]
    pub event_types: Vec<String>,
    /// Optional minimum severity filter.
    #[prost(enumeration = "Severity", optional, tag = "2")]
    pub min_severity: Option<i32>,
}

/// Request to get system configuration.
#[derive(Clone, PartialEq, Message)]
pub struct GetConfigRequest {
    /// Optional component name filter (empty = all).
    #[prost(string, optional, tag = "1")]
    pub component: Option<String>,
}

/// Response containing system configuration.
#[derive(Clone, PartialEq, Message)]
pub struct GetConfigResponse {
    /// Configuration key-value pairs.
    #[prost(map = "string, string", tag = "1")]
    pub config: HashMap<String, String>,
}

/// Request to update system configuration.
#[derive(Clone, PartialEq, Message)]
pub struct UpdateConfigRequest {
    /// Component name.
    #[prost(string, tag = "1")]
    pub component: String,
    /// Configuration key.
    #[prost(string, tag = "2")]
    pub key: String,
    /// New configuration value.
    #[prost(string, tag = "3")]
    pub value: String,
}

/// Response confirming configuration update.
#[derive(Clone, PartialEq, Message)]
pub struct UpdateConfigResponse {
    /// Whether the update was successful.
    #[prost(bool, tag = "1")]
    pub success: bool,
    /// Previous value if it existed.
    #[prost(string, optional, tag = "2")]
    pub previous_value: Option<String>,
}

/// Request to get system metrics.
#[derive(Clone, PartialEq, Message)]
pub struct GetMetricsRequest {
    /// Optional component name filter (empty = all).
    #[prost(string, optional, tag = "1")]
    pub component: Option<String>,
}

/// Response containing system metrics.
#[derive(Clone, PartialEq, Message)]
pub struct GetMetricsResponse {
    /// Metric name-value pairs.
    #[prost(map = "string, double", tag = "1")]
    pub metrics: HashMap<String, f64>,
    /// Collection timestamp.
    #[prost(message, optional, tag = "2")]
    pub collected_at: Option<prost_types::Timestamp>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_priority_from_i32() {
        assert_eq!(MessagePriority::from_i32(0), Some(MessagePriority::Normal));
        assert_eq!(MessagePriority::from_i32(1), Some(MessagePriority::Low));
        assert_eq!(MessagePriority::from_i32(2), Some(MessagePriority::High));
        assert_eq!(
            MessagePriority::from_i32(3),
            Some(MessagePriority::Critical)
        );
        assert_eq!(MessagePriority::from_i32(99), None);
    }

    #[test]
    fn agent_availability_from_i32() {
        assert_eq!(
            AgentAvailability::from_i32(0),
            Some(AgentAvailability::Idle)
        );
        assert_eq!(
            AgentAvailability::from_i32(1),
            Some(AgentAvailability::Busy)
        );
        assert_eq!(
            AgentAvailability::from_i32(2),
            Some(AgentAvailability::Offline)
        );
        assert_eq!(AgentAvailability::from_i32(42), None);
    }

    #[test]
    fn task_status_from_i32() {
        assert_eq!(TaskStatus::from_i32(0), Some(TaskStatus::Success));
        assert_eq!(TaskStatus::from_i32(1), Some(TaskStatus::Failure));
        assert_eq!(TaskStatus::from_i32(2), Some(TaskStatus::Partial));
        assert_eq!(TaskStatus::from_i32(-1), None);
    }

    #[test]
    fn severity_from_i32() {
        assert_eq!(Severity::from_i32(0), Some(Severity::Info));
        assert_eq!(Severity::from_i32(1), Some(Severity::Warning));
        assert_eq!(Severity::from_i32(2), Some(Severity::Error));
        assert_eq!(Severity::from_i32(3), Some(Severity::Critical));
        assert_eq!(Severity::from_i32(4), None);
    }

    #[test]
    fn message_envelope_roundtrip() {
        let envelope = MessageEnvelope {
            message_id: "test-id".to_string(),
            timestamp: None,
            schema_version: "1.0.0".to_string(),
            message_type: "test".to_string(),
            correlation_id: Some("corr-1".to_string()),
            trace_id: None,
            source_agent_id: Some("agent-1".to_string()),
            target_agent_id: None,
            priority: MessagePriority::High as i32,
            payload: vec![1, 2, 3],
            headers: HashMap::from([("key".to_string(), "value".to_string())]),
        };

        let encoded = envelope.encode_to_vec();
        let decoded = MessageEnvelope::decode(encoded.as_slice()).unwrap();
        assert_eq!(envelope, decoded);
    }

    #[test]
    fn agent_info_roundtrip() {
        let info = AgentInfo {
            agent_id: "agent-1".to_string(),
            agent_type: "worker".to_string(),
            availability: AgentAvailability::Idle as i32,
            active_tasks: 3,
            started_at: None,
            load: 0.42,
        };

        let encoded = info.encode_to_vec();
        let decoded = AgentInfo::decode(encoded.as_slice()).unwrap();
        assert_eq!(info, decoded);
    }

    #[test]
    fn task_result_roundtrip() {
        let result = TaskResult {
            task_id: "task-1".to_string(),
            status: TaskStatus::Success as i32,
            result: None,
            error: None,
            duration_ms: 150,
            agent_id: "agent-1".to_string(),
        };

        let encoded = result.encode_to_vec();
        let decoded = TaskResult::decode(encoded.as_slice()).unwrap();
        assert_eq!(result, decoded);
    }

    #[test]
    fn system_event_roundtrip() {
        let event = SystemEvent {
            event_type: "agent.started".to_string(),
            source: "supervisor".to_string(),
            severity: Severity::Info as i32,
            message: "Agent started successfully".to_string(),
            data: None,
            timestamp: None,
        };

        let encoded = event.encode_to_vec();
        let decoded = SystemEvent::decode(encoded.as_slice()).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn default_enums() {
        assert_eq!(MessagePriority::default(), MessagePriority::Normal);
        assert_eq!(AgentAvailability::default(), AgentAvailability::Idle);
        assert_eq!(TaskStatus::default(), TaskStatus::Success);
        assert_eq!(Severity::default(), Severity::Info);
    }
}

//! Typed message structs for framework communication.
//!
//! All 10 message types from the Phase 4 data model. Each type derives
//! `Serialize` and `Deserialize` for MessagePack and JSON transport.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::priority::MessagePriority;

/// Task completion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task completed successfully.
    Success,
    /// Task failed.
    Failure,
    /// Task partially completed.
    Partial,
}

/// Event severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational event.
    Info,
    /// Warning condition.
    Warning,
    /// Error condition.
    Error,
    /// Critical system failure.
    Critical,
}

/// Workflow execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is actively running.
    Active,
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed.
    Failed,
}

// ---- Message Types ----

/// Assignment of a task to an agent for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Unique task identifier.
    pub task_id: Uuid,
    /// Task category for routing.
    pub task_type: String,
    /// Task-specific parameters.
    pub payload: serde_json::Value,
    /// Execution priority.
    pub priority: MessagePriority,
    /// Optional task deadline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
    /// Specific agent target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_agent: Option<Uuid>,
    /// Requesting agent.
    pub requester_id: Uuid,
    /// Arbitrary key-value metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Result of a completed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Matching task ID from `TaskAssignment`.
    pub task_id: Uuid,
    /// Completion status.
    pub status: TaskStatus,
    /// Task output data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error description if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Execution time in milliseconds.
    pub duration_ms: u64,
    /// Agent that processed the task.
    pub agent_id: Uuid,
}

/// Periodic agent heartbeat with availability and load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    /// Heartbeat source agent.
    pub agent_id: Uuid,
    /// Current transport availability status.
    pub availability: mister_smith_core::AgentAvailability,
    /// Current load factor (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<f64>,
    /// Number of in-progress tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_tasks: Option<u32>,
    /// Agent uptime in seconds.
    pub uptime_secs: u64,
}

/// System-wide event notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Event category.
    pub event_type: String,
    /// Component that generated the event.
    pub source: String,
    /// Event severity.
    pub severity: Severity,
    /// Human-readable description.
    pub message: String,
    /// Structured event data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Initiation of a multi-step workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStart {
    /// Workflow identifier.
    pub workflow_id: Uuid,
    /// Current workflow status.
    pub status: WorkflowStatus,
    /// Remaining steps in the workflow.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
    /// Initial output/parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
}

/// Completion of a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepComplete {
    /// Workflow identifier.
    pub workflow_id: Uuid,
    /// Completed step identifier.
    pub step_id: String,
    /// Step completion status.
    pub status: WorkflowStatus,
    /// Step output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
}

/// Final result of a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Workflow identifier.
    pub workflow_id: Uuid,
    /// Final workflow status.
    pub status: WorkflowStatus,
    /// Workflow output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
}

/// Command to spawn a new agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpawn {
    /// Agent identifier for the new agent.
    pub agent_id: Uuid,
    /// Type of agent to spawn.
    pub agent_type: String,
    /// Agent configuration.
    pub config: serde_json::Value,
}

/// Command to terminate an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTerminate {
    /// Agent to terminate.
    pub agent_id: Uuid,
    /// Reason for termination.
    pub reason: String,
    /// Whether to attempt graceful shutdown.
    pub graceful: bool,
}

/// Configuration update notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdate {
    /// Component being configured.
    pub component: String,
    /// Configuration key.
    pub key: String,
    /// New configuration value.
    pub value: String,
    /// Previous value (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{from_msgpack, to_msgpack};

    #[test]
    fn task_assignment_roundtrip() {
        let msg = TaskAssignment {
            task_id: Uuid::new_v4(),
            task_type: "code-review".into(),
            payload: serde_json::json!({"file": "main.rs"}),
            priority: MessagePriority::High,
            deadline: Some(Utc::now()),
            assigned_agent: None,
            requester_id: Uuid::new_v4(),
            metadata: HashMap::from([("env".into(), "prod".into())]),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: TaskAssignment = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.task_id, decoded.task_id);
        assert_eq!(msg.task_type, decoded.task_type);
        assert_eq!(msg.priority, decoded.priority);
    }

    #[test]
    fn task_result_roundtrip() {
        let msg = TaskResult {
            task_id: Uuid::new_v4(),
            status: TaskStatus::Success,
            result: Some(serde_json::json!({"output": "done"})),
            error: None,
            duration_ms: 1234,
            agent_id: Uuid::new_v4(),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: TaskResult = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.task_id, decoded.task_id);
        assert_eq!(msg.status, decoded.status);
        assert_eq!(msg.duration_ms, decoded.duration_ms);
    }

    #[test]
    fn agent_heartbeat_roundtrip() {
        let msg = AgentHeartbeat {
            agent_id: Uuid::new_v4(),
            availability: mister_smith_core::AgentAvailability::Idle,
            load: Some(0.42),
            active_tasks: Some(3),
            uptime_secs: 3600,
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: AgentHeartbeat = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.agent_id, decoded.agent_id);
        assert_eq!(msg.availability, decoded.availability);
        assert_eq!(msg.load, decoded.load);
    }

    #[test]
    fn system_event_roundtrip() {
        let msg = SystemEvent {
            event_type: "config_changed".into(),
            source: "config-service".into(),
            severity: Severity::Warning,
            message: "Config reloaded".into(),
            data: Some(serde_json::json!({"key": "timeout"})),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: SystemEvent = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.event_type, decoded.event_type);
        assert_eq!(msg.severity, decoded.severity);
    }

    #[test]
    fn workflow_start_roundtrip() {
        let msg = WorkflowStart {
            workflow_id: Uuid::new_v4(),
            status: WorkflowStatus::Active,
            next_steps: vec!["step-1".into(), "step-2".into()],
            output: None,
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: WorkflowStart = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.workflow_id, decoded.workflow_id);
        assert_eq!(msg.next_steps, decoded.next_steps);
    }

    #[test]
    fn step_complete_roundtrip() {
        let msg = StepComplete {
            workflow_id: Uuid::new_v4(),
            step_id: "step-1".into(),
            status: WorkflowStatus::Completed,
            output: Some(serde_json::json!({"result": 42})),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: StepComplete = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.step_id, decoded.step_id);
        assert_eq!(msg.status, decoded.status);
    }

    #[test]
    fn workflow_result_roundtrip() {
        let msg = WorkflowResult {
            workflow_id: Uuid::new_v4(),
            status: WorkflowStatus::Failed,
            output: Some(serde_json::json!({"error": "timeout"})),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: WorkflowResult = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.status, decoded.status);
    }

    #[test]
    fn agent_spawn_roundtrip() {
        let msg = AgentSpawn {
            agent_id: Uuid::new_v4(),
            agent_type: "worker".into(),
            config: serde_json::json!({"threads": 4}),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: AgentSpawn = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.agent_id, decoded.agent_id);
        assert_eq!(msg.agent_type, decoded.agent_type);
    }

    #[test]
    fn agent_terminate_roundtrip() {
        let msg = AgentTerminate {
            agent_id: Uuid::new_v4(),
            reason: "user requested".into(),
            graceful: true,
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: AgentTerminate = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.agent_id, decoded.agent_id);
        assert_eq!(msg.graceful, decoded.graceful);
    }

    #[test]
    fn config_update_roundtrip() {
        let msg = ConfigUpdate {
            component: "nats".into(),
            key: "max_reconnects".into(),
            value: "10".into(),
            previous_value: Some("5".into()),
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: ConfigUpdate = from_msgpack(&bytes).unwrap();
        assert_eq!(msg.component, decoded.component);
        assert_eq!(msg.key, decoded.key);
        assert_eq!(msg.previous_value, decoded.previous_value);
    }
}

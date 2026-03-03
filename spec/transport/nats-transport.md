---
title: NATS Transport Specifications
type: transport-protocol
category: messaging
status: active
priority: high
---

## NATS Transport Specifications

> **Source**: Extracted from `transport-layer-specifications.md` (Sections 2, 12)
> **Canonical Reference**: See `/tech-framework.md` for authoritative technology stack specifications

## Technical Overview

This document defines the complete NATS messaging specifications for the Mister Smith AI Agent Framework, focusing on distributed messaging patterns, subject hierarchies, and JetStream persistence configurations.

## Technology Stack

```rust
// Core Dependencies
async-nats = "0.34"
tokio = { version = "1.38", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
```

## Transport Integration Points

### Core Transport Layer

- **Connection Management**: `../transport-core.md#connection-pooling`
- **Message Routing**: `../transport-core.md#routing-patterns`
- **Health Monitoring**: `../transport-core.md#health-checks`
- **Error Handling**: `../transport-core.md#error-strategies`

### Security Integration

- **Authentication**: `../security/authentication.md#nats-auth`
- **Authorization**: `../security/authorization.md#subject-permissions`
- **TLS Configuration**: `../security/tls-configuration.md#nats-tls`

### Data Management Integration

- **Message Schemas**: `../data-management/message-schemas.md#nats-formats`
- **Persistence Patterns**: `../data-management/data-persistence.md#jetstream-storage`
- **Agent Communication**: `../data-management/agent-communication.md#nats-messaging`

## NATS Messaging Foundations

### Subject Hierarchy Structure

```rust
// Primary Subject Patterns
const SUBJECT_PATTERNS: &[&str] = &[
    // Agent Communication
    "agents.{agent_id}.commands.{command_type}",
    "agents.{agent_id}.status.{status_type}",
    "agents.{agent_id}.heartbeat",
    "agents.{agent_id}.capabilities",
    "agents.{agent_id}.metrics.{metric_type}",
    "agents.{agent_id}.events.{event_type}",
    
    // Task Distribution
    "tasks.{task_type}.assignment",
    "tasks.{task_type}.queue.{priority}",
    "tasks.{task_id}.progress",
    "tasks.{task_id}.result",
    "tasks.{task_id}.error",
    
    // System Management
    "system.control.{operation}",
    "system.discovery.{service_type}",
    "system.health.{component}",
    "system.config.{component}.{action}",
    "system.alerts.{severity}",
    
    // Claude CLI Integration
    "cli.startup",
    "cli.hooks.{hook_type}.{agent_id}",
    "cli.responses.{agent_id}",
    "cli.context.{group_id}.{context_type}",
    
    // Workflow Orchestration
    "workflow.{workflow_id}.orchestration",
    "workflow.{workflow_id}.coordination",
    "workflow.{workflow_id}.dependencies",
];
```

### Wildcard Subscription Patterns

```rust
// Monitoring Patterns
const WILDCARD_SUBSCRIPTIONS: &[&str] = &[
    "agents.*.status",           // All agent status updates
    "agents.*.heartbeat",        // All agent heartbeats
    "tasks.>"                    // All task-related subjects
    "system.alerts.>",           // All system alerts
    "cli.hooks.>"                // All CLI hook events
];

// Discovery Patterns
const DISCOVERY_PATTERNS: &[&str] = &[
    "system.discovery.*",        // Service discovery
    "agents.*.capabilities",     // Agent capability announcements
    "workflow.*.coordination",   // Workflow coordination
];
```

### Subject Naming Conventions

```rust
// Agent ID Format: "agent-{uuid-v4}"
const AGENT_ID_PATTERN: &str = r"^agent-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";

// Task ID Format: "task-{uuid-v4}"
const TASK_ID_PATTERN: &str = r"^task-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";

// Workflow ID Format: "workflow-{uuid-v4}"
const WORKFLOW_ID_PATTERN: &str = r"^workflow-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
```

### Message Format Specifications

#### Claude CLI Hook Integration

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEventMessage {
    pub hook_type: HookType,
    pub agent_id: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<serde_json::Value>,
    pub session_info: SessionInfo,
    pub timestamp: DateTime<Utc>,
    pub context_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookType {
    Startup,
    PreTask,
    PostTask,
    OnError,
    OnFileChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub agent_id: String,
    pub session_id: Uuid,
    pub model: String,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResponse {
    pub decision: HookDecision,
    pub reason: Option<String>,
    pub continue_processing: bool,
    pub stop_reason: Option<String>,
    pub modifications: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookDecision {
    Approve,
    Block,
    Continue,
}
```

#### Task Communication Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutputMessage {
    pub task_info: TaskInfo,
    pub output_line: String,
    pub task_status: TaskStatus,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub info_type: TaskInfoType,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskInfoType {
    AgentId,
    Description,
    Progress,
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}
```

### NATS Performance Characteristics

#### Core NATS Performance

```rust
// Core NATS Performance Metrics
const CORE_NATS_METRICS: NatsMetrics = NatsMetrics {
    throughput_msgs_per_sec: 3_000_000,
    latency_p50_microseconds: 50,
    latency_p95_microseconds: 150,
    latency_p99_microseconds: 500,
    delivery_guarantee: DeliveryGuarantee::AtMostOnce,
    connection_overhead_bytes: 1024,
    message_overhead_bytes: 22,
};

// JetStream Performance Metrics
const JETSTREAM_METRICS: NatsMetrics = NatsMetrics {
    throughput_msgs_per_sec: 200_000,
    latency_p50_microseconds: 1_000,
    latency_p95_microseconds: 5_000,
    latency_p99_microseconds: 15_000,
    delivery_guarantee: DeliveryGuarantee::AtLeastOnce,
    connection_overhead_bytes: 2048,
    message_overhead_bytes: 64,
};

#[derive(Debug, Clone)]
pub struct NatsMetrics {
    pub throughput_msgs_per_sec: u32,
    pub latency_p50_microseconds: u32,
    pub latency_p95_microseconds: u32,
    pub latency_p99_microseconds: u32,
    pub delivery_guarantee: DeliveryGuarantee,
    pub connection_overhead_bytes: u32,
    pub message_overhead_bytes: u32,
}

#[derive(Debug, Clone)]
pub enum DeliveryGuarantee {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}
```

#### Performance Optimization Patterns

```rust
// Connection Pool Configuration
const NATS_CONNECTION_CONFIG: NatsConnectionConfig = NatsConnectionConfig {
    pool_size: 10,
    max_reconnect_attempts: 5,
    reconnect_delay_ms: 1000,
    ping_interval_ms: 30000,
    max_pending_bytes: 64 * 1024 * 1024, // 64MB
    max_pending_messages: 65536,
    flush_timeout_ms: 5000,
};

// Subscription Configuration
const SUBSCRIPTION_CONFIG: SubscriptionConfig = SubscriptionConfig {
    queue_group: Some("agent-workers"),
    max_pending_messages: 1000,
    max_pending_bytes: 1024 * 1024, // 1MB
    ack_wait_ms: 30000,
    max_deliver: 3,
};
```

### JetStream Persistence Configuration

#### Stream Configuration Specifications

```rust
use async_nats::jetstream::stream::{Config as StreamConfig, RetentionPolicy, StorageType};
use async_nats::jetstream::consumer::{Config as ConsumerConfig, AckPolicy, DeliverPolicy};
use std::time::Duration;

// Agent Events Stream
const AGENT_EVENTS_STREAM: StreamConfig = StreamConfig {
    name: "agent-events".to_string(),
    subjects: vec!["agents.*.events.>".to_string(), "agents.*.status".to_string()],
    storage: StorageType::File,
    retention: RetentionPolicy::Limits,
    max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
    max_bytes: Some(1024 * 1024 * 1024), // 1GB
    max_messages: Some(1_000_000),
    max_message_size: Some(1024 * 1024), // 1MB
    discard: async_nats::jetstream::stream::DiscardPolicy::Old,
    num_replicas: 3,
    duplicate_window: Some(Duration::from_secs(120)), // 2 minutes
    ..Default::default()
};

// Task Lifecycle Stream
const TASK_LIFECYCLE_STREAM: StreamConfig = StreamConfig {
    name: "task-lifecycle".to_string(),
    subjects: vec![
        "tasks.*.assignment".to_string(),
        "tasks.*.result".to_string(),
        "tasks.*.progress".to_string(),
        "tasks.*.error".to_string(),
    ],
    storage: StorageType::File,
    retention: RetentionPolicy::Limits,
    max_age: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
    max_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
    max_messages: Some(10_000_000),
    max_message_size: Some(10 * 1024 * 1024), // 10MB
    discard: async_nats::jetstream::stream::DiscardPolicy::Old,
    num_replicas: 3,
    duplicate_window: Some(Duration::from_secs(300)), // 5 minutes
    ..Default::default()
};
```

#### Consumer Configuration Patterns

```rust
// Event Processing Consumer
const EVENT_PROCESSOR_CONSUMER: ConsumerConfig = ConsumerConfig {
    name: Some("event-processor".to_string()),
    deliver_policy: DeliverPolicy::New,
    ack_policy: AckPolicy::Explicit,
    ack_wait: Duration::from_secs(30),
    max_deliver: Some(3),
    filter_subject: Some("agents.*.events.>".to_string()),
    replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
    max_ack_pending: Some(1000),
    ..Default::default()
};

// Task Assignment Consumer
const TASK_ASSIGNMENT_CONSUMER: ConsumerConfig = ConsumerConfig {
    name: Some("task-assignment".to_string()),
    deliver_policy: DeliverPolicy::All,
    ack_policy: AckPolicy::Explicit,
    ack_wait: Duration::from_secs(300), // 5 minutes
    max_deliver: Some(5),
    filter_subject: Some("tasks.*.assignment".to_string()),
    replay_policy: async_nats::jetstream::consumer::ReplayPolicy::Instant,
    max_ack_pending: Some(100),
    ..Default::default()
};
```

#### Resource Management

```rust
// JetStream Resource Limits
const JETSTREAM_LIMITS: JetStreamLimits = JetStreamLimits {
    max_memory: 2 * 1024 * 1024 * 1024, // 2GB
    max_storage: 100 * 1024 * 1024 * 1024, // 100GB
    max_streams: 50,
    max_consumers: 500,
    max_connections: 10_000,
    max_subscriptions: 100_000,
};

#[derive(Debug, Clone)]
pub struct JetStreamLimits {
    pub max_memory: u64,
    pub max_storage: u64,
    pub max_streams: u32,
    pub max_consumers: u32,
    pub max_connections: u32,
    pub max_subscriptions: u32,
}
```

### Message Flow Patterns

#### Agent Communication Flow

```rust
use async_nats::{Client, Message};
use tokio::time::{Duration, timeout};

// 1. Direct Agent Command Pattern
pub async fn send_agent_command(
    client: &Client,
    agent_id: &str,
    command: AgentCommand,
) -> Result<(), NatsError> {
    let subject = format!("agents.{}.commands.{}", agent_id, command.command_type);
    let payload = serde_json::to_vec(&command)?;
    
    client.publish(subject, payload.into()).await?;
    Ok(())
}

// 2. Request-Response Pattern
pub async fn query_agent_capabilities(
    client: &Client,
    agent_id: &str,
) -> Result<AgentCapabilities, NatsError> {
    let subject = format!("agents.{}.capabilities", agent_id);
    let query = CapabilityQuery::new();
    let payload = serde_json::to_vec(&query)?;
    
    let response = timeout(
        Duration::from_secs(5),
        client.request(subject, payload.into())
    ).await??;
    
    let capabilities: AgentCapabilities = serde_json::from_slice(&response.payload)?;
    Ok(capabilities)
}

// 3. Publish-Subscribe Pattern
pub async fn subscribe_to_agent_events(
    client: &Client,
    handler: impl Fn(Message) -> Result<(), NatsError> + Send + 'static,
) -> Result<(), NatsError> {
    let mut subscription = client.subscribe("agents.*.events.>").await?;
    
    tokio::spawn(async move {
        while let Some(message) = subscription.next().await {
            if let Err(e) = handler(message) {
                eprintln!("Error handling agent event: {}", e);
            }
        }
    });
    
    Ok(())
}
```

#### Message Schema Definitions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCommand {
    pub command_id: Uuid,
    pub command_type: CommandType,
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub priority: MessagePriority,
    pub timeout_ms: Option<u64>,
    pub reply_to: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    Execute,
    Stop,
    Pause,
    Resume,
    Configure,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub status: AgentState,
    pub timestamp: DateTime<Utc>,
    pub capacity: f64, // 0.0 to 1.0
    pub current_tasks: u32,
    pub max_tasks: u32,
    pub uptime_seconds: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Busy,
    Error,
    Offline,
    Starting,
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub assigned_agent: String,
    pub timestamp: DateTime<Utc>,
    pub priority: MessagePriority,
    pub deadline: Option<DateTime<Utc>>,
    pub requirements: TaskRequirements,
    pub task_data: serde_json::Value,
    pub dependencies: Vec<Uuid>,
    pub callback_subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Analysis,
    Synthesis,
    Execution,
    Validation,
    Monitoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Bulk = 4,
}
```

## NATS Implementation Specifications

### Subject Taxonomy Implementation

```rust
use std::collections::HashMap;
use lazy_static::lazy_static;

// Subject Template Registry
lazy_static! {
    pub static ref SUBJECT_TEMPLATES: HashMap<&'static str, Vec<&'static str>> = {
        let mut templates = HashMap::new();
        
        // Agent Communication Subjects
        templates.insert("agent", vec![
            "agents.{agent_id}.commands.{command_type}",
            "agents.{agent_id}.status.{status_type}",
            "agents.{agent_id}.heartbeat",
            "agents.{agent_id}.capabilities",
            "agents.{agent_id}.metrics.{metric_type}",
            "agents.{agent_id}.logs.{level}",
            "agents.{agent_id}.events.{event_type}",
        ]);
        
        // Task Distribution Subjects
        templates.insert("task", vec![
            "tasks.{task_type}.assignment",
            "tasks.{task_type}.queue.{priority}",
            "tasks.{task_id}.progress",
            "tasks.{task_id}.result",
            "tasks.{task_id}.error",
            "tasks.{task_id}.cancel",
        ]);
        
        // System Management Subjects
        templates.insert("system", vec![
            "system.control.{operation}",
            "system.discovery.{service_type}",
            "system.health.{component}",
            "system.config.{component}.{action}",
            "system.alerts.{severity}",
            "system.audit.{action}",
        ]);
        
        // Workflow Orchestration Subjects
        templates.insert("workflow", vec![
            "workflow.{workflow_id}.orchestration",
            "workflow.{workflow_id}.coordination",
            "workflow.{workflow_id}.dependencies",
            "workflow.{workflow_id}.rollback",
        ]);
        
        // Claude CLI Integration Subjects
        templates.insert("cli", vec![
            "cli.startup",
            "cli.hooks.{hook_type}.{agent_id}",
            "cli.responses.{agent_id}",
            "cli.mutations.{agent_id}",
            "cli.context.{group_id}.{context_type}",
        ]);
        
        templates
    };
}

// Subject Builder
pub struct SubjectBuilder;

impl SubjectBuilder {
    pub fn agent_command(agent_id: &str, command_type: &str) -> String {
        format!("agents.{}.commands.{}", agent_id, command_type)
    }
    
    pub fn agent_status(agent_id: &str, status_type: &str) -> String {
        format!("agents.{}.status.{}", agent_id, status_type)
    }
    
    pub fn task_assignment(task_type: &str) -> String {
        format!("tasks.{}.assignment", task_type)
    }
    
    pub fn task_result(task_id: &Uuid) -> String {
        format!("tasks.{}.result", task_id)
    }
    
    pub fn system_alert(severity: &str) -> String {
        format!("system.alerts.{}", severity)
    }
    
    pub fn cli_hook(hook_type: &str, agent_id: &str) -> String {
        format!("cli.hooks.{}.{}", hook_type, agent_id)
    }
    
    pub fn workflow_orchestration(workflow_id: &Uuid) -> String {
        format!("workflow.{}.orchestration", workflow_id)
    }
}
```

### Subject Validation

```rust
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SUBJECT_VALIDATORS: HashMap<&'static str, Regex> = {
        let mut validators = HashMap::new();
        
        validators.insert(
            "agent_id",
            Regex::new(r"^agent-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$").unwrap(),
        );
        
        validators.insert(
            "task_id",
            Regex::new(r"^task-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$").unwrap(),
        );
        
        validators.insert(
            "workflow_id",
            Regex::new(r"^workflow-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$").unwrap(),
        );
        
        validators.insert(
            "command_type",
            Regex::new(r"^(execute|stop|pause|resume|configure|query)$").unwrap(),
        );
        
        validators.insert(
            "status_type",
            Regex::new(r"^(idle|busy|error|offline|starting|stopping)$").unwrap(),
        );
        
        validators
    };
}

pub fn validate_subject_component(component_type: &str, value: &str) -> Result<(), SubjectValidationError> {
    match SUBJECT_VALIDATORS.get(component_type) {
        Some(regex) => {
            if regex.is_match(value) {
                Ok(())
            } else {
                Err(SubjectValidationError::InvalidFormat {
                    component: component_type.to_string(),
                    value: value.to_string(),
                })
            }
        }
        None => Err(SubjectValidationError::UnknownComponent(component_type.to_string())),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubjectValidationError {
    #[error("Invalid format for {component}: {value}")]
    InvalidFormat { component: String, value: String },
    #[error("Unknown component type: {0}")]
    UnknownComponent(String),
}
```

### Message Payload Schema Implementation

#### Schema Validation Framework

```rust
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Message validation traits
pub trait MessageValidation {
    fn validate_message(&self) -> Result<(), ValidationError>;
    fn get_schema() -> schemars::schema::RootSchema;
}

// Agent Command Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AgentCommandMessage {
    pub command_id: Uuid,
    #[validate(custom = "validate_command_type")]
    pub command_type: String,
    #[validate(regex = "AGENT_ID_REGEX")]
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    #[validate(range(min = 0, max = 4))]
    pub priority: u8,
    #[validate(range(min = 1000))]
    pub timeout_ms: Option<u64>,
    #[validate(regex = "SUBJECT_REGEX")]
    pub reply_to: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct CommandMetadata {
    pub source: String,
    pub trace_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub session_id: Option<Uuid>,
}

// Agent Status Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AgentStatusMessage {
    #[validate(regex = "AGENT_ID_REGEX")]
    pub agent_id: String,
    #[validate(custom = "validate_agent_status")]
    pub status: String,
    pub timestamp: DateTime<Utc>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub capacity: f64,
    #[validate(range(min = 0))]
    pub current_tasks: u32,
    #[validate(range(min = 1))]
    pub max_tasks: u32,
    #[validate(range(min = 0))]
    pub uptime_seconds: u64,
    pub last_heartbeat: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ResourceUsage {
    #[validate(range(min = 0.0, max = 100.0))]
    pub cpu_percent: f64,
    #[validate(range(min = 0))]
    pub memory_mb: u64,
    #[validate(range(min = 0))]
    pub disk_mb: u64,
}

// Task Assignment Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct TaskAssignmentMessage {
    pub task_id: Uuid,
    #[validate(custom = "validate_task_type")]
    pub task_type: String,
    #[validate(regex = "AGENT_ID_REGEX")]
    pub assigned_agent: String,
    pub timestamp: DateTime<Utc>,
    #[validate(range(min = 0, max = 4))]
    pub priority: u8,
    pub deadline: Option<DateTime<Utc>>,
    pub requirements: TaskRequirements,
    pub task_data: serde_json::Value,
    pub dependencies: Vec<Uuid>,
    #[validate(regex = "SUBJECT_REGEX")]
    pub callback_subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct TaskRequirements {
    pub capabilities: Vec<String>,
    pub resources: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ResourceRequirements {
    #[validate(range(min = 1))]
    pub cpu_cores: u32,
    #[validate(range(min = 256))]
    pub memory_mb: u32,
    #[validate(range(min = 100))]
    pub disk_mb: u32,
}

// Task Result Schema
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct TaskResultMessage {
    pub task_id: Uuid,
    #[validate(regex = "AGENT_ID_REGEX")]
    pub agent_id: String,
    #[validate(custom = "validate_task_status")]
    pub status: String,
    pub timestamp: DateTime<Utc>,
    #[validate(range(min = 0))]
    pub execution_time_ms: u64,
    pub result_data: serde_json::Value,
    pub error_details: Option<ErrorDetails>,
    pub metrics: Option<ExecutionMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ErrorDetails {
    pub error_code: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    #[validate(range(min = 0))]
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ExecutionMetrics {
    pub cpu_usage_percent: Option<f64>,
    pub memory_peak_mb: Option<f64>,
    pub io_operations: Option<u64>,
}
```

#### Schema Validation Implementation

```rust
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref AGENT_ID_REGEX: Regex = Regex::new(
        r"^agent-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
    ).unwrap();
    
    static ref SUBJECT_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9._-]+$"
    ).unwrap();
}

fn validate_command_type(command_type: &str) -> Result<(), ValidationError> {
    match command_type {
        "execute" | "stop" | "pause" | "resume" | "configure" | "query" => Ok(()),
        _ => Err(ValidationError::new("Invalid command type")),
    }
}

fn validate_agent_status(status: &str) -> Result<(), ValidationError> {
    match status {
        "idle" | "busy" | "error" | "offline" | "starting" | "stopping" => Ok(()),
        _ => Err(ValidationError::new("Invalid agent status")),
    }
}

fn validate_task_type(task_type: &str) -> Result<(), ValidationError> {
    match task_type {
        "analysis" | "synthesis" | "execution" | "validation" | "monitoring" => Ok(()),
        _ => Err(ValidationError::new("Invalid task type")),
    }
}

fn validate_task_status(status: &str) -> Result<(), ValidationError> {
    match status {
        "completed" | "failed" | "cancelled" | "timeout" => Ok(()),
        _ => Err(ValidationError::new("Invalid task status")),
    }
}

// Message validation implementation
impl MessageValidation for AgentCommandMessage {
    fn validate_message(&self) -> Result<(), ValidationError> {
        self.validate()
    }
    
    fn get_schema() -> schemars::schema::RootSchema {
        schema_for!(AgentCommandMessage)
    }
}

// Implement for other message types...
```

### JetStream Configuration Implementation

#### Stream Configuration Management

```rust
use async_nats::jetstream::{Config, Context};
use async_nats::jetstream::stream::{Config as StreamConfig, RetentionPolicy, StorageType, DiscardPolicy};
use async_nats::jetstream::consumer::{Config as ConsumerConfig, AckPolicy, DeliverPolicy, ReplayPolicy};
use std::time::Duration;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct JetStreamManager {
    pub context: Context,
    pub stream_configs: HashMap<String, StreamConfig>,
    pub consumer_configs: HashMap<String, ConsumerConfig>,
}

impl JetStreamManager {
    pub async fn new(client: async_nats::Client) -> Result<Self, Box<dyn std::error::Error>> {
        let context = async_nats::jetstream::new(client);
        
        let mut stream_configs = HashMap::new();
        let mut consumer_configs = HashMap::new();
        
        // Configure streams
        stream_configs.insert("agent_events".to_string(), Self::agent_events_stream());
        stream_configs.insert("task_lifecycle".to_string(), Self::task_lifecycle_stream());
        stream_configs.insert("system_monitoring".to_string(), Self::system_monitoring_stream());
        stream_configs.insert("claude_cli_integration".to_string(), Self::claude_cli_stream());
        
        // Configure consumers
        consumer_configs.insert("agent_status_monitor".to_string(), Self::agent_status_consumer());
        consumer_configs.insert("task_processor".to_string(), Self::task_processor_consumer());
        consumer_configs.insert("system_alerting".to_string(), Self::system_alerting_consumer());
        
        Ok(JetStreamManager {
            context,
            stream_configs,
            consumer_configs,
        })
    }
    
    // Agent Events Stream Configuration
    fn agent_events_stream() -> StreamConfig {
        StreamConfig {
            name: "agent_events".to_string(),
            subjects: vec![
                "agents.*.events.>".to_string(),
                "agents.*.status".to_string(),
                "agents.*.heartbeat".to_string(),
            ],
            storage: StorageType::File,
            retention: RetentionPolicy::Limits,
            max_age: Some(Duration::from_secs(7 * 24 * 60 * 60)), // 7 days
            max_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_messages: Some(1_000_000),
            max_message_size: Some(1024 * 1024), // 1MB
            discard: DiscardPolicy::Old,
            num_replicas: 3,
            duplicate_window: Some(Duration::from_secs(120)), // 2 minutes
            ..Default::default()
        }
    }
    
    // Task Lifecycle Stream Configuration
    fn task_lifecycle_stream() -> StreamConfig {
        StreamConfig {
            name: "task_lifecycle".to_string(),
            subjects: vec![
                "tasks.*.assignment".to_string(),
                "tasks.*.result".to_string(),
                "tasks.*.progress".to_string(),
                "tasks.*.error".to_string(),
            ],
            storage: StorageType::File,
            retention: RetentionPolicy::Limits,
            max_age: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days
            max_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
            max_messages: Some(10_000_000),
            max_message_size: Some(10 * 1024 * 1024), // 10MB
            discard: DiscardPolicy::Old,
            num_replicas: 3,
            duplicate_window: Some(Duration::from_secs(300)), // 5 minutes
            ..Default::default()
        }
    }
    
    // System Monitoring Stream Configuration
    fn system_monitoring_stream() -> StreamConfig {
        StreamConfig {
            name: "system_monitoring".to_string(),
            subjects: vec![
                "system.>.health".to_string(),
                "system.>.alerts".to_string(),
                "system.>.audit".to_string(),
            ],
            storage: StorageType::File,
            retention: RetentionPolicy::Limits,
            max_age: Some(Duration::from_secs(90 * 24 * 60 * 60)), // 90 days
            max_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
            max_messages: Some(5_000_000),
            max_message_size: Some(512 * 1024), // 512KB
            discard: DiscardPolicy::Old,
            num_replicas: 3,
            duplicate_window: Some(Duration::from_secs(60)), // 1 minute
            ..Default::default()
        }
    }
    
    // Claude CLI Integration Stream Configuration
    fn claude_cli_stream() -> StreamConfig {
        StreamConfig {
            name: "claude_cli_integration".to_string(),
            subjects: vec!["cli.>".to_string()],
            storage: StorageType::Memory,
            retention: RetentionPolicy::Limits,
            max_age: Some(Duration::from_secs(3600)), // 1 hour
            max_bytes: Some(256 * 1024 * 1024), // 256MB
            max_messages: Some(100_000),
            max_message_size: Some(1024 * 1024), // 1MB
            discard: DiscardPolicy::New,
            num_replicas: 1,
            duplicate_window: Some(Duration::from_secs(30)), // 30 seconds
            ..Default::default()
        }
    }
    
    // Agent Status Monitor Consumer
    fn agent_status_consumer() -> ConsumerConfig {
        ConsumerConfig {
            name: Some("agent_status_monitor".to_string()),
            deliver_policy: DeliverPolicy::New,
            ack_policy: AckPolicy::Explicit,
            ack_wait: Duration::from_secs(30),
            max_deliver: Some(3),
            filter_subject: Some("agents.*.status".to_string()),
            replay_policy: ReplayPolicy::Instant,
            max_ack_pending: Some(1000),
            ..Default::default()
        }
    }
    
    // Task Processor Consumer
    fn task_processor_consumer() -> ConsumerConfig {
        ConsumerConfig {
            name: Some("task_processor".to_string()),
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::Explicit,
            ack_wait: Duration::from_secs(300), // 5 minutes
            max_deliver: Some(5),
            filter_subject: Some("tasks.*.assignment".to_string()),
            replay_policy: ReplayPolicy::Instant,
            max_ack_pending: Some(100),
            ..Default::default()
        }
    }
    
    // System Alerting Consumer
    fn system_alerting_consumer() -> ConsumerConfig {
        ConsumerConfig {
            name: Some("system_alerting".to_string()),
            deliver_policy: DeliverPolicy::New,
            ack_policy: AckPolicy::Explicit,
            ack_wait: Duration::from_secs(60),
            max_deliver: Some(3),
            filter_subject: Some("system.*.alerts.>".to_string()),
            replay_policy: ReplayPolicy::Instant,
            max_ack_pending: Some(500),
            ..Default::default()
        }
    }
    
    // Initialize all streams and consumers
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create streams
        for (name, config) in &self.stream_configs {
            match self.context.create_stream(config.clone()).await {
                Ok(_) => println!("Created stream: {}", name),
                Err(e) => eprintln!("Failed to create stream {}: {}", name, e),
            }
        }
        
        // Create consumers
        for (name, config) in &self.consumer_configs {
            let stream_name = match name.as_str() {
                "agent_status_monitor" => "agent_events",
                "task_processor" => "task_lifecycle",
                "system_alerting" => "system_monitoring",
                _ => continue,
            };
            
            match self.context.create_consumer_on_stream(config.clone(), stream_name).await {
                Ok(_) => println!("Created consumer: {}", name),
                Err(e) => eprintln!("Failed to create consumer {}: {}", name, e),
            }
        }
        
        Ok(())
    }
}
```

#### Resource Management

```rust
#[derive(Debug, Clone)]
pub struct JetStreamResourceLimits {
    pub max_memory: u64,
    pub max_storage: u64,
    pub max_streams: u32,
    pub max_consumers: u32,
    pub max_connections: u32,
    pub max_subscriptions: u32,
}

impl Default for JetStreamResourceLimits {
    fn default() -> Self {
        JetStreamResourceLimits {
            max_memory: 2 * 1024 * 1024 * 1024,      // 2GB
            max_storage: 100 * 1024 * 1024 * 1024,   // 100GB
            max_streams: 50,
            max_consumers: 500,
            max_connections: 10_000,
            max_subscriptions: 100_000,
        }
    }
}

// Resource monitoring
pub async fn monitor_jetstream_resources(
    context: &Context,
    limits: &JetStreamResourceLimits,
) -> Result<ResourceUsageReport, Box<dyn std::error::Error>> {
    let account_info = context.account_info().await?;
    
    Ok(ResourceUsageReport {
        memory_usage: account_info.memory,
        storage_usage: account_info.storage,
        streams_count: account_info.streams,
        consumers_count: account_info.consumers,
        limits: limits.clone(),
    })
}

#[derive(Debug, Clone)]
pub struct ResourceUsageReport {
    pub memory_usage: u64,
    pub storage_usage: u64,
    pub streams_count: u32,
    pub consumers_count: u32,
    pub limits: JetStreamResourceLimits,
}

impl ResourceUsageReport {
    pub fn is_within_limits(&self) -> bool {
        self.memory_usage <= self.limits.max_memory
            && self.storage_usage <= self.limits.max_storage
            && self.streams_count <= self.limits.max_streams
            && self.consumers_count <= self.limits.max_consumers
    }
    
    pub fn memory_usage_percentage(&self) -> f64 {
        (self.memory_usage as f64 / self.limits.max_memory as f64) * 100.0
    }
    
    pub fn storage_usage_percentage(&self) -> f64 {
        (self.storage_usage as f64 / self.limits.max_storage as f64) * 100.0
    }
}
```

## Implementation Examples

### Complete NATS Client Integration

```rust
use async_nats::Client;
use tokio::time::{interval, Duration};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NatsTransportClient {
    client: Client,
    jetstream_manager: Arc<JetStreamManager>,
}

impl NatsTransportClient {
    pub async fn new(server_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = async_nats::connect(server_url).await?;
        let jetstream_manager = Arc::new(JetStreamManager::new(client.clone()).await?);
        
        // Initialize JetStream resources
        jetstream_manager.initialize().await?;
        
        Ok(NatsTransportClient {
            client,
            jetstream_manager,
        })
    }
    
    pub async fn publish_agent_command(
        &self,
        agent_id: &str,
        command: AgentCommandMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subject = SubjectBuilder::agent_command(agent_id, &command.command_type);
        let payload = serde_json::to_vec(&command)?;
        
        self.client.publish(subject, payload.into()).await?;
        Ok(())
    }
    
    pub async fn subscribe_to_agent_events<F>(
        &self,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(AgentStatusMessage) + Send + Sync + 'static,
    {
        let mut subscription = self.client.subscribe("agents.*.events.>".to_string()).await?;
        let handler = Arc::new(handler);
        
        tokio::spawn(async move {
            while let Some(message) = subscription.next().await {
                match serde_json::from_slice::<AgentStatusMessage>(&message.payload) {
                    Ok(status) => handler(status),
                    Err(e) => eprintln!("Failed to deserialize agent event: {}", e),
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn start_heartbeat_publisher(
        &self,
        agent_id: String,
        interval_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let subject = format!("agents.{}.heartbeat", agent_id);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                let heartbeat = AgentHeartbeat {
                    agent_id: agent_id.clone(),
                    timestamp: chrono::Utc::now(),
                    sequence: 0, // Increment in real implementation
                };
                
                match serde_json::to_vec(&heartbeat) {
                    Ok(payload) => {
                        if let Err(e) = client.publish(subject.clone(), payload.into()).await {
                            eprintln!("Failed to publish heartbeat: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to serialize heartbeat: {}", e),
                }
            }
        });
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentHeartbeat {
    agent_id: String,
    timestamp: DateTime<Utc>,
    sequence: u64,
}
```

### Connection Management and Health Monitoring

```rust
use async_nats::{ConnectOptions, Event};
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct NatsConnectionManager {
    connection_healthy: Arc<AtomicBool>,
    event_sender: mpsc::UnboundedSender<ConnectionEvent>,
}

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
    Reconnecting,
    Error(String),
}

impl NatsConnectionManager {
    pub async fn connect_with_monitoring(
        server_url: &str,
    ) -> Result<(Client, NatsConnectionManager), Box<dyn std::error::Error>> {
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
        let connection_healthy = Arc::new(AtomicBool::new(false));
        
        let options = ConnectOptions::new()
            .ping_interval(Duration::from_secs(30))
            .reconnect_delay_callback(|attempts| {
                std::cmp::min(Duration::from_secs(attempts), Duration::from_secs(60))
            })
            .event_callback({
                let event_sender = event_sender.clone();
                move |event| {
                    let event_msg = match event {
                        Event::Connected => {
                            ConnectionEvent::Connected
                        }
                        Event::Disconnected => {
                            ConnectionEvent::Disconnected
                        }
                        Event::ClientError(e) => {
                            ConnectionEvent::Error(e.to_string())
                        }
                        _ => return,
                    };
                    
                    let _ = event_sender.send(event_msg);
                }
            });
        
        let client = async_nats::connect_with_options(server_url, options).await?;
        
        let manager = NatsConnectionManager {
            connection_healthy: connection_healthy.clone(),
            event_sender,
        };
        
        // Start connection monitoring
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                match event {
                    ConnectionEvent::Connected => {
                        connection_healthy.store(true, Ordering::Relaxed);
                        println!("NATS connection established");
                    }
                    ConnectionEvent::Disconnected => {
                        connection_healthy.store(false, Ordering::Relaxed);
                        println!("NATS connection lost");
                    }
                    ConnectionEvent::Error(e) => {
                        connection_healthy.store(false, Ordering::Relaxed);
                        eprintln!("NATS connection error: {}", e);
                    }
                    _ => {}
                }
            }
        });
        
        Ok((client, manager))
    }
    
    pub fn is_healthy(&self) -> bool {
        self.connection_healthy.load(Ordering::Relaxed)
    }
}
```

## Framework Integration

### Transport Layer Integration

- **[Transport Core](./transport-core.md)** - Connection pooling, health monitoring, error handling
- **[gRPC Transport](./grpc-transport.md)** - RPC communication patterns
- **[HTTP Transport](./http-transport.md)** - RESTful API integration

### Security Integration

- **[Authentication](../security/authentication.md)** - NATS authentication patterns
- **[Authorization](../security/authorization.md)** - Subject-based permissions
- **[TLS Configuration](../security/tls-configuration.md)** - Secure NATS connections

### Data Management Integration

- **[Message Schemas](../data-management/message-schemas.md)** - Structured message formats
- **[Persistence Operations](../data-management/persistence-operations.md)** - JetStream storage patterns
- **[Agent Communication](../data-management/agent-communication.md)** - Multi-agent messaging

### Core Architecture Integration

- **[Async Patterns](../core-architecture/async-patterns.md)** - Tokio integration
- **[Supervision Trees](../core-architecture/supervision-trees.md)** - Fault tolerance
- **[Component Architecture](../core-architecture/component-architecture.md)** - Modular design

## Implementation Guidelines

### When to Use NATS

- **High-throughput messaging**: 3M+ messages/sec with core NATS
- **Distributed agent communication**: Pub/sub patterns with subject hierarchies
- **Event-driven architectures**: Real-time agent coordination
- **Microservices communication**: Lightweight, cloud-native messaging
- **Claude CLI integration**: Hook system integration

### When to Use Alternatives

- **gRPC**: Typed RPC calls, streaming between services
- **HTTP**: RESTful APIs, WebSocket real-time communication
- **Direct TCP**: Ultra-low latency, custom protocols

### Performance Considerations

- **Core NATS**: Fire-and-forget, maximum throughput
- **JetStream**: At-least-once delivery, ~200k msgs/sec
- **Connection pooling**: Shared connections across components
- **Subject optimization**: Efficient routing with proper hierarchies

---

*NATS Transport Specifications - Technical Implementation Guide*

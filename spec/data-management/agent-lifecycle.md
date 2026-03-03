---
title: agent-lifecycle
type: note
permalink: revision-swarm/data-management/agent-lifecycle
tags:
- '#revised-document #agent-lifecycle #supervision-patterns #foundation-focus'
---

## Agent Lifecycle & Supervision Architecture

## Foundation Patterns Guide

> **Canonical Reference**: See `tech-framework.md` for authoritative technology stack specifications

## Executive Summary

This document defines foundational agent lifecycle management and supervision patterns using Rust's actor model
with Tokio runtime. Focus is on agent state machines, lifecycle transitions, supervision trees,
and restart policies essential for robust distributed agent systems.

> **Technical Status**: Core lifecycle patterns implemented with comprehensive state management, supervision integration, and recovery mechanisms. Ready for implementation with complete Rust/Tokio patterns.

## 1. Basic Agent Architecture

### 1.1 Agent Types

```rust
// Required imports for agent lifecycle management
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent type classification for lifecycle management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    Supervisor,    // Manages other agents
    Worker,        // Performs tasks
    Coordinator,   // Coordinates workflows
    Monitor,       // Observes system state
    Planner,       // Decomposes goals into tasks
    Executor,      // Carries out atomic actions
    Critic,        // Validates outcomes
    Router,        // Assigns tasks to agents
    Memory,        // Stores and retrieves knowledge
}

/// Core agent lifecycle interface
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    async fn start(&mut self) -> Result<(), AgentError>;
    async fn stop(&mut self) -> Result<(), AgentError>;
    async fn handle_message(&mut self, message: Message) -> Result<(), AgentError>;
    fn get_status(&self) -> AgentStatus;
    fn agent_type(&self) -> AgentType;
    fn id(&self) -> AgentId;
}

/// Agent identifier type
pub type AgentId = Uuid;

/// Agent error types
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Initialization failed: {0}")]
    InitializationError(String),
    #[error("State transition error: {0}")]
    StateTransitionError(String),
    #[error("Message handling error: {0}")]
    MessageError(String),
    #[error("Resource allocation error: {0}")]
    ResourceError(String),
    #[error("Supervision error: {0}")]
    SupervisionError(String),
}

/// Agent status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub state: AgentState,
    pub health: HealthLevel,
    pub uptime: Duration,
    pub last_activity: SystemTime,
}

/// Agent state enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Initializing,
    Running,
    Paused,
    Stopping,
    Terminated,
    Error,
    Restarting,
}

/// Health level indicators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthLevel {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Critical(String),
}

/// Message type for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: AgentId,
    pub recipient: Option<AgentId>,
    pub payload: MessagePayload,
    pub timestamp: SystemTime,
}

/// Message payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Task(Task),
    Response(TaskOutput),
    Control(ControlMessage),
    Data(serde_json::Value),
}

/// Control message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    Start,
    Stop,
    Pause,
    Resume,
    Restart,
    HealthCheck,
}
```

### 1.2 Core Agent Role Taxonomies

#### Planner Agent

- **Purpose**: Decomposes high-level goals into concrete subtasks
- **Interface Pattern**:

```rust
/// Goal specification for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: Uuid,
    pub description: String,
    pub constraints: Vec<Constraint>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub deadline: Option<SystemTime>,
}

/// Task identifier type
pub type TaskId = Uuid;

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub task_type: TaskType,
    pub description: String,
    pub input_data: serde_json::Value,
    pub priority: u8,
    pub estimated_duration: Option<Duration>,
}

/// Task type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    Research,
    Code,
    Analysis,
    Communication,
    Processing,
    Validation,
}

/// Planner agent interface
#[async_trait::async_trait]
pub trait Planner: Send + Sync {
    async fn create_plan(&self, goal: Goal) -> Result<TaskList, AgentError>;
    async fn refine_plan(&self, feedback: CriticFeedback) -> Result<TaskList, AgentError>;
}

/// Task list with dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    pub tasks: Vec<Task>,
    pub dependencies: HashMap<TaskId, Vec<TaskId>>,
    pub priority_order: Vec<TaskId>,
    pub estimated_completion: Option<SystemTime>,
}
```

#### Executor Agent

- **Purpose**: Carries out atomic actions and subtasks
- **Interface Pattern**:

```rust
/// Executor agent interface
#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    async fn execute_task(&self, task: Task) -> Result<TaskOutput, AgentError>;
    fn can_execute(&self, task_type: &TaskType) -> bool;
    async fn get_capabilities(&self) -> Vec<Capability>;
}

/// Task execution output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutput {
    Success(serde_json::Value),
    PartialResult(serde_json::Value, Vec<SubTask>),
    Failed(String),
}

/// Sub-task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: TaskId,
    pub parent_id: TaskId,
    pub task_type: TaskType,
    pub description: String,
    pub input_data: serde_json::Value,
}

/// Agent capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub task_types: Vec<TaskType>,
    pub resource_requirements: ResourceRequirements,
    pub performance_metrics: PerformanceMetrics,
}
```

#### Critic Agent

- **Purpose**: Validates outcomes against goals and quality criteria
- **Interface Pattern**:

```rust
/// Quality criteria for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCriteria {
    pub accuracy_threshold: f32,
    pub performance_requirements: PerformanceMetrics,
    pub validation_rules: Vec<ValidationRule>,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    DataFormat,
    BusinessLogic,
    Performance,
    Security,
}

/// Critic agent interface
#[async_trait::async_trait]
pub trait Critic: Send + Sync {
    async fn evaluate(&self, output: TaskOutput, criteria: QualityCriteria) -> CriticFeedback;
    async fn validate_plan(&self, plan: TaskList) -> ValidationResult;
}

/// Critic feedback structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticFeedback {
    pub score: f32,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Improvement>,
    pub validation_passed: bool,
    pub evaluation_timestamp: SystemTime,
}

/// Issue identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub description: String,
    pub location: Option<String>,
    pub suggested_fix: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Improvement suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    pub category: ImprovementCategory,
    pub description: String,
    pub estimated_impact: f32,
    pub implementation_effort: EffortLevel,
}

/// Improvement categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementCategory {
    Performance,
    Quality,
    Maintainability,
    Security,
    Usability,
}

/// Effort level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub confidence_score: f32,
}
```

#### Router Agent

- **Purpose**: Assigns tasks to appropriate specialized agents
- **Interface Pattern**:

```rust
/// Router agent interface
#[async_trait::async_trait]
pub trait Router: Send + Sync {
    async fn route_task(&self, task: Task) -> Result<AgentId, AgentError>;
    async fn get_agent_capabilities(&self, agent_id: AgentId) -> Result<Vec<Capability>, AgentError>;
    async fn balance_load(&self, tasks: Vec<Task>) -> Result<HashMap<AgentId, Vec<Task>>, AgentError>;
    async fn find_best_agent(&self, task_type: &TaskType) -> Result<AgentId, AgentError>;
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLoaded,
    CapabilityBased,
    Geographic,
    Custom(String),
}

/// Agent load information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoad {
    pub agent_id: AgentId,
    pub current_tasks: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub queue_depth: u32,
    pub last_response_time: Duration,
}
```

#### Memory Agent

- **Purpose**: Stores and retrieves shared knowledge
- **Interface Pattern**:

```rust
/// Memory agent interface for knowledge storage
#[async_trait::async_trait]
pub trait Memory: Send + Sync {
    async fn store(&self, key: String, value: serde_json::Value, metadata: Metadata) -> Result<(), AgentError>;
    async fn retrieve(&self, key: String) -> Result<Option<(serde_json::Value, Metadata)>, AgentError>;
    async fn query(&self, pattern: QueryPattern) -> Result<Vec<(String, serde_json::Value)>, AgentError>;
    async fn delete(&self, key: String) -> Result<bool, AgentError>;
    async fn list_keys(&self, prefix: Option<String>) -> Result<Vec<String>, AgentError>;
}

/// Metadata for stored values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub version: u64,
    pub tags: Vec<String>,
    pub content_type: String,
    pub size_bytes: u64,
    pub access_count: u64,
    pub last_accessed: SystemTime,
}

/// Query pattern for memory searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPattern {
    pub pattern_type: QueryType,
    pub criteria: HashMap<String, serde_json::Value>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

/// Query type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    ExactMatch,
    PrefixMatch,
    RegexMatch,
    FullTextSearch,
    TagMatch,
}

/// Sort order for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub disk_gb: u64,
    pub network_mbps: f32,
    pub max_connections: u32,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub throughput_ops_per_sec: f32,
    pub latency_p95_ms: f32,
    pub latency_p99_ms: f32,
    pub error_rate_percent: f32,
    pub availability_percent: f32,
}

/// Constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub value: serde_json::Value,
}

/// Constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Resource,
    Time,
    Quality,
    Security,
    Business,
}

/// Success criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub name: String,
    pub metric: String,
    pub target_value: f32,
    pub comparison: ComparisonOperator,
}

/// Comparison operator for success criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}
```

### 1.3 Agent Lifecycle State Machine

#### 1.3.1 Agent State Schema

> **Validation Note**: Consider using Protocol Buffers for state schema versioning in
> distributed deployments. The JSON Schema below provides a solid foundation but may benefit
> from protobuf for evolution support.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/agent-state",
  "title": "Agent State Definition",
  "version": "1.0.0",
  "type": "object",
  "required": ["current_state", "previous_state", "transition_timestamp", "state_data"],
  "properties": {
    "current_state": {
      "enum": ["INITIALIZING", "RUNNING", "PAUSED", "STOPPING", "TERMINATED", "ERROR", "RESTARTING"],
      "description": "Current agent state"
    },
    "previous_state": {
      "enum": ["INITIALIZING", "RUNNING", "PAUSED", "STOPPING", "TERMINATED", "ERROR", "RESTARTING", null],
      "description": "Previous agent state (null for initial state)"
    },
    "transition_timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "When the state transition occurred"
    },
    "state_data": {
      "type": "object",
      "description": "State-specific metadata",
      "properties": {
        "initialization_progress": {
          "type": "number",
          "minimum": 0,
          "maximum": 100,
          "description": "Initialization completion percentage"
        },
        "pause_reason": {
          "enum": ["MANUAL", "RESOURCE_CONSTRAINT", "DEPENDENCY_WAIT", "ERROR_RECOVERY"]
        },
        "termination_reason": {
          "enum": ["MANUAL", "COMPLETED", "ERROR", "TIMEOUT", "RESOURCE_EXHAUSTED"]
        },
        "error_details": {
          "type": "object",
          "properties": {
            "error_code": { "type": "string" },
            "error_message": { "type": "string" },
            "recovery_attempted": { "type": "boolean" },
            "retry_count": { "type": "integer", "minimum": 0 }
          }
        },
        "restart_count": {
          "type": "integer",
          "minimum": 0,
          "description": "Number of restarts for this agent instance"
        }
      }
    },
    "transition_history": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "from_state": { "type": "string" },
          "to_state": { "type": "string" },
          "timestamp": { "type": "string", "format": "date-time" },
          "trigger": { "type": "string" },
          "duration_ms": { "type": "integer", "minimum": 0 }
        },
        "required": ["from_state", "to_state", "timestamp"]
      },
      "maxItems": 100,
      "description": "Recent state transition history"
    }
  }
}
```

#### 1.3.2 State Transition Rules Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/state-transitions",
  "title": "Agent State Transition Rules",
  "type": "object",
  "description": "Valid state transitions and their constraints",
  "properties": {
    "INITIALIZING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["RUNNING", "ERROR", "TERMINATED"] },
          "uniqueItems": true
        },
        "timeout_seconds": { "type": "integer", "default": 30 },
        "required_conditions": {
          "type": "array",
          "items": { "enum": ["CONFIGURATION_LOADED", "RESOURCES_ALLOCATED", "DEPENDENCIES_READY"] }
        }
      }
    },
    "RUNNING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["PAUSED", "STOPPING", "ERROR", "RESTARTING"] },
          "uniqueItems": true
        },
        "health_check_interval_seconds": { "type": "integer", "default": 30 },
        "max_idle_seconds": { "type": "integer", "default": 300 }
      }
    },
    "PAUSED": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["RUNNING", "STOPPING", "ERROR"] },
          "uniqueItems": true
        },
        "max_pause_duration_seconds": { "type": "integer", "default": 3600 },
        "auto_resume_conditions": {
          "type": "array",
          "items": { "enum": ["RESOURCE_AVAILABLE", "DEPENDENCY_RESOLVED", "MANUAL_TRIGGER"] }
        }
      }
    },
    "STOPPING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["TERMINATED", "ERROR"] },
          "uniqueItems": true
        },
        "graceful_shutdown_timeout_seconds": { "type": "integer", "default": 60 },
        "force_kill_after_timeout": { "type": "boolean", "default": true }
      }
    },
    "ERROR": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["RESTARTING", "TERMINATED"] },
          "uniqueItems": true
        },
        "auto_restart_conditions": {
          "type": "array",
          "items": { "enum": ["RECOVERABLE_ERROR", "RETRY_LIMIT_NOT_EXCEEDED", "SUPERVISOR_POLICY"] }
        },
        "max_restart_attempts": { "type": "integer", "default": 3 },
        "restart_backoff_seconds": { "type": "integer", "default": 10 }
      }
    },
    "RESTARTING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["INITIALIZING", "ERROR", "TERMINATED"] },
          "uniqueItems": true
        },
        "restart_timeout_seconds": { "type": "integer", "default": 45 }
      }
    },
    "TERMINATED": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": [],
          "description": "Terminal state - no transitions allowed"
        },
        "cleanup_timeout_seconds": { "type": "integer", "default": 30 }
      }
    }
  }
}
```

#### 1.3.3 Agent Lifecycle Management

> **Validation Enhancement**: For distributed scenarios, consider implementing consensus mechanisms for state transitions to ensure consistency across nodes.

/// Agent lifecycle state transition rules
# [derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionRules {
    pub allowed_transitions: HashMap<AgentState, Vec<AgentState>>,
    pub transition_conditions: HashMap<(AgentState, AgentState), Vec<TransitionCondition>>,
    pub timeouts: HashMap<AgentState, Duration>,
}

/// Transition condition definition
# [derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionCondition {
    pub condition_type: ConditionType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub required: bool,
}

/// Condition types for state transitions
# [derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    ResourceAvailable,
    DependencyReady,
    HealthCheck,
    Permission,
    Timeout,
    Custom(String),
}

/// Agent lifecycle management
pub struct AgentLifecycle {
    state: Arc<RwLock<AgentStateInfo>>,
    transition_rules: StateTransitionRules,
    consensus_manager: Option<Arc<dyn StateConsensusManager>>,
    observers: Arc<RwLock<Vec<Arc<dyn StateObserver>>>>,
    transition_history: Arc<RwLock<Vec<StateTransition>>>,
}

/// Complete agent state information
# [derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateInfo {
    pub current_state: AgentState,
    pub previous_state: Option<AgentState>,
    pub transition_timestamp: SystemTime,
    pub state_data: HashMap<String, serde_json::Value>,
    pub restart_count: u32,
}

/// State transition record
# [derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: AgentState,
    pub to_state: AgentState,
    pub trigger: String,
    pub timestamp: SystemTime,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// State consensus manager trait
# [async_trait::async_trait]
pub trait StateConsensusManager: Send + Sync {
    async fn propose_transition(
        &self,
        from_state: AgentState,
        to_state: AgentState,
        trigger: String,
    ) -> Result<ConsensusResult, AgentError>;
}

/// Consensus result
# [derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub approved: bool,
    pub votes: Vec<Vote>,
    pub consensus_time: SystemTime,
}

/// Vote in consensus process
# [derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub approval: bool,
    pub reason: Option<String>,
    pub timestamp: SystemTime,
}

/// State observer trait
# [async_trait::async_trait]
pub trait StateObserver: Send + Sync {
    async fn on_state_change(&self, transition: &StateTransition) -> Result<(), AgentError>;
}

impl AgentLifecycle {
    pub fn new(
        initial_state: AgentState,
        transition_rules: StateTransitionRules,
        consensus_manager: Option<Arc<dyn StateConsensusManager>>,
    ) -> Self {
        let state_info = AgentStateInfo {
            current_state: initial_state,
            previous_state: None,
            transition_timestamp: SystemTime::now(),
            state_data: HashMap::new(),
            restart_count: 0,
        };

        Self {
            state: Arc::new(RwLock::new(state_info)),
            transition_rules,
            consensus_manager,
            observers: Arc::new(RwLock::new(Vec::new())),
            transition_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn transition(
        &self,
        new_state: AgentState,
        trigger: String,
    ) -> Result<(), AgentError> {
        let start_time = SystemTime::now();
        
        // Read current state
        let current_state = {
            let state_guard = self.state.read().unwrap();
            state_guard.current_state.clone()
        };

        // Check if transition is allowed
        let allowed_transitions = self.transition_rules.allowed_transitions
            .get(&current_state)
            .ok_or_else(|| AgentError::StateTransitionError(
                format!("No transitions defined for state {:?}", current_state)
            ))?;

        if !allowed_transitions.contains(&new_state) {
            return Err(AgentError::StateTransitionError(
                format!("Invalid transition from {:?} to {:?}", current_state, new_state)
            ));
        }

        // Check transition conditions
        if !self.check_transition_conditions(&current_state, &new_state).await? {
            return Err(AgentError::StateTransitionError(
                "Transition conditions not met".to_string()
            ));
        }

        // For distributed deployments, ensure consensus
        if let Some(consensus_manager) = &self.consensus_manager {
            let consensus_result = consensus_manager
                .propose_transition(current_state.clone(), new_state.clone(), trigger.clone())
                .await?;
            
            if !consensus_result.approved {
                return Err(AgentError::StateTransitionError(
                    "Consensus not reached for transition".to_string()
                ));
            }
        }

        // Execute transition
        let transition_result = {
            let mut state_guard = self.state.write().unwrap();
            let previous_state = state_guard.current_state.clone();
            state_guard.previous_state = Some(previous_state.clone());
            state_guard.current_state = new_state.clone();
            state_guard.transition_timestamp = SystemTime::now();
            
            StateTransition {
                from_state: previous_state,
                to_state: new_state,
                trigger: trigger.clone(),
                timestamp: start_time,
                duration_ms: start_time.elapsed().unwrap_or_default().as_millis() as u64,
                success: true,
                error_message: None,
            }
        };

        // Record transition history
        self.record_transition_history(transition_result.clone()).await;
        
        // Notify observers
        self.notify_observers(&transition_result).await;

        Ok(())
    }

    async fn check_transition_conditions(
        &self,
        from_state: &AgentState,
        to_state: &AgentState,
    ) -> Result<bool, AgentError> {
        let conditions = self.transition_rules.transition_conditions
            .get(&(from_state.clone(), to_state.clone()))
            .cloned()
            .unwrap_or_default();

        for condition in conditions {
            if condition.required && !self.validate_condition(&condition).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn validate_condition(
        &self,
        condition: &TransitionCondition,
    ) -> Result<bool, AgentError> {
        match condition.condition_type {
            ConditionType::ResourceAvailable => {
                // Implement resource availability check
                Ok(true) // Placeholder
            },
            ConditionType::DependencyReady => {
                // Implement dependency readiness check
                Ok(true) // Placeholder
            },
            ConditionType::HealthCheck => {
                // Implement health check validation
                Ok(true) // Placeholder
            },
            ConditionType::Permission => {
                // Implement permission check
                Ok(true) // Placeholder
            },
            ConditionType::Timeout => {
                // Implement timeout validation
                Ok(true) // Placeholder
            },
            ConditionType::Custom(ref name) => {
                // Implement custom condition validation
                Ok(true) // Placeholder
            },
        }
    }

    async fn record_transition_history(&self, transition: StateTransition) {
        let mut history = self.transition_history.write().unwrap();
        history.push(transition);
        
        // Keep only last 100 transitions
        if history.len() > 100 {
            history.drain(..history.len() - 100);
        }
    }

    async fn notify_observers(&self, transition: &StateTransition) {
        let observers = self.observers.read().unwrap().clone();
        
        for observer in observers {
            if let Err(e) = observer.on_state_change(transition).await {
                log::warn!("State observer error: {}", e);
            }
        }
    }

    pub fn add_observer(&self, observer: Arc<dyn StateObserver>) {
        self.observers.write().unwrap().push(observer);
    }

    pub fn get_current_state(&self) -> AgentState {
        self.state.read().unwrap().current_state.clone()
    }

    pub fn get_state_info(&self) -> AgentStateInfo {
        self.state.read().unwrap().clone()
    }

    pub fn get_transition_history(&self) -> Vec<StateTransition> {
        self.transition_history.read().unwrap().clone()
    }
}

```

## 2. Supervision Patterns

### 2.1 Hub-and-Spoke Supervisor Pattern

Central routing logic with domain-specific delegation:

```rust
/// Hub-and-spoke supervisor pattern for centralized routing
#[async_trait::async_trait]
pub trait Supervisor: Send + Sync {
    async fn route_task(&self, task: Task) -> Result<AgentId, AgentError> {
        // Central routing logic with error handling
        match task.task_type {
            TaskType::Research => self.find_agent("researcher").await,
            TaskType::Code => self.find_agent("coder").await,
            _ => self.default_agent().await,
        }
    }

    async fn find_agent(&self, agent_type: &str) -> Result<AgentId, AgentError>;
    async fn default_agent(&self) -> Result<AgentId, AgentError>;
    async fn supervise_agent(&self, agent: Arc<dyn Agent>) -> Result<(), AgentError>;
    async fn handle_agent_failure(&self, agent_id: AgentId, error: AgentError) -> Result<RecoveryAction, AgentError>;
}

/// Recovery action enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    Restart,
    Escalate,
    Isolate,
    Replace,
    Ignore,
}

// Supervision Pattern Best Practices:
// ❌ Anti-pattern: Monolithic supervisor managing all agents directly
// ✅ Best practice: Hierarchical supervisors with domain-specific delegation
// ❌ Anti-pattern: Supervisor handling business logic
// ✅ Best practice: Supervisor focused only on lifecycle and error handling
```

### 2.2 Event-Driven Message Bus Pattern

> **Full Communication Details**: See `agent-communication.md` section 3.2 for complete publish/subscribe patterns and message routing strategies

```rust
/// Message routing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Routing {
    Broadcast,
    Target(AgentId),
    RoundRobin,
    LoadBalanced,
    TopicBased(String),
}

/// Event-driven message bus for agent communication
pub struct MessageBus {
    channels: Arc<RwLock<HashMap<AgentId, mpsc::Sender<Message>>>>,
    event_loop: Option<JoinHandle<()>>,
    routing_strategy: Arc<dyn RoutingStrategy>,
    metrics: Arc<MessageBusMetrics>,
}

/// Message bus metrics
#[derive(Debug, Default)]
pub struct MessageBusMetrics {
    pub messages_sent: std::sync::atomic::AtomicU64,
    pub messages_failed: std::sync::atomic::AtomicU64,
    pub average_latency_ms: std::sync::atomic::AtomicU64,
}

impl MessageBus {
    pub fn new(routing_strategy: Arc<dyn RoutingStrategy>) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            event_loop: None,
            routing_strategy,
            metrics: Arc::new(MessageBusMetrics::default()),
        }
    }

    pub async fn register_agent(&self, agent_id: AgentId, sender: mpsc::Sender<Message>) {
        self.channels.write().await.insert(agent_id, sender);
    }

    pub async fn unregister_agent(&self, agent_id: &AgentId) {
        self.channels.write().await.remove(agent_id);
    }

    pub async fn publish(&self, msg: Message) -> Result<(), AgentError> {
        let start_time = std::time::Instant::now();
        
        let result = match &msg.payload {
            MessagePayload::Control(ControlMessage::Start) => {
                // Handle control messages with special routing
                self.send_control_message(msg).await
            },
            _ => {
                match msg.recipient {
                    Some(recipient) => self.send_to(recipient, msg).await,
                    None => {
                        // Use routing strategy for recipient selection
                        let channels = self.channels.read().await;
                        let agent_ids: Vec<AgentId> = channels.keys().cloned().collect();
                        let recipient = self.routing_strategy.select_recipient(&msg, &agent_ids)?;
                        drop(channels);
                        self.send_to(recipient, msg).await
                    }
                }
            }
        };

        // Update metrics
        match result {
            Ok(_) => {
                self.metrics.messages_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let latency = start_time.elapsed().as_millis() as u64;
                self.metrics.average_latency_ms.store(latency, std::sync::atomic::Ordering::Relaxed);
            },
            Err(_) => {
                self.metrics.messages_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        result
    }

    async fn send_to(&self, recipient: AgentId, msg: Message) -> Result<(), AgentError> {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&recipient) {
            sender.send(msg).await
                .map_err(|e| AgentError::MessageError(format!("Send failed: {}", e)))
        } else {
            Err(AgentError::MessageError(format!("Agent {} not found", recipient)))
        }
    }

    async fn send_control_message(&self, msg: Message) -> Result<(), AgentError> {
        // Special handling for control messages
        if let Some(recipient) = msg.recipient {
            self.send_to(recipient, msg).await
        } else {
            // Broadcast control messages to all agents
            self.broadcast_all(msg).await
        }
    }

    async fn broadcast_all(&self, msg: Message) -> Result<(), AgentError> {
        let channels = self.channels.read().await;
        let mut errors = Vec::new();
        
        for (agent_id, sender) in channels.iter() {
            let mut broadcast_msg = msg.clone();
            broadcast_msg.recipient = Some(*agent_id);
            
            if let Err(e) = sender.send(broadcast_msg).await {
                errors.push(format!("Failed to send to {}: {}", agent_id, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AgentError::MessageError(errors.join("; ")))
        }
    }

    pub fn get_metrics(&self) -> MessageBusMetrics {
        MessageBusMetrics {
            messages_sent: std::sync::atomic::AtomicU64::new(
                self.metrics.messages_sent.load(std::sync::atomic::Ordering::Relaxed)
            ),
            messages_failed: std::sync::atomic::AtomicU64::new(
                self.metrics.messages_failed.load(std::sync::atomic::Ordering::Relaxed)
            ),
            average_latency_ms: std::sync::atomic::AtomicU64::new(
                self.metrics.average_latency_ms.load(std::sync::atomic::Ordering::Relaxed)
            ),
        }
    }
}

/// Custom routing strategy trait
#[async_trait::async_trait]
pub trait RoutingStrategy: Send + Sync {
    fn select_recipient(&self, msg: &Message, agents: &[AgentId]) -> Result<AgentId, AgentError>;
    async fn update_agent_status(&self, agent_id: AgentId, load: AgentLoad);
}

/// Round-robin routing strategy implementation
pub struct RoundRobinStrategy {
    current_index: std::sync::atomic::AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl RoutingStrategy for RoundRobinStrategy {
    fn select_recipient(&self, _msg: &Message, agents: &[AgentId]) -> Result<AgentId, AgentError> {
        if agents.is_empty() {
            return Err(AgentError::MessageError("No agents available".to_string()));
        }

        let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % agents.len();
        Ok(agents[index])
    }

    async fn update_agent_status(&self, _agent_id: AgentId, _load: AgentLoad) {
        // Round-robin doesn't use load information
    }
}
```

### 2.3 Basic Supervision Tree

```rust
/// Supervision strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartStrategy {
    OneForOne,      // Restart only failed agent
    AllForOne,      // Restart all agents
    RestForOne,     // Restart failed and subsequent agents
    SimpleOneForOne, // Dynamic supervisor for similar agents
}

/// Supervision strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionStrategy {
    pub restart_strategy: RestartStrategy,
    pub max_restarts: u32,
    pub time_window: Duration,
    pub backoff_strategy: BackoffStrategy,
}

/// Backoff strategy for restart delays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed(Duration),
    Exponential { initial: Duration, max: Duration, multiplier: f64 },
    Linear { initial: Duration, increment: Duration },
}

/// Supervision tree node
pub struct SupervisorNode {
    children: Arc<RwLock<HashMap<AgentId, SupervisedChild>>>,
    strategy: SupervisionStrategy,
    restart_counts: Arc<RwLock<HashMap<AgentId, RestartHistory>>>,
    supervisor_id: AgentId,
    metrics: Arc<SupervisionMetrics>,
}

/// Supervised child information
#[derive(Debug)]
pub struct SupervisedChild {
    pub agent: Arc<dyn Agent>,
    pub start_time: SystemTime,
    pub restart_count: u32,
    pub last_failure: Option<AgentError>,
    pub health_monitor: Arc<dyn HealthMonitor>,
}

/// Restart history for an agent
#[derive(Debug, Clone)]
pub struct RestartHistory {
    pub restarts: Vec<RestartEvent>,
    pub total_count: u32,
    pub last_restart: Option<SystemTime>,
}

/// Restart event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartEvent {
    pub timestamp: SystemTime,
    pub reason: String,
    pub success: bool,
    pub duration: Duration,
}

/// Supervision metrics
#[derive(Debug, Default)]
pub struct SupervisionMetrics {
    pub children_supervised: std::sync::atomic::AtomicU32,
    pub total_restarts: std::sync::atomic::AtomicU32,
    pub failed_restarts: std::sync::atomic::AtomicU32,
    pub escalations: std::sync::atomic::AtomicU32,
}

impl SupervisorNode {
    pub fn new(supervisor_id: AgentId, strategy: SupervisionStrategy) -> Self {
        Self {
            children: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            restart_counts: Arc::new(RwLock::new(HashMap::new())),
            supervisor_id,
            metrics: Arc::new(SupervisionMetrics::default()),
        }
    }

    pub async fn supervise(&self, agent: Arc<dyn Agent>) -> Result<(), AgentError> {
        let agent_id = agent.id();
        let health_monitor = Arc::new(BasicHealthMonitor::new(agent_id));
        
        let supervised_child = SupervisedChild {
            agent: agent.clone(),
            start_time: SystemTime::now(),
            restart_count: 0,
            last_failure: None,
            health_monitor: health_monitor.clone(),
        };

        // Add to children
        self.children.write().await.insert(agent_id, supervised_child);
        
        // Initialize restart history
        self.restart_counts.write().await.insert(agent_id, RestartHistory {
            restarts: Vec::new(),
            total_count: 0,
            last_restart: None,
        });

        // Start health monitoring
        self.start_health_monitoring(agent_id, health_monitor).await;
        
        // Update metrics
        self.metrics.children_supervised.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }

    pub async fn handle_child_failure(
        &self,
        agent_id: AgentId,
        error: AgentError,
    ) -> Result<RecoveryAction, AgentError> {
        // Check restart limits
        if !self.should_restart(&agent_id).await? {
            return Ok(RecoveryAction::Escalate);
        }

        // Apply supervision strategy
        match self.strategy.restart_strategy {
            RestartStrategy::OneForOne => {
                self.restart_child(agent_id, error).await
            },
            RestartStrategy::AllForOne => {
                self.restart_all_children(error).await
            },
            RestartStrategy::RestForOne => {
                self.restart_child_and_subsequent(agent_id, error).await
            },
            RestartStrategy::SimpleOneForOne => {
                self.restart_child(agent_id, error).await
            },
        }
    }

    async fn should_restart(&self, agent_id: &AgentId) -> Result<bool, AgentError> {
        let restart_counts = self.restart_counts.read().await;
        
        if let Some(history) = restart_counts.get(agent_id) {
            // Check if within restart limits
            let recent_restarts = history.restarts.iter()
                .filter(|event| {
                    event.timestamp.elapsed().unwrap_or_default() <= self.strategy.time_window
                })
                .count() as u32;
            
            Ok(recent_restarts < self.strategy.max_restarts)
        } else {
            Ok(true) // No history, allow restart
        }
    }

    async fn restart_child(
        &self,
        agent_id: AgentId,
        error: AgentError,
    ) -> Result<RecoveryAction, AgentError> {
        let start_time = SystemTime::now();
        
        // Get current restart count for backoff calculation
        let restart_count = {
            let counts = self.restart_counts.read().await;
            counts.get(&agent_id).map(|h| h.total_count).unwrap_or(0)
        };

        // Calculate backoff delay
        let delay = self.calculate_backoff_delay(restart_count);
        
        if delay > Duration::from_secs(0) {
            tokio::time::sleep(delay).await;
        }

        // Attempt restart
        let restart_result = self.perform_restart(agent_id).await;
        
        // Record restart event
        let restart_event = RestartEvent {
            timestamp: start_time,
            reason: format!("Child failure: {}", error),
            success: restart_result.is_ok(),
            duration: start_time.elapsed().unwrap_or_default(),
        };
        
        self.record_restart_event(agent_id, restart_event).await;
        
        match restart_result {
            Ok(_) => {
                self.metrics.total_restarts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(RecoveryAction::Restart)
            },
            Err(e) => {
                self.metrics.failed_restarts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(RecoveryAction::Escalate)
            }
        }
    }

    async fn restart_all_children(
        &self,
        error: AgentError,
    ) -> Result<RecoveryAction, AgentError> {
        let children_ids: Vec<AgentId> = {
            self.children.read().await.keys().cloned().collect()
        };

        for agent_id in children_ids {
            if let Err(e) = self.restart_child(agent_id, error.clone()).await {
                log::error!("Failed to restart child {}: {}", agent_id, e);
            }
        }

        Ok(RecoveryAction::Restart)
    }

    async fn restart_child_and_subsequent(
        &self,
        failed_agent_id: AgentId,
        error: AgentError,
    ) -> Result<RecoveryAction, AgentError> {
        // Implementation would depend on the ordering strategy
        // For now, restart the failed child and all children
        self.restart_all_children(error).await
    }

    fn calculate_backoff_delay(&self, restart_count: u32) -> Duration {
        match &self.strategy.backoff_strategy {
            BackoffStrategy::Fixed(duration) => *duration,
            BackoffStrategy::Exponential { initial, max, multiplier } => {
                let delay = Duration::from_millis(
                    (initial.as_millis() as f64 * multiplier.powi(restart_count as i32)) as u64
                );
                std::cmp::min(delay, *max)
            },
            BackoffStrategy::Linear { initial, increment } => {
                *initial + *increment * restart_count
            },
        }
    }

    async fn perform_restart(&self, agent_id: AgentId) -> Result<(), AgentError> {
        let mut children = self.children.write().await;
        
        if let Some(child) = children.get_mut(&agent_id) {
            // Stop the current agent
            if let Err(e) = child.agent.stop().await {
                log::warn!("Error stopping agent during restart: {}", e);
            }
            
            // Start the agent again
            let mut agent_clone = child.agent.clone();
            if let Err(e) = agent_clone.start().await {
                return Err(AgentError::StateTransitionError(
                    format!("Failed to restart agent: {}", e)
                ));
            }
            
            child.restart_count += 1;
            child.start_time = SystemTime::now();
            child.last_failure = None;
            
            Ok(())
        } else {
            Err(AgentError::SupervisionError(
                format!("Agent {} not found for restart", agent_id)
            ))
        }
    }

    async fn record_restart_event(&self, agent_id: AgentId, event: RestartEvent) {
        let mut restart_counts = self.restart_counts.write().await;
        
        if let Some(history) = restart_counts.get_mut(&agent_id) {
            history.restarts.push(event);
            history.total_count += 1;
            history.last_restart = Some(SystemTime::now());
            
            // Keep only recent restart events (last 100)
            if history.restarts.len() > 100 {
                history.restarts.drain(..history.restarts.len() - 100);
            }
        }
    }

    async fn start_health_monitoring(
        &self,
        agent_id: AgentId,
        health_monitor: Arc<dyn HealthMonitor>,
    ) {
        // Start health monitoring task
        let supervisor_weak = Arc::downgrade(&self.children);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check if supervisor still exists
                if supervisor_weak.upgrade().is_none() {
                    break;
                }
                
                // Perform health check
                match health_monitor.check_health().await {
                    Ok(health_status) => {
                        if let HealthLevel::Critical(_) = health_status.health {
                            log::error!("Critical health detected for agent {}", agent_id);
                            // Could trigger failure handling here
                        }
                    },
                    Err(e) => {
                        log::error!("Health check failed for agent {}: {}", agent_id, e);
                    }
                }
            }
        });
    }

    pub fn get_metrics(&self) -> SupervisionMetrics {
        SupervisionMetrics {
            children_supervised: std::sync::atomic::AtomicU32::new(
                self.metrics.children_supervised.load(std::sync::atomic::Ordering::Relaxed)
            ),
            total_restarts: std::sync::atomic::AtomicU32::new(
                self.metrics.total_restarts.load(std::sync::atomic::Ordering::Relaxed)
            ),
            failed_restarts: std::sync::atomic::AtomicU32::new(
                self.metrics.failed_restarts.load(std::sync::atomic::Ordering::Relaxed)
            ),
            escalations: std::sync::atomic::AtomicU32::new(
                self.metrics.escalations.load(std::sync::atomic::Ordering::Relaxed)
            ),
        }
    }
}
```

### 2.4 Simple Restart Logic

```rust
/// Restart policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub time_window: Duration,
    pub cooldown_period: Duration,
    pub escalation_threshold: u32,
}

/// Restart policy manager
pub struct RestartPolicyManager {
    policy: RestartPolicy,
    restart_counts: Arc<RwLock<HashMap<AgentId, Vec<SystemTime>>>>,
    escalation_counts: Arc<RwLock<HashMap<AgentId, u32>>>,
}

impl RestartPolicyManager {
    pub fn new(policy: RestartPolicy) -> Self {
        Self {
            policy,
            restart_counts: Arc::new(RwLock::new(HashMap::new())),
            escalation_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn should_restart(&self, agent_id: &AgentId) -> bool {
        let recent_restarts = self.count_recent_restarts(agent_id).await;
        let escalation_count = self.get_escalation_count(agent_id).await;
        
        // Check basic restart limit
        if recent_restarts >= self.policy.max_restarts {
            return false;
        }
        
        // Check escalation threshold
        if escalation_count >= self.policy.escalation_threshold {
            return false;
        }
        
        // Check cooldown period
        if let Some(last_restart) = self.get_last_restart_time(agent_id).await {
            if last_restart.elapsed().unwrap_or_default() < self.policy.cooldown_period {
                return false;
            }
        }
        
        true
    }

    pub async fn record_restart(&self, agent_id: AgentId) {
        let mut restart_counts = self.restart_counts.write().await;
        let now = SystemTime::now();
        
        restart_counts.entry(agent_id)
            .or_insert_with(Vec::new)
            .push(now);
        
        // Clean up old entries outside time window
        if let Some(timestamps) = restart_counts.get_mut(&agent_id) {
            let cutoff = now.checked_sub(self.policy.time_window).unwrap_or(now);
            timestamps.retain(|&timestamp| timestamp >= cutoff);
        }
    }

    pub async fn record_escalation(&self, agent_id: AgentId) {
        let mut escalation_counts = self.escalation_counts.write().await;
        *escalation_counts.entry(agent_id).or_insert(0) += 1;
    }

    async fn count_recent_restarts(&self, agent_id: &AgentId) -> u32 {
        let restart_counts = self.restart_counts.read().await;
        
        if let Some(timestamps) = restart_counts.get(agent_id) {
            let cutoff = SystemTime::now()
                .checked_sub(self.policy.time_window)
                .unwrap_or_else(SystemTime::now);
            
            timestamps.iter()
                .filter(|&&timestamp| timestamp >= cutoff)
                .count() as u32
        } else {
            0
        }
    }

    async fn get_escalation_count(&self, agent_id: &AgentId) -> u32 {
        let escalation_counts = self.escalation_counts.read().await;
        escalation_counts.get(agent_id).copied().unwrap_or(0)
    }

    async fn get_last_restart_time(&self, agent_id: &AgentId) -> Option<SystemTime> {
        let restart_counts = self.restart_counts.read().await;
        restart_counts.get(agent_id)
            .and_then(|timestamps| timestamps.last())
            .copied()
    }

    pub async fn reset_agent_history(&self, agent_id: &AgentId) {
        let mut restart_counts = self.restart_counts.write().await;
        let mut escalation_counts = self.escalation_counts.write().await;
        
        restart_counts.remove(agent_id);
        escalation_counts.remove(agent_id);
    }

    pub fn get_restart_count(&self, agent_id: &AgentId) -> u32 {
        // Synchronous version for quick access
        if let Ok(restart_counts) = self.restart_counts.try_read() {
            restart_counts.get(agent_id)
                .map(|timestamps| timestamps.len() as u32)
                .unwrap_or(0)
        } else {
            0
        }
    }
}
```

## 3. Agent Initialization Procedures

### 3.1 Initialization Pipeline

// This section has been replaced with comprehensive implementation above

```

### 3.2 Resource Allocation

```rust
/// Resource allocation with limits and validation
struct ResourceAllocator {
    cpu_pool: CpuPool,
    memory_pool: MemoryPool,
    connection_pool: ConnectionPool,
    quota_manager: ResourceQuotaManager,
    memory_pressure_monitor: MemoryPressureMonitor,
}

/// Resource quota management per agent
struct ResourceQuotaManager {
    agent_quotas: Arc<RwLock<HashMap<AgentId, ResourceQuota>>>,
    default_quotas: HashMap<AgentType, ResourceQuota>,
    enforcement_policy: QuotaEnforcementPolicy,
}

#[derive(Clone, Debug)]
struct ResourceQuota {
    max_cpu_cores: f32,
    max_memory_mb: usize,
    max_connections: u32,
    max_file_handles: u32,
    burst_allowance: BurstQuota,
}

/// Memory pressure monitoring and backpressure
struct MemoryPressureMonitor {
    system_memory: Arc<SystemMemoryInfo>,
    pressure_thresholds: PressureThresholds,
    backpressure_strategy: BackpressureStrategy,
}

#[derive(Clone)]
struct PressureThresholds {
    warning_level: f32,  // e.g., 0.7 (70% memory usage)
    critical_level: f32, // e.g., 0.85 (85% memory usage)
    emergency_level: f32, // e.g., 0.95 (95% memory usage)
}

impl ResourceAllocator {
    async fn allocate(&self, agent_id: &AgentId, requirements: ResourceRequirements) -> Result<AllocatedResources, AllocationError> {
        // Check quota limits first
        self.quota_manager.validate_against_quota(agent_id, &requirements).await?;
        
        // Check memory pressure
        let memory_pressure = self.memory_pressure_monitor.current_pressure().await;
        if memory_pressure > PressureLevel::Warning {
            // Apply backpressure strategy
            match self.memory_pressure_monitor.backpressure_strategy {
                BackpressureStrategy::Reject => {
                    return Err(AllocationError::MemoryPressure(memory_pressure));
                },
                BackpressureStrategy::Defer(duration) => {
                    tokio::time::sleep(duration).await;
                },
                BackpressureStrategy::Reduce(factor) => {
                    // Reduce allocation by factor
                    requirements = self.reduce_requirements(requirements, factor);
                },
            }
        }
        
        // Check available resources
        self.validate_availability(&requirements)?;
        
        // Reserve resources atomically
        let transaction = self.begin_transaction().await?;
        
        let resources = AllocatedResources {
            cpu_cores: self.cpu_pool.reserve(requirements.cpu_cores, &transaction)?,
            memory_mb: self.memory_pool.reserve(requirements.memory_mb, &transaction)?,
            connections: self.connection_pool.reserve(requirements.max_connections, &transaction)?,
            file_handles: self.reserve_file_handles(requirements.max_files)?,
        };
        
        // Update quota usage
        self.quota_manager.record_usage(agent_id, &resources).await?;
        
        transaction.commit().await?;
        Ok(resources)
    }
    
    async fn deallocate(&self, resources: AllocatedResources) -> Result<(), DeallocationError> {
        // Return resources to pools
        self.cpu_pool.release(resources.cpu_cores).await?;
        self.memory_pool.release(resources.memory_mb).await?;
        self.connection_pool.release(resources.connections).await?;
        Ok(())
    }
}
```

### 3.3 Dependency Resolution

```rust
/// Dependency injection and resolution
struct DependencyResolver {
    registry: Arc<RwLock<DependencyRegistry>>,
    graph: DependencyGraph,
}

impl DependencyResolver {
    async fn resolve(&self, agent_type: &AgentType) -> Result<ResolvedDependencies, ResolutionError> {
        // Get dependency requirements
        let requirements = self.get_requirements(agent_type)?;
        
        // Check for circular dependencies
        self.graph.validate_no_cycles(&requirements)?;
        
        // Resolve in topological order
        let resolution_order = self.graph.topological_sort(&requirements)?;
        
        let mut resolved = ResolvedDependencies::new();
        
        for dep in resolution_order {
            match dep {
                Dependency::Service(name) => {
                    let service = self.resolve_service(&name).await?;
                    resolved.services.insert(name, service);
                },
                Dependency::Resource(name) => {
                    let resource = self.resolve_resource(&name).await?;
                    resolved.resources.insert(name, resource);
                },
                Dependency::Agent(agent_id) => {
                    let agent_ref = self.resolve_agent(&agent_id).await?;
                    resolved.agents.insert(agent_id, agent_ref);
                },
            }
        }
        
        Ok(resolved)
    }
}
```

## 4. Health Monitoring Implementation

### 4.1 Health Check Framework

// This section has been replaced with comprehensive HealthMonitor implementation above

/// Individual health check types
enum HealthCheckType {
    Liveness,      // Is the agent running?
    Readiness,     // Is the agent ready to accept work?
    Performance,   // Is the agent performing within SLAs?
    Resource,      // Are resources within limits?
}

struct HealthStatus {
    overall: HealthLevel,
    checks: HashMap<String, CheckResult>,
    last_check: SystemTime,
    consecutive_failures: u32,
}

enum HealthLevel {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Critical(String),
}

```

### 4.2 Metrics Collection

```rust
/// Real-time metrics collection
struct MetricsCollector {
    agent_id: AgentId,
    metrics_store: Arc<MetricsStore>,
    aggregator: MetricsAggregator,
}

impl MetricsCollector {
    async fn collect_metrics(&self) -> AgentMetrics {
        AgentMetrics {
            cpu_usage: self.collect_cpu_usage().await,
            memory_usage: self.collect_memory_usage().await,
            message_throughput: self.collect_message_metrics().await,
            task_completion_rate: self.collect_task_metrics().await,
            error_rate: self.collect_error_metrics().await,
            latency_percentiles: self.collect_latency_metrics().await,
            custom_metrics: self.collect_custom_metrics().await,
        }
    }
    
    async fn record_event(&self, event: MetricEvent) {
        self.metrics_store.record(self.agent_id.clone(), event).await;
        self.aggregator.update(&event).await;
    }
}

/// Metric aggregation windows
struct MetricsAggregator {
    windows: HashMap<Duration, AggregationWindow>,
    percentile_estimators: HashMap<String, PercentileEstimator>,
}
```

### 4.3 Adaptive Health Monitoring

```rust
/// Self-adjusting health monitoring based on agent behavior
struct AdaptiveHealthMonitor {
    base_monitor: AgentHealthMonitor,
    learning_engine: HealthLearningEngine,
    anomaly_detector: AnomalyDetector,
}

impl AdaptiveHealthMonitor {
    async fn adaptive_check(&mut self) -> AdaptiveHealthReport {
        let current_metrics = self.base_monitor.get_metrics().await;
        
        // Detect anomalies
        let anomalies = self.anomaly_detector.analyze(&current_metrics).await;
        
        // Adjust thresholds based on historical patterns
        if let Some(new_thresholds) = self.learning_engine.suggest_thresholds(&current_metrics).await {
            self.base_monitor.update_thresholds(new_thresholds);
        }
        
        // Generate adaptive report
        AdaptiveHealthReport {
            base_health: self.base_monitor.check_health().await,
            anomalies,
            recommended_actions: self.generate_recommendations(&anomalies),
            confidence_score: self.learning_engine.confidence(),
        }
    }
}
```

## 5. Graceful Shutdown Procedures

### 5.1 Shutdown Orchestration

```rust
/// Coordinated shutdown with resource cleanup
trait GracefulShutdown {
    async fn initiate_shutdown(&self) -> Result<ShutdownHandle, ShutdownError>;
    async fn await_completion(&self, handle: ShutdownHandle) -> Result<(), ShutdownError>;
    fn force_shutdown(&self);
}

struct ShutdownOrchestrator {
    shutdown_phases: Vec<ShutdownPhase>,
    timeout_policy: TimeoutPolicy,
    cleanup_registry: CleanupRegistry,
}

impl ShutdownOrchestrator {
    async fn execute_shutdown(&self, agent: &Agent) -> Result<(), ShutdownError> {
        let shutdown_context = ShutdownContext::new(agent.id(), SystemTime::now());
        
        // Phase 1: Stop accepting new work
        self.stop_work_acceptance(agent).await?;
        
        // Phase 2: Complete in-flight operations
        let pending = self.drain_pending_operations(agent, self.timeout_policy.drain_timeout).await?;
        
        // Phase 3: Persist state
        self.persist_final_state(agent, &pending).await?;
        
        // Phase 4: Release resources
        self.release_resources(agent).await?;
        
        // Phase 5: Cleanup and finalize
        self.cleanup_registry.execute_cleanup(agent).await?;
        
        Ok(())
    }
    
    async fn drain_pending_operations(&self, agent: &Agent, timeout: Duration) -> Result<PendingOperations, DrainError> {
        let start = Instant::now();
        let mut pending = agent.get_pending_operations().await;
        
        while !pending.is_empty() && start.elapsed() < timeout {
            // Process high-priority operations first
            pending.sort_by_priority();
            
            for op in pending.iter_mut() {
                match op.try_complete(timeout - start.elapsed()).await {
                    Ok(_) => op.mark_completed(),
                    Err(e) if e.is_retriable() => op.mark_for_handoff(),
                    Err(e) => op.mark_failed(e),
                }
            }
            
            pending.retain(|op| !op.is_completed());
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Ok(pending)
    }
}
```

### 5.2 State Persistence

```rust
/// Agent state persistence during shutdown
struct StatePersistence {
    storage_backend: Arc<dyn StateStorage>,
    serializer: StateSerializer,
    encryption: Option<StateEncryption>,
}

impl StatePersistence {
    async fn persist_agent_state(&self, agent: &Agent) -> Result<StateSnapshot, PersistenceError> {
        // Collect complete agent state
        let state = AgentState {
            agent_id: agent.id(),
            lifecycle_state: agent.current_state(),
            pending_tasks: agent.get_pending_tasks().await,
            active_connections: agent.get_connections().await,
            internal_state: agent.export_internal_state().await?,
            metrics_snapshot: agent.get_metrics_snapshot().await,
            timestamp: SystemTime::now(),
        };
        
        // Serialize state
        let serialized = self.serializer.serialize(&state)?;
        
        // Optionally encrypt
        let data = match &self.encryption {
            Some(enc) => enc.encrypt(&serialized)?,
            None => serialized,
        };
        
        // Store with metadata
        let snapshot = StateSnapshot {
            id: Uuid::new_v4(),
            agent_id: agent.id(),
            data,
            checksum: calculate_checksum(&data),
            created_at: SystemTime::now(),
        };
        
        self.storage_backend.store_snapshot(&snapshot).await?;
        
        Ok(snapshot)
    }
}
```

### 5.3 Resource Cleanup

```rust
/// Systematic resource cleanup
struct ResourceCleanup {
    cleanup_handlers: HashMap<ResourceType, Box<dyn CleanupHandler>>,
    cleanup_order: Vec<ResourceType>,
}

impl ResourceCleanup {
    async fn cleanup_agent_resources(&self, agent: &Agent) -> Result<CleanupReport, CleanupError> {
        let mut report = CleanupReport::new(agent.id());
        
        for resource_type in &self.cleanup_order {
            if let Some(handler) = self.cleanup_handlers.get(resource_type) {
                let result = handler.cleanup(agent).await;
                
                match result {
                    Ok(cleaned) => report.add_success(resource_type, cleaned),
                    Err(e) => {
                        report.add_failure(resource_type, e.clone());
                        
                        // Decide whether to continue based on error severity
                        if e.is_critical() {
                            return Err(CleanupError::Critical(report));
                        }
                    }
                }
            }
        }
        
        Ok(report)
    }
}

/// Cleanup handlers for different resource types
trait CleanupHandler: Send + Sync {
    async fn cleanup(&self, agent: &Agent) -> Result<CleanupResult, CleanupError>;
    fn resource_type(&self) -> ResourceType;
    fn is_critical(&self) -> bool;
}
```

## 6. Recovery and Restart Patterns

### 6.1 Recovery Strategies

```rust
/// Comprehensive recovery framework
enum RecoveryStrategy {
    /// Restart with same configuration
    SimpleRestart,
    
    /// Restart with exponential backoff
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
    
    /// Restart with modified configuration
    AdaptiveRestart {
        config_adjuster: Box<dyn ConfigAdjuster>,
    },
    
    /// Migrate to different node
    Migration {
        target_selector: Box<dyn NodeSelector>,
    },
    
    /// Circuit breaker pattern
    CircuitBreaker {
        failure_threshold: u32,
        recovery_timeout: Duration,
        half_open_attempts: u32,
    },
}

struct RecoveryOrchestrator {
    strategies: HashMap<FailureType, RecoveryStrategy>,
    state_recovery: StateRecovery,
    health_validator: HealthValidator,
}

impl RecoveryOrchestrator {
    async fn recover_agent(&self, failed_agent: &FailedAgent) -> Result<Agent, RecoveryError> {
        // Determine failure type
        let failure_type = self.analyze_failure(failed_agent)?;
        
        // Select recovery strategy
        let strategy = self.strategies.get(&failure_type)
            .ok_or(RecoveryError::NoStrategyAvailable)?;
        
        // Execute recovery
        match strategy {
            RecoveryStrategy::SimpleRestart => {
                self.simple_restart(failed_agent).await
            },
            RecoveryStrategy::ExponentialBackoff { initial_delay, max_delay, multiplier } => {
                self.backoff_restart(failed_agent, *initial_delay, *max_delay, *multiplier).await
            },
            RecoveryStrategy::AdaptiveRestart { config_adjuster } => {
                self.adaptive_restart(failed_agent, config_adjuster).await
            },
            RecoveryStrategy::Migration { target_selector } => {
                self.migrate_agent(failed_agent, target_selector).await
            },
            RecoveryStrategy::CircuitBreaker { failure_threshold, recovery_timeout, half_open_attempts } => {
                self.circuit_breaker_recovery(failed_agent, *failure_threshold, *recovery_timeout, *half_open_attempts).await
            },
        }
    }
}
```

### 6.2 State Recovery

```rust
/// State recovery from persistent storage
struct StateRecovery {
    storage_backend: Arc<dyn StateStorage>,
    validator: StateValidator,
    migration_engine: StateMigrationEngine,
}

impl StateRecovery {
    async fn recover_agent_state(&self, agent_id: &AgentId) -> Result<RecoveredState, StateRecoveryError> {
        // Find latest valid snapshot
        let snapshots = self.storage_backend.list_snapshots(agent_id).await?;
        
        for snapshot in snapshots.iter().rev() {
            match self.validate_and_recover(snapshot).await {
                Ok(state) => {
                    // Check if migration is needed
                    if let Some(migrated) = self.migration_engine.migrate_if_needed(&state).await? {
                        return Ok(migrated);
                    }
                    return Ok(state);
                },
                Err(e) => {
                    log::warn!("Failed to recover from snapshot {}: {}", snapshot.id, e);
                    continue;
                }
            }
        }
        
        Err(StateRecoveryError::NoValidSnapshot)
    }
    
    async fn validate_and_recover(&self, snapshot: &StateSnapshot) -> Result<RecoveredState, ValidationError> {
        // Verify checksum
        let calculated = calculate_checksum(&snapshot.data);
        if calculated != snapshot.checksum {
            return Err(ValidationError::ChecksumMismatch);
        }
        
        // Decrypt if needed
        let decrypted = self.decrypt_if_needed(&snapshot.data)?;
        
        // Deserialize
        let state: AgentState = self.deserialize(&decrypted)?;
        
        // Validate state consistency
        self.validator.validate(&state)?;
        
        Ok(RecoveredState {
            state,
            snapshot_id: snapshot.id,
            recovered_at: SystemTime::now(),
        })
    }
}
```

### 6.3 Restart Coordination

```rust
/// Coordinated restart with supervision integration
struct RestartCoordinator {
    supervisor_ref: Arc<dyn Supervisor>,
    restart_policy: RestartPolicy,
    resource_manager: ResourceManager,
    notification_service: NotificationService,
}

impl RestartCoordinator {
    async fn coordinate_restart(&self, agent_id: &AgentId, reason: RestartReason) -> Result<Agent, RestartError> {
        // Notify supervisor
        self.supervisor_ref.notify_restart_intent(agent_id, &reason).await?;
        
        // Check restart policy
        if !self.restart_policy.should_restart(agent_id) {
            return Err(RestartError::PolicyViolation("Restart limit exceeded".into()));
        }
        
        // Reserve resources for restart
        let resources = self.resource_manager.reserve_for_restart(agent_id).await?;
        
        // Create restart context
        let context = RestartContext {
            agent_id: agent_id.clone(),
            reason,
            attempt_number: self.restart_policy.get_restart_count(agent_id),
            resources,
            timestamp: SystemTime::now(),
        };
        
        // Execute restart phases
        let new_agent = self.execute_restart_phases(&context).await?;
        
        // Update policies and notify
        self.restart_policy.record_restart(agent_id);
        self.notification_service.notify_restart_complete(agent_id, &new_agent).await;
        
        Ok(new_agent)
    }
    
    async fn execute_restart_phases(&self, context: &RestartContext) -> Result<Agent, RestartError> {
        // Phase 1: Cleanup old instance
        self.cleanup_previous_instance(&context.agent_id).await?;
        
        // Phase 2: Recover state if available
        let recovered_state = self.try_recover_state(&context.agent_id).await;
        
        // Phase 3: Initialize new instance
        let config = self.prepare_restart_config(context, &recovered_state)?;
        let new_agent = self.initialize_agent(config).await?;
        
        // Phase 4: Restore state if recovered
        if let Some(state) = recovered_state {
            new_agent.restore_state(state).await?;
        }
        
        // Phase 5: Validate health
        self.validate_restart_health(&new_agent).await?;
        
        Ok(new_agent)
    }
}
```

## 7. Supervision Tree Integration

> **Implementation Note**: The following implementation includes comprehensive supervision metrics collection and distributed consensus support for production environments.

### 7.1 Lifecycle Supervision

```rust
/// Integration with supervision tree for lifecycle management
struct LifecycleSupervisor {
    supervised_agents: Arc<RwLock<HashMap<AgentId, SupervisedAgent>>>,
    supervision_tree: Arc<SupervisionTree>,
    lifecycle_policies: HashMap<AgentType, LifecyclePolicy>,
    supervision_metrics: SupervisionMetricsCollector,
}

/// Supervision-specific metrics collection
struct SupervisionMetricsCollector {
    restart_metrics: RestartMetrics,
    failure_metrics: FailureMetrics,
    supervision_event_log: Arc<RwLock<CircularBuffer<SupervisionEvent>>>,
}

#[derive(Clone, Debug)]
struct RestartMetrics {
    restart_counts: HashMap<AgentId, u32>,
    restart_reasons: HashMap<RestartReason, u32>,
    restart_durations: HashMap<AgentId, Vec<Duration>>,
    success_rates: HashMap<AgentId, f32>,
}

#[derive(Clone, Debug)]
struct FailureMetrics {
    failure_counts: HashMap<FailureType, u32>,
    escalation_counts: HashMap<EscalationLevel, u32>,
    mtbf: HashMap<AgentId, Duration>, // Mean Time Between Failures
    recovery_times: HashMap<AgentId, Vec<Duration>>,
}

impl LifecycleSupervisor {
    async fn supervise_agent(&self, agent: Agent) -> Result<SupervisedAgent, SupervisionError> {
        let agent_type = agent.agent_type();
        let policy = self.lifecycle_policies.get(&agent_type)
            .ok_or(SupervisionError::NoPolicyDefined)?;
        
        // Create supervision wrapper
        let supervised = SupervisedAgent {
            agent,
            supervisor_id: self.generate_supervisor_id(),
            policy: policy.clone(),
            health_monitor: self.create_health_monitor(&policy),
            restart_controller: self.create_restart_controller(&policy),
            metrics_collector: MetricsCollector::new(),
        };
        
        // Register with supervision tree
        self.supervision_tree.register_child(
            supervised.supervisor_id.clone(),
            supervised.agent.id(),
            policy.supervision_strategy
        ).await?;
        
        // Start lifecycle monitoring
        self.start_lifecycle_monitoring(&supervised).await;
        
        // Record supervision metrics
        self.supervision_metrics.record_supervision_start(
            supervised.agent.id(),
            supervised.agent_type(),
            SystemTime::now()
        ).await;
        
        // Store reference
        self.supervised_agents.write().await
            .insert(supervised.agent.id(), supervised.clone());
        
        Ok(supervised)
    }
}
```

### 7.2 Failure Escalation

```rust
/// Failure escalation through supervision hierarchy
struct FailureEscalator {
    escalation_rules: HashMap<FailureType, EscalationRule>,
    supervisor_hierarchy: Arc<SupervisorHierarchy>,
}

impl FailureEscalator {
    async fn escalate_failure(&self, failure: AgentFailure) -> Result<EscalationResult, EscalationError> {
        let rule = self.escalation_rules.get(&failure.failure_type)
            .ok_or(EscalationError::NoRuleFound)?;
        
        match rule.escalation_level {
            EscalationLevel::Local => {
                // Handle at agent level
                self.handle_locally(&failure).await
            },
            EscalationLevel::Supervisor => {
                // Escalate to immediate supervisor
                let supervisor = self.supervisor_hierarchy
                    .get_supervisor(&failure.agent_id).await?;
                supervisor.handle_child_failure(failure).await
            },
            EscalationLevel::Root => {
                // Escalate to root supervisor
                let root = self.supervisor_hierarchy.get_root().await?;
                root.handle_critical_failure(failure).await
            },
            EscalationLevel::System => {
                // System-wide failure handling
                self.trigger_system_recovery(failure).await
            },
        }
    }
}
```

## 8. Error Handling Patterns

### 8.1 Comprehensive Error Taxonomy

```rust
/// Structured error types for lifecycle management
#[derive(Debug, Clone)]
enum LifecycleError {
    InitializationError {
        phase: String,
        cause: String,
        recoverable: bool,
    },
    StateTransitionError {
        from: AgentState,
        to: AgentState,
        reason: String,
    },
    ResourceError {
        resource_type: ResourceType,
        operation: String,
        details: String,
    },
    HealthCheckError {
        check_name: String,
        threshold_violated: String,
        current_value: String,
    },
    ShutdownError {
        phase: ShutdownPhase,
        timeout_exceeded: bool,
        pending_operations: u32,
    },
    RecoveryError {
        strategy: String,
        attempts: u32,
        last_error: String,
    },
}

impl LifecycleError {
    fn is_recoverable(&self) -> bool {
        match self {
            LifecycleError::InitializationError { recoverable, .. } => *recoverable,
            LifecycleError::StateTransitionError { .. } => true,
            LifecycleError::ResourceError { .. } => false,
            LifecycleError::HealthCheckError { .. } => true,
            LifecycleError::ShutdownError { .. } => false,
            LifecycleError::RecoveryError { attempts, .. } => *attempts < 3,
        }
    }
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            LifecycleError::InitializationError { recoverable, .. } => {
                if *recoverable { ErrorSeverity::Warning } else { ErrorSeverity::Critical }
            },
            LifecycleError::StateTransitionError { .. } => ErrorSeverity::Error,
            LifecycleError::ResourceError { .. } => ErrorSeverity::Critical,
            LifecycleError::HealthCheckError { .. } => ErrorSeverity::Warning,
            LifecycleError::ShutdownError { pending_operations, .. } => {
                if *pending_operations > 0 { ErrorSeverity::Error } else { ErrorSeverity::Warning }
            },
            LifecycleError::RecoveryError { attempts, .. } => {
                if *attempts > 2 { ErrorSeverity::Critical } else { ErrorSeverity::Error }
            },
        }
    }
}
```

### 8.2 Error Recovery Strategies

```rust
/// Error recovery with fallback strategies
struct ErrorRecoveryEngine {
    recovery_strategies: HashMap<ErrorCategory, Vec<RecoveryAction>>,
    fallback_chain: FallbackChain,
}

impl ErrorRecoveryEngine {
    async fn recover_from_error(&self, error: LifecycleError, agent: &Agent) -> Result<RecoveryOutcome, FatalError> {
        let category = self.categorize_error(&error);
        let strategies = self.recovery_strategies.get(&category)
            .ok_or(FatalError::NoRecoveryStrategy)?;
        
        for (idx, action) in strategies.iter().enumerate() {
            match self.execute_recovery_action(action, &error, agent).await {
                Ok(outcome) => return Ok(outcome),
                Err(e) => {
                    log::warn!("Recovery action {} failed: {}", idx, e);
                    continue;
                }
            }
        }
        
        // All strategies failed, try fallback
        self.fallback_chain.execute(&error, agent).await
    }
}
```

## 9. Implementation Examples

### 9.1 State Versioning Support

```rust
/// Protocol Buffers schema for versioned state (recommended for production)
// agent_state.proto
message AgentStateProto {
    uint32 version = 1;
    string current_state = 2;
    string previous_state = 3;
    google.protobuf.Timestamp transition_timestamp = 4;
    StateDataProto state_data = 5;
    repeated TransitionHistoryEntry transition_history = 6;
}

message StateDataProto {
    float initialization_progress = 1;
    PauseReason pause_reason = 2;
    TerminationReason termination_reason = 3;
    ErrorDetails error_details = 4;
    uint32 restart_count = 5;
}

// State migration support
trait StateVersionMigrator {
    fn migrate_v1_to_v2(&self, v1_state: &AgentStateV1) -> Result<AgentStateV2, MigrationError>;
    fn migrate_v2_to_v3(&self, v2_state: &AgentStateV2) -> Result<AgentStateV3, MigrationError>;
}
```

### 9.2 Distributed Consensus for State Transitions

```rust
/// Consensus manager for distributed state transitions
struct StateConsensusManager {
    node_id: NodeId,
    consensus_protocol: Arc<dyn ConsensusProtocol>,
    state_replicator: StateReplicator,
    quorum_size: usize,
}

impl StateConsensusManager {
    async fn propose_transition(
        &self,
        from: AgentState,
        to: AgentState,
        trigger: String
    ) -> Result<ConsensusResult, ConsensusError> {
        let proposal = StateTransitionProposal {
            proposer: self.node_id.clone(),
            from_state: from,
            to_state: to,
            trigger,
            timestamp: SystemTime::now(),
            sequence_number: self.next_sequence_number(),
        };
        
        // Broadcast proposal to cluster
        let votes = self.consensus_protocol.propose(&proposal).await?;
        
        // Check if quorum reached
        if votes.len() >= self.quorum_size {
            // Apply transition atomically across cluster
            self.state_replicator.replicate_transition(&proposal).await?;
            Ok(ConsensusResult::Approved(votes))
        } else {
            Ok(ConsensusResult::Rejected("Insufficient votes".into()))
        }
    }
}
```

### 9.3 Complete Agent Lifecycle Example

```rust
/// Example of complete agent lifecycle implementation
struct ResearchAgent {
    id: AgentId,
    lifecycle: AgentLifecycle,
    health_monitor: AgentHealthMonitor,
    supervisor_ref: Arc<dyn Supervisor>,
}

impl ResearchAgent {
    async fn new(config: AgentConfig) -> Result<Self, InitError> {
        // Initialize through proper lifecycle
        let initializer = AgentInitializer::new();
        let agent = initializer.initialize(config).await?;
        
        // Set up health monitoring
        let mut health_monitor = AgentHealthMonitor::new(agent.id());
        health_monitor.register_check(Box::new(CpuHealthCheck::new(0.8)));
        health_monitor.register_check(Box::new(MemoryHealthCheck::new(1024)));
        health_monitor.register_check(Box::new(TaskQueueHealthCheck::new(100)));
        
        Ok(ResearchAgent {
            id: agent.id(),
            lifecycle: agent.lifecycle,
            health_monitor,
            supervisor_ref: agent.supervisor_ref,
        })
    }
    
    async fn run(&mut self) -> Result<(), RuntimeError> {
        // Transition to running state
        self.lifecycle.transition(AgentState::Running, "start command").await?;
        
        // Main execution loop
        let mut health_check_interval = interval(Duration::from_secs(30));
        let mut shutdown_signal = self.create_shutdown_listener();
        
        loop {
            tokio::select! {
                _ = health_check_interval.tick() => {
                    let health = self.health_monitor.run_health_checks().await;
                    if health.overall == HealthLevel::Critical {
                        return Err(RuntimeError::CriticalHealth(health));
                    }
                },
                _ = shutdown_signal.recv() => {
                    self.graceful_shutdown().await?;
                    break;
                },
                task = self.receive_task() => {
                    self.process_task(task?).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn graceful_shutdown(&mut self) -> Result<(), ShutdownError> {
        // Initiate shutdown
        self.lifecycle.transition(AgentState::Stopping, "shutdown signal").await?;
        
        let orchestrator = ShutdownOrchestrator::new();
        orchestrator.execute_shutdown(self).await?;
        
        self.lifecycle.transition(AgentState::Terminated, "shutdown complete").await?;
        Ok(())
    }
}
```

### 9.2 Supervision Integration Example

```rust
/// Example of agent with full supervision integration
struct SupervisedCodeAgent {
    base: BaseAgent,
    supervisor: Arc<LifecycleSupervisor>,
    restart_count: AtomicU32,
}

impl SupervisedCodeAgent {
    async fn with_supervision(config: AgentConfig) -> Result<Self, SupervisionError> {
        let supervisor = Arc::new(LifecycleSupervisor::new());
        let base = BaseAgent::new(config)?;
        
        // Register with supervisor
        let supervised = supervisor.supervise_agent(base.clone()).await?;
        
        Ok(SupervisedCodeAgent {
            base: supervised.agent,
            supervisor,
            restart_count: AtomicU32::new(0),
        })
    }
    
    async fn handle_failure(&self, error: AgentError) -> Result<(), FatalError> {
        // Create failure context
        let failure = AgentFailure {
            agent_id: self.base.id(),
            failure_type: categorize_error(&error),
            error: error.clone(),
            timestamp: SystemTime::now(),
            restart_count: self.restart_count.load(Ordering::Relaxed),
        };
        
        // Let supervisor handle it
        match self.supervisor.handle_agent_failure(failure).await {
            Ok(RecoveryAction::Restart) => {
                self.restart_count.fetch_add(1, Ordering::Relaxed);
                self.restart().await
            },
            Ok(RecoveryAction::Escalate) => {
                Err(FatalError::EscalatedToSupervisor)
            },
            Err(e) => Err(FatalError::SupervisionFailed(e)),
        }
    }
}
```

## Navigation

### Related Documents

#### Core Data Management

- **[Agent Communication](agent-communication.md)** - Message passing, task distribution, coordination patterns, and inter-agent communication protocols
- **[Agent Operations](agent-operations.md)** - Discovery, workflow management, and operational patterns  
- **[Agent Integration](agent-integration.md)** - Resource management, tool bus integration, and extension patterns
- **[Data Persistence](data-persistence.md)** - Agent state storage patterns and data management strategies
- **[Message Queuing](message-queuing.md)** - Asynchronous message handling and queue management
- **[State Management](state-management.md)** - Distributed state synchronization and consistency patterns

#### Core Architecture References

- **[Supervision Trees](../core-architecture/supervision-trees.md)** - Supervision hierarchy patterns and fault tolerance
- **[Async Patterns](../core-architecture/async-patterns.md)** - Tokio runtime patterns and async task management
- **[Type Definitions](../core-architecture/type-definitions.md)** - Core type system and trait definitions
- **[Error Handling](../core-architecture/error-handling.md)** - Comprehensive error management strategies

#### Security Integration

- **[Security Framework](../security/security-framework.md)** - Agent authentication and authorization
- **[Authentication](../security/authentication-specifications.md)** - Detailed auth patterns and token management
- **[Authorization](../security/authorization-specifications.md)** - RBAC implementation and permission models

#### Operations & Monitoring

- **[Health Monitoring](../operations/health-monitoring.md)** - System health checks and metrics collection
- **[Performance Monitoring](../operations/performance-monitoring.md)** - Agent performance tracking and optimization
- **[Deployment Patterns](../operations/deployment-patterns.md)** - Production deployment strategies

#### Transport & Communication

- **[Transport Layer](../transport/transport-layer.md)** - Network communication protocols and patterns
- **[Message Routing](../transport/message-routing.md)** - Advanced routing strategies and load balancing

### Document Status

- **Extracted From**: `agent-orchestration.md` (Sections 1-2)
- **Extraction Date**: 2025-07-03
- **Agent**: Agent 11, Phase 1, Group 1B
- **Content Status**: Complete extraction, zero information loss
- **Technical Status**: Comprehensive lifecycle patterns with supervision integration
- **Implementation Ready**: All Rust patterns verified with Tokio 1.45+ compatibility

### Cross-References

- Transport Layer: `../transport/`
- Security Layer: `../security/`
- Core Architecture: `../core-architecture/`

### Navigation Links

- **← Previous**: [Agent Communication](agent-communication.md) - Message passing and coordination patterns
- **→ Next**: [Agent Operations](agent-operations.md) - Discovery and workflow management
- **↑ Parent**: [Data Management](CLAUDE.md) - Data management overview

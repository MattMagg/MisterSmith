# Claude Code CLI Implementation Roadmap

## Technical Implementation Strategy for Mister Smith Framework

### Implementation Overview

This roadmap provides a systematic approach to integrating Claude Code CLI capabilities into the Mister Smith multi-agent framework.
The implementation is structured in phases that build upon each other while maintaining system stability.

### Framework Integration Points

**Core Architecture Dependencies**:

- [`ms-framework-docs/core-architecture/system-architecture.md`](../core-architecture/system-architecture.md) - System component integration
- [`ms-framework-docs/core-architecture/supervision-trees.md`](../core-architecture/supervision-trees.md) - Process supervision patterns
- [`ms-framework-docs/core-architecture/tokio-runtime.md`](../core-architecture/tokio-runtime.md) - Async runtime integration

**Transport Layer Dependencies**:

- [`ms-framework-docs/transport/nats-transport.md`](../transport/nats-transport.md) - NATS messaging patterns
- [`ms-framework-docs/transport/message-routing.md`](../transport/message-routing.md) - Message routing specifications

**Agent Management Dependencies**:

- [`ms-framework-docs/data-management/agent-orchestration.md`](../data-management/agent-orchestration.md) - Agent lifecycle management
- [`ms-framework-docs/data-management/agent-communication.md`](../data-management/agent-communication.md) - Inter-agent communication patterns

---

## Phase 1: Core CLI Integration Foundation

### Phase 1 Objective

Establish basic Claude CLI process management and NATS integration.

### Phase 1 Technical Deliverables

#### 1.1 Claude CLI Controller Implementation

**Location**: `src/claude_cli/controller.rs`

```rust
use crate::framework::{
    agent::{AgentId, AgentPool},
    config::ClaudeCliConfig,
    supervision::SupervisionTree,
    transport::nats::NatsClient,
};

pub struct ClaudeCliController {
    config: ClaudeCliConfig,
    active_sessions: Arc<RwLock<HashMap<AgentId, ClaudeSession>>>,
    agent_pool: AgentPool,
    nats_client: NatsClient,
    resource_manager: ResourceManager,
    supervision_tree: SupervisionTree,
}

impl ClaudeCliController {
    /// Create new Claude CLI agent instance with framework integration
    pub async fn spawn_agent(&self, agent_config: AgentConfig) -> Result<AgentId, ControllerError> {
        // Validate resources before spawning
        self.resource_manager.validate_spawn_request(&agent_config).await?;
        
        // Create supervised child process
        let child_spec = self.create_child_specification(&agent_config)?;
        let agent_id = self.supervision_tree.start_child(child_spec).await?;
        
        // Register with agent pool
        self.agent_pool.register_agent(agent_id.clone(), agent_config).await?;
        
        Ok(agent_id)
    }
    
    /// Graceful shutdown with cleanup and supervision tree integration
    pub async fn terminate_agent(&self, agent_id: &AgentId) -> Result<(), ControllerError> {
        // Signal graceful shutdown to supervision tree
        self.supervision_tree.terminate_child(agent_id).await?;
        
        // Cleanup resources
        self.resource_manager.cleanup_agent_resources(agent_id).await?;
        
        // Remove from active sessions
        self.active_sessions.write().await.remove(agent_id);
        
        Ok(())
    }
    
    /// Get agent status with health monitoring
    pub async fn get_agent_status(&self, agent_id: &AgentId) -> Result<AgentStatus, ControllerError> {
        let session = self.active_sessions.read().await
            .get(agent_id)
            .ok_or(ControllerError::AgentNotFound)?;
            
        Ok(AgentStatus {
            id: agent_id.clone(),
            health: session.health_status.clone(),
            resource_usage: self.resource_manager.get_agent_usage(agent_id).await?,
            uptime: session.created_at.elapsed(),
        })
    }
    
    /// List active agents with pool management
    pub async fn list_active_agents(&self) -> Result<Vec<AgentStatus>, ControllerError> {
        let sessions = self.active_sessions.read().await;
        let mut statuses = Vec::new();
        
        for (agent_id, _) in sessions.iter() {
            if let Ok(status) = self.get_agent_status(agent_id).await {
                statuses.push(status);
            }
        }
        
        Ok(statuses)
    }
}
```

**Framework Integration Pattern**:

```rust
// Integration with framework supervision tree
impl SupervisedChild for ClaudeCliAgent {
    async fn start(&mut self) -> Result<(), SupervisionError> {
        // Start Claude CLI process with hook configuration
        let mut cmd = Command::new("claude");
        cmd.args(&["--hooks", &self.hooks_config_path]);
        
        self.process = Some(cmd.spawn()?);
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), SupervisionError> {
        if let Some(process) = &mut self.process {
            // Send SIGTERM for graceful shutdown
            process.kill().await?;
        }
        Ok(())
    }
    
    async fn restart(&mut self) -> Result<(), SupervisionError> {
        self.stop().await?;
        self.start().await
    }
}
```

#### 1.2 Basic Hook Bridge Service

**Location**: `src/claude_cli/hook_bridge.rs`

```rust
use crate::framework::{
    transport::nats::{NatsClient, Subject},
    message::{HookMessage, HookResponse},
    timeout::TimeoutManager,
};

pub struct HookBridge {
    nats_client: NatsClient,
    hook_configs: Vec<HookConfig>,
    timeout_manager: TimeoutManager,
    subject_router: SubjectRouter,
}

impl HookBridge {
    /// Parse Claude CLI hook JSON and convert to framework message
    pub async fn process_hook_input(&self, hook_input: &str) -> Result<HookMessage, HookError> {
        // Parse JSON from Claude CLI hook
        let raw_hook: serde_json::Value = serde_json::from_str(hook_input)?;
        
        // Extract hook metadata
        let hook_type = raw_hook["hook_type"].as_str()
            .ok_or(HookError::MissingHookType)?;
        let agent_id = raw_hook["agent_id"].as_str()
            .ok_or(HookError::MissingAgentId)?;
        let timestamp = raw_hook["timestamp"].as_str()
            .ok_or(HookError::MissingTimestamp)?;
        
        // Create framework hook message
        let hook_message = HookMessage {
            hook_type: hook_type.parse()?,
            agent_id: agent_id.parse()?,
            timestamp: timestamp.parse()?,
            payload: raw_hook["payload"].clone(),
            metadata: self.extract_metadata(&raw_hook)?,
        };
        
        Ok(hook_message)
    }
    
    /// Route to appropriate NATS subjects based on hook type and agent
    pub async fn determine_nats_subject(&self, hook_message: &HookMessage) -> Result<Subject, HookError> {
        let base_subject = match hook_message.hook_type {
            HookType::Startup => "agent.lifecycle.startup",
            HookType::PreTask => "agent.task.pre_execution",
            HookType::PostTask => "agent.task.post_execution",
            HookType::OnError => "agent.error.notification",
            HookType::OnFileChange => "agent.file.change",
        };
        
        // Create agent-specific subject
        let subject = format!("{}.{}", base_subject, hook_message.agent_id);
        
        Ok(Subject::from(subject))
    }
    
    /// Process framework responses and convert to Claude CLI hook response
    pub async fn handle_hook_response(&self, response: HookResponse) -> Result<String, HookError> {
        let claude_response = match response.action {
            HookAction::Continue => json!({
                "decision": "continue",
                "message": response.message.unwrap_or_default()
            }),
            HookAction::Block => json!({
                "decision": "block",
                "message": response.message.unwrap_or("Operation blocked".to_string())
            }),
            HookAction::Approve => json!({
                "decision": "approve",
                "message": response.message.unwrap_or("Operation approved".to_string())
            }),
        };
        
        Ok(claude_response.to_string())
    }
}
```

**NATS Subject Integration Pattern**:

```rust
// Integration with framework NATS subject taxonomy
// Reference: ms-framework-docs/transport/nats-transport.md

pub struct SubjectRouter {
    base_subjects: HashMap<HookType, &'static str>,
}

impl SubjectRouter {
    pub fn new() -> Self {
        let mut base_subjects = HashMap::new();
        base_subjects.insert(HookType::Startup, "agent.lifecycle.startup");
        base_subjects.insert(HookType::PreTask, "agent.task.pre_execution");
        base_subjects.insert(HookType::PostTask, "agent.task.post_execution");
        base_subjects.insert(HookType::OnError, "agent.error.notification");
        base_subjects.insert(HookType::OnFileChange, "agent.file.change");
        
        Self { base_subjects }
    }
    
    pub fn route_hook(&self, hook_type: HookType, agent_id: &AgentId) -> Subject {
        let base = self.base_subjects.get(&hook_type)
            .unwrap_or(&"agent.unknown");
        Subject::from(format!("{}.{}", base, agent_id))
    }
}
```

#### 1.3 Configuration Management

**Location**: `config/claude-cli.toml`

```toml
[claude_cli]
# Agent Pool Configuration
max_concurrent_agents = 25
min_idle_agents = 2
agent_spawn_timeout = 30
agent_shutdown_timeout = 15

# Claude API Configuration
default_model = "claude-3-5-sonnet-20241022"
api_timeout = 300
max_retries = 3
backoff_multiplier = 2.0

# Hook System Configuration
hook_timeout = 60
hook_retry_attempts = 2
hook_response_buffer_size = 1024

# Output Processing
output_format = "stream-json"
output_buffer_size = 8192
stream_chunk_size = 1024

# Framework Integration
nats_connection_timeout = 10
nats_request_timeout = 30
supervision_restart_strategy = "one_for_one"
supervision_max_restarts = 3
supervision_max_seconds = 60

# Resource Management
max_memory_per_agent = "512MB"
max_cpu_per_agent = 0.5
resource_check_interval = 30
```

**Configuration Structure**:

```rust
// Framework-integrated configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCliConfig {
    pub agent_pool: AgentPoolConfig,
    pub api: ApiConfig,
    pub hooks: HookConfig,
    pub output: OutputConfig,
    pub framework: FrameworkConfig,
    pub resources: ResourceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    pub nats_connection_timeout: Duration,
    pub nats_request_timeout: Duration,
    pub supervision_restart_strategy: SupervisionStrategy,
    pub supervision_max_restarts: u32,
    pub supervision_max_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_memory_per_agent: ByteSize,
    pub max_cpu_per_agent: f64,
    pub resource_check_interval: Duration,
}
```

#### 1.4 Hook Bridge Scripts

**Location**: `scripts/nats-hook-bridge`

```bash
#!/bin/bash
# Bridge Claude CLI hooks to NATS subjects
HOOK_TYPE="$1"
INPUT=$(cat)
AGENT_ID=$(echo "$INPUT" | jq -r '.session_info.agent_id // "unknown"')
SUBJECT="agent.${AGENT_ID}.${HOOK_TYPE}"
echo "$INPUT" | nats pub "$SUBJECT" --stdin
```

### Phase 1 Framework Modifications

#### Update System Architecture

**File**: [`ms-framework-docs/core-architecture/system-architecture.md`](../core-architecture/system-architecture.md)

Add Claude CLI Controller as new component in architecture diagram and component descriptions:

```rust
// System Architecture Component Registration
pub struct SystemArchitecture {
    // ... existing components
    claude_cli_controller: ClaudeCliController,
    // ... existing components
}

// Component Dependencies
impl SystemArchitecture {
    pub fn new() -> Self {
        Self {
            claude_cli_controller: ClaudeCliController::new(
                nats_client,
                supervision_tree,
                resource_manager,
            ),
        }
    }
}
```

#### Update Transport Layer

**File**: [`ms-framework-docs/transport/nats-transport.md`](../transport/nats-transport.md)

Integrate Claude CLI hook subjects with existing NATS subject taxonomy:

```
# Extended NATS Subject Taxonomy for Claude CLI Integration

## Hook Event Subjects
agent.lifecycle.startup.{agent_id}        # Agent startup hooks
agent.task.pre_execution.{agent_id}       # Pre-task execution hooks
agent.task.post_execution.{agent_id}      # Post-task execution hooks
agent.error.notification.{agent_id}       # Error notification hooks
agent.file.change.{agent_id}             # File change detection hooks

## Hook Response Subjects
agent.lifecycle.startup.{agent_id}.response
agent.task.pre_execution.{agent_id}.response
agent.task.post_execution.{agent_id}.response
agent.error.notification.{agent_id}.response
agent.file.change.{agent_id}.response

## Claude CLI Control Subjects
claude_cli.agent.spawn.{agent_id}         # Agent spawn requests
claude_cli.agent.terminate.{agent_id}     # Agent termination requests
claude_cli.agent.status.{agent_id}        # Agent status queries
claude_cli.pool.status                    # Pool status queries
```

#### Update Supervision Trees

**File**: [`ms-framework-docs/core-architecture/supervision-trees.md`](../core-architecture/supervision-trees.md)

Add Claude CLI agent supervision patterns:

```rust
// Claude CLI Agent Supervision Strategy
pub struct ClaudeCliSupervisor {
    strategy: SupervisionStrategy,
    max_restarts: u32,
    max_seconds: u64,
}

impl SupervisionStrategy for ClaudeCliSupervisor {
    async fn handle_child_failure(&self, child_id: &AgentId, error: &SupervisionError) -> SupervisionAction {
        match error {
            SupervisionError::ProcessCrash => SupervisionAction::Restart,
            SupervisionError::ResourceExhaustion => SupervisionAction::RestartWithDelay(Duration::from_secs(30)),
            SupervisionError::ConfigurationError => SupervisionAction::Terminate,
            _ => SupervisionAction::Restart,
        }
    }
}
```

### Phase 1 Success Criteria

- Claude CLI instances can be spawned and terminated
- Basic hook events are published to NATS subjects
- Configuration system is functional
- Resource limits are enforced

---

## Phase 2: Hook System Integration

### Phase 2 Objective

Complete hook system integration with comprehensive error handling and timeout management.

### Phase 2 Technical Deliverables

#### 2.1 Enhanced Hook Bridge

**Location**: `src/claude_cli/hook_bridge.rs`

```rust
use crate::framework::{
    transport::nats::{NatsClient, Subject, Message},
    message::{HookEvent, FrameworkResponse},
    error::HookError,
    timeout::TimeoutManager,
};

impl HookBridge {
    /// Enhanced hook processing with decision control and framework integration
    pub async fn process_hook_with_decision(&self, hook_input: HookInput) -> Result<HookResponse, HookError> {
        // Parse and validate hook input against framework schemas
        let event = self.parse_and_validate_hook(hook_input)?;
        
        // Publish to NATS and wait for framework response
        let response = self.publish_and_await_response(&event).await?;
        
        // Convert framework response to Claude CLI hook response
        self.convert_to_hook_response(response)
    }
    
    /// Timeout management for hook execution with circuit breaker pattern
    async fn publish_and_await_response(&self, event: &HookEvent) -> Result<FrameworkResponse, HookError> {
        let subject = self.determine_nats_subject(event);
        let response_subject = format!("{}.response", subject);
        
        // Check circuit breaker state
        if !self.circuit_breaker.is_closed(&subject) {
            return Err(HookError::CircuitBreakerOpen);
        }
        
        // Subscribe to response subject with framework message handling
        let mut response_sub = self.nats_client.subscribe(&response_subject).await?;
        
        // Create framework message with correlation ID
        let correlation_id = self.generate_correlation_id();
        let framework_message = self.create_framework_message(event, &correlation_id)?;
        
        // Publish hook event to framework
        self.nats_client.publish(&subject, &framework_message).await?;
        
        // Wait for response with timeout and correlation ID validation
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.hook_timeout),
            self.wait_for_correlated_response(&mut response_sub, &correlation_id)
        ).await;
        
        match result {
            Ok(Ok(response)) => {
                self.circuit_breaker.record_success(&subject);
                Ok(response)
            },
            Ok(Err(e)) => {
                self.circuit_breaker.record_failure(&subject);
                Err(e)
            },
            Err(_) => {
                self.circuit_breaker.record_failure(&subject);
                Err(HookError::TimeoutError)
            }
        }
    }
    
    /// Wait for correlated response with framework message validation
    async fn wait_for_correlated_response(
        &self,
        response_sub: &mut async_nats::Subscriber,
        correlation_id: &str
    ) -> Result<FrameworkResponse, HookError> {
        while let Some(msg) = response_sub.next().await {
            let framework_msg: Message = serde_json::from_slice(&msg.payload)?;
            
            // Validate correlation ID
            if framework_msg.correlation_id == correlation_id {
                return Ok(framework_msg.payload.into());
            }
            
            // Log unexpected message
            log::warn!("Received uncorrelated message: {}", framework_msg.correlation_id);
        }
        
        Err(HookError::NoResponse)
    }
    
    /// Create framework message with proper metadata
    fn create_framework_message(&self, event: &HookEvent, correlation_id: &str) -> Result<Message, HookError> {
        Ok(Message {
            correlation_id: correlation_id.to_string(),
            message_type: MessageType::HookEvent,
            timestamp: Utc::now(),
            payload: serde_json::to_value(event)?,
            metadata: self.create_message_metadata(event)?,
        })
    }
    
    /// Generate correlation ID for request-response tracking
    fn generate_correlation_id(&self) -> String {
        format!("{}-{}", self.instance_id, uuid::Uuid::new_v4())
    }
}
```

**Circuit Breaker Integration**:

```rust
// Circuit breaker pattern for hook reliability
pub struct HookCircuitBreaker {
    failure_threshold: u32,
    timeout_threshold: Duration,
    half_open_timeout: Duration,
    states: Arc<RwLock<HashMap<String, CircuitState>>>,
}

#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

impl HookCircuitBreaker {
    pub fn is_closed(&self, subject: &str) -> bool {
        let states = self.states.read().unwrap();
        match states.get(subject) {
            Some(CircuitState::Closed) | None => true,
            Some(CircuitState::Open { opened_at }) => {
                opened_at.elapsed() > self.half_open_timeout
            },
            Some(CircuitState::HalfOpen) => true,
        }
    }
    
    pub fn record_success(&self, subject: &str) {
        let mut states = self.states.write().unwrap();
        states.insert(subject.to_string(), CircuitState::Closed);
    }
    
    pub fn record_failure(&self, subject: &str) {
        let mut states = self.states.write().unwrap();
        states.insert(subject.to_string(), CircuitState::Open { 
            opened_at: Instant::now() 
        });
    }
}
```

#### 2.2 Hook Configuration Management

**Location**: `.claude/hooks.json`

```json
{
  "PreToolUse": [
    {
      "matcher": "Task",
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge pre_task",
          "timeout": 30,
          "retry_attempts": 2,
          "retry_backoff": 5,
          "required": true
        }
      ]
    },
    {
      "matcher": "Edit|Write|MultiEdit",
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge pre_file_operation",
          "timeout": 15,
          "retry_attempts": 1,
          "required": false
        }
      ]
    }
  ],
  "PostToolUse": [
    {
      "matcher": ".*",
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge post_task",
          "timeout": 30,
          "retry_attempts": 2,
          "retry_backoff": 5,
          "required": false
        }
      ]
    }
  ],
  "OnError": [
    {
      "matcher": ".*",
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge error_notification",
          "timeout": 10,
          "retry_attempts": 3,
          "retry_backoff": 2,
          "required": false
        }
      ]
    }
  ],
  "OnFileChange": [
    {
      "matcher": ".*",
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge file_change",
          "timeout": 5,
          "retry_attempts": 1,
          "required": false
        }
      ]
    }
  ]
}
```

**Hook Configuration Validation**:

```rust
// Hook configuration validation and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfiguration {
    pub pre_tool_use: Vec<HookMatcher>,
    pub post_tool_use: Vec<HookMatcher>,
    pub on_error: Vec<HookMatcher>,
    pub on_file_change: Vec<HookMatcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMatcher {
    pub matcher: String,
    pub hooks: Vec<HookSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSpec {
    pub r#type: String,
    pub command: String,
    pub timeout: u64,
    pub retry_attempts: u32,
    pub retry_backoff: u64,
    pub required: bool,
}

impl HookConfiguration {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate all matchers are valid regex
        for matcher in &self.pre_tool_use {
            regex::Regex::new(&matcher.matcher)
                .map_err(|e| ConfigError::InvalidMatcher(e.to_string()))?;
        }
        
        // Validate timeout values
        for matcher in &self.post_tool_use {
            for hook in &matcher.hooks {
                if hook.timeout == 0 {
                    return Err(ConfigError::InvalidTimeout);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_matching_hooks(&self, tool_name: &str, phase: HookPhase) -> Vec<&HookSpec> {
        let matchers = match phase {
            HookPhase::PreToolUse => &self.pre_tool_use,
            HookPhase::PostToolUse => &self.post_tool_use,
            HookPhase::OnError => &self.on_error,
            HookPhase::OnFileChange => &self.on_file_change,
        };
        
        matchers.iter()
            .filter(|matcher| {
                regex::Regex::new(&matcher.matcher)
                    .map(|re| re.is_match(tool_name))
                    .unwrap_or(false)
            })
            .flat_map(|matcher| &matcher.hooks)
            .collect()
    }
}
```

#### 2.3 Error Handling and Recovery

**Location**: `src/claude_cli/error_handling.rs`

```rust
pub enum HookError {
    ParseError(serde_json::Error),
    NatsError(async_nats::Error),
    TimeoutError,
    ValidationError(String),
    NoResponse,
}

impl HookBridge {
    async fn handle_hook_error(&self, error: HookError, event: &HookEvent) -> HookResponse {
        // Log error for observability
        self.metrics.increment_hook_errors();
        
        match error {
            HookError::TimeoutError => {
                // Allow execution to continue on timeout
                HookResponse::Continue
            },
            HookError::ValidationError(_) => {
                // Block execution on validation errors
                HookResponse::Block("Validation failed".to_string())
            },
            _ => {
                // Default to continue for other errors
                HookResponse::Continue
            }
        }
    }
}
```

### Phase 2 Framework Modifications

#### Update Observability Framework

**File**: `ms-framework-docs/observability/observability-monitoring-framework.md`

Add hook execution metrics:

- Hook processing latency
- Hook success/failure rates
- Hook timeout occurrences
- NATS message delivery metrics

### Phase 2 Success Criteria

- All 5 hook types (startup, pre_task, post_task, on_error, on_file_change) are functional
- Hook responses can control Claude CLI execution (approve/block/continue)
- Error handling prevents system failures
- Timeout management prevents hanging hooks

---

## Phase 3: Parallel Execution Enhancement

### Phase 3 Objective

Implement robust parallel task coordination and resource management for 25-30 concurrent agents.

### Phase 3 Technical Deliverables

#### 3.1 Task Output Parser

**Location**: `src/claude_cli/task_output_parser.rs`

```rust
pub struct TaskOutputParser {
    task_regex: Regex,
    nats_client: async_nats::Client,
    metrics: ParsingMetrics,
}

impl TaskOutputParser {
    pub async fn process_output_stream(&self, mut stream: Lines<BufReader<ChildStdout>>) -> Result<(), ParseError> {
        while let Some(line) = stream.next_line().await? {
            if let Some(task_info) = self.extract_task_info(&line) {
                self.route_task_output(task_info, &line).await?;
            } else {
                self.route_general_output(&line).await?;
            }
        }
        Ok(())
    }
    
    fn extract_task_info(&self, line: &str) -> Option<TaskInfo> {
        // Handle multiple output formats:
        // "● Task(Patch Agent 1)" -> TaskInfo::AgentId(1)
        // "● Task(Explore backend)" -> TaskInfo::Description("Explore backend")
        self.task_regex.captures(line).map(|caps| {
            let identifier = caps.get(1).unwrap().as_str();
            if let Ok(id) = identifier.parse::<u32>() {
                TaskInfo::AgentId(id)
            } else {
                TaskInfo::Description(identifier.to_string())
            }
        })
    }
}
```

#### 3.2 Agent Pool Management

**Location**: `src/claude_cli/agent_pool.rs`

```rust
pub struct AgentPool {
    max_size: usize,
    active_agents: Arc<RwLock<HashMap<AgentId, AgentPoolEntry>>>,
    spawn_semaphore: Arc<Semaphore>,
    health_checker: HealthChecker,
    metrics: PoolMetrics,
}

impl AgentPool {
    pub async fn register_with_health_check(&self, session: ClaudeSession) -> Result<AgentId, PoolError> {
        // Acquire spawn permit (blocks if at capacity)
        let permit = self.spawn_semaphore.acquire().await?;
        
        // Register agent
        let agent_id = session.agent_id.clone();
        let entry = AgentPoolEntry {
            session,
            registered_at: Instant::now(),
            permit,
            health_status: HealthStatus::Healthy,
        };
        
        self.active_agents.write().await.insert(agent_id.clone(), entry);
        
        // Start health monitoring
        self.health_checker.start_monitoring(agent_id.clone()).await?;
        
        Ok(agent_id)
    }
    
    pub async fn cleanup_unhealthy_agents(&self) -> Result<Vec<AgentId>, PoolError> {
        let unhealthy_agents = self.health_checker.get_unhealthy_agents().await?;
        
        for agent_id in &unhealthy_agents {
            self.force_terminate_agent(agent_id).await?;
        }
        
        Ok(unhealthy_agents)
    }
}
```

#### 3.3 Resource Management

**Location**: `src/claude_cli/resource_manager.rs`

```rust
pub struct ResourceManager {
    memory_monitor: MemoryMonitor,
    cpu_monitor: CpuMonitor,
    network_monitor: NetworkMonitor,
    limits: ResourceLimits,
}

impl ResourceManager {
    pub async fn validate_spawn_request(&self, request: &SpawnRequest) -> Result<(), ResourceError> {
        // Check system resource availability
        let current_usage = self.get_current_usage().await?;
        let projected_usage = self.project_usage_with_new_agent(&current_usage, request)?;
        
        // Validate against limits
        if projected_usage.memory > self.limits.max_memory {
            return Err(ResourceError::MemoryLimitExceeded);
        }
        
        if projected_usage.cpu > self.limits.max_cpu {
            return Err(ResourceError::CpuLimitExceeded);
        }
        
        Ok(())
    }
    
    pub async fn monitor_resource_usage(&self) -> Result<(), ResourceError> {
        // Continuous monitoring loop
        loop {
            let usage = self.get_current_usage().await?;
            
            // Check for resource pressure
            if usage.memory > self.limits.warning_threshold_memory {
                self.trigger_memory_pressure_response().await?;
            }
            
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}
```

### Phase 3 Framework Modifications

#### Update Agent Orchestration

**File**: `ms-framework-docs/data-management/agent-orchestration.md`

Enhanced parallel coordination patterns already added.

#### Update Deployment Architecture

**File**: `ms-framework-docs/deployment/deployment-architecture-specifications.md`

Add scaling patterns for 25-30 concurrent Claude CLI agents:

- Resource allocation strategies
- Load balancing patterns
- Performance optimization guidelines

### Phase 3 Success Criteria

- 25-30 concurrent Claude CLI agents can be sustained
- Task output is correctly parsed and routed
- Resource limits prevent system overload
- Health monitoring detects and recovers from failures

---

## Phase 4: MCP Integration

### Phase 4 Objective

Integrate Model Context Protocol servers for extended tool capabilities.

### Phase 4 Technical Deliverables

#### 4.1 MCP Server Integration

**Location**: `src/claude_cli/mcp_integration.rs`

```rust
pub struct McpIntegration {
    mcp_config: McpConfig,
    server_registry: McpServerRegistry,
    tool_mapper: ToolMapper,
}

impl McpIntegration {
    pub async fn register_mcp_server(&mut self, server_config: McpServerConfig) -> Result<ServerId, McpError> {
        // Start MCP server process
        let server = McpServer::start(server_config).await?;
        
        // Register tools with framework tool registry
        let tools = server.list_tools().await?;
        for tool in tools {
            self.tool_mapper.register_mcp_tool(tool, server.id()).await?;
        }
        
        // Add to server registry
        let server_id = self.server_registry.register(server).await?;
        
        Ok(server_id)
    }
    
    pub async fn handle_mcp_tool_call(&self, tool_name: &str, args: serde_json::Value) -> Result<ToolResult, McpError> {
        // Parse MCP tool name: "mcp__server__tool"
        let (server_name, tool_name) = self.parse_mcp_tool_name(tool_name)?;
        
        // Route to appropriate MCP server
        let server = self.server_registry.get_server(&server_name)?;
        server.call_tool(tool_name, args).await
    }
}
```

#### 4.2 Tool Registry Enhancement

**Location**: `src/tools/tool_registry.rs`

```rust
impl ToolRegistry {
    pub async fn register_mcp_tools(&mut self, server_id: ServerId, tools: Vec<McpTool>) -> Result<(), RegistryError> {
        for tool in tools {
            let tool_id = format!("mcp__{}__{}", server_id, tool.name);
            let registry_entry = ToolRegistryEntry {
                id: tool_id,
                name: tool.name,
                description: tool.description,
                provider: ToolProvider::Mcp(server_id),
                schema: tool.input_schema,
            };
            
            self.tools.insert(registry_entry.id.clone(), registry_entry);
        }
        
        Ok(())
    }
}
```

### Phase 4 Framework Modifications

#### Create MCP Integration Specifications

**File**: `ms-framework-docs/integration/mcp-integration-specifications.md`

New document defining:

- MCP server lifecycle management
- Tool naming conventions
- Permission system integration
- Error handling patterns

### Phase 4 Success Criteria

- MCP servers can be registered and managed
- MCP tools are available through framework tool registry
- Tool calls are correctly routed to MCP servers
- Permission system controls MCP tool access

---

## Phase 5: Advanced Features and Optimization

### Phase 5 Objective

Implement advanced coordination patterns and performance optimizations.

### Phase 5 Technical Deliverables

#### 5.1 Advanced Coordination Patterns

- Hierarchical task decomposition
- Dynamic load balancing
- Adaptive resource allocation
- Circuit breaker patterns

#### 5.2 Performance Optimization

- Connection pooling for Anthropic API
- Shared memory for common data
- Optimized NATS message routing
- Garbage collection coordination

#### 5.3 Advanced Technical Features

- Comprehensive operation logging and tracing
- Advanced permission and access control patterns
- Process isolation and resource sandboxing
- System state persistence and recovery procedures

### Phase 5 Success Criteria

- System can handle peak loads efficiently
- Advanced coordination patterns are functional
- Technical features meet security requirements
- Performance metrics meet target thresholds
- System demonstrates production-ready stability patterns

---

## Implementation Dependencies

### External Dependencies

- Claude Code CLI binary in system PATH
- Anthropic API key configuration
- NATS server running and accessible
- Rust toolchain with required crates

### Framework Dependencies

- Existing NATS messaging infrastructure
- Tokio supervision tree patterns
- Agent lifecycle management
- Configuration management system

### Resource Requirements

#### System Resources

- **Memory**: 8-16GB system memory (2-4GB for framework, 4-12GB for Claude CLI agents)
- **CPU**: 4-8 CPU cores (concurrent agent processing)
- **Storage**: 1-2GB for logs, configurations, and temporary files
- **Network**: Stable internet connectivity for Anthropic API calls

#### Framework Integration Resources

- **NATS Server**: Minimum 1GB memory, 2 CPU cores
- **PostgreSQL**: 2-4GB memory for agent state persistence
- **Tokio Runtime**: Configured for async I/O intensive workloads

#### Performance Targets

- **Agent Spawn Time**: < 5 seconds per agent
- **Hook Response Time**: < 100ms for framework integration
- **Message Throughput**: 1000+ messages/second NATS capacity
- **Resource Utilization**: < 80% CPU, < 90% memory under normal load

---

## Technical Integration Validation

### Framework Compatibility Checklist

- [ ] **Supervision Tree Integration**: Claude CLI processes managed by framework supervision
- [ ] **NATS Transport Integration**: Hook events routed through framework message bus
- [ ] **Resource Management**: Agent resource usage monitored and controlled
- [ ] **Configuration Management**: Framework configuration system extended for Claude CLI
- [ ] **Error Handling**: Framework error patterns applied to Claude CLI operations
- [ ] **Observability**: Framework monitoring extended to Claude CLI metrics

### Implementation Validation Commands

```bash
# Validate framework integration
cargo test --test claude_cli_integration

# Verify NATS subject routing
nats sub "agent.*.>" --count=10

# Check resource monitoring
cargo run --bin resource_monitor -- --component claude_cli

# Validate hook configuration
claudeconfig validate --hooks .claude/hooks.json

# Test agent pool management
cargo run --bin agent_pool_test -- --max-agents 25
```

This roadmap provides a systematic approach to Claude Code CLI integration while maintaining system stability and performance throughout the implementation process. The technical specifications ensure full integration with the Mister Smith framework architecture.

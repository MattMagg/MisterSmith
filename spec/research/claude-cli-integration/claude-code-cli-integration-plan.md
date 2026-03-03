# Claude Code CLI Integration Plan

## Technical Integration Strategy for Mister Smith Framework

### Overview

This document provides technical integration specifications for incorporating Claude Code CLI capabilities into the Mister Smith multi-agent framework. The integration leverages existing framework architecture while adding minimal new components.

**Cross-References:**

- [System Architecture](../../core-architecture/system-architecture.md)
- [Agent Orchestration](../../data-management/agent-orchestration.md)
- [NATS Transport](../../transport/nats-transport.md)
- [Component Architecture](../../core-architecture/component-architecture.md)

---

## Integration Architecture

### 1. Component Integration Map

```text
Mister Smith Framework (Rust)
├── NATS Server (existing messaging backbone)
│   └── Subjects: control.*, agent.*, ctx.*
├── Agent Supervisor (existing Tokio-based)
│   └── Supervision Tree Integration
├── Claude CLI Controller (NEW COMPONENT)
│   ├── Agent Pool Manager
│   │   └── Resource Allocation & Lifecycle
│   ├── Hook Bridge Service
│   │   └── NATS Event Publishing
│   └── Output Parser Service
│       └── Stream Processing & Routing
├── Hook Bridge Scripts (NEW COMPONENT)
│   ├── NATS Publishers
│   │   └── JSON Event Serialization
│   └── JSON Processors
│       └── Hook Input Validation
└── Claude CLI Instances (1-25)
    ├── claude --output-format=stream-json
    │   └── Streaming Output Processing
    ├── Hook Scripts (NATS integration)
    │   └── Bidirectional Communication
    └── MCP Servers (optional)
        └── Extended Tool Registry
```

### 2. Data Flow Architecture

```text
User Request → Framework → Claude CLI Controller → Claude CLI Instance
                ↓                                        ↓
         NATS Subjects ← Hook Bridge ← Claude Hooks ← Task Execution
                ↓                                        ↓
         Agent Orchestrator ← Output Parser ← Task Output Stream
```

---

## Required Framework Modifications

### 1. New Components to Add

#### A. Claude CLI Controller

**Purpose**: Manage Claude CLI instance lifecycle and resource allocation

**Integration Points:**

- [Agent Orchestration](../../data-management/agent-orchestration.md)
- [Supervision Trees](../../core-architecture/supervision-trees.md)
- [NATS Transport](../../transport/nats-transport.md)

**Implementation**:

```rust
pub struct ClaudeCliController {
    config: ClaudeCliConfig,
    active_sessions: HashMap<AgentId, ClaudeSession>,
    agent_pool: AgentPool,
    nats_client: async_nats::Client,
    hook_bridge: Arc<HookBridge>,
    metrics: ControllerMetrics,
}

impl ClaudeCliController {
    pub async fn spawn_agent(&mut self, request: SpawnRequest) -> Result<AgentId, SpawnError> {
        // Resource validation
        self.validate_resources()?;
        
        // Build Claude CLI command
        let command = self.build_command(&request)?;
        
        // Spawn process with supervision
        let session = ClaudeSession::spawn(command, &self.nats_client).await?;
        
        // Register with agent pool
        let agent_id = self.agent_pool.register(session).await?;
        
        Ok(agent_id)
    }
    
    pub async fn terminate_agent(&mut self, agent_id: AgentId) -> Result<(), TerminateError> {
        // Graceful shutdown with cleanup
        if let Some(session) = self.active_sessions.remove(&agent_id) {
            session.terminate_gracefully().await?;
            self.agent_pool.unregister(agent_id).await?;
        }
        Ok(())
    }
}
```

#### B. Hook Bridge Service

**Purpose**: Bridge Claude Code hooks to NATS messaging system

**Integration Points:**

- [NATS Transport](../../transport/nats-transport.md)
- [Message Schemas](../../data-management/message-schemas.md)
- [Connection Management](../../data-management/connection-management.md)

**Implementation**:

```rust
pub struct HookBridge {
    nats_client: async_nats::Client,
    hook_configs: Vec<HookConfig>,
    timeout_manager: TimeoutManager,
}

impl HookBridge {
    pub async fn process_hook_event(&self, hook_input: HookInput) -> Result<HookResponse, HookError> {
        // Parse hook JSON input
        let event = self.parse_hook_input(hook_input)?;
        
        // Publish to appropriate NATS subject
        let subject = self.determine_subject(&event);
        self.nats_client.publish(subject, event.to_json()).await?;
        
        // Return appropriate response
        Ok(HookResponse::Continue)
    }
    
    fn determine_subject(&self, event: &HookEvent) -> String {
        match event.hook_type {
            HookType::Startup => "control.startup".to_string(),
            HookType::PreTask => format!("agent.{}.pre", event.agent_id),
            HookType::PostTask => format!("agent.{}.post", event.agent_id),
            HookType::OnError => format!("agent.{}.error", event.agent_id),
            HookType::OnFileChange => format!("ctx.{}.file_change", event.context_id),
        }
    }
}
```

#### C. Task Output Parser

**Purpose**: Parse Claude CLI task output and route to NATS subjects

**Integration Points:**

- [Message Framework](../../data-management/message-framework.md)
- [Data Integration Patterns](../../data-management/data-integration-patterns.md)
- [Agent Communication](../../data-management/agent-communication.md)

**Implementation**:

```rust
pub struct TaskOutputParser {
    agent_id_extractor: Regex,
    output_router: OutputRouter,
    stream_processor: StreamProcessor,
}

impl TaskOutputParser {
    pub async fn process_output_stream(&self, stream: OutputStream) -> Result<(), ParseError> {
        let mut lines = stream.lines();
        
        while let Some(line) = lines.next().await {
            let line = line?;
            
            // Extract agent ID from task output
            if let Some(agent_id) = self.extract_agent_id(&line) {
                // Route to appropriate NATS subject
                let subject = format!("agents.{}.output", agent_id);
                self.output_router.publish(subject, &line).await?;
            }
        }
        
        Ok(())
    }
    
    fn extract_agent_id(&self, line: &str) -> Option<u32> {
        self.agent_id_extractor
            .captures(line)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}
```

### 2. Existing Components to Enhance

#### A. Agent Orchestration Enhancement

**Integration Points:**

- [Agent Orchestration](../../data-management/agent-orchestration.md)
- [Agent Lifecycle](../../data-management/agent-lifecycle.md)
- [Supervision Trees](../../core-architecture/supervision-trees.md)

**Additions Required**:

```rust
// Add to existing AgentOrchestrator
impl AgentOrchestrator {
    pub async fn spawn_claude_cli_agent(&mut self, request: ClaudeAgentRequest) -> Result<AgentId, OrchestrationError> {
        // Delegate to Claude CLI Controller
        let agent_id = self.claude_cli_controller.spawn_agent(request).await?;
        
        // Register with existing supervision tree
        self.supervision_tree.add_agent(agent_id, AgentType::ClaudeCli).await?;
        
        Ok(agent_id)
    }
    
    pub async fn coordinate_parallel_tasks(&mut self, tasks: Vec<TaskRequest>) -> Result<Vec<TaskResult>, CoordinationError> {
        // Use Claude CLI Task tool for parallel execution
        let task_requests = tasks.into_iter()
            .map(|task| self.build_claude_task_request(task))
            .collect();
            
        // Spawn parallel Claude CLI agents
        let agent_ids = self.spawn_parallel_agents(task_requests).await?;
        
        // Coordinate execution and collect results
        self.coordinate_execution(agent_ids).await
    }
}
```

#### B. Transport Layer Enhancement

**Integration Points:**

- [NATS Transport](../../transport/nats-transport.md)
- [Message Schemas](../../data-management/message-schemas.md)
- [Connection Management](../../data-management/connection-management.md)

**Hook Integration Subjects** (already defined, document usage):

```text
control.startup               # Claude CLI initialization
agent.{id}.pre               # Pre-task hook processing
agent.{id}.post              # Post-task hook processing  
agent.{id}.error             # Error hook handling
agent.{id}.hook_response     # Hook mutation responses
ctx.{gid}.file_change        # File change notifications
```

**New Message Formats**:

```rust
#[derive(Serialize, Deserialize)]
pub struct HookEvent {
    pub hook_type: HookType,
    pub agent_id: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub session_info: SessionInfo,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskOutputEvent {
    pub agent_id: u32,
    pub output_line: String,
    pub task_status: TaskStatus,
    pub timestamp: DateTime<Utc>,
}
```

---

## Configuration Management

### 1. Claude CLI Configuration

**File**: `config/claude-cli.toml`

```toml
[claude_cli]
max_concurrent_agents = 25
default_model = "claude-3-5-sonnet-20241022"
api_timeout = 300
retry_attempts = 3
hook_timeout = 60
output_buffer_size = 8192

[claude_cli.spawn_limits]
rate_limit_per_minute = 10
memory_limit_mb = 200
cpu_limit_percent = 50

[claude_cli.hooks]
config_path = ".claude/hooks.json"
enable_nats_bridge = true
hook_execution_timeout = 30

[claude_cli.mcp]
enable_mcp_integration = true
mcp_config_path = ".claude/mcp.json"
```

### 2. Hook Configuration

**File**: `.claude/hooks.json`

```json
{
  "PreToolUse": [
    {
      "matcher": "Task",
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge pre_task",
          "timeout": 30
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
          "timeout": 30
        }
      ]
    }
  ],
  "Notification": [
    {
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge notification",
          "timeout": 10
        }
      ]
    }
  ],
  "Stop": [
    {
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge stop",
          "timeout": 30
        }
      ]
    }
  ],
  "SubagentStop": [
    {
      "hooks": [
        {
          "type": "command",
          "command": "nats-hook-bridge subagent_stop",
          "timeout": 30
        }
      ]
    }
  ]
}
```

### 3. Hook Bridge Scripts

**File**: `scripts/nats-hook-bridge`

```bash
#!/bin/bash
# NATS Hook Bridge Script for Claude Code CLI Integration

HOOK_TYPE="$1"
NATS_URL="${NATS_URL:-nats://localhost:4222}"

# Read JSON input from stdin
INPUT=$(cat)

# Extract agent ID from session info
AGENT_ID=$(echo "$INPUT" | jq -r '.session_info.agent_id // "unknown"')

# Determine NATS subject
case "$HOOK_TYPE" in
    "pre_task")
        SUBJECT="agent.${AGENT_ID}.pre"
        ;;
    "post_task")
        SUBJECT="agent.${AGENT_ID}.post"
        ;;
    "notification")
        SUBJECT="control.notification"
        ;;
    "stop")
        SUBJECT="agent.${AGENT_ID}.stop"
        ;;
    "subagent_stop")
        SUBJECT="agent.${AGENT_ID}.subagent_stop"
        ;;
    *)
        SUBJECT="control.unknown_hook"
        ;;
esac

# Publish to NATS
echo "$INPUT" | nats pub "$SUBJECT" --stdin

# Return success (continue execution)
exit 0
```

---

## Implementation Sequence

### Phase 1: Core CLI Integration

**Dependencies**: None

**Components**:

1. **Claude CLI Controller implementation**
   - Agent lifecycle management
   - Resource allocation system
   - Process supervision integration

2. **Basic hook bridge for NATS integration**
   - Hook event capture
   - NATS message publishing
   - Error handling and recovery

3. **Task output parsing and routing**
   - Stream processing implementation
   - Message routing logic
   - Output format standardization

4. **Configuration management**
   - TOML configuration parsing
   - Environment variable handling
   - Validation and defaults

**Framework Files to Create/Modify**:

- Create: `core-architecture/claude-cli-integration.md`
- Modify: `core-architecture/system-architecture.md`
- Modify: `transport/nats-transport.md`
- Create: `config/claude-cli.toml`

### Phase 2: Hook System Integration

**Dependencies**: Phase 1

**Components**:

1. **Complete hook bridge implementation**
   - Full hook type coverage
   - Bidirectional communication
   - State synchronization

2. **JSON message format standardization**
   - Schema validation
   - Serialization/deserialization
   - Version compatibility

3. **Hook configuration management**
   - Dynamic hook registration
   - Configuration hot-reload
   - Hook script validation

4. **Error handling and timeout management**
   - Graceful degradation
   - Retry mechanisms
   - Circuit breaker patterns

**Framework Files to Modify**:

- Enhance: `transport/nats-transport.md`
- Modify: `operations/observability-monitoring-framework.md`
- Create: `scripts/nats-hook-bridge`

### Phase 3: Parallel Execution Enhancement

**Dependencies**: Phase 1, 2

**Components**:

1. **Multi-agent coordination patterns**
   - Task distribution algorithms
   - Result aggregation
   - Dependency management

2. **Resource pool management**
   - Agent pool sizing
   - Load balancing
   - Resource contention handling

3. **Performance optimization**
   - Connection pooling
   - Message batching
   - Memory optimization

**Framework Files to Modify**:

- Enhance: `data-management/agent-orchestration.md`
- Modify: `operations/deployment-architecture-specifications.md`
- Enhance: `operations/observability-monitoring-framework.md`

### Phase 4: MCP Integration

**Dependencies**: Phase 1

**Components**:

1. **MCP server integration**
   - MCP protocol implementation
   - Server lifecycle management
   - Tool capability discovery

2. **Tool registry enhancement**
   - Dynamic tool registration
   - Capability negotiation
   - Permission validation

3. **Slash command workflow integration**
   - Command parsing
   - Context preservation
   - Response handling

**Framework Files to Create/Modify**:

- Create: `integration/mcp-integration-specifications.md`
- Enhance: `data-management/tool-registry.md`
- Modify: `security/security-framework.md`

### Phase 5: Advanced Features

**Dependencies**: All previous phases

**Components**:

1. **Advanced coordination patterns**
   - Distributed consensus
   - Event sourcing
   - CQRS implementation

2. **Performance optimization**
   - Connection multiplexing
   - Message compression
   - Async I/O optimization

3. **Monitoring and observability enhancements**
   - Distributed tracing
   - Metrics collection
   - Log aggregation

---

## Technical Specifications

### 1. Performance Requirements

```rust
// Performance targets for Claude CLI integration
const PERFORMANCE_TARGETS: PerformanceTargets = PerformanceTargets {
    agent_spawn_time_ms: 5000,
    max_concurrent_agents: 25,
    max_memory_usage_mb: 6000,
    hook_latency_ms: 100,
    message_throughput_per_sec: 1000,
};
```

**Monitoring Integration:**

- [Observability Framework](../../operations/observability-monitoring-framework.md)
- [Performance Monitoring](../../operations/performance-monitoring.md)
- [Health Checks](../../operations/health-checks.md)

### 2. Reliability Requirements

```rust
// Reliability targets for Claude CLI integration
const RELIABILITY_TARGETS: ReliabilityTargets = ReliabilityTargets {
    agent_uptime_percentage: 99.0,
    hook_success_rate: 99.5,
    error_recovery_time_sec: 30,
    message_delivery_rate: 99.9,
};
```

**Error Handling Integration:**

- [Error Handling](../../core-architecture/error-handling.md)
- [Supervision Trees](../../core-architecture/supervision-trees.md)
- [Recovery Patterns](../../core-architecture/recovery-patterns.md)

### 3. Integration Compatibility

```rust
// Integration compatibility requirements
const INTEGRATION_REQUIREMENTS: IntegrationRequirements = IntegrationRequirements {
    claude_cli_feature_coverage: 100,
    breaking_changes_allowed: false,
    configuration_complexity: ConfigComplexity::Single,
    backward_compatibility: true,
};
```

**Framework Integration:**

- [System Architecture](../../core-architecture/system-architecture.md)
- [Component Architecture](../../core-architecture/component-architecture.md)
- [Integration Patterns](../../core-architecture/integration-patterns.md)

### 4. Security Requirements

```rust
// Security requirements for Claude CLI integration
const SECURITY_REQUIREMENTS: SecurityRequirements = SecurityRequirements {
    authentication_required: true,
    authorization_granularity: AuthzGranularity::Agent,
    audit_logging: true,
    secure_communication: true,
};
```

**Security Integration:**

- [Security Framework](../../security/security-framework.md)
- [Authentication](../../security/authentication.md)
- [Authorization](../../security/authorization.md)

---

## Implementation Validation

### Testing Strategy

1. **Unit Tests**
   - Component isolation testing
   - Mock service validation
   - Error condition coverage

2. **Integration Tests**
   - End-to-end workflow validation
   - Cross-component communication
   - Performance benchmarking

3. **Load Tests**
   - Concurrent agent spawning
   - Message throughput limits
   - Resource exhaustion scenarios

### Verification Commands

```bash
# Verify Claude CLI integration
cargo test --package claude-cli-integration

# Benchmark performance
cargo bench --package claude-cli-integration

# Load test with multiple agents
cargo run --bin load-test -- --agents 25 --duration 300
```

**Testing Integration:**

- [Testing Framework](../../testing/testing-framework.md)
- [Integration Tests](../../testing/integration-tests.md)
- [Load Testing](../../testing/load-testing.md)

This integration plan provides technical specifications for incorporating Claude Code CLI capabilities while maintaining the integrity and performance of the existing Mister Smith framework.

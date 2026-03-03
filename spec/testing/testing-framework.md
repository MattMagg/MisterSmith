# MISTER SMITH TESTING FRAMEWORK

**Agent-Focused Testing Strategy for Multi-Agent AI Framework**

## RELATED DOCUMENTS

### Core Architecture

- [System Architecture](../core-architecture/system-architecture.md) - Overall system design and component integration
- [Component Architecture](../core-architecture/component-architecture.md) - Service and module organization
- [Integration Implementation](../core-architecture/integration-implementation.md) - Contract-based testing framework and integration test harness
- [Async Patterns](../core-architecture/async-patterns.md) - Asynchronous testing patterns and concurrency validation
- [Supervision Trees](../core-architecture/supervision-trees.md) - Fault tolerance and recovery testing
- [Type Definitions](../core-architecture/type-definitions.md) - Type-safe testing patterns and validation schemas

### Data Management

- [Message Framework](../data-management/message-framework.md) - Message validation and serialization testing
- [Core Message Schemas](../data-management/core-message-schemas.md) - Schema validation and compatibility testing
- [Agent Communication](../data-management/agent-communication.md) - Communication protocol testing patterns
- [Agent Lifecycle](../data-management/agent-lifecycle.md) - Lifecycle state validation and transition testing
- [Database Schemas](../data-management/database-schemas.md) - Database integration and migration testing
- [Data Persistence](../data-management/data-persistence.md) - Persistence layer testing and data integrity validation

### Security

- [Security Framework](../security/security-framework.md) - Security testing patterns and vulnerability assessment
- [Authentication Specifications](../security/authentication-specifications.md) - Authentication flow testing
- [Authorization Specifications](../security/authorization-specifications.md) - Authorization policy testing and access control validation

### Transport Layer

- [Transport Core](../transport/transport-core.md) - Transport protocol testing and network simulation
- [NATS Transport](../transport/nats-transport.md) - NATS integration testing and message delivery validation
- [gRPC Transport](../transport/grpc-transport.md) - gRPC service testing and contract validation
- [HTTP Transport](../transport/http-transport.md) - HTTP API testing and endpoint validation

### Operations

- [Configuration Management](../operations/configuration-management.md) - Configuration validation and environment testing
- [Observability Monitoring](../operations/observability-monitoring-framework.md) - Monitoring and metrics validation
- [Process Management](../operations/process-management-specifications.md) - Process lifecycle and resource management testing

### Testing Resources

- [Test Schemas](test-schemas.md) - Test data structures and message schemas
- [Testing CLAUDE Guide](CLAUDE.md) - Testing directory navigation and instructions

## TESTING PHILOSOPHY

### Core Principles

- **Comprehensive Coverage**: 90%+ line coverage for core modules, 100% for security-critical components
- **Agent-Centric Testing**: All tests designed for AI agent scenarios and workflows
- **Async-First Architecture**: Native support for asynchronous agent operations
- **Mock-Driven Development**: Isolated testing with comprehensive mock frameworks
- **Performance-Aware**: Continuous benchmarking and performance regression detection
- **Security-Validated**: Mandatory security testing for all authentication and authorization flows

### Testing Hierarchy

```text
P0: Critical Path Tests (agent lifecycle, security, data integrity)
P1: Core Functionality Tests (communication, persistence, configuration)
P2: Edge Case and Error Handling Tests
P3: Performance and Scalability Tests
P4: Compatibility and Regression Tests
```

## MULTI-AGENT TESTING SPECIFICATIONS

### Agent Communication Testing Patterns

Testing patterns specifically designed for multi-agent system validation:

#### Contract-Based Testing

- Consumer-driven contract testing for agent communication protocols
- Service boundary validation between agent types
- Message schema contract verification across agent domains
- Protocol compliance testing for transport layers

#### Chaos Engineering for Agent Systems

- Failure injection patterns for agent resilience testing
- Network partition simulation between agent clusters
- Service degradation testing under load
- Agent isolation and recovery testing

#### Performance Monitoring for Agent Operations

- Continuous performance regression detection for agent workflows
- Benchmark result analysis and trending for multi-agent scenarios
- Resource utilization tracking across agent populations
- Scalability testing for agent orchestration patterns

#### Security Validation for Agent Communications

- Automated vulnerability scanning for agent-to-agent communications
- Authentication/authorization flow testing for agent identity systems
- Secure transport verification for agent message passing
- Agent privilege escalation and access control testing

#### Test Analytics for Agent Systems

- Test result aggregation and analysis across agent domains
- Trend detection and reporting for multi-agent system health
- Quality gate enforcement for agent deployment readiness
- Cross-agent interaction pattern validation

## UNIT TEST PATTERNS

### Agent Testing Template

```rust
use tokio_test;
use mockall::predicate::*;
use crate::agents::{Agent, AgentConfig, AgentStatus};
use crate::testing::{MockMessagingService, TestAgentBuilder};

#[tokio::test]
async fn test_agent_lifecycle() {
    // Arrange
    let mut mock_messaging = MockMessagingService::new();
    mock_messaging
        .expect_send_message()
        .times(1)
        .returning(|_| Ok(()));
    
    let agent = TestAgentBuilder::new()
        .with_id("test-agent-001")
        .with_messaging_service(mock_messaging)
        .build();
    
    // Act
    let result = agent.start().await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(agent.status(), AgentStatus::Running);
}

#[test]
fn test_agent_configuration_validation() {
    // Property-based testing for agent configurations
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn agent_config_roundtrip(
            id in "[a-zA-Z0-9-]{1,50}",
            max_memory in 1..1000u64,
            timeout in 1..3600u32
        ) {
            let config = AgentConfig {
                id: id.clone(),
                max_memory_mb: max_memory,
                timeout_seconds: timeout,
            };
            
            let serialized = serde_json::to_string(&config)?;
            let deserialized: AgentConfig = serde_json::from_str(&serialized)?;
            
            assert_eq!(config.id, deserialized.id);
            assert_eq!(config.max_memory_mb, deserialized.max_memory_mb);
            assert_eq!(config.timeout_seconds, deserialized.timeout_seconds);
        }
    }
}
```

### Data Validation Testing Pattern

```rust
use crate::data::{MessageSchema, ValidationResult};
use crate::testing::TestDataGenerator;

#[test]
fn test_message_schema_validation() {
    let generator = TestDataGenerator::new();
    
    // Valid message test
    let valid_message = generator.create_valid_message();
    assert!(MessageSchema::validate(&valid_message).is_valid());
    
    // Invalid message tests
    let invalid_messages = generator.create_invalid_messages();
    for message in invalid_messages {
        let result = MessageSchema::validate(&message);
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }
}

#[test]
fn test_data_persistence_integrity() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let repository = TestRepository::new(&db_path);
    let test_data = TestDataGenerator::new().create_agent_data();
    
    // Test write-read cycle
    repository.save(&test_data).unwrap();
    let retrieved_data = repository.load(&test_data.id).unwrap();
    
    assert_eq!(test_data, retrieved_data);
}
```

### Security Testing Pattern

```rust
use crate::security::{AuthenticationService, AuthorizationPolicy};
use crate::testing::{MockTokenProvider, TestSecurityContext};

#[tokio::test]
async fn test_authentication_flow() {
    let mut mock_token_provider = MockTokenProvider::new();
    mock_token_provider
        .expect_validate_token()
        .with(eq("valid-token"))
        .returning(|_| Ok(true));
    
    let auth_service = AuthenticationService::new(mock_token_provider);
    let result = auth_service.authenticate("valid-token").await;
    
    assert!(result.is_ok());
    assert!(result.unwrap().is_authenticated);
}

#[test]
fn test_authorization_policies() {
    let policy = AuthorizationPolicy::from_rules(vec![
        "agent:read:*",
        "agent:write:own",
        "admin:*:*",
    ]);
    
    // Test agent read access
    assert!(policy.check("agent", "read", "any-resource"));
    
    // Test agent write access (own resources only)
    assert!(policy.check("agent", "write", "agent-123"));
    assert!(!policy.check("agent", "write", "other-agent"));
    
    // Test admin access
    assert!(policy.check("admin", "delete", "any-resource"));
}
```

## INTEGRATION TEST SPECIFICATIONS

### Multi-Agent Communication Testing

```rust
use testcontainers::{clients::Cli, images::generic::GenericImage};
use crate::transport::NatsMessagingService;
use crate::agents::{AgentOrchestrator, AgentType};

#[tokio::test]
async fn test_multi_agent_coordination() {
    // Start NATS test container
    let docker = Cli::default();
    let nats_container = docker.run(
        GenericImage::new("nats", "latest")
            .with_exposed_port(4222)
    );
    
    let nats_url = format!(
        "nats://localhost:{}", 
        nats_container.get_host_port_ipv4(4222)
    );
    
    // Setup orchestrator with test agents
    let messaging = NatsMessagingService::new(&nats_url).await.unwrap();
    let mut orchestrator = AgentOrchestrator::new(messaging);
    
    // Spawn test agents
    let analyst_id = orchestrator.spawn_agent(AgentType::Analyst).await.unwrap();
    let architect_id = orchestrator.spawn_agent(AgentType::Architect).await.unwrap();
    
    // Test communication flow
    let task = TestTask::new("analyze-system-requirements");
    let result = orchestrator.execute_workflow(task).await.unwrap();
    
    assert!(result.is_complete());
    assert_eq!(result.participating_agents().len(), 2);
}
```

### Database Integration Testing

```rust
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase};
use crate::data::{DatabaseManager, MigrationRunner};

#[tokio::test]
async fn test_database_migrations() {
    let db_url = "postgres://test:test@localhost/test_db";
    
    // Create test database
    if !Postgres::database_exists(db_url).await.unwrap_or(false) {
        Postgres::create_database(db_url).await.unwrap();
    }
    
    let pool = PgPool::connect(db_url).await.unwrap();
    let migration_runner = MigrationRunner::new(&pool);
    
    // Run migrations
    migration_runner.run_all().await.unwrap();
    
    // Verify schema
    let schema_version = migration_runner.current_version().await.unwrap();
    assert!(schema_version > 0);
    
    // Test data operations
    let db_manager = DatabaseManager::new(pool);
    let test_agent = TestAgentBuilder::new().build();
    
    db_manager.save_agent(&test_agent).await.unwrap();
    let retrieved = db_manager.load_agent(&test_agent.id).await.unwrap();
    
    assert_eq!(test_agent.id, retrieved.id);
}
```

### Claude CLI Integration Testing

```rust
use crate::claude::{ClaudeCliService, ClaudeCommand};
use crate::testing::{MockClaudeProcess, TestTaskBuilder};

#[tokio::test]
async fn test_claude_cli_integration() {
    let mut mock_process = MockClaudeProcess::new();
    mock_process
        .expect_execute()
        .returning(|cmd| Ok(format!("Executed: {}", cmd)));
    
    let claude_service = ClaudeCliService::new(mock_process);
    let task = TestTaskBuilder::new()
        .with_type("code-analysis")
        .with_files(vec!["src/main.rs"])
        .build();
    
    let result = claude_service.execute_task(task).await.unwrap();
    
    assert!(result.is_success());
    assert!(!result.output().is_empty());
}
```

### Multi-Agent Workflow Testing

```rust
use crate::orchestration::{WorkflowEngine, WorkflowSpec, AgentPool};
use crate::testing::{MockAgentPool, TestWorkflowBuilder};

#[tokio::test]
async fn test_multi_agent_workflow_execution() {
    let mut mock_agent_pool = MockAgentPool::new();
    mock_agent_pool
        .expect_allocate_agent()
        .returning(|agent_type| Ok(TestAgent::new(agent_type)));
    
    let workflow_engine = WorkflowEngine::new(mock_agent_pool);
    
    let workflow = TestWorkflowBuilder::new()
        .with_stage("analysis", vec!["analyst-agent"])
        .with_stage("implementation", vec!["architect-agent", "developer-agent"])
        .with_stage("validation", vec!["reviewer-agent"])
        .build();
    
    let result = workflow_engine.execute(workflow).await.unwrap();
    
    assert!(result.is_complete());
    assert_eq!(result.stages_completed(), 3);
    assert!(result.agents_participated().len() >= 4);
}

#[tokio::test]
async fn test_agent_failure_recovery() {
    let mut mock_agent_pool = MockAgentPool::new();
    
    // First agent fails, second succeeds
    mock_agent_pool
        .expect_allocate_agent()
        .times(2)
        .returning(|agent_type| {
            static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
            let count = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            
            if count == 0 {
                Err(AgentError::AllocationFailed)
            } else {
                Ok(TestAgent::new(agent_type))
            }
        });
    
    let workflow_engine = WorkflowEngine::new(mock_agent_pool);
    let workflow = TestWorkflowBuilder::new()
        .with_retry_policy(RetryPolicy::exponential_backoff(3))
        .with_stage("analysis", vec!["analyst-agent"])
        .build();
    
    let result = workflow_engine.execute(workflow).await.unwrap();
    
    assert!(result.is_complete());
    assert_eq!(result.retry_count(), 1);
}
```

### Agent State Synchronization Testing

```rust
use crate::sync::{AgentStateManager, StateSync, ConflictResolution};
use crate::testing::{MockStateStore, TestAgentState};

#[tokio::test]
async fn test_agent_state_synchronization() {
    let mut mock_store = MockStateStore::new();
    mock_store
        .expect_get_state()
        .returning(|agent_id| Ok(TestAgentState::default()));
    
    mock_store
        .expect_update_state()
        .returning(|agent_id, state| Ok(()));
    
    let state_manager = AgentStateManager::new(mock_store);
    
    // Create multiple agents with conflicting state updates
    let agent1 = TestAgent::new("agent-1");
    let agent2 = TestAgent::new("agent-2");
    
    agent1.update_shared_state("key1", "value1").await.unwrap();
    agent2.update_shared_state("key1", "value2").await.unwrap();
    
    // Test conflict resolution
    let resolved_state = state_manager
        .resolve_conflicts(vec![&agent1, &agent2])
        .await
        .unwrap();
    
    assert!(resolved_state.contains_key("key1"));
    assert!(resolved_state.is_consistent());
}
```

### Agent Load Balancing Testing

```rust
use crate::load_balancing::{LoadBalancer, BalancingStrategy, AgentCapacity};
use crate::testing::{MockMetricsCollector, TestTaskDistributor};

#[tokio::test]
async fn test_agent_load_balancing() {
    let mut mock_metrics = MockMetricsCollector::new();
    mock_metrics
        .expect_get_agent_load()
        .returning(|agent_id| {
            match agent_id {
                "agent-1" => Ok(0.2), // 20% load
                "agent-2" => Ok(0.8), // 80% load
                "agent-3" => Ok(0.1), // 10% load
                _ => Ok(0.0),
            }
        });
    
    let load_balancer = LoadBalancer::new(
        BalancingStrategy::LeastLoaded,
        mock_metrics
    );
    
    let agents = vec!["agent-1", "agent-2", "agent-3"];
    let selected_agent = load_balancer
        .select_agent(agents, TaskType::Compute)
        .await
        .unwrap();
    
    // Should select agent-3 (lowest load)
    assert_eq!(selected_agent, "agent-3");
}

#[tokio::test]
async fn test_agent_capacity_scaling() {
    let mut mock_metrics = MockMetricsCollector::new();
    mock_metrics
        .expect_get_system_load()
        .returning(|| Ok(0.9)); // 90% system load
    
    let load_balancer = LoadBalancer::new(
        BalancingStrategy::AdaptiveScaling,
        mock_metrics
    );
    
    let scaling_decision = load_balancer
        .evaluate_scaling_need()
        .await
        .unwrap();
    
    assert!(scaling_decision.should_scale_up());
    assert!(scaling_decision.recommended_agents() > 0);
}
```

## MOCK OBJECT PATTERNS

### Mock Service Generators

#### Automated Mock Generation

```rust
use mockall::automock;
use async_trait::async_trait;
use crate::testing::mock_generators::{MockServiceBuilder, MockBehavior};

// Generate mocks with builder pattern
pub struct MockServiceBuilder {
    service_type: ServiceType,
    behaviors: Vec<MockBehavior>,
    default_responses: HashMap<String, Box<dyn Any>>,
}

impl MockServiceBuilder {
    pub fn new(service_type: ServiceType) -> Self {
        Self {
            service_type,
            behaviors: Vec::new(),
            default_responses: HashMap::new(),
        }
    }
    
    pub fn with_behavior(mut self, behavior: MockBehavior) -> Self {
        self.behaviors.push(behavior);
        self
    }
    
    pub fn with_default_response<T: 'static>(mut self, method: &str, response: T) -> Self {
        self.default_responses.insert(method.to_string(), Box::new(response));
        self
    }
    
    pub fn build<T: MockableService>(&self) -> T {
        let mut mock = T::new();
        
        // Apply behaviors
        for behavior in &self.behaviors {
            match behavior {
                MockBehavior::AlwaysSucceed => {
                    mock.set_all_responses(|_| Ok(Default::default()));
                },
                MockBehavior::FailAfterN(n) => {
                    let counter = Arc::new(AtomicUsize::new(0));
                    mock.set_all_responses(move |_| {
                        let count = counter.fetch_add(1, Ordering::SeqCst);
                        if count >= *n {
                            Err("Simulated failure".into())
                        } else {
                            Ok(Default::default())
                        }
                    });
                },
                MockBehavior::DelayResponse(duration) => {
                    mock.set_all_responses(move |_| {
                        std::thread::sleep(*duration);
                        Ok(Default::default())
                    });
                },
            }
        }
        
        mock
    }
}

// Usage example
#[test]
fn test_with_mock_generator() {
    let mock_messaging = MockServiceBuilder::new(ServiceType::Messaging)
        .with_behavior(MockBehavior::FailAfterN(3))
        .with_behavior(MockBehavior::DelayResponse(Duration::from_millis(100)))
        .build::<MockMessagingService>();
        
    // Test resilience with the configured mock
    let agent = Agent::new("test", mock_messaging);
    assert!(agent.retry_operation().is_ok()); // Succeeds first 3 times
}
```

### Behavior Verification Patterns

#### Comprehensive Interaction Verification

```rust
use mockall::Sequence;
use std::sync::{Arc, Mutex};

pub struct BehaviorVerifier {
    call_history: Arc<Mutex<Vec<CallRecord>>>,
    expectations: Vec<ExpectationRule>,
}

#[derive(Clone, Debug)]
pub struct CallRecord {
    method: String,
    arguments: Vec<String>,
    timestamp: SystemTime,
    thread_id: std::thread::ThreadId,
}

impl BehaviorVerifier {
    pub fn new() -> Self {
        Self {
            call_history: Arc::new(Mutex::new(Vec::new())),
            expectations: Vec::new(),
        }
    }
    
    pub fn expect_sequence(&mut self, methods: Vec<&str>) {
        self.expectations.push(ExpectationRule::Sequence(
            methods.into_iter().map(String::from).collect()
        ));
    }
    
    pub fn expect_concurrent(&mut self, methods: Vec<&str>) {
        self.expectations.push(ExpectationRule::Concurrent(
            methods.into_iter().map(String::from).collect()
        ));
    }
    
    pub fn verify(&self) -> Result<(), VerificationError> {
        let history = self.call_history.lock().unwrap();
        
        for expectation in &self.expectations {
            match expectation {
                ExpectationRule::Sequence(expected_methods) => {
                    let actual_methods: Vec<_> = history.iter()
                        .map(|r| r.method.as_str())
                        .collect();
                    
                    if !Self::verify_sequence(&actual_methods, expected_methods) {
                        return Err(VerificationError::SequenceMismatch);
                    }
                },
                ExpectationRule::Concurrent(expected_methods) => {
                    let thread_groups = Self::group_by_thread(&history);
                    if !Self::verify_concurrent(thread_groups, expected_methods) {
                        return Err(VerificationError::ConcurrencyViolation);
                    }
                }
            }
        }
        
        Ok(())
    }
}

// Advanced mock with behavior verification
#[automock]
#[async_trait]
pub trait VerifiableService {
    async fn operation_a(&self) -> Result<(), Error>;
    async fn operation_b(&self, data: &str) -> Result<String, Error>;
    async fn operation_c(&self) -> Result<(), Error>;
}

#[tokio::test]
async fn test_complex_behavior_verification() {
    let verifier = Arc::new(BehaviorVerifier::new());
    let mut mock = MockVerifiableService::new();
    
    // Setup mock with verification
    let verifier_clone = verifier.clone();
    mock.expect_operation_a()
        .returning(move || {
            verifier_clone.record_call("operation_a", vec![]);
            Ok(())
        });
    
    // Test concurrent operations
    let mock = Arc::new(mock);
    let handles: Vec<_> = (0..5).map(|_| {
        let mock_clone = mock.clone();
        tokio::spawn(async move {
            mock_clone.operation_a().await.unwrap();
        })
    }).collect();
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify behavior
    verifier.expect_concurrent(vec!["operation_a"; 5]);
    assert!(verifier.verify().is_ok());
}
```

### State-Based Mocking

#### Advanced State Machine Testing

```rust
// Validated Pattern: Comprehensive agent lifecycle state validation
use std::sync::{Arc, RwLock};

pub struct AgentStateMachine {
    state: Arc<RwLock<AgentState>>,
    transitions: HashMap<(AgentStatus, Event), AgentStatus>,
    validators: Vec<Box<dyn StateValidator>>,
}

impl AgentStateMachine {
    pub fn validate_transition(&self, from: AgentStatus, event: Event, to: AgentStatus) -> bool {
        self.transitions.get(&(from, event))
            .map(|expected| *expected == to)
            .unwrap_or(false)
    }
    
    pub fn test_edge_cases(&self) -> Vec<EdgeCaseResult> {
        // Comprehensive edge case coverage for state transitions
        vec![
            self.test_concurrent_state_changes(),
            self.test_error_state_recovery(),
            self.test_timeout_transitions(),
        ]
    }
}

#[test]
fn test_agent_lifecycle_state_machine() {
    let state_machine = AgentStateMachine::new();
    
    // Test all valid state transitions
    assert!(state_machine.validate_transition(
        AgentStatus::Created, 
        Event::Start, 
        AgentStatus::Starting
    ));
    
    // Test invalid transitions
    assert!(!state_machine.validate_transition(
        AgentStatus::Failed,
        Event::Start,
        AgentStatus::Running
    ));
    
    // Test edge cases
    let edge_results = state_machine.test_edge_cases();
    assert!(edge_results.iter().all(|r| r.passed));
}
```

#### Stateful Mock Implementation

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub struct StatefulMock<S: Clone> {
    state: Arc<RwLock<S>>,
    transitions: HashMap<String, Box<dyn Fn(&mut S) + Send + Sync>>,
    validators: HashMap<String, Box<dyn Fn(&S) -> bool + Send + Sync>>,
}

impl<S: Clone + Default> StatefulMock<S> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(S::default())),
            transitions: HashMap::new(),
            validators: HashMap::new(),
        }
    }
    
    pub fn with_initial_state(state: S) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            transitions: HashMap::new(),
            validators: HashMap::new(),
        }
    }
    
    pub fn add_transition<F>(&mut self, trigger: &str, transition: F)
    where
        F: Fn(&mut S) + Send + Sync + 'static,
    {
        self.transitions.insert(trigger.to_string(), Box::new(transition));
    }
    
    pub fn add_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&S) -> bool + Send + Sync + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }
    
    pub fn trigger(&self, event: &str) -> Result<(), StateError> {
        let transition = self.transitions.get(event)
            .ok_or(StateError::UnknownTransition)?;
        
        let mut state = self.state.write().unwrap();
        transition(&mut *state);
        Ok(())
    }
    
    pub fn validate(&self, validator_name: &str) -> Result<bool, StateError> {
        let validator = self.validators.get(validator_name)
            .ok_or(StateError::UnknownValidator)?;
        
        let state = self.state.read().unwrap();
        Ok(validator(&*state))
    }
}

// Example: Agent lifecycle state machine mock
#[derive(Clone, Default)]
struct AgentState {
    status: AgentStatus,
    message_count: usize,
    errors: Vec<String>,
    connected_peers: Vec<String>,
}

#[test]
fn test_agent_state_machine() {
    let mut mock = StatefulMock::with_initial_state(AgentState {
        status: AgentStatus::Idle,
        ..Default::default()
    });
    
    // Define state transitions
    mock.add_transition("start", |state| {
        state.status = AgentStatus::Starting;
    });
    
    mock.add_transition("connect", |state| {
        if state.status == AgentStatus::Starting {
            state.status = AgentStatus::Running;
            state.connected_peers.push("peer-1".to_string());
        }
    });
    
    mock.add_transition("receive_message", |state| {
        state.message_count += 1;
    });
    
    mock.add_transition("error", |state| {
        state.errors.push("Test error".to_string());
        if state.errors.len() > 3 {
            state.status = AgentStatus::Failed;
        }
    });
    
    // Define validators
    mock.add_validator("is_healthy", |state| {
        state.status == AgentStatus::Running && state.errors.is_empty()
    });
    
    mock.add_validator("can_process", |state| {
        matches!(state.status, AgentStatus::Running) && state.connected_peers.len() > 0
    });
    
    // Test state transitions
    mock.trigger("start").unwrap();
    mock.trigger("connect").unwrap();
    
    assert!(mock.validate("is_healthy").unwrap());
    assert!(mock.validate("can_process").unwrap());
    
    // Simulate errors
    for _ in 0..4 {
        mock.trigger("error").unwrap();
    }
    
    assert!(!mock.validate("is_healthy").unwrap());
}
```

### Integration Test Mocks

#### Complex Service Integration Mocks

```rust
use wiremock::{MockServer, Mock, ResponseTemplate, Request};
use wiremock::matchers::{method, path, header, body_partial_json};

pub struct IntegrationMockBuilder {
    server: MockServer,
    scenarios: Vec<ScenarioConfig>,
    state_machine: StatefulMock<IntegrationState>,
}

impl IntegrationMockBuilder {
    pub async fn new() -> Self {
        Self {
            server: MockServer::start().await,
            scenarios: Vec::new(),
            state_machine: StatefulMock::new(),
        }
    }
    
    pub fn add_scenario(mut self, scenario: ScenarioConfig) -> Self {
        self.scenarios.push(scenario);
        self
    }
    
    pub async fn build(self) -> ConfiguredMockServer {
        for scenario in self.scenarios {
            self.setup_scenario(scenario).await;
        }
        
        ConfiguredMockServer {
            server: self.server,
            state_machine: self.state_machine,
        }
    }
    
    async fn setup_scenario(&self, scenario: ScenarioConfig) {
        match scenario {
            ScenarioConfig::Authentication { success_rate } => {
                Mock::given(method("POST"))
                    .and(path("/auth/token"))
                    .respond_with(move |req: &Request| {
                        if rand::random::<f64>() < success_rate {
                            ResponseTemplate::new(200).set_body_json(json!({
                                "token": "test-token",
                                "expires_in": 3600
                            }))
                        } else {
                            ResponseTemplate::new(401).set_body_json(json!({
                                "error": "Authentication failed"
                            }))
                        }
                    })
                    .mount(&self.server)
                    .await;
            },
            ScenarioConfig::RateLimited { limit, window } => {
                let counter = Arc::new(Mutex::new(HashMap::new()));
                Mock::given(method("GET"))
                    .and(path_regex(r"^/api/.*"))
                    .respond_with(move |req: &Request| {
                        let client_id = req.headers()
                            .get("X-Client-ID")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("default");
                        
                        let mut counts = counter.lock().unwrap();
                        let count = counts.entry(client_id.to_string()).or_insert(0);
                        *count += 1;
                        
                        if *count > limit {
                            ResponseTemplate::new(429)
                                .insert_header("X-RateLimit-Limit", limit.to_string())
                                .insert_header("X-RateLimit-Remaining", "0")
                        } else {
                            ResponseTemplate::new(200).set_body_json(json!({
                                "data": "Success"
                            }))
                        }
                    })
                    .mount(&self.server)
                    .await;
            },
            ScenarioConfig::CircuitBreaker { failure_threshold } => {
                let failure_count = Arc::new(AtomicUsize::new(0));
                let circuit_open = Arc::new(AtomicBool::new(false));
                
                Mock::given(method("POST"))
                    .and(path("/api/process"))
                    .respond_with(move |_: &Request| {
                        if circuit_open.load(Ordering::SeqCst) {
                            return ResponseTemplate::new(503).set_body_json(json!({
                                "error": "Circuit breaker open"
                            }));
                        }
                        
                        // Simulate failures
                        if rand::random::<f64>() < 0.3 {
                            let failures = failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                            if failures >= failure_threshold {
                                circuit_open.store(true, Ordering::SeqCst);
                            }
                            ResponseTemplate::new(500)
                        } else {
                            failure_count.store(0, Ordering::SeqCst);
                            ResponseTemplate::new(200)
                        }
                    })
                    .mount(&self.server)
                    .await;
            }
        }
    }
}

// Integration test using complex mocks
#[tokio::test]
async fn test_resilient_client_integration() {
    let mock_server = IntegrationMockBuilder::new()
        .add_scenario(ScenarioConfig::Authentication { success_rate: 0.9 })
        .add_scenario(ScenarioConfig::RateLimited { limit: 10, window: 60 })
        .add_scenario(ScenarioConfig::CircuitBreaker { failure_threshold: 3 })
        .build()
        .await;
    
    let client = ResilientClient::new(&mock_server.uri())
        .with_retry_policy(RetryPolicy::exponential_backoff())
        .with_circuit_breaker(CircuitBreakerConfig::default());
    
    // Test authentication resilience
    let auth_result = client.authenticate("test-user", "test-pass").await;
    assert!(auth_result.is_ok());
    
    // Test rate limiting
    for i in 0..15 {
        let result = client.get_resource(&format!("/api/resource/{}", i)).await;
        if i < 10 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(ClientError::RateLimited)));
        }
    }
    
    // Test circuit breaker
    let mut failures = 0;
    for _ in 0..10 {
        if client.process_data("test-data").await.is_err() {
            failures += 1;
        }
    }
    assert!(failures >= 3); // Circuit should open after threshold
}
```

### Performance Test Mocks

#### High-Performance Mock Implementation

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use dashmap::DashMap;

pub struct PerformanceMock {
    call_metrics: Arc<DashMap<String, CallMetrics>>,
    latency_simulator: Arc<LatencySimulator>,
    resource_simulator: Arc<ResourceSimulator>,
}

#[derive(Default)]
struct CallMetrics {
    count: AtomicU64,
    total_duration: AtomicU64,
    min_duration: AtomicU64,
    max_duration: AtomicU64,
}

impl PerformanceMock {
    pub fn new() -> Self {
        Self {
            call_metrics: Arc::new(DashMap::new()),
            latency_simulator: Arc::new(LatencySimulator::new()),
            resource_simulator: Arc::new(ResourceSimulator::new()),
        }
    }
    
    pub async fn simulate_operation(&self, operation: &str) -> Result<(), Error> {
        let start = Instant::now();
        
        // Simulate latency
        self.latency_simulator.simulate(operation).await;
        
        // Simulate resource usage
        self.resource_simulator.consume(operation).await?;
        
        // Record metrics
        let duration = start.elapsed().as_nanos() as u64;
        let metrics = self.call_metrics.entry(operation.to_string())
            .or_insert_with(CallMetrics::default);
        
        metrics.count.fetch_add(1, Ordering::Relaxed);
        metrics.total_duration.fetch_add(duration, Ordering::Relaxed);
        
        // Update min/max with CAS loop
        loop {
            let current_min = metrics.min_duration.load(Ordering::Relaxed);
            if duration >= current_min && current_min != 0 {
                break;
            }
            if metrics.min_duration.compare_exchange(
                current_min,
                duration,
                Ordering::Relaxed,
                Ordering::Relaxed
            ).is_ok() {
                break;
            }
        }
        
        Ok(())
    }
    
    pub fn get_metrics(&self) -> HashMap<String, OperationMetrics> {
        self.call_metrics.iter()
            .map(|entry| {
                let key = entry.key().clone();
                let metrics = entry.value();
                let count = metrics.count.load(Ordering::Relaxed);
                let total = metrics.total_duration.load(Ordering::Relaxed);
                
                (key, OperationMetrics {
                    count,
                    average_duration_ns: if count > 0 { total / count } else { 0 },
                    min_duration_ns: metrics.min_duration.load(Ordering::Relaxed),
                    max_duration_ns: metrics.max_duration.load(Ordering::Relaxed),
                })
            })
            .collect()
    }
}

// Latency simulation for realistic performance testing
struct LatencySimulator {
    profiles: RwLock<HashMap<String, LatencyProfile>>,
}

impl LatencySimulator {
    async fn simulate(&self, operation: &str) {
        let profiles = self.profiles.read().await;
        if let Some(profile) = profiles.get(operation) {
            let latency = profile.sample();
            tokio::time::sleep(Duration::from_micros(latency)).await;
        }
    }
}

// Performance benchmarking with mocks
#[tokio::test]
async fn test_system_performance_under_load() {
    let mock = Arc::new(PerformanceMock::new());
    
    // Configure realistic latency profiles
    mock.latency_simulator.add_profile(
        "database_read",
        LatencyProfile::normal(1000, 200), // 1ms ± 200μs
    );
    mock.latency_simulator.add_profile(
        "cache_read",
        LatencyProfile::normal(50, 10), // 50μs ± 10μs
    );
    mock.latency_simulator.add_profile(
        "network_call",
        LatencyProfile::pareto(5000, 1.2), // Long tail latency
    );
    
    // Simulate concurrent load
    let handles: Vec<_> = (0..1000).map(|i| {
        let mock_clone = mock.clone();
        tokio::spawn(async move {
            let operation = match i % 3 {
                0 => "database_read",
                1 => "cache_read",
                _ => "network_call",
            };
            
            for _ in 0..100 {
                mock_clone.simulate_operation(operation).await.unwrap();
            }
        })
    }).collect();
    
    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Analyze performance metrics
    let metrics = mock.get_metrics();
    
    // Verify performance requirements
    assert!(metrics["cache_read"].average_duration_ns < 100_000); // <100μs
    assert!(metrics["database_read"].average_duration_ns < 2_000_000); // <2ms
    
    // Check for performance anomalies
    for (op, metric) in &metrics {
        let variance = metric.max_duration_ns / metric.average_duration_ns;
        assert!(variance < 10, "High variance detected in {}: {}x", op, variance);
    }
}
```

### Messaging Service Mock

```rust
use mockall::automock;
use async_trait::async_trait;

#[automock]
#[async_trait]
pub trait MessagingService {
    async fn send_message(&self, topic: &str, message: &[u8]) -> Result<(), MessagingError>;
    async fn subscribe(&self, topic: &str) -> Result<MessageStream, MessagingError>;
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), MessagingError>;
}

// Usage in tests
let mut mock = MockMessagingService::new();
mock.expect_send_message()
    .with(eq("agent.tasks"), always())
    .times(1)
    .returning(|_, _| Ok(()));
```

### Database Repository Mock

```rust
#[automock]
#[async_trait]
pub trait AgentRepository {
    async fn save(&self, agent: &Agent) -> Result<(), RepositoryError>;
    async fn load(&self, id: &str) -> Result<Option<Agent>, RepositoryError>;
    async fn list_active(&self) -> Result<Vec<Agent>, RepositoryError>;
    async fn delete(&self, id: &str) -> Result<(), RepositoryError>;
}
```

### External Service Mock

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_json};

#[tokio::test]
async fn test_external_api_integration() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/api/analyze"))
        .and(body_json(serde_json::json!({
            "type": "code-analysis",
            "files": ["src/main.rs"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({
                "status": "success",
                "results": ["Analysis complete"]
            })
        ))
        .mount(&mock_server)
        .await;
    
    let client = ExternalApiClient::new(&mock_server.uri());
    let result = client.analyze_code(vec!["src/main.rs"]).await.unwrap();
    
    assert_eq!(result.status, "success");
}
```

## TEST DATA GENERATION

### Property-Based Test Data

```rust
use proptest::prelude::*;

pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn agent_id_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9-]{8,32}"
    }
    
    pub fn agent_config_strategy() -> impl Strategy<Value = AgentConfig> {
        (
            Self::agent_id_strategy(),
            1..1000u64,
            1..3600u32,
        ).prop_map(|(id, memory, timeout)| AgentConfig {
            id,
            max_memory_mb: memory,
            timeout_seconds: timeout,
        })
    }
    
    pub fn message_strategy() -> impl Strategy<Value = Message> {
        (
            Self::agent_id_strategy(),
            Self::agent_id_strategy(),
            any::<MessageType>(),
            any::<Vec<u8>>(),
        ).prop_map(|(sender, receiver, msg_type, payload)| Message {
            sender,
            receiver,
            message_type: msg_type,
            payload,
            timestamp: SystemTime::now(),
        })
    }
}
```

### Test Builder Pattern

```rust
pub struct TestAgentBuilder {
    id: Option<String>,
    config: Option<AgentConfig>,
    messaging: Option<Box<dyn MessagingService>>,
}

impl TestAgentBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            config: None,
            messaging: None,
        }
    }
    
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    
    pub fn with_config(mut self, config: AgentConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn with_messaging_service(mut self, service: Box<dyn MessagingService>) -> Self {
        self.messaging = Some(service);
        self
    }
    
    pub fn build(self) -> Agent {
        let id = self.id.unwrap_or_else(|| "test-agent".to_string());
        let config = self.config.unwrap_or_default();
        let messaging = self.messaging.unwrap_or_else(|| Box::new(MockMessagingService::new()));
        
        Agent::new(id, config, messaging)
    }
}
```

## ADVANCED TESTING PATTERNS

### Property-Based Testing

```rust
// Validated Pattern: Configuration roundtrip validation
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_agent_config_properties(
        id in "[a-zA-Z0-9-]{8,32}",
        memory in 128..8192u64,
        timeout in 10..3600u32,
        max_tasks in 1..100u16
    ) {
        let config = AgentConfig {
            id: id.clone(),
            max_memory_mb: memory,
            timeout_seconds: timeout,
            max_concurrent_tasks: max_tasks,
        };
        
        // Property 1: Serialization roundtrip
        let serialized = serde_json::to_string(&config)?;
        let deserialized: AgentConfig = serde_json::from_str(&serialized)?;
        prop_assert_eq!(config, deserialized);
        
        // Property 2: Validation constraints
        prop_assert!(config.is_valid());
        prop_assert!(config.max_memory_mb >= 128);
        prop_assert!(config.timeout_seconds >= 10);
    }
    
    #[test]
    fn test_message_schema_properties(
        sender in "[a-zA-Z0-9-]{8,32}",
        receiver in "[a-zA-Z0-9-]{8,32}",
        payload_size in 1..1048576usize
    ) {
        let message = Message::new(sender, receiver, vec![0u8; payload_size]);
        
        // Property: Message size constraints
        prop_assert!(message.total_size() <= MAX_MESSAGE_SIZE);
        prop_assert!(message.is_routable());
    }
}
```

### Chaos Engineering Patterns

```rust
// Validated Pattern: Resilience testing with failure injection
use crate::chaos::{ChaosEngine, FailureMode};

pub struct ChaosTestFramework {
    engine: ChaosEngine,
    monitors: Vec<Box<dyn SystemMonitor>>,
}

impl ChaosTestFramework {
    pub fn new() -> Self {
        Self {
            engine: ChaosEngine::new(),
            monitors: vec![
                Box::new(LatencyMonitor::new()),
                Box::new(ErrorRateMonitor::new()),
                Box::new(ResourceMonitor::new()),
            ],
        }
    }
    
    pub async fn run_chaos_test(&self, scenario: ChaosScenario) -> ChaosTestResult {
        // Start monitoring
        for monitor in &self.monitors {
            monitor.start().await;
        }
        
        // Execute chaos scenario
        match scenario {
            ChaosScenario::NetworkPartition { duration, affected_agents } => {
                self.engine.inject_network_partition(affected_agents, duration).await?;
            },
            ChaosScenario::ServiceDegradation { service, latency_ms } => {
                self.engine.inject_latency(service, latency_ms).await?;
            },
            ChaosScenario::ResourceExhaustion { resource_type, percentage } => {
                self.engine.consume_resources(resource_type, percentage).await?;
            },
        }
        
        // Collect results
        self.collect_chaos_metrics().await
    }
}

#[tokio::test]
async fn test_system_resilience_under_chaos() {
    let chaos_framework = ChaosTestFramework::new();
    
    // Test network partition resilience
    let result = chaos_framework.run_chaos_test(
        ChaosScenario::NetworkPartition {
            duration: Duration::from_secs(30),
            affected_agents: vec!["agent-1", "agent-2"],
        }
    ).await;
    
    assert!(result.system_recovered);
    assert!(result.data_consistency_maintained);
    assert!(result.sla_maintained);
}

#[tokio::test]
async fn test_circuit_breaker_activation() {
    let chaos_framework = ChaosTestFramework::new();
    
    // Test circuit breaker under service degradation
    let result = chaos_framework.run_chaos_test(
        ChaosScenario::ServiceDegradation {
            service: "database",
            latency_ms: 5000,
        }
    ).await;
    
    assert!(result.circuit_breaker_activated);
    assert!(result.fallback_mechanism_engaged);
}
```

## PERFORMANCE TEST SPECIFICATIONS

### Benchmarking Framework

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use crate::agents::AgentOrchestrator;

fn bench_agent_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_spawn");
    
    for agent_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("spawn_agents", agent_count),
            agent_count,
            |b, &count| {
                b.iter(|| {
                    let orchestrator = AgentOrchestrator::new_test();
                    let agents = (0..count)
                        .map(|i| orchestrator.spawn_agent(AgentType::Worker))
                        .collect::<Vec<_>>();
                    black_box(agents);
                });
            },
        );
    }
    group.finish();
}

fn bench_message_throughput(c: &mut Criterion) {
    let messaging = NatsMessagingService::new_test();
    
    c.bench_function("message_throughput", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let message = format!("test-message-{}", i);
                messaging.send_message("test.topic", message.as_bytes());
            }
        });
    });
}

criterion_group!(benches, bench_agent_spawn, bench_message_throughput);
criterion_main!(benches);
```

### Load Testing Specifications

```rust
use tokio::time::{Duration, Instant};
use futures::future::join_all;

#[tokio::test]
async fn test_concurrent_agent_operations() {
    let orchestrator = AgentOrchestrator::new_test();
    let start_time = Instant::now();
    
    // Spawn 100 agents concurrently
    let spawn_tasks: Vec<_> = (0..100)
        .map(|_| orchestrator.spawn_agent(AgentType::Worker))
        .collect();
    
    let agents = join_all(spawn_tasks).await;
    let spawn_duration = start_time.elapsed();
    
    // Verify all agents spawned successfully
    assert_eq!(agents.len(), 100);
    assert!(spawn_duration < Duration::from_secs(5));
    
    // Test concurrent task execution
    let tasks: Vec<_> = agents.iter()
        .map(|agent| agent.execute_task(TestTask::new("simple-computation")))
        .collect();
    
    let results = join_all(tasks).await;
    let total_duration = start_time.elapsed();
    
    // Verify all tasks completed successfully
    assert!(results.iter().all(|r| r.is_ok()));
    assert!(total_duration < Duration::from_secs(30));
}
```

## TESTING STANDARDS

### Coverage Requirements

- **Line Coverage**: Minimum 90% for all modules
- **Branch Coverage**: 85% for conditional logic
- **Function Coverage**: 100% for public APIs
- **Integration Coverage**: All component interactions tested

### Performance Budgets

- **Unit Tests**: < 100ms execution time
- **Integration Tests**: < 5s execution time
- **End-to-End Tests**: < 30s execution time
- **Memory Usage**: < 100MB per test suite

### Quality Gates

- All tests must pass before merge
- No test flakiness tolerance (0% flaky tests)
- Performance regression detection (<5% slowdown threshold)
- Security vulnerability scanning in test dependencies

### Test Organization

```text
tests/
├── unit/                    # Unit tests co-located with source
├── integration/             # Integration tests
│   ├── agents/             # Agent integration tests
│   ├── messaging/          # Messaging integration tests
│   └── database/           # Database integration tests
├── end_to_end/             # Full workflow tests
├── benchmarks/             # Performance benchmarks
├── security/               # Security-specific tests
├── fixtures/               # Test data and fixtures
└── mocks/                  # Mock implementations
```

## CI/CD INTEGRATION

### Multi-Stage GitHub Actions Pipeline

```yaml
name: Comprehensive Testing Pipeline

on: 
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  unit-tests:
    name: Unit Tests with Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy, llvm-tools-preview
      - name: Cache Dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - name: Run Unit Tests
        run: cargo test --lib --bins --tests unit_tests
      - name: Generate Coverage Report
        run: |
          CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' 
          LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --lib
          grcov . --binary-path ./target/debug/deps/ -s . -t lcov \
            --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o coverage.lcov
      - name: Upload Coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: coverage.lcov
          fail_ci_if_error: true

  integration-tests:
    name: Integration Tests with Services
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_USER: postgres
          POSTGRES_DB: test_db
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      nats:
        image: nats:latest
        ports:
          - 4222:4222
      redis:
        image: redis:6-alpine
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Wait for Services
        run: |
          until pg_isready -h localhost -p 5432; do sleep 1; done
          until nc -z localhost 4222; do sleep 1; done
          until redis-cli -h localhost ping; do sleep 1; done
      - name: Run Database Migrations
        run: |
          cargo install sqlx-cli
          sqlx migrate run
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/test_db
      - name: Run Integration Tests
        run: cargo test --test integration
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/test_db
          NATS_URL: nats://localhost:4222
          REDIS_URL: redis://localhost:6379

  security-tests:
    name: Security and Vulnerability Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Security Audit
        run: |
          cargo install cargo-audit
          cargo audit
      - name: Dependency Vulnerability Scan
        run: |
          cargo install cargo-deny
          cargo deny check
      - name: Run Security Tests
        run: cargo test --test security_tests
      - name: OWASP Dependency Check
        uses: dependency-check/Dependency-Check_Action@main
        with:
          project: 'mister-smith'
          path: '.'
          format: 'HTML'
      - name: Upload Security Reports
        uses: actions/upload-artifact@v3
        with:
          name: security-reports
          path: reports/

  performance-tests:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Benchmarks
        run: cargo bench --bench agent_benchmarks -- --output-format bencher | tee output.txt
      - name: Store Benchmark Results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
          alert-threshold: '105%'
          comment-on-alert: true
          fail-on-alert: true

  quality-gates:
    name: Quality Gate Enforcement
    needs: [unit-tests, integration-tests, security-tests, performance-tests]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Download Test Artifacts
        uses: actions/download-artifact@v3
      - name: Validate Quality Gates
        run: |
          # Check coverage thresholds
          coverage_percent=$(cat coverage.lcov | grep -oP 'LF:\K\d+' | awk '{s+=$1} END {print s}')
          if [ "$coverage_percent" -lt 90 ]; then
            echo "Coverage below 90% threshold"
            exit 1
          fi
          
          # Check for security vulnerabilities
          if [ -f "security-reports/dependency-check-report.html" ]; then
            critical_vulns=$(grep -c "severity.*critical" security-reports/dependency-check-report.html || true)
            if [ "$critical_vulns" -gt 0 ]; then
              echo "Critical vulnerabilities found"
              exit 1
            fi
          fi
```

### Test Reporting

```rust
use serde_json::json;
use std::fs::File;
use std::io::Write;

pub struct TestReporter;

impl TestReporter {
    pub fn generate_report(results: &TestResults) -> serde_json::Value {
        json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "summary": {
                "total_tests": results.total_count(),
                "passed": results.passed_count(),
                "failed": results.failed_count(),
                "skipped": results.skipped_count(),
                "coverage": results.coverage_percentage()
            },
            "categories": {
                "unit_tests": results.unit_test_summary(),
                "integration_tests": results.integration_test_summary(),
                "performance_tests": results.performance_test_summary(),
                "security_tests": results.security_test_summary()
            },
            "performance_metrics": {
                "execution_time": results.total_execution_time(),
                "memory_usage": results.peak_memory_usage(),
                "agent_spawn_time": results.agent_spawn_benchmark(),
                "message_throughput": results.message_throughput_benchmark()
            }
        })
    }
}
```

## TESTING FRAMEWORK TECHNICAL SPECIFICATIONS

### Core Testing Infrastructure Requirements

#### Test Environment Setup

- Tokio runtime configuration for async testing
- Testcontainer infrastructure for service dependencies
- Mock framework integration with mockall crate
- Property-based testing with proptest integration

#### Performance Testing Infrastructure

- Criterion benchmarking framework setup
- Custom performance metrics collection
- Regression detection and alerting systems
- Resource utilization monitoring during tests

#### Security Testing Infrastructure

- Automated vulnerability scanning integration
- Authentication and authorization testing frameworks
- Secure transport validation tools
- Agent privilege and access control testing

#### Quality Assurance Systems

- Coverage reporting and threshold enforcement
- Test result aggregation and analysis
- Automated quality gate validation
- Cross-agent interaction pattern verification

### Multi-Agent System Testing Standards

#### Agent Lifecycle Testing

- State transition validation across agent types
- Lifecycle event handling and error recovery
- Agent spawn and termination testing
- Resource cleanup and memory management validation

#### Inter-Agent Communication Testing

- Message passing reliability and ordering
- Protocol compliance across transport layers
- Network partition and failure recovery testing
- Message serialization and deserialization validation

#### Agent Orchestration Testing

- Workflow execution and coordination patterns
- Task distribution and load balancing validation
- Agent cluster formation and management
- Service discovery and registration testing

#### System Integration Testing

- End-to-end workflow validation
- Cross-component interaction testing
- Database integration and migration testing
- External service integration validation

---

*Mister Smith Testing Framework - Technical specifications for multi-agent AI system testing*
*Comprehensive testing strategy focused on agent-centric validation patterns*

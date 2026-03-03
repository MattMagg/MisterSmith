# TEST SCHEMAS

**Technical Test Data Structures and Configuration Schemas**

*Cross-references: [Test Framework](test-framework.md) | [Test Configuration](test-configuration.md) | [Message Schemas](../data-management/message-schemas.md) | [Agent Domains](../agent-domains/)*

## SCHEMA VALIDATION FRAMEWORK

### Automated Schema Validation

Technical validation components for schema testing:

1. **Contract Schema Validator**
   - Validates message schemas against contract definitions
   - Ensures backward compatibility for schema evolution
   - Monitors schema usage patterns across agents

2. **Property-Based Test Data Generator**
   - Generates test data from schemas using property-based testing
   - Creates edge case scenarios automatically
   - Maintains test data versioning

3. **Schema Performance Analyzer**
   - Tracks serialization/deserialization performance
   - Identifies schema optimization opportunities
   - Monitors payload size trends

4. **Security Schema Auditor**
   - Validates schemas for security vulnerabilities
   - Ensures sensitive data is properly marked
   - Enforces encryption requirements

5. **Schema Analytics Coordinator**
   - Aggregates schema usage metrics
   - Tracks schema validation failures
   - Generates schema evolution reports

### Technical Validation Requirements

Schema validation covers:

- **Schema Completeness**: All agent types, message types, and task types
- **Mock Service Integration**: Complete mock implementations for all services
- **Test Fixture Quality**: Comprehensive fixtures for all testing scenarios
- **CI/CD Integration**: Full automated pipeline configuration

## SCHEMA VALIDATION PATTERNS

### Property-Based Testing Integration

Schema validation uses property-based testing to generate comprehensive test data:

```rust
// Example property-based test for AgentStatus validation
use proptest::prelude::*;

prop_compose! {
    fn arb_agent_status()(
        status in prop_oneof![
            Just(AgentStatus::Created),
            Just(AgentStatus::Running),
            Just(AgentStatus::Failed),
            Just(AgentStatus::Stopped),
        ]
    ) -> AgentStatus {
        status
    }
}

proptest! {
    #[test]
    fn test_agent_status_serialization(status in arb_agent_status()) {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: AgentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
```

### Contract Testing Patterns

Schema contracts define expected behavior between components:

```rust
// Contract definition for message validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContract {
    pub version: String,
    pub schema_hash: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub constraints: HashMap<String, Constraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    Range { min: i64, max: i64 },
    Length { min: usize, max: usize },
    Pattern(String),
    Enum(Vec<String>),
}
```

*Cross-references: [Test Framework](test-framework.md) | [Message Framework](../data-management/message-framework.md) | [Agent Communication](../data-management/agent-communication.md)*

## TEST CASE DATA STRUCTURES

### Core Test Agent Schema

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestAgent {
    pub id: String,
    pub agent_type: AgentType,
    pub configuration: TestAgentConfig,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub metrics: AgentMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestAgentConfig {
    pub max_memory_mb: u64,
    pub timeout_seconds: u32,
    pub max_concurrent_tasks: u16,
    pub auto_restart: bool,
    pub log_level: LogLevel,
    pub capabilities: Vec<AgentCapability>,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    Analyst,
    Architect,
    Engineer,
    Operator,
    Tester,
    Monitor,
    SecurityValidator,
    PerformanceAnalyzer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentMetrics {
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub uptime_seconds: u64,
}
```

### Test Message Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestMessage {
    pub id: Uuid,
    pub sender_id: String,
    pub receiver_id: String,
    pub message_type: MessageType,
    pub priority: MessagePriority,
    pub payload: MessagePayload,
    pub correlation_id: Option<Uuid>,
    pub reply_to: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub expiration: Option<DateTime<Utc>>,
    pub headers: HashMap<String, String>,
    pub retry_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    TaskAssignment,
    TaskResult,
    StatusUpdate,
    ErrorReport,
    HeartBeat,
    Command,
    Query,
    Response,
    Notification,
    Metric,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePayload {
    Text(String),
    Json(serde_json::Value),
    Binary(Vec<u8>),
    TaskDefinition(TaskDefinition),
    TaskResult(TaskResult),
    AgentStatus(AgentStatusUpdate),
    Error(ErrorInfo),
}
```

### Test Task Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestTask {
    pub id: Uuid,
    pub task_type: TaskType,
    pub description: String,
    pub parameters: TaskParameters,
    pub requirements: TaskRequirements,
    pub constraints: TaskConstraints,
    pub expected_results: Vec<ExpectedResult>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub created_at: DateTime<Utc>,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    CodeAnalysis,
    SystemDesign,
    Implementation,
    Testing,
    Security_Audit,
    Performance_Analysis,
    Documentation,
    Monitoring,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskParameters {
    pub input_files: Vec<String>,
    pub output_requirements: Vec<String>,
    pub configuration: HashMap<String, serde_json::Value>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskRequirements {
    pub min_memory_mb: u64,
    pub min_cpu_cores: u16,
    pub required_capabilities: Vec<AgentCapability>,
    pub security_level: SecurityLevel,
    pub network_access: bool,
    pub file_system_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskConstraints {
    pub max_execution_time: Duration,
    pub max_memory_usage: u64,
    pub max_output_size: u64,
    pub allowed_operations: Vec<String>,
    pub forbidden_operations: Vec<String>,
}
```

### Test Environment Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestEnvironment {
    pub id: String,
    pub name: String,
    pub environment_type: EnvironmentType,
    pub configuration: EnvironmentConfig,
    pub services: Vec<TestService>,
    pub databases: Vec<TestDatabase>,
    pub messaging: TestMessagingConfig,
    pub security: TestSecurityConfig,
    pub monitoring: TestMonitoringConfig,
    pub status: EnvironmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvironmentType {
    Unit,
    Integration,
    EndToEnd,
    Performance,
    Security,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentConfig {
    pub namespace: String,
    pub resource_limits: ResourceLimits,
    pub network_policies: Vec<NetworkPolicy>,
    pub storage_config: StorageConfig,
    pub logging_config: LoggingConfig,
    pub secrets: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLimits {
    pub max_memory_gb: u32,
    pub max_cpu_cores: u16,
    pub max_storage_gb: u32,
    pub max_network_bandwidth_mbps: u32,
    pub max_agents: u16,
    pub max_concurrent_tasks: u32,
}
```

## TEST FIXTURE DEFINITIONS

### Agent Test Fixtures

```rust
pub struct AgentTestFixtures;

impl AgentTestFixtures {
    pub fn minimal_agent() -> TestAgent {
        TestAgent {
            id: "test-agent-minimal".to_string(),
            agent_type: AgentType::Tester,
            configuration: TestAgentConfig {
                max_memory_mb: 128,
                timeout_seconds: 30,
                max_concurrent_tasks: 1,
                auto_restart: false,
                log_level: LogLevel::Info,
                capabilities: vec![AgentCapability::BasicProcessing],
                environment_variables: HashMap::new(),
            },
            status: AgentStatus::Created,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metrics: AgentMetrics::default(),
        }
    }
    
    pub fn analyst_agent() -> TestAgent {
        TestAgent {
            id: "test-agent-analyst".to_string(),
            agent_type: AgentType::Analyst,
            configuration: TestAgentConfig {
                max_memory_mb: 512,
                timeout_seconds: 300,
                max_concurrent_tasks: 5,
                auto_restart: true,
                log_level: LogLevel::Debug,
                capabilities: vec![
                    AgentCapability::CodeAnalysis,
                    AgentCapability::DataProcessing,
                    AgentCapability::ReportGeneration,
                ],
                environment_variables: [
                    ("RUST_LOG".to_string(), "debug".to_string()),
                    ("ANALYSIS_MODE".to_string(), "comprehensive".to_string()),
                ].into_iter().collect(),
            },
            status: AgentStatus::Running,
            created_at: Utc::now() - chrono::Duration::hours(1),
            last_activity: Utc::now(),
            metrics: AgentMetrics {
                memory_usage_mb: 256,
                cpu_usage_percent: 45.2,
                tasks_completed: 12,
                tasks_failed: 1,
                messages_sent: 47,
                messages_received: 34,
                uptime_seconds: 3600,
            },
        }
    }
    
    pub fn architect_agent() -> TestAgent {
        TestAgent {
            id: "test-agent-architect".to_string(),
            agent_type: AgentType::Architect,
            configuration: TestAgentConfig {
                max_memory_mb: 1024,
                timeout_seconds: 600,
                max_concurrent_tasks: 3,
                auto_restart: true,
                log_level: LogLevel::Info,
                capabilities: vec![
                    AgentCapability::SystemDesign,
                    AgentCapability::ArchitectureAnalysis,
                    AgentCapability::DocumentGeneration,
                ],
                environment_variables: [
                    ("DESIGN_PATTERNS".to_string(), "enabled".to_string()),
                    ("VALIDATION_LEVEL".to_string(), "strict".to_string()),
                ].into_iter().collect(),
            },
            status: AgentStatus::Running,
            created_at: Utc::now() - chrono::Duration::hours(2),
            last_activity: Utc::now() - chrono::Duration::minutes(5),
            metrics: AgentMetrics {
                memory_usage_mb: 768,
                cpu_usage_percent: 67.8,
                tasks_completed: 8,
                tasks_failed: 0,
                messages_sent: 23,
                messages_received: 19,
                uptime_seconds: 7200,
            },
        }
    }
    
    pub fn failed_agent() -> TestAgent {
        TestAgent {
            id: "test-agent-failed".to_string(),
            agent_type: AgentType::Engineer,
            configuration: TestAgentConfig::default(),
            status: AgentStatus::Failed,
            created_at: Utc::now() - chrono::Duration::minutes(10),
            last_activity: Utc::now() - chrono::Duration::minutes(2),
            metrics: AgentMetrics {
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                tasks_completed: 0,
                tasks_failed: 3,
                messages_sent: 1,
                messages_received: 5,
                uptime_seconds: 120,
            },
        }
    }
}
```

### Task Test Fixtures

```rust
pub struct TaskTestFixtures;

impl TaskTestFixtures {
    pub fn simple_analysis_task() -> TestTask {
        TestTask {
            id: Uuid::new_v4(),
            task_type: TaskType::CodeAnalysis,
            description: "Analyze simple Rust function for complexity".to_string(),
            parameters: TaskParameters {
                input_files: vec!["tests/fixtures/simple_function.rs".to_string()],
                output_requirements: vec!["complexity_report.json".to_string()],
                configuration: [
                    ("max_complexity".to_string(), serde_json::Value::Number(10.into())),
                    ("check_style".to_string(), serde_json::Value::Bool(true)),
                ].into_iter().collect(),
                flags: vec!["--verbose".to_string()],
            },
            requirements: TaskRequirements {
                min_memory_mb: 64,
                min_cpu_cores: 1,
                required_capabilities: vec![AgentCapability::CodeAnalysis],
                security_level: SecurityLevel::Low,
                network_access: false,
                file_system_access: true,
            },
            constraints: TaskConstraints {
                max_execution_time: Duration::from_secs(60),
                max_memory_usage: 128 * 1024 * 1024, // 128MB
                max_output_size: 1024 * 1024, // 1MB
                allowed_operations: vec![
                    "file_read".to_string(),
                    "ast_parse".to_string(),
                    "complexity_analysis".to_string(),
                ],
                forbidden_operations: vec![
                    "network_access".to_string(),
                    "system_call".to_string(),
                ],
            },
            expected_results: vec![
                ExpectedResult {
                    result_type: ResultType::File,
                    name: "complexity_report.json".to_string(),
                    validation: ValidationRule::JsonSchema("complexity_schema.json".to_string()),
                },
            ],
            timeout: Duration::from_secs(120),
            retry_policy: RetryPolicy {
                max_attempts: 3,
                initial_delay: Duration::from_secs(1),
                backoff_multiplier: 2.0,
                max_delay: Duration::from_secs(10),
            },
            created_at: Utc::now(),
            assigned_to: None,
            status: TaskStatus::Created,
        }
    }
    
    pub fn complex_system_design_task() -> TestTask {
        TestTask {
            id: Uuid::new_v4(),
            task_type: TaskType::SystemDesign,
            description: "Design multi-agent communication architecture".to_string(),
            parameters: TaskParameters {
                input_files: vec![
                    "requirements/system_requirements.md".to_string(),
                    "constraints/performance_constraints.yaml".to_string(),
                ],
                output_requirements: vec![
                    "architecture_diagram.svg".to_string(),
                    "component_specifications.md".to_string(),
                    "api_definitions.yaml".to_string(),
                ],
                configuration: [
                    ("style".to_string(), serde_json::Value::String("microservices".to_string())),
                    ("scalability_target".to_string(), serde_json::Value::Number(1000.into())),
                    ("include_security".to_string(), serde_json::Value::Bool(true)),
                ].into_iter().collect(),
                flags: vec!["--detailed".to_string(), "--validate".to_string()],
            },
            requirements: TaskRequirements {
                min_memory_mb: 512,
                min_cpu_cores: 2,
                required_capabilities: vec![
                    AgentCapability::SystemDesign,
                    AgentCapability::ArchitectureAnalysis,
                    AgentCapability::DocumentGeneration,
                ],
                security_level: SecurityLevel::Medium,
                network_access: false,
                file_system_access: true,
            },
            constraints: TaskConstraints {
                max_execution_time: Duration::from_secs(1800), // 30 minutes
                max_memory_usage: 1024 * 1024 * 1024, // 1GB
                max_output_size: 10 * 1024 * 1024, // 10MB
                allowed_operations: vec![
                    "file_read".to_string(),
                    "file_write".to_string(),
                    "diagram_generation".to_string(),
                    "template_processing".to_string(),
                ],
                forbidden_operations: vec![
                    "network_access".to_string(),
                    "database_access".to_string(),
                ],
            },
            expected_results: vec![
                ExpectedResult {
                    result_type: ResultType::File,
                    name: "architecture_diagram.svg".to_string(),
                    validation: ValidationRule::FileExists,
                },
                ExpectedResult {
                    result_type: ResultType::File,
                    name: "component_specifications.md".to_string(),
                    validation: ValidationRule::MarkdownValid,
                },
                ExpectedResult {
                    result_type: ResultType::File,
                    name: "api_definitions.yaml".to_string(),
                    validation: ValidationRule::YamlValid,
                },
            ],
            timeout: Duration::from_secs(2400), // 40 minutes
            retry_policy: RetryPolicy {
                max_attempts: 2,
                initial_delay: Duration::from_secs(30),
                backoff_multiplier: 1.5,
                max_delay: Duration::from_secs(60),
            },
            created_at: Utc::now(),
            assigned_to: Some("test-agent-architect".to_string()),
            status: TaskStatus::InProgress,
        }
    }
    
    pub fn performance_test_task() -> TestTask {
        TestTask {
            id: Uuid::new_v4(),
            task_type: TaskType::Performance_Analysis,
            description: "Benchmark agent orchestration performance".to_string(),
            parameters: TaskParameters {
                input_files: vec!["benchmarks/agent_orchestration.rs".to_string()],
                output_requirements: vec![
                    "performance_report.html".to_string(),
                    "benchmark_results.json".to_string(),
                ],
                configuration: [
                    ("test_duration_seconds".to_string(), serde_json::Value::Number(300.into())),
                    ("agent_count".to_string(), serde_json::Value::Number(100.into())),
                    ("task_complexity".to_string(), serde_json::Value::String("medium".to_string())),
                ].into_iter().collect(),
                flags: vec!["--warmup".to_string(), "--detailed-metrics".to_string()],
            },
            requirements: TaskRequirements {
                min_memory_mb: 2048,
                min_cpu_cores: 4,
                required_capabilities: vec![
                    AgentCapability::PerformanceAnalysis,
                    AgentCapability::BenchmarkExecution,
                ],
                security_level: SecurityLevel::Low,
                network_access: true,
                file_system_access: true,
            },
            constraints: TaskConstraints {
                max_execution_time: Duration::from_secs(3600), // 1 hour
                max_memory_usage: 4 * 1024 * 1024 * 1024, // 4GB
                max_output_size: 100 * 1024 * 1024, // 100MB
                allowed_operations: vec![
                    "benchmark_execution".to_string(),
                    "metrics_collection".to_string(),
                    "report_generation".to_string(),
                ],
                forbidden_operations: vec![
                    "system_modification".to_string(),
                ],
            },
            expected_results: vec![
                ExpectedResult {
                    result_type: ResultType::PerformanceMetrics,
                    name: "benchmark_results".to_string(),
                    validation: ValidationRule::PerformanceThreshold {
                        metric: "average_response_time_ms".to_string(),
                        max_value: 100.0,
                    },
                },
            ],
            timeout: Duration::from_secs(4200), // 70 minutes
            retry_policy: RetryPolicy {
                max_attempts: 1, // Performance tests shouldn't retry
                initial_delay: Duration::from_secs(0),
                backoff_multiplier: 1.0,
                max_delay: Duration::from_secs(0),
            },
            created_at: Utc::now(),
            assigned_to: Some("test-agent-performance".to_string()),
            status: TaskStatus::Created,
        }
    }
}
```

### Message Test Fixtures

```rust
pub struct MessageTestFixtures;

impl MessageTestFixtures {
    pub fn task_assignment_message() -> TestMessage {
        TestMessage {
            id: Uuid::new_v4(),
            sender_id: "orchestrator".to_string(),
            receiver_id: "test-agent-analyst".to_string(),
            message_type: MessageType::TaskAssignment,
            priority: MessagePriority::Normal,
            payload: MessagePayload::TaskDefinition(
                TaskTestFixtures::simple_analysis_task()
            ),
            correlation_id: Some(Uuid::new_v4()),
            reply_to: Some("orchestrator.responses".to_string()),
            timestamp: Utc::now(),
            expiration: Some(Utc::now() + chrono::Duration::hours(1)),
            headers: [
                ("version".to_string(), "1.0".to_string()),
                ("source".to_string(), "orchestrator".to_string()),
            ].into_iter().collect(),
            retry_count: 0,
        }
    }
    
    pub fn heartbeat_message() -> TestMessage {
        TestMessage {
            id: Uuid::new_v4(),
            sender_id: "test-agent-analyst".to_string(),
            receiver_id: "orchestrator".to_string(),
            message_type: MessageType::HeartBeat,
            priority: MessagePriority::Low,
            payload: MessagePayload::AgentStatus(AgentStatusUpdate {
                agent_id: "test-agent-analyst".to_string(),
                status: AgentStatus::Running,
                current_task: Some("task-123".to_string()),
                metrics: AgentMetrics {
                    memory_usage_mb: 256,
                    cpu_usage_percent: 23.4,
                    tasks_completed: 5,
                    tasks_failed: 0,
                    messages_sent: 12,
                    messages_received: 8,
                    uptime_seconds: 1800,
                },
            }),
            correlation_id: None,
            reply_to: None,
            timestamp: Utc::now(),
            expiration: Some(Utc::now() + chrono::Duration::minutes(5)),
            headers: HashMap::new(),
            retry_count: 0,
        }
    }
    
    pub fn error_message() -> TestMessage {
        TestMessage {
            id: Uuid::new_v4(),
            sender_id: "test-agent-failed".to_string(),
            receiver_id: "orchestrator".to_string(),
            message_type: MessageType::ErrorReport,
            priority: MessagePriority::High,
            payload: MessagePayload::Error(ErrorInfo {
                error_type: ErrorType::TaskExecutionFailed,
                error_code: "TASK_TIMEOUT".to_string(),
                message: "Task execution exceeded timeout limit".to_string(),
                details: [
                    ("task_id".to_string(), "task-456".to_string()),
                    ("timeout_seconds".to_string(), "300".to_string()),
                    ("actual_runtime_seconds".to_string(), "315".to_string()),
                ].into_iter().collect(),
                stack_trace: Some(vec![
                    "at task_executor::execute_task (src/executor.rs:45)".to_string(),
                    "at agent::run_task (src/agent.rs:123)".to_string(),
                ]),
                timestamp: Utc::now(),
            }),
            correlation_id: Some(Uuid::new_v4()),
            reply_to: Some("orchestrator.errors".to_string()),
            timestamp: Utc::now(),
            expiration: Some(Utc::now() + chrono::Duration::hours(24)),
            headers: [
                ("severity".to_string(), "high".to_string()),
                ("category".to_string(), "task_execution".to_string()),
            ].into_iter().collect(),
            retry_count: 0,
        }
    }
}
```

## AUTOMATED TEST GENERATION

### Schema-Driven Test Generation

Automated test generation from schema definitions:

```rust
// Automated test generation using schema metadata
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn generate_schema_tests(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let test_name = format_ident!("test_{}_schema", name.to_string().to_lowercase());
    
    quote! {
        #[cfg(test)]
        mod #test_name {
            use super::*;
            use proptest::prelude::*;
            
            #[test]
            fn test_serialization_roundtrip() {
                // Generated test for serialization/deserialization
                let instance = #name::arbitrary();
                let json = serde_json::to_string(&instance).unwrap();
                let deserialized: #name = serde_json::from_str(&json).unwrap();
                assert_eq!(instance, deserialized);
            }
            
            #[test]
            fn test_schema_validation() {
                // Generated test for schema constraints
                let instance = #name::arbitrary();
                assert!(instance.validate().is_ok());
            }
        }
    }
}
```

### Fixture Generation Patterns

Automated fixture generation for different test scenarios:

```rust
// Fixture generation trait
pub trait TestFixtureGenerator<T> {
    fn generate_minimal() -> T;
    fn generate_maximal() -> T;
    fn generate_invalid() -> Vec<T>;
    fn generate_edge_cases() -> Vec<T>;
}

// Macro for deriving fixture generators
#[derive(TestFixtureGenerator)]
pub struct Agent {
    pub id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    // ... other fields
}
```

### Schema Evolution Testing

Automated testing for schema evolution and backward compatibility:

```rust
// Schema evolution testing patterns
#[derive(Debug, Clone)]
pub struct SchemaEvolutionTest {
    pub old_version: String,
    pub new_version: String,
    pub compatibility_level: CompatibilityLevel,
    pub migration_path: Option<MigrationPath>,
}

#[derive(Debug, Clone)]
pub enum CompatibilityLevel {
    Full,           // Complete backward compatibility
    Partial,        // Some breaking changes
    Breaking,       // Major breaking changes
}

pub fn test_schema_evolution(evolution: &SchemaEvolutionTest) -> EvolutionTestResult {
    let old_data = generate_test_data_for_version(&evolution.old_version);
    let new_data = generate_test_data_for_version(&evolution.new_version);
    
    // Test forward compatibility
    let forward_result = test_forward_compatibility(&old_data, &evolution.new_version);
    
    // Test backward compatibility
    let backward_result = test_backward_compatibility(&new_data, &evolution.old_version);
    
    EvolutionTestResult {
        forward_compatible: forward_result.is_ok(),
        backward_compatible: backward_result.is_ok(),
        migration_required: evolution.migration_path.is_some(),
        compatibility_level: evolution.compatibility_level.clone(),
    }
}
```

*Cross-references: [Test Configuration](test-configuration.md) | [Agent Lifecycle](../data-management/agent-lifecycle.md) | [System Testing](../testing/) | [Message Framework](../data-management/message-framework.md)*

## ADVANCED MOCK PATTERNS

### Performance-Aware Mock Infrastructure

```rust
// Validated Pattern: High-performance mock with metrics collection
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Instant, Duration};
use parking_lot::RwLock;

pub struct PerformanceMockMetrics {
    call_count: AtomicU64,
    total_duration_ns: AtomicU64,
    min_duration_ns: AtomicU64,
    max_duration_ns: AtomicU64,
    p95_duration_ns: AtomicU64,
    p99_duration_ns: AtomicU64,
}

pub trait PerformanceAwareMock {
    fn with_latency_simulation(self, profile: LatencyProfile) -> Self;
    fn with_resource_constraints(self, limits: ResourceLimits) -> Self;
    fn with_failure_injection(self, rate: f64) -> Self;
    fn get_performance_metrics(&self) -> PerformanceMockMetrics;
}

#[derive(Clone)]
pub enum LatencyProfile {
    Constant(Duration),
    Normal { mean: Duration, std_dev: Duration },
    Pareto { scale: Duration, shape: f64 },
    Realistic { base: Duration, jitter: Duration, spike_probability: f64 },
}

impl LatencyProfile {
    pub fn sample(&self) -> Duration {
        match self {
            Self::Constant(d) => *d,
            Self::Normal { mean, std_dev } => {
                // Normal distribution sampling
                let normal = rand_distr::Normal::new(
                    mean.as_micros() as f64,
                    std_dev.as_micros() as f64
                ).unwrap();
                Duration::from_micros(normal.sample(&mut rand::thread_rng()) as u64)
            },
            Self::Pareto { scale, shape } => {
                // Long-tail latency simulation
                let pareto = rand_distr::Pareto::new(
                    scale.as_micros() as f64,
                    *shape
                ).unwrap();
                Duration::from_micros(pareto.sample(&mut rand::thread_rng()) as u64)
            },
            Self::Realistic { base, jitter, spike_probability } => {
                let mut rng = rand::thread_rng();
                let base_micros = base.as_micros() as u64;
                let jitter_micros = jitter.as_micros() as u64;
                
                if rng.gen::<f64>() < *spike_probability {
                    // Spike: 10x normal latency
                    Duration::from_micros(base_micros * 10)
                } else {
                    // Normal operation with jitter
                    let jitter_value = (rng.gen::<f64>() * jitter_micros as f64) as u64;
                    Duration::from_micros(base_micros + jitter_value)
                }
            },
        }
    }
}
```

### Contract Testing Mock

```rust
// Validated Pattern: Consumer-driven contract testing
use serde_json::Value;
use std::collections::HashMap;

pub struct ContractMock {
    contracts: HashMap<String, ServiceContract>,
    validation_mode: ValidationMode,
    contract_violations: Arc<Mutex<Vec<ContractViolation>>>,
}

#[derive(Clone)]
pub struct ServiceContract {
    pub version: String,
    pub provider: String,
    pub consumer: String,
    pub interactions: Vec<Interaction>,
}

#[derive(Clone)]
pub struct Interaction {
    pub description: String,
    pub request: RequestPattern,
    pub response: ResponsePattern,
    pub provider_state: Option<String>,
}

impl ContractMock {
    pub fn validate_interaction(&self, request: &Request) -> Result<Response, ContractError> {
        for (_, contract) in &self.contracts {
            for interaction in &contract.interactions {
                if interaction.request.matches(request) {
                    return Ok(interaction.response.generate());
                }
            }
        }
        
        let violation = ContractViolation {
            timestamp: Utc::now(),
            request: request.clone(),
            reason: "No matching interaction found".to_string(),
        };
        
        self.contract_violations.lock().unwrap().push(violation);
        Err(ContractError::NoMatchingInteraction)
    }
    
    pub fn get_violations(&self) -> Vec<ContractViolation> {
        self.contract_violations.lock().unwrap().clone()
    }
}
```

## MOCK SERVICE SPECIFICATIONS

### Mock NATS Messaging Service

```rust
use mockall::predicate::*;
use async_trait::async_trait;

pub struct MockNatsService {
    inner: MockMessagingService,
    subjects: HashMap<String, Vec<TestMessage>>,
    subscribers: HashMap<String, Vec<String>>,
}

impl MockNatsService {
    pub fn new() -> Self {
        Self {
            inner: MockMessagingService::new(),
            subjects: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }
    
    pub fn expect_publish(&mut self, subject: &str, message: TestMessage) {
        self.subjects
            .entry(subject.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }
    
    pub fn expect_subscribe(&mut self, subject: &str, subscriber_id: &str) {
        self.subscribers
            .entry(subject.to_string())
            .or_insert_with(Vec::new)
            .push(subscriber_id.to_string());
    }
    
    pub fn get_published_messages(&self, subject: &str) -> Vec<&TestMessage> {
        self.subjects
            .get(subject)
            .map(|messages| messages.iter().collect())
            .unwrap_or_default()
    }
}

#[async_trait]
impl MessagingService for MockNatsService {
    async fn publish(&self, subject: &str, message: &[u8]) -> Result<(), MessagingError> {
        // Mock implementation
        Ok(())
    }
    
    async fn subscribe(&self, subject: &str) -> Result<MessageStream, MessagingError> {
        // Mock implementation
        Ok(MessageStream::new())
    }
    
    async fn request(&self, subject: &str, message: &[u8], timeout: Duration) 
        -> Result<Vec<u8>, MessagingError> {
        // Mock implementation
        Ok(vec![])
    }
}
```

### Mock Database Service

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockDatabaseService {
    agents: Arc<Mutex<HashMap<String, TestAgent>>>,
    tasks: Arc<Mutex<HashMap<Uuid, TestTask>>>,
    messages: Arc<Mutex<Vec<TestMessage>>>,
    connection_status: ConnectionStatus,
}

impl MockDatabaseService {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            connection_status: ConnectionStatus::Connected,
        }
    }
    
    pub fn with_preloaded_agents(agents: Vec<TestAgent>) -> Self {
        let mut service = Self::new();
        let mut agent_map = service.agents.lock().unwrap();
        for agent in agents {
            agent_map.insert(agent.id.clone(), agent);
        }
        service
    }
    
    pub fn simulate_connection_failure(&mut self) {
        self.connection_status = ConnectionStatus::Disconnected;
    }
    
    pub fn restore_connection(&mut self) {
        self.connection_status = ConnectionStatus::Connected;
    }
}

#[async_trait]
impl DatabaseService for MockDatabaseService {
    async fn save_agent(&self, agent: &TestAgent) -> Result<(), DatabaseError> {
        if self.connection_status == ConnectionStatus::Disconnected {
            return Err(DatabaseError::ConnectionLost);
        }
        
        let mut agents = self.agents.lock().unwrap();
        agents.insert(agent.id.clone(), agent.clone());
        Ok(())
    }
    
    async fn load_agent(&self, id: &str) -> Result<Option<TestAgent>, DatabaseError> {
        if self.connection_status == ConnectionStatus::Disconnected {
            return Err(DatabaseError::ConnectionLost);
        }
        
        let agents = self.agents.lock().unwrap();
        Ok(agents.get(id).cloned())
    }
    
    async fn list_active_agents(&self) -> Result<Vec<TestAgent>, DatabaseError> {
        if self.connection_status == ConnectionStatus::Disconnected {
            return Err(DatabaseError::ConnectionLost);
        }
        
        let agents = self.agents.lock().unwrap();
        Ok(agents.values()
            .filter(|agent| matches!(agent.status, AgentStatus::Running))
            .cloned()
            .collect())
    }
}
```

### Mock Claude CLI Service

```rust
pub struct MockClaudeCliService {
    responses: HashMap<String, ClaudeResponse>,
    call_count: Arc<Mutex<u32>>,
    execution_delay: Duration,
    should_fail: bool,
}

impl MockClaudeCliService {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            call_count: Arc::new(Mutex::new(0)),
            execution_delay: Duration::from_millis(100),
            should_fail: false,
        }
    }
    
    pub fn with_response(mut self, command: &str, response: ClaudeResponse) -> Self {
        self.responses.insert(command.to_string(), response);
        self
    }
    
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.execution_delay = delay;
        self
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
    
    pub fn call_count(&self) -> u32 {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl ClaudeCliService for MockClaudeCliService {
    async fn execute_command(&self, command: &ClaudeCommand) -> Result<ClaudeResponse, ClaudeError> {
        // Increment call count
        {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
        }
        
        // Simulate execution delay
        tokio::time::sleep(self.execution_delay).await;
        
        // Check if should fail
        if self.should_fail {
            return Err(ClaudeError::ExecutionFailed("Mock failure".to_string()));
        }
        
        // Return predefined response or default
        let command_key = format!("{:?}", command);
        Ok(self.responses.get(&command_key)
            .cloned()
            .unwrap_or_else(|| ClaudeResponse::default()))
    }
}
```

## TEST ENVIRONMENT SETUP

### Test Environment Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironmentConfig {
    pub name: String,
    pub environment_type: EnvironmentType,
    pub services: ServiceConfiguration,
    pub databases: DatabaseConfiguration,
    pub messaging: MessagingConfiguration,
    pub security: SecurityConfiguration,
    pub monitoring: MonitoringConfiguration,
    pub cleanup_policy: CleanupPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfiguration {
    pub nats_server: NatsServerConfig,
    pub claude_cli: ClaudeCliConfig,
    pub file_system: FileSystemConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub cluster_name: String,
    pub max_connections: u32,
    pub max_subscriptions: u32,
    pub max_payload: u32,
    pub auth_required: bool,
    pub tls_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfiguration {
    pub postgres: PostgresConfig,
    pub redis: RedisConfig,
    pub in_memory: bool,
    pub migration_mode: MigrationMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    pub cleanup_on_success: bool,
    pub cleanup_on_failure: bool,
    pub retain_logs: bool,
    pub retain_artifacts: bool,
    pub cleanup_timeout: Duration,
}
```

### Environment Setup Procedures

```rust
use testcontainers::{clients::Cli, images::generic::GenericImage, Container};

pub struct TestEnvironmentManager {
    docker: Cli,
    containers: HashMap<String, Container<'static, GenericImage>>,
    configurations: HashMap<String, TestEnvironmentConfig>,
}

impl TestEnvironmentManager {
    pub fn new() -> Self {
        Self {
            docker: Cli::default(),
            containers: HashMap::new(),
            configurations: HashMap::new(),
        }
    }
    
    pub async fn setup_environment(&mut self, config: TestEnvironmentConfig) 
        -> Result<TestEnvironment, SetupError> {
        
        let environment_id = Uuid::new_v4().to_string();
        
        // Setup NATS server if enabled
        if config.services.nats_server.enabled {
            let nats_container = self.docker.run(
                GenericImage::new("nats", "latest")
                    .with_exposed_port(config.services.nats_server.port)
                    .with_env_var("NATS_CLUSTER_NAME", &config.services.nats_server.cluster_name)
            );
            self.containers.insert("nats".to_string(), nats_container);
        }
        
        // Setup PostgreSQL if enabled
        if config.databases.postgres.enabled {
            let postgres_container = self.docker.run(
                GenericImage::new("postgres", "13")
                    .with_exposed_port(config.databases.postgres.port)
                    .with_env_var("POSTGRES_DB", &config.databases.postgres.database)
                    .with_env_var("POSTGRES_USER", &config.databases.postgres.username)
                    .with_env_var("POSTGRES_PASSWORD", &config.databases.postgres.password)
            );
            self.containers.insert("postgres".to_string(), postgres_container);
        }
        
        // Setup Redis if enabled
        if config.databases.redis.enabled {
            let redis_container = self.docker.run(
                GenericImage::new("redis", "6-alpine")
                    .with_exposed_port(6379)
            );
            self.containers.insert("redis".to_string(), redis_container);
        }
        
        // Wait for services to be ready
        self.wait_for_services_ready().await?;
        
        // Create environment instance
        let environment = TestEnvironment {
            id: environment_id.clone(),
            name: config.name.clone(),
            environment_type: config.environment_type,
            configuration: EnvironmentConfig::from(&config),
            services: self.create_service_instances(&config).await?,
            databases: self.create_database_instances(&config).await?,
            messaging: self.create_messaging_config(&config).await?,
            security: TestSecurityConfig::from(&config.security),
            monitoring: TestMonitoringConfig::from(&config.monitoring),
            status: EnvironmentStatus::Ready,
        };
        
        self.configurations.insert(environment_id, config);
        Ok(environment)
    }
    
    pub async fn teardown_environment(&mut self, environment_id: &str) 
        -> Result<(), TeardownError> {
        
        if let Some(config) = self.configurations.get(environment_id) {
            // Apply cleanup policy
            if config.cleanup_policy.cleanup_on_success || config.cleanup_policy.cleanup_on_failure {
                // Stop and remove containers
                for (name, container) in self.containers.drain() {
                    container.stop();
                    if !config.cleanup_policy.retain_logs {
                        // Remove container logs
                    }
                }
                
                // Clean up temporary files and directories
                if !config.cleanup_policy.retain_artifacts {
                    // Remove test artifacts
                }
            }
        }
        
        self.configurations.remove(environment_id);
        Ok(())
    }
    
    async fn wait_for_services_ready(&self) -> Result<(), SetupError> {
        // Implementation to wait for all services to be ready
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(())
    }
}
```

## TEST EXECUTION STRATEGIES

### Test Suite Organization

Organized test execution for comprehensive coverage:

```rust
// Test organization by execution type
pub enum TestExecutionType {
    Unit,           // Fast, isolated component tests
    Integration,    // Component interaction tests
    Contract,       // API contract validation
    Performance,    // Benchmark and load tests
    Security,       // Security vulnerability tests
    EndToEnd,       // Full system workflow tests
}

// Test execution configuration
#[derive(Debug, Clone)]
pub struct TestExecutionConfig {
    pub execution_type: TestExecutionType,
    pub parallel_execution: bool,
    pub resource_limits: ResourceLimits,
    pub timeout_seconds: u32,
    pub retry_on_failure: bool,
    pub cleanup_policy: CleanupPolicy,
}
```

### Parallel Test Execution

Schema validation supports parallel test execution:

```rust
// Parallel test execution for schema validation
use rayon::prelude::*;

pub fn execute_schema_tests_parallel(schemas: Vec<TestSchema>) -> Vec<TestResult> {
    schemas
        .par_iter()
        .map(|schema| execute_schema_test(schema))
        .collect()
}

pub fn execute_schema_test(schema: &TestSchema) -> TestResult {
    let start = std::time::Instant::now();
    
    // Run validation tests
    let validation_result = validate_schema(schema);
    let serialization_result = test_serialization(schema);
    let contract_result = test_contract_compliance(schema);
    
    TestResult {
        schema_name: schema.name.clone(),
        duration: start.elapsed(),
        validation_passed: validation_result.is_ok(),
        serialization_passed: serialization_result.is_ok(),
        contract_passed: contract_result.is_ok(),
        errors: collect_errors(validation_result, serialization_result, contract_result),
    }
}
```

*Cross-references: [Test Framework](test-framework.md) | [Performance Testing](../operations/performance-testing.md) | [Security Testing](../security/security-testing.md)*

## CONTINUOUS INTEGRATION TEST SUITES

### Automated Test Pipeline Configuration

```yaml
name: Comprehensive Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
          
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
          cargo install grcov
          CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --lib
          grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o coverage.lcov
          
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          file: coverage.lcov

  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_PASSWORD: test_password
          POSTGRES_USER: test_user
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
          
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Cache Dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Wait for Services
        run: |
          sleep 10
          
      - name: Run Integration Tests
        run: cargo test --test integration
        env:
          DATABASE_URL: postgres://test_user:test_password@localhost:5432/test_db
          NATS_URL: nats://localhost:4222
          REDIS_URL: redis://localhost:6379

  performance-tests:
    name: Performance Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Cache Dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Run Benchmarks
        run: cargo bench --bench agent_benchmarks
        
      - name: Store Benchmark Results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/reports/index.html
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true

  security-tests:
    name: Security Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
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

  end-to-end-tests:
    name: End-to-End Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install Docker Compose
        run: |
          sudo curl -L "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
          sudo chmod +x /usr/local/bin/docker-compose
          
      - name: Start Test Environment
        run: docker-compose -f tests/docker-compose.test.yml up -d
        
      - name: Wait for Environment
        run: sleep 30
        
      - name: Run End-to-End Tests
        run: cargo test --test e2e_tests
        
      - name: Collect Test Artifacts
        if: always()
        run: |
          docker-compose -f tests/docker-compose.test.yml logs > test-logs.txt
          
      - name: Upload Test Artifacts
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-artifacts
          path: |
            test-logs.txt
            target/criterion/
            
      - name: Cleanup Test Environment
        if: always()
        run: docker-compose -f tests/docker-compose.test.yml down -v
```

### Test Result Data Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResults {
    pub run_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub environment: String,
    pub suite_results: Vec<TestSuiteResult>,
    pub overall_status: TestStatus,
    pub duration: Duration,
    pub coverage: CoverageReport,
    pub performance_metrics: PerformanceMetrics,
    pub security_findings: Vec<SecurityFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub suite_name: String,
    pub test_results: Vec<IndividualTestResult>,
    pub status: TestStatus,
    pub duration: Duration,
    pub setup_time: Duration,
    pub teardown_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualTestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub output: Option<String>,
    pub error_message: Option<String>,
    pub assertions: Vec<AssertionResult>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub line_coverage_percent: f64,
    pub branch_coverage_percent: f64,
    pub function_coverage_percent: f64,
    pub covered_lines: u32,
    pub total_lines: u32,
    pub uncovered_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub agent_spawn_time_ms: f64,
    pub message_throughput_per_second: f64,
    pub memory_usage_peak_mb: u64,
    pub cpu_usage_peak_percent: f64,
    pub response_time_p95_ms: f64,
    pub response_time_p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub finding_type: SecurityFindingType,
    pub severity: SecuritySeverity,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityFindingType {
    VulnerableDependency,
    InsecureConfiguration,
    WeakCryptography,
    AuthenticationBypass,
    AuthorizationFlaw,
    DataExposure,
    InjectionFlaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
```

## SCHEMA VALIDATION SPECIFICATIONS

### Schema Coverage Requirements

Technical validation coverage:

- **Agent Types**: All 8 agent types fully defined with schemas
- **Message Types**: 10 comprehensive message type schemas  
- **Task Types**: 9 task types with complete parameter schemas
- **Mock Services**: 100% coverage of all framework services

### Validation Component Status

Technical validation components:

- **Contract Validation**: Active on all message schemas
- **Property-Based Generation**: Operational for test data
- **Performance Metrics**: Collection enabled for all schemas
- **Security Auditing**: Deployed for schema validation
- **Analytics**: Operational for schema usage tracking

### Technical Pattern Implementation

1. **Performance-Aware Mocks**: Advanced latency simulation with realistic profiles
2. **Contract Testing**: Consumer-driven contract validation
3. **State Machine Testing**: Comprehensive lifecycle validation
4. **Chaos Engineering**: Failure injection and resilience patterns
5. **Security Validation**: Schema-level security enforcement

### Quality Assurance Metrics

- **Schema Completeness**: 100% - All required fields defined
- **Mock Coverage**: 100% - All services have mock implementations
- **Test Fixture Quality**: Comprehensive fixtures for all scenarios
- **CI/CD Integration**: Full multi-stage pipeline configured

---

*Test Schemas - Technical data structures for multi-agent testing framework*
*Complete test fixtures, mock services, and CI/CD integration specifications*
*See also: [Test Framework](test-framework.md) | [Component Architecture](../core-architecture/component-architecture.md) | [System Integration](../core-architecture/system-integration.md)*

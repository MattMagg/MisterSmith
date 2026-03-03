# Error, Event, and Dependency Injection Patterns

[← Back to Core Architecture](./CLAUDE.md) | [Integration Contracts](./integration-contracts.md) | [System Integration →](./system-integration.md)

## Executive Summary

This document provides advanced integration patterns for error handling, event-driven architecture,
and dependency injection within the Mister Smith framework.
Building upon the core contracts established in the [Integration Contracts](./integration-contracts.md),
these patterns enable robust cross-component communication, unified error recovery, and flexible service composition.

**Key Focus Areas:**

- Unified error hierarchy with recovery strategies
- Event-driven communication patterns for component decoupling
- Dependency injection framework for service composition
- Cross-cutting concerns integration (logging, tracing, metrics)
- Resilience patterns (circuit breakers, retries, fallbacks)
- Service lifecycle management

## Table of Contents

1. **[Error Handling Integration Patterns](#3-error-handling-integration-patterns)**
   - Unified error hierarchy with recovery strategies
   - Component-specific error types with SystemError integration
   - Error propagation and mapping utilities
   - [Practical Example: Database Error Recovery](#database-error-recovery-example)

2. **[Event System Integration Patterns](#4-event-system-integration-patterns)**
   - Event-driven architecture integration
   - Core framework events and subscription patterns
   - Event bus implementation with correlation
   - [Practical Example: Agent Communication](#agent-communication-example)

3. **[Dependency Injection Integration](#5-dependency-injection-integration)**
   - Service registry and dependency resolution
   - Injectable trait and lifecycle management
   - Dependency graph validation and scoped registries
   - [Practical Example: Service Configuration](#service-configuration-example)

## Related Documents

- [Integration Contracts](./integration-contracts.md) - Core contracts and interfaces
- [Component Architecture](./component-architecture.md) - Component design patterns
- [Async Patterns](./async-patterns.md) - Asynchronous integration patterns
- [System Integration](./system-integration.md) - System-level integration approaches
- [Tokio Runtime](./tokio-runtime.md) - Runtime configuration and lifecycle management

## 3. Error Handling Integration Patterns

The error handling patterns provide a unified approach to error management across all framework components, incorporating data flow validation and recovery strategies.

### 3.1 Unified Error Hierarchy

```rust
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Data persistence error: {0}")]
    DataPersistence(#[from] DataError),
    
    #[error("Supervision error: {0}")]
    Supervision(#[from] SupervisionError),
    
    #[error("Claude CLI integration error: {0}")]
    ClaudeCliIntegration(#[from] ClaudeCliError),
    
    #[error("Event system error: {0}")]
    EventSystem(#[from] EventError),
    
    #[error("Dependency injection error: {0}")]
    DependencyInjection(#[from] DIError),
    
    #[error("Integration test error: {0}")]
    IntegrationTest(#[from] TestError),
    
    #[error("Cross-component validation error: {0}")]
    CrossComponentValidation(String),
    
    #[error("Data flow integrity error: {0}")]
    DataFlowIntegrity(#[from] DataFlowError),
    
    #[error("Message validation error: {0}")]
    MessageValidation(#[from] MessageValidationError),
    
    #[error("Unknown system error: {0}")]
    Unknown(String),
}

// Data Flow Integrity Error Types (Agent 12 Validation)
#[derive(Debug, Error)]
pub enum DataFlowError {
    #[error("Message schema validation failed: {0}")]
    SchemaValidation(String),
    
    #[error("Data transformation integrity check failed: {0}")]
    TransformationIntegrity(String),
    
    #[error("Cross-component data consistency error: {0}")]
    ConsistencyViolation(String),
    
    #[error("Message replay attack detected: correlation_id={0}")]
    ReplayAttack(String),
    
    #[error("Data flow performance threshold exceeded: {metric}={value}, threshold={threshold}")]
    PerformanceViolation { metric: String, value: f64, threshold: f64 },
}

#[derive(Debug, Error)]
pub enum MessageValidationError {
    #[error("Missing required header: {0}")]
    MissingHeader(String),
    
    #[error("Invalid message payload: {0}")]
    InvalidPayload(String),
    
    #[error("Message correlation mismatch: expected={expected}, actual={actual}")]
    CorrelationMismatch { expected: String, actual: String },
    
    #[error("Message timeout: correlation_id={0}")]
    MessageTimeout(String),
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Optimistic lock conflict")]
    OptimisticLockConflict,
    
    #[error("Data integrity violation: {0}")]
    IntegrityViolation(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Cache operation failed: {0}")]
    CacheFailed(String),
}

impl ErrorRecovery for DataError {
    fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            DataError::ConnectionFailed(_) => RecoveryAction::Retry { 
                delay: Duration::from_secs(1) 
            },
            DataError::QueryFailed(_) => RecoveryAction::Retry { 
                delay: Duration::from_millis(500) 
            },
            DataError::TransactionFailed(_) => RecoveryAction::Retry { 
                delay: Duration::from_secs(2) 
            },
            DataError::MigrationFailed(_) => RecoveryAction::Escalate { 
                to_component: "database_admin".to_string() 
            },
            DataError::OptimisticLockConflict => RecoveryAction::Retry { 
                delay: Duration::from_millis(100) 
            },
            DataError::IntegrityViolation(_) => RecoveryAction::Escalate { 
                to_component: "data_integrity_monitor".to_string() 
            },
            _ => RecoveryAction::Ignore,
        }
    }
    
    fn is_retryable(&self) -> bool {
        matches!(self, 
            DataError::ConnectionFailed(_) | 
            DataError::QueryFailed(_) | 
            DataError::TransactionFailed(_) |
            DataError::OptimisticLockConflict |
            DataError::CacheFailed(_)
        )
    }
    
    fn max_retry_attempts(&self) -> u32 {
        match self {
            DataError::ConnectionFailed(_) => 5,
            DataError::OptimisticLockConflict => 3,
            _ => 3,
        }
    }
    
    fn backoff_strategy(&self) -> BackoffStrategy {
        match self {
            DataError::OptimisticLockConflict => BackoffStrategy::Fixed { 
                interval: Duration::from_millis(100) 
            },
            _ => BackoffStrategy::Exponential { 
                initial: Duration::from_millis(500), 
                factor: 2.0, 
                max: Duration::from_secs(30) 
            },
        }
    }
}

impl SystemError {
    pub fn error_code(&self) -> &'static str {
        match self {
            SystemError::Agent(_) => "SYS_AGENT",
            SystemError::Transport(_) => "SYS_TRANSPORT",
            SystemError::Security(_) => "SYS_SECURITY",
            SystemError::Configuration(_) => "SYS_CONFIG",
            SystemError::DataPersistence(_) => "SYS_DATA",
            SystemError::Supervision(_) => "SYS_SUPERVISION",
            SystemError::ClaudeCliIntegration(_) => "SYS_CLI",
            SystemError::EventSystem(_) => "SYS_EVENT",
            SystemError::DependencyInjection(_) => "SYS_DI",
            SystemError::IntegrationTest(_) => "SYS_TEST",
            SystemError::CrossComponentValidation(_) => "SYS_VALIDATION",
            SystemError::Unknown(_) => "SYS_UNKNOWN",
        }
    }
    
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SystemError::Security(_) => ErrorSeverity::Critical,
            SystemError::DataPersistence(_) => ErrorSeverity::High,
            SystemError::Supervision(_) => ErrorSeverity::High,
            SystemError::Agent(_) => ErrorSeverity::Medium,
            SystemError::Transport(_) => ErrorSeverity::Medium,
            SystemError::Configuration(_) => ErrorSeverity::Medium,
            SystemError::ClaudeCliIntegration(_) => ErrorSeverity::Low,
            SystemError::EventSystem(_) => ErrorSeverity::Low,
            SystemError::DependencyInjection(_) => ErrorSeverity::Medium,
            SystemError::IntegrationTest(_) => ErrorSeverity::Low,
            SystemError::CrossComponentValidation(_) => ErrorSeverity::Medium,
            SystemError::DataFlowIntegrity(_) => ErrorSeverity::High,
            SystemError::MessageValidation(_) => ErrorSeverity::Medium,
            SystemError::Unknown(_) => ErrorSeverity::High,
        }
    }
    
    pub fn component(&self) -> &'static str {
        match self {
            SystemError::Agent(_) => "agent",
            SystemError::Transport(_) => "transport",
            SystemError::Security(_) => "security",
            SystemError::Configuration(_) => "config",
            SystemError::DataPersistence(_) => "data",
            SystemError::Supervision(_) => "supervision",
            SystemError::ClaudeCliIntegration(_) => "claude-cli",
            SystemError::EventSystem(_) => "events",
            SystemError::DependencyInjection(_) => "di",
            SystemError::IntegrationTest(_) => "test",
            SystemError::CrossComponentValidation(_) => "validation",
            SystemError::Unknown(_) => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub trait ErrorRecovery {
    fn recovery_strategy(&self) -> RecoveryAction;
    fn is_retryable(&self) -> bool;
    fn max_retry_attempts(&self) -> u32;
    fn backoff_strategy(&self) -> BackoffStrategy;
    fn context(&self) -> ErrorContext;
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Retry { delay: Duration },
    Restart { component: String },
    Failover { backup_component: String },
    Escalate { to_component: String },
    Ignore,
    Shutdown { graceful: bool },
}

#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    Fixed { interval: Duration },
    Linear { initial: Duration, increment: Duration },
    Exponential { initial: Duration, factor: f64, max: Duration },
    Custom { calculator: fn(attempt: u32) -> Duration },
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub trace_id: String,
    pub span_id: String,
    pub component: String,
    pub operation: String,
    pub metadata: HashMap<String, String>,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

impl ErrorRecovery for SystemError {
    fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            SystemError::Agent(e) => e.recovery_strategy(),
            SystemError::Transport(e) => e.recovery_strategy(),
            SystemError::Security(_) => RecoveryAction::Escalate { 
                to_component: "security_manager".to_string() 
            },
            SystemError::Configuration(_) => RecoveryAction::Restart { 
                component: "config_manager".to_string() 
            },
            SystemError::DataPersistence(e) => e.recovery_strategy(),
            SystemError::Supervision(_) => RecoveryAction::Escalate { 
                to_component: "root_supervisor".to_string() 
            },
            SystemError::ClaudeCliIntegration(_) => RecoveryAction::Retry { 
                delay: Duration::from_secs(1) 
            },
            SystemError::EventSystem(_) => RecoveryAction::Restart { 
                component: "event_bus".to_string() 
            },
            SystemError::DependencyInjection(_) => RecoveryAction::Restart { 
                component: "service_registry".to_string() 
            },
            SystemError::IntegrationTest(_) => RecoveryAction::Ignore,
            SystemError::CrossComponentValidation(_) => RecoveryAction::Escalate { 
                to_component: "integration_validator".to_string() 
            },
            SystemError::DataFlowIntegrity(e) => e.recovery_strategy(),
            SystemError::MessageValidation(e) => e.recovery_strategy(),
            SystemError::Unknown(_) => RecoveryAction::Escalate { 
                to_component: "error_handler".to_string() 
            },
        }
    }
    
    fn is_retryable(&self) -> bool {
        match self {
            SystemError::Transport(_) => true,
            SystemError::ClaudeCliIntegration(_) => true,
            SystemError::EventSystem(_) => true,
            SystemError::Agent(e) => e.is_retryable(),
            SystemError::DataPersistence(e) => e.is_retryable(),
            _ => false,
        }
    }
    
    fn max_retry_attempts(&self) -> u32 {
        match self {
            SystemError::Transport(_) => 3,
            SystemError::ClaudeCliIntegration(_) => 5,
            SystemError::EventSystem(_) => 3,
            SystemError::Agent(e) => e.max_retry_attempts(),
            SystemError::DataPersistence(e) => e.max_retry_attempts(),
            _ => 0,
        }
    }
    
    fn backoff_strategy(&self) -> BackoffStrategy {
        match self {
            SystemError::Transport(_) => BackoffStrategy::Exponential { 
                initial: Duration::from_millis(100), 
                factor: 2.0, 
                max: Duration::from_secs(30) 
            },
            SystemError::ClaudeCliIntegration(_) => BackoffStrategy::Linear { 
                initial: Duration::from_millis(500), 
                increment: Duration::from_millis(500) 
            },
            SystemError::EventSystem(_) => BackoffStrategy::Fixed { 
                interval: Duration::from_secs(1) 
            },
            SystemError::Agent(e) => e.backoff_strategy(),
            SystemError::DataPersistence(e) => e.backoff_strategy(),
            _ => BackoffStrategy::Fixed { interval: Duration::from_secs(5) },
        }
    }
    
    fn context(&self) -> ErrorContext {
        ErrorContext {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            component: self.component().to_string(),
            operation: format!("{:?}", self),
            metadata: HashMap::new(),
            occurred_at: chrono::Utc::now(),
        }
    }
}

// Error propagation and mapping utilities with data flow validation
pub struct ErrorPropagator {
    mappings: HashMap<String, Box<dyn ErrorMapping>>,
    handlers: HashMap<String, Box<dyn ErrorHandler>>,
    flow_validator: DataFlowValidator,
    correlation_tracker: CorrelationTracker,
}

// Data Flow Validator (Agent 12 Integration)
pub struct DataFlowValidator {
    schema_validator: SchemaValidator,
    transformation_validator: TransformationValidator,
    consistency_checker: ConsistencyChecker,
    performance_monitor: PerformanceMonitor,
}

impl DataFlowValidator {
    pub async fn validate_data_flow<T>(&self, data: &T, context: &FlowContext) -> Result<ValidationResult, DataFlowError> 
    where 
        T: Serialize + Send + Sync
    {
        // Schema validation
        self.schema_validator.validate(data, &context.schema)?;
        
        // Transformation integrity
        if let Some(transformation) = &context.transformation {
            self.transformation_validator.validate_transformation(data, transformation)?;
        }
        
        // Cross-component consistency
        self.consistency_checker.check_consistency(data, &context.component_states).await?;
        
        // Performance validation
        let metrics = self.performance_monitor.current_metrics();
        self.validate_performance_thresholds(&metrics)?;
        
        Ok(ValidationResult {
            valid: true,
            warnings: vec![],
            performance_metrics: metrics,
        })
    }
    
    fn validate_performance_thresholds(&self, metrics: &PerformanceMetrics) -> Result<(), DataFlowError> {
        // Based on Agent 12 performance analysis
        if metrics.message_routing_latency > Duration::from_millis(1) {
            return Err(DataFlowError::PerformanceViolation {
                metric: "message_routing_latency".to_string(),
                value: metrics.message_routing_latency.as_millis() as f64,
                threshold: 1.0,
            });
        }
        
        if metrics.state_persistence_latency > Duration::from_millis(5) {
            return Err(DataFlowError::PerformanceViolation {
                metric: "state_persistence_latency".to_string(),
                value: metrics.state_persistence_latency.as_millis() as f64,
                threshold: 5.0,
            });
        }
        
        Ok(())
    }
}

pub trait ErrorMapping: Send + Sync {
    fn map(&self, error: Box<dyn std::error::Error>) -> SystemError;
}

pub trait ErrorHandler: Send + Sync {
    async fn handle(&self, error: &SystemError) -> Result<RecoveryResult, SystemError>;
}

#[derive(Debug)]
pub enum RecoveryResult {
    Recovered,
    PartialRecovery { remaining_error: SystemError },
    Failed { escalated_error: SystemError },
}

impl ErrorPropagator {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            handlers: HashMap::new(),
            flow_validator: DataFlowValidator::new(),
            correlation_tracker: CorrelationTracker::new(),
        }
    }
    
    pub async fn validate_and_propagate_error(
        &self, 
        error: Box<dyn std::error::Error>, 
        flow_context: &FlowContext
    ) -> Result<(), SystemError> {
        // Validate data flow integrity before propagation
        if let Some(data) = flow_context.associated_data.as_ref() {
            self.flow_validator.validate_data_flow(data, flow_context).await?;
        }
        
        // Track error correlation
        self.correlation_tracker.track_error(
            &flow_context.correlation_id, 
            &error
        ).await;
        
        // Map and handle error
        let system_error = self.map_error(&flow_context.component, error);
        self.handle_error(&system_error).await
    }
    
    pub fn add_mapping(&mut self, component: &str, mapping: Box<dyn ErrorMapping>) {
        self.mappings.insert(component.to_string(), mapping);
    }
    
    pub fn add_handler(&mut self, component: &str, handler: Box<dyn ErrorHandler>) {
        self.handlers.insert(component.to_string(), handler);
    }
    
    pub fn map_error(&self, component: &str, error: Box<dyn std::error::Error>) -> SystemError {
        if let Some(mapping) = self.mappings.get(component) {
            mapping.map(error)
        } else {
            SystemError::Unknown(format!("Unmapped error from component {}: {}", component, error))
        }
    }
    
    pub async fn handle_error(&self, error: &SystemError) -> Result<RecoveryResult, SystemError> {
        let component = error.component();
        if let Some(handler) = self.handlers.get(component) {
            handler.handle(error).await
        } else {
            Err(SystemError::Unknown(format!("No handler for component: {}", component)))
        }
    }
}

// Component-specific error types with SystemError integration
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Agent processing error: {0}")]
    ProcessingError(String),
    
    #[error("Agent communication timeout")]
    Timeout,
    
    #[error("Agent state corruption: {0}")]
    StateCorruption(String),
    
    #[error("Agent capability mismatch: required {required}, available {available}")]
    CapabilityMismatch { required: String, available: String },
}

impl ErrorRecovery for AgentError {
    fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            AgentError::InitializationFailed(_) => RecoveryAction::Restart { 
                component: "agent".to_string() 
            },
            AgentError::ProcessingError(_) => RecoveryAction::Retry { 
                delay: Duration::from_millis(500) 
            },
            AgentError::Timeout => RecoveryAction::Retry { 
                delay: Duration::from_secs(1) 
            },
            AgentError::StateCorruption(_) => RecoveryAction::Restart { 
                component: "agent".to_string() 
            },
            AgentError::CapabilityMismatch { .. } => RecoveryAction::Failover { 
                backup_component: "default_agent".to_string() 
            },
        }
    }
    
    fn is_retryable(&self) -> bool {
        matches!(self, AgentError::ProcessingError(_) | AgentError::Timeout)
    }
    
    fn max_retry_attempts(&self) -> u32 {
        match self {
            AgentError::ProcessingError(_) => 3,
            AgentError::Timeout => 5,
            _ => 0,
        }
    }
    
    fn backoff_strategy(&self) -> BackoffStrategy {
        BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            factor: 1.5,
            max: Duration::from_secs(10),
        }
    }
    
    fn context(&self) -> ErrorContext {
        ErrorContext {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            component: "agent".to_string(),
            operation: format!("{:?}", self),
            metadata: HashMap::new(),
            occurred_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Message serialization error: {0}")]
    SerializationError(String),
    
    #[error("Destination not found: {0}")]
    DestinationNotFound(String),
    
    #[error("Transport timeout")]
    Timeout,
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Message too large: {size} bytes, max {max} bytes")]
    MessageTooLarge { size: usize, max: usize },
}

impl ErrorRecovery for DataFlowError {
    fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            DataFlowError::SchemaValidation(_) => RecoveryAction::Escalate {
                to_component: "schema_registry".to_string()
            },
            DataFlowError::TransformationIntegrity(_) => RecoveryAction::Retry {
                delay: Duration::from_millis(500)
            },
            DataFlowError::ConsistencyViolation(_) => RecoveryAction::Restart {
                component: "data_flow_manager".to_string()
            },
            DataFlowError::ReplayAttack(_) => RecoveryAction::Escalate {
                to_component: "security_manager".to_string()
            },
            DataFlowError::PerformanceViolation { .. } => RecoveryAction::Escalate {
                to_component: "performance_monitor".to_string()
            },
        }
    }
    
    fn is_retryable(&self) -> bool {
        matches!(self, DataFlowError::TransformationIntegrity(_))
    }
    
    fn max_retry_attempts(&self) -> u32 {
        match self {
            DataFlowError::TransformationIntegrity(_) => 3,
            _ => 0,
        }
    }
    
    fn backoff_strategy(&self) -> BackoffStrategy {
        BackoffStrategy::Fixed {
            interval: Duration::from_millis(500),
        }
    }
    
    fn context(&self) -> ErrorContext {
        ErrorContext {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            component: "data_flow".to_string(),
            operation: format!("{:?}", self),
            metadata: HashMap::new(),
            occurred_at: chrono::Utc::now(),
        }
    }
}

impl ErrorRecovery for TransportError {
    fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            TransportError::ConnectionFailed(_) => RecoveryAction::Retry { 
                delay: Duration::from_secs(1) 
            },
            TransportError::SerializationError(_) => RecoveryAction::Ignore,
            TransportError::DestinationNotFound(_) => RecoveryAction::Ignore,
            TransportError::Timeout => RecoveryAction::Retry { 
                delay: Duration::from_millis(500) 
            },
            TransportError::ProtocolError(_) => RecoveryAction::Restart { 
                component: "transport".to_string() 
            },
            TransportError::AuthenticationFailed => RecoveryAction::Escalate { 
                to_component: "security_manager".to_string() 
            },
            TransportError::MessageTooLarge { .. } => RecoveryAction::Ignore,
        }
    }
    
    fn is_retryable(&self) -> bool {
        matches!(self, TransportError::ConnectionFailed(_) | TransportError::Timeout)
    }
    
    fn max_retry_attempts(&self) -> u32 {
        match self {
            TransportError::ConnectionFailed(_) => 3,
            TransportError::Timeout => 5,
            _ => 0,
        }
    }
    
    fn backoff_strategy(&self) -> BackoffStrategy {
        BackoffStrategy::Exponential {
            initial: Duration::from_millis(250),
            factor: 2.0,
            max: Duration::from_secs(30),
        }
    }
    
    fn context(&self) -> ErrorContext {
        ErrorContext {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            component: "transport".to_string(),
            operation: format!("{:?}", self),
            metadata: HashMap::new(),
            occurred_at: chrono::Utc::now(),
        }
    }
}
```

### Database Error Recovery Example

Here's a practical example of using the error handling patterns for database operations:

```rust
use std::time::Duration;
use async_trait::async_trait;

// Practical database service implementation
pub struct DatabaseService {
    pool: ConnectionPool,
    error_propagator: ErrorPropagator,
    retry_policy: RetryPolicy,
}

impl DatabaseService {
    pub async fn execute_with_recovery<T, F>(&self, operation: F) -> Result<T, SystemError>
    where
        F: Fn() -> BoxFuture<'static, Result<T, DataError>> + Send + Sync,
        T: Send + 'static,
    {
        let mut attempts = 0;
        let mut last_error = None;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Use the error recovery trait to determine strategy
                    let recovery = e.recovery_strategy();
                    
                    match recovery {
                        RecoveryAction::Retry { delay } => {
                            if attempts >= e.max_retry_attempts() {
                                return Err(SystemError::DataPersistence(e));
                            }
                            
                            attempts += 1;
                            
                            // Apply backoff strategy
                            let backoff_delay = match e.backoff_strategy() {
                                BackoffStrategy::Exponential { initial, factor, max } => {
                                    let delay = initial * factor.powf(attempts as f64);
                                    std::cmp::min(delay, max)
                                }
                                BackoffStrategy::Fixed { interval } => interval,
                                _ => delay,
                            };
                            
                            tokio::time::sleep(backoff_delay).await;
                            last_error = Some(e);
                            continue;
                        }
                        RecoveryAction::Failover { backup_component } => {
                            // Switch to backup database
                            self.switch_to_backup(&backup_component).await?;
                            // Retry operation once with backup
                            return operation().await
                                .map_err(|e| SystemError::DataPersistence(e));
                        }
                        RecoveryAction::Escalate { to_component } => {
                            // Log and escalate to monitoring system
                            self.escalate_error(&e, &to_component).await;
                            return Err(SystemError::DataPersistence(e));
                        }
                        _ => return Err(SystemError::DataPersistence(e)),
                    }
                }
            }
        }
    }
    
    async fn switch_to_backup(&self, backup_name: &str) -> Result<(), SystemError> {
        // Implementation for switching to backup database
        self.pool.switch_to_backup(backup_name).await
            .map_err(|e| SystemError::DataPersistence(DataError::ConnectionFailed(e.to_string())))
    }
    
    async fn escalate_error(&self, error: &DataError, component: &str) {
        // Send error details to monitoring component
        let context = error.context();
        tracing::error!(
            target: component,
            trace_id = %context.trace_id,
            span_id = %context.span_id,
            component = %context.component,
            operation = %context.operation,
            "Database error escalated: {:?}", error
        );
    }
}

// Example usage
pub async fn example_database_operation(db: &DatabaseService) -> Result<User, SystemError> {
    db.execute_with_recovery(|| {
        Box::pin(async {
            // Simulated database query
            let query = "SELECT * FROM users WHERE id = $1";
            let result = db.pool.query_one(query, &[&user_id]).await
                .map_err(|e| DataError::QueryFailed(e.to_string()))?;
            
            User::from_row(&result)
                .map_err(|e| DataError::SerializationFailed(e.to_string()))
        })
    }).await
}
```

## 4. Event System Integration Patterns

The event system provides asynchronous, decoupled communication between components with built-in data flow validation and correlation tracking.

### 4.1 Event-Driven Architecture Integration

```rust
use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

// Core framework types
pub type AgentId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    System,
    User,
    Background,
    Specialized(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
    SimpleOneForOne,
}

// Event error types with data flow validation
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Failed to publish event")]
    PublishFailed,
    #[error("Operation timed out")]
    Timeout,
    #[error("Subscription failed")]
    SubscriptionFailed,
    #[error("Event validation failed: {0}")]
    ValidationFailed(String),
    #[error("Event correlation tracking failed: {0}")]
    CorrelationFailed(String),
}

#[async_trait]
pub trait EventBus: Send + Sync + Clone {
    async fn publish<E>(&self, event: E) -> Result<(), EventError> 
    where 
        E: Event + Send + Sync + 'static;
    
    async fn subscribe<E>(&self) -> Result<EventSubscription<E>, EventError> 
    where 
        E: Event + Send + Sync + 'static;
    
    async fn request<E, R>(&self, event: E, timeout: Duration) -> Result<R, EventError> 
    where 
        E: Event + Send + Sync + 'static,
        R: Event + Send + Sync + 'static;
    
    async fn emit_and_wait<E>(&self, event: E) -> Result<Vec<EventResponse>, EventError>
    where 
        E: Event + Send + Sync + 'static;
    
    fn metrics(&self) -> EventBusMetrics;
}

pub trait Event: Send + Sync + Clone + 'static {
    fn event_type(&self) -> &'static str;
    fn event_id(&self) -> Uuid;
    fn correlation_id(&self) -> Option<Uuid>;
    fn metadata(&self) -> &EventMetadata;
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
    fn source_component(&self) -> &str;
    fn target_component(&self) -> Option<&str>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub trace_id: String,
    pub span_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub custom_fields: HashMap<String, String>,
    // Data flow integrity fields (Agent 12)
    pub message_checksum: Option<String>,
    pub schema_version: String,
    pub transformation_audit: Vec<TransformationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRecord {
    pub component: String,
    pub operation: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub checksum_before: String,
    pub checksum_after: String,
}

pub struct EventSubscription<E> {
    receiver: mpsc::Receiver<E>,
    subscription_id: Uuid,
    event_type: &'static str,
}

impl<E: Event> EventSubscription<E> {
    pub async fn next(&mut self) -> Option<E> {
        self.receiver.recv().await
    }
    
    pub fn subscription_id(&self) -> Uuid {
        self.subscription_id
    }
    
    pub fn event_type(&self) -> &'static str {
        self.event_type
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub event_id: Uuid,
    pub responder: String,
    pub response_type: String,
    pub payload: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
}

// Core framework events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    Agent(AgentEvent),
    Transport(TransportEvent),
    Security(SecurityEvent),
    Configuration(ConfigurationEvent),
    Data(DataEvent),
    Supervision(SupervisionEvent),
    ClaudeCli(ClaudeCliEvent),
}

impl Event for SystemEvent {
    fn event_type(&self) -> &'static str {
        match self {
            SystemEvent::Agent(_) => "system.agent",
            SystemEvent::Transport(_) => "system.transport",
            SystemEvent::Security(_) => "system.security",
            SystemEvent::Configuration(_) => "system.configuration",
            SystemEvent::Data(_) => "system.data",
            SystemEvent::Supervision(_) => "system.supervision",
            SystemEvent::ClaudeCli(_) => "system.claude_cli",
        }
    }
    
    fn event_id(&self) -> Uuid {
        match self {
            SystemEvent::Agent(e) => e.event_id(),
            SystemEvent::Transport(e) => e.event_id(),
            SystemEvent::Security(e) => e.event_id(),
            SystemEvent::Configuration(e) => e.event_id(),
            SystemEvent::Data(e) => e.event_id(),
            SystemEvent::Supervision(e) => e.event_id(),
            SystemEvent::ClaudeCli(e) => e.event_id(),
        }
    }
    
    fn correlation_id(&self) -> Option<Uuid> {
        match self {
            SystemEvent::Agent(e) => e.correlation_id(),
            SystemEvent::Transport(e) => e.correlation_id(),
            SystemEvent::Security(e) => e.correlation_id(),
            SystemEvent::Configuration(e) => e.correlation_id(),
            SystemEvent::Data(e) => e.correlation_id(),
            SystemEvent::Supervision(e) => e.correlation_id(),
            SystemEvent::ClaudeCli(e) => e.correlation_id(),
        }
    }
    
    fn metadata(&self) -> &EventMetadata {
        match self {
            SystemEvent::Agent(e) => e.metadata(),
            SystemEvent::Transport(e) => e.metadata(),
            SystemEvent::Security(e) => e.metadata(),
            SystemEvent::Configuration(e) => e.metadata(),
            SystemEvent::Data(e) => e.metadata(),
            SystemEvent::Supervision(e) => e.metadata(),
            SystemEvent::ClaudeCli(e) => e.metadata(),
        }
    }
    
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            SystemEvent::Agent(e) => e.timestamp(),
            SystemEvent::Transport(e) => e.timestamp(),
            SystemEvent::Security(e) => e.timestamp(),
            SystemEvent::Configuration(e) => e.timestamp(),
            SystemEvent::Data(e) => e.timestamp(),
            SystemEvent::Supervision(e) => e.timestamp(),
            SystemEvent::ClaudeCli(e) => e.timestamp(),
        }
    }
    
    fn source_component(&self) -> &str {
        match self {
            SystemEvent::Agent(e) => e.source_component(),
            SystemEvent::Transport(e) => e.source_component(),
            SystemEvent::Security(e) => e.source_component(),
            SystemEvent::Configuration(e) => e.source_component(),
            SystemEvent::Data(e) => e.source_component(),
            SystemEvent::Supervision(e) => e.source_component(),
            SystemEvent::ClaudeCli(e) => e.source_component(),
        }
    }
    
    fn target_component(&self) -> Option<&str> {
        match self {
            SystemEvent::Agent(e) => e.target_component(),
            SystemEvent::Transport(e) => e.target_component(),
            SystemEvent::Security(e) => e.target_component(),
            SystemEvent::Configuration(e) => e.target_component(),
            SystemEvent::Data(e) => e.target_component(),
            SystemEvent::Supervision(e) => e.target_component(),
            SystemEvent::ClaudeCli(e) => e.target_component(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Started { agent_id: AgentId, agent_type: AgentType },
    Stopped { agent_id: AgentId, reason: String },
    ProcessingStarted { agent_id: AgentId, task_id: Uuid },
    ProcessingCompleted { agent_id: AgentId, task_id: Uuid, duration: Duration },
    ProcessingFailed { agent_id: AgentId, task_id: Uuid, error: String },
    HealthChanged { agent_id: AgentId, status: HealthStatus },
    ConfigurationUpdated { agent_id: AgentId, config_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportEvent {
    ConnectionEstablished { transport_id: String, endpoint: String },
    ConnectionLost { transport_id: String, endpoint: String, reason: String },
    MessageSent { transport_id: String, destination: String, message_id: Uuid },
    MessageReceived { transport_id: String, source: String, message_id: Uuid },
    MessageFailed { transport_id: String, message_id: Uuid, error: String },
    QueueCreated { transport_id: String, queue_name: String },
    QueueDeleted { transport_id: String, queue_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    AuthenticationAttempt { user_id: String, success: bool, method: String },
    AuthorizationCheck { user_id: String, resource: String, action: String, allowed: bool },
    SecurityViolation { user_id: String, violation_type: String, details: String },
    TokenIssued { user_id: String, token_type: String, expires_at: chrono::DateTime<chrono::Utc> },
    TokenRevoked { user_id: String, token_id: String, reason: String },
    PermissionChanged { user_id: String, permission: String, granted: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationEvent {
    ConfigurationLoaded { source: String, keys_count: usize },
    ConfigurationChanged { key: String, old_value: Option<String>, new_value: String },
    ConfigurationReloaded { source: String, changed_keys: Vec<String> },
    ConfigurationError { source: String, error: String },
    SecretUpdated { key: String, source: String },
    WatcherRegistered { key: String, watcher_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataEvent {
    ConnectionEstablished { database: String, connection_id: String },
    ConnectionLost { database: String, connection_id: String, error: String },
    QueryExecuted { database: String, query_type: String, duration: Duration, rows_affected: Option<u64> },
    QueryFailed { database: String, query: String, error: String },
    TransactionStarted { database: String, transaction_id: String },
    TransactionCommitted { database: String, transaction_id: String, duration: Duration },
    TransactionRolledBack { database: String, transaction_id: String, reason: String },
    SchemaChanged { database: String, change_type: String, table: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisionEvent {
    SupervisorStarted { supervisor_id: String, strategy: SupervisionStrategy },
    SupervisorStopped { supervisor_id: String, reason: String },
    ChildAdded { supervisor_id: String, child_id: String, child_type: String },
    ChildRemoved { supervisor_id: String, child_id: String, reason: String },
    ChildFailed { supervisor_id: String, child_id: String, error: String, restart_count: u32 },
    ChildRestarted { supervisor_id: String, child_id: String, restart_reason: String },
    EscalationTriggered { supervisor_id: String, child_id: String, escalation_level: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaudeCliEvent {
    ProcessSpawned { process_id: String, command: String, args: Vec<String> },
    ProcessCompleted { process_id: String, exit_code: i32, duration: Duration },
    ProcessFailed { process_id: String, error: String },
    MessageReceived { process_id: String, message_type: String, content: String },
    MessageSent { process_id: String, message_type: String, content: String },
    HookRegistered { hook_name: String, process_id: String },
    HookTriggered { hook_name: String, process_id: String, payload: serde_json::Value },
}

// Event Bus Implementation with Data Flow Validation
pub struct DefaultEventBus {
    publishers: Arc<RwLock<HashMap<String, broadcast::Sender<Box<dyn Event>>>>>,
    subscribers: Arc<RwLock<HashMap<Uuid, EventSubscriptionInfo>>>,
    metrics: Arc<RwLock<EventBusMetrics>>,
    correlator: EventCorrelator,
    // Data flow integrity components (Agent 12)
    message_validator: MessageValidator,
    transformation_tracker: TransformationTracker,
    replay_detector: ReplayDetector,
    flow_monitor: DataFlowMonitor,
}

// Message Validator (Agent 12 Integration)
pub struct MessageValidator {
    schema_registry: Arc<RwLock<HashMap<String, MessageSchema>>>,
    validation_cache: Arc<RwLock<LruCache<String, ValidationResult>>>,
}

impl MessageValidator {
    pub async fn validate_message<E: Event>(&self, event: &E) -> Result<ValidationResult, EventError> {
        let event_type = event.event_type();
        let schemas = self.schema_registry.read().await;
        
        let schema = schemas.get(event_type)
            .ok_or_else(|| EventError::ValidationFailed(format!("No schema for event type: {}", event_type)))?;
        
        // Validate required fields
        let metadata = event.metadata();
        if metadata.schema_version != schema.version {
            return Err(EventError::ValidationFailed(
                format!("Schema version mismatch: expected {}, got {}", schema.version, metadata.schema_version)
            ));
        }
        
        // Validate data integrity
        if let Some(expected_checksum) = &metadata.message_checksum {
            let actual_checksum = self.calculate_checksum(event)?;
            if actual_checksum != *expected_checksum {
                return Err(EventError::ValidationFailed("Checksum mismatch".to_string()));
            }
        }
        
        Ok(ValidationResult {
            valid: true,
            schema_version: schema.version.clone(),
            validation_timestamp: chrono::Utc::now(),
        })
    }
    
    fn calculate_checksum<E: Event>(&self, event: &E) -> Result<String, EventError> {
        // Calculate SHA256 checksum of serialized event
        let serialized = serde_json::to_vec(event)
            .map_err(|e| EventError::ValidationFailed(format!("Serialization failed: {}", e)))?;
        
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

// Replay Attack Detector (Agent 12 Security Enhancement)
pub struct ReplayDetector {
    seen_messages: Arc<RwLock<LruCache<String, chrono::DateTime<chrono::Utc>>>>,
    time_window: Duration,
}

impl ReplayDetector {
    pub async fn check_replay(&self, event_id: Uuid, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), EventError> {
        let mut seen = self.seen_messages.write().await;
        
        let event_id_str = event_id.to_string();
        if let Some(previous_timestamp) = seen.get(&event_id_str) {
            return Err(EventError::ValidationFailed(
                format!("Replay attack detected: event {} already processed at {}", event_id, previous_timestamp)
            ));
        }
        
        // Check timestamp is within acceptable window
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(timestamp);
        
        if age > chrono::Duration::from_std(self.time_window).unwrap() {
            return Err(EventError::ValidationFailed(
                format!("Event too old: {} seconds", age.num_seconds())
            ));
        }
        
        seen.put(event_id_str, timestamp);
        Ok(())
    }
}

#[derive(Debug)]
struct EventSubscriptionInfo {
    event_type: String,
    subscriber_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DefaultEventBus {
    pub fn new() -> Self {
        Self {
            publishers: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(EventBusMetrics::default())),
            correlator: EventCorrelator::new(),
        }
    }
    
    async fn get_or_create_publisher(&self, event_type: &str) -> broadcast::Sender<Box<dyn Event>> {
        let mut publishers = self.publishers.write().await;
        publishers.entry(event_type.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                tx
            })
            .clone()
    }
}

#[async_trait]
impl EventBus for DefaultEventBus {
    async fn publish<E>(&self, event: E) -> Result<(), EventError> 
    where 
        E: Event + Send + Sync + 'static
    {
        let event_type = event.event_type();
        
        // Data flow validation (Agent 12)
        self.message_validator.validate_message(&event).await?;
        
        // Replay attack detection
        self.replay_detector.check_replay(
            event.event_id(), 
            event.timestamp()
        ).await?;
        
        // Track transformation
        self.transformation_tracker.record_transformation(
            &event,
            "event_bus_publish",
            event.source_component()
        ).await;
        
        let publisher = self.get_or_create_publisher(event_type).await;
        
        // Update metrics with flow monitoring
        {
            let mut metrics = self.metrics.write().await;
            metrics.events_published += 1;
            *metrics.events_by_type.entry(event_type.to_string()).or_insert(0) += 1;
            
            // Data flow monitoring (Agent 12)
            self.flow_monitor.record_event_flow(
                event.event_id(),
                event_type,
                event.source_component(),
                event.target_component()
            ).await;
        }
        
        // Correlate event
        self.correlator.add_event(&event).await;
        
        // Publish event
        let boxed_event: Box<dyn Event> = Box::new(event);
        publisher.send(boxed_event)
            .map_err(|_| EventError::PublishFailed)?;
        
        Ok(())
    }
    
    async fn subscribe<E>(&self) -> Result<EventSubscription<E>, EventError> 
    where 
        E: Event + Send + Sync + 'static
    {
        let event_type = std::any::type_name::<E>();
        let subscription_id = Uuid::new_v4();
        
        let publisher = self.get_or_create_publisher(event_type).await;
        let receiver = publisher.subscribe();
        
        // Create filtered receiver
        let (tx, rx) = mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Ok(event) = receiver.recv().await {
                if let Ok(typed_event) = event.downcast_ref::<E>() {
                    if tx.send(typed_event.clone()).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        // Record subscription
        {
            let mut subscribers = self.subscribers.write().await;
            subscribers.insert(subscription_id, EventSubscriptionInfo {
                event_type: event_type.to_string(),
                subscriber_id: "event_subscriber".to_string(),
                created_at: chrono::Utc::now(),
            });
        }
        
        Ok(EventSubscription {
            receiver: rx,
            subscription_id,
            event_type,
        })
    }
    
    async fn request<E, R>(&self, event: E, timeout: Duration) -> Result<R, EventError> 
    where 
        E: Event + Send + Sync + 'static,
        R: Event + Send + Sync + 'static
    {
        let correlation_id = Uuid::new_v4();
        let response_type = std::any::type_name::<R>();
        
        // Subscribe to response events
        let mut response_subscription = self.subscribe::<R>().await?;
        
        // Publish request event with correlation ID
        let mut request_event = event;
        // Set correlation ID on request event (implementation needed)
        self.publish(request_event).await?;
        
        // Wait for correlated response
        let timeout_future = tokio::time::sleep(timeout);
        tokio::pin!(timeout_future);
        
        loop {
            tokio::select! {
                response = response_subscription.next() => {
                    if let Some(response) = response {
                        if response.correlation_id() == Some(correlation_id) {
                            return Ok(response);
                        }
                    }
                }
                _ = &mut timeout_future => {
                    return Err(EventError::Timeout);
                }
            }
        }
    }
    
    async fn emit_and_wait<E>(&self, event: E) -> Result<Vec<EventResponse>, EventError>
    where 
        E: Event + Send + Sync + 'static
    {
        let event_id = event.event_id();
        let response_timeout = Duration::from_secs(30);
        
        // Subscribe to response events and collect until timeout
        // Implementation needed for production use
        self.publish(event).await?;
        Ok(vec![])
    }
    
    fn metrics(&self) -> EventBusMetrics {
        // Return current metrics (implementation needed)
        EventBusMetrics::default()
    }
}

#[derive(Debug, Default)]
pub struct EventBusMetrics {
    pub events_published: u64,
    pub events_consumed: u64,
    pub active_subscriptions: u64,
    pub events_by_type: HashMap<String, u64>,
    pub average_latency: Duration,
    pub error_rate: f64,
}

// Event correlation utilities
pub struct EventCorrelator {
    correlations: Arc<RwLock<HashMap<Uuid, Vec<EventInfo>>>>,
    cleanup_interval: Duration,
}

#[derive(Debug, Clone)]
struct EventInfo {
    event_id: Uuid,
    event_type: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    source_component: String,
}

impl EventCorrelator {
    pub fn new() -> Self {
        let correlator = Self {
            correlations: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        };
        
        // Start cleanup task
        let correlations = correlator.correlations.clone();
        let interval = correlator.cleanup_interval;
        tokio::spawn(async move {
            let mut cleanup_timer = tokio::time::interval(interval);
            loop {
                cleanup_timer.tick().await;
                let cutoff = chrono::Utc::now() - chrono::Duration::seconds(600); // 10 minutes
                
                let mut correlations = correlations.write().await;
                correlations.retain(|_, events| {
                    events.iter().any(|event| event.timestamp > cutoff)
                });
            }
        });
        
        correlator
    }
    
    pub async fn add_event<E: Event>(&self, event: &E) {
        if let Some(correlation_id) = event.correlation_id() {
            let event_info = EventInfo {
                event_id: event.event_id(),
                event_type: event.event_type().to_string(),
                timestamp: event.timestamp(),
                source_component: event.source_component().to_string(),
            };
            
            let mut correlations = self.correlations.write().await;
            correlations.entry(correlation_id)
                .or_insert_with(Vec::new)
                .push(event_info);
        }
    }
    
    pub async fn get_correlated_events(&self, correlation_id: Uuid) -> Vec<EventInfo> {
        let correlations = self.correlations.read().await;
        correlations.get(&correlation_id).cloned().unwrap_or_default()
    }
}
```

### Agent Communication Example

This example demonstrates how agents use the event system for communication:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

// Agent that processes data and publishes results
pub struct DataProcessingAgent {
    id: AgentId,
    event_bus: Arc<dyn EventBus>,
    processor: DataProcessor,
}

#[async_trait]
impl Agent for DataProcessingAgent {
    type Input = ProcessingRequest;
    type Output = ProcessingResult;
    type Error = AgentError;
    type Config = ProcessingConfig;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Process the data
        let result = self.processor.process(&input.data).await?;
        
        // Create processing completed event
        let event = SystemEvent::Agent(AgentEvent::ProcessingCompleted {
            agent_id: self.id,
            task_id: input.task_id,
            duration: result.processing_time,
        });
        
        // Publish event to notify other agents
        self.event_bus.publish(event).await
            .map_err(|e| AgentError::ProcessingError(format!("Failed to publish event: {}", e)))?;
        
        // If this is part of a workflow, publish workflow event
        if let Some(workflow_id) = input.workflow_id {
            let workflow_event = WorkflowEvent {
                workflow_id,
                step: "data_processing".to_string(),
                status: WorkflowStatus::StepCompleted,
                metadata: result.metadata.clone(),
            };
            
            self.event_bus.publish(workflow_event).await
                .map_err(|e| AgentError::ProcessingError(format!("Failed to publish workflow event: {}", e)))?;
        }
        
        Ok(result)
    }
}

// Coordinator agent that orchestrates workflow
pub struct WorkflowCoordinatorAgent {
    id: AgentId,
    event_bus: Arc<dyn EventBus>,
    workflow_state: Arc<RwLock<HashMap<Uuid, WorkflowState>>>,
}

impl WorkflowCoordinatorAgent {
    pub async fn start(&self) -> Result<(), AgentError> {
        // Subscribe to agent events
        let mut agent_events = self.event_bus.subscribe::<SystemEvent>().await
            .map_err(|e| AgentError::InitializationFailed(format!("Failed to subscribe: {}", e)))?;
        
        // Subscribe to workflow events
        let mut workflow_events = self.event_bus.subscribe::<WorkflowEvent>().await
            .map_err(|e| AgentError::InitializationFailed(format!("Failed to subscribe: {}", e)))?;
        
        // Handle events concurrently
        tokio::select! {
            _ = self.handle_agent_events(agent_events) => {},
            _ = self.handle_workflow_events(workflow_events) => {},
        }
        
        Ok(())
    }
    
    async fn handle_agent_events(&self, mut events: EventSubscription<SystemEvent>) {
        while let Some(event) = events.next().await {
            match event {
                SystemEvent::Agent(AgentEvent::ProcessingCompleted { agent_id, task_id, .. }) => {
                    // Update workflow state
                    let mut state = self.workflow_state.write().await;
                    if let Some(workflow) = state.values_mut()
                        .find(|w| w.contains_task(&task_id)) {
                        workflow.mark_task_completed(task_id);
                        
                        // Check if workflow is complete
                        if workflow.is_complete() {
                            self.complete_workflow(workflow.id).await;
                        } else {
                            // Schedule next task
                            self.schedule_next_task(workflow).await;
                        }
                    }
                }
                SystemEvent::Agent(AgentEvent::ProcessingFailed { agent_id, task_id, error }) => {
                    // Handle failure with retry or escalation
                    self.handle_task_failure(agent_id, task_id, error).await;
                }
                _ => {}
            }
        }
    }
    
    async fn schedule_next_task(&self, workflow: &WorkflowState) -> Result<(), AgentError> {
        if let Some(next_task) = workflow.get_next_task() {
            // Create request for next agent
            let request = ProcessingRequest {
                task_id: next_task.id,
                workflow_id: Some(workflow.id),
                data: next_task.data.clone(),
                priority: workflow.priority,
            };
            
            // Use request-response pattern to assign task
            let response = self.event_bus.request::<TaskAssignmentRequest, TaskAssignmentResponse>(
                TaskAssignmentRequest {
                    task: next_task.clone(),
                    preferred_agent_type: next_task.agent_type.clone(),
                },
                Duration::from_secs(5)
            ).await?;
            
            tracing::info!(
                "Task {} assigned to agent {}",
                next_task.id,
                response.assigned_agent_id
            );
        }
        
        Ok(())
    }
}

// Example workflow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub workflow_id: Uuid,
    pub step: String,
    pub status: WorkflowStatus,
    pub metadata: HashMap<String, String>,
}

impl Event for WorkflowEvent {
    fn event_type(&self) -> &'static str {
        "workflow.status"
    }
    
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn correlation_id(&self) -> Option<Uuid> {
        Some(self.workflow_id)
    }
    
    fn metadata(&self) -> &EventMetadata {
        // Implementation details...
    }
    
    // Other trait methods...
}
```

## 5. Dependency Injection Integration

The dependency injection patterns provide a flexible, type-safe approach to service composition and lifecycle management.

### 5.1 Service Registry and Dependency Resolution

```rust
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use futures::future::BoxFuture;
use tokio::sync::RwLock;

// Additional error types for DI
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    General(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Security error: {0}")]
    General(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Data error: {0}")]
    General(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SupervisionError {
    #[error("Supervision error: {0}")]
    General(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ClaudeCliError {
    #[error("Claude CLI error: {0}")]
    General(String),
}

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Test error: {0}")]
    General(String),
}

#[async_trait]
pub trait Injectable: Send + Sync + 'static {
    type Config: Send + Sync + for<'de> serde::Deserialize<'de>;
    type Error: Into<SystemError> + Send + Sync;
    
    async fn create(config: Self::Config, registry: &ServiceRegistry) -> Result<Self, Self::Error> 
    where 
        Self: Sized;
    
    fn dependencies() -> Vec<Dependency>;
    fn service_info() -> ServiceInfo;
    
    async fn initialize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn health_check(&self) -> ServiceHealth {
        ServiceHealth::Healthy
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub service_type: TypeId,
    pub service_name: String,
    pub required: bool,
    pub scope: DependencyScope,
}

#[derive(Debug, Clone)]
pub enum DependencyScope {
    Singleton,
    PerRequest,
    Scoped { scope_name: String },
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub scope: ServiceScope,
}

#[derive(Debug, Clone)]
pub enum ServiceScope {
    Singleton,
    Transient,
    Scoped { scope_name: String },
}

#[derive(Debug, Clone)]
pub enum ServiceHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

#[async_trait]
pub trait ServiceRegistry: Send + Sync {
    async fn register<T>(&self, service: T) -> Result<(), DIError> 
    where 
        T: Injectable;
    
    async fn register_factory<T, F>(&self, factory: F) -> Result<(), DIError>
    where 
        T: Injectable,
        F: ServiceFactory<T> + Send + Sync + 'static;
    
    async fn resolve<T>(&self) -> Result<Arc<T>, DIError> 
    where 
        T: Injectable;
    
    async fn resolve_named<T>(&self, name: &str) -> Result<Arc<T>, DIError> 
    where 
        T: Injectable;
    
    async fn resolve_all<T>(&self) -> Result<Vec<Arc<T>>, DIError> 
    where 
        T: Injectable;
    
    async fn create_scope(&self, scope_name: &str) -> Result<ScopedRegistry, DIError>;
    
    fn service_info(&self, service_type: TypeId) -> Option<ServiceInfo>;
    fn list_services(&self) -> Vec<ServiceInfo>;
    
    async fn start_services(&self) -> Result<(), DIError>;
    async fn stop_services(&self) -> Result<(), DIError>;
    
    fn health_status(&self) -> RegistryHealth;
}

#[async_trait]
pub trait ServiceFactory<T>: Send + Sync 
where 
    T: Injectable
{
    async fn create(&self, registry: &ServiceRegistry) -> Result<T, DIError>;
}

pub struct DefaultServiceRegistry {
    services: Arc<RwLock<HashMap<TypeId, ServiceEntry>>>,
    factories: Arc<RwLock<HashMap<TypeId, Box<dyn ServiceFactoryTrait>>>>,
    named_services: Arc<RwLock<HashMap<String, TypeId>>>,
    scoped_registries: Arc<RwLock<HashMap<String, ScopedRegistry>>>,
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    lifecycle_manager: ServiceLifecycleManager,
    // Data flow validation for dependency injection (Agent 12)
    injection_validator: InjectionValidator,
    service_flow_monitor: ServiceFlowMonitor,
}

// Injection Validator (Agent 12 Integration)
pub struct InjectionValidator {
    dependency_checker: DependencyChecker,
    circular_detector: CircularDependencyDetector,
    flow_tracker: ServiceFlowTracker,
}

impl InjectionValidator {
    pub async fn validate_injection_flow<T: Injectable>(
        &self,
        service_type: TypeId,
        dependencies: &[Dependency]
    ) -> Result<(), DIError> {
        // Validate dependency flow integrity
        for dep in dependencies {
            self.dependency_checker.validate_dependency_flow(
                service_type,
                dep.service_type,
                &dep.scope
            )?;
        }
        
        // Check for data flow consistency
        self.flow_tracker.validate_service_data_flow::<T>().await?;
        
        Ok(())
    }
    
    pub fn validate_service_lifecycle(
        &self,
        service_info: &ServiceInfo,
        dependencies: &[Dependency]
    ) -> Result<(), DIError> {
        // Ensure proper lifecycle management for data flow
        match service_info.scope {
            ServiceScope::Singleton => {
                // Singletons should not depend on scoped services
                for dep in dependencies {
                    if matches!(dep.scope, DependencyScope::PerRequest | DependencyScope::Scoped { .. }) {
                        return Err(DIError::ServiceCreationFailed {
                            reason: "Singleton cannot depend on scoped service".to_string()
                        });
                    }
                }
            },
            _ => {}
        }
        
        Ok(())
    }
}

struct ServiceEntry {
    instance: Option<Arc<dyn Any + Send + Sync>>,
    info: ServiceInfo,
    scope: ServiceScope,
    health: ServiceHealth,
    dependencies: Vec<Dependency>,
    created_at: chrono::DateTime<chrono::Utc>,
    last_accessed: chrono::DateTime<chrono::Utc>,
}

trait ServiceFactoryTrait: Send + Sync {
    fn create_service(&self, registry: &ServiceRegistry) -> BoxFuture<Result<Arc<dyn Any + Send + Sync>, DIError>>;
}

impl<T, F> ServiceFactoryTrait for F 
where 
    T: Injectable,
    F: ServiceFactory<T> + Send + Sync,
{
    fn create_service(&self, registry: &ServiceRegistry) -> BoxFuture<Result<Arc<dyn Any + Send + Sync>, DIError>> {
        Box::pin(async move {
            let service = self.create(registry).await?;
            let arc_service: Arc<T> = Arc::new(service);
            Ok(arc_service as Arc<dyn Any + Send + Sync>)
        })
    }
}

impl DefaultServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
            named_services: Arc::new(RwLock::new(HashMap::new())),
            scoped_registries: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::new())),
            lifecycle_manager: ServiceLifecycleManager::new(),
        }
    }
    
    async fn validate_dependencies<T: Injectable>(&self) -> Result<(), DIError> {
        let dependencies = T::dependencies();
        let services = self.services.read().await;
        
        for dependency in &dependencies {
            if dependency.required && !services.contains_key(&dependency.service_type) {
                return Err(DIError::DependencyNotFound {
                    service: std::any::type_name::<T>().to_string(),
                    dependency: dependency.service_name.clone(),
                });
            }
        }
        
        // Check for circular dependencies
        {
            let mut graph = self.dependency_graph.write().await;
            graph.add_service::<T>(dependencies.clone())?;
            graph.validate_no_cycles()?;
        }
        
        // Data flow validation (Agent 12)
        self.injection_validator.validate_injection_flow::<T>(
            TypeId::of::<T>(),
            &dependencies
        ).await?;
        
        // Validate service lifecycle for data flow integrity
        let service_info = T::service_info();
        self.injection_validator.validate_service_lifecycle(
            &service_info,
            &dependencies
        )?;
        
        Ok(())
    }
    
    async fn create_service_instance<T: Injectable>(&self) -> Result<Arc<T>, DIError> {
        // Get or create factory
        let factory = {
            let factories = self.factories.read().await;
            if let Some(factory) = factories.get(&TypeId::of::<T>()) {
                factory.clone()
            } else {
                return Err(DIError::FactoryNotFound {
                    service: std::any::type_name::<T>().to_string(),
                });
            }
        };
        
        // Create service instance
        let any_service = factory.create_service(self).await?;
        let service = any_service.downcast::<T>()
            .map_err(|_| DIError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                actual: "unknown".to_string(),
            })?;
        
        Ok(service)
    }
}

#[async_trait]
impl ServiceRegistry for DefaultServiceRegistry {
    async fn register<T>(&self, service: T) -> Result<(), DIError> 
    where 
        T: Injectable
    {
        self.validate_dependencies::<T>().await?;
        
        let type_id = TypeId::of::<T>();
        let info = T::service_info();
        let dependencies = T::dependencies();
        
        let entry = ServiceEntry {
            instance: Some(Arc::new(service) as Arc<dyn Any + Send + Sync>),
            info: info.clone(),
            scope: info.scope.clone(),
            health: ServiceHealth::Healthy,
            dependencies,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
        };
        
        {
            let mut services = self.services.write().await;
            services.insert(type_id, entry);
        }
        
        {
            let mut named_services = self.named_services.write().await;
            named_services.insert(info.name.clone(), type_id);
        }
        
        Ok(())
    }
    
    async fn register_factory<T, F>(&self, factory: F) -> Result<(), DIError>
    where 
        T: Injectable,
        F: ServiceFactory<T> + Send + Sync + 'static
    {
        self.validate_dependencies::<T>().await?;
        
        let type_id = TypeId::of::<T>();
        let info = T::service_info();
        let dependencies = T::dependencies();
        
        {
            let mut factories = self.factories.write().await;
            factories.insert(type_id, Box::new(factory));
        }
        
        let entry = ServiceEntry {
            instance: None,
            info: info.clone(),
            scope: info.scope.clone(),
            health: ServiceHealth::Healthy,
            dependencies,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
        };
        
        {
            let mut services = self.services.write().await;
            services.insert(type_id, entry);
        }
        
        {
            let mut named_services = self.named_services.write().await;
            named_services.insert(info.name.clone(), type_id);
        }
        
        Ok(())
    }
    
    async fn resolve<T>(&self) -> Result<Arc<T>, DIError> 
    where 
        T: Injectable
    {
        let type_id = TypeId::of::<T>();
        
        {
            let mut services = self.services.write().await;
            if let Some(entry) = services.get_mut(&type_id) {
                entry.last_accessed = chrono::Utc::now();
                
                if let Some(instance) = &entry.instance {
                    if let Ok(service) = instance.clone().downcast::<T>() {
                        return Ok(service);
                    }
                } else {
                    // Create instance for factory-registered services
                    drop(services); // Release lock before calling create_service_instance
                    let instance = self.create_service_instance::<T>().await?;
                    
                    // Re-acquire lock and store instance
                    let mut services = self.services.write().await;
                    if let Some(entry) = services.get_mut(&type_id) {
                        entry.instance = Some(instance.clone() as Arc<dyn Any + Send + Sync>);
                    }
                    
                    return Ok(instance);
                }
            }
        }
        
        Err(DIError::ServiceNotFound {
            service: std::any::type_name::<T>().to_string(),
        })
    }
    
    async fn resolve_named<T>(&self, name: &str) -> Result<Arc<T>, DIError> 
    where 
        T: Injectable
    {
        let type_id = {
            let named_services = self.named_services.read().await;
            named_services.get(name).copied()
                .ok_or_else(|| DIError::ServiceNotFound {
                    service: name.to_string(),
                })?
        };
        
        if type_id == TypeId::of::<T>() {
            self.resolve::<T>().await
        } else {
            Err(DIError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                actual: name.to_string(),
            })
        }
    }
    
    async fn resolve_all<T>(&self) -> Result<Vec<Arc<T>>, DIError> 
    where 
        T: Injectable
    {
        let services = self.services.read().await;
        let mut results = Vec::new();
        
        for (type_id, entry) in services.iter() {
            if *type_id == TypeId::of::<T>() {
                if let Some(instance) = &entry.instance {
                    if let Ok(service) = instance.clone().downcast::<T>() {
                        results.push(service);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn create_scope(&self, scope_name: &str) -> Result<ScopedRegistry, DIError> {
        let scoped_registry = ScopedRegistry::new(scope_name, self).await?;
        
        {
            let mut scoped_registries = self.scoped_registries.write().await;
            scoped_registries.insert(scope_name.to_string(), scoped_registry.clone());
        }
        
        Ok(scoped_registry)
    }
    
    fn service_info(&self, service_type: TypeId) -> Option<ServiceInfo> {
        // Block to ensure lock is released properly
        let services_result = {
            let services = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.services.read().await
                })
            });
            
            services.get(&service_type).map(|entry| entry.info.clone())
        };
        
        services_result
    }
    
    fn list_services(&self) -> Vec<ServiceInfo> {
        // Block to ensure lock is released properly
        let services = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.services.read().await
            })
        });
        
        services.values()
            .map(|entry| entry.info.clone())
            .collect()
    }
    
    async fn start_services(&self) -> Result<(), DIError> {
        self.lifecycle_manager.start_all(self).await
    }
    
    async fn stop_services(&self) -> Result<(), DIError> {
        self.lifecycle_manager.stop_all(self).await
    }
    
    fn health_status(&self) -> RegistryHealth {
        // Block to ensure lock is released properly
        let services = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.services.read().await
            })
        });
        
        let mut unhealthy_services = Vec::new();
        let mut critical_services_down = Vec::new();
        
        for (_, entry) in services.iter() {
            match &entry.health {
                ServiceHealth::Degraded { reason } => {
                    unhealthy_services.push(format!("{}: {}", entry.info.name, reason));
                }
                ServiceHealth::Unhealthy { reason } => {
                    // Check if this is a critical service based on tags
                    if entry.info.tags.contains(&"critical".to_string()) {
                        critical_services_down.push(format!("{}: {}", entry.info.name, reason));
                    } else {
                        unhealthy_services.push(format!("{}: {}", entry.info.name, reason));
                    }
                }
                ServiceHealth::Healthy => {}
            }
        }
        
        if !critical_services_down.is_empty() {
            RegistryHealth::Unhealthy { critical_services_down }
        } else if !unhealthy_services.is_empty() {
            RegistryHealth::Degraded { unhealthy_services }
        } else {
            RegistryHealth::Healthy
        }
    }
}

// Dependency graph for cycle detection
struct DependencyGraph {
    nodes: HashMap<TypeId, DependencyNode>,
}

struct DependencyNode {
    service_name: String,
    dependencies: Vec<TypeId>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
    
    fn add_service<T: Injectable>(&mut self, dependencies: Vec<Dependency>) -> Result<(), DIError> {
        let type_id = TypeId::of::<T>();
        let service_name = std::any::type_name::<T>().to_string();
        
        let dependency_types: Vec<TypeId> = dependencies
            .into_iter()
            .map(|d| d.service_type)
            .collect();
        
        self.nodes.insert(type_id, DependencyNode {
            service_name,
            dependencies: dependency_types,
        });
        
        Ok(())
    }
    
    fn validate_no_cycles(&self) -> Result<(), DIError> {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();
        
        for &node_id in self.nodes.keys() {
            if !visited.get(&node_id).unwrap_or(&false) {
                if self.has_cycle_util(node_id, &mut visited, &mut rec_stack)? {
                    return Err(DIError::CircularDependency {
                        cycle: self.find_cycle_path(node_id),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn has_cycle_util(
        &self, 
        node_id: TypeId, 
        visited: &mut HashMap<TypeId, bool>,
        rec_stack: &mut HashMap<TypeId, bool>
    ) -> Result<bool, DIError> {
        visited.insert(node_id, true);
        rec_stack.insert(node_id, true);
        
        if let Some(node) = self.nodes.get(&node_id) {
            for &dep_id in &node.dependencies {
                if !visited.get(&dep_id).unwrap_or(&false) {
                    if self.has_cycle_util(dep_id, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if *rec_stack.get(&dep_id).unwrap_or(&false) {
                    return Ok(true);
                }
            }
        }
        
        rec_stack.insert(node_id, false);
        Ok(false)
    }
    
    fn find_cycle_path(&self, start_node: TypeId) -> Vec<String> {
        let mut path = Vec::new();
        let mut visited = HashMap::new();
        let mut current = start_node;
        
        // Perform DFS to find the cycle
        loop {
            if visited.contains_key(&current) {
                // Found the cycle start point
                let cycle_start_index = path.iter()
                    .position(|name| name == visited.get(&current).unwrap())
                    .unwrap_or(0);
                
                // Return only the cycle portion
                return path[cycle_start_index..].to_vec();
            }
            
            if let Some(node) = self.nodes.get(&current) {
                let service_name = node.service_name.clone();
                visited.insert(current, service_name.clone());
                path.push(service_name);
                
                // Find the first dependency that leads to a cycle
                let mut found_next = false;
                for &dep_id in &node.dependencies {
                    if self.is_in_cycle(dep_id, start_node) {
                        current = dep_id;
                        found_next = true;
                        break;
                    }
                }
                
                if !found_next {
                    // No cycle found from this path
                    break;
                }
            } else {
                break;
            }
        }
        
        path
    }
    
    fn is_in_cycle(&self, node_id: TypeId, target: TypeId) -> bool {
        let mut visited = HashSet::new();
        self.is_in_cycle_util(node_id, target, &mut visited)
    }
    
    fn is_in_cycle_util(&self, current: TypeId, target: TypeId, visited: &mut HashSet<TypeId>) -> bool {
        if current == target {
            return true;
        }
        
        if visited.contains(&current) {
            return false;
        }
        
        visited.insert(current);
        
        if let Some(node) = self.nodes.get(&current) {
            for &dep_id in &node.dependencies {
                if self.is_in_cycle_util(dep_id, target, visited) {
                    return true;
                }
            }
        }
        
        false
}

// Scoped registry for request-scoped services
#[derive(Clone)]
pub struct ScopedRegistry {
    scope_name: String,
    parent: Arc<dyn ServiceRegistry>,
    scoped_services: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ScopedRegistry {
    async fn new(scope_name: &str, parent: &dyn ServiceRegistry) -> Result<Self, DIError> {
        Ok(Self {
            scope_name: scope_name.to_string(),
            parent: Arc::new(parent), // This won't work due to object safety
            scoped_services: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

// Service lifecycle management
struct ServiceLifecycleManager {
    startup_order: Vec<TypeId>,
    shutdown_order: Vec<TypeId>,
}

impl ServiceLifecycleManager {
    fn new() -> Self {
        Self {
            startup_order: Vec::new(),
            shutdown_order: Vec::new(),
        }
    }
    
    async fn start_all(&self, registry: &dyn ServiceRegistry) -> Result<(), DIError> {
        // Ordered service startup implementation needed
        Ok(())
    }
    
    async fn stop_all(&self, registry: &dyn ServiceRegistry) -> Result<(), DIError> {
        // Ordered service shutdown implementation needed
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum RegistryHealth {
    Healthy,
    Degraded { unhealthy_services: Vec<String> },
    Unhealthy { critical_services_down: Vec<String> },
}

// DI Errors
#[derive(Debug, thiserror::Error)]
pub enum DIError {
    #[error("Service not found: {service}")]
    ServiceNotFound { service: String },
    
    #[error("Factory not found for service: {service}")]
    FactoryNotFound { service: String },
    
    #[error("Dependency not found: {service} requires {dependency}")]
    DependencyNotFound { service: String, dependency: String },
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },
    
    #[error("Service creation failed: {reason}")]
    ServiceCreationFailed { reason: String },
    
    #[error("Scope not found: {scope}")]
    ScopeNotFound { scope: String },
}

// Example service implementations with DI integration
pub struct ExampleAgentService {
    transport: Arc<dyn Transport>,
    config: Arc<dyn ConfigProvider>,
    event_bus: Arc<dyn EventBus>,
}

#[async_trait]
impl Injectable for ExampleAgentService {
    type Config = ExampleAgentConfig;
    type Error = AgentError;
    
    async fn create(config: Self::Config, registry: &ServiceRegistry) -> Result<Self, Self::Error> {
        let transport = registry.resolve::<dyn Transport>().await
            .map_err(|_| AgentError::InitializationFailed("transport not available".to_string()))?;
        let config_provider = registry.resolve::<dyn ConfigProvider>().await
            .map_err(|_| AgentError::InitializationFailed("config provider not available".to_string()))?;
        let event_bus = registry.resolve::<dyn EventBus>().await
            .map_err(|_| AgentError::InitializationFailed("event bus not available".to_string()))?;
        
        Ok(Self {
            transport,
            config: config_provider,
            event_bus,
        })
    }
    
    fn dependencies() -> Vec<Dependency> {
        vec![
            Dependency {
                service_type: TypeId::of::<dyn Transport>(),
                service_name: "transport".to_string(),
                required: true,
                scope: DependencyScope::Singleton,
            },
            Dependency {
                service_type: TypeId::of::<dyn ConfigProvider>(),
                service_name: "config_provider".to_string(),
                required: true,
                scope: DependencyScope::Singleton,
            },
            Dependency {
                service_type: TypeId::of::<dyn EventBus>(),
                service_name: "event_bus".to_string(),
                required: true,
                scope: DependencyScope::Singleton,
            },
        ]
    }
    
    fn service_info() -> ServiceInfo {
        ServiceInfo {
            name: "example_agent_service".to_string(),
            version: "1.0.0".to_string(),
            description: "Example agent service with DI integration".to_string(),
            tags: vec!["agent".to_string(), "example".to_string()],
            scope: ServiceScope::Singleton,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ExampleAgentConfig {
    pub max_concurrent_tasks: usize,
    pub timeout: Duration,
    pub retry_attempts: u32,
}
```

### Service Configuration Example

This example demonstrates practical dependency injection for configuring services:

```rust
use std::sync::Arc;
use async_trait::async_trait;

// Example: Setting up a complete agent system with DI
pub async fn setup_agent_system() -> Result<Arc<dyn ServiceRegistry>, DIError> {
    let registry = DefaultServiceRegistry::new();
    
    // Register configuration provider
    let config_provider = HierarchicalConfig::builder()
        .add_environment_variables()
        .add_file("config/base.toml")?
        .add_file("config/production.toml")?
        .add_kubernetes_secrets("mister-smith")
        .build();
    
    registry.register(config_provider).await?;
    
    // Register transport with factory
    registry.register_factory::<NatsTransport, _>(NatsTransportFactory::new()).await?;
    
    // Register event bus with dependencies
    registry.register_factory::<DefaultEventBus, _>(EventBusFactory::new()).await?;
    
    // Register security services
    registry.register_factory::<SecurityManager, _>(SecurityManagerFactory::new()).await?;
    
    // Register data services
    registry.register_factory::<DatabaseService, _>(DatabaseServiceFactory::new()).await?;
    
    // Register agent services with complex dependencies
    registry.register_factory::<DataProcessingAgent, _>(
        DataProcessingAgentFactory::new()
    ).await?;
    
    registry.register_factory::<WorkflowCoordinatorAgent, _>(
        WorkflowCoordinatorAgentFactory::new()
    ).await?;
    
    // Start all services in dependency order
    registry.start_services().await?;
    
    Ok(Arc::new(registry))
}

// Factory for transport service
pub struct NatsTransportFactory {
    connection_options: ConnectionOptions,
}

#[async_trait]
impl ServiceFactory<NatsTransport> for NatsTransportFactory {
    async fn create(&self, registry: &ServiceRegistry) -> Result<NatsTransport, DIError> {
        // Resolve configuration dependency
        let config_provider = registry.resolve::<dyn ConfigProvider>().await?;
        
        let nats_config = config_provider.get::<NatsConfig>(
            &ConfigKey::new("transport", "nats", "connection")
        ).await
        .map_err(|e| DIError::ServiceCreationFailed {
            reason: format!("Failed to load NATS config: {}", e),
        })?;
        
        // Create NATS client with config
        let client = async_nats::connect_with_options(
            &nats_config.servers,
            self.connection_options.clone()
        ).await
        .map_err(|e| DIError::ServiceCreationFailed {
            reason: format!("Failed to connect to NATS: {}", e),
        })?;
        
        // Create JetStream context
        let jetstream = async_nats::jetstream::new(client.clone());
        
        Ok(NatsTransport {
            client,
            jetstream,
            config: nats_config,
        })
    }
}

// Factory for agent with multiple dependencies
pub struct DataProcessingAgentFactory {
    agent_id: AgentId,
}

#[async_trait]
impl ServiceFactory<DataProcessingAgent> for DataProcessingAgentFactory {
    async fn create(&self, registry: &ServiceRegistry) -> Result<DataProcessingAgent, DIError> {
        // Resolve all required dependencies
        let event_bus = registry.resolve::<dyn EventBus>().await?;
        let config_provider = registry.resolve::<dyn ConfigProvider>().await?;
        let database = registry.resolve::<DatabaseService>().await?;
        let security = registry.resolve::<SecurityManager>().await?;
        
        // Load agent-specific configuration
        let agent_config = config_provider.get::<ProcessingConfig>(
            &ConfigKey::new("agents", "data_processing", "config")
                .with_environment("production")
        ).await?;
        
        // Create processor with injected dependencies
        let processor = DataProcessor::new(
            database.clone(),
            security.clone(),
            agent_config.processing_options.clone(),
        );
        
        let agent = DataProcessingAgent {
            id: self.agent_id,
            event_bus,
            processor,
            config: agent_config,
        };
        
        Ok(agent)
    }
}

// Example: Request-scoped services for HTTP handlers
pub async fn handle_http_request(
    registry: Arc<dyn ServiceRegistry>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Create request-scoped registry
    let request_scope = registry.create_scope(&format!("request-{}", request.id)).await?;
    
    // Register request-specific services
    request_scope.register(RequestContext {
        request_id: request.id,
        user_id: request.user_id.clone(),
        trace_id: request.trace_id.clone(),
    }).await?;
    
    // Resolve handler with request-scoped dependencies
    let handler = request_scope.resolve::<RequestHandler>().await?;
    
    // Process request
    let response = handler.process(request).await?;
    
    Ok(response)
}

// Health check using dependency injection
pub async fn perform_system_health_check(
    registry: &dyn ServiceRegistry,
) -> Result<SystemHealthReport, Error> {
    let mut report = SystemHealthReport::new();
    
    // Check registry health
    let registry_health = registry.health_status();
    report.add_component("service_registry", registry_health);
    
    // Check all registered services
    for service_info in registry.list_services() {
        if service_info.tags.contains(&"healthcheck".to_string()) {
            // Services tagged for health checking
            match registry.resolve_named::<dyn HealthCheckable>(&service_info.name).await {
                Ok(service) => {
                    let health = service.check_health().await;
                    report.add_component(&service_info.name, health);
                }
                Err(e) => {
                    report.add_component(
                        &service_info.name,
                        ComponentHealth::Unhealthy {
                            error: format!("Failed to resolve service: {}", e),
                        }
                    );
                }
            }
        }
    }
    
    Ok(report)
}
```

---

## Data Flow Integrity Validation Integration

The following patterns integrate data flow validation across all framework components:

#### 1. End-to-End Data Flow Validation (95/100)

- **Message Path Tracking**: Complete tracking from agent orchestration through message processing to data persistence
- **State Hydration**: Validated mechanisms for state restoration with consistency checks
- **Cross-Component Boundaries**: Explicit validation at component interfaces

#### 2. Message Transformation Integrity (94/100)

```rust
// Transformation validation pattern integrated
pub trait TransformationValidator {
    async fn validate_transformation<T, U>(
        &self,
        input: &T,
        output: &U,
        transformation: &Transformation
    ) -> Result<(), DataFlowError>
    where
        T: Serialize,
        U: Serialize;
}
```

#### 3. Error Handling Coverage (91/100)

- **Message-Level**: Complete validation error classification
- **Storage-Level**: Transaction rollback with flow tracking
- **Agent-Level**: State transition error recovery with flow preservation

#### 4. Performance Monitoring Integration

```rust
// Performance thresholds from Agent 12 analysis
pub struct DataFlowPerformanceThresholds {
    pub message_routing_latency: Duration,      // < 1ms
    pub state_persistence_latency: Duration,    // < 5ms
    pub cross_agent_comm_latency: Duration,     // < 2ms
    pub database_query_latency: Duration,       // < 10ms
}
```

#### 5. Security Enhancements (88/100)

- **Replay Attack Prevention**: Integrated into event publishing flow
- **Message Authentication**: Added checksum validation
- **Audit Trail**: Transformation tracking for security compliance

### Connection Management Patterns

#### 1. Connection Pool Data Flow Validation

```rust
pub struct ConnectionPoolFlowValidator {
    connection_tracker: ConnectionTracker,
    flow_monitor: FlowMonitor,
    health_validator: HealthValidator,
}

impl ConnectionPoolFlowValidator {
    pub async fn validate_connection_flow<R: Resource>(
        &self,
        pool: &ConnectionPool<R>,
        resource: &R
    ) -> Result<(), DataFlowError> {
        // Validate connection state transitions
        self.connection_tracker.validate_state_transition(resource)?;
        
        // Monitor data flow through connection
        self.flow_monitor.track_connection_usage(resource).await;
        
        // Validate connection health for data integrity
        if !resource.is_healthy() {
            return Err(DataFlowError::ConsistencyViolation(
                "Unhealthy connection in data flow".to_string()
            ));
        }
        
        Ok(())
    }
}
```

#### 2. Cross-System Integration Validation

```rust
// Integration point validation from Agent 12
pub struct IntegrationPointValidator {
    schema_validator: SchemaValidator,
    contract_validator: ContractValidator,
    flow_validator: FlowValidator,
}

impl IntegrationPointValidator {
    pub async fn validate_integration(
        &self,
        source_component: &str,
        target_component: &str,
        data: &[u8]
    ) -> Result<IntegrationValidation, DataFlowError> {
        // Validate schema compatibility
        let source_schema = self.schema_validator.get_schema(source_component)?;
        let target_schema = self.schema_validator.get_schema(target_component)?;
        
        self.schema_validator.validate_compatibility(
            &source_schema,
            &target_schema
        )?;
        
        // Validate contract adherence
        self.contract_validator.validate_contract(
            source_component,
            target_component,
            data
        ).await?;
        
        // Validate data flow patterns
        self.flow_validator.validate_flow_pattern(
            source_component,
            target_component
        )?;
        
        Ok(IntegrationValidation {
            compatible: true,
            warnings: vec![],
            performance_impact: None,
        })
    }
}
```

### Validation Metrics and Monitoring

#### Real-time Flow Monitoring

```rust
pub struct DataFlowMonitor {
    metrics_collector: MetricsCollector,
    anomaly_detector: AnomalyDetector,
    alert_manager: AlertManager,
}

impl DataFlowMonitor {
    pub async fn monitor_flow_health(&self) -> FlowHealthReport {
        let metrics = self.metrics_collector.collect_flow_metrics().await;
        
        FlowHealthReport {
            message_throughput: metrics.messages_per_second,
            average_latency: metrics.average_latency,
            error_rate: metrics.error_rate,
            data_consistency_score: metrics.consistency_score,
            integration_health: metrics.integration_health,
        }
    }
}
```

---

## Cross-Domain Integration Validation

### Architectural Consistency Verification

Based on comprehensive validation (ref: `/validation-bridge/team-alpha-validation/agent04-architectural-consistency-validation.md`),
the integration patterns demonstrate **STRONG** consistency and completeness across all framework domains:

#### Integration Points Validated ✅

**1. Core Architecture ↔ Data Management Integration**

- **Status**: COMPLETE
- Agent lifecycle integration through supervision trees
- Consistent agent trait implementations across domains
- Shared task management framework with unified TaskId/AgentId types
- Message schema consistency with UUID-based identification

**2. Security ↔ Transport Layer Integration**

- **Status**: EXCELLENT
- mTLS implementation consistent across NATS, gRPC, and HTTP transports
- Unified certificate management patterns
- JWT patterns consistently applied with bearer token extraction
- Claims validation unified across all transport mechanisms

**3. Operations ↔ All Domain Integration**

- **Status**: STRONG
- OpenTelemetry patterns applied uniformly
- Consistent instrumentation approach using spans and metrics
- Unified metrics collection with standardized naming conventions
- Error rate tracking consistent across all agent types

**4. Testing ↔ Framework-wide Integration**

- **Status**: STRONG
- Consistent `tokio-test` usage for async testing
- Unified mock framework approach across domains
- Standardized test structure with 90%+ coverage requirements
- Integration test patterns validated across domain boundaries

#### Evidence of Integration Excellence

**Transport Layer mTLS Configuration** (consistent pattern):

```yaml
tls:
  cert_file: "/etc/ssl/agent-cert.pem"
  key_file: "/etc/ssl/agent-key.pem"
  ca_file: "/etc/ssl/ca-cert.pem"
```

**Cross-Domain Error Propagation**:

```rust
// Every domain properly extends SystemError
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("System error: {0}")]
    System(#[from] SystemError),
    // Domain-specific errors...
}
```

#### Integration Gaps Identified

**Neural Training Framework Integration**:

- Limited architectural integration points specified
- Dependency flows with core framework need clarification
- Recommendation: Extend integration patterns for ML workflows

**Event Bus Implementation**:

- Currently in pseudocode (system-architecture.md lines 3256-3291)
- Blocks full event-driven integration validation
- Priority 0 for implementation completion

---

## Summary

This document provides comprehensive integration patterns for error handling, event-driven communication,
and dependency injection within the Mister Smith framework. These patterns enable robust,
scalable, and maintainable multi-agent systems.

### Key Integration Patterns

- **Error Handling**: Unified hierarchy with automatic recovery strategies, backoff policies, and data flow validation
- **Event System**: Asynchronous pub-sub and request-response patterns with correlation tracking
- **Dependency Injection**: Type-safe service composition with lifecycle management and circular dependency detection

### Implementation Requirements

- Consolidate type definitions in a shared contracts crate
- Complete event correlation and lifecycle management implementations
- Implement production-ready error propagation across component boundaries
- Add comprehensive metrics collection for all integration points

### Related Documentation

- [Integration Contracts](./integration-contracts.md) - Core contracts and interfaces
- [Component Architecture](./component-architecture.md) - Component design patterns
- [Async Patterns](./async-patterns.md) - Asynchronous integration patterns
- [System Integration](./system-integration.md) - System-level integration approaches
- [Tokio Runtime](./tokio-runtime.md) - Runtime configuration

---

[← System Integration](system-integration.md) | [↑ Core Architecture](CLAUDE.md) | [Implementation Configuration →](implementation-config.md)

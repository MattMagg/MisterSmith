//! Error hierarchy for the Mister Smith framework.
//!
//! Provides a comprehensive error taxonomy with 11 domain-specific error types
//! converging to a top-level [`SystemError`] via `#[from]` conversions.

use std::time::Duration;
use thiserror::Error;

/// Runtime subsystem errors.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Failed to build the Tokio runtime.
    #[error("Failed to build runtime: {0}")]
    BuildFailed(#[from] std::io::Error),
    /// Runtime startup failed.
    #[error("Runtime startup failed: {0}")]
    StartupFailed(String),
    /// Runtime shutdown failed.
    #[error("Runtime shutdown failed: {0}")]
    ShutdownFailed(String),
    /// Runtime configuration is invalid.
    #[error("Runtime configuration invalid: {0}")]
    ConfigurationInvalid(String),
}

/// Supervision tree errors.
#[derive(Debug, Error)]
pub enum SupervisionError {
    /// Supervision strategy execution failed.
    #[error("Supervision strategy failed: {0}")]
    StrategyFailed(String),
    /// Child restart failed.
    #[error("Child restart failed: {0}")]
    RestartFailed(String),
    /// Escalation to parent supervisor failed.
    #[error("Escalation failed: {0}")]
    EscalationFailed(String),
    /// Maximum restart attempts exceeded within the failure window.
    #[error("Maximum restart attempts exceeded")]
    RestartLimitExceeded,
    /// Supervision tree integrity is compromised.
    #[error("Supervision tree corrupted: {0}")]
    TreeCorrupted(String),
}

/// Actor system errors.
#[derive(Debug, Error)]
pub enum ActorError {
    /// Actor startup failed.
    #[error("Actor startup failed: {0}")]
    StartupFailed(Box<dyn std::error::Error + Send + Sync>),
    /// Actor mailbox is full.
    #[error("Mailbox is full")]
    MailboxFull,
    /// Actor has stopped.
    #[error("Actor has stopped")]
    ActorStopped,
    /// Actor system has stopped.
    #[error("Actor system has stopped")]
    SystemStopped,
    /// Ask operation timed out.
    #[error("Ask operation timed out")]
    AskTimeout,
    /// Message deserialization failed.
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    /// Message handling failed.
    #[error("Message handling failed: {0}")]
    MessageHandlingFailed(String),
}

/// Task execution errors.
#[derive(Debug, Error)]
pub enum TaskError {
    /// Task execution failed.
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),
    /// Task timed out.
    #[error("Task timed out")]
    TimedOut,
    /// Task was cancelled.
    #[error("Task was cancelled")]
    TaskCancelled,
    /// Task executor is shutting down.
    #[error("Task executor is shutting down")]
    ExecutorShutdown,
    /// Task queue is full.
    #[error("Task queue is full")]
    QueueFull,
    /// Task serialization failed.
    #[error("Task serialization failed: {0}")]
    SerializationFailed(String),
}

/// Stream processing errors.
#[derive(Debug, Error)]
pub enum StreamError {
    /// Stream processing failed.
    #[error("Stream processing failed: {0}")]
    ProcessingFailed(String),
    /// A named processor in the pipeline failed.
    #[error("Processor '{0}' failed: {1}")]
    ProcessorFailed(String, String),
    /// Sink is full.
    #[error("Sink is full")]
    SinkFull,
    /// Sink is blocked.
    #[error("Sink is blocked")]
    SinkBlocked,
    /// Stream ended unexpectedly.
    #[error("Stream ended unexpectedly")]
    StreamEnded,
    /// Backpressure handling failed.
    #[error("Backpressure handling failed: {0}")]
    BackpressureFailed(String),
}

/// Event system errors.
#[derive(Debug, Error)]
pub enum EventError {
    /// Event handler failed.
    #[error("Event handler failed: {0}")]
    HandlerFailed(String),
    /// Event serialization failed.
    #[error("Event serialization failed: {0}")]
    SerializationFailed(String),
    /// Event publication failed.
    #[error("Event publication failed: {0}")]
    PublicationFailed(String),
    /// Event subscription failed.
    #[error("Event subscription failed: {0}")]
    SubscriptionFailed(String),
    /// Event store operation failed.
    #[error("Event store operation failed: {0}")]
    StoreFailed(String),
}

/// Tool system errors.
#[derive(Debug, Error)]
pub enum ToolError {
    /// Tool execution failed.
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),
    /// Tool not found.
    #[error("Tool not found: {0}")]
    NotFound(String),
    /// Tool access denied.
    #[error("Tool access denied: {0}")]
    AccessDenied(String),
    /// Tool parameter validation failed.
    #[error("Tool parameter validation failed: {0}")]
    ParameterValidationFailed(String),
    /// Tool operation timed out.
    #[error("Tool timeout: {0}")]
    Timeout(String),
}

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration validation failed.
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    /// Configuration file not found.
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    /// Configuration parsing failed.
    #[error("Configuration parsing failed: {0}")]
    ParseFailed(String),
    /// Configuration merge failed.
    #[error("Configuration merge failed: {0}")]
    MergeFailed(String),
}

/// Resource management errors.
#[derive(Debug, Error)]
pub enum ResourceError {
    /// Resource acquisition failed.
    #[error("Resource acquisition failed: {0}")]
    AcquisitionFailed(String),
    /// Resource pool exhausted.
    #[error("Resource pool exhausted")]
    PoolExhausted,
    /// Resource health check failed.
    #[error("Resource health check failed: {0}")]
    HealthCheckFailed(String),
    /// Resource cleanup failed.
    #[error("Resource cleanup failed: {0}")]
    CleanupFailed(String),
}

/// Network and transport errors.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Network connection failed.
    #[error("Network connection failed: {0}")]
    ConnectionFailed(String),
    /// Network operation timed out.
    #[error("Network timeout: {0}")]
    Timeout(String),
    /// Network protocol error.
    #[error("Network protocol error: {0}")]
    ProtocolError(String),
}

/// Persistence and storage errors.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Database operation failed.
    #[error("Database operation failed: {0}")]
    DatabaseFailed(String),
    /// Serialization failed.
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    /// Data corruption detected.
    #[error("Data corruption detected: {0}")]
    DataCorrupted(String),
    /// Entity or key not found.
    #[error("Not found: {0}")]
    NotFound(String),
    /// Unique constraint violation.
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),
    /// Optimistic concurrency conflict — expected revision does not match actual.
    #[error("Version conflict on key '{key}': expected {expected}, actual {actual_str}", actual_str = .actual.map_or("unknown".to_string(), |v| v.to_string()))]
    VersionConflict {
        /// The key that had a conflict.
        key: String,
        /// The expected revision.
        expected: u64,
        /// The actual revision found (None when the backend does not expose it).
        actual: Option<u64>,
    },
    /// Storage backend is unreachable.
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    /// KV entry expired before it could be read or flushed.
    #[error("TTL expired: {0}")]
    TtlExpired(String),
    /// Schema migration failed.
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// Security subsystem errors.
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    /// Token has expired.
    #[error("Token expired")]
    TokenExpired,
    /// Token has been revoked.
    #[error("Token revoked")]
    TokenRevoked,
    /// Token signature is invalid.
    #[error("Invalid signature")]
    InvalidSignature,
    /// Token is malformed or otherwise invalid.
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    /// Authorization denied.
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    /// Insufficient permissions for the requested action.
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    /// Certificate loading failed.
    #[error("Certificate load failed: {0}")]
    CertificateLoadFailed(String),
    /// Certificate generation failed.
    #[error("Certificate generation failed: {0}")]
    CertificateGenerationFailed(String),
    /// TLS configuration failed.
    #[error("TLS config failed: {0}")]
    TlsConfigFailed(String),
    /// Key loading failed.
    #[error("Key load failed: {0}")]
    KeyLoadFailed(String),
    /// Token generation failed.
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
    /// Rate limited — caller should retry after the given duration.
    #[error("Rate limited (retry after {0:?})")]
    RateLimited(Duration),
}

/// Top-level error type aggregating all subsystem errors.
///
/// All domain-specific errors can be converted to `SystemError` via `#[from]`.
#[derive(Debug, Error)]
pub enum SystemError {
    /// Runtime error.
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    /// Supervision error.
    #[error("Supervision error: {0}")]
    Supervision(#[from] SupervisionError),
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    /// Resource error.
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
    /// Network error.
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    /// Persistence error.
    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),
    /// Actor error.
    #[error("Actor system error: {0}")]
    Actor(#[from] ActorError),
    /// Task error.
    #[error("Task execution error: {0}")]
    Task(#[from] TaskError),
    /// Stream error.
    #[error("Stream processing error: {0}")]
    Stream(#[from] StreamError),
    /// Event error.
    #[error("Event system error: {0}")]
    Event(#[from] EventError),
    /// Tool error.
    #[error("Tool system error: {0}")]
    Tool(#[from] ToolError),
    /// Security error.
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
}

/// Error severity for system-wide error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Low severity — non-critical, can be retried.
    Low,
    /// Medium severity — requires attention but not urgent.
    Medium,
    /// High severity — needs prompt resolution.
    High,
    /// Critical severity — system stability at risk.
    Critical,
}

/// Strategy for recovering from errors.
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff.
    Retry {
        /// Maximum number of retry attempts.
        max_attempts: u32,
        /// Delay between retries.
        delay: Duration,
    },
    /// Restart the failed component.
    Restart,
    /// Escalate to a supervisor.
    Escalate,
    /// Reload configuration and retry.
    Reload,
    /// Open circuit breaker to prevent cascade.
    CircuitBreaker,
    /// Failover to a backup system.
    Failover,
    /// Ignore the error (log only).
    Ignore,
}

impl SystemError {
    /// Returns the severity level for this error.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SystemError::Runtime(_) => ErrorSeverity::Critical,
            SystemError::Supervision(_) => ErrorSeverity::High,
            SystemError::Configuration(_) => ErrorSeverity::Medium,
            SystemError::Resource(_) => ErrorSeverity::Medium,
            SystemError::Network(_) => ErrorSeverity::Low,
            SystemError::Persistence(_) => ErrorSeverity::High,
            SystemError::Actor(_) => ErrorSeverity::Medium,
            SystemError::Task(_) => ErrorSeverity::Low,
            SystemError::Stream(_) => ErrorSeverity::Medium,
            SystemError::Event(_) => ErrorSeverity::Low,
            SystemError::Tool(_) => ErrorSeverity::Low,
            SystemError::Security(_) => ErrorSeverity::Medium,
        }
    }

    /// Returns the recommended recovery strategy for this error.
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            SystemError::Runtime(_) => RecoveryStrategy::Restart,
            SystemError::Supervision(_) => RecoveryStrategy::Escalate,
            SystemError::Configuration(_) => RecoveryStrategy::Reload,
            SystemError::Resource(_) => RecoveryStrategy::Retry {
                max_attempts: 3,
                delay: Duration::from_millis(1000),
            },
            SystemError::Network(_) => RecoveryStrategy::CircuitBreaker,
            SystemError::Persistence(_) => RecoveryStrategy::Failover,
            _ => RecoveryStrategy::Retry {
                max_attempts: 1,
                delay: Duration::from_millis(100),
            },
        }
    }
}

/// Convenience result type using [`SystemError`].
pub type FrameworkResult<T> = Result<T, SystemError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_conversions_compile() {
        let _: SystemError = RuntimeError::StartupFailed("test".into()).into();
        let _: SystemError = SupervisionError::RestartLimitExceeded.into();
        let _: SystemError = ConfigError::ValidationFailed("test".into()).into();
        let _: SystemError = ResourceError::PoolExhausted.into();
        let _: SystemError = NetworkError::Timeout("test".into()).into();
        let _: SystemError = PersistenceError::DatabaseFailed("test".into()).into();
        let _: SystemError = ActorError::MailboxFull.into();
        let _: SystemError = TaskError::TimedOut.into();
        let _: SystemError = StreamError::SinkFull.into();
        let _: SystemError = EventError::HandlerFailed("test".into()).into();
        let _: SystemError = ToolError::NotFound("test".into()).into();
        let _: SystemError = SecurityError::TokenExpired.into();
    }

    #[test]
    fn severity_mapping() {
        let runtime_err: SystemError = RuntimeError::StartupFailed("test".into()).into();
        assert_eq!(runtime_err.severity(), ErrorSeverity::Critical);

        let network_err: SystemError = NetworkError::Timeout("test".into()).into();
        assert_eq!(network_err.severity(), ErrorSeverity::Low);

        let supervision_err: SystemError = SupervisionError::RestartLimitExceeded.into();
        assert_eq!(supervision_err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn display_output() {
        let err: SystemError = RuntimeError::StartupFailed("boot failure".into()).into();
        let msg = err.to_string();
        assert!(msg.contains("Runtime error"));
        assert!(msg.contains("boot failure"));
    }

    #[test]
    fn framework_result_usage() {
        fn may_fail() -> FrameworkResult<u32> {
            Err(ConfigError::ValidationFailed("bad value".into()).into())
        }
        assert!(may_fail().is_err());
    }
}

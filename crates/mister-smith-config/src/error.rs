//! Configuration error types.

use thiserror::Error;

/// Errors that can occur during configuration loading and validation.
#[derive(Debug, Error)]
pub enum ConfigValidationError {
    /// A validation constraint was violated.
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// A required field is missing.
    #[error("Missing field: {0}")]
    MissingField(String),
    /// A field has an invalid value.
    #[error("Invalid value for '{field}': {reason}")]
    InvalidValue {
        /// The field name.
        field: String,
        /// Why the value is invalid.
        reason: String,
    },
    /// An environment variable error occurred.
    #[error("Environment variable error: {0}")]
    EnvVarError(String),
    /// A file I/O error occurred.
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    /// TOML deserialization failed.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

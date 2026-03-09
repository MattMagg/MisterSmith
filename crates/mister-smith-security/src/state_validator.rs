//! State validation and sanitization for persistence-to-agent boundaries.
//!
//! This module implements the `StateValidator` contract described in the
//! Phase 9.1 security hardening spec. Validation is intentionally structured
//! as a quarantine pipeline:
//!
//! 1. Enforce a serialized-size limit before deeper inspection.
//! 2. Sanitize control characters that should never reach agent context.
//! 3. Validate against a registered JSON Schema when one exists.
//! 4. Reject known malicious prompt-injection markers.
//! 5. Return taint-labeled data so downstream systems can audit or monitor it.

use std::collections::HashMap;
use std::io::{self, Write};

use parking_lot::RwLock;
use serde_json::Value;

use mister_smith_core::PersistenceError;

/// Classification assigned to validated state data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaintLabel {
    /// The state passed all checks without modification.
    Clean,
    /// The state was modified during sanitization before it was returned.
    Sanitized,
    /// The state passed validation, but monitoring should treat it cautiously.
    Suspicious,
    /// The state failed validation and must not be forwarded to the agent.
    Rejected,
}

/// Validated state returned from the quarantine pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedState {
    /// The validated JSON payload safe to forward to the caller.
    pub data: Value,
    /// The schema identifier used for validation.
    pub schema_version: String,
    /// The classification assigned during validation.
    pub taint_label: TaintLabel,
}

/// Typed validation failures produced by a [`StateValidator`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// The serialized payload exceeded the configured limit.
    #[error("state size {actual_bytes} exceeds maximum allowed {max_bytes} bytes")]
    SizeExceeded {
        /// Actual serialized payload size in bytes.
        actual_bytes: usize,
        /// Maximum allowed serialized size in bytes.
        max_bytes: usize,
    },
    /// The provided schema could not be compiled.
    #[error("schema registration failed for '{schema_ref}': {message}")]
    SchemaRegistration {
        /// State type or schema identifier the schema was registered under.
        schema_ref: String,
        /// Human-readable compilation failure details.
        message: String,
    },
    /// The payload did not satisfy the registered schema.
    #[error("schema violation for '{schema_ref}' at '{path}': {message}")]
    SchemaViolation {
        /// State type or schema identifier that failed validation.
        schema_ref: String,
        /// JSON Pointer describing the failing instance location.
        path: String,
        /// Human-readable validation failure details.
        message: String,
    },
    /// A known malicious pattern was detected in the payload.
    #[error("malicious pattern '{pattern}' detected at '{path}'")]
    MaliciousPattern {
        /// Lower-cased marker string that triggered the rejection.
        pattern: String,
        /// JSON Pointer describing the location of the match.
        path: String,
    },
    /// Sanitization produced duplicate object keys.
    #[error("sanitization conflict at '{path}' for key '{key}'")]
    SanitizationConflict {
        /// JSON Pointer describing the object containing the conflict.
        path: String,
        /// The duplicate key produced during sanitization.
        key: String,
    },
    /// JSON serialization failed while sizing or copying the payload.
    #[error("state serialization failed: {0}")]
    SerializationFailed(String),
}

impl ValidationError {
    /// Returns the taint classification implied by this validation failure.
    #[must_use]
    pub fn taint_label(&self) -> TaintLabel {
        TaintLabel::Rejected
    }
}

impl From<ValidationError> for PersistenceError {
    fn from(error: ValidationError) -> Self {
        match error {
            ValidationError::SerializationFailed(message) => {
                PersistenceError::SerializationFailed(message)
            }
            other => PersistenceError::DataCorrupted(other.to_string()),
        }
    }
}

/// Trait defining validation at the persistence-to-agent boundary.
pub trait StateValidator: Send + Sync {
    /// Validate a state payload for the given state type.
    fn validate(&self, state_type: &str, state: &Value) -> Result<ValidatedState, ValidationError>;

    /// Check the serialized size of a state payload before deeper validation.
    ///
    /// Returns the serialized size in bytes when the payload fits.
    fn check_size(&self, state: &Value) -> Result<usize, ValidationError>;
}

#[derive(Debug)]
struct RegisteredSchema {
    schema_version: String,
    validator: jsonschema::Validator,
}

/// JSON Schema-backed state validator.
///
/// Schemas are registered by state type and compiled once for reuse across
/// validations.
#[derive(Debug)]
pub struct JsonSchemaStateValidator {
    max_bytes: usize,
    schemas: RwLock<HashMap<String, RegisteredSchema>>,
    malicious_patterns: Vec<String>,
}

/// Default prompt-injection patterns checked during malicious-pattern
/// detection. These are always included even when callers supply extra
/// patterns via [`JsonSchemaStateValidator::new_with_patterns`].
const DEFAULT_PATTERNS: &[&str] = &[
    "ignore previous instructions",
    "reveal the system prompt",
    "developer message",
    "override safety",
];

impl JsonSchemaStateValidator {
    /// Create a new validator with the provided serialized size limit.
    #[must_use]
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            schemas: RwLock::new(HashMap::new()),
            malicious_patterns: DEFAULT_PATTERNS.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    /// Create a new validator that merges caller-supplied patterns with the
    /// built-in defaults.
    #[must_use]
    pub fn new_with_patterns(max_bytes: usize, extra_patterns: Vec<String>) -> Self {
        let mut patterns: Vec<String> =
            DEFAULT_PATTERNS.iter().map(|s| (*s).to_string()).collect();
        patterns.extend(extra_patterns);
        Self {
            max_bytes,
            schemas: RwLock::new(HashMap::new()),
            malicious_patterns: patterns,
        }
    }

    /// Register a compiled schema for a state type.
    pub fn register_schema(
        &self,
        state_type: impl Into<String>,
        schema: Value,
    ) -> Result<(), ValidationError> {
        let state_type = state_type.into();
        let validator = jsonschema::validator_for(&schema).map_err(|error| {
            ValidationError::SchemaRegistration {
                schema_ref: state_type.clone(),
                message: error.to_string(),
            }
        })?;

        self.schemas.write().insert(
            state_type.clone(),
            RegisteredSchema {
                schema_version: state_type,
                validator,
            },
        );

        Ok(())
    }

    fn sanitize_state(&self, state: &Value, path: &str) -> Result<(Value, bool), ValidationError> {
        match state {
            Value::String(value) => {
                let sanitized: String = value.chars().filter(|ch| is_safe_character(*ch)).collect();
                let changed = sanitized != *value;
                Ok((Value::String(sanitized), changed))
            }
            Value::Array(values) => {
                let mut changed = false;
                let mut sanitized = Vec::with_capacity(values.len());
                for (index, value) in values.iter().enumerate() {
                    let child_path = format!("{path}/{index}");
                    let (value, value_changed) = self.sanitize_state(value, &child_path)?;
                    changed |= value_changed;
                    sanitized.push(value);
                }
                Ok((Value::Array(sanitized), changed))
            }
            Value::Object(entries) => {
                let mut changed = false;
                let mut sanitized = serde_json::Map::with_capacity(entries.len());
                for (key, value) in entries {
                    let sanitized_key = sanitize_string(key);
                    changed |= sanitized_key != *key;
                    let child_path =
                        format!("{path}/{}", escape_json_pointer_segment(&sanitized_key));
                    let (sanitized_value, value_changed) =
                        self.sanitize_state(value, &child_path)?;
                    changed |= value_changed;

                    if sanitized
                        .insert(sanitized_key.clone(), sanitized_value)
                        .is_some()
                    {
                        return Err(ValidationError::SanitizationConflict {
                            path: path.to_string(),
                            key: sanitized_key,
                        });
                    }
                }
                Ok((Value::Object(sanitized), changed))
            }
            _ => Ok((state.clone(), false)),
        }
    }

    fn detect_malicious_pattern(&self, state: &Value, path: &str) -> Option<(String, String)> {
        match state {
            Value::String(value) => {
                let normalized = value.to_ascii_lowercase();
                self.malicious_patterns.iter().find_map(|pattern| {
                    normalized
                        .contains(pattern)
                        .then(|| (pattern.clone(), path.to_string()))
                })
            }
            Value::Array(values) => values.iter().enumerate().find_map(|(index, value)| {
                let child_path = format!("{path}/{index}");
                self.detect_malicious_pattern(value, &child_path)
            }),
            Value::Object(entries) => entries.iter().find_map(|(key, value)| {
                let child_path = format!("{path}/{}", escape_json_pointer_segment(key));
                let normalized_key = key.to_ascii_lowercase();
                if let Some(pattern) = self
                    .malicious_patterns
                    .iter()
                    .find(|pattern| normalized_key.contains(pattern.as_str()))
                {
                    return Some((pattern.clone(), child_path.clone()));
                }
                self.detect_malicious_pattern(value, &child_path)
            }),
            _ => None,
        }
    }
}

impl StateValidator for JsonSchemaStateValidator {
    fn validate(&self, state_type: &str, state: &Value) -> Result<ValidatedState, ValidationError> {
        self.check_size(state)?;

        let (sanitized, was_sanitized) = self.sanitize_state(state, "")?;
        let registered = self.schemas.read();

        if let Some(schema) = registered.get(state_type) {
            // Capture schema result without early-returning so we can still run
            // the injection scan. Injection detection takes priority because a
            // schema failure might be benign (extra field) while an injection is
            // always hostile.
            let schema_result =
                schema
                    .validator
                    .validate(&sanitized)
                    .map_err(|error| ValidationError::SchemaViolation {
                        schema_ref: state_type.to_string(),
                        path: error.instance_path().as_str().to_string(),
                        message: error.to_string(),
                    });

            // Always run pattern scan — injection detection takes priority over
            // schema errors.
            if let Some((pattern, path)) = self.detect_malicious_pattern(&sanitized, "") {
                return Err(ValidationError::MaliciousPattern { pattern, path });
            }

            // Now propagate schema error if any.
            schema_result?;

            return Ok(ValidatedState {
                data: sanitized,
                schema_version: schema.schema_version.clone(),
                taint_label: if was_sanitized {
                    TaintLabel::Sanitized
                } else {
                    TaintLabel::Clean
                },
            });
        }

        drop(registered);

        if let Some((pattern, path)) = self.detect_malicious_pattern(&sanitized, "") {
            return Err(ValidationError::MaliciousPattern { pattern, path });
        }

        Ok(ValidatedState {
            data: sanitized,
            schema_version: state_type.to_string(),
            taint_label: TaintLabel::Suspicious,
        })
    }

    fn check_size(&self, state: &Value) -> Result<usize, ValidationError> {
        let mut counter = ByteCounter::new();
        serde_json::to_writer(&mut counter, state)
            .map_err(|error| ValidationError::SerializationFailed(error.to_string()))?;
        let actual_bytes = counter.count;

        if actual_bytes > self.max_bytes {
            return Err(ValidationError::SizeExceeded {
                actual_bytes,
                max_bytes: self.max_bytes,
            });
        }

        Ok(actual_bytes)
    }
}

fn is_safe_character(ch: char) -> bool {
    !ch.is_control() || matches!(ch, '\n' | '\r' | '\t')
}

fn sanitize_string(value: &str) -> String {
    value.chars().filter(|ch| is_safe_character(*ch)).collect()
}

fn escape_json_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

struct ByteCounter {
    count: usize,
}

impl ByteCounter {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.count += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

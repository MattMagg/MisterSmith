# Contract: State Validator

## Overview

The `StateValidator` contract defines the data quarantine layer between persistence retrieval
and agent consumption. All state retrieved from PostgreSQL or JetStream KV must pass through
validation before entering an agent's working context.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/04-security-and-trust.md` | Infectious jailbreaks via shared memory, MINJA memory injection |
| `crates/mister-smith-persistence/src/repository/agent.rs` | Current `get_state()` returns raw `Option<Value>` — gap identified |
| `spec/security/` | Existing audit patterns for validation events |

## Public API

```rust
pub trait StateValidator: Send + Sync {
    /// Validate state against a registered schema, enforcing size limits first
    fn validate(
        &self,
        state: &serde_json::Value,
        schema_ref: &SchemaRef,
    ) -> Result<ValidatedState, ValidationError>;

    /// Check if state exceeds size limits (called before schema validation)
    fn check_size(
        &self,
        state: &serde_json::Value,
        max_bytes: usize,
    ) -> Result<(), ValidationError>;

    /// Register a schema for a state type
    fn register_schema(
        &self,
        schema_ref: SchemaRef,
        schema: serde_json::Value,
    ) -> Result<(), ValidationError>;
}
```

## Validation Pipeline

```text
Raw state from persistence
    |
    v
[1. Size check] -- exceeds limit --> Reject (TaintLabel::Rejected)
    |
    v
[2. Schema validation] -- fails schema --> Reject or Sanitize
    |
    v
[3. Pattern check] -- known malicious pattern --> Quarantine
    |
    v
[4. Taint labeling] --> ValidatedState with TaintLabel
```

### Stage 1: Size Check

- Estimate serialized size of the `Value` before parsing.
- Reject if size exceeds configured maximum (default 1 MiB per agent state).
- Purpose: prevent memory exhaustion from oversized state objects before any parsing.

### Stage 2: Schema Validation

- Validate against registered JSON Schema for the state type.
- Use `jsonschema` crate for Rust-speed validation (645x faster than legacy validators).
- Missing schema: treat as `TaintLabel::Suspicious` (pass with warning, not reject).

### Stage 3: Pattern Check

- Check for known malicious patterns (configurable pattern list).
- Initial patterns: excessive string length, deeply nested objects, known injection markers.
- Purpose: catch payloads that are schema-valid but semantically malicious.

### Stage 4: Taint Labeling

- `Clean` — passed all checks without modification.
- `Sanitized` — modified during validation (e.g., truncated fields, removed nested objects).
- `Suspicious` — passed validation but flagged (e.g., missing schema, unusual patterns).
- `Rejected` — failed validation; not forwarded to agent.

## AgentRepository Integration

The `StateValidator` integrates at the `AgentRepository::get_state()` boundary:

```rust
impl AgentRepository {
    pub async fn get_state(
        &self,
        agent_id: &str,
        validator: &dyn StateValidator,
    ) -> Result<Option<ValidatedState>, PersistenceError> {
        let raw_state = self.raw_get_state(agent_id).await?;
        match raw_state {
            Some(state) => {
                let validated = validator.validate(&state, &self.schema_ref(agent_id))?;
                Ok(Some(validated))
            }
            None => Ok(None),
        }
    }
}
```

## Error Contract

```rust
pub enum ValidationError {
    SizeExceeded { actual_bytes: usize, max_bytes: usize },
    SchemaViolation { path: String, message: String },
    MaliciousPattern { pattern: String, description: String },
    SchemaNotFound { schema_ref: String },
}
```

These map into the existing `PersistenceError` hierarchy for agent state operations.

## Behavioral Requirements

1. Size check MUST execute before schema validation — never parse oversized payloads.
2. Schema validation MUST use compiled schemas (not re-compile per validation).
3. Missing schemas MUST NOT cause hard failures — log a warning and label as `Suspicious`.
4. Validation overhead MUST be sub-millisecond for typical agent state sizes (<100 KiB).
5. Audit events MUST be emitted for `Sanitized`, `Suspicious`, and `Rejected` outcomes.

## Validation Requirements

- Valid state passes with `TaintLabel::Clean`.
- Oversized state rejected before schema validation.
- Schema-invalid state rejected with descriptive error.
- Missing schema produces `Suspicious` label, not failure.
- Known malicious pattern triggers quarantine.
- Audit events emitted for non-clean outcomes.

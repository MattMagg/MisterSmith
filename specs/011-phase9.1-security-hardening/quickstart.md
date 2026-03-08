# Quickstart: Phase 9.1 — Security Hardening

## Prerequisites

- Phase 1-8 workspace crates present and building
- Phase 9 `MessageEnvelope` additions (`plane`, `stream_class`) stable
- Active feature directory: `specs/011-phase9.1-security-hardening/`
- NATS server running with JetStream enabled (for Auth Callout and account isolation tests)
- PostgreSQL available (for StateValidator integration tests)

## Planned Build Flow After Implementation

```bash
# 1. Message signing and verification
cargo test -p mister-smith-security -- signer

# 2. State validation
cargo test -p mister-smith-security -- validator

# 3. Auth Callout service (env-gated with NATS_URL)
NATS_URL=nats://localhost:4222 cargo test -p mister-smith-security -- auth_callout --ignored

# 4. AgentSandbox and I/O firewall (env-gated with NATS_URL)
NATS_URL=nats://localhost:4222 cargo test -p mister-smith-security -- sandbox --ignored

# 5. Quarantine actors
cargo test -p mister-smith-security -- quarantine
cargo test -p mister-smith-agents -- quarantine

# 6. Delegation chain validation
cargo test -p mister-smith-security -- delegation

# 7. Transport backward compatibility
cargo test -p mister-smith-transport -- envelope signature nonce

# 8. Full workspace
cargo clippy --workspace -- -D warnings
```

## Usage Sketch

### Message Signing

```rust
use mister_smith_security::MessageSigner;

let signer = HmacSigner::new(signing_key);

// Sign before publish
let signature = signer.sign(&envelope)?;
envelope.signature = Some(signature);
envelope.nonce = Some(signer.generate_nonce());

// Verify on receive
if !signer.verify(&envelope, &envelope.signature.unwrap())? {
    return Err(SecurityError::InvalidSignature);
}
if signer.is_replay(&envelope.nonce.unwrap()) {
    return Err(SecurityError::ReplayDetected);
}
signer.record_nonce(&envelope.nonce.unwrap());
```

### State Validation

```rust
use mister_smith_security::StateValidator;

let validator = JsonSchemaValidator::new();
validator.register_schema("agent-state-v1", schema)?;

// Validates size, schema, and patterns before returning to agent
let validated = validator.validate(&raw_state, &"agent-state-v1".into())?;
match validated.taint_label {
    TaintLabel::Clean => { /* safe to use */ }
    TaintLabel::Suspicious => { /* use with monitoring */ }
    _ => { /* should not reach here — rejected earlier */ }
}
```

### Agent Sandbox

```rust
use mister_smith_agents::sandbox::{AgentClass, SandboxCredentials};

// Classify and spawn agent
let class = AgentClass::Ephemeral;
let credentials = sandbox.create_credentials(agent_id, class)?;

// ... agent operates with restricted permissions ...

// Auto-cleanup on completion
sandbox.cleanup(agent_id)?; // removes credentials, state, NATS subjects
```

## Security Verification Scenarios

### Scenario 1: Message Forgery Prevention

1. Agent A sends a signed message to Agent B.
2. An attacker modifies the message content in transit.
3. Agent B verifies the signature — verification fails.
4. Agent B rejects the message and emits an audit event.

### Scenario 2: Replay Attack Prevention

1. Agent A sends a signed message with nonce "abc123".
2. An attacker captures and replays the same message.
3. Agent B detects nonce "abc123" has been seen before.
4. Agent B rejects the replay and emits an audit event.

### Scenario 3: Infectious Jailbreak Prevention

1. A malicious payload is stored in shared state (JetStream KV).
2. Agent C retrieves the state through `AgentRepository::get_state()`.
3. The `StateValidator` detects the malicious pattern.
4. The state is quarantined — Agent C receives an error, not the payload.
5. An audit event records the quarantine decision.

### Scenario 4: Ephemeral Agent Isolation

1. A persistent Orchestrator agent spawns an ephemeral Worker agent.
2. The Worker receives credentials scoped to the ephemeral NATS account.
3. The Worker attempts to subscribe to the Orchestrator's state subjects.
4. NATS account isolation prevents the subscription — the Worker never sees the data.
5. Task results from Worker to Orchestrator pass through a quarantine actor.

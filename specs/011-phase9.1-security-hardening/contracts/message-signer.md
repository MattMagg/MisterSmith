# Contract: Message Signer

## Overview

The `MessageSigner` contract defines HMAC-SHA256 message signing and verification for
`MessageEnvelope` contents. It prevents inter-agent message forgery (97% ASR without signing)
and replay attacks.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/04-security-and-trust.md` | Inter-agent hijacking at 97% ASR drives the signing requirement |
| `spec/data-management/agent-orchestration.md` | CRITICAL GAP flag on MessageEnvelope security |
| `spec/security/` | Existing SecurityLayer patterns extended by message signing |

## Public API

```rust
pub trait MessageSigner: Send + Sync {
    /// Compute HMAC-SHA256 signature over envelope contents (excluding signature field)
    fn sign(&self, envelope: &MessageEnvelope) -> Result<String, SecurityError>;

    /// Verify signature matches envelope contents; accepts active key and grace keys
    fn verify(&self, envelope: &MessageEnvelope, signature: &str) -> Result<bool, SecurityError>;

    /// Generate a monotonic nonce for replay prevention
    fn generate_nonce(&self) -> String;

    /// Check if a nonce has been seen before (replay detection)
    fn is_replay(&self, nonce: &str) -> bool;

    /// Record a nonce as seen
    fn record_nonce(&self, nonce: &str);

    /// Rotate the signing key; current active key moves to grace list
    fn rotate_key(&self, new_key: HmacKey) -> Result<(), SecurityError>;
}
```

## MessageEnvelope Fields

```rust
pub struct MessageEnvelope {
    // ... existing fields ...
    // Phase 9 additions:
    pub plane: Option<MessagePlane>,
    pub stream_class: Option<StreamClass>,
    // Phase 9.1 additions:
    pub signature: Option<String>,
    pub nonce: Option<String>,
    pub capability_token: Option<String>,
}
```

All Phase 9.1 fields use `Option<T>` with `#[serde(default)]` for backward compatibility.

## Signing Algorithm

1. Serialize the envelope contents (excluding `signature` field) to a canonical byte
   representation (deterministic JSON serialization with sorted keys).
2. Compute HMAC-SHA256 over the canonical bytes using the active signing key.
3. Hex-encode the resulting MAC as the signature string.

## Verification Algorithm

1. Extract the `signature` field from the envelope.
2. If `signature` is `None` and signing is required, reject.
3. Re-serialize the envelope contents (excluding `signature`) to canonical bytes.
4. Verify the signature against the active key first, then each grace key.
5. If any key produces a matching MAC, accept. Otherwise, reject.

## Nonce Management

- Nonces must be monotonically increasing (UUID v7 or timestamp + counter).
- A bounded `HashSet` tracks recently-seen nonces (configurable window, default 10,000).
- When the window is full, the oldest nonces are evicted (FIFO).
- Messages with previously-seen nonces are rejected as replay attacks.

## Key Rotation

- `rotate_key(new_key)` atomically moves the current active key to the grace list and installs
  the new key.
- Grace keys are accepted for verification for a configurable TTL (default 300 seconds).
- After the TTL expires, grace keys are removed.
- No messages should be lost during rotation — the grace period covers in-flight messages.

## Error Contract

```rust
pub enum SigningError {
    InvalidSignature,
    ReplayDetected { nonce: String },
    SigningFailed(String),
    KeyRotationFailed(String),
    MissingSignature,
}
```

These map into the existing `SecurityError` hierarchy in `mister-smith-core`.

## Behavioral Requirements

1. Signing must be fast enough for per-message use (HMAC-SHA256 is ~1 GiB/sec on modern CPUs).
2. Key rotation must be atomic — no window where no key is active.
3. Nonce replay detection must be thread-safe (concurrent message processing).
4. Backward compatibility: messages without signatures are accepted when signing is optional,
   rejected when signing is required.

## Validation Requirements

- Sign/verify round-trip succeeds with matching keys.
- Forged message (modified content after signing) is rejected.
- Replayed message (same nonce) is rejected.
- Key rotation grace period accepts both old and new keys.
- Nonce window eviction works correctly under high throughput.
- Pre-Phase-9.1 messages (no signature fields) deserialize without error.

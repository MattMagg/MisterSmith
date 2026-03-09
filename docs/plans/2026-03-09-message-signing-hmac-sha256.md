# Message Signing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Phase 9.1.1 message signing to transport and security so inter-agent messages carry HMAC-SHA256 signatures with nonce replay protection and key-rotation grace handling.

**Architecture:** Extend the transport envelope contract with backward-compatible security fields, add a deterministic canonical-signing implementation in `mister-smith-security`, and wire that signer into `SecureTransport<T>` so publish/request paths sign while receive paths verify and audit failures. Keep the implementation inside existing crate boundaries and use typed `SecurityError` variants for invalid signatures, missing signatures, replay detection, and rotation failures.

**Tech Stack:** Rust workspace, `ring` HMAC, `serde_json`, `mister-smith-transport`, `mister-smith-security`, existing audit logger

---

### Task 1: Envelope Contract Extension

**Files:**
- Modify: `crates/mister-smith-transport/src/envelope.rs`
- Modify: `crates/mister-smith-transport/proto/common.proto`
- Modify: `crates/mister-smith-grpc/src/proto.rs`

**Step 1: Write the failing test**

Add transport tests proving:
- pre-9.1 JSON without `signature`, `nonce`, or `capability_token` still deserializes
- new fields round-trip through JSON and MessagePack
- omitted optional fields do not serialize as `null`

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-transport envelope signature nonce`
Expected: FAIL because the new fields do not exist yet.

**Step 3: Write minimal implementation**

Add the three `Option<String>` fields with `#[serde(default)]` and matching builder/default behavior. Update protobuf message definitions so shared message contracts stay aligned with transport.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-transport envelope signature nonce`
Expected: PASS

### Task 2: Typed Security Errors And Signer Surface

**Files:**
- Modify: `crates/mister-smith-core/src/error.rs`
- Modify: `crates/mister-smith-security/src/lib.rs`
- Modify: `crates/mister-smith-security/src/config.rs`
- Create: `crates/mister-smith-security/src/message_signer.rs`

**Step 1: Write the failing test**

Add signer tests that assert the public API exists and that invalid signature, missing signature, replay, and rotation failures map to typed `SecurityError` variants.

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-security signer`
Expected: FAIL because the signer module and error variants do not exist.

**Step 3: Write minimal implementation**

Introduce:
- `MessageSigner` trait
- signer config and key types
- new `SecurityError` variants required by the signer contract
- public exports from `mister-smith-security`

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-security signer`
Expected: partial PASS or next failure moving into implementation details

### Task 3: HMAC Signing, Canonicalization, Nonces, And Rotation

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/mister-smith-security/Cargo.toml`
- Modify: `crates/mister-smith-security/src/message_signer.rs`
- Test: `crates/mister-smith-security/tests/signer_tests.rs`

**Step 1: Write the failing test**

Add tests for:
- sign/verify round-trip
- forged payload rejection
- replay rejection
- nonce window eviction
- active plus grace-key verification during rotation
- grace-key expiry rejection after TTL

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-security signer_tests -- --nocapture`
Expected: FAIL because the concrete signer behavior is missing.

**Step 3: Write minimal implementation**

Implement a concrete HMAC signer using `ring` with:
- deterministic JSON canonicalization using sorted keys and sorted headers
- hex-encoded HMAC-SHA256 signatures
- monotonic nonce generation using timestamp plus atomic counter
- bounded replay window with FIFO eviction
- atomic key rotation with grace-key TTL cleanup

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-security signer_tests -- --nocapture`
Expected: PASS

### Task 4: SecureTransport Integration

**Files:**
- Modify: `crates/mister-smith-security/src/middleware/nats_mw.rs`
- Modify: `crates/mister-smith-security/tests/signer_tests.rs`
- Test: `crates/mister-smith-security/tests/audit_tests.rs`

**Step 1: Write the failing test**

Add integration tests covering:
- publish signs outgoing envelopes and injects nonce/signature
- subscribe rejects forged or replayed envelopes
- request signs outbound request and verifies inbound response
- invalid/replayed traffic records audit events

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-security secure_transport signer`
Expected: FAIL because `SecureTransport<T>` only enforces RBAC today.

**Step 3: Write minimal implementation**

Extend `SecureTransport<T>` with optional signer/audit behavior:
- sign on `publish` and `request`
- wrap subscriptions and request responses with verification
- emit audit events on invalid signature, missing signature, and replay detection
- keep RBAC behavior intact

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-security secure_transport signer`
Expected: PASS

### Task 5: Workspace Validation

**Files:**
- Modify: `Linear workpad comment`

**Step 1: Run targeted validation**

Run:
- `cargo test -p mister-smith-transport envelope signature nonce`
- `cargo test -p mister-smith-security signer`

Expected: PASS

**Step 2: Run cross-crate validation**

Run: `cargo build --workspace`
Expected: PASS

**Step 3: Record evidence**

Update the Linear workpad with completed checklist items, exact validation commands, and any remaining blockers or follow-up work.

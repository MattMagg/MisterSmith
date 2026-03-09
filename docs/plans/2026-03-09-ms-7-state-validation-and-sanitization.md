# State Validation And Sanitization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add schema-driven state validation between persistence retrieval and agent consumption so no raw state crosses the boundary unchecked.

**Architecture:** Introduce a new `StateValidator` contract in `mister-smith-security` with a concrete JSON Schema validator and explicit taint labeling. Change `AgentRepository::get_state()` to return validated state only, mapping rejected validation outcomes into persistence errors while recording security audit events for non-clean outcomes.

**Tech Stack:** Rust 1.88, `serde_json`, `jsonschema`, existing `AuditLogger`/`SecurityAuditEvent`, existing `PersistenceError`

---

### Task 1: Add failing validator tests

**Files:**
- Create: `crates/mister-smith-security/tests/validator_tests.rs`
- Read for patterns: `crates/mister-smith-security/tests/audit_tests.rs`
- Read for patterns: `crates/mister-smith-security/tests/jwt_tests.rs`

**Step 1: Write the failing tests**

- Add tests for:
  - valid state returns `TaintLabel::Clean`
  - oversized state fails before schema validation
  - schema mismatch is rejected
  - malicious string pattern is rejected
  - missing schema is labeled `Suspicious`

**Step 2: Run the test to verify it fails**

Run: `cargo test -p mister-smith-security --test validator_tests`

Expected: FAIL with missing `state_validator` module/types.

### Task 2: Implement the validator contract and concrete JSON Schema validator

**Files:**
- Create: `crates/mister-smith-security/src/state_validator.rs`
- Modify: `crates/mister-smith-security/src/lib.rs`
- Modify: `crates/mister-smith-security/Cargo.toml`

**Step 1: Add the public API**

- Define `TaintLabel`, `ValidatedState`, and a typed validation error enum.
- Define a `StateValidator` trait with `validate()` and `check_size()`.
- Re-export the new types from the crate root.

**Step 2: Add the concrete implementation**

- Implement a JSON Schema-backed validator with:
  - schema registration by state type
  - compiled schema caching
  - size enforcement before schema validation
  - recursive malicious-pattern detection
  - suspicious labeling for missing schemas

**Step 3: Run the validator tests**

Run: `cargo test -p mister-smith-security --test validator_tests`

Expected: PASS

### Task 3: Add repository integration tests first

**Files:**
- Modify: `crates/mister-smith-persistence/src/repository/agent.rs`
- Read for patterns: `crates/mister-smith-app/src/bridges.rs`
- Read for patterns: `crates/mister-smith-security/src/audit/mod.rs`

**Step 1: Add failing unit tests in `agent.rs`**

- Add focused tests covering:
  - `get_state()` returns validated data, not raw JSON
  - rejected validation maps to `PersistenceError`
  - sanitized/suspicious/rejected outcomes write audit events with useful details

**Step 2: Run the test target to verify it fails**

Run: `cargo test -p mister-smith-persistence repository::agent`

Expected: FAIL because `get_state()` has the old signature/behavior.

### Task 4: Integrate validation into the repository boundary

**Files:**
- Modify: `crates/mister-smith-persistence/src/repository/agent.rs`

**Step 1: Change the repository boundary**

- Update `get_state()` to accept a validator and audit logger.
- Keep the raw hybrid-manager read private to the repository layer.
- Return `ValidatedState` so taint metadata remains available to callers.

**Step 2: Record audit events**

- Emit `SecurityAuditEvent`s for:
  - `Sanitized`
  - `Suspicious`
  - `Rejected`

**Step 3: Re-run repository tests**

Run: `cargo test -p mister-smith-persistence repository::agent`

Expected: PASS

### Task 5: Validate the full scoped change

**Files:**
- Modify: `crates/mister-smith-security/tests/validator_tests.rs` if assertions need tightening after integration

**Step 1: Run crate tests**

Run: `cargo test -p mister-smith-security`
Run: `cargo test -p mister-smith-persistence`

Expected: PASS

**Step 2: Run workspace compile validation**

Run: `cargo build --workspace`

Expected: PASS

**Step 3: Update workpad**

- Check off completed items.
- Record validation evidence.
- Note any follow-up work if sanitization policy needs to evolve beyond the initial rule set.

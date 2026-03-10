# Tasks: Phase 9.1 — Security Hardening

**Input**: Design documents from `/specs/011-phase9.1-security-hardening/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Included. Phase 9.1 requires message signing/verification tests, Auth Callout
integration tests, state validation tests, sandbox isolation tests, quarantine actor tests,
and deploy artifact verification.

**Organization**: Tasks are grouped by subphase `9.1.1` through `9.1.6`, mapped to user
stories US1-US6 and audit findings F1-F7.

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- **Security crate**: `crates/mister-smith-security/src/`
- **Security tests**: `crates/mister-smith-security/tests/`
- **Transport**: `crates/mister-smith-transport/src/`
- **Persistence**: `crates/mister-smith-persistence/src/`
- **Agents**: `crates/mister-smith-agents/src/`
- **Deploy**: `deploy/`

---

## Subphase 9.1.1 — Message Signing (User Story 1, Finding F1, Priority: P1)

**Goal**: HMAC-SHA256 message signing with nonce-based replay prevention and key rotation.

- [x] S001 [US1] Add `signature: Option<String>`, `nonce: Option<String>`, and
  `capability_token: Option<String>` to `MessageEnvelope` in
  `crates/mister-smith-transport/src/envelope.rs` with `#[serde(default)]`.
- [x] S002 [US1] Create `crates/mister-smith-security/src/message_signer.rs` with
  `MessageSigner` trait: `sign()`, `verify()`, `generate_nonce()`, active key, grace keys
  for rotation, bounded nonce window for replay detection.
- [x] S003 [P] [US1] Implement HMAC-SHA256 signing using `ring` crate: compute signature
  over serialized envelope contents (excluding signature field), produce hex-encoded output.
- [x] S004 [P] [US1] Implement monotonic nonce generation (UUID v7 or timestamp + counter)
  and replay detection with bounded `HashSet` (configurable window size, FIFO eviction).
- [x] S005 [US1] Implement key rotation: `rotate_key()` moves active key to grace list,
  `verify()` accepts both active and grace keys, grace keys expire after configurable TTL.
- [x] S006 [US1] Integrate `MessageSigner` into `SecureTransport<T>` in
  `crates/mister-smith-security/src/middleware/nats_mw.rs` — sign on publish, verify on
  receive, reject invalid/replayed messages with audit event.
- [x] S007 [US1] Add signing tests in `crates/mister-smith-security/tests/signer_tests.rs`:
  sign/verify round-trip, forged message rejection, replay rejection, key rotation grace
  period, nonce window overflow/eviction, MessageEnvelope backward compatibility (missing
  signature fields).

**Checkpoint**: Inter-agent messages carry verifiable HMAC-SHA256 signatures with replay
prevention.

---

## Subphase 9.1.2 — State Validation (User Story 3, Finding F3, Priority: P1)

**Goal**: Pluggable state sanitization between persistence retrieval and agent consumption.

- [x] S008 [US3] Create `crates/mister-smith-security/src/state_validator.rs` with
  `StateValidator` trait: `validate()` and `check_size()` methods, `ValidatedState` result
  type, `TaintLabel` enum (`Clean`, `Sanitized`, `Suspicious`, `Rejected`).
- [x] S009 [US3] Implement JSON Schema-based `StateValidator` using `jsonschema` crate:
  register schemas by type, validate state against registered schema, enforce size limits
  before schema validation.
- [x] S010 [US3] Integrate `StateValidator` into `AgentRepository::get_state()` in
  `crates/mister-smith-persistence/src/repository/agent.rs` — validate before returning
  state to caller, emit audit events for rejected/sanitized state.
- [x] S011 [US3] Add validation tests in `crates/mister-smith-security/tests/validator_tests.rs`:
  valid state passes, oversized state rejected, schema mismatch rejected, malicious pattern
  detected, taint labels correctly assigned.

**Checkpoint**: No raw persistence retrieval reaches agent working context without validation.

---

## Subphase 9.1.3 — Auth Callout Service (User Story 2, Finding F4, Priority: P1)

**Goal**: Dynamic per-connection capability scoping via NATS Auth Callout protocol.

- [x] S012 [US2] Create `crates/mister-smith-security/src/auth_callout.rs` with
  `AuthCalloutHandler`: trust store, signing key, default permissions, JWT generation
  from `TrustProfile` and `PermissionTier`.
- [x] S013 [US2] Implement NATS Auth Callout protocol: subscribe to `$SYS.REQ.USER.AUTH`,
  receive connection requests, look up agent trust profile, generate scoped JWT, respond
  with authorization result.
- [x] S014 [US2] Implement trust-to-permission mapping: `Full` (score >= 0.9), `Standard`
  (>= 0.5), `Restricted` (>= 0.2), `Quarantined` (< 0.2) with corresponding subject
  permissions and JWT TTL.
- [x] S015 [US2] Implement minimal-permission fallback when trust store lookup fails or
  service is degraded — default to `Quarantined` tier, not full access.
- [x] S016 [US2] Add Auth Callout tests in
  `crates/mister-smith-security/tests/auth_callout_tests.rs` (env-gated with `NATS_URL`):
  dynamic JWT generation, trust tier mapping, permission narrowing on trust degradation,
  fallback behavior, JWT TTL enforcement.

**Checkpoint**: Auth Callout service dynamically scopes agent permissions based on trust.

---

## Subphase 9.1.4 — AgentSandbox (User Story 4, Finding F2, Priority: P2)

**Goal**: Persistent/ephemeral agent separation with NATS account isolation.

- [x] S017 [US4] Create `crates/mister-smith-security/src/sandbox.rs` with `AgentClass`
  enum (`Persistent`, `Ephemeral`), `SandboxCredentials`, `IOFirewall`, `CrossingRule`.
- [x] S018 [US4] Implement NATS account-based isolation: persistent agents assigned to
  persistent account, ephemeral agents to ephemeral account, accounts configured with
  non-overlapping subject spaces.
- [x] S019 [US4] Create `crates/mister-smith-agents/src/sandbox.rs` with agent class
  assignment logic: classify agents based on lifecycle, spawn with appropriate
  `SandboxCredentials`, auto-cleanup ephemeral agent credentials on completion/timeout.
- [x] S020 [US4] Implement `IOFirewall` with `CrossingRule` enforcement: validate cross-
  boundary communication against explicit rules, route data through quarantine when
  `requires_quarantine = true`, block unauthorized crossings.
- [x] S021 [US4] Add sandbox tests in `crates/mister-smith-security/tests/sandbox_tests.rs`:
  account isolation (ephemeral cannot subscribe to persistent subjects), credential
  lifecycle (creation, expiration, cleanup), I/O firewall rule enforcement.

**Checkpoint**: Ephemeral agents cannot access persistent agent resources.

---

## Subphase 9.1.5 — Quarantine Actors (User Story 5, Finding F5, Priority: P2)

**Goal**: Cross-boundary data inspection with quarantine for infectious content.

- [x] S022 [US5] Create `crates/mister-smith-security/src/quarantine.rs` with
  `QuarantineAction` enum (`Pass`, `Sanitize`, `Reject`, `Quarantine`) and quarantine
  inspection logic.
- [x] S023 [US5] Create `crates/mister-smith-agents/src/quarantine.rs` with quarantine
  actor implementation: receive cross-boundary data, inspect using `StateValidator` and
  pattern matching, emit `QuarantineAction`, forward or reject data, log to audit.
- [x] S024 [US5] Implement COWPOX-inspired edge monitoring pattern: deploy quarantine actors
  at agent boundary crossings (persistent <-> ephemeral, agent <-> shared state), inspect
  all data transfers through these actors.
- [x] S025 [US5] Add quarantine tests in
  `crates/mister-smith-security/tests/quarantine_tests.rs`: clean data passes with
  sub-millisecond overhead, malicious payload detected and quarantined, audit event emitted
  for quarantined data, concurrent transfers handled without deadlock.

**Checkpoint**: Cross-boundary data transfers are inspected with sub-millisecond overhead for
clean data.

---

## Subphase 9.1.6 — CVE Mitigation + Delegation Chain (User Story 6, Findings F6/F7, Priority: P2)

**Goal**: Infrastructure hardening and delegation chain resolution.

- [x] S026 [US6] Update `deploy/docker-compose.yml` and `deploy/kubernetes/` manifests to
  pin NATS server image to >= v2.11.1. Add version check to deployment documentation.
- [x] S027 [P] [US6] Create a permission audit script that scans NATS authorization
  configurations for wildcard `>` and `$JS.>` permissions and reports violations.
- [x] S028 [P] [US6] Validate `AgentClaims.delegation_chain` in
  `crates/mister-smith-security/src/jwt/claims.rs`: check non-empty entries, enforce
  maximum depth (configurable, default 5), detect and reject circular references.
- [x] S029 [US6] Propagate `delegation_chain` across agent boundaries: when agent A spawns
  agent B, B's delegation chain includes A's identity. Validate chain on JWT issuance.
- [x] S030 [US6] Add delegation chain tests in
  `crates/mister-smith-security/tests/`: valid chain propagation, max depth rejection,
  circular reference detection, empty entry rejection.

**Checkpoint**: Deploy artifacts enforce NATS >= v2.11.1, delegation chain is validated.

---

## Verification & Readiness

- [x] S031 [P] Run security crate verification:
  `cargo test -p mister-smith-security` and `cargo clippy -p mister-smith-security`.
- [x] S032 [P] Run transport crate verification:
  `cargo test -p mister-smith-transport` — verify MessageEnvelope backward compatibility.
- [x] S033 Run persistence integration:
  `cargo test -p mister-smith-persistence` — verify StateValidator integration.
- [x] S034 Run agent integration:
  `cargo test -p mister-smith-agents` — verify sandbox and quarantine actors.
- [x] S035 Run workspace hygiene: `cargo clippy --workspace -- -D warnings`.
- [x] S036 Update `CLAUDE.md` implementation status and deploy documentation.

---

## Dependencies & Execution Order

### Subphase Dependencies

- **9.1.1 (Message Signing)**: Depends on Phase 9 MessageEnvelope `plane`/`stream_class` stable.
- **9.1.2 (State Validation)**: Depends on 9.1.1 (field layout).
- **9.1.3 (Auth Callout)**: Depends on 9.1.1 (for signed JWTs).
- **9.1.4 (AgentSandbox)**: Depends on 9.1.3 (for dynamic credentials).
- **9.1.5 (Quarantine)**: Depends on 9.1.4 (for cross-boundary context).
- **9.1.6 (CVE/Delegation)**: Depends on 9.1.1 (field layout). Can parallel with 9.1.2+.
- **Verification**: Depends on all subphases.

### Parallel Opportunities

- `S003` and `S004` can proceed in parallel (different signing vs nonce logic).
- `S027` and `S028` can proceed in parallel (different files).
- `S031` and `S032` can proceed in parallel (different crates).
- 9.1.6 can proceed in parallel with 9.1.2-9.1.5 (independent scope).

### Estimated Task Count

- Implementation tasks: 30 (S001-S030)
- Verification tasks: 6 (S031-S036)
- Total: 36

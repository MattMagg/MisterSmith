# Tasks: Agent-Boundary Security Hardening

**Input**: Design documents from `/specs/024-agent-boundary-security-hardening/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Draft Status**: This task list is scaffolding only.

- packet `024` is being scaffolded before earlier packets are fully complete
- claims are based on current repo truth and current dossiers
- before implementation, this task list MUST be revised against the then-current
  `docs/current-state.md`, `docs/direction.md`, and any newly landed earlier packet artifacts
- if earlier packet work changes reused contracts, packet `024` wins no authority over those
  contracts until revised

**Tests**: Included. Future implementation should use targeted Rust tests for capability-boundary,
quarantine, validator, sandbox, delegation, and auth-callout seams, plus bounded doc hygiene.

**Organization**: Tasks are grouped by a blocking refresh gate first, then by the three bounded
stories frozen in the packet.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only after all blocking tasks in the current section are complete
  and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Every task includes exact file paths

---

## Phase 0: Mandatory Refresh Gate (Blocking)

**Purpose**: Reconcile packet `024` against landed truth before any implementation work starts.

- [ ] T001 Re-read `docs/current-state.md`, `docs/direction.md`,
      `docs/packet-prep/024-agent-boundary-security-hardening.md`, and newly landed earlier packet
      artifacts, then record any contract drift in
      `specs/024-agent-boundary-security-hardening/spec.md`
- [ ] T002 Refresh draft packet artifacts in
      `specs/024-agent-boundary-security-hardening/plan.md`,
      `specs/024-agent-boundary-security-hardening/research.md`,
      `specs/024-agent-boundary-security-hardening/data-model.md`,
      `specs/024-agent-boundary-security-hardening/contracts/`, and
      `specs/024-agent-boundary-security-hardening/tasks.md` if upstream packet work changed reused
      seams

**Checkpoint**: Packet `024` is refreshed against current truth and safe to implement.

---

## Phase 1: Shared Boundary Contract Freeze (Blocking Prerequisites)

**Purpose**: Keep the packet contracts authoritative before code changes start.

- [ ] T003 Refresh the capability boundary contract in
      `specs/024-agent-boundary-security-hardening/contracts/capability-boundary-contract.md`
- [ ] T004 Refresh the quarantine and schema enforcement contract in
      `specs/024-agent-boundary-security-hardening/contracts/quarantine-and-schema-enforcement.md`
- [ ] T005 Refresh the identity and sandbox boundary contract in
      `specs/024-agent-boundary-security-hardening/contracts/identity-and-sandbox-boundary.md`

**Checkpoint**: The shared contracts match the current repo seams and block scope drift.

---

## User Story 1 - Least-Privilege Capability Boundaries (Priority: P1)

**Goal**: Keep ToolBus and MCP capability boundaries least-privilege, action-bound, and explicit
about discover-versus-execute separation.

**Independent Test**: prove discovery remains bounded and execution rejects mismatched or revoked
delegated action bindings before handler execution.

### Tests For User Story 1

- [ ] T006 [P] [US1] Extend ToolBus capability-boundary coverage in
      `crates/mister-smith-agents/tests/tool_bus_tests.rs`
- [ ] T007 [P] [US1] Extend MCP discovery and action-bound invocation coverage in
      `crates/mister-smith-mcp/src/server.rs` and
      `crates/mister-smith-mcp/src/compatibility.rs`

### Implementation For User Story 1

- [ ] T008 [P] [US1] Reconcile shared capability descriptor and delegated-action use in
      `crates/mister-smith-agents/src/tool_bus.rs`
- [ ] T009 [P] [US1] Tighten MCP boundary action generation and tool metadata publication in
      `crates/mister-smith-mcp/src/server.rs`
- [ ] T010 [US1] Preserve bounded discovery and exact action enforcement in
      `crates/mister-smith-mcp/src/compatibility.rs`

**Checkpoint**: Discovery and execution remain separate across ToolBus and MCP.

---

## User Story 2 - Quarantine And Schema Enforcement (Priority: P1)

**Goal**: Keep cross-boundary payloads and shared-state reads deterministically validated before
agent consumption.

**Independent Test**: prove clean payloads pass, sanitized payloads are marked, suspicious payloads
stay monitored, and rejected or quarantined payloads do not reach agent context.

### Tests For User Story 2

- [ ] T011 [P] [US2] Extend validator pipeline coverage in
      `crates/mister-smith-security/tests/validator_tests.rs`
- [ ] T012 [P] [US2] Extend quarantine and sandbox crossing coverage in
      `crates/mister-smith-security/tests/quarantine_tests.rs`,
      `crates/mister-smith-security/tests/sandbox_tests.rs`,
      `crates/mister-smith-agents/tests/quarantine_tests.rs`, and
      `crates/mister-smith-agents/tests/sandbox_tests.rs`
- [ ] T013 [P] [US2] Extend shared-state mediation coverage in
      `crates/mister-smith-persistence/tests/repository_tests.rs`

### Implementation For User Story 2

- [ ] T014 [P] [US2] Reconcile quarantine decision mapping and audit reasons in
      `crates/mister-smith-security/src/quarantine.rs`
- [ ] T015 [P] [US2] Tighten state validation, taint labels, and malicious-pattern handling in
      `crates/mister-smith-security/src/state_validator.rs`
- [ ] T016 [P] [US2] Preserve required quarantine enforcement for protected crossings in
      `crates/mister-smith-security/src/sandbox.rs` and
      `crates/mister-smith-agents/src/sandbox.rs`
- [ ] T017 [US2] Reconcile validated shared-state mediation in
      `crates/mister-smith-persistence/src/repository/agent.rs`

**Checkpoint**: Cross-boundary content and shared-state reads are deterministically mediated before
agent use.

---

## User Story 3 - Identity, Auth Callout, And Delegation Continuity (Priority: P2)

**Goal**: Keep least-privilege identity and delegation rules bounded without widening into a new
IAM program or breaking packet `016` continuity truth.

**Independent Test**: prove auth-callout fallback stays minimal, delegated authority remains
action-bound, sandbox classes stay isolated, and packet `016` continuity assumptions still hold.

### Tests For User Story 3

- [ ] T018 [P] [US3] Extend auth-callout permission-tier and fallback coverage in
      `crates/mister-smith-security/tests/auth_callout_tests.rs`
- [ ] T019 [P] [US3] Extend delegated-action binding, expiry, and revocation coverage in
      `crates/mister-smith-security/tests/delegation_tests.rs`
- [ ] T020 [P] [US3] Extend identity and sandbox lifecycle isolation coverage in
      `crates/mister-smith-security/tests/sandbox_tests.rs` and
      `crates/mister-smith-agents/tests/sandbox_tests.rs`

### Implementation For User Story 3

- [ ] T021 [P] [US3] Tighten least-privilege auth-callout issuance and quarantined fallback in
      `crates/mister-smith-security/src/auth_callout.rs`
- [ ] T022 [P] [US3] Tighten external envelope, action binding, and revocation continuity in
      `crates/mister-smith-security/src/delegation.rs`
- [ ] T023 [US3] Reconcile packet `016` continuity assumptions in
      `specs/024-agent-boundary-security-hardening/spec.md`,
      `specs/024-agent-boundary-security-hardening/contracts/identity-and-sandbox-boundary.md`,
      and `specs/024-agent-boundary-security-hardening/analyze.md` before landing code changes

**Checkpoint**: Least-privilege identity posture stays bounded and packet `016` continuity remains
honest.

---

## Final Validation And Evidence

- [ ] T024 Run `cargo test -p mister-smith-security`
- [ ] T025 Run `cargo test -p mister-smith-agents --test tool_bus_tests`
- [ ] T026 Run `cargo test -p mister-smith-agents --test quarantine_tests`
- [ ] T027 Run `cargo test -p mister-smith-mcp`
- [ ] T028 Run `cargo test -p mister-smith-persistence`
- [ ] T029 Run `cargo build --workspace`
- [ ] T030 Run `git diff --check`
- [ ] T031 Run `npx markdownlint-cli2 "specs/024-agent-boundary-security-hardening/**/*.md" --config .markdownlint.json`

## Parallel Staging Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-mcp/src/server.rs`
- `crates/mister-smith-mcp/src/compatibility.rs`
- `crates/mister-smith-security/src/delegation.rs`
- `crates/mister-smith-security/src/auth_callout.rs`
- `crates/mister-smith-security/src/quarantine.rs`
- `crates/mister-smith-security/src/state_validator.rs`
- `crates/mister-smith-security/src/sandbox.rs`
- `crates/mister-smith-persistence/src/repository/agent.rs`
- `specs/024-agent-boundary-security-hardening/contracts/`

Only one active lane may own a choke-point path at a time.

## Explicitly Out Of Scope For This Packet

- generic IAM rollout
- SPIFFE implementation
- broader interoperability design
- compliance expansion
- new live rejection proof for delegated HTTP task ingress
- broader runtime-truth or operator-console redesign work

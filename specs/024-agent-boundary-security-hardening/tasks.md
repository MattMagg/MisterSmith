# Tasks: Agent-Boundary Security Hardening

**Input**: Design documents from `/specs/024-agent-boundary-security-hardening/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. Use targeted Rust tests for capability-boundary, quarantine, validator,
sandbox, delegation, and auth-callout seams, plus bounded doc hygiene.

**Organization**: Tasks are grouped by the packet-authority gate first, then by the three bounded
stories frozen in the packet.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only after all blocking tasks in the current section are complete
  and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Every task includes exact file paths

---

## Phase 0: Checklist Completion And Packet Authority (Blocking)

**Purpose**: Make packet `024` implementation-ready before any code changes start.

- [x] T001 Revise `specs/024-agent-boundary-security-hardening/spec.md` and
      `specs/024-agent-boundary-security-hardening/plan.md` to current `main` truth, replacing
      obsolete prep references with live repo authority docs and current code seams
- [x] T002 Revise `specs/024-agent-boundary-security-hardening/research.md`,
      `specs/024-agent-boundary-security-hardening/data-model.md`,
      `specs/024-agent-boundary-security-hardening/quickstart.md`, and
      `specs/024-agent-boundary-security-hardening/analyze.md` to describe the actual open
      hardening gaps on `main`
- [x] T003 Revise `specs/024-agent-boundary-security-hardening/contracts/capability-boundary-contract.md`,
      `specs/024-agent-boundary-security-hardening/contracts/quarantine-and-schema-enforcement.md`,
      and `specs/024-agent-boundary-security-hardening/contracts/identity-and-sandbox-boundary.md`
      so the contracts match current `main`
- [x] T004 Rewrite `specs/024-agent-boundary-security-hardening/tasks.md` so the packet authority
      gate is checklist-first, followed by capability-boundary, quarantine-evidence, and
      identity-fallback hardening
- [x] T005 Audit `specs/024-agent-boundary-security-hardening/checklists/requirements.md` against
      the revised packet docs and mark every checklist item `[X]` only when the revised docs
      satisfy it

**Checkpoint**: Packet `024` is implementation-ready and the checklist is fully complete.

---

## User Story 1 - Least-Privilege Capability Boundaries (Priority: P1)

**Goal**: Keep ToolBus and MCP capability boundaries least-privilege, action-bound, and explicit
about discover-versus-execute separation.

**Independent Test**: prove discovery remains bounded and execution rejects mismatched, revoked, or
descriptorless delegated action bindings before handler execution.

### Tests For User Story 1

- [x] T006 [P] [US1] Extend ToolBus capability-boundary coverage in
      `crates/mister-smith-agents/tests/tool_bus_tests.rs`
- [x] T007 [P] [US1] Extend MCP discovery and action-metadata coverage in
      `crates/mister-smith-mcp/src/server.rs`,
      `crates/mister-smith-mcp/src/client.rs`,
      `crates/mister-smith-mcp/src/bridge.rs`, and
      `crates/mister-smith-mcp/src/compatibility.rs`

### Implementation For User Story 1

- [x] T008 [P] [US1] Remove descriptorless legacy execute authorization in
      `crates/mister-smith-security/src/delegation.rs`
- [x] T009 [P] [US1] Validate current shared capability descriptor use in
      `crates/mister-smith-agents/src/tool_bus.rs`
- [x] T010 [P] [US1] Publish both discover and execute actions in MCP capability metadata in
      `crates/mister-smith-mcp/src/client.rs` and `crates/mister-smith-mcp/src/server.rs`
- [x] T011 [US1] Preserve bounded discovery and exact action enforcement in
      `crates/mister-smith-mcp/src/compatibility.rs` and
      `crates/mister-smith-mcp/src/bridge.rs`

**Checkpoint**: Discovery and execution remain separate across ToolBus and MCP, and execute paths
require a descriptor-bound capability.

---

## User Story 2 - Quarantine And Schema Enforcement (Priority: P1)

**Goal**: Keep cross-boundary payloads and shared-state reads deterministically validated before
agent consumption.

**Independent Test**: prove clean payloads pass, sanitized payloads are marked with a reason,
monitored suspicious payloads keep a reason, and rejected or quarantined payloads do not reach
agent context.

### Tests For User Story 2

- [x] T012 [P] [US2] Validate the size, sanitize, schema, and malicious-pattern pipeline through
      `crates/mister-smith-security/tests/quarantine_tests.rs`
- [x] T013 [P] [US2] Extend quarantine and sandbox crossing coverage in
      `crates/mister-smith-security/tests/quarantine_tests.rs`,
      `crates/mister-smith-security/tests/sandbox_tests.rs`,
      `crates/mister-smith-agents/tests/quarantine_tests.rs`, and
      `crates/mister-smith-agents/tests/sandbox_tests.rs`
- [x] T014 [P] [US2] Validate shared-state mediation coverage in
      `crates/mister-smith-persistence/tests/repository_tests.rs`

### Implementation For User Story 2

- [x] T015 [P] [US2] Tighten quarantine decision mapping, monitored reasons, and audit reasons in
      `crates/mister-smith-security/src/quarantine.rs`
- [x] T016 [P] [US2] Preserve the current size, sanitization, schema, and malicious-pattern
      pipeline in `crates/mister-smith-security/src/state_validator.rs`
- [x] T017 [P] [US2] Preserve required quarantine enforcement for protected crossings in
      `crates/mister-smith-security/src/sandbox.rs` and
      `crates/mister-smith-agents/src/sandbox.rs`
- [x] T018 [US2] Preserve validated shared-state mediation in
      `crates/mister-smith-persistence/src/repository/agent.rs`

**Checkpoint**: Cross-boundary content and shared-state reads are deterministically mediated before
agent use with explicit evidence.

---

## User Story 3 - Identity, Auth Callout, And Delegation Continuity (Priority: P2)

**Goal**: Keep least-privilege identity and delegation rules bounded without widening into a new
IAM program or breaking packet `016` continuity truth.

**Independent Test**: prove auth-callout fallback stays capped at quarantined access, delegated
authority remains action-bound, sandbox classes stay isolated, and packet `016` continuity still
holds.

### Tests For User Story 3

- [x] T019 [P] [US3] Extend auth-callout permission-tier and fallback coverage in
      `crates/mister-smith-security/tests/auth_callout_tests.rs`
- [x] T020 [P] [US3] Extend delegated-action binding, expiry, and revocation coverage in
      `crates/mister-smith-security/tests/delegation_tests.rs`
- [x] T021 [P] [US3] Validate identity and sandbox lifecycle isolation coverage in
      `crates/mister-smith-security/tests/sandbox_tests.rs` and
      `crates/mister-smith-agents/tests/sandbox_tests.rs`

### Implementation For User Story 3

- [x] T022 [P] [US3] Clamp least-privilege auth-callout fallback at the quarantined ceiling in
      `crates/mister-smith-security/src/auth_callout.rs`
- [x] T023 [P] [US3] Keep external envelope, action binding, and revocation continuity strict in
      `crates/mister-smith-security/src/delegation.rs`
- [x] T024 [US3] Preserve packet `016` continuity wording in
      `specs/024-agent-boundary-security-hardening/spec.md`,
      `specs/024-agent-boundary-security-hardening/contracts/identity-and-sandbox-boundary.md`,
      and `specs/024-agent-boundary-security-hardening/analyze.md`

**Checkpoint**: Least-privilege identity posture stays bounded and packet `016` continuity remains
honest.

---

## Final Validation And Evidence

- [x] T025 Run `npx markdownlint-cli2 "specs/024-agent-boundary-security-hardening/**/*.md" --config .markdownlint.json`
- [x] T026 Run `cargo test -p mister-smith-security --test delegation_tests --test auth_callout_tests --test quarantine_tests --test sandbox_tests`
- [x] T027 Run `cargo test -p mister-smith-agents --test tool_bus_tests --test quarantine_tests --test sandbox_tests`
- [x] T028 Run `cargo test -p mister-smith-mcp`
- [x] T029 Run `cargo test -p mister-smith-persistence`
- [x] T030 Run `cargo build --workspace`
- [x] T031 Run `git diff --check`
- [x] T032 Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

## Parallel Staging Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-mcp/src/client.rs`
- `crates/mister-smith-mcp/src/bridge.rs`
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

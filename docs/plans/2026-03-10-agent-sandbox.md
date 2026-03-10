# AgentSandbox Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement Phase 9.1.4 AgentSandbox so persistent and ephemeral agents receive lifecycle-scoped NATS credentials, cross-boundary traffic is deny-by-default with quarantine-aware crossing rules, and ephemeral credentials are cleaned up on completion or timeout.

**Architecture:** Add the core sandbox primitives and credential issuer in `mister-smith-security`, then layer agent classification and spawn/cleanup integration in `mister-smith-agents`. Drive the work test-first with targeted security and agents tests, then validate cross-crate compilation with a workspace build.

**Tech Stack:** Rust workspace crates, `nats-jwt`, `nkeys`, existing `AuthCalloutHandler`/`Permissions`, existing agent runtime and supervision primitives

---

### Task 1: Add failing security tests for sandbox primitives

**Files:**
- Create: `crates/mister-smith-security/tests/sandbox_tests.rs`
- Read: `crates/mister-smith-security/src/auth_callout.rs`
- Read: `crates/mister-smith-security/src/state_validator.rs`
- Read: `specs/011-phase9.1-security-hardening/contracts/agent-sandbox.md`

**Step 1: Write the failing tests**

Add tests for:
- persistent vs ephemeral credentials use distinct accounts and JWT TTLs
- issued JWTs contain non-overlapping publish/subscribe permissions
- cleanup removes ephemeral credentials from the active credential store
- I/O firewall allows same-account traffic, requires quarantine for explicit cross-boundary task subjects, and rejects unauthorized crossings

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-security sandbox -- --nocapture`

Expected: FAIL because `sandbox` module/types do not exist yet.

**Step 3: Write minimal implementation**

Create `crates/mister-smith-security/src/sandbox.rs` with:
- `AgentClass`
- `SandboxCredentials`
- `CrossingRule`
- `CrossingDecision`
- `IOFirewall`
- `SandboxCredentialIssuer` plus small account config helpers

Use existing `Permissions` + `nats-jwt` signing instead of inventing a second credential format.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-security sandbox -- --nocapture`

Expected: PASS

### Task 2: Add failing agents tests for classification and cleanup integration

**Files:**
- Create: `crates/mister-smith-agents/tests/sandbox_tests.rs`
- Read: `crates/mister-smith-agents/src/agent.rs`
- Read: `crates/mister-smith-agents/src/config.rs`
- Read: `crates/mister-smith-agents/tests/lifecycle_tests.rs`

**Step 1: Write the failing tests**

Add tests for:
- default class assignment by agent role and timeout heuristic
- override-based class assignment
- sandboxed spawn returns credentials in the expected account
- ephemeral sandboxed spawn auto-cleans credentials after termination and timeout

**Step 2: Run test to verify it fails**

Run: `cargo test -p mister-smith-agents sandbox -- --nocapture`

Expected: FAIL because `mister_smith_agents::sandbox` does not exist yet.

**Step 3: Write minimal implementation**

Create `crates/mister-smith-agents/src/sandbox.rs` with:
- `AgentSandbox`
- `SandboxedAgentRuntime`
- class assignment helpers
- wrapper spawn helpers around existing `spawn_agent` / `spawn_supervised`
- background cleanup watcher for ephemeral agents

Prefer re-exports from `mister-smith-security` rather than duplicate type definitions.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mister-smith-agents sandbox -- --nocapture`

Expected: PASS

### Task 3: Wire exports and keep the workpad current

**Files:**
- Modify: `crates/mister-smith-security/src/lib.rs`
- Modify: `crates/mister-smith-agents/src/lib.rs`

**Step 1: Export the new modules**

Add `pub mod sandbox;` plus re-exports needed by tests and the quickstart shape.

**Step 2: Run narrow verification**

Run:
- `cargo test -p mister-smith-security sandbox -- --nocapture`
- `cargo test -p mister-smith-agents sandbox -- --nocapture`

Expected: PASS

### Task 4: Run full validation for the touched scope

**Files:**
- Modify if needed based on failures: `crates/mister-smith-security/src/sandbox.rs`
- Modify if needed based on failures: `crates/mister-smith-agents/src/sandbox.rs`

**Step 1: Run crate-level verification**

Run:
- `cargo test -p mister-smith-security`
- `cargo test -p mister-smith-agents`

Expected: PASS

**Step 2: Run cross-crate compile verification**

Run: `cargo build --workspace`

Expected: PASS

**Step 3: Vet the diff**

Run `vet` with the current Codex session history after each logical code unit and once again after the final diff.

### Scope Notes

Included:
- S017-S021 only
- sandbox primitives, account isolation modeling, credential lifecycle, agent classification, cleanup, and tests

Excluded:
- Phase 9.1.5 quarantine actor implementation
- deploy artifact changes for NATS version pinning
- transport-wide subject taxonomy refactors
- speculative persistence-layer changes for durable state rehydration

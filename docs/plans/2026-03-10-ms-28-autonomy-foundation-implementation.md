# MS-28 Autonomy Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish the shared Phase 10 autonomy dependency, core contracts, and typed event envelopes needed by downstream topology, memory, Guard, and delegation work.

**Architecture:** Add the new autonomy contract layer in `mister-smith-core`, then layer typed autonomy event envelopes and summaries in `mister-smith-events`. Keep the scope to cross-crate primitives only: IDs, enums, errors, value objects, and event payloads. Do not implement execution-graph behavior, memory management, or Guard logic in this issue.

**Tech Stack:** Rust workspace crates, `serde`, `thiserror`, `uuid`, `chrono`, `petgraph`

---

### Task 1: Dependency plumbing and red tests

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/mister-smith-agents/Cargo.toml`
- Modify: `crates/mister-smith-core/tests/trait_compilation_tests.rs`
- Create: `crates/mister-smith-events/tests/autonomy_event_tests.rs`

**Step 1: Write the failing tests**

- Extend `trait_compilation_tests.rs` to reference the new autonomy IDs, enums, error conversions, and representative autonomy value objects.
- Create `autonomy_event_tests.rs` to assert `EventType::Autonomy(...)`, typed autonomy envelope serialization, and `AutonomyStatusView` round-trips.

**Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p mister-smith-core --test trait_compilation_tests
cargo test -p mister-smith-events --test autonomy_event_tests
```

Expected: compile failures because the new autonomy types and event surfaces do not exist yet.

### Task 2: Core autonomy contracts

**Files:**
- Create: `crates/mister-smith-core/src/autonomy.rs`
- Modify: `crates/mister-smith-core/src/ids.rs`
- Modify: `crates/mister-smith-core/src/enums.rs`
- Modify: `crates/mister-smith-core/src/error.rs`
- Modify: `crates/mister-smith-core/src/lib.rs`
- Modify: `crates/mister-smith-core/Cargo.toml`

**Step 1: Write minimal implementation**

- Add the autonomy module with the smallest stable shared value types required by the spec-backed data model:
  `TopologyPlan`, `TopologyRationale`, `ContextBudget`, `MetricWindow`, `SemanticSignal`,
  `ProfileSnapshot`, `GuardEvidence`, `GuardDecision`, `InterventionRecord`,
  `DelegationCapability`, `ProvenanceChain`, and `ProvenanceLink`.
- Add autonomy ID newtypes in `ids.rs`.
- Add autonomy enums in `enums.rs` for topology, dependency, branch/node/graph state, memory budget policy, profile/guard targeting, intervention, delegation scope, and revocation.
- Add autonomy domain errors plus a top-level `AutonomyError` in `error.rs`, and wire them into `SystemError`.
- Re-export the new surfaces from `lib.rs`.

**Step 2: Run tests to verify core passes**

Run:

```bash
cargo test -p mister-smith-core --test trait_compilation_tests
```

Expected: PASS

### Task 3: Events autonomy contracts

**Files:**
- Create: `crates/mister-smith-events/src/autonomy.rs`
- Modify: `crates/mister-smith-events/src/types.rs`
- Modify: `crates/mister-smith-events/src/lib.rs`

**Step 1: Write minimal implementation**

- Introduce `AutonomyEventType` and add an `Autonomy(...)` branch to `EventType`.
- Add typed autonomy summaries and event envelopes in `events/src/autonomy.rs`, including:
  `ExecutionGraphSummary`, `TopologyPlanSummary`, `BranchSummary`,
  `ContextPressureSummary`, `CapabilitySummary`, `DelegationAlert`,
  `AutonomyStatusView`, and the typed autonomy event enum/envelopes needed for serialization tests.
- Keep delegation summaries faithful to the core autonomy contracts: preserve policy principals in
  `CapabilitySummary` instead of narrowing all issuers to runtime agents.
- Re-export the autonomy event surfaces from `lib.rs`.

**Step 2: Run tests to verify events pass**

Run:

```bash
cargo test -p mister-smith-events --test autonomy_event_tests
```

Expected: PASS

### Task 4: Workspace validation

**Files:**
- Modify: `Cargo.lock` if dependency metadata changes after build/test

**Step 1: Run focused crate validation**

Run:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-events
```

Expected: PASS

**Step 2: Run cross-workspace compile validation**

Run:

```bash
cargo build --workspace
```

Expected: PASS

### Scope guardrails

- Do not add execution-graph implementations, branch checkpoint logic, memory manager behavior, Guard policies, or delegation enforcement logic in this issue.
- Do not refactor existing generic event bus behavior beyond the typed event discriminator and exports required for the new autonomy surfaces.
- Do not introduce provider-specific or security-only abstractions that are not required by the Phase 10 contracts.

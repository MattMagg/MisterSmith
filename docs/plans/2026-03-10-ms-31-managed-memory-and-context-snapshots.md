# MS-31 Managed Memory and Context Snapshots Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the Phase 10.3 managed-memory layer so persistence can store metadata-rich fragments and snapshots, and agents can assemble bounded role-aware context plus snapshot-based resume without replaying raw history.

**Architecture:** Build a focused memory module inside `mister-smith-persistence` for fragment metadata, snapshot assembly, and async consolidation over existing Phase 6 state/task storage. Then add an `mister-smith-agents` context manager that consumes those types, narrows role context, and threads snapshot-aware context into planner, executor, critic, and memory-role flows without replacing the current orchestration/runtime foundations.

**Tech Stack:** Rust workspace crates, `serde_json`, `chrono`, existing `mister-smith-core` autonomy IDs/errors, `mister-smith-persistence` repositories + PostgreSQL metadata fields, targeted crate tests, ignored performance coverage.

---

### Task 1: Red Tests For Persistence Memory Semantics

**Files:**
- Create: `crates/mister-smith-persistence/tests/memory_manager_tests.rs`
- Modify: `crates/mister-smith-persistence/tests/performance_tests.rs`
- Read while implementing: `crates/mister-smith-persistence/src/repository/agent.rs`, `crates/mister-smith-persistence/src/repository/task.rs`, `specs/012-phase10-frontier-autonomy/contracts/memory-manager.md`

**Step 1: Write failing managed-memory tests**

- Add tests that expect:
  - role-aware assembly respects `ContextBudget`
  - summarize/consolidate policies reduce delivered context instead of widening it
  - snapshots reconstruct context from fragment IDs plus summary metadata
  - consolidation preserves provenance/access metadata

**Step 2: Run the red phase**

Run:

```bash
cargo test -p mister-smith-persistence --test memory_manager_tests
```

Expected: compile or test failures because the memory module and APIs do not exist yet.

**Step 3: Add ignored performance coverage stubs**

- Add ignored tests in `crates/mister-smith-persistence/tests/performance_tests.rs` for:
  - context reduction ratio assertions supporting `SC-202`
  - async consolidation behavior under load

**Step 4: Re-run the persistence test target**

Run:

```bash
cargo test -p mister-smith-persistence --test memory_manager_tests
```

Expected: still red until the new module exists.

### Task 2: Implement Persistence Memory Module And Repository Hooks

**Files:**
- Create: `crates/mister-smith-persistence/src/memory/mod.rs`
- Create: `crates/mister-smith-persistence/src/memory/fragment.rs`
- Create: `crates/mister-smith-persistence/src/memory/manager.rs`
- Create: `crates/mister-smith-persistence/src/memory/snapshot.rs`
- Create: `crates/mister-smith-persistence/src/memory/consolidation.rs`
- Modify: `crates/mister-smith-persistence/src/lib.rs`
- Modify: `crates/mister-smith-persistence/src/repository/agent.rs`
- Modify: `crates/mister-smith-persistence/src/repository/task.rs`

**Step 1: Define the memory data model**

- Add fragment/snapshot types that cover:
  - fragment provenance
  - freshness metadata
  - access policy
  - fragment class
  - snapshot scope
  - summary payloads
  - delivered/reduced context metrics

**Step 2: Implement `ManagedMemoryManager`**

- Keep it narrowly scoped to:
  - store/register fragments
  - assemble role-filtered snapshots under budget
  - materialize snapshot content
  - asynchronously consolidate older fragments into summary/checkpoint fragments

**Step 3: Extend repositories with memory metadata APIs**

- Add agent/task helpers for:
  - persisting serialized fragments and snapshots through existing state/metadata storage
  - listing or loading persisted snapshot references per scope
  - recording enough metadata for snapshot-based resume

**Step 4: Re-run the persistence red test to turn it green**

Run:

```bash
cargo test -p mister-smith-persistence --test memory_manager_tests
```

Expected: green for the new persistence coverage.

**Step 5: Run crate validation**

Run:

```bash
cargo test -p mister-smith-persistence
```

Expected: the crate passes, excluding env-gated ignored tests.

### Task 3: Red Tests For Agent Context Assembly And Resume

**Files:**
- Create: `crates/mister-smith-agents/tests/context_manager_tests.rs`
- Read while implementing: `crates/mister-smith-agents/src/roles/planner.rs`, `crates/mister-smith-agents/src/roles/executor.rs`, `crates/mister-smith-agents/src/roles/critic.rs`, `crates/mister-smith-agents/src/roles/memory.rs`, `crates/mister-smith-agents/src/execution_graph.rs`

**Step 1: Write failing context-manager tests**

- Add tests that expect:
  - planner/executor/critic receive different context payloads from the same fragment set
  - resume uses a stored snapshot instead of raw-history broadcast
  - conservative handling is preserved when metadata is missing or budget is too small

**Step 2: Run the red phase**

Run:

```bash
cargo test -p mister-smith-agents --test context_manager_tests
```

Expected: failures because `context_manager.rs` and the role integrations do not exist yet.

### Task 4: Implement Context Manager And Role Integrations

**Files:**
- Create: `crates/mister-smith-agents/src/context_manager.rs`
- Modify: `crates/mister-smith-agents/src/lib.rs`
- Modify: `crates/mister-smith-agents/src/roles/planner.rs`
- Modify: `crates/mister-smith-agents/src/roles/executor.rs`
- Modify: `crates/mister-smith-agents/src/roles/critic.rs`
- Modify: `crates/mister-smith-agents/src/roles/memory.rs`
- Modify if needed for snapshot linkage: `crates/mister-smith-agents/src/execution_graph.rs`

**Step 1: Add the context manager**

- Implement a small orchestration helper that:
  - asks the persistence memory manager for a snapshot
  - produces role-specific context views
  - exposes snapshot materialization for checkpoint resume

**Step 2: Thread it into the role modules**

- Update planner/executor/critic entry points so they can consume role-assembled context instead of only raw shared transcript payloads.
- Update memory-role flows so the memory role can store fragments, consolidate them, and serve snapshots.

**Step 3: Wire snapshot-aware resume**

- Prefer linking branch/checkpoint state to `MemorySnapshotId` instead of assuming raw-history reconstruction.
- Keep the implementation minimal and compatible with the existing `ExecutionGraph`/checkpoint seams on `main`.

**Step 4: Turn the agent test target green**

Run:

```bash
cargo test -p mister-smith-agents --test context_manager_tests
```

Expected: green for the new agent context coverage.

**Step 5: Run crate validation**

Run:

```bash
cargo test -p mister-smith-agents
```

Expected: the crate passes.

### Task 5: Final Verification And Workpad Update

**Files:**
- Modify: Linear workpad comment for `MS-31`

**Step 1: Run cross-crate verification**

Run:

```bash
cargo build --workspace
```

Expected: success across the workspace.

**Step 2: Reconcile the ticket workpad**

- Check off completed T016-T021 items
- Record validation evidence
- Note any ignored env-gated coverage that still requires external services

**Step 3: Run vet after the final logical unit**

Run `vet` against the session diff if the local tool is available. Treat findings about unrelated edits as noise, but fix any real issue tied to this ticket before claiming completion.

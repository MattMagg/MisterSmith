# MS-29 Execution Graph and Topology Compiler Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Compile planner output into a validated `ExecutionGraph`, select topology deterministically, and normalize orchestration dispatch through that graph before runtime work starts.

**Architecture:** Add a typed execution-graph layer inside `mister-smith-agents` that consumes the Phase 10.0 autonomy contracts already defined in `mister-smith-core` and `mister-smith-events`. Integrate the compiler at the planner/coordinator/orchestrator seam without replacing the existing scheduler-based runtime, so Gate 7/9 behavior stays intact while dispatch gains a frontier-autonomy control plane.

**Tech Stack:** Rust workspace, `mister-smith-agents`, `mister-smith-core`, `mister-smith-events`, `petgraph`, `cargo test`, `cargo build`

---

### Task 1: Add failing graph and topology tests

**Files:**
- Create: `crates/mister-smith-agents/tests/execution_graph_tests.rs`
- Create: `crates/mister-smith-agents/tests/topology_tests.rs`
- Check: `crates/mister-smith-agents/tests/team_tests.rs`
- Check: `crates/mister-smith-agents/tests/gate9_tests.rs`

**Step 1: Write failing execution-graph tests**

Cover:
- valid graph creation from planner-style JSON
- rejection of missing dependencies
- rejection of cycles
- explicit graph state / branch derivation expectations

**Step 2: Run the focused graph test**

Run: `cargo test -p mister-smith-agents --test execution_graph_tests`
Expected: FAIL because `execution_graph.rs` and related APIs do not exist yet

**Step 3: Write failing topology tests**

Cover:
- deterministic `Parallel` or `Hybrid` selection for independent branches
- deterministic `Sequential` or `Pipeline` selection for strict chains
- rationale contains dependency shape plus operational signals
- conservative fallback topology is explicit

**Step 4: Run the focused topology test**

Run: `cargo test -p mister-smith-agents --test topology_tests`
Expected: FAIL because `topology.rs` and compiler APIs do not exist yet

### Task 2: Implement typed execution graph contracts

**Files:**
- Create: `crates/mister-smith-agents/src/execution_graph.rs`
- Modify: `crates/mister-smith-agents/src/lib.rs`

**Step 1: Define core graph types**

Implement:
- `ExecutionNode`
- `ExecutionEdge`
- `ExecutionBranch`
- `ExecutionGraph`
- planner-input normalization helpers

Use the existing Phase 10.0 types:
- `ExecutionGraphId`
- `ExecutionNodeId`
- `ExecutionBranchId`
- `ContextBudget`
- `TopologyPlan`
- `TopologyError`
- `GraphState`
- `NodeState`
- `BranchState`
- `DependencyType`
- `CheckpointPolicy`
- `BranchRecoveryStrategy`

**Step 2: Implement validation**

Validate:
- non-empty nodes
- node IDs unique
- all dependencies resolve to known nodes
- edges agree with node dependency declarations
- graph is acyclic
- branch membership is complete and stable

**Step 3: Re-export the module**

Expose the new graph types from `mister-smith-agents`.

**Step 4: Run the graph test again**

Run: `cargo test -p mister-smith-agents --test execution_graph_tests`
Expected: remaining failures only for missing topology/integration behavior

### Task 3: Implement deterministic topology compilation

**Files:**
- Create: `crates/mister-smith-agents/src/topology.rs`
- Modify: `crates/mister-smith-agents/src/lib.rs`

**Step 1: Define compiler inputs**

Implement:
- `TopologySignals`
- `TopologyCompiler`
- internal dependency-shape analysis helpers

Signal inputs should stay conservative and deterministic:
- dependency depth
- maximum parallel width
- health/budget hints
- branch count

**Step 2: Implement selection policy**

Support:
- `Sequential`
- `Parallel`
- `Pipeline`
- `Hierarchical`
- `Hybrid`

Keep rationale typed via `TopologyRationale` and always include:
- dependency shape
- at least one operational signal
- selected topology reason
- fallback reason when applicable

**Step 3: Validate compiler behavior**

Make `compile()` normalize planner output into `ExecutionGraph`, validate it, and attach the selected `TopologyPlan`.

**Step 4: Run topology tests**

Run: `cargo test -p mister-smith-agents --test topology_tests`
Expected: PASS

### Task 4: Integrate planner, coordinator, and orchestrator

**Files:**
- Modify: `crates/mister-smith-agents/src/roles/planner.rs`
- Modify: `crates/mister-smith-agents/src/roles/coordinator.rs`
- Modify: `crates/mister-smith-agents/src/orchestrator.rs`

**Step 1: Add planner normalization support**

Keep existing JSON-facing planner behavior for compatibility, but add typed helpers so the planner's output can be turned into an `ExecutionGraph`.

**Step 2: Add coordinator visibility**

Track the compiled graph in coordinator state or responses so coordination can report workflow structure rather than only loose task IDs.

**Step 3: Gate orchestrator dispatch on validated graphs**

Ensure dispatch/decomposition goes through:
1. planner output
2. graph compilation
3. graph validation
4. topology selection
5. scheduler submission

Reject invalid graphs before any subtask assignment occurs.

**Step 4: Run targeted integration coverage**

Run:
- `cargo test -p mister-smith-agents --test execution_graph_tests`
- `cargo test -p mister-smith-agents --test topology_tests`
- `cargo test -p mister-smith-agents --test team_tests`

Expected: PASS

### Task 5: Validate and harden

**Files:**
- Check: `crates/mister-smith-agents/src/*`
- Check: `crates/mister-smith-agents/tests/*`

**Step 1: Run vet on the change set**

Run `vet` against the current Codex session after the graph/topology implementation lands and again after the integration changes land.

**Step 2: Run the affected crate tests**

Run: `cargo test -p mister-smith-agents`
Expected: PASS

**Step 3: Run workspace build verification**

Run: `cargo build --workspace`
Expected: PASS

**Step 4: Reconcile Linear workpad**

Update the `## Codex Workpad` comment with completed checklist items, validation output, and any remaining blockers.

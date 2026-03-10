# Quickstart: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

## Prerequisites

- Phase 1-9.1 workspace crates present and building
- Active feature directory: `specs/012-phase10-frontier-autonomy/`
- Existing Phase 9 `ModelEvent` / dual-stream surfaces available as Guard inputs
- Existing Phase 9.1 security substrate available for delegation/provenance enforcement
- NATS + JetStream available for env-gated checkpoint, routing, and operator-state integration
- PostgreSQL available for env-gated managed-memory integration tests

## Planned Build Flow After Implementation

```bash
# 1. Cross-crate compile safety
cargo build --workspace

# 2. Topology compiler and execution-graph tests
cargo test -p mister-smith-agents -- topology
cargo test -p mister-smith-agents -- execution_graph

# 3. Branch checkpoint and resume tests (env-gated where needed)
NATS_URL=nats://localhost:4222 cargo test -p mister-smith-agents -- checkpoint --ignored

# 4. Managed memory / context manager tests
cargo test -p mister-smith-persistence -- memory
DATABASE_URL=postgres://localhost/mister_smith cargo test -p mister-smith-persistence -- snapshot --ignored

# 5. Guard / Advisor and stream-monitor tests
cargo test -p mister-smith-agents -- guard
cargo test -p mister-smith-llm -- stream_monitor

# 6. Delegation and provenance tests
cargo test -p mister-smith-security -- delegation

# 7. Operator autonomy view tests
cargo test -p mister-smith-app -- autonomy

# 8. Lint baseline
cargo clippy --workspace -- -D warnings
```

## Usage Sketch

### Topology-Aware Execution

```rust
let graph = topology_compiler.compile(planner_output)?;
let topology = topology_compiler.select_topology(&graph, &signals)?;
let run = orchestrator.dispatch(graph, topology).await?;
```

### Managed Context Assembly

```rust
let budget = context_manager.budget_for(role, branch_id)?;
let snapshot = context_manager.assemble_snapshot(branch_id, role, budget).await?;
let context = context_manager.materialize(snapshot).await?;
```

### Predictive Supervision

```rust
let profile = guard.observe(target_scope).await?;
let decision = guard.evaluate(profile, stream_signals, checkpoint_state).await?;
guard.apply(decision).await?;
```

### Delegation and Provenance

```rust
let capability = delegation.issue(parent, recipient, scope, expiry)?;
delegation.validate(&capability, &action)?;
audit.record_provenance(&capability, &action).await?;
```

## Verification Scenarios

### Scenario 1: Mixed-Dependency Workflow

1. Submit a workflow with one sequential branch and two independent branches.
2. Compile it into an `ExecutionGraph`.
3. Verify the system chooses a hybrid or parallel-capable topology.
4. Confirm the independent branches execute concurrently while the sequential branch preserves
   order.

### Scenario 2: Branch Failure and Local Recovery

1. Start a checkpoint-enabled workflow.
2. Force one branch to fail after a sibling branch completes.
3. Verify the Guard layer chooses branch-local recovery.
4. Confirm the failed branch resumes or is reassigned from checkpoint without replaying the
   completed sibling branch.

### Scenario 3: Context Pressure and Memory Consolidation

1. Run a workflow that exceeds the working context budget for at least one role.
2. Verify older context is summarized or paged.
3. Confirm the resulting `MemorySnapshot` still reconstructs a valid working context for resume.

### Scenario 4: Operator Inspection of Autonomy State

1. Execute a workflow that causes at least one Guard intervention.
2. Inspect the operator/autonomy view.
3. Verify topology, checkpoint lineage, context pressure, and intervention rationale are visible
   without raw log inspection.

### Scenario 5: Delegation Rejection

1. Issue a bounded delegation chain for a privileged action.
2. Revoke or expire the chain before the downstream execution.
3. Verify the action is blocked and the operator-visible provenance record explains why.

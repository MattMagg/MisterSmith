# Quickstart: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

## Prerequisites

- Phase 1-9.1 workspace crates present and building
- Active feature directory: `specs/012-phase10-frontier-autonomy/`
- Existing Phase 9 `ModelEvent` / dual-stream surfaces available as Guard inputs
- Existing Phase 9.1 security substrate available for delegation/provenance enforcement
- NATS + JetStream available for env-gated checkpoint, routing, and operator-state integration
- PostgreSQL available for env-gated managed-memory integration tests

## Gate Validation Flow (2026-03-15)

```bash
# 1. Cross-crate compile safety
cargo build --workspace

# 2. Targeted Phase 10 gate suites
cargo test -p mister-smith-agents
cargo test -p mister-smith-persistence
cargo test -p mister-smith-security
cargo test -p mister-smith-llm
cargo test -p mister-smith-core
cargo test -p mister-smith-app

# 3. Deploy artifact syntax
python3 scripts/validate_deploy_assets.py deploy/dashboards deploy/alerts
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

## Scenario Mapping To Gate Evidence

- **Scenario 1** maps to `cargo test -p mister-smith-agents`, especially the
  `execution_graph_tests`, `topology_tests`, and `gate10_tests` suites.
- **Scenario 2** maps to `cargo test -p mister-smith-agents`, especially the
  `checkpoint_tests` and `gate10_tests` suites.
- **Scenario 3** maps to `cargo test -p mister-smith-persistence` plus
  `cargo test -p mister-smith-agents`, especially the memory manager, performance, and
  context-manager suites.
- **Scenario 4** maps to `cargo test -p mister-smith-app` and `cargo test -p mister-smith-agents`,
  plus deploy asset validation for the autonomy dashboard and alert rules.
- **Scenario 5** maps to `cargo test -p mister-smith-security`,
  `cargo test -p mister-smith-agents`, and `cargo test -p mister-smith-app`, where revoked,
  expired, or invalid delegation chains are rejected and surfaced to operators.

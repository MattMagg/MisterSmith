# Contract: Topology Compiler

## Overview

The Topology Compiler contract defines how planner output becomes a validated `ExecutionGraph`
with an explicit `TopologyPlan` before dispatch. Phase 10 uses this compiler as the single
authoritative boundary between planning and execution.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/02-orchestration-and-self-organization.md` | Dynamic topology selection and dependency-graph analysis |
| `docs/research-output/consolidated/03-supervision-and-resilience.md` | Branch-local recovery and checkpoint-aware orchestration |
| `spec/data-management/agent-orchestration.md` | Planner/executor/router boundaries |
| `spec/data-management/message-schemas.md` | Workflow coordination message shapes |

## Public API

```rust
pub trait TopologyCompiler: Send + Sync {
    fn compile(&self, planner_output: &PlannerOutput) -> Result<ExecutionGraph, TopologyError>;
    fn validate(&self, graph: &ExecutionGraph) -> Result<(), TopologyError>;
    fn select_topology(
        &self,
        graph: &ExecutionGraph,
        signals: &TopologySignals,
    ) -> Result<TopologyPlan, TopologyError>;
}
```

## Topology Kinds

```text
Sequential    — strictly ordered execution
Parallel      — independent branches execute concurrently
Pipeline      — ordered stages with bounded overlap
Hierarchical  — subtree execution with local aggregation
Hybrid        — mixed strategy across subgraphs
```

## Behavioral Requirements

1. Compilation MUST produce an explicit graph structure before any branch is dispatched.
2. Validation MUST reject cycles, missing dependencies, and unsupported node types.
3. Topology selection MUST record rationale using task-shape, dependency depth, health, and
   budget signals.
4. A fallback topology MAY be selected, but it MUST be explicit in the `TopologyPlan`.
5. The same validated graph MUST be inspectable by operators and reusable by checkpoint recovery.

## Validation Requirements

- Parallel-friendly graph selects `Parallel` or `Hybrid`, not `Sequential` by default.
- Cyclic graph fails validation before dispatch.
- Tight sequential graph does not over-parallelize.
- Compiler rationale includes at least dependency shape and one operational signal.

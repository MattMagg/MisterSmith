# Research: Task-Shape-Aware Orchestration and Dynamic Team Sizing

**Date**: 2026-03-16  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

The repo already crossed the hard threshold that matters for `MS-45`: planner output is no longer
an unstructured fan-out. `main` now includes execution-graph compilation, task-shape
classification, topology selection, and typed autonomy summaries of those choices.

The remaining gap is more specific:

- worker pool width is still not an adaptive operating-system decision
- team sizing is not yet a first-class contract that operators can inspect
- there is no repeatable harness proving when adaptive sizing beats a fixed or sequential posture

The strongest repo-local conclusion is therefore:

- treat landed `MS-60` work as current truth
- freeze the adaptive-team contract once in shared types
- keep dynamic sizing, operator rendering, and evaluation as bounded follow-on work

## Current Repo Findings That Shape The Design

### R1: Task-Shape Classification And Topology Selection Already Exist On `main`

**Sources**:

- `docs/plans/2026-03-16-frontier-direction.md`
- `crates/mister-smith-agents/src/topology.rs`
- `crates/mister-smith-agents/tests/topology_tests.rs`

**Evidence**:

- `TopologyCompiler` already compiles planner output into an `ExecutionGraph`
- `classify_task_shape()` derives dependency shape from the graph
- `TopologyPlan` already carries topology kind, parallelism width, task shape, and rationale
- current tests already assert task-shape classification and topology rationale behavior

**Decision**: write the packet so `MS-60` is treated as landed baseline, not as speculative future
scope.

**Alternatives considered**:

- restate `MS-60` as a new unfinished story: rejected because it would misstate current mainline
  truth
- create a fresh packet only for `MS-61`: rejected because `MS-45` is the actual feature boundary

### R2: The Worker Set Is Still Passed In, Not Selected Adaptively

**Sources**:

- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/team.rs`
- `crates/mister-smith-agents/src/scheduler.rs`

**Evidence**:

- branch routing receives a `worker_ids` slice from the caller
- `select_worker()` chooses among provided workers, not an adaptively sized worker pool
- `Team` records membership but does not model desired versus selected size
- `TaskScheduler` tracks worker loads but does not choose how many workers should be active

**Decision**: put the unfinished feature focus on adaptive team sizing and lifecycle integration
before trying to invent broader orchestration changes.

**Alternatives considered**:

- replace routing with a new scheduling subsystem: rejected because the existing scheduler already
  provides the load/accounting seam the packet needs
- widen scope to a new persistent team service: rejected because it would add a parallel control
  plane

### R3: The Shared Choke Point Is The Decision Contract, Not The Whole Feature

**Sources**:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`

**Evidence**:

- the current autonomy/status types already define task-shape and topology summaries
- the orchestrator assembles `AutonomyStatusView` from the graph and recorded events
- any new team-sizing decision must be represented consistently in core types, event summaries,
  and orchestrator status assembly

**Decision**: freeze the adaptive-team contract once in a blocking serial checkpoint, then allow
bounded parallel work only after those shared types are stable.

**Alternatives considered**:

- let each lane define its own partial sizing fields: rejected because it would re-open the shared
  contract during implementation
- run `MS-61` and `MS-62` as totally independent parallel streams from the start: rejected because
  they both depend on the same contract choke points

### R4: Operator Visibility Should Extend Existing Autonomy Surfaces

**Sources**:

- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-app/src/autonomy.rs`

**Evidence**:

- `AutonomyStatusView` already includes graph, topology, branch, routing, intervention, and
  conservative-reason summaries
- the app already renders autonomy status from typed event projections
- there is no need for a new HTTP or CLI subsystem just to show team-size rationale

**Decision**: extend the current autonomy summary path with a team-sizing decision block instead of
adding a second operator surface.

**Alternatives considered**:

- create a separate adaptive-team report endpoint: rejected because it duplicates the current
  autonomy view
- leave team-size decisions implicit in branch assignment history: rejected because operators would
  still have to infer the decision from low-level events

### R5: The Proof Gap Is Evaluation, Not More Architecture Speculation

**Sources**:

- `docs/plans/2026-03-16-frontier-direction.md`
- `crates/mister-smith-agents/tests/gate10_tests.rs`
- `crates/mister-smith-agents/tests/topology_tests.rs`

**Evidence**:

- current tests prove topology and routing behavior but do not record a repeatable adaptive versus
  baseline comparison artifact
- the frontier direction explicitly requires measurable evidence on at least one parallel workload
  class
- the repo already uses dated `docs/plans/` notes for durable validation artifacts

**Decision**: require a deterministic evaluation harness and one durable evidence note under
`docs/plans/` as part of the packet.

**Alternatives considered**:

- rely only on ad hoc local observation: rejected because the next session would have no durable
  evidence
- require live provider-backed proof for every comparison run: rejected because deterministic local
  workload fixtures are enough for the sizing contract itself

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `docs/plans/2026-03-16-frontier-direction.md` | Defines `MS-45` as the next primary operating-system epic and names the three bounded slices. |
| `crates/mister-smith-agents/src/topology.rs` | Shows landed task-shape classification, topology selection, and rationale generation. |
| `crates/mister-smith-agents/src/orchestrator.rs` | Shows where routing still depends on a provided worker set and where autonomy status is assembled. |
| `crates/mister-smith-agents/src/team.rs` | Shows current team membership is recorded but not adaptively sized. |
| `crates/mister-smith-agents/src/scheduler.rs` | Shows worker load accounting exists and can support adaptive team selection without a new scheduler subsystem. |
| `crates/mister-smith-events/src/autonomy.rs` | Shows current typed autonomy summaries and the shared contract choke point for new sizing fields. |
| `crates/mister-smith-events/src/bus.rs` | Shows the current operator-view reconstruction path that should absorb team-size visibility. |
| `crates/mister-smith-app/src/autonomy.rs` | Shows the current CLI/operator rendering surface for autonomy inspection. |
| `specs/012-phase10-frontier-autonomy/` | Defines the substrate already implemented for topology, routing, and operator visibility. |

## Explicitly Deferred Questions

- whether adaptive sizing should eventually rebalance long-running workflows continuously instead
  of only at defined frontier transitions
- whether worker specialization should influence sizing beyond the current branch-width and
  dependency-depth heuristics
- whether future live-provider proofs should be added after the deterministic evaluation harness is
  in place

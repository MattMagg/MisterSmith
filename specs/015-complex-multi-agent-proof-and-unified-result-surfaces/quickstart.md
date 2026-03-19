# Quickstart: Complex Multi-Agent Proof and Unified Result Surfaces

## Prerequisites

- active feature directory:
  `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/`
- March 19 live runtime path and evaluation notes are treated as baseline truth
- local Rust toolchain available for targeted crate tests
- default live provider path available when real runtime proof is required
- no watched-queue staging is required to validate the packet structure itself

## Required Validation Bundle

```bash
# 1. Narrow automated checks for the proof and result-surface slice
cargo test -p mister-smith-agents
cargo test -p mister-smith-events
cargo test -p mister-smith-app

# 2. Cross-crate compile safety
cargo build --workspace
```

## Scenario 1: Confirm The Current Baseline And Shared Result Contract

1. Verify the current live path still records:
   - supervised planner and executor lifecycles
   - `tool_bus` execution boundary
   - task result persistence
   - retained session `assistant_result`
2. Confirm the packet does not reopen provider, KV, budget, or broad external-agent programs.
3. Confirm the future implementation maps:
   - metadata `final_result` -> canonical result object
   - nested `aggregated_result` -> execution-produced payload
   - `task.result` -> task-facing result envelope
   - session `assistant_result` -> retained session projection
   - operator preview/provenance -> bounded operator projection

## Scenario 2: Prove Successful Harder Graph Execution

1. Submit a harder workload that the planner can support on the default live path.
2. Capture:
   - graph formation evidence
   - branch count or step count
   - terminal task result
   - proof outcome classification
3. Verify the run lands as `graph_formed_and_completed`.
4. Verify the stored result material is inspectable through the canonical result contract.

## Scenario 3: Prove Visible Collapse To Sequential

1. Submit a workload that the planner accepts but compresses to a trivial sequential path.
2. Capture:
   - terminal task result
   - graph or branch evidence showing sequential collapse
   - bounded operator preview and provenance output
3. Verify the run lands as `collapsed_to_sequential`.
4. Verify the result contract still exposes the final-result material consistently across task and
   session views.

## Scenario 4: Prove Visible Planner-Time Failure

1. Submit a workload that stresses the planner beyond current graph-formation limits.
2. Capture:
   - task failure record
   - evidence that graph formation did not complete
   - bounded operator or evaluation evidence for failure classification
3. Verify the run lands as `failed_before_graph`.
4. Verify the evidence is sufficient to distinguish planner-time failure from worker execution
   failure.

## Scenario 5: Inspect Unified Result Surfaces

1. Inspect task status for a terminal workflow.
2. Inspect the corresponding session retained result view.
3. Inspect operator autonomy status for the same workflow.
4. Verify all three surfaces map back to the same canonical result contract and proof outcome.
5. Verify the operator surface shows a bounded preview and provenance block instead of a raw
   payload dump.

## Scenario 6: Capture The Evaluation Artifact

1. Run the proof matrix for:
   - one success case
   - one collapse case
   - one failure-visible case
2. Record the artifact under:
   - `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`
3. Verify the artifact includes:
   - workload class
   - proof outcome classification
   - graph-formation summary
   - bounded result preview
   - payload source and provenance summary

## Example Validation Flow

```bash
# Runtime proof-path coverage
cargo test -p mister-smith-agents step_routing_benchmark_tests -- --nocapture
cargo test -p mister-smith-agents team_sizing_benchmark_tests -- --nocapture
cargo test -p mister-smith-agents gate10_tests -- --nocapture

# Result projection coverage
cargo test -p mister-smith-events autonomy_event_tests -- --nocapture
cargo test -p mister-smith-app autonomy_status_tests -- --nocapture

# Cross-crate compile safety
cargo build --workspace
```

## Expected Proof Artifacts

- one success case classified as `graph_formed_and_completed`
- one collapse case classified as `collapsed_to_sequential`
- one failure-visible case classified as `failed_before_graph`
- a task-facing result envelope that maps to the canonical runtime result object
- a session-facing retained result view that preserves `assistant_result`
- an operator-facing preview and provenance block that points back to the canonical result object

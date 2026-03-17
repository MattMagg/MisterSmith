# Quickstart: Task-Shape-Aware Orchestration and Dynamic Team Sizing

## Prerequisites

- active feature directory: `specs/014-task-shape-aware-orchestration/`
- current Phase 10 topology and autonomy substrate already present on `main`
- local Rust toolchain available for targeted crate tests
- a bounded worker pool or deterministic fixtures available for adaptive sizing tests
- no watched-queue staging is required to validate the packet structure itself

## Required Validation Bundle

```bash
# 1. Narrow automated checks for the adaptive-sizing slice
cargo test -p mister-smith-agents
cargo test -p mister-smith-events
cargo test -p mister-smith-app

# 2. Cross-crate compile safety
cargo build --workspace
```

## Scenario 1: Confirm The Landed Baseline

1. Compile representative planner output that includes:
   - a strict chain
   - a wide fan-out
   - a mixed graph with joins
2. Verify the current runtime still records:
   - task shape
   - selected topology
   - topology rationale
3. Confirm the packet is extending a real baseline instead of reopening `MS-60`.

## Scenario 2: Adaptive Team Sizing Chooses Different Widths

1. Run a representative wide fan-out workload through the deterministic test harness.
2. Capture the selected team size.
3. Run a representative narrow or sequential workload through the same harness.
4. Verify the selected worker count differs and is justified by structure rather than a fixed
   default.

## Scenario 3: Conservative Mode Narrows The Team

1. Re-run the adaptive sizing path with conservative mode or elevated budget pressure enabled.
2. Capture:
   - desired team size
   - selected team size
   - cap reason
3. Verify the selected width narrows deterministically without violating workflow execution
   invariants.

## Scenario 4: Inspect Operator Status

1. Inspect workflow autonomy status for one adaptive workflow.
2. Verify the status includes:
   - task shape
   - selected topology
   - desired and selected team size
   - cap reason when applicable
   - routing rationale correlated to the same workflow

## Scenario 5: Run The Evaluation Harness

1. Run the deterministic comparison harness for at least:
   - one parallel-fanout workload class
   - one strict-chain workload class
2. Record the result in a dated note under `docs/plans/`.
3. Verify the note includes:
   - baseline mode
   - adaptive mode
   - selected team size
   - outcome (`improved`, `matched`, or `regressed`)

## Example Validation Flow

```bash
# Adaptive-team crate validation
cargo test -p mister-smith-agents team_sizing_tests -- --nocapture
cargo test -p mister-smith-agents team_sizing_benchmark_tests -- --nocapture

# Status-surface validation
cargo test -p mister-smith-events autonomy_event_tests -- --nocapture
cargo test -p mister-smith-app autonomy_status_tests -- --nocapture

# Cross-crate compile safety
cargo build --workspace
```

## Expected Proof Artifacts

- at least one workload class where adaptive team size differs from the baseline
- an operator-visible status view that shows desired and selected team width
- a dated evidence note under `docs/plans/` comparing adaptive and baseline execution
- explicit rationale for any cap or conservative narrowing

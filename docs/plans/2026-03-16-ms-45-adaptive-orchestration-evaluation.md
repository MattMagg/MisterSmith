# MS-45 Adaptive Orchestration Evaluation

Date captured: 2026-03-17  
Issue: `MS-62`  
Spec packet: `specs/014-task-shape-aware-orchestration/`

## Objective

Provide a deterministic, repo-local comparison between the adaptive team-sizing path and a fixed
single-worker baseline for representative workload classes.

## Harness

- Test file: `crates/mister-smith-agents/tests/team_sizing_benchmark_tests.rs`
- Workload classes:
  - `parallel_fanout`: root completed, three independent branches become ready
  - `strict_chain`: first step completed, exactly one downstream branch becomes ready
- Strategies:
  - `adaptive`: three workers available to the orchestrator
  - `sequential_baseline`: one worker available to the orchestrator
- Metric:
  - `ready_branch_count`: total ready frontier width before routing
  - `selected_workers`: active team width chosen by the runtime
  - `dispatch_rounds`: `ceil(ready_branch_count / selected_workers)`

## Results

| workload_class | strategy | ready_branch_count | desired_workers | selected_workers | dispatch_rounds | result |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `parallel_fanout` | `adaptive` | 3 | 3 | 3 | 1 | improvement |
| `parallel_fanout` | `sequential_baseline` | 3 | 3 | 1 | 3 | baseline |
| `strict_chain` | `adaptive` | 1 | 1 | 1 | 1 | neutral |
| `strict_chain` | `sequential_baseline` | 1 | 1 | 1 | 1 | baseline |

## Interpretation

- The adaptive path improves the representative parallel workload honestly: the same frontier
  requires `1` dispatch round under adaptive routing versus `3` rounds under the fixed
  single-worker baseline.
- The adaptive path reports an honest neutral result on the representative sequential workload:
  both strategies keep the team at `1` worker and complete the frontier in `1` round.
- This satisfies the packet requirement to show measurable improvement on a workload class where
  parallel structure matters without overstating benefit on a strict dependency chain.

## Validation

- `cargo test -p mister-smith-agents --test team_sizing_benchmark_tests -- --nocapture`

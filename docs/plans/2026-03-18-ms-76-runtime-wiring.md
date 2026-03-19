# MS-76 Runtime Wiring Plan

Date: March 18, 2026
Issue: `MS-76`
Status: Validation complete

## Objective

Wire the live runtime path to the already-landed orchestration substrate without changing the
current provider path. The runtime should use supervised planner and executor lifecycles, and
executor work should cross a real ToolBus boundary instead of returning LLM strategy text.

## Scope

- `crates/mister-smith-app/src/bootstrap.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/shutdown.rs`
- `crates/mister-smith-agents/src/roles/executor.rs`
- targeted tests in `crates/mister-smith-agents/tests/` and `crates/mister-smith-app/tests/`

## Assumptions

- The current live runtime proof on `openai_chatgpt` / `gpt-5.4` remains the baseline and should
  stay intact.
- `openai_chatgpt` tool-calling support is not the path for this slice; the executor needs a local
  ToolBus-backed execution boundary.
- The existing `SupervisedSystem`, `spawn_supervised`, `LlmSupervision`, and `ToolBus` substrate is
  sufficient for a first honest live-path wiring pass.

## Constraints

- Keep root `/Users/macmain/MisterSmith` on clean `main`; all code work stays in this issue-scoped
  workspace.
- Do not replace the fixed provider/model path.
- Do not widen this slice into JetStream KV routing or broader external tool integration.
- Keep the change reviewable and bounded to runtime wiring.

## Non-Goals

- JetStream KV budget/state rollout
- provider-neutral runtime routing
- new external MCP or remote-tool execution surfaces
- reopening older frontier packets outside the wiring needed for the live path

## Milestones

### 1. Supervision Ownership

Make the runtime bootstrap hand the live task service a real `SupervisedSystem`, start the
supervision loop, and stop using dedicated per-workflow OS threads for the main execution path.

Validation:

- affected unit tests pass
- runtime bootstrap/shutdown code compiles cleanly

Status:

- complete
- `bootstrap.rs` now starts the `SupervisedSystem` before runtime task bootstrap and hands the
  shared system into `RuntimeTaskService`
- `shutdown.rs` now stops the supervised system during graceful and forced shutdown
- `execution.rs` now runs workflows on Tokio tasks instead of dedicated per-workflow OS threads

### 2. Supervised Planner and Executor Calls

Move planner and executor invocation onto supervised actor runtimes so the live workflow path uses
the landed agent lifecycle substrate rather than direct struct calls.

Validation:

- targeted `mister-smith-agents` tests pass
- targeted `mister-smith-app` tests pass

Status:

- complete
- planner execution now runs through `spawn_supervised(...)` under a runtime supervisor
- executor execution now runs through supervised actor lifecycles instead of direct struct calls

### 3. ToolBus Execution Boundary

Replace the executor's strategy-only response path with a native ToolBus-backed workflow step tool
that produces the real step result payload used by the runtime.

Validation:

- new executor ToolBus tests pass
- `cargo build --workspace`

Status:

- complete
- executor `ExecutePlan` now uses a native ToolBus-backed `workflow.execute_step` path
- runtime task results expose `runtime_execution_mode` and per-step `execution_boundary` markers

## Validation Evidence

Local validation:

- `cargo fmt --all`
- `cargo test -p mister-smith-agents executor_uses_tool_bus_execution_boundary_when_configured`
- `cargo test -p mister-smith-app workflow_step_tool_marks_payload_as_tool_bus_completed`
- `cargo test -p mister-smith-app normalize_runtime_plan_reindexes_duplicate_numeric_steps`
- `cargo build --workspace`

Live runtime proof:

- runtime boot command:
  `env DATABASE_URL='postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_runtime_slice3' MISTER_SMITH_TRANSPORT__NATS_URL='nats://127.0.0.1:4222' cargo run -q -p mister-smith-app -- run`
- task submission:
  `POST /api/v1/tasks` for workflow `e7087007-2589-48c7-b3d6-aa3dac00d0aa`
- task result proved:
  - `runtime_execution_mode.planner_lifecycle = supervised_actor`
  - `runtime_execution_mode.executor_lifecycle = supervised_actor`
  - `runtime_execution_mode.workflow_runner = tokio_task`
  - `runtime_execution_mode.execution_boundary = tool_bus`
  - all `step_results[*].result.execution_boundary = tool_bus`
  - `tool_name = workflow.execute_step`
- operator inspection proved:
  `cargo run -q -p mister-smith-app -- autonomy status --workflow-id e7087007-2589-48c7-b3d6-aa3dac00d0aa --base-url http://127.0.0.1:8080`
  reported `Completed` hybrid topology with two worker roots and one join branch

Regression found and fixed during validation:

- first live task `458ef0cf-bc46-413d-923e-758fe8df82c0` failed with
  `execution graph compile failed: Invalid topology contract: planner output contains duplicate numeric step reference '1'`
- fix: `normalize_runtime_plan(...)` now reindexes runtime steps deterministically instead of
  preserving planner-supplied numeric step labels for parallel branches

## Stop Conditions

- The runtime still depends on dedicated workflow threads after the change.
- Planner or executor calls still bypass supervised actor lifecycles in the live path.
- Executor completion still returns model strategy text instead of a ToolBus-produced step result.
- Validation exposes a provider-path regression or breaks the existing runtime proof shape.

# Short Multi-Agent Result Evaluation

Date: March 19, 2026
Status: Completed

## Objective

Run one short live task that explicitly asks Mister Smith to split work across multiple agents,
then evaluate two things from direct runtime evidence:

- whether multiple agents were actually used
- whether the task result was materially completed rather than merely accepted

## Environment Used

- repo: `/Users/macmain/MisterSmith`
- branch at start: `main...origin/main` with a clean tracked worktree
- runtime binary: `target/debug/mister-smith`
- runtime base URL: `http://127.0.0.1:62855`
- provider/model path: `openai_chatgpt` / `gpt-5.4`
- temporary database:
  `postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_short_eval_20260319`

## Live Task Used

Submitted through `POST /api/v1/tasks`:

> Create a tiny weekend plan. Split the work into two parallel tracks: one worker invents a
> morning activity, one worker invents an evening activity, then join the work into exactly three
> bullets where the third bullet is a one-sentence theme. Keep the full answer under 45 words.

## Identifiers Produced

- workflow/task id: `de579e73-0637-4ba0-958f-2f8809a68137`
- coordinator agent id: `02478cdf-f2ce-44e9-be34-45393c2b4eae`
- worker ids observed:
  - `e1ef6497-9265-436f-ad64-1dabc04e9228`
  - `3f3c4f4d-ea40-4d37-9ac0-090f3c4689d2`

## Artifacts Captured

All supporting evidence is under:

`docs/plans/artifacts/2026-03-19-short-multi-agent-result-evaluation/`

Key files:

- `runtime.log`
- `task-request.json`
- `task-submit.headers`
- `task-submit-response.json`
- `task-poll.log`
- `task-status-latest.json`
- `task-structural-summary.json`
- `autonomy-status.json`
- `autonomy-status-cli.txt`
- `evaluation-summary.txt`
- `task-submit-invalid.headers`
- `task-submit-invalid-response.txt`

## Observed Live Evidence

### The runtime used multiple agents

- `task-structural-summary.json` shows two distinct worker ids across the step results:
  - `e1ef6497-9265-436f-ad64-1dabc04e9228`
  - `3f3c4f4d-ea40-4d37-9ac0-090f3c4689d2`
- `autonomy-status.json` shows:
  - graph state `Completed`
  - `branch_count = 3`
  - `node_count = 4`
  - topology `Hybrid`
  - `parallelism_width = 2`
  - task shape `fanout-join`
  - routing history on the same two worker ids
- `runtime.log` shows the two root steps executing in parallel at the same timestamp:
  - `Invent morning activity`
  - `Invent evening activity`

### The workflow completed

- `task-submit.headers` shows `HTTP/1.1 202 Accepted`
- `task-submit-response.json` returned the workflow id and coordinator id with `status = queued`
- `task-poll.log` shows the task advancing from `running` to `completed`
- `task-status-latest.json` shows:
  - `status = completed`
  - `provider_kind = openai_chatgpt`
  - `model_id = gpt-5.4`
  - `step_results_count = 4`
  - runtime execution markers:
    - `workflow_runner = tokio_task`
    - `planner_lifecycle = supervised_actor`
    - `executor_lifecycle = supervised_actor`
    - `execution_boundary = tool_bus`
    - `tool_name = workflow.execute_step`
- `autonomy-status-cli.txt` renders the completed hybrid fanout-join graph on the CLI surface

### What the planner actually did

- `task-status-latest.json` shows the planner emitted a real multi-step execution plan:
  - `Invent morning activity`
  - `Invent evening activity`
  - `Join and compress outputs`
  - `Validate final constraints`
- the normalized execution plan explicitly required:
  - `require_parallel_workers = 2`
  - `require_real_multi_agent_workflow = true`

### Important limitation from this run

- the task path proved workflow completion, branching, and worker usage
- it did **not** persist or expose the final composed three-bullet answer on the inspected
  surfaces
- I checked:
  - `task-status-latest.json`
  - `autonomy-status.json`
  - `autonomy-status-cli.txt`
- all three surfaces preserved structural workflow evidence, but not the user-facing final text
- because of that, I can honestly say the workflow completed, but I cannot claim the final answer
  satisfied the exact three-bullet and under-45-word constraints from stored output alone

## Evaluation

### Were there multiple agents?

Yes. This run used at least two distinct worker ids in parallel, and the autonomy graph reported a
hybrid fanout-join topology with width `2`.

### Was the task completed?

Yes, at the workflow-execution level. The task reached terminal `completed`, all four planned
steps completed, and the runtime emitted the normal completion markers.

### Was the actual answer quality proved?

Not fully. The runtime proved multi-agent execution and completion, but the current inspected task
surface did not return the final composed bullet answer, so result-quality verification is still
incomplete on this path.

## Extra Finding

The first submit attempt returned `422 Unprocessable Entity` because `priority` expects a string,
not an integer. That is preserved in `task-submit-invalid.headers` and
`task-submit-invalid-response.txt`.

## Cleanup

- the temporary runtime was shut down after evidence capture
- the temporary database `mistersmith_short_eval_20260319` was dropped
- only note and artifact files were added; no code was changed

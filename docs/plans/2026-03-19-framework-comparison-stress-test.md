# Framework Comparison Stress Test

Date: March 19, 2026
Status: Completed

## Objective

Run a benchmark that is materially closer to Mister Smith's intended differentiators than a toy
task and preserve honest evidence for comparison:

- planner-led decomposition into parallel branches
- dependency-aware join behavior
- operator-visible topology and routing evidence
- failure visibility when the planner cannot carry the workload

## Benchmark Design

I ran two related self-contained incident-analysis tasks on the live `main` runtime path:

1. **Heavy benchmark**
   - four requested parallel tracks
   - explicit contradictions, hypotheses, mitigations, tests, and scorecard
   - intended to stress the planner and graph formation boundary
2. **Trimmed benchmark**
   - same domain and constraints, but shorter packet and three requested tracks
   - intended to find the practical completion boundary after the heavy case

## Environment Used

- repo: `/Users/macmain/MisterSmith`
- runtime base URL: `http://127.0.0.1:63083`
- runtime binary: `target/debug/mister-smith`
- provider/model path: `openai_chatgpt` / `gpt-5.4`
- temporary database:
  `postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/mistersmith_framework_stress_20260319`

## Identifiers Produced

- heavy benchmark workflow/task id: `a0528205-95e0-4c7b-a700-058814930e18`
- trimmed benchmark workflow/task id: `9425e03e-698c-4a76-b1b3-724772b56085`
- coordinator agent id for both submissions: `4fdd6c27-b45b-400e-84f2-612a22fd89a3`
- only worker id observed in the trimmed benchmark:
  `04b5065c-a0c0-4e53-855f-6a60c6947c14`

## Artifacts Captured

All supporting evidence is under:

`docs/plans/artifacts/2026-03-19-framework-comparison-stress-test/`

Key files:

- `runtime.log`
- `task-request.json`
- `task-submit.headers`
- `task-submit-response.json`
- `task-poll.log`
- `task-status-latest.json`
- `task-heavy-summary.json`
- `task-request-medium.json`
- `task-submit-medium.headers`
- `task-submit-medium-response.json`
- `task-poll-medium.log`
- `task-status-medium-latest.json`
- `task-medium-summary.json`
- `autonomy-status-medium.json`
- `autonomy-status-medium-cli.txt`
- `autonomy-workflows-after.json`

## Results

### Heavy benchmark: planner timed out before graph formation

- `task-status-latest.json` shows:
  - `status = failed`
  - `error = planner execution failed: Ask operation timed out`
- `task-poll.log` shows the task stayed `running` for about 53 seconds before failing
- `runtime.log` shows:
  - planner actor start
  - `Actor stop timed out, aborting task`
  - `Workflow run failed ... planner execution failed: Ask operation timed out`
- `autonomy-workflows-after.json` remained empty for this workflow
- `GET /api/v1/autonomy/status/{workflow_id}` returned no autonomy status

Interpretation:

- the system accepted the task and surfaced a clear failure boundary
- the planner did not finish decomposition
- no worker fanout or join graph was formed before failure

### Trimmed benchmark: completed, but collapsed to a single sequential step

- `task-status-medium-latest.json` shows:
  - `status = completed`
  - `step_results_count = 1`
  - only one worker id
- `task-medium-summary.json` shows the planner output ignored the requested multi-agent structure
  and emitted a single step:
  - `action = analyze`
  - one worker
  - no join step
  - `topology_hint = sequential`
- `autonomy-status-medium.json` shows:
  - graph state `Completed`
  - `branch_count = 1`
  - `node_count = 1`
  - topology `Sequential`
  - `parallelism_width = 1`
  - task shape `strict-chain`
- `autonomy-status-medium-cli.txt` renders the same single-branch sequential graph
- `runtime.log` shows one worker execution followed by `Workflow completed`

Interpretation:

- the trimmed benchmark completed successfully
- the planner did **not** honor the requested multi-agent decomposition
- the system fell back to a trivial single-worker plan instead of a meaningful fanout/join

## Did the agents communicate with each other?

Not in a peer-to-peer chat sense.

What Mister Smith showed in earlier successful multi-branch runs is coordination through:

- planner-produced branch structure
- dependency edges between steps
- join steps that consume dependency results
- operator-visible graph and routing state

In this stress test specifically:

- **heavy benchmark**: no, because the planner timed out before any worker graph existed
- **trimmed benchmark**: no, because the planner collapsed the work to one sequential step, so
  there was no inter-worker handoff to observe

So the honest answer is: Mister Smith's current observable coordination model is graph/dependency
based, not explicit worker-to-worker messaging, and this benchmark did not produce a successful
multi-worker communication case.

## Comparison Value

This is a good cross-framework comparison packet because it reveals three concrete dimensions:

1. **Ceiling under planning load**
   - the heavier incident packet failed at planner time with a surfaced timeout rather than silent
     hanging
2. **Constraint obedience**
   - the trimmed packet completed, but the planner ignored the explicit multi-agent decomposition
     requirement and collapsed to sequential work
3. **Operator visibility**
   - Mister Smith did expose the distinction cleanly:
     - failed before graph formation in the heavy case
     - completed sequential single-node graph in the trimmed case

If you compare this against other frameworks, the useful questions are:

- does the system fail clearly or opaquely at higher planning complexity?
- does it honor explicit parallelization constraints?
- when it refuses or collapses the plan, do the operator surfaces make that visible?
- can you distinguish planner failure from worker execution failure without digging into internals?

## Important Limitation

As with the earlier task-path runs, the inspected task/autonomy surfaces did not expose the final
user-facing memo text itself. They exposed structural execution evidence, planner output, topology,
routing, and terminal state. That means:

- execution behavior is directly comparable
- final answer quality is not fully comparable from these surfaces alone

## Bottom Line

For a comparison-oriented benchmark, Mister Smith currently shows:

- strong visibility into whether planning formed a graph or failed before graph creation
- strong visibility into whether a completed workflow was actually parallel or merely sequential
- weak evidence that explicit multi-agent decomposition requirements are enforced under more complex
  prompts
- a planner timeout boundary on the heavier stress case

## Cleanup

- the temporary runtime was shut down after evidence capture
- the temporary database `mistersmith_framework_stress_20260319` was dropped
- only note and artifact files were added; no code was changed

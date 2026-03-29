# 2026-03-28 Session Context Report

Date: 2026-03-28
Status: Durable session context and cold-start handoff note

## Objective

Capture the full context of this session in one repo-local artifact:

- what the user asked
- what the runtime and docs actually showed
- where the current operator/runtime story diverges from the user's expected orchestration model
- the exact evidence behind the most important claims
- what should happen next if this thread is resumed later

## Scope

This report covers the March 28, 2026 session focused on:

- roadmap and current-state clarification for Mister Smith end to end
- whether pub/sub is enabled and still relevant
- whether the original real-time multi-agent coordination vision is still intact
- why the operator console currently feels opaque
- detailed analysis of workflow `9891fb46-578d-4c41-8db3-699aa5298ed1`
- whether the current runtime path is "true orchestration" in the user's sense

## Constraints

- use repo-grounded evidence only
- avoid rewriting broader direction docs during this session
- do not disturb unrelated dirty worktree changes already present in the repo
- produce a standalone context artifact that another agent can read cold

## Non-Goals

- changing runtime semantics
- redesigning the operator console
- landing new instrumentation
- proving an alternate provider path

## Ambient Repo State

At report time, the primary worktree already contained unrelated modified and untracked docs. This
session deliberately avoided mutating those files and added only this standalone report.

## Session Questions

The user asked, in sequence:

1. what the roadmap for Mister Smith e2e is
2. whether pub/sub is enabled
3. whether the original real-time communication and coordination agent-system vision is still the
   case and still in use
4. how to visually trace a run, including agent thinking, communication, and tools
5. why the latest run appeared to complete in about 20-30 seconds when the same task would take a
   grounded manual audit much longer
6. what the "graph" actually is
7. whether the richer coordinator-subagent orchestration model was ever actually planned, or
   whether the current runtime graph layer was a necessary precursor
8. a request for this comprehensive context report

## High-Level Conclusions

### 1. The original richer multi-agent vision was real

The Phase 7 roadmap language explicitly described agent orchestration as the reason the system
exists, including team formation, task decomposition, agent communication through the transport
layer, and supervision. It also explicitly defined a Coordinator assigning subtasks to Workers via
NATS and aggregating results back.

Key references:

- [ROADMAP.md](/Users/macmain/MisterSmith/ROADMAP.md#L453)
- [ROADMAP.md](/Users/macmain/MisterSmith/ROADMAP.md#L489)
- [ROADMAP.md](/Users/macmain/MisterSmith/ROADMAP.md#L542)

### 2. The current live runtime path is narrower than that vision

The shipped runtime path is now centered on compiling a workflow graph, selecting a topology,
routing branches or steps, executing them, and projecting proof surfaces onto task/session/operator
views.

The repo's own current-state and March 27 simplification notes make this explicit:

- the runtime-backed task path is the live and proven path
- ToolBus-backed workflow step execution is part of that path
- the runtime now prefers the smallest workflow that can finish the task
- the live path defaults to sequential execution unless branching is clearly justified

Key references:

- [docs/current-state.md](/Users/macmain/MisterSmith/docs/current-state.md#L126)
- [docs/plans/2026-03-27-runtime-planning-simplification.md](/Users/macmain/MisterSmith/docs/plans/2026-03-27-runtime-planning-simplification.md#L38)

### 3. The current operator/runtime semantics blur two very different ideas

The repo currently uses "orchestration" to refer to the workflow-graph/runtime-control sense:

- plan decomposition
- topology selection
- branch and worker routing
- step execution
- verifier/repair decisions

This is not the same as the user's expected orchestration model:

- one lead agent actively steering subagents in real time
- explicit inter-agent communication during the run
- visible mid-flight course correction
- operator-visible subagent state and collaboration

The gap is semantic as much as it is UI.

## Meaning Of "Graph" In The Current Runtime

The graph is not a graph of agent-to-agent dialogue.

In current code, it is a canonical execution graph composed of:

- `ExecutionGraph`: workflow-level graph object
- `ExecutionBranch`: checkpointable lane of work
- `ExecutionNode`: one executable step with role, action, description, dependencies, budget, and
  metadata
- `ExecutionEdge`: dependency between nodes

Key references:

- [crates/mister-smith-agents/src/execution_graph.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/execution_graph.rs#L58)
- [crates/mister-smith-agents/src/execution_graph.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/execution_graph.rs#L96)
- [crates/mister-smith-agents/src/execution_graph.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/execution_graph.rs#L145)

The graph's shape is described through topology and task-shape metadata:

- `TopologyKind`: `Sequential`, `Parallel`, `Pipeline`, `Hierarchical`, `Hybrid`
- `CoordinationPolicy`: `StrictSequence`, `Barrier`, `Streaming`, `HierarchicalReduce`, `Mixed`
- `TaskShapeKind`: `StrictChain`, `ParallelFanout`, `FanoutJoin`, `HierarchicalFanout`,
  `MixedGraph`

Key references:

- [crates/mister-smith-core/src/enums.rs](/Users/macmain/MisterSmith/crates/mister-smith-core/src/enums.rs#L195)
- [crates/mister-smith-core/src/autonomy.rs](/Users/macmain/MisterSmith/crates/mister-smith-core/src/autonomy.rs#L24)

## How The Current Runtime Uses That Graph

At a high level, the current default path behaves like this:

1. planner turns a task into a structured plan
2. topology compiler classifies task shape and chooses a topology
3. orchestrator registers the execution graph and emits operator-visible graph/topology events
4. runtime sizes the active team and records routing history
5. executor runs the planned steps
6. verifier/repair logic can accept, reject, clarify, retry, re-plan, or stop at step boundaries

Key references:

- [crates/mister-smith-agents/src/orchestrator.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/orchestrator.rs#L280)
- [crates/mister-smith-agents/src/orchestrator.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/orchestrator.rs#L780)
- [docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md](/Users/macmain/MisterSmith/docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md#L7)

## Last Run Analysis

### Run Under Discussion

- workflow/task id: `9891fb46-578d-4c41-8db3-699aa5298ed1`
- screenshot-observed runtime: approximately `7:28:16 PM` to `7:28:48 PM` local time
- screenshot proof outcome: `graph_formed_and_completed`

### What The Runtime Reported

Task and autonomy inspection showed:

- `proof_outcome = graph_formed_and_completed`
- provider/model: `openai_chatgpt` / `gpt-5.4`
- execution boundary: `tool_bus`
- tool name: `workflow.execute_step`
- topology: `Hybrid`
- `parallelism_width = 3`
- `branch_count = 4`
- `node_count = 6`
- `selected_workers = 1`
- `available_workers = 2`
- `routing_history_count = 6`

The execution plan contained six high-level audit steps:

1. establish audit scope and inventory
2. audit frontend and UX surfaces
3. audit backend, API, and data boundaries
4. audit infrastructure, quality gates, and operational readiness
5. join findings and deduplicate issues
6. produce prioritized audit report

### Why The Run Finished So Fast

The critical finding from code inspection:

- the executor invokes `workflow.execute_step` through the ToolBus
- the registered `WorkflowStepTool` currently just echoes the step payload back with
  `status = completed`, `execution_boundary = tool_bus`, and the tool name

Key references:

- executor invokes ToolBus execution:
  [crates/mister-smith-agents/src/roles/executor.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/roles/executor.rs#L272)
- workflow step tool returns the payload as completed:
  [crates/mister-smith-app/src/execution.rs](/Users/macmain/MisterSmith/crates/mister-smith-app/src/execution.rs#L389)

This means the run was primarily proving:

- graph formation
- topology/routing metadata
- step handoff through the runtime execution boundary
- successful completion envelopes

It was not proving:

- real file-by-file repo inspection
- substantive tool execution against the target repo
- evidence-backed audit findings at the depth the user expected
- a visible lead-agent-subagent coordination loop

### Practical Interpretation

The runtime was honest at the orchestration-substrate layer but misleading at the semantic task
layer.

For this run, "completed" meant:

- the graph formed
- the graph ran through the current step-execution boundary
- each step came back marked complete

It did not mean:

- the requested audit was carried out with the same depth and grounded evidence a manual Codex-led
  investigation would produce

## Core User Concern Captured In This Session

The user's frustration is not just that the UI is sparse. The deeper issue is that the operator
surfaces currently project proof of workflow mechanics while sounding close to proof of meaningful
agent work.

The current system therefore makes it too easy to confuse:

- graph success with semantic task success
- branch routing with meaningful multi-agent collaboration
- workflow metadata with real agent communication
- step envelopes with grounded evidence

## Direct Answer To The User's Conceptual Question

The user's expected orchestration model is:

- a main coordinator agent
- active subagent delegation
- real-time communication among agents
- live steering and adjustment while work is in flight
- operator visibility into what each agent is doing and using

The current default runtime path is not yet that.

It is closer to:

- workflow compilation
- bounded adaptive topology selection
- branch routing
- step execution
- verifier-gated repair at step boundaries

That makes the current runtime graph layer a partial substrate for the original vision, but not the
finished expression of that vision.

## Was The Richer Model Ever Planned?

Yes.

Repo evidence shows the richer multi-agent coordination model was explicitly planned and partially
proven as an operating-system substrate:

- Coordinator decomposes tasks
- Workers communicate via NATS
- results aggregate back to the Coordinator
- supervision handles worker failure

Key reference:

- [ROADMAP.md](/Users/macmain/MisterSmith/ROADMAP.md#L542)

## Was The Current Runtime Layer A Necessary Step?

Also yes.

The current graph/topology/runtime layer provides real prerequisites for the richer model:

- stable execution structure
- typed routing and topology metadata
- branch-local checkpoints and recovery
- team sizing decisions
- verifier and repair control loops
- task/session/operator projections

But the repo's later bounded packets also clearly narrowed the live path:

- prefer the smallest workflow
- default to sequential
- branch only when clearly independent
- no decentralized topology rewrite or agent-graph overhaul in the current bounded packet

Key references:

- [docs/plans/2026-03-27-runtime-planning-simplification.md](/Users/macmain/MisterSmith/docs/plans/2026-03-27-runtime-planning-simplification.md#L38)
- [docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md](/Users/macmain/MisterSmith/docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md#L80)

So the best summary is:

- true multi-agent orchestration was planned
- the current runtime graph layer was a real prerequisite
- the shipped path then narrowed into a more proofable workflow engine
- that narrowing is why the user now perceives a mismatch between the original vision and the
  current operator/runtime experience

## Recommended Next Actions

### Product-Semantics Fixes

The system should explicitly distinguish:

- graph execution status
- semantic task completion status
- grounded evidence gathered
- stubbed versus real step execution
- routing metadata versus inter-agent communication

### Operator-Surface Fixes

The operator console should add a true run-trace view that surfaces:

- coordinator identity, if present
- worker identities and current assignments
- per-step tool invocations
- files/endpoints/artifacts touched
- verifier decisions and repair lineage
- explicit "no grounded evidence produced" warnings when the current path only returned envelopes

### Runtime Fixes

The `workflow.execute_step` path should either:

1. perform real grounded execution for the supported task class, or
2. mark the result as simulated/stubbed instead of semantically completed

Until that changes, the current operator wording should stay conservative.

## Recommended Language For Future Sessions

Use wording closer to:

- "workflow graph executed successfully"
- "semantic completion not yet proven"
- "grounded tool execution: none/minimal"
- "result is orchestration proof, not substantive task proof"

That phrasing would align user expectations with actual runtime behavior.

## Validation And Evidence Produced In This Session

Evidence sources used during the session:

- roadmap and current-state docs
- runtime task payload for workflow `9891fb46-578d-4c41-8db3-699aa5298ed1`
- autonomy status payload for the same workflow
- executor runtime code path
- workflow step tool implementation

Most important file references:

- [ROADMAP.md](/Users/macmain/MisterSmith/ROADMAP.md#L451)
- [docs/current-state.md](/Users/macmain/MisterSmith/docs/current-state.md#L126)
- [docs/plans/2026-03-27-runtime-planning-simplification.md](/Users/macmain/MisterSmith/docs/plans/2026-03-27-runtime-planning-simplification.md#L36)
- [crates/mister-smith-agents/src/execution_graph.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/execution_graph.rs#L145)
- [crates/mister-smith-agents/src/roles/executor.rs](/Users/macmain/MisterSmith/crates/mister-smith-agents/src/roles/executor.rs#L272)
- [crates/mister-smith-app/src/execution.rs](/Users/macmain/MisterSmith/crates/mister-smith-app/src/execution.rs#L389)

## Cold-Start Resume Note

If another agent resumes from this file, the next useful action is not another conceptual summary.
The next useful action is one of:

1. implement honest runtime/result labeling for stubbed step execution
2. design a run-trace operator surface that exposes real agent/tool/evidence activity
3. define a concrete path from the current workflow-graph runtime toward true coordinator-subagent
   orchestration semantics

Absent one of those, the same user confusion will recur.

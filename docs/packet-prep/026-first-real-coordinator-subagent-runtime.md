# Packet 026: First Real Coordinator-Subagent Runtime

## Packet Name

First real coordinator-subagent runtime

## Why This Packet Exists

The March 28 session report makes the gap plain:

- the original richer coordinator-worker vision was real
- the current live path is narrower and graph-centric
- the current graph is not visible coordinator-to-subagent collaboration

This packet exists to close that gap honestly. It should be the first packet that makes the live
runtime behave like a real coordinator-subagent system instead of only a compiled graph plus a
placeholder step boundary.

## Why This Stage Is Correct

This packet should not come first. It depends on earlier packets:

- `022` for durable workflow and lifecycle semantics
- `023` for truthful runtime trace and proof boundaries
- `024` for safe agent boundaries
- `025` for better step-level control signals

Once those exist, a first real coordinator-subagent runtime becomes honest and measurable.

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Live-default today:
  - the runtime already compiles execution graphs and can choose sequential, parallel, pipeline,
    hierarchical, and hybrid shapes
  - same-agent session continuity already preserves `session_id` and `coordinator_agent_id`
  - operator-facing status already shows topology, provenance, and supervision summaries
- Landed but not yet a real coordinator-subagent runtime:
  - explicit merge steps, topology rationale, and team-sizing decisions already exist
  - branch routing and checkpoint lineage already exist in the orchestration substrate
- Missing for this packet:
  - visible coordinator-owned delegation records
  - real subagent execution state and feedback loops during a run
  - grounded branch execution below the current placeholder step boundary
  - explicit reuse of the frozen contracts from packets `022` through `025`

## Current Repo Grounding

### Live on the default runtime path now

- execution graphs, topology kinds, and coordination policies
- supervised planner and executor lifecycles
- team-sizing decisions and topology rationale surfaces
- same-agent session continuity and coordinator IDs
- operator-visible topology, provenance, and supervision evidence
- the current runtime proves orchestration shape and routing metadata, but not yet grounded
  coordinator-to-subagent work below the placeholder step boundary

### Landed in repo but not yet a real coordinator-subagent runtime

- the runtime already compiles sequential, parallel, pipeline, hierarchical, and hybrid shapes
- explicit coordinator merge steps can already appear in the execution graph
- the live planning path already prefers the smallest workflow that fits the task

### Missing pieces

- durable, visible coordinator-to-subagent task delegation and feedback loops as real runtime work
- explicit subagent state, communication, and course-correction surfaces mid-run
- grounded subagent execution instead of the current placeholder `workflow.execute_step` result
- honest proof criteria for "real coordinator-subagent runtime"

### High-Signal Repo Anchors

- `crates/mister-smith-agents/src/execution_graph.rs`
  - `ExecutionBranch`
  - `ExecutionNode`
  - `ExecutionGraph`
  - `BranchCheckpoint`
  - This is the current topology and branch-state substrate.
- `crates/mister-smith-agents/src/orchestrator.rs`
  - `register_execution_graph`
  - topology selection, branch routing, and supervision-evidence builders
  - This is the current coordinator-like runtime control surface.
- `crates/mister-smith-core/src/enums.rs`
  - `AgentType`
  - This is the current role taxonomy that proves agent architecture exists even though the full
    packet outcome does not yet.
- `crates/mister-smith-agents/src/roles/coordinator.rs`
  - `CoordinatorAgent`
  - This is the clearest current coordinator role seam.
- `crates/mister-smith-agents/src/roles/executor.rs`
  - `ExecutorAgent::with_tool_bus`
  - This is the current handoff from role runtime into the placeholder execution boundary.
- `crates/mister-smith-agents/src/team.rs`
  - `Team::new`
  - This is the strongest current team-shape and role-composition seam.
- `crates/mister-smith-core/src/autonomy.rs`
  - `TopologyPlan`
  - `TeamSizingDecision`
  - `TaskShapeClassification`
  - This is the current operator-facing graph and topology contract.
- `crates/mister-smith-agents/src/topology.rs`
  - `classify_task_shape`
  - `build_topology_plan`
  - This is the strongest current task-shape and topology-planning seam.
- `crates/mister-smith-app/src/execution.rs`
  - `WorkflowStepTool`
  - `impl Tool for WorkflowStepTool`
  - runtime execution entry points and current placeholder branch completion boundary
  - This is the main reason the packet still is not a real subagent runtime.
- `crates/mister-smith-app/src/conversation.rs`
  - `ConversationRuntimeService`
  - turn/session continuity builders
  - This matters because follow-up coordinator work must preserve current session semantics.
- `docs/2026-03-28-session-context-report.md`
  - This is the clearest narrative note separating today's graph substrate from the richer older
    coordinator-worker vision.
- `ROADMAP.md`
  - Phase 7 coordinator-worker language and task-decomposition intent
  - Use this only to recover the original product intent. `docs/current-state.md` still wins for
    what is live now.
- `docs/plans/2026-03-27-runtime-planning-simplification.md`
  - This is the current smallest-workflow baseline that packet `026` must preserve instead of
    regressing to mandatory fan-out.

## Comparator Docs

- [OpenAI Agents SDK: Agent Orchestration](https://openai.github.io/openai-agents-js/guides/multi-agent/)  
  Why it matters: useful official comparator for handoffs, streamed run items, and multi-agent
  orchestration surfaces. Treat as comparator guidance, not as the Mister Smith contract.
- [Google ADK: Workflow Agents](https://google.github.io/adk-docs/agents/workflow-agents/)  
  Why it matters: good official reference for deterministic sequential, parallel, and loop control
  structures. Treat as comparator guidance, not as the Mister Smith contract.
- [Google ADK: Multi-Agent Systems](https://google.github.io/adk-docs/agents/multi-agents/)  
  Why it matters: good official reference for composing specialized agents under explicit roles.
  Treat as comparator guidance, not as the Mister Smith contract.
- [LangGraph supervisor docs](https://langchain-ai.github.io/langgraphjs/reference/modules/langgraph-supervisor.html)  
  Why it matters: useful official comparator for hierarchical supervisor patterns, memory, and
  streaming. Treat as comparator guidance, not as the Mister Smith contract.

## Research Findings That Matter

- The dynamic-orchestration transfer brief supports local-first hybrid control, not centralized
  orchestration theater.
- The session context report says the current graph layer is a precursor, not the richer
  coordinator-subagent runtime itself.
- The repo research also says more agents only help when structure justifies them. This packet
  should make real coordination visible, but it should not force branching where the task is better
  kept sequential.

## Best-Practice Guidance

- Treat the current graph and role substrate as foundation truth, not as proof that packet `026`
  is already materially achieved.
- Keep the coordinator responsible for delegation, proof boundaries, and merge criteria.
- Make subagent states and boundary events visible to the operator.
- Do not spawn subagents just to mimic collaboration. Each subagent should own a real bounded job.
- Preserve the smallest-workflow rule. Real coordinator-runtime should not mean mandatory fan-out.
- Make mid-run corrections explicit: reassign, clarify, merge, stop, or collapse.
- Do not write a packet-026 spec that claims grounded coordinator-subagent proof while
  `workflow.execute_step` is still the active placeholder execution boundary on `main`.

## Likely Architecture Shape

- real coordinator-owned delegation records and per-subagent execution state
- operator-visible branch, subagent, and handoff timeline
- grounded step execution underneath delegated work
- structured merge semantics for coordinator reassembly and escalation
- session-aware continuity for coordinator-led follow-ups

## Risks / Constraints / Non-Goals

- Do not confuse visible branching with real grounded multi-agent execution.
- Do not widen this packet into general federation or capability-discovery work.
- Do not revert to fixed two-worker-plus-merge shaping.
- Do not use this packet to bypass proof-boundary honesty from packet `023`.

## Open Questions Before Spec Writing

- What is the minimum runtime proof standard for calling a run "real coordinator-subagent"?
- Which delegation events must be first-class operator-visible surfaces?
- How should the coordinator own merge semantics and partial failure recovery?
- When should the system collapse back to sequential execution?
- How much session continuity should transfer across coordinator-led follow-up runs?

## Fixed Constraints Before Spec Writing

- Consume packet `022` through `025` contracts instead of redefining lifecycle, proof-boundary,
  security, or step-policy ownership here.
- Preserve the smallest-workflow rule. Real coordinator-runtime must still be able to collapse
  back to sequential execution honestly.
- Require visible coordinator-owned delegation records and real subagent state before claiming
  "coordinator-subagent runtime."
- Keep interoperability and federation work out of this packet.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> packets `022` through `025` -> session-context gap note ->
current graph and runtime seams -> external comparator docs.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/packet-prep/022-durable-workflow-core.md`
  - use to inherit durable workflow and lifecycle assumptions instead of redefining them here
- `docs/packet-prep/023-runtime-truth-and-run-trace.md`
  - use to inherit run-trace and proof-boundary contracts instead of moving them into packet `026`
- `docs/packet-prep/024-agent-boundary-security-hardening.md`
  - use to inherit least-privilege and quarantine constraints before widening delegation
- `docs/packet-prep/025-step-level-intelligence-v2.md`
  - use to inherit step-policy and escalation semantics before adding visible subagent state
- `docs/2026-03-28-session-context-report.md`
- `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
  - use as historical evidence for what older multi-agent proof claims did and did not prove on
    the live path
- `ROADMAP.md`
  - use only to recover the original coordinator-worker product intent; do not treat it as current
    runtime truth
- `docs/research-output/analysis/2026-03-28-dynamic-orchestration-transfer-brief.md`
- `docs/plans/2026-03-27-runtime-planning-simplification.md`
- `crates/mister-smith-core/src/enums.rs`
  - start from `AgentType`
- `crates/mister-smith-core/src/autonomy.rs`
  - start from `TopologyPlan`, `TeamSizingDecision`, and `TaskShapeClassification`
- `crates/mister-smith-agents/src/topology.rs`
  - start from `classify_task_shape` and `build_topology_plan`
- `crates/mister-smith-agents/src/orchestrator.rs`
  - start from `register_execution_graph`, branch routing, team sizing, and
    supervision-evidence builders
- `crates/mister-smith-agents/src/execution_graph.rs`
  - start from `ExecutionGraph`, `ExecutionBranch`, and `BranchCheckpoint`
- `crates/mister-smith-agents/src/roles/coordinator.rs`
  - start from `CoordinatorAgent`
- `crates/mister-smith-agents/src/roles/executor.rs`
  - start from `ExecutorAgent::with_tool_bus`
- `crates/mister-smith-agents/src/team.rs`
  - start from `Team::new`
- `crates/mister-smith-app/src/execution.rs`
  - start from `WorkflowStepTool`, `impl Tool for WorkflowStepTool`, and current runtime
    execution entry points
  - if this seam is still placeholder-only on current `main`, keep the packet scoped to the first
    honest grounded delegation slice instead of overclaiming full multi-agent proof
- `crates/mister-smith-app/src/conversation.rs`
  - start from `ConversationRuntimeService` and session-turn continuity
- only after the upstream packet constraints are clear, re-confirm the official comparator docs
  linked earlier

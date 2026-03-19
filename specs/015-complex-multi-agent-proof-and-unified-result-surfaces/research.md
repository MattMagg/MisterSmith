# Research: Complex Multi-Agent Proof and Unified Result Surfaces

**Date**: 2026-03-19  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

The March 19 checkpoint already narrows the next product gap: the repo has more frontier substrate
than the default live path currently proves, and the strongest remaining gap is honest complex
multi-agent proof under harder workloads.

The result side of that gap is subtler than “final answer support is missing.” Result plumbing
already exists on `main`, but it is not yet governed by one shared contract or projected
consistently across task, session, and operator surfaces.

The strongest repo-local conclusion is therefore:

- treat the current supervised planner and executor live path, `tool_bus` boundary, topology and
  routing visibility, bounded MCP capability surface, and existing result plumbing as baseline
- freeze one shared result contract before future execution work starts
- prove three runtime outcomes honestly on the default path:
  - `graph_formed_and_completed`
  - `collapsed_to_sequential`
  - `failed_before_graph`
- defer provider, KV, budget, and broader external-agent work unless a narrow non-regression check
  is required

## Current Repo Findings That Shape The Design

### Already Exists In Code

#### R1: The March 19 checkpoint already fixes the next epic shape

**Sources**:

- `docs/plans/2026-03-19-central-development-checkpoint.md`
- `docs/current-state.md`

**Evidence**:

- the checkpoint says the next SpecKit packet should cover the unified contract for complex
  multi-agent graph execution, final result visibility on runtime and operator surfaces, repeatable
  benchmark and evaluation proof, and an explicit post-`MS-77` scope decision
- current-state says the live path already proves one-shot task execution, autonomy inspection,
  bounded same-agent sessions, and supervised planner and executor lifecycles on the default path

**Decision**: the new packet should extend the current live path rather than inventing a new
runtime or revive the old Smith-first development program.

#### R2: Result plumbing already exists on the task path

**Sources**:

- `crates/mister-smith-app/src/execution.rs`
- `docs/plans/2026-03-19-live-run-trace-evaluation.md`

**Evidence**:

- the runtime already computes `aggregated_result`
- the runtime already builds `final_result` with `workflow_id`, provider and model markers,
  `planner_output`, `execution_plan`, `step_results`, and nested `aggregated_result`
- `update_root_record(..., Some(final_result.clone()), ...)` already persists that object into
  `task.result`
- the live-run trace note shows task status exposes terminal result markers and structural
  execution proof

**Decision**: the packet must not claim final-result material is absent. It must instead govern
  how the existing result object becomes the canonical contract.

#### R3: Result retention already exists in session context

**Sources**:

- `crates/mister-smith-app/src/conversation.rs`

**Evidence**:

- completed turns already copy `task.result` into `turn.result_summary`
- retained session context already stores `last_assistant_result`
- retained transcript entries already store `assistant_result`

**Decision**: session work should be framed as consistent projection of the canonical result
  object, not as invention of a new result feature.

#### R4: Structural operator proof is already strong

**Sources**:

- `crates/mister-smith-app/src/autonomy.rs`
- `docs/plans/2026-03-19-live-run-trace-evaluation.md`
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`

**Evidence**:

- autonomy rendering already exposes topology, routing, checkpoints, delegation, and external
  capability decisions
- the live-run trace note shows that operator surfaces already distinguish graph formation,
  execution mode, and completion markers
- `MS-77` already closed bounded external capability discovery and enforcement on the MCP surface

**Decision**: operator work should extend the current autonomy surface with bounded result preview
  and provenance, not introduce a new status subsystem.

### Not Yet First-Class Or Not Yet Projected Consistently

#### R5: The result forms do not yet have one explicit contract

**Sources**:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-app/src/autonomy.rs`

**Evidence**:

- `final_result` exists, but its relationship to `task.result` is implied by assignment rather
  than defined as a shared contract
- `aggregated_result` exists as a nested payload, but there is no packet-level rule that it is not
  a competing top-level result shape
- session retained context stores `assistant_result`, but the relationship between this session
  projection and the canonical runtime result object is not frozen anywhere
- autonomy rendering exposes strong structure but no compact result preview or result provenance

**Decision**: make the shared result contract the first blocking checkpoint in the packet.

#### R6: The harder-workload proof gap is about outcomes, not just more workers

**Sources**:

- `docs/plans/2026-03-19-short-multi-agent-result-evaluation.md`
- `docs/plans/2026-03-19-framework-comparison-stress-test.md`
- `docs/plans/2026-03-19-live-run-trace-evaluation.md`

**Evidence**:

- the short multi-agent evaluation proved a real multi-agent fanout-join run but could not verify
  the final user-facing answer from the inspected surfaces alone
- the framework stress note showed three important boundaries:
  - planner timed out before graph formation on the heavier benchmark
  - trimmed benchmark completed but collapsed to one sequential step
  - structural execution evidence was visible even when answer-quality proof was incomplete
- the live-run trace proved the current runtime path and structural markers are real

**Decision**: the packet must define a proof matrix covering success, collapse, and failure-visible
  behavior instead of anchoring scope to a particular worker count.

### Explicitly Deferred

#### R7: Broader external-agent work is a later bounded epic unless result-surface changes intersect it

**Sources**:

- `docs/plans/2026-03-19-ms-48-closure-audit.md`
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`

**Evidence**:

- `MS-48` closure audit already isolated the remaining external-agent follow-up into the bounded
  `MS-77` lane rather than the main product program
- `MS-77` already delivered one bounded external-agent surface on the MCP boundary with discovery
  and enforcement

**Decision**: keep broader external-agent expansion out of this packet. Only require an MCP
  non-regression check if future result-surface changes intersect the existing bounded surface.

#### R8: Provider, budget, and JetStream KV work remain deferred

**Sources**:

- `docs/current-state.md`
- `docs/plans/2026-03-19-central-development-checkpoint.md`

**Evidence**:

- current-state and the checkpoint both keep provider-neutral routing, budget-backed control, and
  additive external surfaces outside the default live proof path
- the checkpoint explicitly defers broader frontier work until separately spec'd

**Decision**: defer provider-neutral routing, KV, and budget follow-up entirely in this packet.

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `docs/plans/2026-03-19-central-development-checkpoint.md` | Defines the next packet scope and the post-`MS-77` decision point. |
| `docs/current-state.md` | Separates current live-path truth from broader landed substrate. |
| `docs/plans/2026-03-19-live-run-trace-evaluation.md` | Proves the live path, execution markers, and current task/autonomy visibility. |
| `docs/plans/2026-03-19-short-multi-agent-result-evaluation.md` | Shows successful multi-agent structure and the current result-verification gap. |
| `docs/plans/2026-03-19-framework-comparison-stress-test.md` | Shows success, collapse, and failure-visible boundaries under harder workloads. |
| `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md` | Shows the bounded external capability surface that should remain baseline. |
| `docs/plans/2026-03-19-ms-48-closure-audit.md` | Confirms broader external-agent work should remain a later bounded epic. |
| `crates/mister-smith-app/src/execution.rs` | Shows `final_result`, nested `aggregated_result`, and task-result persistence. |
| `crates/mister-smith-app/src/conversation.rs` | Shows retained `assistant_result` and session result-summary storage. |
| `crates/mister-smith-app/src/autonomy.rs` | Shows the current operator rendering surface and the lack of result preview. |

## Explicitly Deferred Questions

- whether the default live path should eventually use provider-neutral routing and budget-backed
  control loops by default
- whether the result preview should eventually include configurable payload expansion beyond a
  bounded summary
- whether future post-`MS-77` work should expose additional external-agent transports after the
  bounded MCP surface
- whether future live proof should expand beyond the current harder-workload matrix into a broader
  performance program

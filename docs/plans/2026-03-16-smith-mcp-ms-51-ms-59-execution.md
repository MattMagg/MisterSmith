# Smith MCP MS-51 Through MS-59 Execution

Date: March 16, 2026
Status: Active

## Objective

Extend Smith from a read-heavy control plane plus issue and workpad mutation helpers into the main
workflow router for Mister Smith development.

## Scope

- `MS-51`: backlog slicing and child-issue creation
- `MS-52`: honest queue staging and watched-queue reconciliation
- `MS-53`: execution inspection, recovery, review, and merge-dispatch control
- `MS-54`: Ralph packet generation from issue and workpad state
- `MS-55`: Ralph outcome recording back into the durable workpad path
- `MS-56`: SpecKit routing decision support
- `MS-57`: SpecKit task-pack translation into bounded backlog slices
- `MS-58`: planning and implementation proof coordinators
- `MS-59`: review, Ralph, and SpecKit proof coordinators

## Constraints

- Linear remains the durable source of truth for issues, blockers, queue placement, and workpads.
- Symphony remains the unattended executor for the single watched Linear project.
- Ralph remains a loop runner driven by regenerated prompt packets.
- SpecKit remains the upstream spec and task-pack scaffold.
- Preserve existing Smith tool names and add workflow-family tools only where the current surface is
  missing a reusable control-plane capability.
- Reuse `save_linear_issue` and `save_issue_workpad` as the only Linear write primitives.

## Non-Goals

- replacing Linear, Symphony, Ralph, or SpecKit with Smith-owned copies
- broad workspace taxonomy cleanup outside the needs of this workflow family
- staging broad umbrella issues such as `MS-58` or `MS-59` directly into the watched queue

## Current Implementation Status

Implemented in the current repo:

- backlog and queue kernel tools:
  `materialize_backlog_slices`, `plan_queue_stage`, `apply_queue_stage`,
  `resolve_issue_lifecycle`
- Ralph and SpecKit workflow-family tools:
  `prepare_ralph_packet`, `record_ralph_outcome`, `prepare_speckit_context`,
  `translate_speckit_tasks`
- shared queue and lifecycle reuse across:
  `get_issue_execution_snapshot`, `sync_linear_with_runtime`, and
  `review_merge_dispatch_cycle`
- deterministic coverage for the new workflow families plus live read-side proofs for queue-stage
  planning and Ralph packet preparation

Still pending before claiming end-to-end workflow closure:

- live apply proof for backlog slicing and Ralph outcome recording
- real watched-queue staging through `apply_queue_stage`
- real Symphony pickup and return-path evidence for one safe proof slice

## Milestones

### Milestone 1: Backlog And Queue Kernel

Implement additive Smith tools and shared evaluators for:

- backlog slice materialization
- honest queue-stage planning and application
- issue lifecycle resolution
- richer issue execution snapshots
- shared queue and lifecycle signals reused by review and runtime sync tools

Validation:

- targeted `cargo build -p mister-smith-mcp`
- targeted `cargo test -p mister-smith-mcp`
- live Smith/Linear proof for at least one slice creation or queue-stage planning path

### Milestone 2: Ralph And SpecKit Integration

Implement additive Smith tools for:

- Ralph packet preparation and outcome recording
- SpecKit routing decisions
- task-pack translation into bounded backlog slices

Validation:

- targeted `cargo build -p mister-smith-mcp`
- targeted `cargo test -p mister-smith-mcp`
- live Smith proof for one Ralph packet and one SpecKit task translation path

### Milestone 3: Real Queue Proof Affordances

Use the new Smith workflow families to support:

- proof child-slice creation
- honest watched-queue staging
- durable workpad continuity across planning, execution, and return paths

Validation:

- real queue-stage planning against the watched project
- one safe proof slice staged through Smith-owned queue controls
- evidence that the staged slice remains compatible with Symphony dispatch rules

## Assumptions

- `MS-50` remains the prerequisite kernel for direct Linear issue and workpad mutation.
- The watched Linear project slug remains `320a0741920c`.
- Proof posture is real queue, not local-only dry runs.

## Stop Conditions

- stop before claiming Smith owns the workflow if queue staging still bypasses Smith
- stop before adding another write path to Linear or workpads
- stop before staging unsafe or externally risky proof slices into the watched queue

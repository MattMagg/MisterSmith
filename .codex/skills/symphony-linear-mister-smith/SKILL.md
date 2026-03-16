---
name: symphony-linear-mister-smith
description: Use when a Mister Smith task spans Symphony runtime, Linear control-plane state, GitHub PR flow, local repo truth, queue triage, runtime reconciliation, or workspace hygiene.
---

# Symphony Linear Mister Smith

Use the `smith` MCP tools first for combined Symphony, Linear, GitHub, and repo operations.

## Primary tools

- `route_workflow_request`
- `get_control_plane_snapshot`
- `get_issue_execution_snapshot`
- `resolve_issue_lifecycle`
- `get_symphony_checkout_snapshot`
- `plan_workspace_adjustments`
- `sync_linear_with_runtime`
- `plan_queue_stage`
- `apply_queue_stage`
- `refresh_symphony`
- `sync_symphony_main`

## Rules

- Snapshot first, mutate second.
- Use `save_linear_issue` and `save_issue_workpad` as the only Smith-owned Linear write path.
- Use `materialize_backlog_slices`, `plan_queue_stage`, and `apply_queue_stage` for backlog and watched-queue moves.
- Use `prepare_ralph_packet`, `record_ralph_outcome`, `prepare_speckit_context`, and `translate_speckit_tasks` when the task crosses Ralph or SpecKit boundaries.
- Prefer Smith workflow tools over raw GraphQL or ad hoc shell commands when the operation is already modeled.
- Only use the repo-local `linear` skill for uncovered Linear gaps.
- When the task spans multiple development workflow surfaces, use `docs/plans/2026-03-16-smith-first-development-system.md` as the repo-local integration model.

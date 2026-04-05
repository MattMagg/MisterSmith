---
name: symphony-linear-mister-smith
description: Legacy-named skill for Mister Smith tasks that span Linear control-plane state, GitHub PR flow, local repo truth, lifecycle recovery, or workspace hygiene.
---

# Symphony Linear Mister Smith

Use the `smith` MCP tools first for direct Codex control-plane operations. Symphony is not part of
the default Smith MCP route.

## Primary tools

- `route_workflow_request`
- `get_control_plane_snapshot`
- `get_issue_execution_snapshot`
- `prepare_direct_execution`
- `resolve_issue_lifecycle`
- `review_merge_status`
- `plan_workspace_adjustments`
- `materialize_backlog_slices`
- `prepare_ralph_packet`
- `record_ralph_outcome`
- `prepare_speckit_context`
- `translate_speckit_tasks`

## Rules

- Snapshot first, mutate second.
- Use `save_linear_issue` and `save_issue_workpad` as the only Smith-owned Linear write path.
- Use `prepare_direct_execution` before implementation when the task needs a runnable plan.
- Use `materialize_backlog_slices` for direct backlog decomposition, not queue staging.
- Use `prepare_ralph_packet`, `record_ralph_outcome`, `prepare_speckit_context`, and
  `translate_speckit_tasks` when the task crosses Ralph or SpecKit boundaries.
- Prefer Smith workflow tools over raw GraphQL or ad hoc shell commands when the operation is already modeled.
- Only use the repo-local `linear` skill for uncovered Linear gaps.
- When the task spans multiple development workflow surfaces, use
  `docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md` as the direct-control-plane note.

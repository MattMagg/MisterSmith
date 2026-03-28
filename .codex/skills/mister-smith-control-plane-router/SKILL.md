---
name: mister-smith-control-plane-router
description: Use when any Mister Smith request touches Symphony, Linear, GitHub PR flow, phase slicing, queue dispatch, runtime reconciliation, workspace hygiene, or bootstrap readiness and needs the correct control-plane route first.
---

# Mister Smith Control-Plane Router

Use the `smith` MCP tools first for any Mister Smith workflow request.

## Primary route

1. Call `route_workflow_request` with the operator request.
2. Call `get_control_plane_snapshot` when the route needs current repo, PR, Linear, or runtime evidence.
3. For broad workflow-architecture or Smith-first development-system requests, read `docs/plans/2026-03-16-smith-first-development-system.md` after routing.
4. Use the routed workflow family before reaching for any raw fallback skills:
   - `linear_workflow` -> `save_linear_issue`, `save_issue_workpad`, `get_issue_execution_snapshot`
   - `backlog_slicing` -> `materialize_backlog_slices`, `translate_speckit_tasks`, `plan_queue_stage`
   - `issue_lifecycle` -> `resolve_issue_lifecycle`, `get_issue_execution_snapshot`, `plan_queue_stage`
   - `development_workflow` -> `get_control_plane_snapshot`, `sync_linear_with_runtime`, `evaluate_issue_legitimacy`
   - `review_dispatch` -> `review_merge_dispatch_cycle`
5. For frozen packet implementation in Mister Smith, stop after Smith-first routing only long
   enough to reconcile runnable state, then explicitly execute the repo-local `speckit.implement`
   flow before code changes.

## Fallback

- Use the repo-local `linear` skill only when the Smith MCP does not expose the needed Linear mutation, queue action, or query.
- Do not use non-Mister-Smith app workflows when the Smith MCP already covers the operation.

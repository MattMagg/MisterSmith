---
name: mister-smith-control-plane-router
description: Use when any Mister Smith request touches Symphony, Linear, GitHub PR flow, phase slicing, queue dispatch, runtime reconciliation, workspace hygiene, or bootstrap readiness and needs the correct control-plane route first.
---

# Mister Smith Control-Plane Router

Use the `smith` MCP tools first for any Mister Smith workflow request.

## Primary route

1. Call `route_workflow_request` with the operator request.
2. Call `get_control_plane_snapshot` when the route needs current repo, PR, Linear, or runtime evidence.
3. Follow the recommended Smith tool chain before reaching for any raw fallback skills.

## Fallback

- Use the repo-local `linear` skill only when the Smith MCP does not expose the needed Linear mutation or query.
- Do not use non-Mister-Smith app workflows when the Smith MCP already covers the operation.

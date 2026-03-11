---
name: symphony-linear-mister-smith
description: Use when a Mister Smith task spans Symphony runtime, Linear control-plane state, GitHub PR flow, local repo truth, queue triage, runtime reconciliation, or workspace hygiene.
---

# Symphony Linear Mister Smith

Use the `smith` MCP tools first for combined Symphony, Linear, GitHub, and repo operations.

## Primary tools

- `get_control_plane_snapshot`
- `get_symphony_checkout_snapshot`
- `plan_workspace_adjustments`
- `sync_linear_with_runtime`
- `refresh_symphony`
- `sync_symphony_main`

## Rules

- Snapshot first, mutate second.
- Prefer Smith workflow tools over raw GraphQL or ad hoc shell commands when the operation is already modeled.
- Only use the repo-local `linear` skill for uncovered Linear gaps.

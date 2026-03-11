---
name: mister-smith-control-plane-bootstrap
description: Use when the Mister Smith control-plane MCP or compatibility skill shims need to be installed, repaired, re-pointed, or audited in the local Codex environment.
---

# Mister Smith Control-Plane Bootstrap

Use the `smith` MCP tools first when checking bootstrap and readiness.

## Workflow

1. Call `audit_workflow_readiness`.
2. If repo-local canonical skills are missing, run `python3 scripts/bootstrap_control_plane.py` from the Mister Smith repo.
3. If readiness still fails, fix the reported checks before continuing.

## Notes

- Accept either `smith` or `mistersmith_control_plane` as the configured MCP server name.
- Treat repo-local canonical skills as the authoritative skill pack for this repository.

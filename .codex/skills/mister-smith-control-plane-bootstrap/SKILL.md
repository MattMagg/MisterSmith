---
name: mister-smith-control-plane-bootstrap
description: Use when the Mister Smith control-plane MCP or compatibility skill shims need to be installed, repaired, re-pointed, or audited in the local Codex environment.
---

# Mister Smith Control Plane Bootstrap

## Overview

This skill manages the local Codex registration for the constitutional control-plane MCP and the
legacy home-level skill shims. The repo-local skill pack is canonical; the home-level skills exist
only for transition compatibility.

## Use This When

- `audit_workflow_readiness` reports missing bootstrap state
- `~/.codex/config.toml` does not contain `mistersmith_control_plane`
- the home-level `symphony-*` or `mister-smith-frontier-mandate` skills still need to point into this repo
- the control-plane repo moved or was reinstalled

## Procedure

1. Verify the control-plane repo exists at `/Users/matthewmaggio/Repos/mister-smith-constitutional-control-plane` or pass an override path.
1. Dry-run the bootstrap manager:

```bash
python3 scripts/bootstrap_control_plane.py --dry-run
```

1. Apply it when the report looks correct:

```bash
python3 scripts/bootstrap_control_plane.py
```

1. Re-run `audit_workflow_readiness`.

## What It Owns

- managed `mistersmith_control_plane` MCP block in `~/.codex/config.toml`
- legacy home-level shims for:
  - `symphony-linear-mister-smith`
  - `stage-mister-smith-phase`
  - `symphony-mister-smith-review-dispatch`
  - `mister-smith-frontier-mandate`

## Rules

- do not hand-edit the managed MCP block unless you are also updating the bootstrap script
- do not treat the home-level shims as canonical docs
- keep `stdio` as the default MCP transport; use HTTP only for manual health checks or smoke runs

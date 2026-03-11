---
name: stage-mister-smith-phase
description: Use when a Mister Smith SpecKit phase or tasks pack needs to be turned into deterministic Linear slices, blocker chains, prep-slice opportunities, and runnable watched-queue work.
---

# Stage Mister Smith Phase

## Overview

This skill is the phase-planning wrapper around the control-plane MCP. The MCP owns the slice
derivation and staging logic; this skill provides the repo-specific operating rules and prompt
shape.

## Use This When

- breaking a new phase into Linear issues
- checking whether a phase is already partially staged
- looking for honest prep slices on blocked work
- staging only the runnable part of a phase into the watched queue

## MCP Tool Chain

1. `plan_phase_execution`
2. `evaluate_issue_legitimacy` for disputed slices
3. `apply_phase_execution_plan`
4. `get_control_plane_snapshot` or `sync_linear_with_runtime` if runtime/queue reconciliation is needed after staging

## Rules

- keep `tasks.md` as the slicing source of truth
- preserve blocker chains instead of flattening them into one giant issue
- create prep slices only when the work is independently valuable and honestly runnable now
- keep blocked/future slices in `MisterSmith Validated Backlog`
- stage only the truly runnable slices into the watched queue

## Prompt Examples

- `Use $stage-mister-smith-phase to plan 012-phase10-frontier-autonomy and stage only the runnable slices.`
- `Use $stage-mister-smith-phase to plan the next phase and split real prep slices out of blocked work where appropriate.`

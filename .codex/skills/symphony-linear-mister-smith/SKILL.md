---
name: symphony-linear-mister-smith
description: Use when a Mister Smith task spans Symphony runtime, Linear control-plane state, GitHub PR flow, local repo truth, queue triage, runtime reconciliation, or workspace hygiene.
---

# Symphony Linear Mister Smith

## Overview

This is the broad operational wrapper for the Mister Smith control plane. Use the control-plane MCP
first, not ad hoc shell-plus-Linear hopping.

## Use This When

- inspecting repo, Linear, GitHub, and Symphony state together
- deciding what should run next or why the queue is idle
- reconciling Linear against runtime truth
- auditing workspace hygiene before making queue or project changes
- operating the watched queue outside of phase-slicing or review-dispatch special cases

## MCP-First Flow

1. Start with `get_control_plane_snapshot`.
2. Use `route_workflow_request` if the task is ambiguous.
3. Use these primary tools:
   - `assess_runnable_candidates`
   - `get_queue_capacity`
   - `sync_linear_with_runtime`
   - `refresh_symphony`
   - `get_symphony_checkout_snapshot`
   - `sync_symphony_main`
   - `plan_workspace_adjustments`
4. Use the repo-local `linear` skill only when the control-plane MCP does not expose the needed Linear operation.
5. Use Rube only for non-Mister-Smith external systems or research.

## Guardrails

- never trust one surface in isolation
- never stage blocked work just because Symphony is idle
- never treat `Todo` as generic backlog
- never redesign the workspace before a fresh snapshot and workspace-adjustment audit
- never touch `/Users/matthewmaggio/Repos/symphony` directly for routine maintenance before checking the control-plane MCP snapshot for that checkout

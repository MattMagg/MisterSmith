---
name: mister-smith-control-plane-router
description: Use when any Mister Smith request touches Symphony, Linear, GitHub PR flow, phase slicing, queue dispatch, runtime reconciliation, workspace hygiene, or bootstrap readiness and needs the correct control-plane route first.
---

# Mister Smith Control Plane Router

## Overview

This is the default entry point for Symphony/Linear/Mister Smith workflow operations. Route through
the constitutional control-plane MCP first, then fall back only when the MCP does not cover the
operation.

## Use This When

- the user asks "what should I use?" for a Mister Smith workflow task
- the request spans repo state, Linear state, GitHub PR state, or Symphony runtime state
- you need to decide between inspection, triage, phase staging, review-dispatch, legitimacy, runtime, or config work
- you are not sure which specialized Mister Smith skill should own the request

Do not use this for generic Rust implementation work that stays inside the repo and does not touch
the Symphony/Linear control plane.

## Routing Order

1. Run `audit_workflow_readiness` if MCP/bootstrap health is in doubt.
1. Run `route_workflow_request` with the user's request.
1. Follow the returned MCP tool chain.
1. Use the matching repo-local specialized skill only as the user-facing wrapper.
1. Fall back in this order:
   - repo-local `linear` skill for raw Linear GraphQL gaps
   - Rube for external/non-Mister-Smith systems and research

## Workflow Matrix

| Intent | Primary MCP tool(s) | Repo-local skill |
| --- | --- | --- |
| inspect state | `get_control_plane_snapshot`, `get_issue_execution_snapshot` | `symphony-linear-mister-smith` |
| triage runnable work | `assess_runnable_candidates`, `get_queue_capacity` | `symphony-linear-mister-smith` |
| phase staging | `plan_phase_execution`, `apply_phase_execution_plan` | `stage-mister-smith-phase` |
| review / merge / refill | `review_merge_dispatch_cycle` | `symphony-mister-smith-review-dispatch` |
| legitimacy / drift | `evaluate_issue_legitimacy`, `classify_follow_up_work` | `mister-smith-frontier-mandate` |
| runtime reconcile | `sync_linear_with_runtime`, `refresh_symphony` | `symphony-linear-mister-smith` |
| Symphony checkout maintenance | `get_symphony_checkout_snapshot`, `sync_symphony_main` | `symphony-linear-mister-smith` |
| bootstrap / config | `audit_workflow_readiness`, `plan_workspace_adjustments` | `mister-smith-control-plane-bootstrap` |

## Prompts

- `Use $mister-smith-control-plane-router to inspect the current Symphony/Linear/Mister Smith state.`
- `Use $mister-smith-control-plane-router to break Phase 10 into staged Linear slices.`
- `Use $mister-smith-control-plane-router to land the current Human Review item and refill the queue.`

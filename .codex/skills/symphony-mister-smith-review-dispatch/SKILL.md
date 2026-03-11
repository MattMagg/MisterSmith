---
name: symphony-mister-smith-review-dispatch
description: Use when a Mister Smith Human Review handoff should be reviewed and landed, or when the watched queue has spare capacity and needs a deterministic refill through the control plane.
---

# Symphony Mister Smith Review Dispatch

## Overview

This skill wraps the repeatable Human Review -> merge -> local sync -> queue refill loop. Prefer the
single deterministic MCP workflow instead of hand-running the sequence unless the MCP is missing a
required step.

## Use This When

- landing the current Human Review item
- refilling the watched queue up to available capacity
- confirming Symphony claimed newly staged work
- cleaning up stale Linear state after merge or dispatch

## Primary MCP Tool Chain

1. `review_merge_dispatch_cycle`
2. `review_and_land_human_review_issue` when you want the merge step by itself
3. `fill_queue_to_capacity` when merge is already complete but refill still needs to happen
4. `sync_linear_with_runtime` if Linear and runtime disagree after dispatch

## Rules

- read the PR, do not trust green checks alone
- sync local `main` after merge
- fill capacity with genuinely runnable work, not just whatever exists
- verify Symphony actually claimed the promoted issue

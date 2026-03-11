---
name: symphony-mister-smith-review-dispatch
description: Use when a Mister Smith Human Review handoff should be reviewed and landed, or when the watched queue has spare capacity and needs a deterministic refill through the control plane.
---

# Symphony Mister Smith Review Dispatch

Use the `smith` MCP tools first for Human Review landing and watched-queue refill.

## Workflow

1. Call `review_merge_dispatch_cycle`.
2. If needed, inspect a specific issue with `get_issue_execution_snapshot`.
3. Only fall back to narrower Smith tools when the dispatch loop identifies a concrete follow-up.

## Rules

- Prefer the deterministic review-dispatch loop over manual PR-by-PR handling.
- Do not bypass Smith queue and runtime reconciliation when refilling capacity.

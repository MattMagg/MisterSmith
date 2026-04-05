---
name: symphony-mister-smith-review-dispatch
description: Legacy-named skill for Mister Smith Human Review handoff, review triage, and merge readiness through Smith MCP.
---

# Symphony Mister Smith Review Dispatch

Use the `smith` MCP tools first for Human Review landing and merge triage.

## Workflow

1. Call `review_merge_status`.
2. If needed, inspect a specific issue with `get_issue_execution_snapshot`.
3. Use `resolve_issue_lifecycle` when the issue needs a next-action decision.
4. Only fall back to narrower Smith tools when review triage identifies a concrete follow-up.

## Rules

- Prefer the deterministic review-status pass over manual PR-by-PR handling when you need a repo-wide view.
- Do not rely on queue refill or Symphony runtime assumptions; the active path is direct Codex review and merge work.

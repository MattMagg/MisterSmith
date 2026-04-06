---
name: stage-mister-smith-phase
description: Use when a Mister Smith SpecKit phase or tasks pack needs to be turned into deterministic Linear slices, blocker chains, prep-slice opportunities, and direct-execution-ready work.
---

# Stage Mister Smith Phase

Use the `smith` MCP tools first to derive runnable phase slices and keep only honestly runnable work in the active execution lane.

## Workflow

1. Call `translate_speckit_tasks`.
2. Review runnable slices, blocked slices, and prep opportunities.
3. Call `materialize_backlog_slices` only for the bounded slices that should exist in Linear now.

## Rules

- Do not move blocked slices into active execution just to create parallelism.
- Preserve blocker chains and prep-slice honesty.

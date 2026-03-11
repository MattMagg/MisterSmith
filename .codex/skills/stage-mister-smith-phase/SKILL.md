---
name: stage-mister-smith-phase
description: Use when a Mister Smith SpecKit phase or tasks pack needs to be turned into deterministic Linear slices, blocker chains, prep-slice opportunities, and runnable watched-queue work.
---

# Stage Mister Smith Phase

Use the `smith` MCP tools first to derive runnable phase slices and stage only honest work.

## Workflow

1. Call `plan_phase_execution`.
2. Review runnable slices, blocked slices, and prep opportunities.
3. Call `apply_phase_execution_plan` to stage only the runnable work.

## Rules

- Do not stage blocked slices just to fill the queue.
- Preserve blocker chains and prep-slice honesty.

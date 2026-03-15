# 2026-03-15 Symphony Throughput Ramp

## Objective

Increase actual Mister Smith development throughput by using Symphony as a multi-issue runner rather
than a single-issue queue consumer.

## Scope

- Inspect the live Symphony, Linear, and smith control-plane state
- Identify why the queue is not using available Symphony concurrency
- Record the honest throughput model for this repo
- Patch the smith control-plane heuristics that currently under-report or misreport queue capacity

## Assumptions

- Symphony itself is healthy and can run multiple agents in parallel
- The watched dispatch boundary remains `MisterSmith Execution Queue`
- Only unblocked issues that are safe to start now should enter `Todo`

## Constraints

- Do not stage blocked work just to fill slots
- Do not claim the repo has a validated backlog if the backlog is actually empty
- Keep the control-plane logic evidence-based and repo-local

## Non-goals

- Do not redesign Symphony itself
- Do not fabricate future work without repo-grounded evidence
- Do not interrupt the active `MS-34` run unless evidence shows the run is invalid

## Live Findings

### Runner reality

- `WORKFLOW.md` configures `agent.max_concurrent_agents: 10`
- Symphony currently has a live workspace at `~/.local/share/symphony-workspaces/MS-34`
- The current one-agent state is not caused by a Symphony runtime cap

### Queue reality

- `MS-34` is `In Progress`
- `MS-35` is still `Todo`, but it is blocked by `MS-34`
- `MisterSmith Validated Backlog` is currently empty
- The only additional non-terminal issue found outside the queue is `MS-37`, which smith currently
  classifies as questionable / triage rather than honest runnable frontier work

### Control-plane bottlenecks

- `review_merge_dispatch_cycle` only suggests refill when `queue_issue_count == 0`
- The refill candidate list is title-heuristic-heavy and Phase-10-specific
- `plan_phase_execution("Phase 10")` is stale and still recommends staging `MS-33`
- The current control plane does not surface the important distinction between:
  - active work
  - blocked `Todo` noise
  - actually runnable refill candidates

## Throughput Model For Mister Smith

Symphony can only run as fast as the watched queue is supplied with honest runnable work. In this
repo, that means:

1. Keep the watched queue for unblocked work only
2. Keep a real validated backlog outside the watched queue
3. Refill before the queue drains completely
4. Slice upcoming work more finely so there is more than one independent issue at a time

The current repo underuses Symphony for two reasons:

1. There are too few validated backlog items
2. The remaining Phase 10 work is sliced too coarsely (`MS-34` is a monolith, `MS-35` is a final gate)

## Immediate Actions

1. Patch smith MCP review/dispatch reporting so it can distinguish blocked `Todo` items from honest
   refill opportunity
2. Patch smith MCP phase planning so it stops recommending already-completed Phase 10 slices
3. Use the corrected control plane to drive future refill decisions

## Recommended Next Ramp

1. Finish `MS-34`
2. Remove blocked noise from `Todo` whenever it appears
3. Before `MS-34` completes, create the next wave of validated, independently runnable issues so
   Symphony has real parallel work
4. Prefer smaller slices with explicit blocker chains over umbrella issues

## Validation

- smith control-plane snapshots
- direct Linear issue inspection
- Symphony repo spec/README/workflow inspection
- targeted `mister-smith-mcp` validation after code changes

## Stop Conditions

- The throughput model is documented in-repo
- smith MCP no longer reports the stale single-slice Phase 10 plan
- smith MCP exposes more honest refill/blocked state for future queue governance

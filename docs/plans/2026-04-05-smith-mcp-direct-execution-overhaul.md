# Smith MCP Direct-Execution Overhaul

Date: April 5, 2026
Status: In progress

## Objective

Convert Smith MCP from a Symphony-oriented development control plane into a direct Codex execution
control plane.

## Scope

- remove Symphony and watched-queue concepts from the active Smith MCP tool surface
- add one direct execution preparation tool for issue-grounded Codex work
- keep Smith-owned direct lifecycle helpers for issue context, workpad mutation, backlog slicing,
  SpecKit prep, Ralph packets, and review or merge guidance
- update active repo guidance so the default Smith-first flow matches the new MCP surface

## Constraints

- this is a clean breaking change; do not keep compatibility shims for removed Symphony tools
- keep Linear as the durable issue and workpad source of truth
- keep GitHub as the PR and review source of truth
- keep Ralph and SpecKit support intact
- do not edit unrelated dirty files already present in the repo

## Non-Goals

- keeping Symphony support in the active Smith MCP route
- preserving watched-queue staging behavior behind deprecated wrappers
- rewriting historical docs that are no longer part of active prompts, skills, or router guidance

## Milestones

### Milestone 1: Durable API cleanup

- remove Symphony and queue-oriented public structs, config, routes, and tool registrations
- add `prepare_direct_execution`
- replace `review_merge_dispatch_cycle` with `review_merge_status`
- redefine lifecycle resolution around direct Codex actions

Validation:

- `cargo build -p mister-smith-mcp`
- targeted `cargo test -p mister-smith-mcp --lib`

### Milestone 2: Active guidance cleanup

- update `AGENTS.md`
- update `.codex/commands/implement.md`
- update `.codex/prompts/speckit.implement.md`
- update active Smith skills that enumerate removed tools or Symphony-first routing

Validation:

- targeted grep over active guidance for removed tool names and Symphony-first instructions

### Milestone 3: Regression coverage and closure

- add discovery and routing tests for the new tool surface
- add direct execution preparation tests
- add lifecycle and backlog tests proving queue-only metadata is gone
- confirm the task-owned diff excludes unrelated existing dirty files

Validation:

- `cargo test -p mister-smith-mcp`
- `git diff --check`

## Stop Conditions

- stop before leaving active prompts or skills pointing at removed tool names
- stop before claiming closure if the crate still exposes Symphony tool registrations
- stop before touching unrelated dirty prompt-improver files already present in the worktree

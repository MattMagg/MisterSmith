# Aggressive Codex Subagents For Mister Smith

Date: March 16, 2026
Status: Active

## Objective

Add a repo-scoped Codex subagent system for Mister Smith that is aggressive by default, aligned to
the Smith-first control-plane model, and wired into the repo's existing planning and execution
prompts.

## Scope

- create repo-local Codex config with high-capacity subagent limits
- add a custom agent roster under `.codex/agents/`
- wire the roster into `AGENTS.md` plus the active planning and implementation prompts
- keep durable Linear, queue, PR, and merge mutations in the parent thread

## Defaults

- repo default fan-out: `max_threads = 24`, `max_depth = 4`
- burst profile: `max_threads = 32`, `max_depth = 6`
- CSV batch jobs default to `job_max_runtime_seconds = 3600`, with `smith-burst` raising that to
  `5400`
- subagents may own repo file edits and local validation, but not final control-plane writes

## Deliverables

- `.codex/config.toml`
- `.codex/agents/*.toml`
- `.codex/agents/README.md`
- root `AGENTS.md` subagent guidance
- command + mirrored prompt wiring for planning and implementation

## Validation

- Codex config parses from the repo without changing the home config
- custom agent names resolve and are inspectable through `/agent`
- prompt wiring references the new roster consistently
- boundary language forbids durable control-plane mutations from child agents

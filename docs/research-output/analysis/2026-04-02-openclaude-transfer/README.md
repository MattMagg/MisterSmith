# OpenClaude To Mister Smith Transfer Analysis

Date: April 2, 2026
Status: Analysis
Source repo: `/Users/macmain/openclaude`
Judged against: `docs/current-state.md`, `docs/direction.md`, packet `023`, and packet `024`

## Purpose

This package captures the parts of `openclaude` that look genuinely useful for Mister Smith.

It does not treat `openclaude` as an authority. It treats it as external comparative input and
filters the ideas through current Mister Smith truth and direction.

## Bottom Line

The strongest near-term transfer ideas are:

1. a cleaner provider adapter and message-translation layer
2. a schema-sanitizing compatibility pass for tools and MCP
3. better long-lived MCP lifecycle management and large-result offloading
4. a real operator-visible plan mode
5. a stronger live-work cockpit for sessions, runs, and subordinate work

The strongest later-stage ideas are:

1. stable child-agent identity with continue-in-place messaging
2. ordered parallel tool execution with deterministic result ordering
3. a first-class subordinate execution-unit model under one workflow
4. resumable remote executors with explicit control channels

## What This Analysis Is Not Recommending

- do not copy `openclaude` provider shims verbatim
- do not collapse developer-workflow features into Mister Smith product truth
- do not add a plugin marketplace as a near-term Smith priority
- do not claim remote child runtime proof from a design note alone

## Reading Order

1. `01-runtime-and-tooling.md`
2. `02-operator-and-ux.md`
3. `03-remote-and-delegated-execution.md`
4. `04-priority-backlog.md`

## Current Mister Smith Anchors Used In This Package

- runtime and session truth:
  - `docs/current-state.md`
  - `specs/023-runtime-truth-and-run-trace/`
- least-privilege and capability boundary truth:
  - `specs/024-agent-boundary-security-hardening/`
- next-direction judgment:
  - `docs/direction.md`
- likely implementation surfaces:
  - `crates/mister-smith-llm/`
  - `crates/mister-smith-agents/`
  - `crates/mister-smith-events/`
  - `crates/mister-smith-app/`
  - `crates/mister-smith-mcp/`
  - `apps/operator-console/`

## Fit Labels

- `High fit now`: can harden or extend the current shipped path without changing product truth
- `Conditional fit next`: useful, but better after current runtime hardening or in a later packet
- `Later or do-not-copy`: interesting, but should not be pulled into the near-term Smith path

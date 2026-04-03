# 2026-04-02 OpenClaude Transfer Analysis

## Objective

Do a second-pass, frontier-focused analysis of `/Users/macmain/openclaude` and revise the existing
transfer docs so they keep only the ideas that should help Mister Smith's coordination runtime,
capability boundary, operator proof, and execution safety.

## Scope

- read-only analysis of `/Users/macmain/openclaude`
- compare current OpenClaude findings against the existing transfer bundle:
  - `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/README.md`
  - `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/01-runtime-and-tooling.md`
  - `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/02-operator-and-ux.md`
  - `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/03-remote-and-delegated-execution.md`
  - `/Users/macmain/MisterSmith/docs/research-output/analysis/2026-04-02-openclaude-transfer/04-priority-backlog.md`
- revise the transfer docs so they explicitly mark keep, update, defer, and remove decisions
- produce packet-targeted recommendations for:
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/`
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/`

## Assumptions

- packets `022` through `025` are already landed and are now the upstream truth to extend, not
  provisional targets to guess at
- this is still analysis and transfer judgment only, not a packet implementation pass
- the transfer bundle should help future packet work, not widen current product claims

## Constraints

- no changes to Mister Smith runtime code or packet implementation state
- no "copy because OpenClaude has it" reasoning
- keep the Mister Smith frontier mandate in force:
  - prefer coordination, supervision, routing, memory, reliability, observability, and execution
    safety
  - reject framework-marketplace parity work unless it clearly strengthens Smith's OS boundary
- keep product/runtime boundary separate from external development workflow tools

## Non-Goals

- no full OpenClaude comparison report
- no provider-feature shopping list
- no plugin-marketplace roadmap
- no claim that later remote executor ideas belong in the current local default runtime path

## Milestones

### Milestone 1: Re-scan OpenClaude for packet-relevant seams

Validation:

- inspect the coordinator, task, MCP, remote, permission, and command-gating paths in source
- capture file-backed evidence for each candidate idea

### Milestone 2: Re-judge the old transfer ideas

Validation:

- every prior idea gets one explicit verdict:
  - `KEEP as-is`
  - `KEEP with update`
  - `SPLIT or DEFER`
  - `REMOVE as misfit`
- verdicts stay grounded in Mister Smith's current packet and direction docs

### Milestone 3: Revise the transfer docs

Validation:

- update the dated transfer bundle in place
- make `/04-priority-backlog.md` the one decision-grade summary for future packet authors

### Milestone 4: Leave packet-ready recommendations

Validation:

- the revised summary names the top additions or clarifications for packet `026` and packet `027`
- every recommendation is labeled by why it matters:
  - frontier leverage
  - implementation correctness
  - protocol boundary
  - operator clarity
- later ideas are explicitly deferred

## Stop Conditions

- stop if a proposed transfer would mainly recreate OpenClaude's framework shell instead of
  strengthening Mister Smith's runtime OS
- stop if a recommendation would widen packet `026` or `027` past their bounded purpose
- stop if the analysis cannot point to concrete source files in `/Users/macmain/openclaude`

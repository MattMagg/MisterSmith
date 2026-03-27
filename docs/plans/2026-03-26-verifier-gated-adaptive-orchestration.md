# 2026-03-26 Verifier-Gated Adaptive Orchestration

## Status

Landed on `main` through `MS-104` through `MS-107` as of 2026-03-27

## Closure Boundary

- packet `020` is now landed on the runtime-backed task path: verifier-gated step decisions,
  first-class handoff clarification, preserved failure-context plus last stable checkpoint repair
  lineage, and operator-visible orchestration-quality provenance
- deterministic validation for the landed slice is current through `MS-107`
- no new live runtime-proof claim is introduced in this packet; the existing provider-backed proof
  baseline remains the packet-019 `openai_chatgpt` / `gpt-5.4` smoke path

## Objective

Freeze one bounded post-packet-019 phase that improves actual workflow and orchestration quality on
the shipped runtime path. The goal is not more provider work or budget work. The goal is to make
Mister Smith better at catching weak intermediate steps, clarifying bad handoffs, and repairing
work locally before a whole task degrades.

## Repo-Grounded Current Truth

- packet `019` is complete on `main`, and the runtime-backed task path now has a bounded
  multi-provider routing profile plus JetStream-backed budget enforcement when explicitly
  configured
- the current shipped runtime path already has supervised planner and executor lifecycles,
  ToolBus-backed execution, task and autonomy provenance, and repeatable local smoke proof on the
  `openai_chatgpt` / `gpt-5.4` baseline
- the current runtime path now has a first-class verifier-gated workflow-step control loop with
  typed verdicts and repair directives
- handoff clarification is now explicit on weak downstream step transitions
- failure context and last stable checkpoint lineage are now preserved for bounded retry or
  re-plan decisions
- task and autonomy inspection now surface verifier and repair provenance without requiring raw
  log archaeology
- research already captured in this repo motivated the landed seam:
  - `docs/research-output/research/discovery-sweep-R4.md`: PRMs and the
    planner-executor-verifier decomposition
  - `docs/research-output/research/targeted-step-level-intelligence-R6.md`: strict
    orchestrator-owned step verification and local backtracking
  - `docs/research-output/research/targeted-supervision-fault-tolerance-R4.md` plus
    `docs/research-prompts/R8/06-predictive-supervision.md`: AgentAsk-style clarification,
    structured reflection, and contextual rollback
  - `docs/research-output/research/discovery-sweep-R7b.md`: adaptive workflow refinement and
    edge-level error mitigation

## Why This Was The Next Bounded Gap

The next benchmark-relevant gain is not another routing packet. It is workflow quality control.
High-score coding runs fail when a weak intermediate step propagates, when a handoff misses a
constraint, or when the system restarts too much work instead of repairing the local branch.

This packet therefore froze and landed one bounded phase:

1. add a verifier-gated step contract on the runtime-backed task path
2. add bounded clarification and repair actions at workflow handoffs
3. preserve failure context and last stable checkpoint for local retry or re-plan
4. surface orchestration-quality provenance so task/autonomy views explain what was accepted,
   clarified, retried, replanned, or stopped

## Scope

- one bounded verifier-gated control loop on the current runtime-backed task path
- handoff clarification as a first-class action instead of implicit prompt drift
- local repair directives and failure-context propagation instead of full-task restart by default
- operator-visible provenance for verifier and repair outcomes
- benchmark-oriented workflow quality improvement, but not a new benchmark harness in this packet

## Assumptions

- current planner/executor supervision and ToolBus boundaries remain the canonical runtime path
- step boundaries can begin at workflow and handoff granularity before any token-stream verifier
  work is attempted
- existing task/autonomy projection surfaces are the correct place to expose orchestration-quality
  provenance
- packet `018` remains separate review work and is not a prerequisite for freezing this phase

## Constraints

- no provider expansion, provider benchmarking, or provider-proof widening
- no budget-policy expansion or budgeting-focused implementation
- no broad operator-console redesign
- no decentralized topology rewrite, CRDT program, or agent-graph overhaul
- no RL training program, PRM training pipeline, or workflow evolution engine in this packet
- no queue staging during this scope-freeze pass

## Non-Goals

- claiming a new SWE-bench score before benchmark work exists
- implementing workflow evolution, topology search, or self-improving archives
- making token-stream PRM evaluation the day-one requirement
- broadening this phase into a general "research-output catch-up" program

## Milestones

### Milestone 1: Freeze packet and verifier/repair contract

Status: landed

Deliverables:

- this planning note
- packet `020` under `specs/`
- router docs updated to point at packet `020` as the next bounded phase

Validation:

- packet and note cite current repo truth and name the missing workflow-quality seam explicitly

### Milestone 2: Add verifier-gated workflow step decisions

Status: landed via `MS-104`

Deliverables:

- typed verifier verdict and repair directive surfaces
- runtime execution gating that can accept, reject, clarify, retry, re-plan, or stop
- preserved current happy path when the verifier policy is not active

Validation:

- targeted app and core tests for step verdict handling and fallback behavior

### Milestone 3: Add clarification and contextual repair

Status: landed via `MS-105`

Deliverables:

- handoff clarification request path
- failure-context propagation plus last stable checkpoint
- bounded local retry and re-plan semantics

Validation:

- targeted tests for clarification loops, retry budgets, and checkpoint-based repair

### Milestone 4: Extend provenance and proof boundaries

Status: landed via `MS-106` and closed with deterministic validation/doc sync in `MS-107`

Deliverables:

- task/autonomy evidence for verifier and repair history
- explicit deterministic versus live-proof language for any new runtime evidence

Validation:

- targeted inspection/result-contract checks
- live proof only if the verifier-gated path can be exercised honestly on the current baseline

## Stop Conditions

- the packet would require a provider or budgeting expansion to show value
- the packet would require a new RL or PRM training stack before a bounded first slice can land
- the packet cannot preserve the current shipped happy path while the verifier loop is introduced
- benchmark claims would get ahead of actual measurable evidence

## Current Role

This note is no longer the forward-development authority. It now serves as the packet-020 scope
and closure note for the landed verifier-gated orchestration slice on `main`. No newer
post-packet-020 bounded phase is frozen yet.

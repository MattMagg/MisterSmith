---
version: R9
created: 2026-03-28
type: prompt
tier: 1
timeline: March 7, 2026 — present
---

# Deep Research Prompt: Workflow Engines, Compensation, and Resume Semantics

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on
NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to
define the standard that the agent framework market will converge toward.

Phases `1-10` are landed. The default runtime path now includes supervised planner and executor
lifecycle management, ToolBus-backed execution boundaries, verifier-gated orchestration quality
provenance, and bounded same-agent session handling. The current repo-wide state is routed through
`docs/current-state.md`, and the research baseline is routed through the manifest and consolidated
research corpus under `docs/research-output/`.

**Fixed inputs**

- `<baseline_boundary>`: March 7, 2026
- `<search_window>`: March 7, 2026 to present
- `<repo_state_router>`: `docs/current-state.md`
- `<routing_manifest>`: `docs/research-output/ROUTING_MANIFEST.md`
- `<baseline_docs>`:
  - `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
  - `docs/research-output/consolidated/02-orchestration-and-self-organization.md`
  - `docs/research-output/consolidated/03-supervision-and-resilience.md`
  - `docs/research-output/consolidated/06-streaming-architecture.md`
  - `docs/research-output/consolidated/08-competitive-landscape-and-ecosystem.md`

Use the documents above as the authoritative baseline. `R8` is a structure reference only. Do not
use prior prompt text as the source of truth for what is already known.

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by OpenAI Agents
SDK, Google ADK, LangChain/LangGraph, CrewAI, AutoGen, Claude SDK, or similar systems. Benchmark
them. Learn from them. Then exceed them.

Pull from durable workflow engines, telecom transaction handling, distributed job control, payment
reversal systems, exchange order lifecycles, operating systems, and actor-based recovery models
when those fields offer stronger patterns for long-running workflow semantics.

Reuse what is already correct. Do not reinvent primitives without benefit. But wherever the choice
affects Mister Smith's execution, supervision, resumability, operator truth, or distributed
behavior, prefer the architecture with the highest long-term leverage rather than the most
conventional framework pattern.

Incremental imitation is failure. Favor designs that create real advantage.

## Research Objective

Survey everything published from March 7, 2026 to the present on durable workflow execution,
checkpoint/resume semantics, cancellation propagation, compensation patterns, reversible side
effects, partial-failure recovery, workflow provenance, and long-running agent workflow state
management.

The goal is to discover what has changed since the current repo baseline and identify techniques
that should influence how Mister Smith handles long-running execution beyond the current
supervision, checkpoint, and streaming design.

This is an open-ended research task. Go beyond the dimensions below if you discover strong leads.

**Older-source rule**: include older sources only if they are absent from the current repo
baseline and materially change the design direction.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The current research corpus already establishes a strong substrate but admits a workflow-semantics
gap. Mister Smith's baseline already includes supervised actor lifecycles, per-workflow checkpoint
streams, contextual rollback signals, circuit breakers, Saga-style compensation as a useful
reference pattern, and a separation between transient, structural, streaming, and semantic failure
classes. It already treats checkpoint streams, role-aware supervision, and stream finalization as
foundational components for reliable runtime behavior.

At the same time, the repo's consolidated findings explicitly say the field is stronger on
topology selection and failure detection than on workflow semantics. The current orchestration
synthesis admits there is still **no concrete taxonomy for LLM tool reversibility or compensation
patterns** across common side-effect types. The supervision synthesis also leaves several workflow
questions open: how partial streams are resumed per provider, how durable circuit-breaker state
should be stored, and how restart semantics interact with partial progress and external side
effects.

The baseline already treats the following as known:

- supervised actors plus durable checkpoints are the current foundation
- restart alone is insufficient when side effects or semantic degradation are involved
- Saga-like compensation is promising but incomplete as a full workflow model
- contextual rollback and failure provenance matter for recovery honesty
- the current literature leaves compensation, reversible tool taxonomies, and workflow transaction
  semantics under-specified

Do not rediscover those findings. Only surface new work that materially contradicts, sharpens, or
extends them.

## Research Dimensions

### 1. Durable Workflow Engines for Agent Systems

- What new workflow-engine architectures or execution models have appeared for long-running agent
  workflows?
- Are there new systems that combine durable execution with actor supervision instead of choosing
  one model over the other?
- What production reports exist for pause/resume, checkpoint, and crash recovery in agent
  workflows?

### 2. Compensation and Reversible Side-Effect Design

- What new research or production patterns exist for compensation in tool-heavy agent workflows?
- Has anyone published a practical taxonomy for reversible vs. irreversible tool actions?
- Are there stronger compensation patterns from workflow engines, payments, trading, telecom, or
  distributed transaction systems that transfer cleanly to agent systems?

### 3. Checkpoint, Resume, and Partial Commit Semantics

- What new checkpointing, rehydration, or partial-commit models exist for multi-agent workflows
  with streaming state?
- Are there better ways to resume from partially completed tool execution or partial model streams?
- How do newer systems separate recoverable state from non-recoverable side effects?

### 4. Cancellation, Timeouts, and Failure Propagation

- What advances exist in cancellation propagation across multi-step or multi-agent workflows?
- Are there new structured-concurrency or supervisory models for long-running AI workflows?
- How do modern systems decide when to abort, retry, degrade, or compensate instead of
  restarting?

### 5. Workflow Provenance and Operator Truth

- What new patterns exist for operator-visible workflow state, compensation history, and resume
  provenance?
- Are there better models for exposing partial completion, unresolved side effects, and degraded
  recovery to operators?
- What evidence exists that stronger workflow provenance changes reliability or operator trust?

### 6. Adjacent-Field Transfer

- What should Mister Smith learn from Temporal, Cadence, Erlang release handling, telecom call
  flows, exchange order lifecycles, and payment reversals?
- Which of those patterns remain underused in agent frameworks despite strong production evidence?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations
2. **Key techniques** — the concrete workflow, resume, or compensation patterns discovered
3. **Applicability to Rust + NATS + OTP** — how well the pattern transfers to Mister Smith
4. **Delta from baseline** — what is genuinely new versus the repo's current research corpus
5. **Frontier classification** — classify the finding as `EXTEND`, `TRANSFORM`, or `NEW`, and
   also as `FRONTIER` or `INCREMENTAL`
6. **Mister Smith implementation vector** — name the likely crates, runtime surfaces, or spec
   areas affected; prefer concrete repo surfaces such as `mister-smith-supervision`,
   `mister-smith-agents`, `mister-smith-llm`, `mister-smith-persistence`, `mister-smith-core`,
   `mister-smith-events`, `mister-smith-app`, or `specs/021-*`
7. **Evidence status** — classify the finding as `production-validated` or `research-only`
8. **Implementation complexity** — rough effort, prerequisites, and operational risk

## Synthesis

After completing all dimensions, provide a synthesis that:

- ranks the top 5 findings by strategic value for Mister Smith
- identifies **contradictions to current assumptions** in the repo baseline
- separates findings into:
  - `production-validated`
  - `research-only`
  - `thin-results`
- recommends which findings should be prototyped, benchmarked, adopted, or only monitored
- names the likely Mister Smith implementation vectors for the highest-value findings
- clearly states where the literature remains weak instead of padding with speculative filler

## Research Methodology

1. Read the baseline docs named above before searching.
2. Search broadly across March 7, 2026 to present. Include papers, releases, design docs,
   postmortems, benchmarks, and production engineering reports.
3. Look beyond agent frameworks into workflow engines, durable execution systems, telecom,
   trading, and transactional distributed systems.
4. Prefer evidence about real checkpointing, cancellation, resume, and compensation behavior over
   generic framework marketing.
5. If a topic yields thin results, say so directly rather than padding.

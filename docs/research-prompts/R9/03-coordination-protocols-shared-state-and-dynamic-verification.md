---
version: R9
created: 2026-03-28
type: prompt
tier: 1
timeline: March 7, 2026 — present
---

# Deep Research Prompt: Coordination Protocols, Shared State, and Dynamic Verification

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on
NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to
define the standard that the agent framework market will converge toward.

The current coordination baseline already uses a three-tier substrate in the research corpus:
delta-CRDTs for shared artifacts, JetStream KV CAS for strict invariants, and NATS request-reply
for ephemeral routing. The repo also already identifies MPST and Rust session typing as a possible
compile-time differentiator. This prompt asks what has changed since that March 7, 2026 baseline.

**Fixed inputs**

- `<baseline_boundary>`: March 7, 2026
- `<search_window>`: March 7, 2026 to present
- `<repo_state_router>`: `docs/current-state.md`
- `<routing_manifest>`: `docs/research-output/ROUTING_MANIFEST.md`
- `<baseline_docs>`:
  - `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
  - `docs/research-output/consolidated/04-security-and-trust.md`
  - `docs/research-output/consolidated/05-coordination-and-state.md`
  - `docs/research-output/consolidated/08-competitive-landscape-and-ecosystem.md`

Use those documents as the authoritative baseline. `R8` is a structure reference only.

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by OpenAI Agents
SDK, Google ADK, LangChain/LangGraph, CrewAI, AutoGen, Claude SDK, or similar systems. Benchmark
them. Learn from them. Then exceed them.

Pull from distributed databases, process algebra, formal methods, programming-language theory,
leaderless coordination, and safety-critical protocol design when those fields offer stronger
coordination or verification primitives.

Reuse what is already correct. Do not reinvent primitives without benefit. But wherever the choice
affects Mister Smith's coordination, shared-state correctness, security, or distributed behavior,
prefer the architecture with the highest long-term leverage rather than the most conventional
agent-framework pattern.

Incremental imitation is failure. Favor designs that create real advantage.

## Research Objective

Survey everything published from March 7, 2026 to the present on CRDT coordination, shared-state
security, dynamic session typing, evolving protocol verification, event-triggered consensus,
semantic conflict handling, and coordination correctness under dynamic topologies.

The goal is to discover what has changed since the repo's coordination baseline and identify
techniques that should influence Mister Smith's coordination and verification layers.

This is an open-ended research task. Go beyond the dimensions below if you discover strong leads.

**Older-source rule**: include older sources only if they are absent from the current repo
baseline and materially change the direction.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The current corpus already accepts a hybrid coordination model. CRDTs are already treated as
useful for shared artifacts but insufficient for invariants. CodeCRDT-style evidence already
establishes syntactic convergence plus residual semantic conflicts. The baseline also already
documents CRDT metadata growth, snapshot and rehydration uncertainty, event-triggered consensus as
promising but under-tuned for LLM agents, and shared-state infection risk from adversarial inputs.

On the verification side, the repo already knows that MPST and Rust session-type libraries are a
unique potential differentiator for compile-time protocol safety, but also that dynamic topology,
join/leave behavior, and evolving DAG verification remain unresolved. The security synthesis
explicitly calls out **formal verification of dynamic agent topologies** as an open problem.

Treat the following as already known:

- CRDTs help with coordination but do not solve semantic correctness by themselves
- MPST is strong for static protocols, weak for dynamic participation
- event-triggered consensus is promising but under-characterized for stochastic LLM-agent state
- shared-state coordination carries an explicit security cost and needs semantic firewalls

Do not rediscover those findings. Only surface work that materially contradicts, sharpens, or
extends them.

## Research Dimensions

### 1. Dynamic Session Types and Protocol Evolution

- What new work exists on session typing or protocol verification for systems where participants
  join, leave, or reshape protocols at runtime?
- Are there stronger hybrid compile-time plus runtime techniques for dynamically evolving
  multi-agent protocols?

### 2. Shared-State Coordination Beyond the Current CRDT Baseline

- Have new CRDT variants, dissemination methods, or semantic-conflict approaches appeared that are
  relevant to agent coordination?
- What has changed on garbage collection, snapshotting, and rehydration?

### 3. Formal Verification of Evolving DAGs and Agent Choreographies

- What new tools or methods exist for verifying dynamically changing agent workflows?
- Are there new TLA+, Alloy, refinement-type, liquid-type, or runtime-verification approaches
  suited to this problem?

### 4. Event-Triggered and Leaderless Coordination

- What new consensus-free or leaderless coordination primitives have emerged?
- Are there better thresholding or signaling models for stochastic LLM-agent state than the older
  event-triggered consensus work?

### 5. Shared-State Infection Controls and Semantic Firewalls

- What new techniques exist for securing shared coordination substrates against adversarial state
  injection?
- Are there new formal or runtime approaches for separating safe shared state from prompt-facing
  state?

### 6. Rust Ecosystem and Production Transfer

- What new Rust libraries, implementation reports, or production systems affect dynamic
  verification and coordination safety?
- Has any production system demonstrated techniques that could transfer directly into Mister Smith?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations
2. **Key techniques** — the concrete coordination or verification mechanisms discovered
3. **Applicability to Rust + NATS + OTP** — how well the pattern transfers to Mister Smith
4. **Delta from baseline** — what is genuinely new versus the repo's current research corpus
5. **Frontier classification** — classify the finding as `EXTEND`, `TRANSFORM`, or `NEW`, and
   also as `FRONTIER` or `INCREMENTAL`
6. **Mister Smith implementation vector** — name the likely crates, runtime surfaces, or spec
   areas affected; prefer concrete repo surfaces such as `mister-smith-nats`,
   `mister-smith-supervision`, `mister-smith-core`, `mister-smith-events`,
   `mister-smith-persistence`, `mister-smith-agents`, `mister-smith-mcp`, or `spec/`
7. **Evidence status** — classify the finding as `production-validated` or `research-only`
8. **Implementation complexity** — rough effort, prerequisites, and risk

## Synthesis

After completing all dimensions, provide a synthesis that:

- ranks the top 5 findings by strategic value for Mister Smith's coordination and verification
  layers
- identifies **contradictions to current assumptions** in the repo baseline
- separates findings into:
  - `production-validated`
  - `research-only`
  - `thin-results`
- recommends which findings should be prototyped, benchmarked, adopted, or only monitored
- names the likely implementation vectors for the strongest findings
- clearly states where the literature remains weak instead of padding with speculation

## Research Methodology

1. Read the baseline docs named above before searching.
2. Search broadly across March 7, 2026 to present. Include papers, releases, crate updates,
   engineering reports, and standards work.
3. Look beyond agent frameworks into distributed databases, PL theory, formal methods, and
   process-algebra communities.
4. Distinguish production-ready techniques from mathematically elegant but immature ones.
5. If a topic yields thin results, say so directly rather than padding.

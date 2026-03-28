---
version: R9
created: 2026-03-28
type: prompt-suite
---

# Mister Smith R9 Deep-Research Prompt Suite

## Purpose

This directory contains the `R9` additive deep-research prompt round for Mister Smith.

Use this suite when you want a new research pass grounded in the current repo baseline rather than
the older `R8` prompt text. The authoritative "already discovered" boundary is:

- `docs/current-state.md`
- `docs/research-output/ROUTING_MANIFEST.md`
- the relevant files under `docs/research-output/consolidated/`

Do **not** treat `docs/pulse-tasks/` or the unexecuted `R8` prompts as the source of truth for
what is already known.

## Baseline Boundary

- **Baseline date**: March 7, 2026
- **Search window**: March 7, 2026 to present
- **Older-source rule**: include older material only if it is missing from the repo baseline and
  materially changes the design direction

## Suite Design

The `R9` suite exists because the repo already has strong prior coverage of routing, orchestration,
streaming, supervision, coordination, security, memory, and competitive landscape. The remaining
value is in a sharper set of prompts that:

- refresh the corpus against the latest developments
- target the documented open gaps in the consolidated research
- force honest separation between frontier findings and thin or hype-driven results
- map findings back to Mister Smith implementation surfaces

Every prompt in this suite uses the same backbone:

`Context` → `Frontier-First Mandate` → `Research Objective` →
`What Has Already Been Researched` → `Research Dimensions` →
`Per-Dimension Output Structure` → `Synthesis` → `Research Methodology`

Every prompt also requires:

- `Frontier classification` using the routing-manifest taxonomy:
  `EXTEND`, `TRANSFORM`, `NEW`, plus `FRONTIER` vs `INCREMENTAL`
- `Mister Smith implementation vector` naming likely crates, runtime surfaces, or spec areas
- explicit separation of:
  - `production-validated`
  - `research-only`
  - `thin-results`
  - `contradictions to current assumptions`

## Prompt Map

- `01-workflow-engines-compensation-and-resume.md`
  Scope: durable execution, resume, cancellation, compensation, reversible tools
  Primary baseline docs:
  `00-MASTER-FINDINGS`, `02-orchestration-and-self-organization`,
  `03-supervision-and-resilience`, `06-streaming-architecture`,
  `08-competitive-landscape-and-ecosystem`
- `02-dynamic-orchestration-and-topology-control.md`
  Scope: topology compilers, adaptive orchestration, decentralized DAG plus OTP reconciliation
  Primary baseline docs:
  `00-MASTER-FINDINGS`, `01-model-routing-and-cost-optimization`,
  `02-orchestration-and-self-organization`, `03-supervision-and-resilience`,
  `08-competitive-landscape-and-ecosystem`
- `03-coordination-protocols-shared-state-and-dynamic-verification.md`
  Scope: CRDTs, dynamic session typing, evolving DAG verification, shared-state infection controls
  Primary baseline docs:
  `00-MASTER-FINDINGS`, `04-security-and-trust`, `05-coordination-and-state`,
  `08-competitive-landscape-and-ecosystem`
- `04-real-time-inter-agent-communication-and-transport.md`
  Scope: real-time transport, bidirectional streams, QoS, resumption, protocol adapters
  Primary baseline docs:
  `00-MASTER-FINDINGS`, `05-coordination-and-state`, `06-streaming-architecture`,
  `07-memory-and-context`, `08-competitive-landscape-and-ecosystem`
- `05-collaborative-communication-handoffs-and-cognitive-alignment.md`
  Scope: clarification loops, trust calibration, negotiation, anti-groupthink communication policy
  Primary baseline docs:
  `00-MASTER-FINDINGS`, `02-orchestration-and-self-organization`,
  `03-supervision-and-resilience`, `07-memory-and-context`

## Recommended Run Order

1. `02-dynamic-orchestration-and-topology-control.md`
   Use first if you want the broadest update to Mister Smith's next orchestration direction.
2. `03-coordination-protocols-shared-state-and-dynamic-verification.md`
   Use next to test whether the coordination and safety primitives have shifted.
3. `04-real-time-inter-agent-communication-and-transport.md`
   Use third to refresh the wire-level and transport assumptions that support orchestration.
4. `05-collaborative-communication-handoffs-and-cognitive-alignment.md`
   Use fourth to refresh policy-level agent communication beyond transport.
5. `01-workflow-engines-compensation-and-resume.md`
   Use when you want the strongest update on durable workflow semantics, resumability, and
   compensation once the other layers are refreshed.

Alternative: run any prompt independently if you only need that surface.

## Scope Boundaries

- `01` owns workflow semantics, cancellation, compensation, and resume behavior
- `02` owns topology shaping and orchestration control
- `03` owns shared-state correctness and protocol verification
- `04` owns transport, multiplexing, backpressure, and protocol adapters
- `05` owns communication policy, handoffs, and cognitive alignment

If a finding spans multiple prompts, keep it in the prompt whose boundary is most directly affected
and cross-reference the others in the synthesis rather than duplicating full treatment.

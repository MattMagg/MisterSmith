---
version: R3
created: 2026-03-07
updated: 2026-03-07
---

# Phase 9+ Research Corpus — Navigation & Context

> **Status: SYNTHESIS COMPLETE**
> Checkpoint: 2026-03-07
> Research rounds completed: 7
> Consolidated synthesis: 9 documents in `consolidated/` (1 master + 8 topic syntheses)

## Governing Directive: Frontier-First

This corpus exists to inform Mister Smith's architecture — not to survey the field. Research is evaluated by strategic advantage created, not by comprehensiveness.

- Findings that create capabilities absent from all competing frameworks are highest priority
- Findings that challenge our assumptions are more valuable than findings that confirm them
- "No Rust implementation exists" is never a valid dismissal — the team builds Rust implementations (19 crates in 2 days). Dismiss only when the approach is mathematically inferior or strategically wrong
- We benchmark against OpenAI Agents SDK, Google ADK, LangChain/LangGraph, CrewAI, AutoGen, Claude SDK, distributed systems, actor systems, operating systems, telecom, and trading infrastructure — but we do not copy them
- Cross-domain theoretical grounding (computational neuroscience, category theory, process calculus) provides design patterns from mature fields

## Research Rounds

| Round | Type | Tool | Output |
|-------|------|------|--------|
| R1 | Ultra2x deep web research | Parallel AI ultra2x | 6 raw reports in `raw/` |
| R2 | User-supplied additional reports | Manual | 12 raw reports in `raw/` (2 per topic) |
| R3 | Triple synthesis (3 raw -> 1 per topic) | 6 parallel Claude agents | 6 synthesis files in `synthesis/` |
| R4 | Academic targeted + discovery sweep | Consensus MCP (200M+ papers) | 6 targeted + 1 discovery in `research/` |
| R5 | Discovery sweep R2 | Consensus MCP | 1 discovery in `research/` |
| R6 | Frontier deep dives | Parallel AI ultra2x | 5 targeted in `research/` |
| R7 | Discovery sweep R3 (user-added) | External research | 4 discovery files in `research/` |

## File Naming Convention

**R# = the research round that produced the file.** This is the round number, not a version counter.

| Directory | Prefix | Description |
|-----------|--------|-------------|
| `synthesis/` | *(none)* | Merged industry reports from Round 3 |
| `research/` | `targeted-` | Focused research on a specific topic |
| `research/` | `discovery-` | Broad sweep for unknowns |

## Directory Structure

```
docs/research-output/
+-- CLAUDE.md                       <- You are here
+-- ROUTING_MANIFEST.md             <- Maps discovery findings to topics + strategic classification
+-- synthesis/                      <- Round 3: merged industry reports (STALE — only covers R1-R2 raw sources)
|   +-- intelligent-model-routing-R3.md
|   +-- agentic-loop-architectures-R3.md
|   +-- streaming-architecture-R3.md
|   +-- supervision-llm-fault-tolerance-R3.md
|   +-- nats-native-agent-patterns-R3.md
|   +-- frontier-agent-architecture-R3.md
+-- research/                       <- Rounds 4-7: external research
|   +-- targeted-model-routing-cascades-R4.md         (Round 4 — Consensus academic)
|   +-- targeted-streaming-backpressure-reactive-R4.md
|   +-- targeted-supervision-fault-tolerance-R4.md
|   +-- targeted-neural-paging-context-management-R4.md
|   +-- targeted-stigmergy-swarm-coordination-R4.md
|   +-- targeted-capability-security-sandboxing-R4.md
|   +-- discovery-sweep-R4.md                         (Round 4 — 1st discovery)
|   +-- discovery-sweep-R5.md                         (Round 5 — 2nd discovery)
|   +-- targeted-dynamic-self-organization-R6.md      (Round 6 — frontier deep dive)
|   +-- targeted-crdt-coordination-R6.md
|   +-- targeted-step-level-intelligence-R6.md
|   +-- targeted-predictive-supervision-R6.md
|   +-- targeted-inter-agent-security-R6.md
|   +-- discovery-sweep-R7a.md                        (Round 7 — 3rd discovery, user-added)
|   +-- discovery-sweep-R7b.md
|   +-- discovery-sweep-R7c.md
|   +-- discovery-sweep-R7d.md
+-- raw/                            <- Rounds 1-2: original source reports (archived, consumed in R3)
    +-- 01-intelligent-model-routing/     (3 files)
    +-- 02-agentic-loop-architectures/    (3 files)
    +-- 03-streaming-architecture/        (3 files)
    +-- 04-supervision-llm-fault-tolerance/ (3 files)
    +-- 05-nats-native-agent-patterns/    (3 files)
    +-- 06-frontier-agent-architecture/   (3 files)
```

## How to Read This Corpus

**For a domain's full state of knowledge:** Start with the synthesis file (R3), then read its paired targeted research (R4/R6), then check discovery sweeps for lateral findings.

**For what we might be missing:** Read discovery sweeps R4, R5, R7a-d and the ROUTING_MANIFEST.md.

**For confidence levels and pending work:** Read `docs/RESEARCH_CHECKPOINT.md`.

## Topic Map

### Original 6 Topics (Synthesis R3 + Targeted Research R4)

| Topic | Synthesis (R3) | Targeted (R4) | Deep Dive (R6) |
|-------|---------------|---------------|-----------------|
| **Model Routing** | `intelligent-model-routing-R3.md` | `targeted-model-routing-cascades-R4.md` | — |
| **Agentic Loops** | `agentic-loop-architectures-R3.md` | — | `targeted-dynamic-self-organization-R6.md` |
| **Streaming** | `streaming-architecture-R3.md` | `targeted-streaming-backpressure-reactive-R4.md` | — |
| **Supervision** | `supervision-llm-fault-tolerance-R3.md` | `targeted-supervision-fault-tolerance-R4.md` | `targeted-predictive-supervision-R6.md` |
| **NATS Patterns** | `nats-native-agent-patterns-R3.md` | `targeted-stigmergy-swarm-coordination-R4.md` | `targeted-crdt-coordination-R6.md` |
| **Frontier Arch** | `frontier-agent-architecture-R3.md` | `targeted-neural-paging-context-management-R4.md` | — |

### Additional Targeted Research (R4)

| File | Topic Area |
|------|-----------|
| `targeted-capability-security-sandboxing-R4.md` | Security (pre-deep-dive) |

### Frontier Deep Dives (R6)

| File | New Dimension |
|------|--------------|
| `targeted-dynamic-self-organization-R6.md` | Meta-orchestration, MaAS, MAS^2, FoA |
| `targeted-crdt-coordination-R6.md` | Observation-driven coordination, delta-CRDTs + JetStream |
| `targeted-step-level-intelligence-R6.md` | PRMs, RSD, CLAI/TALE, per-step routing/budgeting |
| `targeted-predictive-supervision-R6.md` | AWorld fingerprints, OSC CKMs, Guard/Advisor over OTP |
| `targeted-inter-agent-security-R6.md` | CFH attacks, AgentSandbox, Auth Callouts, Macaroons |

### Discovery Sweeps (Rounds 4, 5, 7)

| File | Key Findings |
|------|-------------|
| `discovery-sweep-R4.md` | CRDTs, DAGs, MaAS, PRMs, CLAI, AgentOps, memory tiers, inter-agent attacks, provenance, SLM-default |
| `discovery-sweep-R5.md` | Decentralized DAGs (AgentNet/FoA/DynTaskMAS), MAS^2, OSC CKMs, AWorld, KB-aware routing, event-triggered consensus |
| `discovery-sweep-R7a.md` | Microsoft Agent Framework, Akka Agentic Platform (25k req/sec), Symphony ledger, GNN swarm to 4096 agents, SECP bounded self-modification |
| `discovery-sweep-R7b.md` | RL puppeteer orchestration, AgentAsk clarification, trust calibration, adversarial robustness gaps |
| `discovery-sweep-R7c.md` | GraphBit (68x CPU/140x mem vs Python), persistent KV cache (15.7s->0.6s), Google scaling laws, Vercel fewer-is-more, MPST session types, agent hijacking 97% ASR |
| `discovery-sweep-R7d.md` | PrefillShare shared KV cache, MPST in Rust, biomimetic immunity, game-theoretic mechanism design, infectious jailbreaks (Agent Smith), AdaptOrch topology routing |

## Corpus Statistics

| Metric | Count |
|--------|-------|
| Total files | 42 (6 synthesis + 17 research + 18 raw + 1 manifest) |
| Synthesis files | 6 (Round 3 — stale, only covers R1-R2) |
| Research files | 17 (11 targeted + 6 discovery) |
| Raw source files | 18 (archived, consumed in R3) |
| Peer-reviewed papers cataloged | 2,000+ |
| Industry/technical references | 500+ |
| Research rounds completed | 7 |

## Consolidated Synthesis (START HERE)

The `consolidated/` directory contains the authoritative synthesis of ALL research rounds. These are the documents to read.

```
consolidated/
+-- 00-MASTER-FINDINGS.md                         <- Top 20 findings ranked by impact + implementation roadmap
+-- 01-model-routing-and-cost-optimization.md      <- Two-plane router, PRMs, CLAI, SLM-default, shared KV cache
+-- 02-orchestration-and-self-organization.md      <- MaAS, MAS^2, AdaptOrch, DAGs, RL puppeteer, topology routing
+-- 03-supervision-and-resilience.md               <- OTP extension, predictive supervision, AWorld, OSC CKMs, MAST, neuromorphic homeostasis
+-- 04-security-and-trust.md                       <- CFH, infectious jailbreaks, AgentSandbox, Auth Callouts, Macaroons
+-- 05-coordination-and-state.md                   <- CRDTs, MPST session types, stigmergy, NATS patterns, hybrid model
+-- 06-streaming-architecture.md                   <- Dual-stream, backpressure, disaggregated serving, PrefillShare
+-- 07-memory-and-context.md                       <- Tiered memory, neural paging, KV cache persistence, PICASO
+-- 08-competitive-landscape-and-ecosystem.md      <- GraphBit, Akka, Google scaling laws, Vercel, A2A, Rust ecosystem
```

## Research Tooling

| Tool | Purpose | Scope |
|------|---------|-------|
| **Parallel AI (ultra2x)** | Deep multi-source web research | 11 prompts (6 R1 + 5 R6), ~720K tokens |
| **Consensus MCP** | Academic paper search (200M+ papers) | 190+ searches, 530+ papers, year_min=2025 |
| **Synthesis agents** | Merge 3 reports into 1 per topic | 6 parallel agents (Round 3) |

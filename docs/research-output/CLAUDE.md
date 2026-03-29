---
version: R4
created: 2026-03-07
updated: 2026-03-28
---

# Mister Smith Research Corpus — Navigation & Context

> **Status: corpus active, research rounds complete**
> Checkpoint: 2026-03-28
> Research rounds completed: 7
> Active authority layer: `consolidated/`
> Canonical merged direction source: `docs/direction.md`

## Governing Directive: Frontier-First

This corpus exists to inform Mister Smith's architecture — not to survey the field. Research is evaluated by strategic advantage created, not by comprehensiveness.

- Findings that create capabilities absent from all competing frameworks are highest priority
- Findings that challenge our assumptions are more valuable than findings that confirm them
- "No Rust implementation exists" is never a valid dismissal — the team builds Rust implementations (19 crates in 2 days). Dismiss only when the approach is mathematically inferior or strategically wrong
- We benchmark against OpenAI Agents SDK, Google ADK, LangChain/LangGraph, CrewAI, AutoGen, Claude SDK, distributed systems, actor systems, operating systems, telecom, and trading infrastructure — but we do not copy them
- Cross-domain theoretical grounding (computational neuroscience, category theory, process calculus) provides design patterns from mature fields

## Reading Order Now

Treat the corpus in layers:

1. **Authority**: `consolidated/`
   - This is the research authority layer for Mister Smith's whole-system future direction.
   - Start with `00`, `02`, `03`, `05`, `06`, and `08`.
   - Use `01` for routing follow-ons and historical routing direction.
   - Use `04` for zero-trust, delegation, and security surfaces.
2. **Transfer / judgment**: `analysis/`
   - Imported research is not part of the active planning corpus until it has a repo-grounded transfer brief here.
3. **Evidence backing**: `research/`
   - Use this layer only when a spec, plan, or design note needs narrower evidence than the consolidated docs provide.
4. **Source-only archive**: `raw/`, `synthesis/`
   - Keep for provenance and auditability; do not use as the default reading path.
5. **Intake-only**: `inbox/`
   - Imported reports here should not shape active direction until they have an `analysis/` judgment.
   - `deep-research-report.md` and `deep-research-report (2).md` remain intake-only.
   - `recursive_llm_frameworks.md` is explicitly low-priority intake because it has no active repo mapping today.

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
+-- ROUTING_MANIFEST.md             <- Historical discovery-classification support
+-- analysis/                       <- Repo-grounded transfer briefs for imported research
+-- consolidated/                   <- Authoritative whole-system synthesis and use map
+-- research/                       <- Evidence backing for narrower design claims
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
+-- inbox/                          <- Intake-only imported reports and notes
+-- raw/                            <- Rounds 1-2 source reports (archived, consumed in later layers)
+-- synthesis/                      <- Round 3 legacy synthesis (stale vs consolidated)
```

## How to Read This Corpus

**For whole-system future direction:** Start with `consolidated/`, then read the relevant `analysis/` briefs.
Use `docs/direction.md` when you need the single merged direction source rather than the research corpus itself.

**For bounded implementation work:** Read the current spec or plan, then the relevant consolidated doc(s), then drop into `research/` only if you need narrower evidence.

**For imported-report adjudication:** Read `analysis/` first, not the raw `inbox/` file.

**For confidence levels and corpus status:** Read `docs/RESEARCH_CHECKPOINT.md`.

## Historical Research Provenance Map

This map explains where findings originally entered the corpus. It is provenance support, not the
default reading order.

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
| Total files | 62 (root docs + 9 consolidated + 3 analysis + 17 research + 18 raw + 6 synthesis + 7 inbox) |
| Consolidated authority docs | 9 |
| Analysis transfer briefs | 3 |
| Research files | 17 (11 targeted + 6 discovery) |
| Raw source files | 18 (source-only archive) |
| Legacy synthesis files | 6 (Round 3 — stale, only covers R1-R2) |
| Inbox intake files | 7 (not part of the normal planning path) |
| Peer-reviewed papers cataloged | 2,000+ |
| Industry/technical references | 500+ |
| Research rounds completed | 7 |

## Consolidated Authority Layer (START HERE)

The `consolidated/` directory contains the authoritative synthesis of all research rounds for
Mister Smith's whole-system future direction. This is the research authority layer and the first
place to read inside the corpus. The merged overall direction source lives at `docs/direction.md`.

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

## Active Use Map

- **Whole-system future direction:** start with `00`, `02`, `03`, `05`, `06`, and `08`, then read the relevant `analysis/` brief if imported research is involved.
- **Routing follow-ons and history:** use `01` to preserve routing lessons and cost-structure history without treating it as the sole direction setter.
- **Security and zero-trust surfaces:** use `04` when the question is delegation, trust boundaries, inter-agent security, or operator-visible enforcement.
- **Bounded implementation work:** read the current spec or plan first, then the relevant consolidated doc(s), then `research/` only if narrower evidence is needed.
- **Imported research activation rule:** no inbox report becomes active planning input until it has a repo-grounded judgment in `analysis/`.

## Research Tooling

| Tool | Purpose | Scope |
|------|---------|-------|
| **Parallel AI (ultra2x)** | Deep multi-source web research | 11 prompts (6 R1 + 5 R6), ~720K tokens |
| **Consensus MCP** | Academic paper search (200M+ papers) | 190+ searches, 530+ papers, year_min=2025 |
| **Synthesis agents** | Merge 3 reports into 1 per topic | 6 parallel agents (Round 3) |

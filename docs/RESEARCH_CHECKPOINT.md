---
version: R3
created: 2026-03-07
updated: 2026-03-07
---

# Research Checkpoint — Phase 9+ Architecture Research

> **Checkpoint Date**: 2026-03-07
> **Status**: 7 rounds complete, synthesis DONE. No new research being launched.
> **Governing directive**: Frontier-first — optimize for strategic advantage, not comprehensiveness
> **Current Action**: Research phase complete. Ready for implementation planning.

---

## What Was Searched

### Round 1 — Ultra2x Deep Web Research
- **Tool**: Parallel AI `ultra2x` processor (Rube MCP)
- **Scope**: 6 research prompts, parallel execution, ~385K tokens output
- **Topics**: Model routing, agentic loops, streaming, supervision/fault tolerance, NATS patterns, frontier architecture
- **Output**: 6 raw reports in `raw/`

### Round 2 — User-Supplied Additional Reports
- **Scope**: 12 additional reports (2 per topic) from separate research runs
- **Output**: Added to `raw/` subdirectories

### Round 3 — Triple Synthesis
- **Tool**: 6 parallel Claude agents
- **Scope**: 3 reports merged into 1 per topic. 6 synthesized reports, ~4,700 lines, 400+ deduplicated references
- **Output**: 6 files in `synthesis/` (now named `*-R3.md`)

### Round 4 — Consensus Academic Search (Targeted + Discovery)
- **Tool**: Consensus MCP (200M+ papers)
- **Constraint**: `year_min=2025`
- **Scope**: 7 parallel agents, 160+ searches, 430+ papers. 6 targeted digests + 1 discovery sweep (96 papers)
- **Output**: 6 targeted + 1 discovery at `research/*-R4.md`

### Round 5 — Discovery Sweep
- **Tool**: Consensus (974 papers screened, 50 included)
- **Scope**: 8 thematic search groups across agent dynamics, paradigm shifts, limitations, adjacent fields, Rust/systems, security, protocols, citation graphs
- **Key additions**: Decentralized DAG coordination (AgentNet, FoA, DynTaskMAS), recursive self-generation (MAS^2), cognitive synergy (OSC), profile-aware supervision (AWorld), event-triggered consensus, knowledge-aware routing
- **Output**: `research/discovery-sweep-R5.md`

### Round 6 — Frontier Deep Dives
- **Tool**: Parallel AI ultra2x processor via Rube MCP
- **Task Group ID**: `tgrp_1ef0a430c39b4ca6a9cf4ce18a19adcb`
- **Scope**: 5 targeted frontier prompts, ~335K tokens output, ~1,300 citations
- **Output**: 5 reports at `research/targeted-{topic}-R6.md`
- **Key additions**: AutoMaAS operator lifecycle, MAS^2 tri-agent pattern, FoA VCVs + HNSW, BiPRM 37.7% error detection, RSD 4.4x FLOP reduction, CLAI/TALE 67% token reduction, AWorld fingerprints, OSC 128-dim CKMs, NATS CVE-2025-30215, AgentSandbox ASR reduction to 4.34%, Macaroons for capability delegation
- **Status**: COMPLETE

### Round 7 — Discovery Sweep R3 (User-Added)
- **Source**: 4 external research reports added by user
- **Output**: `research/discovery-sweep-R7a.md` through `R7d.md`
- **Key additions**:
  - **R7a**: Microsoft Agent Framework, Akka Agentic Platform (15k actors, 25k req/sec, 32ms p99), Symphony decentralized ledger, GNN swarm to 4096 agents, SECP bounded self-modification, formal models (category theory, process calculus, Petri nets), Rust crates (autoagents, adk-rust, mistral.rs)
  - **R7b**: RL puppeteer orchestration, AgentAsk clarification modules, trust calibration for heterogeneous ensembles, research gaps in adversarial robustness
  - **R7c**: GraphBit (68x CPU, 140x memory vs Python), persistent quantized KV cache (15.7s->0.6s resume), Google scaling laws (more agents hurts sequential tasks), Vercel fewer-is-more (80%->100% accuracy by removing 80% of tools), MPST session types in Rust (proven in Mozilla Servo), agent hijacking at 97% ASR, EchoLeak CVE-2025-32711
  - **R7d**: PrefillShare shared KV cache for disaggregated multi-model serving, MPST/pi-calculus for compile-time protocol safety, biomimetic fault tolerance (digital immune system, consensus-based threat validation), game-theoretic mechanism design (Proof-of-Thought, auctions), infectious jailbreaks (Agent Smith exponential propagation, COWPOX defense), AdaptOrch topology routing, ZeroClaw 3.4MB binary, A2A protocol details
- **Status**: COMPLETE

### Totals
| Metric | Value |
|--------|-------|
| Research rounds completed | 7 |
| Total source reports | 18 (raw) + 12 (user-added) |
| Synthesized reports | 6 (Round 3 — stale, only covers R1-R2) |
| Research files | 17 (11 targeted + 6 discovery) |
| Peer-reviewed papers cataloged | 2,000+ |
| Industry/technical references | 500+ |
| Total corpus size | 42 files, ~2.5 MB |
| Searches executed | 190+ |

---

## High-Confidence Findings

Conclusions reached independently by 3+ sources OR confirmed by both industry analysis and peer-reviewed papers.

### Architecture
1. **NATS JetStream pull consumers** are the correct inter-agent streaming primitive — implicit backpressure, horizontal scaling, no rebalancing
2. **Actor-per-LLM-stream** with OTP supervision isolates failures without cascading — confirmed by ractor, Erlang literature, CRGC formal verification
3. **JetStream as event-sourcing backbone** enables checkpoint/replay, time-travel debugging, exactly-once semantics, durable execution
4. **Gatekeeper actor per provider** (token-bucket + circuit breaker as supervised process) — not middleware
5. **`ModelEvent` enum with `#[non_exhaustive]` + `#[serde(other)]`** — correct Rust idiom for forward-compatible streaming
6. **Tokio `StreamMap`** over `SelectAll` for dynamic fan-in — O(1) insertion/removal, fair polling
7. **Decentralized DAG-based coordination outperforms centralized orchestration at scale** — AgentNet, FoA, DynTaskMAS independently confirm (strong evidence 9/10)
8. **Topology determines performance more than model capability** — AdaptOrch shows double-digit % improvements from topology routing alone, identical models (R7d)

### Memory & Context
9. **Tiered memory (STM/MTM/LTM)** validated — 49% F1 improvement (MemOS), 91% lower p95 latency (Mem0)
10. **LLMs lose state across tool calls** (FuncBenchGen) — explicit state management is mandatory
11. **Token importance is predictable** (TokenButler) — learned eviction for neural paging is feasible
12. **KV-Distill** achieves 99% context compression while preserving performance
13. **A-MEM** (127 citations) — agentic memory with dynamic linking is the leading architecture
14. **Persistent quantized KV cache** — 4-bit quantization to disk reduces agent resume from 15.7s to 0.6s on M4 Pro (R7c)

### Routing & Cost
15. **Learned routing** achieves up to 85% cost reduction at 95% quality retention (RouteLLM)
16. **SLM-default/LLM-fallback** — 1-12B models with guided decoding match or exceed large models for structured tasks at 10-100x lower cost (106 citations, Liu et al.)
17. **Process Reward Models** enable per-step verification with dynamic model escalation (4.4x FLOP reduction, 63 citations RSD)
18. **Fewer tools = better** — Vercel case study: removing 80% of tools improved accuracy 80%->100%, latency 3.5x (R7c)
19. **More agents can hurt** — Google scaling laws (Kim & Liu 2026): 180 configurations show more agents degrades sequential task performance (R7c)

### Security
20. **Infrastructure-layer security enforcement is mandatory** — GPT-4.1 achieves F1=0.27 on RBAC
21. **Inter-agent communication hijacking** achieves 58-100% ASR even when individual agents resist; 97% ASR in COLM 2025 study (R7c)
22. **Distributed backdoor attacks** activate only in multi-agent collaboration sequences
23. **Infectious jailbreaks spread exponentially** — Agent Smith attack: single poisoned input compromises entire swarm via shared memory (R7d)
24. **AgentSandbox** reduces ASR to 4.34% via persistent/ephemeral separation and I/O Firewall (R6)

### Coordination
25. **Stigmergy-RL formal equivalence** (Vellinger 2025) — JetStream KV + TTL is mathematically grounded
26. **Thermodynamic scaling bound N^2*d^2** — principled cutoff for orchestrated to stigmergic coordination
27. **CRDT-based coordination** (CodeCRDT) — 100% convergence, zero merge failures in 600 trials
28. **MPST session types provide compile-time protocol safety** — proven in Mozilla Servo, maps to Rust affine type system (R7c, R7d)

### Failure Handling
29. **MAST taxonomy** (134 citations) — 14 failure modes in 3 categories map to supervision tree levels
30. **SagaLLM** (PVLDB) — Saga pattern validated for multi-agent LLM workflows

### Competitive Position
31. **Rust agents are 68x CPU / 140x memory more efficient than Python** — GraphBit benchmarks (R7c)
32. **Akka Agentic Platform** achieves 15k actors, 25k req/sec, 32ms p99 — closest JVM competitor benchmark (R7a)

---

## Tentative Findings

Evidence from 1-2 sources, low citation counts, or not validated against Mister Smith's architecture.

### From Rounds 1-4
1. **COCO contextual rollback** — passing failure context to restarted actors. Single paper. Build effort: ~1 week (extend existing restart strategies).
2. **AgentSight eBPF observability** — <3% overhead. Build effort: 1-2 weeks for Rust eBPF probes (libbpf-rs crate exists).
3. **Token Throttling** (gLLM) — novel backpressure primitive. Single paper, not tested with NATS.
4. **Progent DSL** for capability policies — 0% attack success rate, 17 citations. Build effort: ~1 week for Rust policy DSL.
5. **Streaming-VR** — real-time token-level verification. Single paper.
6. **Federation of Agents** (Giusti 2025) — closest architectural match (MQTT pub/sub, versioned capabilities). Patterns transfer directly to NATS.

### From Rounds 5-6 (Now Strengthened by R7)
7. **MaAS automatic team composition** — 52 citations, 6-45% cost of static designs. Now strengthened by AutoMaAS operator lifecycle (R6) and AdaptOrch topology routing (R7d). Moving toward high-confidence.
8. **MAS^2 recursive self-generation** — tri-agent meta-system, 19.6% improvement. Single paper but corroborated by MaAS direction.
9. **OSC Collaborator Knowledge Models** — 128-dim CKMs, cognitive gap analysis. Strengthened by R6 deep dive but still single research group.
10. **AWorld profile-aware supervision** — -57.4% variance. Strengthened by R6 deep dive. Moving toward high-confidence.
11. **Cognitive Load-Aware Inference** — 45% token reduction. Strengthened by TALE 67% reduction (R6). Moving toward high-confidence.

### From Round 7 (New)
12. **PrefillShare shared KV cache** — eliminates redundant prefill across multi-model agent handoffs. Significant latency/throughput improvements. Single research group (R7d).
13. **Biomimetic fault tolerance / digital immune system** — consensus-based threat validation, sub-millisecond Byzantine voting. Novel paradigm, limited deployment data (R7d).
14. **Game-theoretic mechanism design** — Proof-of-Thought, auction-based task allocation, Subgame Perfect Nash Equilibrium for cooperation. Multiple papers but no production deployment (R7d).
15. **SECP bounded self-modification** — protocols that can modify themselves within provable safety bounds. Single paper (R7a).
16. **Joint attention for agent coordination** — aligning agent focus improves coordination (Lee et al., R7c).
17. **COWPOX defense against infectious jailbreaks** — edge-layer monitoring + curing samples. Novel but ICML 2025 poster (R7d).

---

## Explicitly Not Pursuing

| Topic | Reason | Revisit? |
|-------|--------|----------|
| Proprietary agent communication protocols | Standards resolved (MCP + A2A). Building proprietary is strategically wrong. | No |
| Direct LLM-to-LLM chat orchestration | Mathematically proven inferior to stigmergy/blackboard (Vellinger 2025). Wrong approach, not a build problem. | No |
| Quantum-inspired optimization | No applicable results across 2,000+ papers in 2025-2026 literature. The math doesn't help. | If breakthroughs emerge |

### Reclassified (Previously Dismissed, Now Active)

| Topic | Old Dismissal | Why It Was Wrong | New Status |
|-------|--------------|-----------------|------------|
| **Neuromorphic computing** | "No Rust/NATS integration path" | The concepts are already partially embedded in the architecture under different names. NATS pub/sub IS spike-based communication. JetStream KV watches ARE neuromodulatory signals. Guard/Advisor IS homeostatic plasticity. "No Rust impl" is not a valid blocker — 19 crates were built in 2 days. | Active — map concepts explicitly to existing primitives |
| **IronFleet formal verification** | "Multi-year effort" | Full system proof remains disproportionate, but MPST session types give 80% of the benefit. A meaningful subset (key invariants for critical protocols) is achievable in 2-4 weeks with agent assistance. | MPST first (days), targeted formal verification subset if needed (weeks) |
| **Neural paging controller** | "Research-only, no production impls" | "No one's done it" didn't stop the rest of the framework from being built. Interface design is days; learned eviction prototype is 1-2 weeks. | Interface design in Phase 9, prototype in Phase 10 |

### Neuromorphic Concept Mapping

The dismissal of "neuromorphic computing" conflated specialized hardware (Intel Loihi, IBM TrueNorth — genuinely irrelevant) with computational concepts that map directly to the architecture:

| Neuromorphic Concept | What It Actually Is | Mister Smith Analog | Status |
|---------------------|--------------------|--------------------|--------|
| Spike-based communication | Event-driven, fire only on threshold | NATS pub/sub + event-triggered consensus | Already built |
| Lateral inhibition | Competing neurons suppress neighbors | Anti-conformity in debate, agent selection | Identified, needs impl |
| Hebbian learning | "Fire together, wire together" | Dynamic team formation (MaAS/AutoMaAS) — agents that succeed together get paired | Covered in orchestration |
| Sparse distributed representations | Efficient encoding with few active units | VCV capability vectors + HNSW, CRDT state | Covered in coordination |
| Homeostatic plasticity | Self-regulation to maintain stable activity | Predictive supervision, Guard/Advisor layer | Covered in supervision |
| Neuromodulation | Global signals that change network behavior | Control plane signals changing routing via JetStream KV watches | Covered in routing |

**Lesson learned**: "No Rust implementation exists" is never a valid reason to dismiss a concept. The team builds Rust implementations. Dismiss only when the underlying approach is mathematically inferior or strategically wrong.

---

## What Remains Open

### Research Complete — Needs Synthesis Only
All deep dives are done. No more research rounds planned. The following need consolidation into actionable architecture:

- [x] Dynamic self-organization & meta-orchestration (R6)
- [x] CRDT-based coordination over JetStream (R6)
- [x] Step-level intelligence / PRMs + CLAI (R6)
- [x] Predictive supervision / profile-aware (R6)
- [x] Inter-agent security & content validation (R6)
- [x] Competitive landscape & Rust ecosystem (R7)
- [x] Scaling laws & production patterns (R7)

### Not Yet Researched (Lower Priority)
- [ ] **AI-native observability + provenance** — AgentOps pipeline, eBPF bridging, W3C PROV-AGENT
- [ ] **Decentralized DAG execution + OTP integration** — Covered partially in self-org R6; needs design
- [ ] **Knowledge-aware semantic routing** — KB signals, DAAO, KABB; extends routing topic

### Evidence Gaps (Require Experimentation, Not Research)
- No published NATS JetStream latency benchmarks for LLM streaming workloads → **build and benchmark**
- Capability-based security (Macaroons/ZCAP-LD) for AI agents → **build in Rust** (macaroons crate exists for primitives; agent-specific delegation logic is ~1-2 weeks)
- RouteLLM's 85% cost savings not independently replicated → **benchmark with Mister Smith router**
- Stigmergic coordination at scale (>100 LLM agents) → **simulate with mock agents first**
- Neural paging with JetStream as backing store → **entirely novel, prototype in Phase 10**
- CRDT coordination at agent scale → **benchmark with Diamond-types crate + JetStream KV**
- PRM accuracy on non-mathematical reasoning tasks → **evaluate with Mister Smith's agent roles**
- Step-level routing latency overhead in production → **measure after Phase 10 impl**

**Note on "no implementation exists" gaps**: The absence of an existing implementation is a build task, not a research gap. With demonstrated velocity (19 crates, 983 tests, 8 phases in 2 days with agent assistance), any of these can be prototyped in days to weeks.

---

## Consolidated Synthesis Documents

Being produced now at `docs/research-output/consolidated/`:

| # | Topic | Source Rounds | Status |
|---|-------|--------------|--------|
| 00 | **Master Findings** (top 20 ranked + roadmap) | All | COMPLETE |
| 01 | Model Routing & Cost Optimization | R3, R4, R5, R6, R7 | COMPLETE |
| 02 | Orchestration & Self-Organization | R3, R4, R5, R6, R7 | COMPLETE |
| 03 | Supervision & Resilience | R3, R4, R5, R6, R7 | COMPLETE |
| 04 | Security & Trust | R4, R5, R6, R7 | COMPLETE |
| 05 | Coordination & State | R3, R4, R5, R6, R7 | COMPLETE |
| 06 | Streaming Architecture | R3, R4, R6, R7 | COMPLETE |
| 07 | Memory & Context | R3, R4, R5, R7 | COMPLETE |
| 08 | Competitive Landscape & Ecosystem | R3, R4, R5, R7 | COMPLETE |

---

## Corpus Location & Navigation

| Document | Purpose |
|----------|---------|
| `docs/research-output/CLAUDE.md` | Directory structure, topic map, reading order |
| `docs/research-output/ROUTING_MANIFEST.md` | Discovery finding routing, strategic classification |
| `docs/RESEARCH_CHECKPOINT.md` | This file — confidence tiers, pending queue, state of knowledge |
| `docs/research-output/consolidated/` | Authoritative per-topic synthesis (in progress) |

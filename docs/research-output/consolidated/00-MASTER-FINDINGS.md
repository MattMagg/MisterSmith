# Master Findings — Ranked by Strategic Impact

> **Date**: 2026-03-07
> **Scope**: All 7 research rounds, 2,000+ papers, 500+ industry references
> **Purpose**: The single document to read for "what did we learn and what do we build?"

---

## Top 20 Findings — Ranked by Strategic Advantage for Mister Smith

### Tier 1: Category-Defining (No Competing Framework Does This)

**#1. Dynamic Topology Selection + Meta-Orchestration**
- **What**: AdaptOrch shows double-digit % improvements from topology routing alone (identical models). MaAS achieves 0.5-12% performance gains at 6-45% cost of static designs. MAS^2 recursively generates bespoke agent architectures per problem (+19.6%).
- **Why #1**: This makes Mister Smith's 9-role static team obsolete in the best way — by layering dynamic composition on top. No framework does this.
- **Evidence**: HIGH — 3+ independent research groups (AdaptOrch, MaAS, MAS^2, AutoMaAS), validated across R4/R5/R6/R7d
- **Action**: Layer topology compiler over existing orchestrator. Phase 1: dependency-graph analysis + parallel/sequential selection. Phase 2: AutoMaAS operator lifecycle. Phase 3: MAS^2 generation.
- **Consolidated doc**: `02-orchestration-and-self-organization.md`

**#2. Step-Level Intelligence (PRMs + Speculative Decoding + Token Budgeting)**
- **What**: BiPRM detects 37.7% more errors at 5% latency cost. RSD achieves 4.4x FLOP reduction via start-cheap-escalate-on-rejection. CLAI/TALE achieve 45-67% token reduction. Streaming monitors detect failures at 18% of tokens.
- **Why #2**: This is a new granularity — between task-level routing and token-level streaming. Changes cost model, quality model, and failure detection simultaneously.
- **Evidence**: HIGH — RSD (63 citations), CLAI/TALE validated, BiPRM published 2025
- **Action**: Phase 1: step boundary detection in streaming pipeline. Phase 2: lightweight PRM (1.5B) for verification. Phase 3: RSD speculative decoding with dynamic model escalation.
- **Consolidated doc**: `01-model-routing-and-cost-optimization.md`, `06-streaming-architecture.md`

**#3. CRDT-Based Observation-Driven Coordination**
- **What**: CodeCRDT achieves 100% convergence, zero merge failures in 600 trials. Delta-CRDTs over JetStream KV provide lock-free, partition-tolerant coordination. Stigmergy-RL formal equivalence proves mathematical soundness.
- **Why #3**: Different coordination paradigm. Agents modify shared state and observe — no explicit message passing needed. Maps directly to JetStream KV.
- **Evidence**: HIGH — CodeCRDT empirical, formal proof (Vellinger), Diamond-types 4.6M ops/sec in Rust
- **Action**: Prototype OR-Set capability registry + G-Counter token budget on JetStream KV. Test hybrid model (CRDTs for state, pub/sub for events).
- **Consolidated doc**: `05-coordination-and-state.md`

**#4. Predictive Supervision (Beyond OTP Restart)**
- **What**: AWorld fingerprints reduce variance 57.4% via offline profiling. OSC 128-dim CKMs enable agents to model each other's cognitive states. MetaOrch fuzzy eval achieves 86.3% intervention accuracy. Guard/Advisor layer sits over OTP.
- **Why #4**: Extends Mister Smith's unique OTP advantage into territory no framework — including Erlang/OTP itself — has explored. Predictive, not reactive.
- **Evidence**: MODERATE-HIGH — AWorld, OSC, MetaOrch independently validate the approach
- **Action**: Phase 1: offline agent profiling → performance fingerprints in JetStream KV. Phase 2: Guard actor with rule-based interventions. Phase 3: CKM-based cognitive coordination.
- **Consolidated doc**: `03-supervision-and-resilience.md`

**#5. MPST Compile-Time Protocol Safety**
- **What**: Multiparty Session Types in Rust provide compile-time verification that multi-agent choreographies are deadlock-free. Proven in Mozilla Servo. Maps to Rust's affine type system.
- **Why #5**: Eliminates entire classes of coordination bugs at compile time. No runtime overhead. Unique to Rust.
- **Evidence**: MODERATE — proven in production (Servo) but not yet applied to LLM agent systems
- **Action**: Define critical agent protocols (planner→executor→verifier) as global session types. Prototype with `rumpsteak` crate.
- **Consolidated doc**: `05-coordination-and-state.md`

### Tier 2: Leapfrog (We Can Do This Better Than Anyone)

**#6. Defense-in-Depth Security (AgentSandbox + Auth Callouts + Schema Validation)**
- **What**: AgentSandbox reduces ASR from 58.8% to 4.34% (13x improvement). NATS Auth Callouts enable dynamic per-request capability scoping. Rust jsonschema validates at 645x speed of legacy validators.
- **Evidence**: HIGH — AgentSandbox empirical, CVE-2025-30215 documented, Auth Callouts in NATS 2.10+
- **Action**: Phase 1: NATS RBAC audit + patch to 2.11.1+. Phase 2: Auth Callout service for dynamic JWTs. Phase 3: persistent/ephemeral agent separation.
- **Consolidated doc**: `04-security-and-trust.md`

**#7. Infectious Jailbreak Defense (Quarantine Architecture)**
- **What**: "Agent Smith" attack exponentially compromises swarms via shared memory. COWPOX defense uses edge-layer monitoring + curing samples. Mandatory semantic firewalls between actor boundaries.
- **Evidence**: MODERATE — Agent Smith well-documented, COWPOX is ICML 2025 poster
- **Action**: All cross-boundary data transfers through quarantine actors. Never pass raw JetStream KV/PostgreSQL retrievals into agent context without sanitization.
- **Consolidated doc**: `04-security-and-trust.md`

**#8. Two-Plane Router with Hierarchical Budget Enforcement**
- **What**: Separate microsecond data plane (NATS request-reply, ~50us) from control plane (JetStream KV watches). Budget enforcement via KV CAS. Learned routing (RouteLLM) achieves 27-85% cost savings.
- **Evidence**: HIGH — converged across 3 R3 industry reports + R4 academic validation
- **Action**: Ship as MVP in Phase 9. Data plane routes requests, control plane streams config/telemetry. JetStream KV CAS for budget accounting.
- **Consolidated doc**: `01-model-routing-and-cost-optimization.md`

**#9. SLM-Default / LLM-Fallback Economics**
- **What**: 1-12B models with guided decoding match or exceed large models for structured tasks at 10-100x lower cost. 0.5B outperforms GPT-4o with compute-optimal scaling.
- **Evidence**: HIGH — Liu et al. (106 citations), Sharma & Mehta comprehensive review
- **Action**: Default routing policy: start with cheapest model capable of structured output, escalate on PRM rejection. Integrate guided decoding (XGrammar/Outlines) for schema enforcement.
- **Consolidated doc**: `01-model-routing-and-cost-optimization.md`

**#10. Biomimetic / Neuromorphic-Inspired Fault Tolerance**
- **What**: Consensus-based threat validation with Byzantine-robust voting. Peer observer swarms evaluate behavioral health continuously. Sub-millisecond consensus. Rooted in biological immune system patterns — crossregulation, homeostatic plasticity, lateral inhibition.
- **Why it matters**: Neuromorphic computational concepts are not exotic hardware — they're already partially embedded in the architecture under different names. NATS pub/sub IS spike-based communication. JetStream KV watches ARE neuromodulatory signals. Guard/Advisor IS homeostatic plasticity. Making these mappings explicit provides theoretical grounding from a mature field (computational neuroscience) for patterns that emerged organically.
- **Evidence**: TENTATIVE for digital immune system specifically; HIGH for the underlying neuromorphic primitives (event-driven communication, threshold-based activation, lateral inhibition) which are proven in biological and computational neuroscience
- **Action**: Phase 1: Map neuromorphic concepts explicitly to existing NATS/OTP primitives (no new code — just formalize the mapping). Phase 2: Implement lateral inhibition for agent selection (anti-conformity). Phase 3: Consensus-based threat validation as extension to predictive supervision (#4).
- **Consolidated doc**: `03-supervision-and-resilience.md`

**Neuromorphic Concept Mapping** (cross-cutting — applies to #1, #3, #4, #8, #10):

| Neuromorphic Concept | Mister Smith Primitive | Finding It Grounds |
|---------------------|----------------------|-------------------|
| Spike-based communication | NATS pub/sub + event-triggered consensus | #3, #5 — coordination |
| Lateral inhibition | Anti-conformity in debate, competitive agent selection | #1 — orchestration |
| Hebbian learning | MaAS/AutoMaAS — agents that succeed together get paired | #1 — meta-orchestration |
| Homeostatic plasticity | Guard/Advisor layer, predictive supervision | #4 — supervision |
| Neuromodulation | JetStream KV watch → control plane signals | #8 — two-plane router |
| Sparse distributed representations | VCV capability vectors + HNSW | #14 — agent discovery |

### Tier 3: Critical Production Requirements

**#11. Fewer Agents & Tools = Better** (Google scaling laws + Vercel case study)
- Removing 80% of tools: 80%→100% accuracy, 3.5x latency improvement. More agents hurts sequential tasks.
- **Action**: Design Mister Smith's default configurations for minimal viable agent count. Make tool pruning a first-class concern.

**#12. Persistent Quantized KV Cache** (15.7s→0.6s agent resume)
- 4-bit quantized KV cache to disk. Massive latency reduction for agent context restoration.
- **Action**: Investigate for JetStream-backed agent state persistence. Could transform checkpoint/restore economics.

**#13. Actor-Per-LLM-Stream + Dual-Stream Design**
- Lossless semantic stream + best-effort UI stream. Failure isolation via OTP.
- **Action**: Already partially implemented (Phase 3/4). Formalize dual-stream contract in Phase 9.

**#14. Decentralized Agent Discovery (FoA VCVs + HNSW)**
- Versioned Capability Vectors with semantic matching via HNSW. 13x improvement on HealthBench.
- **Action**: Store VCVs in JetStream KV. Build HNSW index for capability matching. Extends A2A Agent Cards concept.

**#15. MAST Failure Taxonomy** (134 citations, 14 failure modes)
- Maps to supervision tree restart strategies per failure category.
- **Action**: Implement failure classification in supervision layer. Different restart strategies per MAST category.

### Tier 4: Strategic Positioning

**#16. Rust 68x CPU / 140x Memory Advantage** (GraphBit benchmarks)
- Validates Rust choice. Competing Python frameworks structurally cannot match throughput.

**#17. Akka Agentic Platform** (25k req/sec, 32ms p99, 15k actors)
- Closest JVM competitor. Good benchmark target for Mister Smith performance validation.

**#18. A2A Protocol Compliance**
- Linux Foundation standard, 100+ enterprise supporters. Mandatory for federation.
- **Action**: Implement A2A adapter in transport layer. Auto-generate Agent Cards from agent registry.

**#19. Permutation-Invariant Context Composition** (PICASO/SSMs)
- Constant-time context scaling via category theory. Zero online processing overhead.
- **Action**: Prototype in 1-2 weeks. The math is well-defined (SSM state averaging); Rust impl is a focused task.

**#20. Game-Theoretic Mechanism Design** (Proof-of-Thought, auctions)
- Self-balancing ecosystems for 1000+ agent coordination via incentive alignment.
- **Action**: Prototype auction-based task allocation over NATS subjects. 1-2 weeks for basic Proof-of-Thought scoring.

---

## Implementation Roadmap Summary

Timeline calibration: Phases 1-8 (19 crates, 983 tests) were built in 2 days with agent assistance. Effort estimates below reflect this demonstrated velocity.

| Phase | What Ships | Findings Applied | Est. Effort |
|-------|-----------|-----------------|-------------|
| **Phase 9 (LLM Providers)** | Two-plane router, health-aware circuit breakers, budget enforcement via KV CAS, SLM-default routing, dual-stream formalization | #8, #9, #13 | Days |
| **Phase 9.1 (Security Hardening)** | NATS RBAC audit, Auth Callout service, schema validation sidecars, quarantine actors | #6, #7 | Days |
| **Phase 10 (Step Intelligence)** | Step boundary detection, lightweight PRM verification, streaming monitors, neural paging interface | #2, #12 | Days |
| **Phase 11 (Dynamic Orchestration)** | Topology compiler, VCV-based agent discovery, dynamic team composition, neuromorphic concept formalization | #1, #10, #14 | Days-week |
| **Phase 12 (Predictive Supervision)** | Agent profiling, Guard/Advisor layer (homeostatic plasticity), MAST failure classification, lateral inhibition for agent selection | #4, #10, #15 | Days-week |
| **Phase 13 (CRDT Coordination)** | OR-Set capability registry, hybrid CRDT/pub-sub model, MPST session type protocols | #3, #5 | Days-week |
| **Phase 14 (Advanced)** | MAS^2 generation, game-theoretic auctions, PICASO context composition, learned neural paging, Macaroon capability delegation | #10, #19, #20 | Weeks |

---

## What We Don't Know (Evidence Gaps)

These require building and measuring, not more research. "No implementation exists" is a build task, not a blocker.

| Gap | What To Do | Effort |
|-----|-----------|--------|
| NATS JetStream latency under LLM streaming | Benchmark with mock LLM streams | Days |
| Macaroons/ZCAP-LD for agent capability delegation | Build on `macaroons` crate + agent-specific attenuation logic | 1-2 weeks |
| RouteLLM 85% cost savings | Replicate with Mister Smith router + real provider traffic | 1 week |
| Stigmergic coordination at >100 agents | Simulate with mock agents on NATS, measure throughput/latency | 1 week |
| CRDT coordination at agent scale | Benchmark Diamond-types + JetStream KV under concurrent agent writes | Days |
| PRM accuracy on non-mathematical reasoning | Evaluate lightweight PRM against Mister Smith's agent roles | 1-2 weeks |
| Step-level routing latency in production | Measure after Phase 10 implementation | After Phase 10 |
| Neural paging with JetStream backing store | Prototype quantized KV cache persistence + eviction policy | 1-2 weeks |

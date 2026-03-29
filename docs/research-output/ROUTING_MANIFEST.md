---
version: R3
created: 2026-03-07
updated: 2026-03-28
---

# Historical Discovery Routing Manifest

Maps every finding from discovery sweeps to existing topics, flags new dimensions, and classifies each finding's strategic value under the frontier-first mandate.

> **Current role**: historical discovery-classification support, not the default corpus entrypoint.
> Canonical merged direction source: `docs/direction.md`
> Read `consolidated/` first for active direction and `analysis/` first for imported-report adjudication.
> Use this file to preserve classification language, discovery-round routing history, and future intake framing.
> **Coverage**: Discovery sweeps R4 and R5 are fully routed below. R7 (a-d) findings have already informed the consolidated authority layer.
> **File naming note**: Discovery sweeps are named by research round (R4 = Round 4, R5 = Round 5, R7 = Round 7).

## How To Use This File Now

- Use `docs/direction.md` when you need the single merged direction source.
- Use `docs/research-output/consolidated/` for Mister Smith's whole-system future direction.
- Use `docs/research-output/analysis/` when deciding whether imported research should enter active planning.
- Use this manifest only when you need discovery classification detail, historical routing context, or a vocabulary for a new research round.
- Do not use this file as the front door for normal planning, specs, or architecture handoffs.

**Classification key:**

| Tag | Meaning |
|-----|---------|
| **EXTEND** | Strengthens existing topic with new evidence or technique |
| **TRANSFORM** | Changes how we think about the topic — not just "more data" but a paradigm shift |
| **NEW** | Represents a dimension not captured by any existing topic |
| **FRONTIER** | Creates strategic advantage if implemented — not available in competing frameworks |
| **INCREMENTAL** | Useful but does not create differentiation |

---

## Discovery Sweep R1 — 15 Themes Routed

### Theme 1: CRDTs for Multi-Agent Coordination
**Classification:** TRANSFORM + FRONTIER
**Routes to:** stigmergy-swarm-coordination, nats-native-agent-patterns, frontier-agent-architecture
**Why TRANSFORM:** CRDTs replace explicit message passing with observation-driven coordination. This is the formal CS analog to stigmergy — agents modify shared state, others observe. CodeCRDT (100% convergence, zero merge failures) and Lattica (full decentralized CRDT framework) together suggest a fundamentally different coordination primitive that maps directly to JetStream KV.
**Strategic value:** No competing agent framework uses CRDTs for coordination. This is a first-mover opportunity. Rust has mature CRDT crates (`crdts`, `automerge`). JetStream KV could be enhanced with CRDT semantics at the infrastructure layer.
**Frontier-first note:** This is not an optimization — it is a different coordination paradigm. Should be treated as a core architectural primitive, not a nice-to-have.

### Theme 2: DAG-Based Parallel Agent Execution
**Classification:** EXTEND + FRONTIER
**Routes to:** agentic-loop-architectures (primary), supervision-llm-fault-tolerance (DAG re-planning on failure)
**Why EXTEND:** DAG execution generalizes the sequential/tree patterns already covered. Flash-Searcher (35% step reduction) and AgentNet (decentralized self-organizing DAGs) provide concrete implementations.
**Strategic value:** Plan-then-Execute with DAG parallelism provides both performance (parallel independent nodes) and security (control-flow integrity against prompt injection — Del Rosario et al.). The P-t-E security angle is underexplored in competing frameworks.
**Frontier-first note:** AgentNet's decentralized DAG is more interesting than centralized DAG planning — agents self-organize rather than following a fixed plan. This aligns with Mister Smith's supervision tree model.

### Theme 3: MaAS / Agentic Architecture Search
**Classification:** NEW + FRONTIER
**Routes to:** NEW DIMENSION — meta-orchestration
**Why NEW:** MaAS (52 citations) operates at a level above all existing topics. It doesn't route models or coordinate agents — it discovers optimal agent team compositions per task. 6-45% of costs while outperforming static designs. AutoMaAS extends this with self-evolving architecture search.
**Strategic value:** This transforms Mister Smith's static 9-role agent registry into a dynamic, self-optimizing system. No competing framework implements automatic architecture search.
**Frontier-first note:** This is the single most transformative finding across both sweeps. If Mister Smith can dynamically compose agent teams per task — selecting roles, models, tools, and topology — it creates a category of capability that doesn't exist in any current framework.

### Theme 4: AgentOps / AI-Native Observability
**Classification:** NEW + FRONTIER
**Routes to:** NEW DIMENSION — agent observability
**Why NEW:** Standard distributed systems observability (OTel, Prometheus) doesn't address non-deterministic AI flows. AgentSight (eBPF, <3% overhead) bridges intent-to-action semantic gap. AgentOps framework defines 6-stage AI-specific operations pipeline.
**Strategic value:** 79% of practitioners cite non-deterministic execution as a major challenge. Without AI-native observability, debugging multi-agent workflows at scale is guesswork.
**Frontier-first note:** This should be built into the framework from the start, not bolted on. The semantic gap (what was the agent trying to do vs. what did it actually do) is a first-class observability requirement.

### Theme 5: Hierarchical/Episodic Memory
**Classification:** EXTEND
**Routes to:** frontier-agent-architecture (tiered memory), targeted-neural-paging
**Why EXTEND:** MIRIX's 6 memory types and H-MEM's hierarchical index validate and refine the tiered memory architecture already recommended by frontier synthesis.
**Strategic value:** INCREMENTAL over existing research direction. The specific memory type taxonomy is useful for implementation.

### Theme 6: Process Reward Models (PRMs)
**Classification:** TRANSFORM + FRONTIER
**Routes to:** intelligent-model-routing (step-level escalation), agentic-loop-architectures (step verification)
**Why TRANSFORM:** PRMs operate at the reasoning-step level — below task-level routing, above token-level streaming. RSD (4.4x FLOP reduction by dynamically choosing model per step) fundamentally changes the cost model. This is not "better routing" — it's a new granularity of intelligence.
**Strategic value:** Step-level verification and dynamic model escalation per reasoning step would be unique to Mister Smith. Competing frameworks route at the task level.
**Frontier-first note:** The combination of PRMs + SLM-default routing creates a system where cheap models handle most steps and expensive models are invoked surgically. This is the cost structure that makes Mister Smith economically viable at scale.

### Theme 7: Cognitive Load-Aware Inference (CLAI)
**Classification:** EXTEND + FRONTIER
**Routes to:** intelligent-model-routing (cost optimization), streaming-architecture (token economics)
**Why EXTEND:** CLAI's three-load taxonomy (intrinsic/extraneous/germane) provides a principled framework for managing LLM token budgets. 45% token reduction without accuracy loss.
**Strategic value:** Complements routing and PRMs — manages the "thinking budget" per task rather than just which model to use.

### Theme 8: Multi-Agent Security Attacks
**Classification:** TRANSFORM
**Routes to:** targeted-capability-security-sandboxing (primary), supervision-llm-fault-tolerance (detection)
**Why TRANSFORM:** 58-100% attack success via inter-agent communication hijacking — even when individual agents resist. This changes the security model: infrastructure security is necessary but not sufficient. The communication channel itself is an attack surface.
**Strategic value:** Critical risk mitigation. All frameworks are vulnerable but none have implemented inter-agent content validation.
**Frontier-first note:** Mister Smith's NATS messaging is both the attack surface and the natural place to implement content-level security. Subject-based namespace isolation + content validation at the transport layer could provide defense-in-depth that HTTP-based frameworks cannot match.

### Theme 9: Provenance Tracking (PROV-AGENT)
**Classification:** NEW + FRONTIER
**Routes to:** NEW DIMENSION — overlaps with security (audit) and observability (tracing) but is distinct
**Why NEW:** Captures the causal chain of agent decisions — which agent decided what, based on which inputs from which other agents. Existing audit logging captures events; provenance captures causation. W3C PROV standard provides interoperable vocabulary.
**Strategic value:** Essential for debugging non-deterministic workflows, regulatory compliance, and trust. The MCP integration path (PROV-AGENT extends W3C PROV with MCP metadata) is directly relevant.

### Theme 10: Consensus-Free Multi-Agent Debate
**Classification:** EXTEND + FRONTIER
**Routes to:** agentic-loop-architectures (coordination patterns)
**Why EXTEND:** Anti-conformity finding (LLMs exhibit groupthink; forcing consensus degrades quality) challenges assumptions about agent agreement. MARS review pattern (author -> independent reviewers -> meta-reviewer) maps to software dev workflows.
**Strategic value:** The insight that consensus is harmful for LLM agents is counterintuitive and actionable. Most frameworks assume consensus = correctness.

### Theme 11: SLM-Default/LLM-Fallback
**Classification:** EXTEND + FRONTIER
**Routes to:** intelligent-model-routing (primary)
**Why EXTEND:** Validates and strengthens the learned routing direction. 1B model outperforms 405B with compute-optimal test-time scaling (Liu et al., 106 citations). Changes economics from "which cloud model" to "local vs. cloud."
**Strategic value:** High. Local inference with guided decoding for structured outputs eliminates API dependency for routine tasks.

### Theme 12: Context Summarization (SUPO, ReSum)
**Classification:** EXTEND
**Routes to:** targeted-neural-paging, frontier-agent-architecture (memory)
**Strategic value:** INCREMENTAL. Concrete implementations of context management already recommended. Event-centric memory connects to provenance.

### Theme 13: Formal Verification of Agent-Generated Code
**Classification:** NEW
**Routes to:** NEW DIMENSION — code quality assurance
**Why NEW:** Not covered by any existing topic. Astrogator (83% correct verification, 92% incorrect detection) and PREFACE provide model-agnostic verification paths.
**Strategic value:** Longer-term. Could integrate via external tools (Lean4, Dafny, proptest).

### Theme 14: Agent Interoperability Protocols (A2A, ACP, ANP)
**Classification:** EXTEND
**Routes to:** frontier-agent-architecture (protocol interop), nats-native-agent-patterns (service mesh)
**Strategic value:** A2A Agent Cards as discovery mechanism over NATS is immediately actionable. ANP's DID-based identity extends JWT auth.

### Theme 15: RL-Trained Agentic Workflows (Flow-GRPO)
**Classification:** EXTEND
**Routes to:** agentic-loop-architectures (loop patterns)
**Why EXTEND:** Flow-GRPO's 4-module decomposition (planner/executor/verifier/generator) validates Mister Smith's role structure. JoyAgents-R1's adaptive memory evolution using RL rewards as supervisory signals is novel.
**Strategic value:** Understanding RL-trained agent models informs role design even if we don't train models ourselves.

---

## Discovery Sweep R2 — 5 Major Themes Routed

### R2 Theme 1: Decentralized DAG Coordination (AgentNet, FoA, DynTaskMAS)
**Classification:** TRANSFORM + FRONTIER
**Routes to:** agentic-loop-architectures, nats-native-agent-patterns
**Strengthens:** R1 Theme 2 (DAG execution) with stronger evidence — AgentNet eliminates centralized orchestrators entirely, FoA introduces Versioned Capability Vectors for semantic routing, DynTaskMAS shows near-linear throughput scaling to 16 agents.
**New elements beyond R1:** FoA's VCVs make agent capabilities machine-searchable via HNSW semantic embeddings — this goes beyond model routing to agent-level capability discovery. Evolving Orchestration (Dang et al.) uses RL-learned "puppeteer" for adaptive agent sequencing — cyclic reasoning structures, not just DAGs.
**Frontier-first note:** The convergence of AgentNet + FoA + DynTaskMAS across independent groups strongly validates decentralized self-organizing topologies. NATS queue groups + subject-based routing provide the infrastructure primitive. This is where Mister Smith's architecture already has an advantage — NATS natively supports the patterns these papers propose over HTTP.

### R2 Theme 2: Recursive Self-Generating Meta-Agents (MAS^2)
**Classification:** NEW + FRONTIER
**Routes to:** Strengthens the MaAS dimension (R1 Theme 3) but goes further
**Why this matters:** MAS^2 proposes a tri-agent meta-system (generator-implementer-rectifier) that recursively generates bespoke MAS architectures per problem with real-time rectification. Up to 19.6% improvement over static MAS on complex benchmarks. This is not architecture search (MaAS) — it is architecture generation. The system doesn't select from known configurations; it invents new ones.
**Frontier-first note:** Combined with MaAS, this points toward a self-evolving orchestration layer where Mister Smith doesn't just optimize agent teams but generates novel agent configurations that human designers wouldn't have conceived. This is a frontier capability.

### R2 Theme 3: Knowledge-Aware Semantic Routing
**Classification:** EXTEND + FRONTIER
**Routes to:** intelligent-model-routing
**New beyond existing routing research:** KB-Aware Orchestration (Trombino et al.) uses privacy-preserving signals from agents' internal knowledge bases for routing — not just task features or model capabilities. DAAO (Su et al.) uses variational autoencoders for difficulty prediction. KABB (Zhang et al.) uses Bayesian bandits for dynamic expert coordination.
**Strategic value:** These approaches go beyond "which model for which task" to "which agent with which knowledge for which aspect of the task." Privacy-preserving routing signals are especially relevant for enterprise deployments.

### R2 Theme 4: Cognitive Synergy & Profile-Aware Supervision
**Classification:** TRANSFORM + FRONTIER
**Routes to:** supervision-llm-fault-tolerance (profile-aware supervision), agentic-loop-architectures (cognitive collaboration)
**Key findings:**
- OSC Collaborator Knowledge Models: Agents dynamically perceive collaborators' cognitive states and adapt communication. This is not message passing — it is team cognition.
- AWorld Profile-Aware Maneuvering: Offline profiling creates "performance fingerprints" per agent, enabling targeted interventions based on known failure patterns rather than generic fallback.
- MetaOrch: Neural selection with fuzzy evaluation for agent selection.
**Frontier-first note:** Profile-aware supervision transforms Mister Smith's supervision trees from reactive (detect failure, restart) to predictive (know this agent's weaknesses, intervene before failure). This is a genuine leap beyond OTP's restart-based model. OSC's cognitive state alignment is frontier-tier — agents that understand each other's knowledge gaps and compensate proactively.

### R2 Theme 5: Event-Triggered Consensus
**Classification:** EXTEND
**Routes to:** nats-native-agent-patterns, stigmergy-swarm-coordination
**Key findings:** Multiple papers from IEEE TASE demonstrate adaptive event-triggered protocols that reduce communication overhead while maintaining consensus. Hybrid time/event mechanisms minimize data exchanges.
**Strategic value:** Directly applicable to NATS-based agent communication — agents only communicate when state changes exceed thresholds, not on every tick. Reduces NATS message volume at scale.

---

## New Dimensions Identified (Not Captured by Existing 6 Topics)

These findings don't fit into the current topic structure. They represent genuinely new architectural dimensions that the next research round should treat as first-class topics.

### 1. Dynamic Self-Organization & Meta-Orchestration
**Sources:** MaAS (R1.3), MAS^2 (R2.2), AgentNet (R1.2/R2.1), FoA (R2.1), Evolving Orchestration (R2.1)
**What it is:** The system that decides how agents collaborate — not a fixed topology but a learned/generated one that adapts per task. Supersedes static team definitions.
**Why it's a new dimension:** Current topics assume a fixed orchestration model and optimize within it. This dimension optimizes the orchestration model itself.

### 2. AI-Native Observability & Provenance
**Sources:** AgentOps (R1.4), AgentSight/eBPF (R1.4), PROV-AGENT (R1.9)
**What it is:** Observability designed for non-deterministic AI agents — semantic gap bridging, reasoning trace correlation, causal chain tracking.
**Why it's a new dimension:** Phase 8 observability is standard distributed systems monitoring. Agent-specific observability requires fundamentally different instrumentation.

### 3. Step-Level Intelligence
**Sources:** PRMs (R1.6), CLAI (R1.7), CRM temporal conditioning (R1.6), difficulty-aware routing (R2.3)
**What it is:** Intelligence that operates at the reasoning-step level — verifying, routing, and budgeting per step within a task, not just per task.
**Why it's a new dimension:** Current routing is task-level. Current streaming is token-level. Step-level is the missing middle that connects them.

### 4. Cognitive Coordination
**Sources:** CRDTs (R1.1), OSC cognitive synergy (R2.4), consensus-free debate (R1.10), profile-aware supervision (R2.4), stigmergy formal equivalence
**What it is:** Agents that coordinate through shared cognition (CRDTs, cognitive state models) rather than explicit message protocols.
**Why it's a new dimension:** Current coordination research is protocol-centric (who sends what to whom). Cognitive coordination is state-centric (what does each agent know about the others).

---

## Cross-Cutting Insight: Neuromorphic Concept Mapping

Several findings from different domains are unified by a single theoretical framework: neuromorphic computation. The architecture is already partially implementing these concepts under different names. Making the mapping explicit provides theoretical grounding from computational neuroscience and opens design patterns from a mature field.

| Neuromorphic Concept | Mister Smith Primitive | Research Finding |
|---------------------|----------------------|-----------------|
| Spike-based communication | NATS pub/sub + event-triggered consensus | Theme 1 (CRDTs), R2 Theme 5 (event-triggered) |
| Lateral inhibition | Anti-conformity in debate, competitive agent selection | Theme 10 (consensus-free debate) |
| Hebbian learning | MaAS/AutoMaAS — agents that succeed together get paired | Theme 3 (MaAS), R2 Theme 2 (MAS^2) |
| Homeostatic plasticity | Guard/Advisor layer, predictive supervision | R2 Theme 4 (AWorld, OSC) |
| Neuromodulation | JetStream KV watch → control plane routing signals | Theme 7 (CLAI), Theme 11 (SLM-default) |
| Sparse distributed representations | VCV capability vectors + HNSW | R2 Theme 1 (FoA VCVs) |

**Evaluation principle**: "No Rust implementation exists" is never a valid reason to dismiss a concept. The team builds Rust implementations. Dismiss only when the approach is mathematically inferior or strategically wrong.

---

## Priority Ranking Under Frontier-First Mandate

Ordered by strategic advantage created — how much distance this puts between Mister Smith and the field.

| Rank | Finding | Classification | Strategic Value |
|------|---------|---------------|-----------------|
| 1 | Dynamic self-organization (MaAS + MAS^2 + AgentNet) | NEW + FRONTIER | Category-defining. No framework does this. |
| 2 | CRDT-based cognitive coordination | TRANSFORM + FRONTIER | Different paradigm. First-mover in agent frameworks. |
| 3 | Step-level intelligence (PRMs + CLAI) | TRANSFORM + FRONTIER | New granularity. Changes cost model fundamentally. |
| 4 | Profile-aware predictive supervision | TRANSFORM + FRONTIER | Leapfrogs OTP restart model. |
| 5 | Inter-agent security + content validation | TRANSFORM | Critical risk. NATS infrastructure advantage. |
| 6 | Decentralized DAG execution | EXTEND + FRONTIER | Strong evidence. NATS-native advantage. |
| 7 | AI-native observability + provenance | NEW + FRONTIER | Required for production at scale. |
| 8 | Knowledge-aware semantic routing | EXTEND + FRONTIER | Goes beyond model routing to agent routing. |
| 9 | SLM-default + guided decoding | EXTEND + FRONTIER | 10-100x cost reduction for structured tasks. |
| 10 | Biomimetic / neuromorphic-inspired fault tolerance | TRANSFORM + FRONTIER | Cross-cutting theoretical grounding. Partially built. |
| 11 | Cognitive synergy / team cognition | TRANSFORM + FRONTIER | Frontier-tier. Moves beyond message passing. |
| 12 | Recursive self-generation (MAS^2) | NEW + FRONTIER | Speculative but paradigm-shifting if realized. |
| 13 | Consensus-free debate + anti-conformity (lateral inhibition) | EXTEND + FRONTIER | Counterintuitive insight. Actionable. |
| 14 | Event-triggered consensus for NATS (spike-based thresholds) | EXTEND | Practical optimization at scale. |
| 15 | Agent interoperability (A2A + Agent Cards) | EXTEND | Standards compliance. Not differentiating. |
| 16 | Formal verification of generated code | NEW | Buildable — Lean4/Dafny integration via external tools. |

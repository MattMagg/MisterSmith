---
version: R8
created: 2026-03-22
type: prompt
tier: 1
timeline: last 2 months (late January 2026 — present)
---

# Deep Research Prompt: Agent Framework Competitive Intelligence

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to define the standard that the agent framework market will converge toward.

The competitive landscape snapshot was consolidated in early March 2026 from 7 research rounds (2,000+ papers, 500+ industry references). The market is structurally transitioning: Python-based frameworks dominate adoption but carry fundamental performance ceilings, Rust-native frameworks are validating Rust as a first-class agent platform, enterprise/JVM platforms offer production-hardened benchmarks, and protocol standardization is consolidating around MCP and A2A. The research question has shifted from "who are the competitors" to "what has changed in the last two months that alters the competitive calculus."

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by existing agent frameworks. Benchmark them. Learn from them. Then exceed them. Pull from distributed systems, ad-tech bidding, CDN routing, trading systems, and telecom switching when those fields offer stronger patterns.

Incremental imitation is failure. Favor well-reasoned designs that create real advantage.

## Research Objective

Survey everything published in the last ~2 months (late January 2026 to present) on agent framework releases, architecture pivots, production deployment data, benchmark results, protocol evolution, developer experience trends, and Rust-native framework developments. The goal is to discover what has changed since our last deep research round (early March 2026) and identify shifts that should influence Mister Smith's strategic positioning and architectural priorities.

This is an open-ended research task. Go beyond the dimensions listed below if you discover promising leads outside them.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The following are established findings from 7 research rounds (2,000+ papers). Treat these as known. Only surface new work on these topics if it significantly contradicts, extends, or supersedes them.

**Python-Based Frameworks**: OpenAI Agents SDK (centralized orchestration, function-calling, tracing, guardrails — OpenAI model lock-in, no fault tolerance), Google ADK (A2A-native, agent cards — Python runtime, Google coupling; `adk-rust` v0.2.1 exists but early-stage), LangChain/LangGraph (DAG-based workflows, largest ecosystem — Python ceiling, abstraction tax), CrewAI (role-based teams — single-threaded Python), AutoGen (conversational multi-agent — direct LLM-to-LLM chat scales poorly, prone to runaway loops), Claude SDK (MCP-native, safety primitives — Claude coupling, limited orchestration).

**Rust-Native Frameworks**: GraphBit (68x lower CPU, 140x lower memory vs Python stacks, deterministic execution, Python interop layer), GraphFlow (type-safe directed-graph workflows, LangGraph patterns in Rust, early-stage), Kameo (Tokio-based actors, local registries, backpressure, deadlock warnings — actor library only, no LLM integration), ZeroClaw (3.4MB binary, <5MB RAM, sub-10ms startup — ultra-minimal edge target), ccswarm (Claude Code CLI orchestration, Git worktree isolation), ai-agents crate (single YAML agent definition), autoagents (multi-agent with LLMs/memory, March 2026), swarms-rs and agentum (production-grade multi-agent orchestration in Rust).

**Enterprise/JVM Platforms**: Akka Agentic Platform (15,000 actors, 25,000 req/sec, 32ms p99 — the concrete benchmark target), Microsoft Agent Framework (Semantic Kernel + AutoGen unification, compliance-first), Strands Agents 1.0 (AWS, streaming-first, no published benchmarks), Aisera Unify (multi-protocol A2A/MCP/AGNTCY), Opulent OS 2.0 (fault-tolerant workflows, sandboxing).

**Scaling Laws and Production Patterns**: Google scaling laws (Kim & Liu, Dec 2025 — 180 configurations, more agents hurts sequential tasks, centralized coordination reduces error amplification 4.4x), Vercel fewer-is-more case study (removing 80% of tools improved accuracy 80%->100%, latency 3.5x), AdaptOrch topology routing (double-digit improvements from orchestration topology alone), DynTaskMAS (near-linear scaling to 16 agents), GNN swarm scaling (4,096 agents), MAS-squared recursive self-generating architectures (+19.6%), persistent KV cache for agent memory (15.7s -> 0.6s context reload on Apple M4 Pro).

**Protocol Standards**: MCP (Anthropic/Linux Foundation, 97M+ downloads, agent-to-tool, JSON-RPC) with 30+ cataloged attack techniques. A2A (Google/Linux Foundation, 100+ enterprise supporters, agent-to-agent, Agent Cards, JSON-RPC 2.0 over HTTP/SSE). ACP (RESTful HTTP, DID-based identity). ANP (open network discovery, W3C DIDs). WebMCP (browser-native, Chrome 146 preview). SECP (self-evolving coordination, Feb 2026). Phased adoption recommended: MCP -> ACP -> A2A -> ANP.

**Mister Smith's Position**: Only framework combining Rust performance, OTP-style supervision, NATS/JetStream native distribution, and model agnosticism. 20 crates, 1115+ tests. Known gaps: developer experience, A2A adapter, published production benchmarks, AI-specific observability, dynamic topology/team sizing.

## Research Dimensions

### 1. New Framework Releases and Architecture Pivots

- Have any major agent frameworks (OpenAI, Google, LangChain, CrewAI, AutoGen, Anthropic) released significant new versions or architectural changes in the last 2 months?
- Have any new agent frameworks launched that were not in our baseline (beyond GraphBit, GraphFlow, Kameo, ZeroClaw, ccswarm, swarms-rs, agentum, autoagents)?
- Are any frameworks pivoting away from Python toward Rust, Go, or compiled languages for performance-critical components?
- Has any framework adopted OTP-style supervision, actor-based architectures, or NATS-native messaging that would narrow Mister Smith's architectural moat?
- What acquisitions, mergers, or strategic partnerships have occurred in the agent framework space?

### 2. Production Deployment Reports at Scale

- Are there new published production deployment data from multi-agent systems running at scale (>10 agents, >1000 req/sec)?
- Have the Akka benchmarks (25k req/sec, 32ms p99, 15k actors) been updated or challenged by other platforms?
- Are there new case studies documenting failures, postmortems, or lessons learned from production agent deployments?
- What new evidence exists on the Google scaling laws (more agents hurts sequential tasks) — have these been replicated, refined, or challenged?
- Have any organizations published total cost of ownership (TCO) comparisons between agent frameworks in production?

### 3. Head-to-Head Benchmark Data

- Are there new benchmark suites or comparative evaluations that pit agent frameworks against each other?
- Have SWE-Bench, AgentBench, GAIA, or similar benchmarks produced new results that reveal architectural advantages or limitations?
- Are there new latency, throughput, memory, or cost benchmarks for agent orchestration (not just LLM inference)?
- Has anyone benchmarked Rust-native agent frameworks (GraphBit, GraphFlow, swarms-rs) against Python frameworks with controlled methodology?
- What new benchmark methodologies have been proposed for evaluating multi-agent systems (beyond single-task accuracy)?

### 4. Protocol Standards Evolution (MCP / A2A / ACP / ANP)

- What new versions, extensions, or breaking changes have been published for MCP, A2A, ACP, or ANP in the last 2 months?
- Has the MCP security landscape evolved — new vulnerability disclosures, new defense standards, or new security extensions?
- What is the current A2A adoption trajectory — which frameworks and platforms have added A2A support since our baseline?
- Are there new interoperability protocols or standards proposals that compete with or complement MCP/A2A?
- Has WebMCP (Chrome 146) progressed beyond preview, and what implications does browser-native tool exposure have for server-side frameworks?

### 5. Developer Experience and Adoption Trends

- What are the current download/adoption metrics for the major agent frameworks (npm, PyPI, crates.io, GitHub stars)?
- Are there new developer experience innovations (CLI tools, IDE integrations, visual builders, debugging tools) from any framework?
- What patterns are emerging in how developers choose between agent frameworks (performance vs ease-of-use vs ecosystem)?
- Has the Rust agent ecosystem seen measurable adoption growth on crates.io or in GitHub activity?
- Are there new surveys, reports, or analyses on developer sentiment toward agent frameworks?

### 6. Enterprise Agent Platform Convergence

- Are enterprise platforms (AWS, Azure, GCP, Salesforce, ServiceNow) converging on common agent infrastructure patterns?
- What new managed agent services have launched from cloud providers in the last 2 months?
- Are enterprises standardizing on specific protocol combinations (e.g., MCP + A2A), and does this create pressure on independent frameworks?
- Have any enterprise compliance or governance standards emerged specifically for agent orchestration?
- What role are observability vendors (Datadog, New Relic, Honeycomb) playing in the agent platform stack?

### 7. Rust-Native Framework Evolution

- Have GraphBit, GraphFlow, Kameo, ZeroClaw, swarms-rs, agentum, or autoagents released significant updates?
- Are there new Rust crates relevant to agent orchestration (actor systems, LLM bindings, protocol implementations, inference runtimes)?
- Has `adk-rust` (Google ADK Rust port) progressed beyond v0.2.1 with meaningful new capabilities?
- Are there new Rust-native inference runtimes or developments in candle, burn, mistral.rs, or ort bindings?
- Has anyone published performance comparisons between Rust agent frameworks (not just Rust vs Python)?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations (authors, year, venue, DOI/URL if available)
2. **Key techniques** — the specific algorithms, architectures, or patterns discovered
3. **Applicability to Rust + NATS** — how well does each technique transfer to a Rust actor system with NATS messaging?
4. **Delta from baseline** — what is genuinely NEW versus what we already know?
5. **Implementation complexity** — rough assessment of effort and prerequisites
6. **Expected impact** — what improvement does this offer over the current Mister Smith competitive position?

## Synthesis

After completing all dimensions, provide a synthesis that:
- Ranks the top 5 findings by strategic value for Mister Smith
- Identifies which current architectural assumptions are challenged
- Recommends specific next actions (prototype, benchmark, adopt, monitor)
- Notes any dimension that yielded thin results (say so rather than padding)

## Research Methodology

1. Search broadly across the last ~2 months (late January 2026 to present). Include arXiv preprints, conference proceedings, blog posts, GitHub releases, and industry reports.
2. Follow promising leads with targeted deep dives — do not stop at the first result
3. Look beyond agent frameworks into adjacent fields (trading, telecom, CDN, ad-tech) for transferable patterns
4. For each technique, assess whether it has been validated in production or is purely academic
5. Be skeptical of marketing claims — look for benchmarks, papers, and real-world results
6. If a dimension yields thin results, say so rather than padding with speculation
7. Cross-reference against the baseline above — only surface work that genuinely extends what we know

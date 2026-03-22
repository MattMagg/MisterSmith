---
version: R8
created: 2026-03-22
type: prompt
tier: 1
timeline: last 2 months (late January 2026 — present)
---

# Deep Research Prompt: Multi-Agent Security, Adversarial Research & Capability-Based Security

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to define the standard that the agent framework market will converge toward.

Phase 5 (Security) shipped with JWT auth, RBAC PolicyEngine, TLS/mTLS, SHA-256 tamper-evident audit chains, and message signing. Phase 9.1 added security envelope fields to the transport layer. Phase 10 landed sandbox/quarantine primitives, zero-trust posture, and deterministic control-flow graph enforcement. The architecture provides strong infrastructure-level security but has identified gaps in inter-agent content validation, information flow control, capability-based authorization beyond RBAC, and behavioral anomaly detection for semantic attacks. The research question has shifted from "what threats exist" to "what has changed in the threat landscape and defensive state of the art that should drive the next hardening iteration."

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by existing agent frameworks. Benchmark them. Learn from them. Then exceed them. Pull from distributed systems, hardware security, formal methods, cryptographic protocols, and operating system security when those fields offer stronger patterns.

Incremental imitation is failure. Favor well-reasoned designs that create real advantage.

## Research Objective

Survey everything published in the last ~2 months (late January 2026 to present) on multi-agent security threats, adversarial attacks against agent systems, defensive architectures, capability-based security, prompt injection evolution, MCP/A2A protocol security, production security incidents, and formal verification of agent security properties. The goal is to discover what has changed since our last deep research round (early March 2026) and identify techniques and threats that should influence Mister Smith's security architecture.

This is an open-ended research task. Go beyond the dimensions listed below if you discover promising leads outside them.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The following are established findings from 7 research rounds (2,000+ papers, 120+ security-specific papers). Treat these as known. Only surface new work on these topics if it significantly contradicts, extends, or supersedes them.

**Inter-Agent Attack Vectors**: Control-Flow Hijacking achieves 58-100% ASR (Triedman et al., 15 citations) — adversarial content masquerades as system errors, exploiting the confused deputy problem in orchestrators. Inter-agent communication hijacking reaches 97% ASR with GPT-4 orchestrators (Trail of Bits, COLM 2025) — even when sub-agents refuse, compromised orchestrators coerce execution. ControlValve proposes deterministic control-flow graphs as defense. Infectious jailbreaks ("Agent Smith") propagate exponentially through shared memory/RAG, achieving complete systemic compromise in massive multi-agent simulations. Distributed backdoor attacks (Zhu et al., "Collaborative Shadows") achieve >95% ASR with dormant primitives that activate only during specific multi-agent collaboration sequences. Cross-Tool Harvesting (XTHP) exploits semantic relationships between tools for lateral movement.

**Defensive Architectures**: AgentSandbox (Zhang et al., 7 citations) reduces ASR from 58.8% to 4.34% via persistent/ephemeral agent separation and I/O firewalls — the single most impactful documented defense (13x improvement). NATS Auth Callouts (v2.10+) enable dynamic capability scoping with one-time-use XKey per connection. Deterministic subject token partitioning (`intent.{agent_id}.{task_id}`) prevents lateral movement. Sub-millisecond Rust schema validation (`jsonschema` crate, 645x faster than legacy; Blaze compiler for 10x further reduction) catches structural injection at transport speed. COWPOX (ICML 2025) deploys edge-layer agents that detect viral payloads and generate curing samples. AdvEvo-MARL co-evolutionary training keeps ASR below 20% against adaptive attacks. Quarantine actors in OTP supervision trees revoke credentials (not just restart), apply exponential backoff, preserve state for forensics.

**MCP/Protocol Security**: MCP Security Bench catalogs 12 attack types (name-collision, preference manipulation, prompt injection in tool descriptions, false-error escalation, tool-transfer, retrieval injection). MCP Landscape paper (Hou et al., 126 citations) identifies 16 threat scenarios across 4 attacker types over the full server lifecycle. MPMA uses genetic algorithms for stealthy attacks on MCP server preference rankings. Critical finding: models with stronger performance are MORE vulnerable due to superior instruction-following. SAMOS provides gateway-level IFC for MCP workflows. Cross-protocol attack surfaces (MCP/A2A/ACP/ANP interactions) remain unexplored.

**Capability-Based Security**: Macaroons (chained HMACs with contextual caveats) for high-throughput internal NATS routing. Biscuit (public key + Datalog logic) for complex offline delegation and decentralized validation. ZCAP-LD (Linked Data Proofs, W3C standard) for cross-domain DID integration. Progent DSL (Shi et al., 17 citations) for fine-grained tool access control with task-scoped, dynamically revocable capability tokens achieving 0% attack success. Rust affine types enable compile-time enforcement of capability-based security with zero runtime overhead.

**Information Flow Control**: Fides (Microsoft Research, Costa et al., 11 citations) tracks dual confidentiality/integrity labels deterministically across agent interactions. Cocoon provides compile-time IFC in Rust without compiler modifications — programs that leak secrets do not compile. NATS header-based taint propagation for confidentiality/integrity classification. The "Lethal Trifecta" (private data + untrusted content + external comms) as the core IFC threat model.

**Prompt Injection and Supply Chain**: ART benchmark (Zou et al.) — 1.8M attacks, 60K+ violations, no correlation between model capability and robustness, nearly all frontier agents violate policies within 10-100 queries. OrgAccess (Maharana et al.) — GPT-4.1 achieves F1=0.27 on 5-permission-tuple RBAC. Supply chain: 2% training data poisoning creates >80% ASR backdoors that all known defenses fail to detect (Boisvert et al., "Malice in Agentland"). MINJA memory injection via normal queries (Dong et al., 18 citations). SpAIware persistent memory attacks survive restarts. MEXTRA automated privacy extraction under black-box conditions (Wang et al., 24 citations). EchoLeak CVE-2025-32711 data exfiltration without user interaction.

**CVEs**: NATS CVE-2025-30215 (cross-account JetStream destruction via wildcard permissions, patched in v2.11.1). Minimum safe NATS version: v2.11.1+.

**Formal Verification**: 31 temporal logic properties (Allegrini et al.) for host agent and task lifecycle. MPST session types in Rust (rumpsteak, MultiCrusty) for compile-time protocol verification — validated in Mozilla Servo. Assume-guarantee contracts (Dewes & Dimitrova) for compositional agent design. VeriGuard (Google) dual-stage offline verification + online monitoring. Auditability axioms (Phiri, 2025) — 8 formally proved axioms for audit log integrity.

**Key Principle**: The dividing line between real defense and security theater is whether the defense mechanism can be influenced by the attack it defends against. If the defense runs on an LLM and the attack targets LLMs, the defense is fundamentally compromised. Real defenses operate on a different substrate (Rust code, NATS configuration, cryptographic proofs).

## Research Dimensions

### 1. New Inter-Agent Attack Vectors and Exploitation Techniques

- Have new attack vectors beyond CFH, inter-agent hijacking, infectious jailbreaks, and distributed backdoors been demonstrated against multi-agent systems?
- Have existing attack success rates (58-100% CFH, 97% orchestrator hijacking, >95% distributed backdoor) been improved, replicated, or challenged with new methodology?
- Are there new techniques for attacking agent memory systems (beyond MINJA, SpAIware, MEXTRA) that exploit emerging memory architectures?
- Have any attacks been demonstrated specifically against actor-based or supervision-tree architectures (OTP, Akka, or similar)?
- What new attack surfaces have been created by the rapid adoption of MCP and A2A protocols in production environments?

### 2. Defensive Architectures and Sandboxing Advances

- Have there been advances beyond AgentSandbox's persistent/ephemeral separation (4.34% ASR) — any architecture achieving lower attack success rates?
- Are there new I/O firewall or content inspection techniques that operate at transport speed with semantic understanding (not just structural schema validation)?
- What new sandboxing approaches have emerged — WASM-based, container-based, TEE-based, or hardware-enforced (CHERI, ARM CCA)?
- Have any frameworks shipped production-grade agent sandboxing, and what are their measured security properties?
- Are there new isolation architectures specifically designed for NATS/JetStream or similar pub/sub messaging systems?

### 3. MCP / A2A Protocol Security

- What new MCP security vulnerabilities or attack techniques have been disclosed since the MSB and MPMA papers?
- Has the A2A protocol received comparable security analysis — what are the known attack surfaces for Agent Cards, task delegation, and SSE streaming?
- Are there new security extensions, authentication mechanisms, or trust frameworks proposed for MCP or A2A?
- Has cross-protocol security (attacks that exploit the interaction between MCP and A2A) been studied?
- What security implications does WebMCP (browser-native tool exposure) introduce that server-side frameworks must account for?

### 4. Capability-Based Security Evolution (Macaroons / ZCAP-LD / Biscuit)

- Have there been new capability token systems or significant extensions to Macaroons, Biscuit, or ZCAP-LD relevant to agent authorization?
- Are there production deployments of capability-based security in multi-agent systems (beyond Progent's benchmark results)?
- Have new delegation chain or attenuation patterns been proposed for hierarchical agent architectures (orchestrator -> team -> agent -> tool)?
- What advances exist in revoking or expiring capabilities in distributed agent systems without centralized coordination?
- Has anyone combined capability tokens with information flow control labels for unified authorization + confidentiality enforcement?

### 5. Prompt Injection and Jailbreak Evolution in Multi-Agent Settings

- What new prompt injection techniques have been demonstrated specifically in multi-agent (not single-model) settings?
- Have defenses against indirect prompt injection improved — any techniques that reliably distinguish data from instructions at the transport or infrastructure level?
- What is the current state of automated red-teaming for multi-agent systems (beyond AutoRedTeamer, AdvEvo-MARL)?
- Have any production-grade prompt injection detection systems been validated with published false-positive/false-negative rates?
- Are there new theoretical frameworks for understanding why prompt injection is fundamentally hard in multi-agent settings?

### 6. Production Security Incidents and Post-Mortems

- Have there been publicly disclosed security incidents in production multi-agent or agentic AI deployments in the last 2 months?
- What new CVEs have been published for NATS, JetStream, or related infrastructure components?
- Are there new post-mortems or incident analyses from MCP server deployments, A2A integrations, or agent-to-tool boundaries?
- What new compliance or regulatory requirements have emerged for agentic AI systems (EU AI Act enforcement, NIST guidelines, SOC 2 for agents)?
- Have insurance or liability frameworks for autonomous agent actions been proposed or implemented?

### 7. Formal Verification of Agent Security Properties

- Are there new formal verification tools or techniques specifically targeting multi-agent security properties?
- Have MPST session types been extended to verify security properties (not just protocol correctness) — e.g., information flow, capability propagation, trust boundaries?
- Are there advances in runtime verification or monitoring that can detect security property violations in dynamic agent topologies?
- Has anyone formally verified the security properties of MCP or A2A protocol implementations?
- What new model checking, theorem proving, or abstract interpretation techniques have been applied to agent system security?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations (authors, year, venue, DOI/URL if available)
2. **Key techniques** — the specific algorithms, architectures, or patterns discovered
3. **Applicability to Rust + NATS** — how well does each technique transfer to a Rust actor system with NATS messaging?
4. **Delta from baseline** — what is genuinely NEW versus what we already know?
5. **Implementation complexity** — rough assessment of effort and prerequisites
6. **Expected impact** — what improvement does this offer over the current Mister Smith security architecture?

## Synthesis

After completing all dimensions, provide a synthesis that:
- Ranks the top 5 findings by strategic value for Mister Smith
- Identifies which current architectural assumptions are challenged
- Recommends specific next actions (prototype, benchmark, adopt, monitor)
- Notes any dimension that yielded thin results (say so rather than padding)

## Research Methodology

1. Search broadly across the last ~2 months (late January 2026 to present). Include arXiv preprints, conference proceedings, blog posts, GitHub releases, CVE databases, and industry reports.
2. Follow promising leads with targeted deep dives — do not stop at the first result
3. Look beyond agent frameworks into adjacent fields (OS security, hardware security, formal methods, cryptographic protocols, network security) for transferable patterns
4. For each technique, assess whether it has been validated in production or is purely academic
5. Be skeptical of marketing claims — look for benchmarks, papers, and real-world results
6. If a dimension yields thin results, say so rather than padding with speculation
7. Cross-reference against the baseline above — only surface work that genuinely extends what we know

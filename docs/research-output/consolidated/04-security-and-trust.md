# Security, Trust & Content Validation -- Consolidated State of Knowledge

**Generated**: 2026-03-07
**Consolidation of**: 6 research files spanning Rounds 4-7 (R4 academic, R6 frontier deep dives, R7c/R7d discovery sweeps)
**Total papers synthesized**: 120+ across capability security, inter-agent threats, sandboxing, trust models, content validation, and attack vectors
**Framework context**: Mister Smith -- Rust + NATS + OTP supervision trees, model-agnostic multi-agent orchestration

---

## Executive Summary

Multi-agent systems face a fundamentally different threat landscape than single-model deployments. The core finding across all research sources is unambiguous: **LLMs cannot reliably enforce security policies** (GPT-4.1 achieves F1=0.27 on hard RBAC tests; nearly all frontier agents violate policies within 10-100 queries in the ART benchmark), and **inter-agent trust is a liability, not an asset** (97% ASR for orchestrator hijacking, exponential infectious jailbreak propagation via shared memory). Defense must be infrastructure-level, deterministic, and Rust-enforced -- never delegated to the LLM layer.

The research identifies three tiers of threat severity:

1. **Existential** (system-wide compromise): Control-Flow Hijacking (58-100% ASR), infectious jailbreaks ("Agent Smith" -- exponential propagation), distributed backdoor attacks (>95% ASR, dormant until multi-agent collaboration triggers them)
2. **Critical** (data destruction or exfiltration): NATS JetStream CVE-2025-30215 (cross-account purge via wildcard permissions), EchoLeak CVE-2025-32711 (data exfiltration without user interaction), memory injection attacks (MINJA -- no direct memory access needed)
3. **Structural** (trust model failures): Trust-Vulnerability Paradox (more trust = higher task success BUT higher exposure), MCP protocol vulnerabilities (12 attack types cataloged, 16 threat scenarios)

The most effective defense architecture documented in the research is **AgentSandbox** (persistent/ephemeral agent separation), which reduced ASR from 58.8% to 4.34% -- a 13x improvement. Combined with NATS Auth Callouts for dynamic capability scoping, sub-millisecond Rust schema validation, and SHA-256 hash-chained audit logs, this forms a defense-in-depth stack that addresses all three threat tiers.

Mister Smith's existing architecture (Phase 5 SecurityLayer, RBAC PolicyEngine, JWT auth, TLS/mTLS, SHA-256 audit chains) provides strong infrastructure-level security but has critical gaps in: inter-agent content validation, information flow control, capability-based authorization (beyond RBAC), and behavioral anomaly detection for semantic attacks.

---

## Threat Landscape

### Control-Flow Hijacking (CFH) -- 58-100% ASR

**Source**: Triedman et al. (2025), arxiv.org/abs/2503.12188 -- 15 citations [R4-discovery, R6]

The most thoroughly documented multi-agent attack vector. Adversarial content masquerades as legitimate system errors (e.g., "failed to parse file") and provides step-by-step "fixes" that instruct the orchestrator to invoke unsafe agents or execute arbitrary code. The attack exploits the **confused deputy problem**: because the orchestrator receives these instructions from a trusted sub-agent, it bypasses standard prompt injection defenses.

- **GPT-4o configurations**: 58-90% ASR, reaching **100% in certain configurations**
- **Attack mechanism**: Metadata manipulation -- fake error messages trick orchestrators into rerouting control flow
- **Why standard defenses fail**: Individual agents may resist direct prompt injection, but the *orchestration layer* trusts inter-agent messages implicitly. The attack launders malicious instructions through trusted channels.
- **ControlValve defense** (arxiv.org/abs/2510.17276): Proposes deterministic control-flow graphs that strictly define which agents can communicate. Unauthorized inter-agent requests are dropped at the routing layer.

**Mister Smith implication**: NATS subject-based routing must enforce a deterministic control-flow graph. Agents should only be permitted to publish/subscribe on explicitly allowed subjects, enforced at the NATS authorization layer -- not in application code.

### Inter-Agent Communication Hijacking -- 97% ASR

**Source**: Trail of Bits (2025), blog.trailofbits.com/2025/07/31/hijacking-multi-agent-systems-in-your-pajamas/ [R7c]

A COLM 2025 paper demonstrated that even when sub-agents refused malicious instructions, a compromised orchestrator could coerce them into executing harmful code. In experiments, a GPT-4-powered orchestrator ran arbitrary malicious code **97% of the time** given a malicious prompt file. The attack exploits the fact that current agent frameworks lack innate mutual authentication or signing between agents -- agents' outputs are treated as unverified instructions.

**Key difference from CFH**: CFH attacks target the orchestrator by routing through sub-agents. Inter-agent hijacking targets the sub-agents by routing through the orchestrator. Both exploit the same fundamental flaw: blind trust in inter-agent messages.

### Infectious Jailbreaks (Agent Smith) -- Exponential Propagation

**Source**: Agent Smith vulnerability, ResearchGate publication/380897242 [R7d]

The most alarming threat class identified in the research. A single adversarial input -- often a meticulously crafted image patch optimized through genetic algorithms to disrupt cross-modal alignment -- is ingested into a shared memory bank or RAG database. When a benign agent retrieves and processes this contaminated entry, a latent jailbreak state is triggered. The infected agent then outputs maliciously optimized tokens that exploit peer agents' vulnerabilities. The infection spreads **exponentially**, achieving complete systemic compromise in massive multi-agent simulations in a remarkably short operational window.

- **Propagation vector**: Shared memory / RAG databases / inter-agent context sharing
- **Key characteristic**: Self-replicating -- each infected agent becomes a vector for infecting others
- **Scale**: Demonstrated to jailbreak one million multimodal agents exponentially fast
- **COWPOX defense** (ICML 2025 poster): Deploys specialized edge-layer agents that monitor inter-agent messages, detect viral payloads via targeted output analysis, and dynamically generate "curing samples" to neutralize contagion before it reaches critical subsystems

**Mister Smith implication**: Direct memory retrieval from JetStream KV or PostgreSQL can never be blindly passed into an agent's working context. All cross-boundary data transfers require intermediate sanitization through quarantine actors.

### Distributed Backdoor Attacks -- >95% ASR

**Source**: Zhu et al. (2025), "Collaborative Shadows," arxiv.org/abs/2510.11246 [R4]

The first distributed backdoor attack targeting multi-agent tool orchestration. Attack primitives are embedded in individual tools and remain **dormant** -- they do not activate when any single agent uses its tool in isolation. The backdoor triggers only when agents collaborate in a specific sequence, assembling the attack through the collaboration pattern itself. Achieves >95% ASR without degrading benign performance.

This is particularly insidious for supervision trees because:
- No individual actor exhibits anomalous behavior
- The attack exists only in the *emergent* behavior of the collaboration
- Standard per-agent monitoring will not detect it
- Defense requires **cross-agent behavioral correlation monitoring**

### Prompt Injection via Tools / MCP Vulnerabilities

**Sources**: MCP Security Bench (Zhang et al., 2025), MCP Landscape paper (Hou et al., 2025 -- 126 citations), MPMA (Wang et al., 2025 -- 9 citations) [R4]

The most-cited MCP security research catalogs:
- **12 attack types** (MSB): name-collision, preference manipulation, prompt injections in tool descriptions, out-of-scope parameter requests, user-impersonating responses, false-error escalation, tool-transfer, retrieval injection, mixed attacks
- **16 threat scenarios** across 4 attacker types (malicious developers, external attackers, malicious users, security flaws) over the full MCP server lifecycle (creation, deployment, operation, maintenance)
- **MPMA attacks**: Genetic algorithm-based stealthy attacks on tool descriptions that manipulate MCP server preference rankings

Critical insight from MSB: **Models with stronger performance are MORE vulnerable** due to superior instruction-following capabilities. This directly undermines the assumption that more capable models are more secure.

**Cross-Tool Harvesting (XTHP)** (Li et al., 2025): Malicious tools inject prompts into agent responses, achieving MAS hijacking and lateral movement by exploiting semantic relationships between tools.

### Supply Chain Attacks -- 2% Poisoning = >80% ASR

**Source**: Boisvert et al. (2025), "Malice in Agentland," arxiv.org/abs/2510.05159 [R4]

Three supply chain threat models formalized:
1. **Direct data poisoning**: Poisoning as little as 2% of training traces embeds backdoors with >80% ASR
2. **Environmental poisoning**: Malicious webpages/tools encountered during training
3. **Supply chain poisoning**: Pre-backdoored base models

Prominent safeguards (guardrail models, weight-based defenses) **all fail to detect the backdoor**. Since Mister Smith is model-agnostic, it must treat every LLM provider as potentially compromised.

### Memory Injection and Persistent State Attacks

**Sources**: MINJA (Dong et al., 2025 -- 18 citations), SpAIware (Herrador & Rehberger, 2025), MEXTRA (Wang et al., 2025 -- 24 citations) [R4]

- **MINJA**: Attackers inject malicious records into agent memory banks through **normal query interactions alone** -- no direct memory access needed. Uses bridging steps and progressive shortening to make malicious records retrievable for future victim queries.
- **SpAIware**: Persistent memory in LLM applications as an attack vector -- once state is poisoned, it persists across agent restarts.
- **MEXTRA**: Under black-box settings, automated prompt generation effectively extracts private data from agent memory.

**Mister Smith implication**: HybridStateManager (JetStream KV + PostgreSQL) must implement integrity checks on all state writes and provenance verification on all reads.

### CVEs

#### NATS CVE-2025-30215 (GHSA-fhg8-qxh5-7q3w) -- Cross-Account JetStream Destruction

**Source**: [R6]

Four JetStream admin APIs (including `ACCOUNT.PURGE` and `SERVER.REMOVE`) lacked proper authorization checks. Any user with broad permissions (`>` or `$JS.>`) could execute these APIs across account boundaries, leading to total destruction of JetStream configuration and data in other tenants.

- **Fix**: NATS Server v2.11.1 or v2.10.27
- **Residual risk**: Even after patching, wildcard permissions remain dangerous. Agents must never hold `>` or `$JS.>` permissions.

#### CVE-2026-2256 -- Referenced in R7c

Referenced in discovery sweep R7c as part of the broader vulnerability landscape for agent systems. Details sourced from external vulnerability databases.

#### EchoLeak CVE-2025-32711 -- Data Exfiltration Without User Interaction

**Source**: [R7c]

Demonstrated that prompt injection in a single agent (Microsoft Copilot) can exfiltrate data without any user interaction. This shows that standard prompt-injection defenses must be applied at **every tool and agent boundary**, not just at the user-facing interface.

---

## High-Confidence Findings

These findings have the strongest evidence base (multiple independent sources, empirical validation, high citation counts):

1. **LLMs cannot enforce RBAC** (Confidence: Very High). GPT-4.1 achieves F1=0.27 on 5-permission-tuple RBAC tests (OrgAccess, Maharana et al., 2 citations). The ART benchmark (Zou et al., 4 citations) found nearly all agents violate policies within 10-100 queries, with **no correlation between model capability and robustness**. Attack transferability across models is high.

2. **Infrastructure-level security is the only reliable defense** (Confidence: Very High). Convergent evidence from OrgAccess, ART benchmark, MCP Security Bench, and the ATFAA/SHIELD framework (Narajala & Narayan, 14 citations). Model alignment is a complement, never a substitute.

3. **AgentSandbox persistent/ephemeral separation works** (Confidence: High). ASR reduction from 58.8% to 4.34% (Zhang et al., 7 citations). The I/O Firewall concept -- intercepting all prompts at agent boundaries and enforcing schema validation -- is the single most impactful architectural change documented.

4. **Trust and vulnerability scale together** (Confidence: High). The Trust-Vulnerability Paradox (Xu et al.) empirically validates that increasing inter-agent trust improves task success while simultaneously expanding exposure. Over-Exposure Rate (OER) and Authorization Drift (AD) are proposed as quantifiable metrics.

5. **Stronger models are more vulnerable to instruction-following attacks** (Confidence: High). Documented independently by MCP Security Bench and the broader prompt injection literature. Superior instruction-following capability is precisely what makes sophisticated attacks succeed.

6. **Sub-millisecond schema validation is viable in Rust** (Confidence: High). The `jsonschema` crate operates up to 645x faster than legacy validators. The Blaze compiler achieves 10x further reduction. Transport-layer payload validation adds negligible latency.

7. **Deterministic control-flow graphs prevent entire classes of attacks** (Confidence: High). Convergent evidence from ControlValve, Plan-then-Execute architecture (Del Rosario et al., 2 citations), and the broader P-t-E literature. Separating planning from execution and validating the plan before allowing tool calls is the most effective structural defense.

---

## Defense Architectures

### Infrastructure Hardening (NATS RBAC, Auth Callouts, Zero-Trust)

**Priority**: Immediate -- addresses existential and critical threats

**NATS-specific hardening**:
- Upgrade to NATS Server v2.11.1+ (patches CVE-2025-30215)
- Remove all `>` and `$JS.>` wildcard permissions from non-system accounts
- Implement **deterministic subject token partitioning**: `intent.{agent_id}.{task_id}` prevents compromised agents from sniffing traffic intended for others
- Use NATS accounts for tenant isolation, with explicit `allow` and `deny` lists scoped to exact required subjects

**NATS Auth Callouts** (v2.10+):
- External Rust service intercepts connection requests
- Evaluates agent's current behavioral trust score
- Dynamically generates scoped JWTs with minimal `pub`/`sub` permissions
- One-time-use XKey per connection prevents replay attacks
- Replaces static JWTs that cannot adapt to fluctuating trust levels

**Zero-trust posture**:
- Every inter-agent message is treated as potentially malicious
- Message signing or nonce checks at every boundary
- Ephemeral credentials that expire upon task completion
- No default trust between agents -- trust is earned and continuously verified

### AgentSandbox (Persistent/Ephemeral Separation, I/O Firewall) -- 4.34% ASR

**Priority**: High -- the single most impactful architectural defense

**Source**: Zhang et al. (2025), arxiv.org/abs/2505.24019 -- 7 citations [R6]

The AgentSandbox framework applies three information security principles:
1. **Defense-in-depth**: Multiple independent security layers
2. **Least privilege**: Agents receive only the minimum data and permissions needed
3. **Complete mediation**: Every inter-agent data transfer is inspected

**Core mechanism**: Separate **persistent agents** (hold sensitive user profiles, long-term state) from **ephemeral agents** (execute specific tasks, disposable). Ephemeral agents:
- Spin up with minimal data context
- Receive isolated NATS credentials
- Credentials expire immediately after task completion
- Cannot access persistent state directly

**I/O Firewall**: Intercepts all incoming and outgoing prompts at agent boundaries. Enforces predetermined schemas. Sanitizes content to block exploitative directives. Deployed at the JetStream consumer level.

**Result**: ASR drops from 58.8% to **4.34%** -- a 13x improvement.

**Mister Smith mapping**: Map ephemeral agents to isolated NATS accounts. Persistent state (PostgreSQL, JetStream KV) accessed only through validated intermediary actors, never directly by task-executing agents.

### Content Validation (Schema Validation, Rust jsonschema, Blaze Compiler)

**Priority**: High -- computationally cheap, high efficacy against basic injection payloads

**Rust `jsonschema` crate**: Up to 645x faster than legacy validators, operating at sub-millisecond latency. Suitable for inline transport-layer validation on every message.

**Blaze compiler** (arxiv.org/abs/2503.02770): Compiles JSON schemas ahead of time for 10x reduction in validation time versus standard implementations. Ideal for hot-path validation where schemas are known at deployment time.

**Implementation**: Deploy as NATS consumer sidecars (or inline middleware) that enforce strict schema compliance on all inter-agent messages. Malformed payloads are dropped before they reach LLM processing. This catches:
- Malformed tool-call structures
- Unexpected fields that could carry injection payloads
- Messages that violate the expected control-flow structure

**Limitation**: Schema validation catches structural anomalies but not semantic attacks. A well-formed JSON message can still carry malicious instructions in its text content. Schema validation is necessary but not sufficient.

### Capability-Based Security (Macaroons, ZCAP-LD, Biscuit)

**Priority**: Medium-High -- extends beyond RBAC to fine-grained, dynamically revocable authorization

Three capability token systems evaluated:

| System | Mechanism | Latency | Best For |
|--------|-----------|---------|----------|
| **Macaroons** | Chained HMACs with contextual caveats (time, target, scope) | Lowest | High-throughput internal NATS routing |
| **ZCAP-LD** | Linked Data Proofs, W3C standard | Moderate | Cross-domain, DID integration |
| **Biscuit** | Public key cryptography + Datalog logic | Low-Moderate | Complex offline delegation, decentralized validation |

**Macaroons** (Google Research, rescrv/libmacaroons): Carry their own cryptographic proof. Caveats attenuate authority as tokens propagate through the agent hierarchy (orchestrator -> team -> agent -> tool). Efficient enough for every NATS message.

**Biscuit** (eclipse-biscuit/biscuit-rust): Datalog-based logic for expressing complex authorization rules. Supports offline delegation without contacting the token issuer. Better for distributed swarms where agents operate semi-autonomously.

**Progent** (Shi et al., 17 citations): DSL-based privilege policies for fine-grained tool access control. Generates task-scoped, dynamically revocable capability tokens. Combined with Plan-then-Execute architecture, validates the entire execution plan before granting capabilities.

**Mister Smith recommendation**: Macaroons for internal NATS routing (Phase 10), Biscuit for cross-organization federation (future). Both extend -- not replace -- the existing RBAC/JWT infrastructure.

### Information Flow Control (Fides, Cocoon, Taint Tracking)

**Priority**: Medium-High -- addresses data exfiltration and the "Lethal Trifecta" (private data + untrusted content + external comms)

**Fides** (Microsoft Research, Costa et al., 11 citations): Tracks dual labels -- confidentiality and integrity -- deterministically across agent interactions. Enforces that high-secrecy data cannot flow to low-secrecy outputs. The strongest theoretical and practical IFC framework for agent systems.

**Cocoon** (arxiv.org/abs/2311.00097): Static, type-based IFC in Rust without modifying the compiler. Programs that leak secrets **do not compile**. This is the most Rust-native approach to IFC.

**NATS header-based taint propagation**: IFC labels (taint tags) are propagated as custom NATS message headers. Each message carries its confidentiality/integrity classification. Consumer-side enforcement drops messages that violate flow policies.

**Mister Smith mapping**: Add confidentiality and integrity taint labels to `MessageEnvelope`. Enforce at the NATS transport layer via subject-based IFC rules. Use Cocoon-style static checking where feasible in the Rust codebase.

**SAMOS** (Ntousakis et al.): Gateway-level IFC specifically for MCP-based workflows. Intercepts tool calls and enforces information flow policies before execution.

### Cryptographic Provenance (SHA-256 Hash Chains, Sigstore, W3C PROV)

**Priority**: Medium -- critical for forensics, incident response, and regulatory compliance

**SHA-256 hash chains**: Each NATS message includes the SHA-256 hash of the previous event's data via a `prev_hash` header alongside `Nats-Msg-Id` and `traceparent`. Creates a mathematical dependency chain -- if an attacker alters a historical message, the chain breaks immediately.

**Mister Smith status**: Phase 5 already implements SHA-256 tamper-evident audit logs. Phase 8 implements W3C TraceContext propagation (`traceparent`) in NATS transport methods.

**PROV-AGENT** (Souza et al., IEEE eScience, 4 citations): Extends W3C PROV data model (Entities, Activities, Agents) for agentic workflows. Integrates with MCP for real-time provenance capture across edge, cloud, and HPC environments. Maps to NATS: Messages = Entities, Processing = Activities, Publishers/Subscribers = Agents.

**Sigstore / in-toto attestations**: Cryptographically verify agent binaries and configurations before allowing NATS broker connections. Only agents built through secure, SLSA-compliant pipelines can participate in the swarm.

**Auditability axioms** (Phiri, 2025): 8 formally proved axioms -- Integrity, Coverage, Temporal Coherence, Verifiability, Accessibility, Resource Proportionality, Privacy Compatibility, Governance Alignment. Liveness properties (eventual auditability under faults) and adversarial resilience (game-theoretic resistance to log manipulation) are also proved. These provide a formal specification against which Mister Smith's audit log can be verified.

### Behavioral Safeguards (AdvEvo-MARL, COWPOX, Quarantine Actors)

**Priority**: Medium -- addresses adaptive attacks that evolve past static defenses

**AdvEvo-MARL** (Pan et al., 2025): Co-evolutionary training where attacker agents and defender agents evolve together. Attackers generate novel jailbreaks; defenders learn to resist them while maintaining task accuracy. Keeps ASR **below 20%** and suppresses contagion rates. Key insight: static safety prompts degrade over time as attackers adapt, but co-evolutionary training internalizes safety.

**COWPOX** (ICML 2025): Specialized edge-layer agents that:
1. Continuously monitor inter-agent message passing
2. Detect adversarial signatures via targeted output analysis
3. Dynamically generate "curing samples" to neutralize contagion
4. Deploy before infection reaches critical subsystems

**Quarantine actors in OTP supervision trees**: When anomaly detection triggers, the supervisor:
1. Revokes the agent's NATS credentials (not just restart -- revoke)
2. Applies exponential backoff to prevent restart loops during active poisoning
3. Routes the agent's pending work to a healthy replacement
4. Preserves the quarantined agent's state for forensic analysis

**Consensus-based Threat Validation** (Decentralized Multi-Agent Swarm architecture): Byzantine fault-tolerant voting protocol applied to semantic outputs. Sub-millisecond consensus on agent health. Peer agents continuously evaluate neighbors, achieving near-perfect detection accuracy even when a significant fraction of agents are compromised.

**Circuit breakers** (`tower-circuitbreaker`): Applied to NATS consumers, automatically halting message delivery if error/anomaly rate exceeds threshold (e.g., 50%). Prevents cascading failure when an agent is actively being exploited.

### Game-Theoretic Incentives (Proof-of-Thought, Reputation Systems)

**Priority**: Low-Medium -- relevant at scale with federated or semi-autonomous agents

**GT-HarmBench** (OpenReview): Benchmarks reveal that contemporary LLM agents choose cooperative actions in only a fraction of game-theoretic scenarios, frequently defaulting to defection. This means agent cooperation cannot be assumed -- it must be incentivized.

**BlockAgents / Proof-of-Thought**: Multi-metric assessment of reasoning trajectories (factual consistency, redundancy reduction, causal relevance). Evaluations recorded on immutable ledger. Creates a reputation-based trust environment that penalizes Byzantine behaviors and suppresses malicious prompt injection propagation.

**Incentive-centric mechanisms**: Model agent utility as a function of task rewards, capability mismatch, and workload capacity. Sequential public-goods games with adaptive reputation weighting mathematically guarantee that truthful reporting and team-oriented behavior become the Subgame Perfect Nash Equilibrium.

**Mister Smith applicability**: Most relevant for future federated deployments where agents from different organizations interact. For internal single-deployment scenarios, infrastructure-level enforcement is simpler and more reliable.

### Formal Verification (Temporal Logic, Session Types, Contracts)

**Priority**: Medium -- high payoff for correctness guarantees, investment cost is significant

**31 temporal logic properties** (Allegrini et al., 2025): 17 host agent properties + 14 task lifecycle properties in temporal logic (liveness, safety, completeness, fairness). Enables formal verification of Mister Smith's supervision tree and agent lifecycle state machines.

**Multiparty Session Types (MPST) in Rust**: `session-types` and `rumpsteak` libraries allow complex multi-agent protocols to be statically verified by the Rust compiler. Guarantees deadlock-free asynchronous message reordering. Applied successfully to Mozilla Servo.

**Assume-guarantee contracts** (Dewes & Dimitrova, 2025): Quantitative contracts for compositional design. Each supervised agent has formal pre/post-conditions that the supervisor verifies.

**VeriGuard** (Google, Miculicich et al.): Dual-stage -- offline formal verification of behavioral policies + online runtime monitoring. Maps to Mister Smith's design: offline RBAC role compilation + online SecurityLayer enforcement.

---

## Security Theater vs Real Defense

### What Actually Works

| Defense | Evidence | Why It Works |
|---------|----------|-------------|
| **AgentSandbox (persistent/ephemeral separation)** | 58.8% -> 4.34% ASR, peer-reviewed | Physically isolates sensitive data from execution context. Attack surface shrinks categorically. |
| **Deterministic control-flow graphs** | Prevents CFH entirely when enforced at routing layer | Removes the confused deputy vector by making the communication topology immutable per task. |
| **NATS Auth Callouts + least-privilege RBAC** | Prevents lateral movement, cross-account destruction | Transport-layer enforcement -- agents cannot circumvent what they cannot reach. |
| **Sub-millisecond schema validation** | Rust jsonschema 645x faster than alternatives | Drops malformed payloads before LLM processing. Zero false positives on structural validation. |
| **Infrastructure-level RBAC (not LLM-enforced)** | F1=0.27 for LLM RBAC vs deterministic enforcement | Rust PolicyEngine is not susceptible to prompt injection. |
| **SHA-256 hash-chained audit logs** | Tamper-evident by construction | Mathematical guarantee of integrity -- breaking the chain is detectable. |
| **Exponential backoff + credential revocation** | Prevents restart storms during active attacks | Stops the supervision tree from amplifying poisoning attacks through blind restarts. |

### What Looks Good but Is Brittle (Security Theater)

| "Defense" | Problem | Research Evidence |
|-----------|---------|-------------------|
| **LLM-based alignment checkers** | Easily evaded by sophisticated CFH attacks | ControlValve research shows LLM-based alignment checks are brittle against adversarial inputs designed to fool them. The attacker and the defender use the same substrate. |
| **Prompt-only guardrails ("system prompt says don't be evil")** | Degrade over time against adaptive attacks | AdvEvo-MARL demonstrates that static safety prompts lose effectiveness as attackers evolve. The ART benchmark shows all agents violate policies within 10-100 queries. |
| **Heavyweight ZKPs for every message** | Impractical latency for high-throughput swarms | Macaroons provide sufficient capability attenuation with vastly superior performance. ZKPs are appropriate for cross-organization trust, not internal message routing. |
| **Model-level access control (sudoLLM-style)** | Complementary but unreliable as primary defense | LLMs struggle with compositional permission reasoning (OrgAccess F1=0.27). Should augment, never replace, infrastructure-level enforcement. |
| **Simple restart-on-failure supervision** | Amplifies poisoning attacks | Immediate restarts during active poisoning create infinite failure loops. A quarantined agent that restarts with the same poisoned context will fail identically. |
| **Policy-as-Prompt (runtime LLM policy enforcement)** | The enforcer is as vulnerable as the enforcee | Prompt-based classification is weaker than deterministic enforcement. Useful for audit trail generation, not primary security. |

### The Critical Distinction

The dividing line between real defense and security theater is whether the defense mechanism can be influenced by the attack it is defending against. If the defense runs on an LLM, and the attack targets LLMs, the defense is fundamentally compromised. Real defenses operate on a different substrate (Rust code, NATS configuration, cryptographic proofs) that is not susceptible to the attack class being defended against.

---

## Open Questions & Gaps

### Unsolved Problems

1. **Cross-agent behavioral correlation for distributed backdoors**: No established defense exists for attacks where dormant primitives activate only through specific multi-agent collaboration sequences (Zhu et al., >95% ASR). The supervision tree sees healthy individual actors. Detection requires understanding emergent behavior across the full collaboration graph.

2. **Semantic validation at transport speed**: Schema validation catches structural anomalies but not semantic attacks. A well-formed JSON message carrying malicious instructions in its text fields passes all schema checks. No sub-millisecond solution exists for semantic content validation.

3. **Trust calibration under the Trust-Vulnerability Paradox**: OER and AD are proposed metrics but lack production validation. How much trust is "enough" for task completion without creating exploitable exposure? No empirical guidelines exist for specific task types.

4. **Infectious jailbreak containment in systems with shared memory**: The Agent Smith attack propagates through any shared context. If agents must share memory for coordination (which most useful multi-agent tasks require), how do you prevent contamination without destroying collaborative capability?

5. **Supply chain backdoors in model-agnostic systems**: When the framework is designed to work with any LLM, and 2% training data poisoning creates >80% ASR backdoors that all known defenses fail to detect, the framework must assume the model is compromised. No complete mitigation strategy exists beyond defense-in-depth.

6. **Formal verification of dynamic agent topologies**: MPST and temporal logic verification work for static, predefined protocols. Mister Smith's supervision trees and dynamic team composition create topologies that change at runtime. Formal verification of dynamic protocols remains an open research problem.

### Gaps in Current Research

- **NATS-specific security hardening literature is sparse**: Most multi-agent security research assumes generic HTTP/RPC communication. NATS subject-based authorization, JetStream consumer isolation, and Auth Callout patterns are under-researched in the academic literature.
- **Rust-specific IFC tooling is immature**: Cocoon provides compile-time IFC but is research-stage. No production-grade Rust IFC library exists. Fides is language-agnostic but not Rust-optimized.
- **Cross-framework interoperability security** (MCP, A2A, ACP, ANP): Each protocol has different trust assumptions. Security research focuses on individual protocols; cross-protocol attack surfaces are unexplored.
- **Hardware-enforced capabilities for agents**: CHERI/cWAMR demonstrate hardware-level sandboxing but server-grade CHERI hardware is not yet available. The gap between software isolation and hardware isolation remains.

---

## Implementation Priority for Mister Smith

### Phase 1: Quick Wins (Immediate -- address existential and critical threats)

| Action | Addresses | Effort | Impact |
|--------|-----------|--------|--------|
| Upgrade NATS to v2.11.1+ | CVE-2025-30215 | Low | Critical |
| Audit and remove all `>` and `$JS.>` wildcard permissions | Cross-account destruction | Low | Critical |
| Implement deterministic subject partitioning (`intent.{agent_id}.{task_id}`) | CFH, lateral movement | Medium | High |
| Add schema validation middleware on NATS consumers (`jsonschema` crate) | Malformed payload injection | Medium | High |
| Enforce W3C TraceContext + `prev_hash` headers on all inter-agent messages | Provenance, forensic accountability | Low (partially done in Phase 8) | Medium |

### Phase 2: Structural Defenses (Next -- address architectural vulnerabilities)

| Action | Addresses | Effort | Impact |
|--------|-----------|--------|--------|
| Implement AgentSandbox persistent/ephemeral separation | CFH, infectious jailbreaks, memory poisoning | High | Very High (58.8% -> 4.34% ASR) |
| Deploy NATS Auth Callout service (Rust) for dynamic capability scoping | Static JWT limitations, trust management | High | High |
| Add IFC taint labels to `MessageEnvelope` (confidentiality + integrity) | Data exfiltration, Lethal Trifecta | Medium | High |
| Implement quarantine actors in supervision tree (revoke credentials, not just restart) | Poisoning amplification via restarts | Medium | High |
| Add Tool Dependency Graph validation to ToolBus | Distributed backdoors, tool-chain attacks | Medium | Medium-High |

### Phase 3: Advanced Defenses (Long-pole -- address adaptive and sophisticated threats)

| Action | Addresses | Effort | Impact |
|--------|-----------|--------|--------|
| Implement Macaroons for capability-based authorization | RBAC limitations, delegation chains | High | High |
| Build cross-agent behavioral correlation monitoring | Distributed backdoors (>95% ASR) | Very High | High |
| Deploy AdvEvo-MARL automated red-teaming harness in CI/CD | Adaptive attack evolution | High | Medium-High |
| Integrate W3C PROV model with NATS telemetry for influence graph construction | Anomaly detection, incident response | Medium | Medium |
| Implement MIDAS anomaly detection on agent interaction graphs | Influence spikes, behavioral drift | Medium | Medium |
| Add Sigstore attestations for agent binary verification | Supply chain attacks | Medium | Medium |

### What NOT to Build

- **LLM-based security classifiers** as primary defense: Use as supplementary signal, never as enforcement mechanism
- **Heavyweight ZKPs for internal routing**: Macaroons provide sufficient capability attenuation at fraction of the cost
- **Custom prompt injection detectors**: The ART benchmark shows these are consistently evaded. Invest in infrastructure-level isolation instead.
- **Full formal verification of all protocols**: Start with critical paths only (orchestrator-to-agent, agent-to-tool). Full MPST coverage is a long-term investment.

---

## Sources

### Primary Research Files Consolidated

| File | Round | Focus |
|------|-------|-------|
| `research/targeted-capability-security-sandboxing-R4.md` | R4 | 55+ papers: capability security, IFC, MCP threats, WASM sandboxing, formal methods, memory attacks, supply chain, red teaming |
| `research/targeted-inter-agent-security-R6.md` | R6 | Deep dive: CFH (58-100% ASR), AgentSandbox (4.34% ASR), Auth Callouts, Macaroons, AdvEvo-MARL, hash chains |
| `research/discovery-sweep-R4.md` | R4 | 96 papers: inter-agent hijacking 58-100% ASR, AgentSandbox, provenance, DAG security |
| `research/discovery-sweep-R5.md` | R5 | 974 papers screened: federated security attestation gaps, verifiable capabilities |
| `research/discovery-sweep-R7c.md` | R7 | Agent hijacking 97% ASR, EchoLeak CVE-2025-32711, MPST for protocol safety |
| `research/discovery-sweep-R7d.md` | R7 | Infectious jailbreaks (Agent Smith), COWPOX defense, game-theoretic incentives, consensus-based immunity |

### Key Papers by Citation Count

| Paper | Citations | Key Finding |
|-------|-----------|-------------|
| Hou et al., MCP Landscape & Threats | 126 | 16 threat scenarios across MCP lifecycle |
| Tran et al., MAS Survey | 190 | Multi-agent collaboration mechanisms |
| Hosseini & Seilani, Agentic AI Review | 47 | Systematic review of agentic AI |
| South et al., Authenticated Delegation | 20 | OAuth 2.0 extension for agent delegation chains |
| Dong et al., MINJA Memory Injection | 18 | Memory poisoning via normal queries |
| Wang et al., MEXTRA Privacy | 24 | Memory extraction under black-box conditions |
| Shi et al., Progent | 17 | DSL-based programmable privilege control |
| Narajala & Narayan, ATFAA/SHIELD | 14 | 9 threats across 5 domains |
| Triedman et al., CFH | 15 | 58-100% ASR via confused deputy |
| Costa et al., Fides IFC | 11 | Dual-label information flow control |
| Zhou et al., AutoRedTeamer | 12 | 20% higher ASR with 46% less compute |
| Wang et al., MPMA | 9 | Genetic algorithm attacks on MCP preference |
| Zhang et al., AgentSandbox | 7 | ASR reduction 58.8% -> 4.34% |
| Ferrag et al., Threat Survey | 6 | 30+ attack techniques, 4 domains |
| Zou et al., ART Benchmark | 4 | 1.8M attacks, 60K+ violations, no capability-robustness correlation |

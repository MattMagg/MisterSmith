---
version: R1
created: 2026-03-07
updated: 2026-03-07
type: prompt
tier: 2
---

# Deep Research Prompt: Inter-Agent Security — Attack Vectors, Content Validation, and Trust

## Context

Mister Smith is a Rust-based multi-agent orchestration framework built on NATS/JetStream messaging and OTP-style supervision trees. Phase 5 implements infrastructure-level security: JWT authentication, RBAC authorization, TLS/mTLS, and tamper-evident audit logging.

Our research has identified a critical gap: infrastructure security protects the pipes, not the content flowing through them. Multi-agent systems have unique attack vectors where adversarial content in inter-agent messages can hijack other agents, even when individual agents resist prompt injection. The attack success rate is 58-100% (Triedman et al. 2025).

This is the most urgent security research in the corpus.

## Frontier-First Mandate

Mister Smith's NATS messaging layer is both the attack surface and the natural enforcement point. Subject-based namespace isolation, content validation at the transport layer, and capability-scoped message routing could provide defense-in-depth that HTTP-based frameworks cannot match. Build the security model that every other framework will need to copy.

## Research Objective

Map the full threat landscape for inter-agent communication in LLM multi-agent systems and design the defense architecture. This is not about strengthening infrastructure security (JWT, TLS — already done). This is about securing the content, intent, and influence flows between agents.

## What We Already Know (Do Not Rediscover)

- **Inter-agent hijacking** (Triedman et al. 2025, 15 citations): Adversarial content in inter-agent messages causes arbitrary code execution in 58-100% of trials, even when individual agents resist injection
- **End-to-end threat model** (Ferrag et al. 2025, 6 citations): 30+ attack techniques across Input Manipulation, Model Compromise, System/Privacy Attacks, Protocol Vulnerabilities (MCP, ACP, ANP, A2A)
- **AgentSandbox** (Zhang et al. 2025, 7 citations): Defense-in-depth, least privilege, complete mediation for agent lifecycle
- **AdvEvo-MARL** (Pan et al. 2025): Adversarial co-evolution internalizes safety into agents, <20% attack success
- **Distributed backdoor attacks**: Dormant primitives activate only in multi-agent collaboration sequences; static analysis of individual tools is insufficient
- **SharedPool vulnerability**: Pub/sub topologies most vulnerable to adversarial attack; decentralized topologies most resilient (MedSentry)
- **Capability-based security**: Macaroons, ZCAP-LD, Progent DSL (0% attack success, 17 citations), WASM/Wasmtime sandboxing
- **FoA security roadmap**: Zero-knowledge proofs and trusted execution environments for federated agent networks
- **GPT-4.1 RBAC compliance**: F1=0.27 — models cannot be trusted to enforce access control

We need: the full attack taxonomy for NATS-based multi-agent messaging, content validation techniques, trust models, and the defense architecture that makes Mister Smith secure by default.

## Research Dimensions

### 1. Inter-Agent Attack Taxonomy
- Beyond prompt injection — what are all the ways one agent can maliciously influence another through message content?
- What are the attack vectors specific to NATS pub/sub? (subject spoofing, message replay, subscription sniffing, poisoned queue groups)
- What about indirect attacks? (Agent A influences Agent B's outputs, which Agent C trusts, creating a transitive attack chain)
- What about slow-burn attacks? (Subtly biasing agent behavior over many messages rather than a single injection)
- What does the network security literature (IDS/IPS, application firewalls, DPI) say about content-level attacks on messaging systems?
- What about attacks on the coordination layer? (Poisoning CRDT state, corrupting shared blackboards, manipulating capability registries)

### 2. Content Validation and Filtering
- What techniques exist for validating that an agent's output conforms to expected structure and intent before it reaches other agents?
- Can schema validation at the message level catch malicious content? (e.g., tool call arguments must match schema, reasoning steps must follow expected format)
- What about semantic validation? (The message says "delete the database" but the agent's role is "write unit tests" — should this be flagged?)
- What role do embedding-based similarity checks play? (Is this message semantically similar to the agent's expected output distribution?)
- What about information flow control — tracking what data flows between agents and enforcing policies? (Fides and SAMOS frameworks)
- How does NATS subject-based isolation help? (Agents can only publish to subjects matching their role, preventing cross-domain influence)

### 3. Trust Models for Multi-Agent Systems
- What trust models exist for multi-agent systems? (Reputation-based, capability-based, zero-trust, hierarchical trust)
- How do you establish trust between agents that have never interacted before?
- Can trust be quantified and used for routing decisions? (Route sensitive tasks only to highly-trusted agents)
- What does the blockchain/DID literature say about verifiable agent identity and attestation?
- FoA proposes zero-knowledge proofs for capability attestation — is this practical? What is the overhead?
- What about trust degradation? (An agent's trust score decreases after producing low-quality or suspicious output)

### 4. Defense-in-Depth Architecture
- What does a multi-layered defense look like for agent-to-agent communication?
  - Layer 1: Infrastructure (TLS, JWT, RBAC — already implemented)
  - Layer 2: Transport (NATS subject isolation, message schema validation)
  - Layer 3: Content (semantic validation, embedding-based anomaly detection)
  - Layer 4: Behavioral (agent output monitoring, influence tracking, trust scoring)
  - Layer 5: Supervision (automatic isolation of compromised agents, circuit breaking)
- How do these layers compose without adding unacceptable latency?
- What is the false positive rate? Over-aggressive security will paralyze the agent system.
- How do you balance security with agent autonomy? (Too much restriction prevents useful collaboration)

### 5. Provenance and Audit for Security
- How do you trace the causal chain when an attack is detected? (Which message from which agent caused the compromise?)
- Can provenance tracking (W3C PROV-AGENT) serve double duty as both compliance audit and security forensics?
- What about tamper-evident message chains? (Each agent signs its output, creating a verifiable chain of custody)
- How does this compose with Mister Smith's existing SHA-256 hash chain audit logging?
- What does the supply chain security literature say about verifiable provenance?

### 6. Adversarial Robustness Training
- AdvEvo-MARL internalizes safety via adversarial co-evolution. Can similar techniques be applied at the framework level rather than per-agent?
- Can agent system prompts be hardened against known attack patterns? What does the prompt injection defense literature say about system-level defenses?
- What about red-teaming multi-agent systems? Are there automated red-team frameworks?
- Can you build "canary agents" that detect attacks by monitoring the messages flowing through the system?

### 7. NATS-Specific Security Patterns
- NATS accounts provide namespace isolation and explicit import/export rules. How granular can this be for agent security?
- NATS subject-based permissions: can you enforce "Agent X may only publish to subjects matching `team.alpha.executor.*`"?
- NATS message headers: can security metadata (trust score, provenance hash, capability token) ride in headers without modifying message payload?
- JetStream consumer permissions: can you restrict which agents can read from which streams?
- What about NATS authorization callout? Can a custom authorizer evaluate agent permissions dynamically?

## Output Structure

For each dimension:
1. **Threat analysis** — specific attack vectors with severity assessment
2. **Defense techniques** — specific mechanisms with effectiveness data
3. **Applicability to NATS + OTP** — integration with Mister Smith's existing infrastructure
4. **Performance overhead** — latency and computational cost of security measures
5. **Open problems** — what can't be defended against yet

Conclude with:
- Full defense architecture: what Mister Smith should build, in what order, to make inter-agent communication secure by default
- Threat model: top 10 attack vectors ranked by likelihood and impact, with defense status (defended / partially defended / undefended)
- Design principles: the security philosophy that should guide all future agent communication design
- Honest assessment: what is genuinely defensible vs. what is security theater for multi-agent LLM systems?

## Research Methodology

1. Start with Triedman et al. and Ferrag et al. — trace citation graphs for the full threat landscape
2. Search for "multi-agent security", "inter-agent attack", "LLM agent compromise", "prompt injection multi-agent"
3. Deep dive into network security literature: intrusion detection, application firewalls, content inspection for messaging systems
4. Study information flow control (IFC) literature: Fides, SAMOS, decentralized IFC, taint tracking
5. Look at blockchain/DID/zero-knowledge proof literature for agent identity and attestation
6. Study supply chain security (SLSA, Sigstore, in-toto) for provenance patterns
7. Search for NATS security documentation and best practices for multi-tenant messaging
8. Look at adversarial ML literature for robustness techniques that transfer to agent systems
9. Prioritize 2025-2026 papers; include foundational security work where directly applicable

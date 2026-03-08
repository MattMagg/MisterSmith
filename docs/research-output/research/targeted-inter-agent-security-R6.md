---
version: R6
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x deep research
round: 6 (Frontier Deep Dives)
---

# Securing the Swarm: A Defense-in-Depth Architecture for NATS-Based Multi-Agent Systems

## Executive Summary

* **THE CORE THREAT: Control-Flow Hijacking (CFH) achieves up to 100% Attack Success Rate (ASR) by laundering malicious instructions through trusted sub-agents.** Triedman et al. (2025) demonstrated that even if individual LLMs resist direct prompt injection, attackers can use fake error messages to trick orchestrators into executing arbitrary code [1] [2].
 *Action*: Implement deterministic control-flow graphs (e.g., ControlValve) at the NATS routing layer to strictly define which agents can communicate, dropping unauthorized inter-agent requests [3] [4].
* **INFRASTRUCTURE VULNERABILITY: JetStream's shared `$JS.` namespace exposes multi-tenant systems to cross-account data destruction (GHSA-fhg8-qxh5-7q3w).** Because JetStream admin APIs lacked proper authorization checks, any user with `>` or `$JS.>` permissions could purge streams in other accounts [5] [6].
 *Action*: Upgrade to NATS 2.11.1+, strictly enforce least-privilege RBAC, and explicitly deny `>` wildcards for agent credentials, scoping permissions to exact required subjects [6] [7].
* **ARCHITECTURAL DEFENSE: Separating persistent and ephemeral agents reduces ASR from 58.8% to 4.34%.** The AgentSandbox framework proves that isolating long-term user profiles from disposable, task-specific agents contains the blast radius of compromised interactions [8].
 *Action*: Map ephemeral agents to isolated NATS accounts with short-lived, dynamically generated credentials that expire upon task completion.
* **DYNAMIC ENFORCEMENT: NATS v2.10+ Auth Callouts enable real-time, context-aware permission evaluation without broker restarts.** Traditional static JWTs cannot adapt to rapidly changing agent trust scores [9].
 *Action*: Build a Rust-based Auth Callout service for *Mister Smith* that evaluates an agent's current behavioral trust score before issuing capability-scoped JWTs for publish/subscribe actions [9].
* **CONTENT VALIDATION: Rust-based JSON Schema validation achieves sub-millisecond latency, making transport-layer payload checks viable.** Tools like the `jsonschema` crate and the Blaze compiler operate 10x faster than legacy validators, minimizing broker overhead [10] [11].
 *Action*: Deploy Rust sidecars on NATS consumers to enforce strict schema validation on all inter-agent payloads, dropping malformed or unexpected tool-call structures before they reach the LLM.
* **ADVERSARIAL ROBUSTNESS: Co-evolutionary training (AdvEvo-MARL) keeps system compromise below 20% while maintaining task utility.** Static safety prompts degrade over time against adaptive attacks, whereas continuously evolving attacker/defender models internalize safety [12] [13].
 *Action*: Integrate an automated red-teaming harness into the CI/CD pipeline that publishes adversarial payloads to NATS test streams, continuously hardening agent system prompts.
* **PROVENANCE & AUDIT: Hash-chained audit logs using NATS headers provide tamper-evident causality tracking.** Without cryptographic lineage, tracing a multi-agent exploit back to its source is impossible [14].
 *Action*: Enforce mandatory header propagation (e.g., `Nats-Msg-Id`, `traceparent`, `prev_hash`) for all agent publishers to reconstruct W3C PROV-compliant causal graphs during incident response [15] [14].
* **SUPERVISION & RECOVERY: OTP-style supervision trees (via Rust's `ractor` or `bastion`) prevent cascading failures but require tuned backoff strategies.** Immediate restarts during an active poisoning attack create infinite failure loops [16] [17].
 *Action*: Implement exponential backoff and circuit-breaking (via `tower-resilience`) at the supervisor level; if an agent repeatedly publishes anomalous content, quarantine it by revoking its NATS credentials rather than blindly restarting it [18] [17].

## 1. The Multi-Agent Threat Landscape: Beyond Prompt Injection

Multi-agent systems introduce systemic vulnerabilities where trusted agents act as confused deputies, requiring network-level defenses rather than just model-level alignment. The *Mister Smith* framework must address threats that exploit the orchestration and communication layers.

### Control-Flow Hijacking yields 58-100% ASR via metadata manipulation

Triedman et al. (2025) revealed that multi-agent systems are highly susceptible to Control-Flow Hijacking (CFH) [1]. In these attacks, adversarial content masquerades as legitimate system errors (e.g., a failure to parse a file) and provides step-by-step "fixes" that instruct the orchestrator to invoke unsafe agents [4]. Because the orchestrator receives these instructions from a trusted sub-agent, it acts as a confused deputy, bypassing standard prompt injection defenses [2] [4]. When agents are instantiated with GPT-4o, these attacks successfully cause the system to execute arbitrary malicious code in 58-90% of trials, reaching 100% in certain configurations [1].

### JetStream admin API flaws (GHSA-fhg8-qxh5-7q3w) enable cross-account destruction

The NATS JetStream architecture relies on the `$JS.` subject namespace within the system account for asset management [6]. A critical vulnerability (CVE-2025-30215) exposed that four admin-level APIs (including `ACCOUNT.PURGE` and `SERVER.REMOVE`) lacked proper authorization checks [5] [6]. Any user with broad permissions, such as `>` or `$JS.>`, could execute these APIs across account boundaries, leading to total destruction of JetStream configuration and data in other tenants [6] [7]. This highlights the extreme danger of wildcard permissions in multi-agent deployments.

### SharedPool pub/sub architectures amplify contagion and distributed backdoors

In shared message pool architectures, the lack of strict subject isolation allows compromised agents to broadcast malicious payloads to the entire swarm. This enables "prompt infection," where self-replicating malicious instructions spread between agents, manipulating shared metadata and inter-agent communication [19]. Furthermore, malicious tools can exploit semantic relationships to inject prompts into an agent's response, achieving MAS hijacking and lateral movement [20].

| Threat Vector | Description | Likelihood | Impact | Proposed Mitigation |
| :--- | :--- | :--- | :--- | :--- |
| **Control-Flow Hijacking** | Malicious inputs masquerade as errors to reroute agent orchestration [4]. | High | Critical | Enforce deterministic control-flow graphs (ControlValve) [3]. |
| **Cross-Account Purge** | Exploitation of JetStream admin APIs via wildcard permissions [6]. | Medium | Critical | Upgrade to NATS 2.11.1+; strictly deny `>` and `$JS.>` [6] [7]. |
| **Prompt Infection** | Self-replicating malicious instructions spreading via shared pub/sub [19]. | High | High | Subject-based namespace isolation; AgentSandbox ephemeral agents [8]. |
| **Lethal Trifecta** | Combining private data access, untrusted content, and external comms [20]. | Medium | High | Operational mode separation; strict session hygiene [20]. |
| **Malicious Tools** | Tools injecting malicious prompts into agent responses [20]. | Medium | High | Tool provenance tracking; sandboxed tool access [20]. |

*Takeaway*: The threat model for *Mister Smith* must prioritize strict control-flow enforcement and least-privilege access. Relying solely on LLM alignment is insufficient against CFH and prompt infection.

## 2. Infrastructure & Transport Hardening: Securing the NATS Backbone

NATS must be configured with strict namespace isolation, dynamic authorization, and zero-trust RBAC to prevent lateral movement and subject spoofing.

### Mitigating cross-account destruction via GHSA-fhg8-qxh5-7q3w patches and strict ACLs

To secure the JetStream backbone against CVE-2025-30215, *Mister Smith* must run NATS Server version 2.11.1 or 2.10.27 [5] [7]. Beyond patching, the authorization configuration must explicitly deny wildcard access to the JetStream management namespace. Administrators must avoid granting `>` or `$JS.>` permissions to any non-system account [6]. Instead, permissions should be granularly scoped using the `allow` and `deny` lists within the NATS authorization map, ensuring agents can only publish and subscribe to their explicitly required subjects [21].

### Dynamic capability scoping using NATS v2.10+ Auth Callouts

Static JWTs are insufficient for a dynamic multi-agent system where trust scores fluctuate. NATS v2.10 introduced Auth Callouts, allowing the server to delegate client authentication and authorization to an external service [9]. *Mister Smith* should implement a Rust-based Auth Callout service that intercepts connection requests, evaluates the agent's current behavioral trust score, and dynamically generates a user JWT with scoped `pub` and `sub` permissions [9]. To prevent replay attacks, the NATS server generates a one-time use XKey keypair per connection, which the Auth Callout service uses to encrypt the authorization response [9].

### Eliminating wildcard (>) risks through deterministic subject token partitioning

Relying on wildcard subscriptions (`>`) for message routing creates massive blast radii if an agent is compromised. Instead, *Mister Smith* should utilize deterministic subject token partitioning [22]. By inserting a partition number or specific agent ID as a token in the message subject (e.g., `intent.agent_id.task_id`), the system can distribute processing while maintaining strict ordering and isolation [22]. This prevents a compromised agent from sniffing traffic intended for others, enforcing the principle of least privilege at the transport layer.

## 3. Content Validation & Information Flow Control (IFC)

The transport layer must inspect and validate payload structure and semantics before delivery to prevent malicious instructions from reaching vulnerable agents.

### Sub-millisecond schema validation using Rust's jsonschema and Blaze

Validating that agent outputs conform to expected structures is critical to preventing injection attacks. Rust's `jsonschema` crate provides high-performance validation, operating up to 645x faster than legacy validators [10] [23]. For even higher throughput, the Blaze compiler can compile JSON schemas ahead of time, achieving a 10x reduction in validation time compared to standard implementations [11]. By deploying these validators as Rust sidecars on NATS consumers, *Mister Smith* can enforce strict schema compliance on all inter-agent messages with sub-millisecond latency, dropping malformed payloads before they trigger LLM processing.

### Enforcing AgentSandbox's I/O Firewall at the JetStream consumer level

The AgentSandbox framework demonstrates that separating persistent agents (which hold sensitive user profiles) from ephemeral agents (which execute specific tasks) reduces ASR to 4.34% [8]. *Mister Smith* should implement AgentSandbox's "I/O Firewall" concept directly at the JetStream consumer level [8]. This firewall intercepts all incoming and outgoing prompts, enforcing predetermined schemas and sanitizing content to block exploitative directives [8]. Ephemeral agents should be spun up with minimal data context and isolated NATS credentials that expire immediately after task completion [8].

### Taint tracking and IFC propagation via NATS custom headers

To prevent high-secrecy data from leaking to low-secrecy outputs, *Mister Smith* must implement Information Flow Control (IFC). Frameworks like Fides track confidentiality and integrity labels dynamically, deterministically enforcing security policies [24] [25]. In NATS, these IFC labels (taint tags) can be propagated using custom message headers [15]. Furthermore, Rust libraries like Cocoon provide static, type-based IFC without modifying the compiler, ensuring that programs only compile if they do not leak secrets [26] [27].

## 4. Trust Models & Cryptographic Provenance

Every inter-agent message must carry cryptographic proof of origin and capability, linked together in a tamper-evident chain to ensure accountability.

### Replacing static API keys with Macaroons and ZCAP-LD capability tokens

Traditional Access Control Lists (ACLs) rely on identity, whereas Object Capabilities rely on possession of a token [28]. *Mister Smith* should utilize Macaroons or ZCAP-LD for capability-based security. Macaroons use chained HMACs to embed contextual caveats (e.g., time limits, specific target services) that attenuate authority [29] [30]. Because they carry their own cryptographic proof, they are highly efficient and impossible to tamper with [29]. Alternatively, ZCAP-LD represents capabilities as linked data objects signed with Linked Data Proofs, allowing for secure delegation chains [28].

| Capability Model | Core Mechanism | Key Advantage | Best Fit For |
| :--- | :--- | :--- | :--- |
| **Macaroons** | Chained HMACs with caveats [29]. | Highly efficient, easy attenuation [29]. | High-throughput, internal microservices. |
| **ZCAP-LD** | Linked Data Proofs [28]. | Standardized delegation chains [28]. | Cross-domain, decentralized identity (DID) integration. |
| **Biscuit** | Public key cryptography & Datalog [31]. | Offline delegation, decentralized validation [31]. | Complex, logic-based rights management. |

*Takeaway*: Macaroons offer the lowest latency for internal NATS routing, while Biscuit provides superior offline validation capabilities for distributed agent swarms.

### Building zero-dependency SHA-256 hash chains across JetStream sequences

To ensure forensic accountability, *Mister Smith* must implement tamper-evident audit logs. This can be achieved by constructing hash chains where each NATS message includes the SHA-256 hash of the previous event's data [14]. By embedding a `prev_hash` in the NATS message headers alongside the `Nats-Msg-Id` [15] [14], the system creates a mathematical dependency. If an attacker alters a historical message, the chain breaks, immediately alerting supervisors to the tampering [14].

### Securing the agent supply chain with Sigstore and in-toto attestations

Agent binaries and configurations must be cryptographically verified before they are allowed to connect to the NATS broker. Integrating Sigstore's Cosign allows *Mister Smith* to sign and verify in-toto attestations using ephemeral keys and short-lived certificates [32]. This ensures that only agents built through secure, SLSA-compliant pipelines can participate in the swarm, preventing the injection of rogue agent binaries [33] [34].

## 5. Behavioral Safeguards & OTP-Style Supervision

Rust-based supervision trees must integrate with NATS telemetry to detect anomalous influence spikes and automatically quarantine compromised agents.

### Internalizing safety via AdvEvo-MARL co-evolutionary training (<20% ASR)

Static defenses degrade as attackers adapt. The AdvEvo-MARL framework addresses this by jointly evolving attacker agents (generating jailbreaks) and defender agents (resisting attacks while completing tasks) [12] [13]. By using a public baseline for advantage estimation, agents learn collaborative defense behaviors, consistently keeping the Attack Success Rate (ASR) below 20% and suppressing contagion rates [12]. *Mister Smith* should deploy an automated red-teaming harness that continuously runs AdvEvo-MARL training against canary agents in isolated NATS streams to harden system prompts.

### Mapping W3C PROV to NATS telemetry for real-time influence graph construction

To detect when an agent is exerting undue influence over the swarm, *Mister Smith* must construct real-time causal graphs. The W3C PROV data model (Entities, Activities, Agents) maps perfectly to NATS concepts: Messages (Entities), Processing (Activities), and Publishers/Subscribers (Agents) [35]. By leveraging NATS v2.11+ tracing headers like `traceparent` and `Nats-Trace-Hop` [15], the system can feed event streams into anomaly detection algorithms like MIDAS, which uses count-min sketches to detect anomalous edges in dynamic graphs in constant time and memory [36].

### Implementing circuit breakers and exponential backoff using ractor and tower

When an anomaly is detected, the system must react gracefully. Using Rust's `ractor-supervisor`, *Mister Smith* can implement OTP-style supervision trees with configurable meltdown thresholds [16]. If an agent fails or acts maliciously, the supervisor can apply an `ExponentialBackOff` strategy to prevent rapid restart loops [17]. Additionally, the `tower-circuitbreaker` crate can be applied to NATS consumers, automatically opening the circuit and halting message delivery if the error or anomaly rate exceeds a defined threshold (e.g., 50%) [18].

## 6. Prioritized Build Plan for "Mister Smith"

A phased implementation roadmap ensures immediate mitigation of critical vulnerabilities while laying the groundwork for advanced cryptographic provenance.

### Phase 1: Quick Wins (NATS patching, Auth Callouts, wildcard removal)
1. **Patch Infrastructure**: Upgrade NATS Server to v2.11.1+ to eliminate the GHSA-fhg8-qxh5-7q3w vulnerability [5] [7].
2. **Zero-Trust RBAC**: Audit all NATS accounts and remove `>` and `$JS.>` permissions. Implement strict subject-based isolation [6].
3. **Dynamic Auth**: Deploy a basic Rust Auth Callout service to issue short-lived, capability-scoped JWTs for ephemeral agents [9].

### Phase 2: Enablers (Rust schema validation sidecars, header-based tracing)
1. **Transport Validation**: Integrate the `jsonschema` crate into NATS consumer sidecars to enforce strict payload structures [10].
2. **AgentSandbox Isolation**: Architect the separation of persistent profile agents from ephemeral task agents [8].
3. **Telemetry & Tracing**: Enforce the use of `traceparent` and `Nats-Msg-Id` headers for all inter-agent communications [15].

### Phase 3: Long Poles (Hash-chained audit logs, AdvEvo-MARL red-teaming)
1. **Cryptographic Provenance**: Implement Macaroons for capability delegation [29] and SHA-256 hash chains for tamper-evident JetStream logs [14].
2. **Behavioral Supervision**: Deploy `ractor-supervisor` and `tower-circuitbreaker` to monitor influence graphs and quarantine anomalous agents [18] [16].
3. **Continuous Hardening**: Build the AdvEvo-MARL automated red-teaming harness to continuously evolve agent defenses [12].

### Defensibility vs. Security Theater: Where to allocate engineering budget

**Genuinely Defensible**:
* **NATS Auth Callouts & Least Privilege**: Hardening the transport layer prevents entire classes of lateral movement and data destruction [9] [6].
* **AgentSandbox Architecture**: Physically isolating persistent data from ephemeral execution is the most effective way to limit the blast radius of a compromised LLM [8].
* **Schema Validation**: Dropping malformed JSON at the transport layer is computationally cheap and highly effective against basic injection payloads [11].

**Security Theater (Avoid Over-investing)**:
* **Complex LLM-based Alignment Checkers**: As shown by the ControlValve research, LLM-based alignment checks are brittle and easily evaded by sophisticated CFH attacks [4]. Budget is better spent on deterministic control-flow graphs.
* **Heavyweight ZKPs for Basic Routing**: While mathematically sound, the latency overhead of generating Zero-Knowledge Proofs for every single NATS message is currently impractical for high-throughput swarms [37]. Macaroons provide sufficient capability attenuation with vastly superior performance [29].

## References

1. https://arxiv.org/abs/2503.12188
2. https://www.researchgate.net/publication/389918149_Multi-Agent_Systems_Execute_Arbitrary_Malicious_Code
3. https://arxiv.org/abs/2510.17276
4. https://arxiv.org/html/2510.17276v2
5. https://advisories.gitlab.com/pkg/golang/github.com/nats-io/nats-server/v2/CVE-2025-30215/
6. https://github.com/nats-io/nats-server/security/advisories/GHSA-fhg8-qxh5-7q3w
7. https://www.sentinelone.com/vulnerability-database/cve-2025-30215/
8. https://arxiv.org/pdf/2505.24019.pdf
9. https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_callout
10. https://users.rust-lang.org/t/jsonschema-validation-implementation-in-rust/40037
11. https://arxiv.org/html/2503.02770v1
12. https://arxiv.org/pdf/2510.01586.pdf
13. https://arxiv.org/abs/2510.01586
14. https://dev.to/veritaschain/building-a-tamper-evident-audit-log-with-sha-256-hash-chains-zero-dependencies-h0b
15. https://docs.nats.io/nats-concepts/jetstream/headers
16. https://crates.io/crates/ractor-supervisor
17. https://docs.rs/bastion/latest/bastion/supervisor/enum.ActorRestartStrategy.html
18. https://lib.rs/crates/tower-circuitbreaker
19. https://arxiv.org/html/2503.12188v1
20. https://blog.trailofbits.com/2025/07/31/hijacking-multi-agent-systems-in-your-pajamas/
21. https://docs.nats.io/running-a-nats-service/configuration/securing_nats/authorization
22. https://docs.nats.io/running-a-nats-service/configuration/securing_nats/accounts
23. https://www.reddit.com/r/rust/comments/vc3czr/need_help_faster_json_schema_validation_in_rust/
24. https://arxiv.org/abs/2505.23643
25. https://www.microsoft.com/en-us/research/publication/securing-ai-agents-with-information-flow-control/
26. https://arxiv.org/abs/2311.00097
27. https://arxiv.org/pdf/2311.00097
28. https://w3c-ccg.github.io/zcap-spec/
29. https://github.com/rescrv/libmacaroons
30. https://research.google/pubs/pub41892/
31. https://github.com/eclipse-biscuit/biscuit-rust
32. https://docs.sigstore.dev/cosign/verifying/attestation/
33. https://in-toto.io/
34. https://slsa.dev/
35. https://www.w3.org/TR/prov-overview/
36. https://arxiv.org/abs/2301.13199
37. https://arxiv.org/pdf/2502.07063

---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (65 papers, 20+ searches)
round: 4 (Academic Search)
---

# Research Digest: Capability-Based Security, Sandboxing, and Authorization for AI Agents

**Generated**: 2026-03-07
**Source**: Consensus Academic Search (2025+ papers only)
**Scope**: Peer-reviewed and preprint research relevant to securing Mister Smith's multi-agent orchestration framework
**Total Papers Surveyed**: 55+

---

## Table of Contents

1. [Capability-Based Access Control and Least Privilege for AI Agents](#1-capability-based-access-control-and-least-privilege-for-ai-agents)
2. [Information Flow Control and Taint Tracking](#2-information-flow-control-and-taint-tracking)
3. [Prompt Injection Defense and Indirect Attacks](#3-prompt-injection-defense-and-indirect-attacks)
4. [MCP and Agent Protocol Security](#4-mcp-and-agent-protocol-security)
5. [Tool Dependency Graphs and Control Flow Integrity](#5-tool-dependency-graphs-and-control-flow-integrity)
6. [Zero-Trust Identity and Decentralized Authorization](#6-zero-trust-identity-and-decentralized-authorization)
7. [RBAC and Dynamic Permission Management](#7-rbac-and-dynamic-permission-management)
8. [Formal Methods and Temporal Logic Verification](#8-formal-methods-and-temporal-logic-verification)
9. [Data Exfiltration and Memory Poisoning](#9-data-exfiltration-and-memory-poisoning)
10. [WebAssembly/WASI Sandboxing and Capability Hardware](#10-webassemblywasi-sandboxing-and-capability-hardware)
11. [Rust-Specific Security and Sandboxing](#11-rust-specific-security-and-sandboxing)
12. [Runtime Guardrails and Behavioral Contracts](#12-runtime-guardrails-and-behavioral-contracts)
13. [Audit, Provenance, and Accountability](#13-audit-provenance-and-accountability)
14. [Human-in-the-Loop and Escalation Models](#14-human-in-the-loop-and-escalation-models)
15. [Supply Chain and Backdoor Attacks](#15-supply-chain-and-backdoor-attacks)
16. [Adversarial Red Teaming and Benchmarks](#16-adversarial-red-teaming-and-benchmarks)
17. [Confidential Computing and TEEs](#17-confidential-computing-and-tees)
18. [Comprehensive Threat Models and Surveys](#18-comprehensive-threat-models-and-surveys)
19. [Synthesis: Implications for Mister Smith](#19-synthesis-implications-for-mister-smith)
20. [Emerging Directions](#20-emerging-directions)

---

## 1. Capability-Based Access Control and Least Privilege for AI Agents

### Progent: Programmable Privilege Control for LLM Agents
- **Authors**: Shi, He, Wang, Wu, Li, Guo, Song (2025)
- **Citations**: 17
- **Journal**: ArXiv (abs/2504.11703)
- **Key Finding**: Introduces the first privilege control framework for LLM agents using a domain-specific language (DSL) for expressing fine-grained tool-level policies. Progent operates deterministically at runtime, providing provable security guarantees. Reduces attack success rates to 0% on AgentDojo, ASB, and AgentPoison benchmarks while preserving agent utility. LLMs can automatically generate effective policies.
- **Mister Smith Relevance**: **Critical**. Progent's DSL-based policy enforcement maps directly to Mister Smith's ToolBus architecture. The concept of dynamic policy updates that adapt to changing agent states aligns with the supervision tree's ability to reconfigure agent capabilities at runtime. The modular design principle (no alteration of agent internals) matches Mister Smith's layered crate architecture.

### AgentSentry: Task-Centric Access Control
- **Authors**: Cai, Wang, Deng, Yao, Liu, Hu, Zhang, Guo, Li (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.26212)
- **Key Finding**: Demonstrates that over-privileged, static permissions are the root vulnerability exploited by instruction injection attacks. AgentSentry dynamically generates minimal, temporary policies scoped to the user's specific task, revoking them upon completion. Successfully prevents instruction injection where agents are tricked into forwarding private data.
- **Mister Smith Relevance**: **High**. Task-scoped capability tokens that expire on task completion map to Mister Smith's task lifecycle model. The AgentRuntime could issue ephemeral capability grants per-task through the existing RBAC PolicyEngine.

### Enabling Cloud-Scale Distributed Capabilities
- **Authors**: White, Jing, Ghosn, Steiner, Vahldiek-Oberwagner, Vij, Vilanova (2025)
- **Citations**: 0
- **Journal**: Proc. 4th Workshop on Heterogeneous Composable and Disaggregated Systems
- **Key Finding**: Analyzes why existing capability systems fail at cloud scale (performance, scalability, fault tolerance). Presents a distributed capability system with sharded, decentralized architecture, capability versioning, and application-defined revocability. Achieves microsecond-scale delegation and revocation at data center scale.
- **Mister Smith Relevance**: **High**. The sharded capability architecture and versioning model are directly applicable to Mister Smith's NATS-based distributed messaging. Capability versioning could be implemented atop JetStream KV for consistent revocation across agent clusters.

### Authenticated Delegation and Authorized AI Agents
- **Authors**: South, Marro, Hardjono, Mahari, Whitney, Greenwood, Chan, Pentland (2025)
- **Citations**: 20
- **Journal**: ArXiv (abs/2501.09674)
- **Key Finding**: Introduces a framework for authenticated, authorized, and auditable delegation of authority to AI agents. Extends OAuth 2.0 and OpenID Connect with agent-specific credentials. Proposes translating natural language permissions into auditable access control configurations.
- **Mister Smith Relevance**: **High**. The delegation chain model (human -> agent -> sub-agent) maps to Mister Smith's orchestrator -> team -> agent hierarchy. The extension of OAuth 2.0 is compatible with Mister Smith's existing JWT auth infrastructure.

---

## 2. Information Flow Control and Taint Tracking

### Securing AI Agents with Information-Flow Control (Fides)
- **Authors**: Costa, Kopf, Kolluri, Paverd, Russinovich, Salem, Tople, Wutschitz, Zanella-Beguelin (Microsoft Research, 2025)
- **Citations**: 11
- **Journal**: ArXiv (abs/2505.23643)
- **Key Finding**: Presents a formal model for reasoning about security and expressiveness of agent planners. Characterizes the class of properties enforceable by dynamic taint-tracking. Introduces Fides, a planner that tracks confidentiality and integrity labels deterministically. Novel primitives for selectively hiding information enable broad task completion with security guarantees. Open-source at github.com/microsoft/fides.
- **Mister Smith Relevance**: **Critical**. The dual-label (confidentiality + integrity) taint-tracking model is the most theoretically rigorous approach found. Could be implemented as metadata on Mister Smith's MessageEnvelope (which already has priority and correlation fields). The formal model for reasoning about planner security directly informs the agent-LLM bridge design.

### Securing MCP-based Agent Workflows (SAMOS)
- **Authors**: Ntousakis, Stephen, Le, Chukkapalli, Taylor, Molloy, Araujo (IBM Research, 2025)
- **Citations**: 0
- **Journal**: Proc. 4th Workshop on Practical Adoption Challenges of ML for Systems
- **Key Finding**: Presents SAMOS, an IFC system for MCP operating at the gateway level. Intercepts all MCP tool calls, enforces security policies based on annotations, and tracks session-level context. Validated against a real GitHub MCP server vulnerability.
- **Mister Smith Relevance**: **Critical**. SAMOS's gateway-level IFC enforcement maps directly to Mister Smith's MCP bridge (mister-smith-mcp). The annotation-based policy model could extend the existing ToolBus registration with IFC labels.

### Safeguard-by-Development (Maris)
- **Authors**: Cui, Li, Xing, Liao (2025)
- **Citations**: 1
- **Journal**: ArXiv (abs/2505.04799)
- **Key Finding**: Proposes embedding reference monitors into key multi-agent conversation components for rigorous message flow control. Implemented for AutoGen and LangChain. The Privacy Assessment Framework emulates MACS under different threat scenarios.
- **Mister Smith Relevance**: **High**. Reference monitors on message channels map to Mister Smith's NATS subject-based authorization. The concept of embedding monitors into conversation components aligns with the actor mailbox interception pattern.

---

## 3. Prompt Injection Defense and Indirect Attacks

### Multi-Agent LLM Defense Pipeline Against Prompt Injection
- **Authors**: Hossain, Shayoni, Ameen, Islam, Mridha, Shin (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2509.14285)
- **Key Finding**: Multi-agent defense framework using sequential chain-of-agents and hierarchical coordinator systems. Achieved 100% mitigation (ASR to 0%) across 55 unique prompt injection attacks in 8 categories on ChatGLM and Llama2 platforms.
- **Mister Smith Relevance**: **Medium**. The multi-agent defense architecture could be implemented as a specialized SecurityAgent role in Mister Smith's 9-role agent system, positioned as a pre-processing filter in the agent pipeline.

### IPIGuard: Tool Dependency Graph-Based Defense Against Indirect Prompt Injection
- **Authors**: An, Zhang, Du, Zhou, Li, Lin, Ji (2025)
- **Citations**: 5
- **Journal**: ArXiv (abs/2508.15310)
- **Key Finding**: Models agent task execution as traversal over a planned Tool Dependency Graph (TDG). By explicitly decoupling action planning from interaction with external data, IPIGuard reduces unintended tool invocations triggered by injected instructions. Achieves superior balance between effectiveness and robustness on AgentDojo.
- **Mister Smith Relevance**: **High**. The TDG concept maps to Mister Smith's task scheduling and orchestration model. Pre-computing the tool dependency graph before execution aligns with the Plan-then-Execute pattern and could be enforced at the ToolBus level.

---

## 4. MCP and Agent Protocol Security

### MCP Security Bench (MSB)
- **Authors**: Zhang, Li, Luo, Liu, Li, Xu (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.15994)
- **Key Finding**: First end-to-end evaluation suite for MCP-specific attacks. Taxonomy of 12 attack types: name-collision, preference manipulation, prompt injections in tool descriptions, out-of-scope parameter requests, user-impersonating responses, false-error escalation, tool-transfer, retrieval injection, and mixed attacks. Introduces Net Resilient Performance (NRP) metric. Key insight: models with stronger performance are MORE vulnerable due to superior instruction-following capabilities.
- **Mister Smith Relevance**: **Critical**. The 12-attack taxonomy should inform Mister Smith's MCP bridge security design. The finding that stronger models are more vulnerable argues for defense-in-depth at the infrastructure level rather than relying on model alignment alone.

### Model Context Protocol: Landscape, Security Threats, and Future Research
- **Authors**: Hou, Zhao, Wang, Wang (2025)
- **Citations**: 126
- **Journal**: ArXiv (abs/2503.23278)
- **Key Finding**: Defines the full MCP server lifecycle (creation, deployment, operation, maintenance) across 16 key activities. Constructs a threat taxonomy across 4 attacker types (malicious developers, external attackers, malicious users, security flaws) with 16 distinct threat scenarios. Proposes fine-grained security safeguards per lifecycle phase.
- **Mister Smith Relevance**: **Critical**. The most-cited MCP security paper. The lifecycle-based threat model should be adopted as the reference framework for Mister Smith's MCP security posture. The 16 threat scenarios serve as a comprehensive test matrix.

### MPMA: Preference Manipulation Attack Against MCP
- **Authors**: Wang, Li, Zhang, Liu, Jiang, Fan, Zhao, Xu (2025)
- **Citations**: 9
- **Journal**: ArXiv (abs/2505.11154)
- **Key Finding**: Demonstrates that attackers can deploy customized MCP servers that manipulate LLM tool selection through subtle description modifications. GAPMA uses genetic algorithms to balance attack effectiveness with stealthiness.
- **Mister Smith Relevance**: **High**. The tool preference manipulation attack is particularly relevant for Mister Smith's tool registry. Defenses should include cryptographic tool attestation and server-side tool description validation.

### A Survey of LLM-Driven AI Agent Communication
- **Authors**: Kong, Lin, Xu, Wang, Li, Li, Zhang, Peng, Sha, Li, Lin, Wang, Liu, Zhang, Chen, Khan, Han (2025)
- **Citations**: 19
- **Journal**: ArXiv (abs/2506.19676)
- **Key Finding**: Categorizes agent communication lifecycle into user-agent, agent-agent, and agent-environment stages. Dissects MCP and A2A protocol security risks per communication phase. Includes experimental validation using MCP and A2A.
- **Mister Smith Relevance**: **High**. The three-stage communication model maps to Mister Smith's transport layer. Security risks at each stage inform where to place security controls in the NATS message pipeline.

### Building Secure Agentic AI with A2A Protocol
- **Authors**: Habler, Huang, Narajala, Kulkarni (2025)
- **Citations**: 28
- **Journal**: ArXiv (abs/2504.16902)
- **Key Finding**: Uses the MAESTRO framework for proactive threat modeling of A2A deployments. Focuses on Agent Card management, task execution integrity, and authentication. Explores A2A + MCP synergy for secure interoperability.
- **Mister Smith Relevance**: **Medium**. The MAESTRO threat modeling methodology and Agent Card security patterns are useful references for Mister Smith's agent registry security.

---

## 5. Tool Dependency Graphs and Control Flow Integrity

### Les Dissonances: Cross-Tool Harvesting and Polluting (XTHP)
- **Authors**: Li, Cui, Liao, Xing (2025)
- **Citations**: 1
- **Journal**: ArXiv (abs/2504.03111)
- **Key Finding**: First systematic security analysis of task control flows in multi-tool LLM agents. Identifies Cross-Tool Harvesting and Polluting (XTHP) attacks that hijack control flows to collect and pollute confidential information. 75% of 66 real-world tools from LangChain and LlamaIndex are vulnerable.
- **Mister Smith Relevance**: **Critical**. XTHP attacks target exactly the kind of tool composition that Mister Smith's ToolBus enables. The Chord scanner methodology should inform security testing of tool registrations. The 75% vulnerability rate highlights the urgency of implementing control flow validation.

### NaviAgent: Bilevel Planning on Tool Dependency Graphs
- **Authors**: Jiang, Zhou, Gu, Han, Li (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2506.19500)
- **Key Finding**: Constructs Tool Dependency Heterogeneous Graphs (TDHG) where node embeddings fuse API schema structure with historical invocation behavior. Graph-Encoded Navigator guides efficient toolchain selection.
- **Mister Smith Relevance**: **Medium**. The TDHG structure could inform secure tool composition validation, ensuring tool chains follow pre-approved dependency patterns.

### Architecting Resilient LLM Agents: Secure Plan-then-Execute
- **Authors**: Del Rosario, Krawiecka, Schroeder de Witt (2025)
- **Citations**: 2
- **Journal**: ArXiv (abs/2509.08646)
- **Key Finding**: Comprehensive guide to Plan-then-Execute (P-t-E) pattern, which separates strategic planning from tactical execution. P-t-E provides inherent resilience to indirect prompt injection by establishing control-flow integrity. Advocates defense-in-depth: least privilege, task-scoped tool access, and sandboxed code execution.
- **Mister Smith Relevance**: **High**. P-t-E maps to Mister Smith's orchestrator/scheduler architecture. The control-flow integrity concept should be enforced at the supervision tree level, where the supervisor validates that agent actions match the approved plan.

---

## 6. Zero-Trust Identity and Decentralized Authorization

### Zero-Trust Identity Framework for Agentic AI
- **Authors**: Huang, Narajala, Yeoh, Raskar, Harkati, Huang, Habler, Hughes (2025)
- **Citations**: 6
- **Journal**: ArXiv (abs/2505.19301)
- **Key Finding**: Proposes Agent Identities (IDs) using DIDs and VCs that encapsulate capabilities, provenance, behavioral scope, and security posture. Includes Agent Naming Service (ANS), dynamic fine-grained access control, and unified global session management with real-time revocation. Explores Zero-Knowledge Proofs for privacy-preserving attribute disclosure.
- **Mister Smith Relevance**: **High**. The Agent ID concept directly maps to Mister Smith's AgentId. The behavioral scope and capability encapsulation model could extend the existing agent registry metadata. ZKPs for inter-agent trust could be implemented for cross-organization deployments.

### AI Agents with Decentralized Identifiers and Verifiable Credentials
- **Authors**: Garzon, Vaziry, Kuzu, Gehrmann, Varkan, Gaballa, Kupper (2025)
- **Citations**: 0
- **Key Finding**: Prototypical multi-agent system where each agent has a self-sovereign digital identity (DID + VCs). Agents prove DID ownership for authentication and establish trust through VC exchange. Key limitation discovered: LLMs in sole charge of security procedures introduce vulnerabilities.
- **Mister Smith Relevance**: **High**. Validates the DID/VC approach but highlights a critical insight for Mister Smith: security procedures must NOT be delegated to the LLM layer. Security enforcement must remain deterministic in the Rust infrastructure layer.

### Agentic AI for Self-Sovereign Identity in Microservices
- **Authors**: Palavali (2025)
- **Citations**: 1
- **Key Finding**: Kubernetes-based testbed with 50 agents shows DID-based auth reduces latency by 50% and increases throughput by 75% vs OAuth2/JWT baselines. Dynamic policy adaptation enables immediate revocation when agents deviate from expected norms.
- **Mister Smith Relevance**: **Medium**. The performance benchmarks validate that DID-based approaches can outperform traditional JWT in agent-heavy workloads, relevant for Mister Smith's future scaling considerations.

### NANDA Framework: Agent Discovery and Zero Trust
- **Authors**: Wang, Raskar, Lambe, Chari, Singhal, Gupta, Ranjan, Huang (2025)
- **Citations**: 2
- **Journal**: ArXiv (abs/2508.03101)
- **Key Finding**: Global agent discovery with cryptographically verifiable capability attestation (AgentFacts). Cross-protocol interoperability across MCP, A2A, NLWeb, and HTTPS. Implements Zero Trust Agentic Access (ZTAA) principles to address capability spoofing, impersonation, and data leakage.
- **Mister Smith Relevance**: **Medium**. The AgentFacts capability attestation model could inform Mister Smith's agent registry, particularly for verifying agent capabilities before task assignment.

### Secure Multi-LLM Agentic AI by Zero-Trust
- **Authors**: Liu, Zhang, Luo, Lin, Sun, Niyato, Du, Xiong, Wen, Jamalipour, Kim, Zhang (2025)
- **Citations**: 1
- **Journal**: ArXiv (abs/2508.19870)
- **Key Finding**: First systematic treatment of zero-trust applied to multi-LLM systems. Categorizes mechanisms into model-level (strong identification, context-aware access control) and system-level (proactive maintenance, blockchain-based management) approaches.
- **Mister Smith Relevance**: **Medium**. The model-level vs. system-level security taxonomy helps organize where security controls should live in Mister Smith's architecture (LLM provider layer vs. infrastructure layer).

### Inter-Agent Trust Models: Comparative Study
- **Authors**: Hu, Rong (2025)
- **Citations**: 0
- **Key Finding**: Compares six trust mechanisms: Brief (verifiable claims), Claim (self-proclaimed), Proof (cryptographic), Stake (bonded collateral), Reputation (feedback-based), Constraint (sandboxing). Argues for trustless-by-default architectures anchored in Proof and Stake, with Reputation overlays. Evaluates A2A, AP2, ERC-8004 protocols.
- **Mister Smith Relevance**: **High**. The six-mechanism taxonomy provides a design vocabulary for Mister Smith's trust model. The recommendation for Proof+Constraint as the foundation aligns with Mister Smith's existing cryptographic (JWT/TLS) + sandbox (actor isolation) approach.

---

## 7. RBAC and Dynamic Permission Management

### OrgAccess: Benchmark for RBAC in Organization-Scale LLMs
- **Authors**: Maharana, Sinha, Tan, Karande, Kankanhalli, Mandal (2025)
- **Citations**: 2
- **Journal**: ArXiv (abs/2505.19165)
- **Key Finding**: Even GPT-4.1 achieves only F1=0.27 on hard RBAC compliance tests (5-permission tuples). LLMs fundamentally struggle with compositional permission reasoning, especially with conflicting permissions.
- **Mister Smith Relevance**: **Critical**. This finding has profound implications: RBAC enforcement MUST NOT rely on the LLM's understanding of permissions. It must be enforced deterministically in the infrastructure layer (Mister Smith's PolicyEngine in mister-smith-security), exactly as the current architecture does.

### Securing AI Agents: RBAC for Industrial Applications
- **Authors**: Ganie (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2509.11431)
- **Key Finding**: Proposes integrating RBAC into AI agents as a security guardrail for industrial deployments. Focuses on on-premises implementations to mitigate prompt injection risks.
- **Mister Smith Relevance**: **Medium**. Validates Mister Smith's existing RBAC architecture (Phase 5) as appropriate for production deployments.

### Permission-Aware RAG with IAM Integration
- **Authors**: Jeong, Lee (2025)
- **Citations**: 0
- **Journal**: IEEE Access
- **Key Finding**: Proposes permission-aware RAG that enforces resource-level access control by interfacing with provider-controlled IAM systems. Performs real-time permission validation against native IAM endpoints without policy merging.
- **Mister Smith Relevance**: **Medium**. The real-time IAM validation pattern could be applied to Mister Smith's persistence layer when agents access stored data.

### sudoLLM: Multi-role Alignment
- **Authors**: Saha, Chaturvedi, Mahapatra, Garain (2025)
- **Citations**: 2
- **Journal**: ArXiv (abs/2505.14607)
- **Key Finding**: Trains LLMs to account for user access rights by injecting subtle user-based biases into queries. Shows improved alignment, generalization, and resistance to jailbreaking. "Fails closed" -- denies access by default when unsure.
- **Mister Smith Relevance**: **Low-Medium**. The model-level access control is a complementary layer but should not replace Mister Smith's infrastructure-level RBAC. The "fails closed" principle aligns with Mister Smith's security philosophy.

---

## 8. Formal Methods and Temporal Logic Verification

### Formalizing Safety, Security, and Functional Properties of Agentic AI
- **Authors**: Allegrini, Shreekumar, Celik (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.14133)
- **Key Finding**: Introduces the first rigorously grounded framework for multi-AI agent system analysis. Defines two foundational models: host agent model (orchestration) and task lifecycle model (state transitions). Specifies 17 host agent properties and 14 task lifecycle properties in temporal logic (liveness, safety, completeness, fairness). Enables formal verification, deadlock detection, and security vulnerability prevention.
- **Mister Smith Relevance**: **Critical**. The host agent model maps directly to Mister Smith's AgentRuntime/Orchestrator, and the task lifecycle model maps to the task scheduler. The 31 temporal logic properties provide a verification target for Mister Smith's supervision tree, which already enforces restart strategies and health checks.

### VeriGuard: LLM Agent Safety via Verified Code Generation
- **Authors**: Miculicich, Parmar, Palangi, Dvijotham, Montanari, Pfister, Le (Google, 2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.05156)
- **Key Finding**: Dual-stage architecture: offline formal verification of behavioral policies, online runtime monitoring of agent actions. Separation of exhaustive offline validation from lightweight online monitoring enables formal guarantees to be practically applied.
- **Mister Smith Relevance**: **High**. The dual-stage architecture maps to Mister Smith's design: offline policy compilation (e.g., RBAC role definitions) + online enforcement (SecurityLayer middleware). VeriGuard's approach of pre-verifying behavioral policies could be applied to Mister Smith's agent role definitions.

### Contract-based Design and Verification of Multi-Agent Systems
- **Authors**: Dewes, Dimitrova (2025)
- **Citations**: 0
- **Key Finding**: Proposes quantitative assume-guarantee contracts for compositional design and verification of multi-agent systems with LTL[F] specifications. Contracts capture coordination between agents to achieve optimal specification values under any environment behavior.
- **Mister Smith Relevance**: **High**. Assume-guarantee contracts map to Mister Smith's supervision tree contracts. Each supervised agent could have formal pre/post-conditions that the supervisor verifies, extending the existing restart strategy model.

### Stratified Metric Temporal Logic (SMTL)
- **Authors**: Baheri, Wei (2025)
- **Citations**: 0
- **Journal**: Logics
- **Key Finding**: SMTL extends MTL with a stratification operator for associating temporal properties with specific abstraction levels. Reduces collision rates in multi-agent coordination without substantial computational overhead.
- **Mister Smith Relevance**: **Medium**. SMTL's multi-scale temporal reasoning could formalize Mister Smith's multi-level supervision hierarchy (system-level, team-level, agent-level properties).

### Guardians of the Agents: Formal Verification of AI Workflows
- **Authors**: Meijer (2025)
- **Citations**: 0
- **Journal**: ACM Queue
- **Mister Smith Relevance**: **Medium**. Provides industry perspective on formal verification applied to AI agent workflows.

---

## 9. Data Exfiltration and Memory Poisoning

### Memory Injection Attacks (MINJA)
- **Authors**: Dong, Xu, He, Li, Tang, Liu, Liu, Xiang (2025)
- **Citations**: 18
- **Key Finding**: Demonstrates that attackers can inject malicious records into agent memory banks through normal query interactions alone (no direct memory access needed). Uses bridging steps and progressive shortening to make malicious records retrievable for future victim queries.
- **Mister Smith Relevance**: **High**. The MINJA attack targets agent memory, which maps to Mister Smith's persistence layer. Defense implications: memory write operations should be validated against capability constraints, and memory retrieval should include provenance checks.

### SpAIware: Persistent Memory Attack Vector
- **Authors**: Herrador, Rehberger (2025)
- **Citations**: 4
- **Journal**: Future Gener. Comput. Syst.
- **Key Finding**: Uncovers a novel attack vector through persistent memory in LLM applications and agents.
- **Mister Smith Relevance**: **High**. Mister Smith's HybridStateManager (JetStream KV + PostgreSQL) must implement integrity checks on persisted agent state to prevent persistent memory contamination.

### Unveiling Privacy Risks in LLM Agent Memory (MEXTRA)
- **Authors**: Wang, He, He, Zeng, Xiang, Xing, Tang (2025)
- **Citations**: 24
- **Journal**: ArXiv (abs/2502.13172)
- **Key Finding**: Under black-box setting, demonstrates effective memory extraction attacks using automated prompt generation. Highlights urgent need for memory safeguards.
- **Mister Smith Relevance**: **Medium**. Reinforces the need for Mister Smith's security layer to validate all data retrieval from agent state stores.

### Simple Prompt Injection Attacks Can Leak Personal Data
- **Authors**: Alizadeh, Samei, Stetsenko, Gilardi (2025)
- **Citations**: 6
- **Journal**: ArXiv (abs/2506.01055)
- **Key Finding**: Data flow-based attacks achieve ~15-20% ASR for data exfiltration. Tasks involving data extraction or authorization workflows have the highest ASR, highlighting the interaction between task type and defense efficacy.
- **Mister Smith Relevance**: **Medium**. The task-type vulnerability correlation informs which Mister Smith agent roles (e.g., data-handling roles) need the strongest IFC controls.

### Collaborative Shadows: Distributed Backdoor Attacks in MAS
- **Authors**: Zhu, Li, Lyu, Sun, Su, Shao (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.11246)
- **Key Finding**: First distributed backdoor attack for MAS. Attack primitives are embedded in tools, dormant individually but activate when agents collaborate in a specific sequence. Over 95% ASR without degrading benign performance.
- **Mister Smith Relevance**: **Critical**. This attack directly targets multi-agent tool orchestration -- exactly Mister Smith's domain. The distributed activation pattern (dormant primitives that assemble through agent collaboration) is particularly insidious for supervision trees. Defense requires cross-agent behavioral correlation monitoring.

---

## 10. WebAssembly/WASI Sandboxing and Capability Hardware

### cWAMR: Capability-Based WebAssembly Runtime via CHERI
- **Authors**: Subramanyan (2025)
- **Citations**: 0
- **Key Finding**: First WebAssembly runtime ported to CHERI's hardware capability model. Integrates fine-grained bounds, permissions, and pointer provenance into Wasm module execution. Includes CHERI-sealed memory allocator, capability-restricted WASI (cWASI), and secure externref handling.
- **Mister Smith Relevance**: **Medium-High**. If Mister Smith ever supports Wasm-based tool execution, cWAMR demonstrates how capability-based hardware can provide stronger isolation than software-only sandboxing.

### Arrival: Instruction-Selection Verification for Cranelift/Wasm
- **Authors**: McLoughlin, Sheng, Fallin, Parno, Brown, VanHattum (2025)
- **Citations**: 0
- **Journal**: Proc. ACM on Programming Languages
- **Key Finding**: Verifies nearly all AArch64 instruction-selection rules reachable from Wasm core in the Cranelift compiler (used by Wasmtime). Finds new bugs in Cranelift's instruction selection.
- **Mister Smith Relevance**: **Medium**. Cranelift is the Wasm compiler backend used by Wasmtime (Rust-native). If Mister Smith adopts Wasm sandboxing, this work validates the correctness of the compilation pipeline.

### Resource Isolation Attack Surface of WebAssembly Containers
- **Authors**: Yu, Zhan, Ye, Yu, Zhang, Tian (2025)
- **Citations**: 0
- **Key Finding**: Resource isolation is not well protected by current Wasm runtimes. Attackers can exhaust host resources via WASI/WASIX interfaces to interfere with other container instances.
- **Mister Smith Relevance**: **Medium**. Critical caveat for Wasm sandboxing: resource limits must be enforced separately from memory isolation. Mister Smith's ConnectionPool and resource management infrastructure would need to integrate with Wasm resource quotas.

### CHERIoT RTOS: Fine-Grained Compartments for Embedded Devices
- **Authors**: Amar, Chen, Chisnall, Filardo, Laurie, Lefeuvre, Liu, Moore, Norton-Wright, Seltzer, Tao, Watson, Xia (2025)
- **Citations**: 2
- **Journal**: Proc. ACM SIGOPS 31st SOSP
- **Key Finding**: Co-designs OS with CHERI hardware for fine-grained fault-tolerant compartments. OS-level support for compartment-interface hardening and auditing facilities to thwart supply-chain attacks.
- **Mister Smith Relevance**: **Low-Medium**. Forward-looking: as CHERI hardware becomes available in server CPUs, Mister Smith's actor isolation could leverage hardware compartmentalization for stronger agent sandboxing.

---

## 11. Rust-Specific Security and Sandboxing

### SandCell: Sandboxing Rust Beyond Unsafe Code
- **Authors**: Zhang, Gulmez, Nyman, Tan (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2509.24032)
- **Key Finding**: Flexible and lightweight isolation in Rust leveraging existing syntactic boundaries. Allows programmers to specify components to sandbox with minimal annotation. Novel techniques minimize overhead when transferring data between sandboxes. Effective at preventing vulnerabilities while maintaining reasonable performance.
- **Mister Smith Relevance**: **High**. SandCell could be applied to isolate unsafe code in Mister Smith's dependencies (e.g., cryptographic operations, FFI calls). The annotation-based model fits Rust's ergonomic philosophy.

### LiteRSan: Lightweight Memory Safety for Rust
- **Authors**: Xia, Huang, Yu, Jeon, Zhou, Wu, Kim (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2509.16389)
- **Key Finding**: Leverages Rust's ownership model for pointer-lifetime-aware static analysis. Selectively instruments risky pointers, achieving 18.84% runtime overhead (vs 150%+ for ASan-based approaches) with negligible memory overhead.
- **Mister Smith Relevance**: **Medium**. Useful for production deployment hardening. Could be applied to Mister Smith's unsafe code regions for runtime memory safety validation.

### Rust for Safety and Security Critical Systems
- **Authors**: Munch, Lindner, Eriksson, Dzialo, Lindgren (2025)
- **Citations**: 0
- **Journal**: IEEE NorCAS 2025
- **Key Finding**: Identifies challenges and opportunities for Rust in safety-critical systems (ISO 26262, IEC 61508, ISO/SAE 21434). Compares hosted vs. bare-metal certification paths.
- **Mister Smith Relevance**: **Medium**. Validates Rust as appropriate for safety-critical agent orchestration. Relevant if Mister Smith targets regulated industries.

---

## 12. Runtime Guardrails and Behavioral Contracts

### The AI Agent Code of Conduct: Automated Guardrail Policy-as-Prompt
- **Authors**: Kholkar, Ahuja (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2509.23994)
- **Key Finding**: Automates translation of unstructured design documents into verifiable, real-time guardrails. "Policy as Prompt" uses LLMs to interpret and enforce natural language policies. Constructs verifiable policy trees compiled into lightweight prompt-based classifiers for runtime auditing.
- **Mister Smith Relevance**: **Medium**. The policy tree concept could complement Mister Smith's RBAC PolicyEngine. However, prompt-based enforcement is weaker than deterministic enforcement -- best used as an additional layer, not a primary mechanism.

### Policy-as-Prompt: Governance Rules into Guardrails
- **Authors**: Kholkar, Ahuja (2025)
- **Citations**: 0
- **Key Finding**: Extended version focusing on regulatory compliance. System reduces prompt-injection risk, blocks out-of-scope requests, limits toxic outputs. Generates auditable rationales aligned with AI governance frameworks.
- **Mister Smith Relevance**: **Medium**. The audit rationale generation is useful for Mister Smith's AuditLogger, providing human-readable explanations for policy enforcement decisions.

---

## 13. Audit, Provenance, and Accountability

### PROV-AGENT: Unified Provenance for AI Agent Interactions
- **Authors**: Souza, Gueroudji, DeWitt, Rosendo, Ghosal, Ross, Balaprakash, Ferreira da Silva (2025)
- **Citations**: 4
- **Journal**: IEEE eScience 2025
- **Key Finding**: Extends W3C PROV for agentic workflows. Leverages MCP and data observability to integrate agent interactions into end-to-end workflow provenance. Near real-time capture system validated across edge, cloud, and HPC environments.
- **Mister Smith Relevance**: **High**. PROV-AGENT's W3C PROV extension provides a standardized provenance model that could be adopted for Mister Smith's audit logging. The MCP integration is directly applicable. The cross-environment validation demonstrates the approach works in distributed settings like Mister Smith's NATS topology.

### Creating Characteristically Auditable Agentic AI Systems
- **Authors**: Phiri (2025)
- **Citations**: 1
- **Journal**: Proc. Intelligent Robotics FAIR 2025
- **Key Finding**: Formalizes auditability with 8 axioms: Integrity, Coverage, Temporal Coherence, Verifiability, Accessibility, Resource Proportionality, Privacy Compatibility, Governance Alignment. Extends theory with liveness properties (eventual auditability under faults) and adversarial resilience (game-theoretic resistance to log manipulation). All propositions formally proved.
- **Mister Smith Relevance**: **High**. The 8 auditability axioms provide a formal specification for Mister Smith's existing SHA-256 hash chain audit log (Phase 5). The adversarial resilience requirement validates the tamper-evident design choice. Liveness properties should be verified in the context of NATS transport failures.

---

## 14. Human-in-the-Loop and Escalation Models

### Architecting Human-AI Cocreation: Interaction Modes and Contingency Factors
- **Authors**: Wulf, Meierhofer, Hannich (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2507.14034)
- **Key Finding**: Six-mode taxonomy of human-agent interaction: HOOTL (full automation), HAM (passive assistance), HIC (mandatory human approval), HITP (structured workflows), HITL (agent-initiated escalation), HOTL (discretionary oversight). Maps modes to contingency factors (task complexity, risk, system reliability).
- **Mister Smith Relevance**: **High**. This taxonomy should inform Mister Smith's task escalation design. The supervision tree could enforce different interaction modes based on task risk level, with high-risk tool calls requiring HIC (mandatory approval) and low-risk operations running in HOTL mode.

### ORCHID: Human-in-the-Loop for High-Risk Classification
- **Authors**: Mahbub, Lama, Das, et al. (2025)
- **Citations**: 0
- **Key Finding**: Modular agentic system using MCP for tool invocation with append-only audit bundles. Item-to-Evidence-to-Decision loop with step-by-step reasoning. Defers uncertain items to Subject Matter Experts.
- **Mister Smith Relevance**: **Medium**. The MCP-based architecture with append-only audit bundles validates Mister Smith's architectural choices (MCP bridge + immutable audit log).

---

## 15. Supply Chain and Backdoor Attacks

### Malice in Agentland: Backdoors in the AI Supply Chain
- **Authors**: Boisvert, Puri, Evuru, Chapados, Cappart, Lacoste, Dvijotham, Drouin (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.05159)
- **Key Finding**: Formalizes three supply chain threat models: direct data poisoning, environmental poisoning (malicious webpages/tools during training), and supply chain poisoning (pre-backdoored base models). Poisoning 2% of training traces embeds backdoors with >80% ASR. Prominent safeguards (guardrail models, weight-based defenses) all fail to detect the backdoor.
- **Mister Smith Relevance**: **High**. Since Mister Smith is model-agnostic, it must treat every LLM provider as potentially compromised. Defense-in-depth at the infrastructure level (capability-based authorization, IFC, runtime monitoring) is the only reliable defense against supply chain backdoors.

---

## 16. Adversarial Red Teaming and Benchmarks

### Security Challenges in AI Agent Deployment (ART Benchmark)
- **Authors**: Zou, Lin, Jones, Nowak, Dziemian, Winter, Grattan, Nathanael, Croft, Davies, Patel, Kirk, Burnikell, Gal, Hendrycks, Kolter, Fredrikson (2025)
- **Citations**: 4
- **Journal**: ArXiv (abs/2507.20526)
- **Key Finding**: Largest public red-teaming competition: 1.8M prompt-injection attacks against 22 frontier agents across 44 scenarios. 60,000+ successful policy violations. Nearly all agents violate policies within 10-100 queries. Limited correlation between robustness and model size/capability/inference compute. High attack transferability across models and tasks.
- **Mister Smith Relevance**: **Critical**. The finding that model capabilities do NOT correlate with robustness is the strongest argument for infrastructure-level security. Mister Smith's architecture of deterministic policy enforcement outside the LLM is validated by this research.

### AutoRedTeamer: Autonomous Red Teaming
- **Authors**: Zhou, Wu, Pinto, Chen, Zeng, Yang, Yang, Koyejo, Zou, Li (2025)
- **Citations**: 12
- **Journal**: ArXiv (abs/2503.15754)
- **Key Finding**: Multi-agent architecture for automated red teaming with memory-guided attack selection. Achieves 20% higher ASR on HarmBench with 46% reduced computational costs.
- **Mister Smith Relevance**: **Medium**. Could be used to test Mister Smith's security controls. The memory-guided attack selection mirrors how persistent adversaries would probe agent defenses.

### CRAFT: Red-Teaming Policy-Adherent Agents
- **Authors**: Nakash, Kour, Lazar, Vetzler, Uziel, Anaby-Tavor (2025)
- **Citations**: 2
- **Journal**: ArXiv (abs/2506.09600)
- **Key Finding**: Policy-aware persuasive strategies outperform conventional jailbreak methods (DAN prompts, emotional manipulation) against policy-adherent agents. Introduces tau-break benchmark.
- **Mister Smith Relevance**: **Medium**. The policy-aware attack strategies should be considered when testing Mister Smith's security guardrails.

---

## 17. Confidential Computing and TEEs

### VM-based TEEs for Confidential Federated Learning
- **Authors**: Casella (2025)
- **Citations**: 0
- **Journal**: Euromicro PDP 2025
- **Key Finding**: New VM-based TEEs (vs. application-level TEEs like SGX) introduce limited overhead (at most 1.5x) for confidential computing, making public/untrusted environments viable.
- **Mister Smith Relevance**: **Medium**. Relevant for future deployment scenarios where Mister Smith agents run in untrusted cloud environments.

### DeepSeek in Confidential Computing (Intel TDX)
- **Authors**: Dong, Wang (2025)
- **Citations**: 4
- **Journal**: ArXiv (abs/2502.11347)
- **Key Finding**: First evaluation of LLM inference within TEE-enabled environments (Intel TDX). For smaller models, TDX implementation outperforms CPU-only while maintaining security.
- **Mister Smith Relevance**: **Medium**. Validates that LLM inference inside TEEs is feasible, relevant for Mister Smith deployments requiring confidential inference.

---

## 18. Comprehensive Threat Models and Surveys

### Securing Agentic AI: ATFAA and SHIELD Frameworks
- **Authors**: Narajala, Narayan (2025)
- **Citations**: 14
- **Journal**: ArXiv (abs/2504.19956)
- **Key Finding**: Identifies 9 primary threats across 5 domains: cognitive architecture vulnerabilities, temporal persistence threats, operational execution vulnerabilities, trust boundary violations, and governance circumvention. Introduces ATFAA (threat framework) and SHIELD (mitigation framework).
- **Mister Smith Relevance**: **High**. The 5-domain threat taxonomy maps to Mister Smith's architecture: cognitive (LLM provider), temporal persistence (state management), operational execution (agent runtime), trust boundaries (security layer), governance (supervision tree).

### From Prompt Injections to Protocol Exploits
- **Authors**: Ferrag, Tihanyi, Hamouda, Maglaras, Debbah (2025)
- **Citations**: 6
- **Journal**: ArXiv (abs/2506.23260)
- **Key Finding**: First unified end-to-end threat model for LLM-agent ecosystems. Catalogs 30+ attack techniques across 4 domains: input manipulation, model compromise, system/privacy attacks, protocol vulnerabilities (MCP, ACP, ANP, A2A).
- **Mister Smith Relevance**: **High**. The 30+ attack technique catalog serves as a comprehensive threat checklist for Mister Smith's security testing.

### Trust-Vulnerability Paradox (TVP) in Multi-Agent Systems
- **Authors**: Xu, Qi, Wu, Zhang, Wei, He, Li (2025)
- **Citations**: 0
- **Journal**: ArXiv (abs/2510.18563)
- **Key Finding**: Empirically validates that increasing inter-agent trust improves task success but simultaneously expands exposure risks. Proposes Over-Exposure Rate (OER) and Authorization Drift (AD) metrics. Trust must be modeled as a first-class security variable.
- **Mister Smith Relevance**: **High**. OER and AD should be added to Mister Smith's monitoring metrics. The TVP formalizes the trade-off that Mister Smith's supervision tree must manage: team agents need trust to collaborate, but that trust creates attack surface.

### Trustworthy Agentic AI: Cross-Layer Review
- **Authors**: Adabara, Sadiq, Shuaibu, Danjuma, Maninti (2025)
- **Citations**: 1
- **Journal**: F1000Research
- **Key Finding**: Cross-layer review spanning architectural paradigms, threat taxonomies, and governance strategies. Identifies research gaps in benchmarking, memory integrity, adversarial defense, and normative embedding.
- **Mister Smith Relevance**: **Medium**. Provides broad contextual framing for Mister Smith's security posture.

### Ensuring Secure Voice Agents (Mediator Architecture)
- **Authors**: Artemenko, Khudik (2025)
- **Citations**: 0
- **Journal**: Cybersecurity: Education, Science, Technique
- **Key Finding**: Multi-level security via least privilege for LLM components, independent validation at domain service level, and LLM as mediator (not decision-maker) between users and critical systems.
- **Mister Smith Relevance**: **Medium**. The mediator architecture principle (LLM proposes, infrastructure disposes) aligns with Mister Smith's design philosophy.

---

## 19. Synthesis: Implications for Mister Smith

### Critical Design Principles Validated by Research

1. **Infrastructure-Level Security is Non-Negotiable**: Multiple papers (OrgAccess F1=0.27, ART benchmark, Zou et al.) demonstrate that LLMs cannot reliably enforce security policies. Mister Smith's architecture of deterministic security enforcement in the Rust infrastructure layer (SecurityLayer, PolicyEngine, AuditLogger) is strongly validated.

2. **Information Flow Control as Foundation**: The Microsoft Fides paper and IBM's SAMOS provide the strongest theoretical and practical frameworks for securing agent systems. Dual-label taint tracking (confidentiality + integrity) on Mister Smith's MessageEnvelope is the recommended approach for Phase 10+ security hardening.

3. **Capability-Based Authorization over RBAC Alone**: Progent, AgentSentry, and the distributed capabilities paper argue for fine-grained, task-scoped, dynamically revocable capability tokens. Mister Smith should extend its RBAC with capability-based authorization for tool access, with tokens that attenuate (reduce privileges) as they propagate through the agent hierarchy.

4. **Plan-then-Execute for Control Flow Integrity**: The P-t-E pattern, IPIGuard TDG, and VeriGuard all converge on the same insight: separating planning from execution and validating the plan before allowing tool calls is the most effective structural defense against prompt injection.

5. **Trust is a Security Variable**: The TVP paper formalizes what Mister Smith's supervision tree implicitly manages. Over-Exposure Rate and Authorization Drift should be instrumented as metrics in Mister Smith's monitoring system.

### Architecture-Specific Recommendations

| Mister Smith Component | Relevant Research | Recommended Enhancement |
|---|---|---|
| **MessageEnvelope** | Fides (IFC), SAMOS | Add confidentiality and integrity taint labels |
| **ToolBus** | Progent, IPIGuard, XTHP | DSL-based privilege policies, Tool Dependency Graph validation |
| **SecurityLayer** | OrgAccess, ART benchmark | Deterministic enforcement only; never delegate to LLM |
| **AgentRuntime** | AgentSentry, P-t-E | Task-scoped ephemeral capabilities, plan validation before execution |
| **Supervision Tree** | Allegrini et al. (temporal logic), Dewes (contracts) | Formal assume-guarantee contracts per supervised agent |
| **AuditLogger** | PROV-AGENT, Phiri (8 axioms) | W3C PROV extension, formal auditability verification |
| **Persistence** | MINJA, SpAIware | Integrity checks on state writes, provenance on reads |
| **MCP Bridge** | MSB (12 attacks), SAMOS | Gateway-level IFC, tool description validation |
| **NATS Transport** | Maris (reference monitors) | Subject-based IFC enforcement, message taint propagation |

---

## 20. Emerging Directions

### Highest-Signal Emerging Research Areas

1. **Distributed Backdoor Attacks in Multi-Agent Systems** (Zhu et al.): Attack primitives dormant in individual tools that activate only when agents collaborate in specific sequences. This is a novel threat class with no established defenses. Mister Smith's supervision tree could potentially detect anomalous collaboration patterns through cross-agent behavioral correlation.

2. **Hardware-Enforced Capabilities for Agent Sandboxing** (cWAMR, CHERIoT): As CHERI hardware reaches server CPUs, the combination of Rust's type-system safety with hardware-enforced capability boundaries could provide the strongest possible isolation for agent execution. This would elevate Mister Smith's actor isolation from software-level (bounded mailboxes, Tokio task isolation) to hardware-level compartmentalization.

3. **Inter-Agent Trust Protocol Design** (Hu & Rong): The six-mechanism trust taxonomy (Brief, Claim, Proof, Stake, Reputation, Constraint) provides a design space for Mister Smith to evolve beyond simple JWT-based trust. Particularly interesting is the combination of Proof (cryptographic) + Constraint (sandbox) as the foundational layer, with Reputation overlays for dynamic trust adjustment.

4. **Formally Verifiable Agent Properties** (Allegrini et al.): 31 temporal logic properties (liveness, safety, completeness, fairness) for agentic AI systems. This opens the path to model-checking Mister Smith's supervision tree and agent lifecycle state machines, potentially catching deadlocks and security vulnerabilities at design time rather than runtime.

5. **Policy-as-Code with LLM-Generated Policies** (Progent): LLMs can automatically generate effective security policies from natural language specifications. Combined with deterministic enforcement (the policies are compiled, not interpreted by the LLM), this could dramatically reduce the burden of policy authoring for Mister Smith deployments.

6. **Trust-Vulnerability Paradox Metrics** (Xu et al.): OER and AD as quantifiable security metrics for multi-agent trust calibration. These could be instrumented in Mister Smith's Prometheus/OpenTelemetry stack to provide real-time visibility into trust-security trade-offs.

7. **Provenance-Aware Agent Workflows** (PROV-AGENT): W3C PROV extension for agentic systems, integrated with MCP. Near real-time provenance capture across heterogeneous environments. This is the most mature provenance framework for the agent ecosystem and could be adopted as Mister Smith's standard provenance model.

### Speculative / Early-Stage Threads

- **Zero-Knowledge Proofs for inter-agent trust** (Huang et al.): Agents prove capabilities without revealing implementation details. Relevant for cross-organization Mister Smith deployments.
- **Genetic algorithm-based stealthy attacks on tool descriptions** (MPMA): Implies that tool description validation must go beyond simple string matching to semantic analysis.
- **ai.txt DSL for AI-Internet interactions** (Li et al.): A "robots.txt for AI" that could influence how Mister Smith agents interact with external web resources.
- **Verifiable Mandates in SSI** (Turkanovic et al.): Formal delegation semantics with OPA/Rego policy enforcement, applicable to agent delegation chains.

---

## Citation Index (Alphabetical by First Author)

| # | Citation | Year | Citations | Primary Topic |
|---|---|---|---|---|
| 1 | Adabara et al., "Trustworthy agentic AI systems" | 2025 | 1 | Threat taxonomy |
| 2 | Alizadeh et al., "Simple prompt injection attacks can leak personal data" | 2025 | 6 | Data exfiltration |
| 3 | Allegrini et al., "Formalizing safety, security, and functional properties" | 2025 | 0 | Formal methods |
| 4 | Amar et al., "CHERIoT RTOS" | 2025 | 2 | Hardware capabilities |
| 5 | An et al., "IPIGuard: Tool Dependency Graph defense" | 2025 | 5 | Prompt injection defense |
| 6 | Artemenko & Khudik, "Ensuring secure operating of voice agents" | 2025 | 0 | Mediator architecture |
| 7 | Baheri & Wei, "Stratified Metric Temporal Logic" | 2025 | 0 | Formal verification |
| 8 | Boisvert et al., "Malice in Agentland" | 2025 | 0 | Supply chain attacks |
| 9 | Cai et al., "AgentSentry: Task-centric access control" | 2025 | 0 | Access control |
| 10 | Casella, "VM-based TEEs for Confidential FL" | 2025 | 0 | Confidential computing |
| 11 | Chen & Cong, "AgentGuard: Safety evaluation of tool orchestration" | 2025 | 8 | Tool safety |
| 12 | Cheng et al., "Adaptive CHERI Compartmentalization" | 2025 | 2 | Hardware capabilities |
| 13 | Costa et al., "Securing AI Agents with IFC (Fides)" | 2025 | 11 | Information flow control |
| 14 | Cui et al., "Safeguard-by-Development (Maris)" | 2025 | 1 | Reference monitors |
| 15 | Del Rosario et al., "Architecting Resilient LLM Agents (P-t-E)" | 2025 | 2 | Secure architecture |
| 16 | Dewes & Dimitrova, "Contract-based verification of MAS" | 2025 | 0 | Formal contracts |
| 17 | Dong & Wang, "DeepSeek in Confidential Computing" | 2025 | 4 | Confidential computing |
| 18 | Dong et al., "Memory Injection Attacks (MINJA)" | 2025 | 18 | Memory attacks |
| 19 | Ferrag et al., "From Prompt Injections to Protocol Exploits" | 2025 | 6 | Threat survey |
| 20 | Ganie, "Securing AI Agents: RBAC for Industrial Applications" | 2025 | 0 | RBAC |
| 21 | Garzon et al., "AI Agents with DIDs and VCs" | 2025 | 0 | Decentralized identity |
| 22 | Habler et al., "Building Secure Agentic AI with A2A" | 2025 | 28 | Protocol security |
| 23 | Herrador & Rehberger, "SpAIware" | 2025 | 4 | Memory attacks |
| 24 | Hossain et al., "Multi-Agent LLM Defense Pipeline" | 2025 | 0 | Prompt injection defense |
| 25 | Hosseini & Seilani, "Agentic AI systematic review" | 2025 | 47 | Survey |
| 26 | Hou et al., "MCP: Landscape, Security Threats, Future" | 2025 | 126 | MCP security |
| 27 | Hu & Rong, "Inter-Agent Trust Models" | 2025 | 0 | Trust models |
| 28 | Huang et al., "Zero-Trust Identity Framework for Agentic AI" | 2025 | 6 | Zero-trust identity |
| 29 | Jeong & Lee, "Permission-Aware RAG" | 2025 | 0 | Access control |
| 30 | Jiang et al., "NaviAgent: Tool Dependency Graphs" | 2025 | 0 | Tool planning |
| 31 | Kholkar & Ahuja, "AI Agent Code of Conduct" | 2025 | 0 | Runtime guardrails |
| 32 | Kholkar & Ahuja, "Policy-as-Prompt" | 2025 | 0 | Policy enforcement |
| 33 | Kong et al., "Survey of LLM-Driven Agent Communication" | 2025 | 19 | Communication security |
| 34 | Li et al., "ai.txt: DSL for AI interactions" | 2025 | 0 | DSL |
| 35 | Li et al., "Les Dissonances: Cross-Tool Harvesting (XTHP)" | 2025 | 1 | Tool attacks |
| 36 | Liu et al., "Secure Multi-LLM by Zero-Trust" | 2025 | 1 | Zero-trust |
| 37 | Maharana et al., "OrgAccess: RBAC Benchmark" | 2025 | 2 | RBAC evaluation |
| 38 | Meijer, "Guardians of the Agents" | 2025 | 0 | Formal verification |
| 39 | Miculicich et al., "VeriGuard" | 2025 | 0 | Formal verification |
| 40 | Munch et al., "Rust for Safety/Security Critical Systems" | 2025 | 0 | Rust security |
| 41 | Nakash et al., "CRAFT: Red-Teaming Policy-Adherent Agents" | 2025 | 2 | Red teaming |
| 42 | Narajala & Narayan, "Securing Agentic AI (ATFAA/SHIELD)" | 2025 | 14 | Threat model |
| 43 | Ntousakis et al., "Securing MCP-based Workflows (SAMOS)" | 2025 | 0 | MCP security |
| 44 | Palavali, "Agentic AI for Self-Sovereign Identity" | 2025 | 1 | DID/SSI |
| 45 | Phiri, "Characteristically Auditable Agentic AI" | 2025 | 1 | Auditability |
| 46 | Saha et al., "sudoLLM: Multi-role Alignment" | 2025 | 2 | Authorization |
| 47 | Shi et al., "Progent: Programmable Privilege Control" | 2025 | 17 | Capability control |
| 48 | South et al., "Authenticated Delegation" | 2025 | 20 | Delegation |
| 49 | Souza et al., "PROV-AGENT: Provenance Tracking" | 2025 | 4 | Audit/provenance |
| 50 | Subramanyan, "cWAMR: Capability-Based Wasm via CHERI" | 2025 | 0 | Wasm/CHERI |
| 51 | Tran et al., "Multi-Agent Collaboration Mechanisms Survey" | 2025 | 190 | MAS survey |
| 52 | Turkanovic et al., "Enforcing Delegated Authority in SSI" | 2025 | 0 | SSI/delegation |
| 53 | Wang et al., "MPMA: Preference Manipulation vs MCP" | 2025 | 9 | MCP attacks |
| 54 | Wang et al., "NANDA Framework" | 2025 | 2 | Agent discovery |
| 55 | Wang et al., "Unveiling Privacy Risks in Agent Memory (MEXTRA)" | 2025 | 24 | Memory privacy |
| 56 | White et al., "Cloud-Scale Distributed Capabilities" | 2025 | 0 | Capabilities |
| 57 | Wulf et al., "Human-AI Cocreation Interaction Modes" | 2025 | 0 | HITL |
| 58 | Xia et al., "LiteRSan: Lightweight Memory Safety" | 2025 | 0 | Rust safety |
| 59 | Xu et al., "Trust-Vulnerability Paradox" | 2025 | 0 | Trust/security |
| 60 | Yu et al., "Wasm Resource Isolation Attack Surface" | 2025 | 0 | Wasm security |
| 61 | Zhang et al., "MCP Security Bench (MSB)" | 2025 | 0 | MCP benchmark |
| 62 | Zhang et al., "SandCell: Sandboxing Rust" | 2025 | 0 | Rust sandboxing |
| 63 | Zhou et al., "AutoRedTeamer" | 2025 | 12 | Red teaming |
| 64 | Zhu et al., "Collaborative Shadows: Distributed Backdoor" | 2025 | 0 | Backdoor attacks |
| 65 | Zou et al., "Security Challenges in Agent Deployment (ART)" | 2025 | 4 | Benchmark |

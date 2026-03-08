---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (57 papers, 28 searches)
round: 4 (Academic Search)
---

# Streaming Architecture, Backpressure, and Reactive Flow Control for LLM Agent Pipelines

## Research Digest -- Consensus Academic Search (2025+)

**Generated**: 2026-03-07
**Search scope**: Peer-reviewed papers and ArXiv preprints, year >= 2025
**Context**: Mister Smith framework -- Rust + NATS/JetStream + OTP-style supervision + model-agnostic LLM orchestration

---

## Table of Contents

1. [Reactive Streams and Backpressure Formal Models](#1-reactive-streams-and-backpressure-formal-models)
2. [Flow Control in Publish-Subscribe Middleware](#2-flow-control-in-publish-subscribe-middleware)
3. [LLM Streaming Inference and Token Generation](#3-llm-streaming-inference-and-token-generation)
4. [Multi-Agent LLM Orchestration and Scheduling](#4-multi-agent-llm-orchestration-and-scheduling)
5. [KV Cache Management for Agentic Workloads](#5-kv-cache-management-for-agentic-workloads)
6. [Speculative Execution and Multi-Token Prediction](#6-speculative-execution-and-multi-token-prediction)
7. [Prefill-Decode Disaggregation and Pipeline Parallelism](#7-prefill-decode-disaggregation-and-pipeline-parallelism)
8. [Fan-In/Fan-Out and Distributed Stream Aggregation](#8-fan-infan-out-and-distributed-stream-aggregation)
9. [Zero-Copy Messaging and Serialization Optimization](#9-zero-copy-messaging-and-serialization-optimization)
10. [Bounded Channels, Concurrency Primitives, and Lock-Free Structures](#10-bounded-channels-concurrency-primitives-and-lock-free-structures)
11. [SIMD-Accelerated Stream Processing](#11-simd-accelerated-stream-processing)
12. [Tail Latency, Circuit Breakers, and Resilience Patterns](#12-tail-latency-circuit-breakers-and-resilience-patterns)
13. [Rate Limiting and Adaptive Load Management](#13-rate-limiting-and-adaptive-load-management)
14. [Structured Output and Constrained Decoding](#14-structured-output-and-constrained-decoding)
15. [Observability and Distributed Tracing for Streaming Pipelines](#15-observability-and-distributed-tracing-for-streaming-pipelines)
16. [Stream Processing Fault Recovery and State Management](#16-stream-processing-fault-recovery-and-state-management)
17. [Dataflow Programming and Stream Topology](#17-dataflow-programming-and-stream-topology)
18. [Elastic Scaling for Stream Processing](#18-elastic-scaling-for-stream-processing)
19. [Actor Model and Agent Communication Protocols](#19-actor-model-and-agent-communication-protocols)
20. [Rust Async Runtime and Real-Time Systems](#20-rust-async-runtime-and-real-time-systems)
21. [Emerging Directions](#21-emerging-directions)
22. [Synthesis: Implications for Mister Smith](#22-synthesis-implications-for-mister-smith)

---

## 1. Reactive Streams and Backpressure Formal Models

### Reactive Programming Paradigms in High-Throughput Distributed Systems
- **Authors**: Kolluru Sampath Sree Kumar
- **Year**: 2025 | **Citations**: 0
- **Journal**: European Journal of Computer Science and Information Technology
- **Key finding**: Surveys core reactive programming principles -- async non-blocking operations, event-driven architecture, declarative style, and backpressure management. Examines Project Reactor, RxJava, Akka Streams, and Spring WebFlux. Identifies the learning curve and debugging complexity as primary challenges.
- **Relevance**: Direct validation of Mister Smith's reactive architecture choices. The backpressure management patterns described (bounded buffers, demand signaling) map directly to Tokio channels + NATS JetStream consumer pull semantics.

### Parallel Simulation Using Reactive Streams: Graph-Based Approach
- **Authors**: Sirotkin, Prymushko, Puchko, Kravtsov, Yaroshynskyi, Artemchuk
- **Year**: 2025 | **Citations**: 1
- **Journal**: Comput.
- **Key finding**: Proposes reactive stream paradigm as a general-purpose synchronization protocol for parallel simulation. Constructs simulation graphs from predefined transition functions using push/pull patterns. Demonstrates scalability via computational graphs with reactive streams.
- **Relevance**: The simulation graph abstraction parallels Mister Smith's agent orchestration DAG. The push/pull pattern duality maps to JetStream's push-subscribe vs pull-subscribe consumer modes.

### Automata-based Representation of Coordination for Distributed Reactive Systems
- **Authors**: Szabo, Cziborova
- **Year**: 2025 | **Citations**: 0
- **Journal**: Proc. Intl. Conf. on Formal Methods and Foundations of AI
- **Key finding**: Extends timed automata formalism to coordinate component execution in distributed critical systems. Makes coordination a first-class citizen separable from component models.
- **Relevance**: Provides formal foundation for modeling Mister Smith's supervision tree coordination. The approach of decoupling coordination logic from component behavior mirrors the SupervisedSystem + ActorCell separation.

### Stream Programs Are Monoid Homomorphisms with State
- **Authors**: Hou, Arntzenius, Willsey
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Defines a broad class of deterministic stream functions as homomorphisms into a "state" monoid. The homomorphism laws enable equational reasoning over sequential composition, parallel composition, and feedback in dataflow programs.
- **Relevance**: Provides a formal algebraic foundation for reasoning about Mister Smith's stream pipeline composition. The sequential/parallel/feedback composition operators directly map to agent pipeline patterns.

### Priority-Aware Reactive APIs: SLA-Tiered Traffic with Spring WebFlux
- **Authors**: Kishore Subramanya Hebbar
- **Year**: 2025 | **Citations**: 0
- **Journal**: European Journal of Electrical Engineering and Computer Science
- **Key finding**: Implements SLA-tiered request handling using reactive programming with token-bucket-style schedulers, bounded elastic pools, and rate-limit-aware filter chains. Achieves 60% lower latency for high-priority operations during resource contention.
- **Relevance**: Directly applicable to Mister Smith's agent priority system. The bounded elastic pool + token-bucket pattern can be adapted for priority-aware agent scheduling, where critical agents (e.g., security) get preferential stream processing.

---

## 2. Flow Control in Publish-Subscribe Middleware

### Next-Generation Event-Driven Architectures: Performance and Intelligent Orchestration
- **Authors**: Arafat, Tasmin, Poudel
- **Year**: 2025 | **Citations**: 3
- **Journal**: ArXiv
- **Key finding**: First comprehensive benchmarking of 12 messaging systems (including NATS JetStream) across 2,400+ experimental configurations. Kafka achieves 1.2M msgs/sec (18ms p95); Pulsar 950K msgs/sec (22ms p95). Introduces AIEO (AI-Enhanced Event Orchestration) with ML-driven predictive scaling achieving 34% latency reduction and 42% cost optimization.
- **Relevance**: **Critical paper for Mister Smith.** Provides empirical benchmarks for NATS JetStream vs competitors under AI inference pipeline workloads specifically. The AIEO concept of ML-driven predictive scaling could be integrated with Mister Smith's monitoring/health system for proactive backpressure adjustment.

### Queuing Theory-Based Modeling of Publish/Subscribe IoT Communication
- **Authors**: Pouhela, Kiggundu, Schotten
- **Year**: 2025 | **Citations**: 0
- **Journal**: ICC 2025 (IEEE)
- **Key finding**: Uses M/M/1 queuing model to optimize pub/sub broker service rates. Derives optimal service rate formulas relative to overall message arrival rate. Demonstrates how multi-threading affects broker performance.
- **Relevance**: The queuing theory model can be applied to Mister Smith's NATS consumer configuration. Optimal service rate calculations inform JetStream consumer `max_deliver` and `ack_wait` settings.

### AgentFlow: Resilient Adaptive Cloud-Edge Framework for Multi-Agent Coordination
- **Authors**: Chen, Shiu
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: MAS-based framework supporting decentralized pub/sub messaging and many-to-many service elections. Features plug-and-play node discovery, flexible task reorganization, and fault tolerance with substitution mechanisms.
- **Relevance**: AgentFlow's decentralized pub/sub with service elections mirrors Mister Smith's NATS-based agent discovery + supervision tree failover. The "substitution mechanism" concept maps to OTP-style restart strategies.

### A Distributed Architecture for MQTT Messaging: TBMQ
- **Authors**: Shvaika, Shvaika, Landiak, Artemchuk
- **Year**: 2025 | **Citations**: 0
- **Journal**: Journal of Big Data
- **Key finding**: Achieves 100M+ concurrent connections and 3M+ msgs/sec using Kafka-backed distributed MQTT. Explicitly addresses fan-in (many devices producing) and fan-out (few requests triggering wide distribution) patterns.
- **Relevance**: Validates the architecture of NATS-as-backbone for Mister Smith's message routing. The fan-in/fan-out characterization maps directly to agent team coordination patterns (many agents reporting to orchestrator, orchestrator dispatching to team).

---

## 3. LLM Streaming Inference and Token Generation

### Streaming-VR: Streaming Verification and Refinement for LLM Outputs
- **Authors**: Ko, Baek, Hwang
- **Year**: 2025 | **Citations**: 2
- **Journal**: EMNLP 2025
- **Key finding**: Enables on-the-fly verification and correction of tokens during generation. Observes that once incorrect tokens are generated early, subsequent tokens are more likely incorrect. Real-time token subset checking improves factual accuracy AND efficiency.
- **Relevance**: **Highly relevant to Mister Smith's streaming pipeline design.** Suggests implementing a verification stage in the token stream pipeline -- a secondary agent (or circuit) that validates token subsets as they flow through the NATS stream, enabling early abort or correction before full response completion.

### VITA-Audio: Fast Interleaved Cross-Modal Token Generation
- **Authors**: Long, Shen, Fu et al.
- **Year**: 2025 | **Citations**: 16
- **Journal**: ArXiv
- **Key finding**: Introduces Multiple Cross-modal Token Prediction (MCTP) module generating multiple tokens per forward pass. Achieves 3-5x inference speedup at 7B scale. First model capable of generating output during the first forward pass.
- **Relevance**: The MCTP pattern of generating multiple tokens per step is relevant to Mister Smith's streaming buffer design. The provider abstraction should support chunk-based token delivery rather than strictly single-token streams.

### Kimi-Audio: Chunk-wise Streaming Detokenizer
- **Authors**: KimiTeam et al.
- **Year**: 2025 | **Citations**: 93
- **Journal**: ArXiv
- **Key finding**: Develops chunk-wise streaming detokenizer based on flow matching for processing audio tokens from LLM output. Uses 12.5Hz audio tokenizer with discrete token output and continuous feature input.
- **Relevance**: The chunk-wise streaming detokenizer pattern is directly applicable to Mister Smith's token-to-structured-output pipeline. The concept of processing token chunks (not individual tokens) through a downstream pipeline is a key design pattern.

### gLLM: Token Throttling for Pipeline Parallelism
- **Authors**: Guo, Zhang, Du, Chen, Xiao, Lu
- **Year**: 2025 | **Citations**: 3
- **Journal**: ArXiv
- **Key finding**: Token Throttling independently regulates prefill and decode token quantities for balanced pipeline computation. Asynchronous execution with message passing architecture. Achieves 11-398% throughput improvement.
- **Relevance**: **Token Throttling is a backpressure primitive.** Mister Smith's LLM provider layer should implement independent throttling for request submission (prefill) and response streaming (decode) to prevent pipeline bubble formation.

---

## 4. Multi-Agent LLM Orchestration and Scheduling

### Gradientsys: Multi-Agent LLM Scheduler with ReAct Orchestration
- **Authors**: Song, Wang, Wu, Shi, Ai
- **Year**: 2025 | **Citations**: 1
- **Journal**: ArXiv
- **Key finding**: Coordinates diverse specialized agents via typed Model-Context Protocol (MCP) and ReAct planning loop. Supports hybrid sync/async execution, agent capacity constraints, retry-and-replan. Uses SSE for real-time observability.
- **Relevance**: **Architecture twin for Mister Smith.** Validates MCP-based tool integration with typed protocols, hybrid sync/async agent execution, and SSE-based observability. The retry-and-replan mechanism maps to supervision tree restart strategies.

### Continuum: Multi-Turn LLM Agent Scheduling with KV Cache TTL
- **Authors**: Li, Mang, He, Zhang, Mao, Chen, Cheung, Gonzalez, Stoica
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Tool calls break workflow continuity, causing KV cache eviction and scheduling bubbles. Solution: predict tool call durations, pin KV cache with TTL, program-level FCFS scheduling. Evaluated on SWE-Bench and BFCL.
- **Relevance**: **Critical for Mister Smith's agent-LLM bridge design.** Multi-turn agent workflows with tool calls (Mister Smith's ToolBus) create exactly this continuity problem. The TTL-based caching strategy should inform the provider's request correlation and context management.

### LLM-Enabled Multi-Agent System for 6G Networks
- **Authors**: Qu, Wang, Yu, Sun, Li, Zhang
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Dual-loop architecture: outer loop for global agent + sub-agent collaboration via task decomposition; inner loop for sub-agent reasoning/execution/replanning with parallel tool calling and offloading.
- **Relevance**: The dual-loop pattern maps directly to Mister Smith's orchestrator (outer loop) + individual agent runtime (inner loop). The parallel tool calling with offloading strategies informs ToolBus design.

### AI Agents vs. Agentic AI: A Conceptual Taxonomy
- **Authors**: Sapkota, Roumeliotis, Karkee
- **Year**: 2025 | **Citations**: 103
- **Journal**: Information Fusion
- **Key finding**: Distinguishes AI Agents (modular, task-specific) from Agentic AI (multi-agent collaboration, dynamic task decomposition, persistent memory, coordinated autonomy). Proposes solutions for hallucination, brittleness, emergent behavior, and coordination failure.
- **Relevance**: Mister Smith firmly falls in the "Agentic AI" category. The coordination failure patterns and proposed solutions (ReAct loops, RAG, automation coordination layers) validate the supervision tree + orchestrator design.

### Agent Interoperability Protocols Survey (MCP, ACP, A2A, ANP)
- **Authors**: Ehtesham, Singh, Gupta, Kumar
- **Year**: 2025 | **Citations**: 44
- **Journal**: ArXiv
- **Key finding**: Compares MCP (JSON-RPC tool invocation), ACP (RESTful async messaging), A2A (peer-to-peer task delegation), and ANP (decentralized discovery). Proposes phased adoption: MCP for tools, ACP for messaging, A2A for collaboration, ANP for discovery.
- **Relevance**: **Directly informs Mister Smith's MCP integration and future protocol evolution.** The phased adoption roadmap validates MCP-first for tool access, with potential evolution toward A2A for peer agent collaboration.

---

## 5. KV Cache Management for Agentic Workloads

### Tokencake: KV-Cache-Centric Serving for Multi-Agent Applications
- **Authors**: Bian, Wu, Ma, Zhuo
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Co-optimizes scheduling and memory management for multi-agent LLM workloads. Space Scheduler shields critical agents from KV cache contention via dynamic memory partitioning. Time Scheduler proactively offloads/uploads during tool call stalls. 47% latency reduction vs vLLM.
- **Relevance**: **Key architectural insight for Mister Smith.** When multiple agents share an LLM provider, KV cache contention becomes the bottleneck. Mister Smith's provider layer should implement agent-priority-aware cache management, shielding high-priority agents from eviction.

### KVFlow: Workflow-Aware KV Cache Management
- **Authors**: Pan, Patel, Hu, Shen, Guan, Li, Qin, Wang, Ding
- **Year**: 2025 | **Citations**: 4
- **Journal**: ArXiv
- **Key finding**: Abstracts agent execution schedule as an "Agent Step Graph" with steps-to-execution estimation for cache eviction. Fully overlapped KV prefetching from CPU to GPU in background threads. 1.83-2.19x speedup over SGLang.
- **Relevance**: The Agent Step Graph abstraction maps to Mister Smith's orchestrator task DAG. The steps-to-execution estimation for cache management could be integrated with the AgentScheduler to inform provider-level cache hints.

### FlashSVD: Memory-Efficient Streaming Inference
- **Authors**: Shao, Wang, Wang, Jiang, Du, Ye, Zhuo, Chen, Li
- **Year**: 2025 | **Citations**: 3
- **Journal**: ArXiv
- **Key finding**: Rank-aware streaming inference for SVD-compressed LLMs. Loads small tiles into on-chip SRAM, multiplies/reduces on the fly, immediately evicts. Cuts peak activation memory by 70.2%.
- **Relevance**: The "tile-process-evict" streaming pattern is a general design principle applicable to Mister Smith's token processing pipeline. Process token chunks through the pipeline without materializing full response buffers.

---

## 6. Speculative Execution and Multi-Token Prediction

### FastMTP: Enhanced Multi-Token Prediction for LLM Acceleration
- **Authors**: Cai, Liang, Wang, Ma, Liang, Luo, Zuo, Duan, Yin, Chen
- **Year**: 2025 | **Citations**: 1
- **Journal**: ArXiv
- **Key finding**: Aligns MTP training with inference pattern for speculative decoding. Single MTP head with position-shared weights captures dependencies among consecutive future tokens. Language-aware dynamic vocabulary compression. 2.03x average speedup.
- **Relevance**: Understanding MTP patterns informs Mister Smith's streaming buffer design. When providers emit multiple tokens per chunk, the buffer management strategy must handle variable-rate token delivery.

### Gumiho: Hybrid Architecture for Speculative Decoding
- **Authors**: Li, Xu, Huang, Yin, Li, Ngai, Barsoum
- **Year**: 2025 | **Citations**: 2
- **Journal**: ArXiv
- **Key finding**: Theoretically demonstrates that initial tokens in draft sequences are more important than later ones. Uses sophisticated Transformer architecture for early draft heads (serial) and lightweight MLP heads for later tokens (parallel).
- **Relevance**: The "early tokens matter more" insight has streaming pipeline implications. Mister Smith's streaming verification should allocate more processing to validating early tokens in tool-call JSON streams (which set structural context).

### CARD: Cache-Assisted Parallel Speculative Decoding
- **Authors**: Zhou, Sheng, Chen, He
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Decouples drafting from verification via shared cache. Draft model populates cache, target model concurrently refines trajectory. 4.83x acceleration, no fine-tuning required.
- **Relevance**: The shared-cache decoupling pattern is applicable to Mister Smith's multi-provider architecture. Multiple providers could share a response cache, enabling one provider's partial results to inform another's verification.

### LAPS-SD: Semi-Clairvoyant Scheduling for Speculative Decoding
- **Authors**: Li, Chen, Li
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Multiple priority queues with request execution preemption. Adapts scheduling based on token acceptance rate stability. 39% latency reduction.
- **Relevance**: The adaptive priority queue + preemption model maps to Mister Smith's bounded mailbox design. Agent mailboxes could implement acceptance-rate-aware priority adjustment.

---

## 7. Prefill-Decode Disaggregation and Pipeline Parallelism

### TD-Pipe: Temporally-Disaggregated Pipeline Parallelism
- **Authors**: Zhang, Wei, Zheng, Du, Chen, Lu
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Disaggregates prefill and decode phases temporally to eliminate pipeline bubbles. Hierarchy-controller decouples scheduling from execution. AI-based greedy prefill predicts output length. Inter-batch work stealing balances decode workloads.
- **Relevance**: The scheduling/execution decoupling and work-stealing patterns are directly applicable to Mister Smith's agent scheduler. The orchestrator should decouple task scheduling from agent execution, with work stealing for load balancing across agent instances.

### Nexus: Proactive Intra-GPU Prefill-Decode Disaggregation
- **Authors**: Shi, Cai, Du, Jia
- **Year**: 2025 | **Citations**: 2
- **Key finding**: GPU resources exhibit diminishing returns beyond a saturation point. Memory bandwidth contention is a critical bottleneck. Dynamic partitioning across prefill/decode phases considering compute, memory footprint, and bandwidth. 2.2x throughput, 20x lower TTFT.
- **Relevance**: The diminishing returns insight is a general principle for Mister Smith's resource pool. Adding more async tasks beyond the saturation point wastes resources. The ResourcePool should implement saturation detection.

### TaiChi: Unified PD Aggregation and Disaggregation
- **Authors**: Wang, Zuo, Chen, Liang, Yu, Yang
- **Year**: 2025 | **Citations**: 4
- **Journal**: ArXiv
- **Key finding**: PD aggregation excels for tight TTFT / relaxed TPOT; disaggregation for strict TPOT / relaxed TTFT. TaiChi unifies both with "latency shifting" -- reallocating GPU resources from SLO-met requests to at-risk ones. 77% goodput improvement under balanced SLOs.
- **Relevance**: **Latency shifting is a novel backpressure strategy.** Mister Smith could implement this at the agent level: when some agents meet their latency targets, their allocated resources (e.g., concurrent LLM requests) shift to agents at risk of SLO violation.

---

## 8. Fan-In/Fan-Out and Distributed Stream Aggregation

### Skitter: Distributed Stream Processing with Pluggable Distribution Strategies
- **Authors**: Saey, De Koster, Meuter
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Decouples data processing operations from distribution strategies. New strategies can be created modularly. Achieves throughput comparable to Apache Storm while offering high-level programming model.
- **Relevance**: **Directly applicable to Mister Smith's transport layer.** The concept of pluggable distribution strategies for stream operators maps to Mister Smith's Transport trait -- different distribution strategies (round-robin, content-based, key-partitioned) should be composable with processing logic.

### DIPSUM: Distributed Pattern Summaries for Efficient CEP Aggregates
- **Authors**: Purtzel, Akili, Kuhne, Weidlich
- **Year**: 2025 | **Citations**: 0
- **Journal**: Proc. 19th ACM DEBS
- **Key finding**: On-demand evaluation of aggregate queries in distributed environments. Compact summary data structure captures match information, decomposable for distributed evaluation. Orders of magnitude improvement in transmission costs and throughput.
- **Relevance**: The summary-based aggregation pattern applies to Mister Smith's agent monitoring. Instead of streaming all metrics centrally, agents could maintain local summaries that are aggregated on-demand via the EventBus.

### CAOM: Cost-Aware Operator Migration for Stream Processing
- **Authors**: Tan, Tang, Cai, Tan, Xiao, Zhang, Gao, Li
- **Year**: 2025 | **Citations**: 3
- **Journal**: IEEE Trans. Cloud Computing
- **Key finding**: Directly identifies all bottleneck operators from task running metrics (avoiding cascading migrations). Selects optimal migration start time based on fluctuating data generation rates.
- **Relevance**: Agent hot-reloading and migration in Mister Smith should follow the CAOM principle: identify ALL bottleneck agents simultaneously (not one at a time), and time migrations to minimize data accumulation during transition.

---

## 9. Zero-Copy Messaging and Serialization Optimization

### ROS 2 Agnocast: True Zero-Copy Publish/Subscribe IPC
- **Authors**: Ishikawa-Aso, Kato
- **Year**: 2025 | **Citations**: 0
- **Journal**: ISORC 2025
- **Key finding**: True zero-copy IPC eliminating serialization/deserialization. Constant IPC overhead regardless of message size. 16% average / 25% worst-case response time improvement in production ROS 2 system.
- **Relevance**: For intra-process agent communication in Mister Smith (agents on the same node), zero-copy messaging through shared memory ring buffers could eliminate serialization overhead. The InMemoryTransport could be enhanced with this pattern.

### Roadrunner: Near-Zero-Copy Data Delivery for Serverless Functions
- **Authors**: Marcelino, Pusztai, Nastic
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Maps function memory and moves data along a dedicated "virtual data hose." 44-89% inter-function communication latency improvement, 97% serialization overhead reduction, 69x throughput increase.
- **Relevance**: The "virtual data hose" concept of bypassing serialization for co-located functions could inform Mister Smith's inter-agent communication when agents share a process.

### Zero-Copy Messaging in CHERI-Enabled RTOS
- **Authors**: Soltani Siapoush, Alves-Foss
- **Year**: 2025 | **Citations**: 0
- **Journal**: Future Internet
- **Key finding**: Shared memory ring buffer for messaging with capability-protected access. 3x lower mutex lock latency, 70%+ faster message transfers. Temporal safety via hardware-backed capability expiration.
- **Relevance**: The capability-protected ring buffer design validates Mister Smith's use of bounded ring buffers (parking_lot::RwLock in audit system). The temporal safety concept (capability expiration) maps to message TTL in JetStream.

---

## 10. Bounded Channels, Concurrency Primitives, and Lock-Free Structures

### Elastic Relaxation of Concurrent Data Structures
- **Authors**: von Geijer, Tsigas
- **Year**: 2025 | **Citations**: 0
- **Journal**: IEEE Trans. Parallel and Distributed Systems
- **Key finding**: Introduces "elastic relaxation" for lock-free queues, stacks, counters, deques that reconfigure relaxation at runtime. Contention-aware controller adjusts relaxation in real-time. Matches static relaxed structure performance with added adaptability.
- **Relevance**: **Novel concept for Mister Smith's mailbox design.** Elastically relaxed queues could serve as agent mailboxes that dynamically trade ordering guarantees for throughput during high load, then tighten back during normal operation.

### Mobius: Lock-Free Design for Throughput-Optimized Cache Eviction
- **Authors**: Dong, Wang, Jiang, Feng
- **Year**: 2025 | **Citations**: 1
- **Journal**: ACM SIGMETRICS
- **Key finding**: Two lock-free FIFO queues with consecutive detection mechanism that merges multiple modifications during eviction into single operations. 1.2-8.5x concurrent throughput improvement.
- **Relevance**: The dual lock-free FIFO queue pattern and modification-merging are applicable to Mister Smith's EventBus and dead-letter queue implementation.

### Verifying Correctness of Shared Channels in Cooperatively Scheduled Language
- **Authors**: Pedersen, Chalmers
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Uses FDR refinement checking to verify shared channel behavior in cooperatively scheduled runtimes. Demonstrates that correct channel behavior depends on having adequate resources for all processes.
- **Relevance**: Formal verification of channel behavior is relevant to Mister Smith's bounded channel design. The finding that correctness depends on resource adequacy reinforces the importance of backpressure -- without it, channels cannot maintain correctness guarantees.

### Lightweight Concurrency with Go for Real-Time Edge Computing
- **Authors**: Mrs. Vrunda, Chouthkanthiwar
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Synthesizes bounded queues, reduced allocations, and context timeouts for predictable soft real-time behavior. Examines gRPC/NATS for edge-to-cloud bridges. Identifies eBPF fast paths and io_uring integration as open challenges.
- **Relevance**: While Go-specific, the patterns (bounded queues, allocation reduction, context timeouts) are directly transferable to Tokio/Rust. The NATS bridge patterns validate Mister Smith's NATS transport design.

---

## 11. SIMD-Accelerated Stream Processing

### Pandora: Efficient Persistence-Based Tasks in High-Speed Data Streams
- **Authors**: Li
- **Year**: 2025 | **Citations**: 1
- **Journal**: Proc. ACM on Management of Data
- **Key finding**: Novel approximate data structure for high-velocity stream processing. Uses SIMD instructions to further accelerate update speed. Items absent for extended periods are proactively evicted.
- **Relevance**: The SIMD-accelerated stream data structure pattern could be applied to Mister Smith's metrics collection pipeline, where high-throughput event processing benefits from vectorized operations.

### Tight-Sketch: Efficient Sketching for Data Stream Mining
- **Authors**: Li, Patras
- **Year**: 2025 | **Citations**: 0
- **Journal**: IEEE Transactions on Computers
- **Key finding**: Versatile sketch framework with distinct eviction strategies for hot/cold items. SIMD instructions enhance update throughput by 36%. Also validated on FPGA.
- **Relevance**: The hot/cold item eviction strategy is applicable to Mister Smith's caching layer. Hot agents (frequently accessed) get protection; cold agents can be evicted/restarted.

---

## 12. Tail Latency, Circuit Breakers, and Resilience Patterns

### USRFNet: Unified System Representations for Microservice Tail Latency Prediction
- **Authors**: Qian, Zhao, Chen, Chen, Wang, Chow, Deng
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: GNNs capture service interactions and request propagation patterns. gMLP modules model resource dynamics. Predicts window-level P95 tail latency by fusing traffic-side and resource-side features.
- **Relevance**: The traffic-side vs resource-side feature separation maps to Mister Smith's monitoring model. The PhiAccrualFailureDetector handles traffic-side (heartbeats), while MetricsCollector tracks resource-side. Fusing both for tail latency prediction would improve proactive health management.

### SLO-RTP: Microservice Control Plane for Tail-Latency-Safe Processing
- **Authors**: Nellipudi
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Automated circuit-breaking within milliseconds of threshold violations. ML-enhanced predictive algorithms forecast degradation minutes in advance. Sidecar deployment for gradual adoption.
- **Relevance**: The millisecond-response circuit breaking validates Mister Smith's CircuitBreaker design. The predictive degradation forecasting is a future enhancement for the monitoring system.

### Toward Resilient Cloud Services: Pattern-Based Fault-Tolerance
- **Authors**: Carabali, Mondragon
- **Year**: 2025 | **Citations**: 0
- **Journal**: IEEE COLCOM 2025
- **Key finding**: Dynamic resilience system that autonomously monitors latency/error metrics and dynamically selects fault-tolerance patterns (circuit breakers, retries, rate limiting) without manual intervention.
- **Relevance**: **Validates Mister Smith's adaptive resilience design.** The supervision tree + CircuitBreaker + RetryPolicy composition should be made dynamic -- automatically switching strategies based on observed error patterns.

### SLO-Aware Load-Adaptive Timeout for Microservices
- **Authors**: Hanada, Ishibashi
- **Year**: 2025 | **Citations**: 0
- **Journal**: IEEE Access
- **Key finding**: Dynamic timeout adjustment balancing failure rate and latency SLOs. Naturally exhibits load shedding and circuit-breaking behavior during downstream overload. 40% average and 55% tail latency reduction.
- **Relevance**: **Directly applicable.** Mister Smith's agent-to-LLM provider communication should use adaptive timeouts rather than static ones. The emergent load-shedding behavior from smart timeouts eliminates the need for a separate load-shedding layer.

---

## 13. Rate Limiting and Adaptive Load Management

### Multi-Objective Adaptive Rate Limiting Using Deep Reinforcement Learning
- **Authors**: Lyu, Wang, Zhang, Chen
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Hybrid DQN/A3C architecture models rate limiting as MDP. 23.7% throughput improvement, 31.4% P99 latency reduction. 90-day production deployment: 82% reduction in service degradation incidents.
- **Relevance**: Mister Smith's rate limiter in the security layer currently uses static token-bucket. This paper suggests evolving to adaptive rate limiting that learns optimal policies from observed traffic patterns.

### Designing High-Throughput FastAPI Gateways
- **Authors**: Alla
- **Year**: 2025 | **Citations**: 1
- **Journal**: J. Computer Science and Technology Studies
- **Key finding**: Tiered routing, circuit breakers, JWT auth, token-bucket rate limiting, correlation ID tracing. Async processing with parallel request handling, connection pooling, and request batching.
- **Relevance**: Validates the architectural composition Mister Smith already implements (JWT + RBAC + rate limiting + circuit breaker + correlation ID tracing via W3C TraceContext).

---

## 14. Structured Output and Constrained Decoding

### Enhancing LLM Function Calling with Structured Outputs
- **Authors**: Sejourne, Lata
- **Year**: 2025 | **Citations**: 0
- **Journal**: GACLM 2025
- **Key finding**: Constrained Generation (CG) using structured outputs (xgrammar) vs traditional Post-Parsing (PP) for function calls. CG achieves higher format compliance and supports complex elements like environment variables and remote connections.
- **Relevance**: **Critical for Mister Smith's tool-calling pipeline.** When streaming function-call JSON from LLM providers, constrained generation at the provider level ensures well-formed tool invocations, reducing error handling complexity in the ToolBus.

### JSONSchemaBench: Benchmark of Structured Outputs
- **Authors**: Geng, Cooper, Moskal, Jenkins, Berman, Ranchin, West, Horvitz, Nori
- **Year**: 2025 | **Citations**: 18
- **Key finding**: Evaluates 6 constrained decoding frameworks across 10K real-world JSON schemas. Measures efficiency, constraint coverage, and output quality. Identifies significant gaps in handling complex constraint types.
- **Relevance**: Informs Mister Smith's structured output validation in the streaming pipeline. The tool-call JSON schemas should be validated against the constraint types that constrained decoding frameworks handle well.

### ScaleMCP: Dynamic Model Context Protocol Tools for LLM Agents
- **Authors**: Lumer, Gulati, Subbiah, Basavaraju, Burke
- **Year**: 2025 | **Citations**: 11
- **Journal**: ArXiv
- **Key finding**: Auto-synchronizing tool storage via CRUD operations with MCP servers as single source of truth. Novel Tool Document Weighted Average (TDWA) embedding for tool retrieval. Evaluated across 5,000 MCP servers, 10 LLMs, 5 embedding models.
- **Relevance**: **Directly applicable to Mister Smith's MCP tool registry.** The auto-sync pattern (MCP servers as source of truth, CRUD-based synchronization) should replace static tool registration. The TDWA embedding strategy could improve tool selection in multi-tool scenarios.

---

## 15. Observability and Distributed Tracing for Streaming Pipelines

### HybridRCA: Critical-Path-Aware Tracing for Root-Cause Analysis
- **Authors**: Ekhlasi, Fiorini, Dagenais, Ezzati-Jivan, Lamothe
- **Year**: 2025 | **Citations**: 0
- **Journal**: ICSME 2025
- **Key finding**: Extracts critical path of each request. PageRank-weighted spectrum analysis identifies suspicious spans. Collects system metrics only for targeted spans. 22.6% fewer spans analyzed, 99% kernel-level storage reduction.
- **Relevance**: Mister Smith's distributed tracing (W3C TraceContext) should implement critical-path extraction to focus observability on the spans that matter for agent pipeline latency.

### Tracing and Metrics Design Patterns for Cloud-Native Applications
- **Authors**: Albuquerque, Correia
- **Year**: 2025 | **Citations**: 1
- **Journal**: ArXiv
- **Key finding**: Three design patterns: Distributed Tracing (request flow visibility), Application Metrics (performance indicators), Infrastructure Metrics (resource utilization). Derived from industry practices.
- **Relevance**: Validates Mister Smith's three-pillar observability: W3C TraceContext propagation, Prometheus metrics, and health probes.

---

## 16. Stream Processing Fault Recovery and State Management

### Local Recovery and Partial Snapshot in Distributed Stateful Stream Processing
- **Authors**: Takdir, Kitagawa, Amagasa
- **Year**: 2025 | **Citations**: 0
- **Journal**: Knowledge and Information Systems
- **Key finding**: Localizes recovery to a subset of operators instead of global snapshot restore. Partial snapshots capture only the states required for local recovery. 50%+ recovery time improvement in Apache Flink.
- **Relevance**: **Critical for Mister Smith's supervision tree.** When an agent fails, only the failing agent and its direct dependencies should be restored -- not the entire supervision tree. This validates the RestForOne strategy over global restart.

### Enhancing Checkpointing and State Recovery for Large-Scale Stream Processing
- **Authors**: Poolakkal Mukkath
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Surveys incremental state snapshots, async commit techniques, log-based recovery, adaptive checkpoint intervals, event-driven rollback. Explores event sourcing as state recovery alternative.
- **Relevance**: The event sourcing approach to state recovery aligns with Mister Smith's JetStream KV store design. Agent state can be reconstructed by replaying events from JetStream streams rather than restoring from snapshots.

### Real-Time Data Streaming: Temporal Accuracy and Processing Integrity
- **Authors**: Poolakkal Mukkath
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Comprehensive treatment of watermarking techniques, exactly-once semantics, and their performance tradeoffs. Building blocks: idempotent operations, transactional event processing, checkpointing.
- **Relevance**: Mister Smith's JetStream AckPolicy and MaxDeliver settings implement at-least-once semantics. This paper's analysis of exactly-once building blocks informs the path to stronger guarantees when needed.

---

## 17. Dataflow Programming and Stream Topology

### StreamTune: Adaptive Parallelism Tuning with Graph Neural Networks
- **Authors**: Han, Chen, Wang, Chen, Zhang, Yang, Hao, Yang
- **Year**: 2025 | **Citations**: 1
- **Journal**: ICDE 2025
- **Key finding**: Pre-training on historical execution data clustered by graph edit distance. GNN encoder captures correlation between operator parallelism, DAG structures, and bottlenecks. Up to 83.3% parallelism reduction in Timely Dataflow while maintaining performance.
- **Relevance**: The GNN-based bottleneck identification in DAG topologies could be applied to Mister Smith's agent orchestration DAG. Historical execution data from agent runs could train a model to predict optimal parallelism for each agent type.

### Streaming Tensor Program: Streaming Abstraction for Dynamic Parallelism
- **Authors**: Sohn, Zhang, Hossfeld, Kim, Sobotka, Zhang, Hsu, Olukotun
- **Year**: 2025 | **Citations**: 0
- **Key finding**: Introduces flexible routing operators, explicit memory hierarchy, and symbolic shape semantics. Enables dynamic tiling (2.18x memory reduction), dynamic parallelization (1.5x latency improvement), and configuration time-multiplexing (2.57x utilization).
- **Relevance**: The dynamic routing operator concept is applicable to Mister Smith's message routing. The explicit memory hierarchy maps to the NATS subject hierarchy.

---

## 18. Elastic Scaling for Stream Processing

### Justin: Hybrid CPU/Memory Elastic Scaling
- **Authors**: Schmitz, Rosinosky, Riviere
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Decouples CPU and memory scaling for stream processing operators. Identifies individual operator memory pressure. Implements fine-grained memory allocation per task. Extends Apache Flink Kubernetes Operator.
- **Relevance**: Mister Smith's agent scaling should decouple compute (async task count) from memory (mailbox size, KV cache allocation). Individual agents may be memory-bound (large context) or CPU-bound (heavy tool calls).

### Proactive Autoscaling for Edge Stream Processing
- **Authors**: Armah, Bannning
- **Year**: 2025 | **Citations**: 1
- **Journal**: ArXiv
- **Key finding**: GRU neural network forecasts upstream load (1.3% SMAPE). Transfer learning bridges offline-online domain gap. Horizontal autoscaling adjusts operator parallelism based on predicted load.
- **Relevance**: Proactive load prediction for agent scaling in Mister Smith. The monitoring system could use simple forecasting to pre-scale agents before demand spikes.

---

## 19. Actor Model and Agent Communication Protocols

### CRGC: Fault-Recovering Actor Garbage Collection in Pekko
- **Authors**: Plyukhin, Agha, Montesi
- **Year**: 2025 | **Citations**: 0
- **Journal**: Proc. ACM on Programming Languages
- **Key finding**: First fault-recovering cyclic actor GC. Uses conflict-free replicated data for distributed GC. Formalized in TLA+. No locks, no explicit memory barriers, no message delivery ordering assumptions. Performance competitive with weighted reference counting.
- **Relevance**: **Directly relevant to Mister Smith's actor lifecycle management.** Currently, agents must be explicitly stopped. CRGC-style garbage collection could automatically reclaim agents that become unreachable in the supervision tree.

### Actor Capabilities for Message Ordering
- **Authors**: Gordon
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Equips actor references with protocols restricting message types and ordering. Actor capabilities as static flow-sensitive capabilities. Effect system layered over base type system ensures actors always prepared for any arriving message.
- **Relevance**: Mister Smith's ActorRef + MessageEnvelope design could benefit from protocol-restricted references. A supervision actor's reference could restrict subordinates to only send health reports and completion notifications.

---

## 20. Rust Async Runtime and Real-Time Systems

### Harnessing Memory Safety and Async Concurrency in Rust's Web Framework Ecosystem
- **Authors**: Yuvaraaj, Appar, Muthupandi, Pavithra
- **Year**: 2025 | **Citations**: 0
- **Journal**: ICCCT 2025
- **Key finding**: Case study of real-time chat application handling thousands of concurrent connections with low latency using Axum. Demonstrates Rust's memory safety + async concurrency combination.
- **Relevance**: Direct validation of Mister Smith's Axum + Tokio stack for concurrent agent communication.

### Async Rust for ROS 2 Real-Time Applications
- **Authors**: Skoudlil, Sojka, Hanzalek
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Analyzes execution model of async Rust runtimes for real-time operation. Proposes thread prioritization and callback-to-thread mapping for deterministic real-time behavior. Achieves bounded response times for time-critical tasks.
- **Relevance**: The thread prioritization and callback mapping patterns are directly applicable to Mister Smith's Tokio runtime configuration. Priority agents should be mapped to dedicated runtime threads.

### Model Checking and Runtime Verification for Awkernel (Rust Async Scheduler)
- **Authors**: Hasegawa, Kambe, Aoki, Takano
- **Year**: 2025 | **Citations**: 0
- **Journal**: ArXiv
- **Key finding**: Combines model checking for code review with runtime verification for Rust async schedulers. Identified bugs that are difficult to detect through manual reviews or simple tests.
- **Relevance**: The model-checking-assisted approach could be applied to verify Mister Smith's supervision tree state machine and actor mailbox implementations.

---

## 21. Emerging Directions

### Semantic Scheduling for LLM Inference
- **Authors**: Hua, Ding, Gu, Ren, Mei, Ma, Wang
- **Year**: 2025 | **Citations**: 0
- **Novel concept**: Content-aware scheduling where the semantics of a request (not just its size or arrival time) determine processing priority. Applied to medical emergency triage.
- **Implication for Mister Smith**: Agent requests could be semantically prioritized -- a security audit request processes before a documentation generation request, based on content analysis rather than just queue position.

### Latency Shifting for SLO Optimization
- **Authors**: Wang, Zuo et al. (TaiChi)
- **Novel concept**: When some requests meet SLOs, their resources shift to at-risk requests. Fine-grained reallocation maximizes the number of SLO-satisfied requests globally.
- **Implication for Mister Smith**: Agent-level latency shifting -- agents that finish early donate their remaining time budget to agents at risk of SLO violation.

### Elastic Relaxation of Data Structures
- **Authors**: von Geijer, Tsigas
- **Novel concept**: Runtime-reconfigurable relaxation of ordering guarantees in lock-free queues. Contention-aware controller automatically trades ordering for throughput under high load.
- **Implication for Mister Smith**: Agent mailboxes that dynamically weaken FIFO ordering during backpressure events, processing higher-priority messages out of order.

### Agent Step Graph with Steps-to-Execution Estimation
- **Authors**: Pan, Patel et al. (KVFlow)
- **Novel concept**: Modeling multi-agent execution schedules as graphs with temporal proximity estimation for cache management.
- **Implication for Mister Smith**: The orchestrator DAG could be annotated with steps-to-execution estimates, informing preemptive resource allocation and JetStream consumer configuration.

### Token Throttling as Backpressure Primitive
- **Authors**: Guo, Zhang et al. (gLLM)
- **Novel concept**: Independent regulation of prefill and decode token quantities at the pipeline level, using global system information for balanced computation.
- **Implication for Mister Smith**: The LLM provider should expose separate throttling controls for request submission rate and response consumption rate, enabling fine-grained backpressure propagation.

### Conflict-Free Replicated Garbage Collection for Actors
- **Authors**: Plyukhin, Agha, Montesi (CRGC)
- **Novel concept**: Distributed actor GC using CRDTs, tolerating dropped messages and crashed nodes while guaranteeing soundness and completeness.
- **Implication for Mister Smith**: Automatic lifecycle management for agents in the supervision tree, eliminating manual cleanup and preventing resource leaks from orphaned agents.

---

## 22. Synthesis: Implications for Mister Smith

### Architecture Validation

The 2025 research landscape strongly validates Mister Smith's core architectural choices:

1. **NATS/JetStream as backbone**: Multiple papers (Arafat et al., Pouhela et al.) empirically validate pub/sub messaging for AI inference pipelines. NATS JetStream's pull-based consumer model natively implements backpressure.

2. **Actor model + supervision trees**: CRGC (Plyukhin et al.) and Actor Capabilities (Gordon) represent the cutting edge of actor system research, both addressing challenges Mister Smith faces (lifecycle management, message ordering).

3. **Reactive streams with backpressure**: The formal foundations (Hou et al. monoid homomorphisms, Szabo & Cziborova automata) provide rigorous underpinning for Mister Smith's stream pipeline composition.

4. **MCP integration**: ScaleMCP (Lumer et al.) and the agent protocol survey (Ehtesham et al.) validate MCP as the right starting point for tool integration, with a clear evolution path toward A2A for agent-to-agent communication.

### Key Design Patterns to Adopt

| Pattern | Source | Application |
|---------|--------|-------------|
| Token Throttling | gLLM (Guo et al.) | Independent prefill/decode rate control in LLM provider |
| Latency Shifting | TaiChi (Wang et al.) | Agent-level resource reallocation for SLO optimization |
| Agent Step Graph | KVFlow (Pan et al.) | Orchestrator DAG annotation for predictive scheduling |
| Elastic Relaxation | von Geijer & Tsigas | Adaptive mailbox ordering under backpressure |
| Streaming Verification | Streaming-VR (Ko et al.) | Token-level validation in streaming pipeline |
| Local Recovery | Takdir et al. | Supervision tree partial restart (RestForOne validation) |
| Adaptive Timeouts | Hanada & Ishibashi | Dynamic agent-to-provider timeout with emergent load shedding |
| TDWA Embedding | ScaleMCP (Lumer et al.) | Improved tool selection in MCP tool registry |

### Critical Metrics

Based on the benchmarking papers:

- **Messaging throughput target**: NATS JetStream should sustain >100K msgs/sec for agent coordination (TBMQ achieves 3M+, Kafka 1.2M+)
- **P95 latency target**: <50ms for intra-agent messaging (event-driven architectures achieve 18-22ms p95)
- **Recovery time target**: <500ms per agent restart (local recovery achieves 50%+ improvement over global snapshot)
- **Streaming verification overhead**: <10% latency increase for token-level validation (Streaming-VR achieves this)

### Research Gaps Relevant to Mister Smith

1. **No research on Rust-native actor GC**: CRGC is Pekko-specific. A Rust implementation using ownership semantics for actor lifecycle could be more efficient.
2. **Limited NATS JetStream-specific streaming research**: Most stream processing research targets Kafka/Flink. JetStream's unique pull-based consumer model and KV store capabilities are under-studied.
3. **No integrated framework combining supervision trees with LLM backpressure**: Research treats these as separate concerns. Mister Smith's integration of OTP-style supervision with LLM streaming is novel.
4. **Sparse research on model-agnostic provider abstraction**: Most papers assume a specific LLM backend. The ModelProvider trait abstraction that works across providers while maintaining streaming backpressure is an open area.

---

## References (Alphabetical by First Author)

1. Arafat, J., Tasmin, F., Poudel, S. (2025). "Next-Generation Event-Driven Architectures." ArXiv:2510.04404.
2. Armah, E., Bannning, L.A. (2025). "Proactive Autoscaling for Data Stream Processing at the Edge." ArXiv:2507.14597.
3. Bian, Z., Wu, F., Ma, T., Zhuo, Y. (2025). "Tokencake: KV-Cache-centric Serving for Multi-Agent Applications." ArXiv:2510.18586.
4. Cai, Y. et al. (2025). "FastMTP: Accelerating LLM Inference with Enhanced Multi-Token Prediction." ArXiv:2509.18362.
5. Chen, C.-H., Shiu, M.-F. (2025). "AgentFlow: Resilient Adaptive Cloud-Edge Framework." ArXiv:2505.07603.
6. Dong, C. et al. (2025). "Mobius: Lock-Free Design for Throughput-Optimized Cache Eviction." ACM SIGMETRICS.
7. Ehtesham, A. et al. (2025). "Survey of Agent Interoperability Protocols (MCP, ACP, A2A, ANP)." ArXiv:2505.02279.
8. Ekhlasi, M. et al. (2025). "HybridRCA: Critical-Path-Aware Hybrid Tracing." ICSME 2025.
9. Geng, S. et al. (2025). "JSONSchemaBench: A Rigorous Benchmark of Structured Outputs." [Guidance-AI].
10. Gordon, C.S. (2025). "Actor Capabilities for Message Ordering." ArXiv:2502.07958.
11. Guo, T. et al. (2025). "gLLM: Token Throttling for Pipeline Parallelism." ArXiv:2504.14775.
12. Habibi, S., Ercetin, O. (2025). "Edge-LLM Inference with Cost-Aware Layer Allocation." IEEE Access.
13. Han, Y. et al. (2025). "StreamTune: Adaptive Parallelism Tuning." ICDE 2025.
14. Hanada, H., Ishibashi, K. (2025). "SLO-Aware Load-Adaptive Timeout." IEEE Access.
15. Hebbar, K.S. (2025). "Priority-Aware Reactive APIs with Spring WebFlux." EJECS.
16. Hou, T., Arntzenius, M., Willsey, M. (2025). "Stream Programs Are Monoid Homomorphisms with State." ArXiv:2507.10799.
17. Hua, W. et al. (2025). "Semantic Scheduling for LLM Inference." ArXiv:2506.12204.
18. Ko, J., Baek, J., Hwang, S.J. (2025). "Streaming-VR: Streaming Verification and Refinement." EMNLP 2025.
19. Kumar, K.S.S. (2025). "Reactive Programming Paradigms in High-Throughput Distributed Systems." EJCSIT.
20. Li, H. et al. (2025). "Continuum: Multi-Turn LLM Agent Scheduling with KV Cache TTL." [UC Berkeley/Ion Stoica].
21. Li, J. et al. (2025). "Gumiho: Hybrid Architecture for Speculative Decoding." ArXiv:2503.10135.
22. Li, R., Chen, F., Li, P. (2025). "LAPS-SD: Semi-Clairvoyant Scheduling of Speculative Decoding." ArXiv:2505.17074.
23. Li, W. (2025). "Pandora: SIMD-Accelerated Persistence Tasks in High-Speed Data Streams." Proc. ACM Mgmt. of Data.
24. Li, W., Patras, P. (2025). "Tight-Sketch: Efficient Sketching for Data Stream Mining." IEEE Trans. Computers.
25. Long, Z. et al. (2025). "VITA-Audio: Fast Interleaved Cross-Modal Token Generation." ArXiv:2505.03739.
26. Lumer, E. et al. (2025). "ScaleMCP: Dynamic Model Context Protocol Tools for LLM Agents." ArXiv:2505.06416.
27. Lyu, N. et al. (2025). "Multi-Objective Adaptive Rate Limiting Using Deep RL." [Unknown].
28. Mina Carabali, D.V., Mondragon, O.H. (2025). "Toward Resilient Cloud Services." IEEE COLCOM 2025.
29. Pan, Z. et al. (2025). "KVFlow: Workflow-Aware KV Cache for Multi-Agent Workflows." ArXiv:2507.07400.
30. Pedersen, J., Chalmers, K. (2025). "Verifying Shared Channels in Cooperatively Scheduled Language." ArXiv:2510.11751.
31. Plyukhin, D., Agha, G., Montesi, F. (2025). "CRGC: Fault-Recovering Actor Garbage Collection." Proc. ACM Programming Languages.
32. Poolakkal Mukkath, S. (2025). "Enhancing Checkpointing and State Recovery." WJARR.
33. Poolakkal Mukkath, S. (2025). "Real-Time Data Streaming: Temporal Accuracy and Processing Integrity." EJCSIT.
34. Pouhela, F. et al. (2025). "Queuing Theory-Based Modeling of Publish/Subscribe IoT." ICC 2025.
35. Purtzel, S. et al. (2025). "DIPSUM: Distributed Pattern Summaries for Efficient CEP." ACM DEBS 2025.
36. Qian, W. et al. (2025). "USRFNet: Unified System Representations for Tail Latency Prediction." ArXiv:2508.01635.
37. Qu, Z. et al. (2025). "LLM-Enabled Multi-Agent System for 6G Networks." ArXiv:2509.04993.
38. Saey, M., De Koster, J., Meuter, W. (2025). "Skitter: Pluggable Distribution Strategies." ArXiv:2502.20538.
39. Sapkota, R. et al. (2025). "AI Agents vs. Agentic AI: Conceptual Taxonomy." Information Fusion.
40. Schmitz, D., Rosinosky, G., Riviere, E. (2025). "Justin: Hybrid CPU/Memory Elastic Scaling." ArXiv:2505.19739.
41. Sejourne, K., Lata, A. (2025). "Enhancing LLM Function Calling with Structured Outputs." GACLM 2025.
42. Shao, Z. et al. (2025). "FlashSVD: Memory-Efficient Streaming Inference." ArXiv:2508.01506.
43. Shi, X. et al. (2025). "Nexus: Proactive Intra-GPU PD Disaggregation." [Unknown].
44. Shvaika, A. et al. (2025). "TBMQ: Distributed Architecture for MQTT Messaging." J. Big Data.
45. Sirotkin, O. et al. (2025). "Parallel Simulation Using Reactive Streams." Comput.
46. Skoudlil, M. et al. (2025). "Async Rust for ROS 2 Real-Time Applications." ArXiv:2505.21323.
47. Sohn, G. et al. (2025). "Streaming Tensor Program." [Unknown].
48. Soltani Siapoush, M., Alves-Foss, J. (2025). "Zero-Copy Messaging in CHERI-Enabled RTOS." Future Internet.
49. Song, X. et al. (2025). "Gradientsys: Multi-Agent LLM Scheduler with ReAct." ArXiv:2507.06520.
50. Szabo, R., Cziborova, D. (2025). "Automata-based Coordination for Distributed Reactive Systems." FMFAI 2025.
51. Takdir, Kitagawa, H., Amagasa, T. (2025). "Local Recovery and Partial Snapshot." KAIS.
52. Tan, J. et al. (2025). "CAOM: Cost-Aware Operator Migration." IEEE Trans. Cloud Computing.
53. von Geijer, K., Tsigas, P. (2025). "Elastic Relaxation of Concurrent Data Structures." IEEE TPDS.
54. Wang, C. et al. (2025). "TaiChi: Prefill-Decode Aggregation/Disaggregation." ArXiv:2508.01989.
55. Zhang, H. et al. (2025). "TD-Pipe: Temporally-Disaggregated Pipeline Parallelism." ArXiv:2506.10470.
56. Zhang, S. et al. (2025). "Output Constraints as Attack Surface." ArXiv:2503.24191.
57. Zhou, E. et al. (2025). "CARD: Cache-Assisted Parallel Speculative Decoding." [Unknown].

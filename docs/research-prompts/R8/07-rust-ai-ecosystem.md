---
version: R8
created: 2026-03-22
type: prompt
tier: 1
timeline: last 2 months (late January 2026 — present)
---

# Deep Research Prompt: Rust Ecosystem for AI, ML & Agent Workloads

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to define the standard that the agent framework market will converge toward.

The system is a 20-crate Rust workspace (MSRV 1.88.0, edition 2021) with a mature dependency stack spanning messaging, HTTP, gRPC, MCP, persistence, security, and observability. The Rust ecosystem is not just the implementation language — it is a strategic advantage. GraphBit demonstrated 68x CPU and 140x memory efficiency of Rust over Python for agent graph workloads. WASM/WASI has been validated for tool sandboxing. The research question is: what has changed in the Rust AI/ML/agent ecosystem that should influence Mister Smith's dependency decisions, architecture, or build pipeline?

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by existing agent frameworks. Benchmark them. Learn from them. Then exceed them. The Python agent ecosystem (LangChain, CrewAI, AutoGen) is the dominant paradigm; the opportunity is Rust-native alternatives that eliminate the Python overhead entirely while matching or exceeding capability.

Incremental imitation is failure. Favor well-reasoned designs that create real advantage.

## Research Objective

Survey everything published in the last ~2 months (late January 2026 to present) on the Rust ecosystem for AI, ML inference, agent orchestration, async runtimes, WASM sandboxing, embedding/vector search, and compiler/language evolution. The goal is to discover what has changed since our last deep research round (early March 2026) and identify crates, patterns, language features, or ecosystem shifts that should influence Mister Smith's next iteration.

This is an open-ended research task. Go beyond the dimensions listed below if you discover promising leads outside them.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The following are established facts about Mister Smith's Rust stack and the known Rust AI ecosystem. Treat these as known. Only surface new work on these topics if it significantly contradicts, extends, or supersedes them.

**Core Dependency Stack**: Tokio 1.49.0 (full feature set: rt-multi-thread, io, net, time, sync, fs, process, signal), async-nats 0.46.0 (JetStream, KV, object-store, service features), Axum 0.8.8, Tonic 0.14.x + Prost 0.14.x, rmcp 1.1.0 (MCP client/server, streamable-HTTP), sqlx 0.8.6 (PostgreSQL, runtime-tokio-rustls), jsonwebtoken 10.x (aws_lc_rs backend), rustls 0.23 (TLS 1.3, mTLS), opentelemetry 0.31.0 + tracing 0.1.44 + metrics-exporter-prometheus 0.18.1, serde/serde_json, rmp-serde (MessagePack), thiserror 1.x (deliberate stay on 1.x until ecosystem converges), clap 4.x.

**Version Decisions**: async-nats migrated from 0.37 to 0.46 (9 minor versions, publish backpressure added, feature-gated modules). thiserror stays on 1.x deliberately. jsonwebtoken upgraded to 10.x with trait-based crypto backend. MSRV 1.88.0 driven by async-nats 0.46.0 requirement. Rust edition 2024 available but not adopted.

**Rust AI/ML Crates on Radar**: candle (Hugging Face inference), burn (deep learning framework), ort (ONNX Runtime bindings), llama-cpp-rs, whisper-rs, tokenizers (Hugging Face). These are tracked but none are direct dependencies today — Mister Smith calls external LLM APIs rather than running inference locally.

**WASM/WASI**: Validated for tool sandboxing. wasmtime is the primary runtime under consideration. WASI preview 2 progress tracked. Component model developments relevant for agent tool isolation.

**Performance Baseline**: GraphBit demonstrated 68x CPU / 140x memory advantage of Rust over Python for agent graph workloads. This is the known efficiency case for Rust-native agent infrastructure.

**Infrastructure**: nats-server >= v2.11.1 for CVE-2025-30215 mitigation. PostgreSQL 15+ for relational store, JetStream KV for distributed ephemeral state. Docker Compose local stack, Kubernetes deploy artifacts.

## Research Dimensions

### 1. Rust AI/ML Inference Crates and Frameworks

- What major releases or new entrants have appeared in Rust ML inference (candle, burn, ort, or entirely new crates)?
- Have there been new Rust bindings to inference engines (TensorRT-LLM, vLLM, SGLang) that could enable local inference from Rust?
- Are there new Rust crates for specific ML tasks relevant to agent workloads — embedding generation, classification, reward modeling, or structured output parsing?
- What is the current state of Rust-native transformer inference performance versus Python-based alternatives (benchmarks, not claims)?
- Have there been new developments in Rust crates for quantized model inference (GPTQ, AWQ, GGUF) that lower the barrier to local SLM execution?

### 2. async-nats and NATS Rust Ecosystem Evolution

- Have there been new async-nats releases since 0.46.0 with features, breaking changes, or performance improvements?
- What nats-server releases have shipped — security patches, new capabilities (priority queues, auth callout changes, new JetStream features)?
- Are there new NATS ecosystem tools, client libraries, or community projects relevant to agent messaging patterns?
- Have there been production reports or benchmarks of async-nats at scale that validate or challenge Mister Smith's messaging architecture?
- Are there new NATS patterns (request-many, micro framework extensions, service mesh integration) that Mister Smith should adopt?

### 3. Tokio Ecosystem and Async Runtime Advances

- What new Tokio releases, runtime improvements, or scheduler changes have shipped?
- Have there been updates to tower, tower-http, hyper, axum, or tonic that affect Mister Smith's HTTP/gRPC stack?
- Are there new async primitives, structured concurrency proposals, or task scheduling mechanisms in the Tokio ecosystem?
- What progress has been made on Tokio Console, runtime diagnostics, or async debugging tools?
- Are there competing async runtimes (smol, glommio, monoio) with developments that challenge Tokio's position for agent workloads?

### 4. WASM/WASI for Agent Tool Sandboxing

- What new wasmtime or wasmer releases have shipped with capabilities relevant to agent tool isolation?
- What is the current state of WASI preview 2 and the component model — are they stable enough for production sandboxing?
- Are there new Rust-to-WASM compilation toolchains, component model tools, or developer experience improvements?
- Has anyone deployed WASM sandboxing for LLM tool execution in production — what were the performance and security characteristics?
- Are there new capability-based security models in WASM runtimes that map well to Mister Smith's security architecture (JWT/RBAC, capability tokens)?

### 5. Rust Embedding and Vector Search Libraries

- What new or updated Rust crates exist for vector similarity search, ANN indexing, or embedding storage?
- Are there new Rust-native alternatives to FAISS, or updated Qdrant/Milvus Rust clients?
- Have there been Rust crates for embedding generation (running embedding models locally) that avoid the Python dependency?
- What is the performance state of Rust vector search libraries versus their C++/Python counterparts?
- Are there new approaches to integrating vector search with NATS JetStream KV or actor-based architectures?

### 6. Rust Compiler and Language Features for Agent Workloads

- What Rust compiler releases have shipped since 1.88.0 — are there new stable features that affect agent workload patterns?
- What is the current state of async traits (RPITIT), async closures, and other async ergonomic improvements?
- Are there new SIMD, portable-simd, or auto-vectorization improvements relevant to ML inference or data processing?
- What progress has been made on Rust edition 2024 adoption — are key dependencies adopting it, and should Mister Smith migrate?
- Are there new cargo features, workspace improvements, or build system changes that affect a 20-crate workspace?

### 7. Rust-Native Alternatives to Python Agent Tooling

- Are there new Rust crates for agent evaluation, benchmarking, or testing (alternatives to Python-based eval frameworks)?
- Have there been Rust-native tracing, observability, or debugging tools specifically designed for LLM/agent workloads?
- Are there new Rust orchestration frameworks or agent libraries that compete with or complement Mister Smith's architecture?
- What new Rust crates exist for structured output parsing, JSON schema validation, or constrained decoding integration?
- Have there been developments in Rust-native MCP implementations beyond rmcp — new servers, tools, or protocol extensions?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations (authors, year, venue, DOI/URL if available)
2. **Key techniques** — the specific crates, features, patterns, or architectures discovered
3. **Applicability to Rust + NATS** — how well does each finding integrate with Mister Smith's 20-crate workspace and async-nats messaging?
4. **Delta from baseline** — what is genuinely NEW versus what we already know?
5. **Implementation complexity** — rough assessment of effort: dependency bump, new integration, or significant refactor?
6. **Expected impact** — what improvement does this offer over the current Mister Smith stack (performance, capability, security, developer experience)?

## Synthesis

After completing all dimensions, provide a synthesis that:
- Ranks the top 5 findings by strategic value for Mister Smith's Rust stack
- Identifies which current dependency decisions should be revisited (version bumps, replacements, new additions)
- Recommends specific next actions (upgrade, adopt, prototype, benchmark, monitor)
- Notes any dimension that yielded thin results (say so rather than padding)
- Flags any ecosystem risks (deprecations, unmaintained crates, breaking changes on the horizon)

## Research Methodology

1. Search broadly across the last ~2 months (late January 2026 to present). Include crates.io releases, GitHub repositories, Rust blog posts, This Week in Rust, arXiv preprints, conference talks, and community discussions.
2. Follow promising leads with targeted deep dives — do not stop at the first result
3. Look beyond the Rust AI niche into the broader Rust ecosystem for patterns transferable to agent workloads (networking, databases, compilers, game engines)
4. For each crate or feature, assess maturity: production-ready, beta, experimental, or proof-of-concept
5. Be skeptical of README benchmarks — look for independent benchmarks, production reports, and real-world usage
6. If a dimension yields thin results, say so rather than padding with speculation
7. Cross-reference against the baseline above — only surface work that genuinely extends what we know

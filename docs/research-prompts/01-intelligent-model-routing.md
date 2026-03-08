# Deep Research Prompt: Intelligent Model Routing for Agent Frameworks

## Directive Context

Mister Smith is a Rust-based multi-agent orchestration framework built on NATS/JetStream messaging, OTP-style supervision trees, and an actor-based architecture. It is model-agnostic -- it works with any LLM.

Phase 9 adds LLM provider integration. A `ModelProvider` trait abstracts completion, streaming, embeddings, and tool calling across providers. The framework needs a `ModelRouter` layer above individual providers that selects, routes, and manages model selection intelligently.

No competing agent framework (OpenAI Agents SDK, Google ADK, LangChain, CrewAI, AutoGen, Claude SDK) implements cost-aware, capability-aware model routing at the framework level. They all delegate to external gateways (LiteLLM, OpenRouter, Bifrost). Building this natively in Rust with NATS integration is a primary differentiator.

## Research Objective

Discover the most innovative, effective, and production-viable techniques for intelligent LLM model routing. Go beyond what existing agent frameworks do. Look at adjacent fields -- ad-tech bidding, CDN routing, trading systems, telecom switching, load balancing algorithms -- for transferable patterns that the LLM ecosystem has not yet adopted.

This is an open-ended research task. The goal is to map the landscape, identify what works, and surface techniques worth implementing. Do not limit yourself to the dimensions listed below if you discover promising leads outside them.

## Research Dimensions

### 1. Cascading / Speculative Inference

Investigate cascading inference: starting with a cheap/fast model, evaluating confidence in the result, and escalating to an expensive model only when needed.

Key questions to answer through research:
- What confidence metrics are used to decide when to escalate? How reliable are they?
- What are the real-world latency tradeoffs? Is cascading viable for interactive use cases?
- What production deployments exist, and what results do they report?
- What does FrugalGPT and similar cost-optimization research contribute here?

### 2. Mixture-of-Agents / Model Composition

Investigate approaches where multiple models collaborate on a single response, rather than routing to a single model.

Key questions:
- What is the current state of Mixture-of-Agents (MoA) approaches? Together AI published foundational work here -- what were their findings?
- Are there routing patterns that compose multiple models for a single request (e.g., draft + refine, generate + verify)?
- What are the cost/quality tradeoffs of multi-model composition vs. single-model routing?

### 3. Learned Routing / Query-Complexity Classification

Investigate how routing decisions can be informed by analyzing the incoming request before selecting a model.

Key questions:
- What is the state of the art in lightweight query-complexity estimation?
- RouteLLM from lm-sys uses a trained classifier for routing -- what architecture, training data, and latency does it involve?
- Are there approaches that work at microsecond latency without requiring ML inference for the routing decision itself?
- What simple heuristics (token count, tool presence, system prompt analysis) provide effective routing signals?
- What does the academic literature say about cross-attention routing, causal LLM routing, or training-free routers?

### 4. Market-Based / Auction Routing

Investigate whether auction-based or market-based mechanisms can be applied to LLM model selection.

Key questions:
- Has anyone applied real-time bidding concepts (from ad-tech) to LLM routing?
- In CDN routing, requests are routed based on proximity, load, and content availability. What transfers to model selection?
- Could providers "bid" on requests based on current capacity, estimated quality, and price?
- Are there game-theoretic or economic models for multi-provider allocation?

### 5. Health-Aware Routing with Circuit Breakers

Investigate how production LLM gateways handle provider health and failover.

Key questions:
- How do Bifrost, Kong AI Gateway, Azure AI Gateway, and similar systems track provider health?
- What health signals matter most? Latency percentiles? Error rates? Rate limit proximity? Token quota remaining?
- How do they implement cooldown, recovery, and circuit breaker patterns?
- How does health-aware routing integrate with supervision trees (Erlang/OTP-style) for automatic failover?

### 6. Budget-Aware Routing

Investigate how spending limits and cost optimization are enforced in multi-agent LLM systems.

Key questions:
- What patterns exist for per-agent, per-team, or per-request budget enforcement?
- How do LiteLLM, OpenRouter, and enterprise gateways track and enforce spending?
- Can routing decisions dynamically select cheaper models as budget depletes?
- How do organizations balance cost constraints against quality requirements?

### 7. NATS-Native Routing Patterns

Investigate how message bus architectures can enable model routing, drawing from trading systems, telecom switches, and real-time messaging infrastructure.

Key questions:
- Could NATS subject-based routing (e.g., `llm.route.{capability}.{priority}`) provide natural model selection?
- Could NATS queue groups provide natural load balancing across multiple provider instances?
- Could JetStream KV watches enable runtime routing policy changes without restart?
- What is the latency overhead of NATS request-reply for synchronous LLM routing vs. direct HTTP calls?
- What patterns from high-frequency trading or telecom switching transfer to this problem?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** -- what exists today, with specific citations (papers, repositories, products, blog posts)
2. **Key techniques** -- the specific algorithms, architectures, or patterns discovered
3. **Applicability to Rust + NATS** -- how well does each technique transfer to a Rust actor system with NATS messaging?
4. **Implementation complexity** -- rough assessment of effort and prerequisites
5. **Expected impact** -- what improvement does this technique offer over naive round-robin or static routing?

## Synthesis

After completing all dimensions, provide a synthesis section that recommends which combination of techniques is most viable for a Rust-based agent framework, considering:

- **Microsecond-level routing decision latency** (Rust's performance advantage over Python/JS frameworks)
- **NATS-native distribution** (unique infrastructure not available to competing frameworks)
- **Supervision tree integration** (automatic failover and recovery through OTP-style patterns)
- **Production viability** (validated techniques over academic novelty)
- **Incremental adoption** (what can ship first vs. what requires more research)

The synthesis should be the research agent's own conclusion based on evidence gathered, not a restatement of the dimensions above.

## Research Methodology

1. Start with broad searches to map the landscape before diving deep into any single dimension
2. Follow promising leads with targeted deep dives -- do not stop at the first result
3. Look beyond agent frameworks into adjacent fields (trading, telecom, CDN, ad-tech) for transferable patterns
4. Prioritize recent sources (2025-2026) but include foundational work if still relevant
5. For each technique discovered, assess whether it has been validated in production or is purely academic
6. Be skeptical of marketing claims -- look for benchmarks, papers, and real-world results
7. If a dimension yields thin results, say so rather than padding with speculation

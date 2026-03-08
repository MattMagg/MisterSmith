---
version: R3
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x (3 reports) → Synthesized
round: 3 (Triple Synthesis)
---

# Streaming Architecture for Mister Smith: Comprehensive Research Synthesis

## Executive Summary

This report synthesizes three independent research investigations into streaming architecture for the Mister Smith framework (Rust + Tokio + NATS JetStream). The convergence across all three reports is striking: despite different analytical approaches, they arrive at the same core architectural recommendations with high confidence. The synthesis below merges all unique insights, preserves all technical details and benchmarks, and highlights where reports agree (high-confidence findings) or diverge (alternative perspectives).

**High-Confidence Findings (all three reports converge independently):**

- **Typed event streams over raw SSE chunking**: All three reports independently conclude that streaming must be modeled as a typed `ModelEvent` enum (with start/delta/end lifecycle, tool-call semantics, and observability events), not raw text tokens. This conclusion is independently validated by OpenAI Responses API, Vercel AI SDK, Microsoft Agent Framework, and OpenAI Agents SDK designs. [A10, B20, B21, C6, C13]
- **Stream finalizers as first-class components**: All three reports identify the Microsoft Agent Framework's `ResponseStream` / `get_final_response()` pattern as the canonical approach for deterministic stream closure, and recommend implementing it as a supervised Rust actor. [A12, A13, B1, B2, C-turn10, C-turn11]
- **Pull-based NATS JetStream consumers for distributed backpressure**: All three reports recommend JetStream pull consumers with `MaxAckPending` as the distributed flow-control mechanism, superior to push consumers for token-heavy LLM pipelines. [A1, B8, B10, C5]
- **Actor-per-stream with OTP-style supervision**: All three reports recommend wrapping each LLM stream in a supervised actor (using Erlang-inspired one-for-one restart strategies) to isolate failures and manage lifecycle. [A4, A5, B17, B18, B19, C-turn19]
- **Bounded Tokio channels for in-process backpressure**: All three reports identify `tokio::sync::mpsc` bounded channels as the foundational in-process backpressure primitive, with `OwnedPermit` for cooperative flow control. [A22, A23, B9, C3]

**Key Actionable Insights:**

1. **SIMD-accelerated incremental parsing** enables sub-millisecond validation of partial tool-call arguments mid-stream, failing fast on schema hallucinations before the stream completes. [A2, A3]
2. **Proxy buffering destroys SSE latency** -- NGINX and Cloudflare buffer SSE responses up to 100KB by default, requiring explicit `X-Accel-Buffering: no` and `proxy_buffering off` headers. [A6, A7]
3. **`StreamMap` outperforms `SelectAll`** for dynamic fan-in of concurrent sub-agent streams, providing O(1) removal and safe pinning. [A8, A9]
4. **Tail latency amplification** in multi-agent pipelines means a 99th-percentile slowdown in a single tool-call stream can stall the entire aggregated response; `buffer_unordered` with strict timeouts is required. [A11]
5. **Backpressure should be a policy matrix, not a boolean** -- different event classes (tool-call boundaries vs. text deltas) require different backpressure strategies (lossless vs. lossy/coalescible). [C-turn17]
6. **WebSocket mode for provider connections** can reduce end-to-end latency by ~40% in tool-heavy agentic loops (20+ tool calls), per OpenAI's own measurements. [C8]
7. **Dual-stream design** -- emit a lossless semantic event stream for correctness/replay and a best-effort UI stream that may coalesce under pressure. [C-turn13]
8. **Push-based incremental JSON parsers** (actson, json-event-parser) enable tool execution to begin before the model finishes generating the last brace, removing seconds of latency in tool-heavy workflows. [B16, C-turn20]

---

## Table of Contents

1. [Stream Finalization and Aggregation](#1-stream-finalization-and-aggregation)
2. [Incremental Structured Output Parsing](#2-incremental-structured-output-parsing)
3. [Reactive Backpressure and Flow Control](#3-reactive-backpressure-and-flow-control)
4. [Concurrency, Multiplexing, and Fan-Out](#4-concurrency-multiplexing-and-fan-out)
5. [Stream-as-Actor and Supervision](#5-stream-as-actor-and-supervision)
6. [Event-Typed Streaming and ModelEvent Taxonomy](#6-event-typed-streaming-and-modelevent-taxonomy)
7. [Transport Architecture: WebSocket vs SSE vs NATS Native](#7-transport-architecture-websocket-vs-sse-vs-nats-native)
8. [Production Readiness: Observability, Benchmarking, and Chaos Engineering](#8-production-readiness-observability-benchmarking-and-chaos-engineering)
9. [Synthesized Architecture Recommendation](#9-synthesized-architecture-recommendation)
10. [Implementation Roadmap](#10-implementation-roadmap)
11. [Evidence Gaps and Required Experiments](#11-evidence-gaps-and-required-experiments)
12. [References](#12-references)

---

## 1. Stream Finalization and Aggregation

**Confidence: HIGH** -- All three reports converge on the same pattern independently.

### The Finalizer Pattern

Transitioning from raw chunks to validated artifacts requires stateful finalizers that accumulate streaming deltas and produce a schema-validated final artifact upon stream completion. Multiple production frameworks have converged on this "two-phase" streaming model:

**Microsoft Agent Framework** introduces a dual-consumption pattern. When an agent runs in streaming mode, it returns a `ResponseStream` object that wraps an async stream of updates plus a `finalizer` function (e.g., `AgentResponse.from_updates`). Developers asynchronously iterate over chunks for real-time UI updates, then call `get_final_response()` to retrieve the fully aggregated, schema-validated artifact. The stream's internal finalizer automatically handles structured output parsing once all updates are received. [A12, A13, B1, B2, C-turn10, C-turn11]

**OpenAI Agents SDK** treats streaming as a first-class run lifecycle: you must consume the event iterator until it ends; only then is the run considered complete. Post-processing (session persistence, approvals bookkeeping, history compaction) may still occur after the last visible token. This "lifecycle correctness over UI tokens" principle means that "stream completion" happens when the iterator ends, not when you receive an apparent stop token. [C-turn12]

**Vercel AI SDK** defines an SSE protocol with explicit start/delta/end parts (including tool-input start/delta/available, tool-output available) and explicit message/step termination markers, including a `[DONE]` end-of-stream marker. This is effectively a wire-level finalization contract for stitched multi-step/tool-driven assistant runs. [A10, C-turn13]

### Key Techniques

- **Explicit finalizer hook**: Wrap `Stream<Update>` with a finalizer that reduces updates into a final response object (and optionally a typed structured value). [A12, B1, C-turn11]
- **Two-phase assembly**: Emit deltas for early consumption while keeping a canonical assembled buffer for final parse/validation on completion. [B1, B2]
- **Repair + enforce stage**: Deterministic small-step repairs (trailing commas, quote fixes) and optional coercions with audit logs (`changes_made`) to produce a validated object. Community Rust work documents these repair steps, emphasizing that repair is often necessary before final validation and that the stream must be considered authoritative only after finalization. [B3]
- **Lifecycle correctness**: Declare that "stream completion" happens when the iterator ends, not when you receive an apparent stop token, enabling post-run work to continue after last visible output. [C-turn12]
- **Step demarcation** for multi-call agentic loops: Emit explicit step boundaries so clients can reason about multiple backend LLM calls/tools stitched into one conversation turn (Vercel's `start-step` / `finish-step`). [C-turn13]

### Rust Implementation

For Mister Smith, this translates to a `StreamFinalizer` struct that wraps a `Stream<Item = ModelEvent>`. As the stream yields `ChunkDelta` variants, the finalizer accumulates state. Upon receiving a `Stop` event, it executes a final `serde` validation pass.

```rust
struct StreamFinalizer {
    rx: mpsc::Receiver<ModelEvent>,
    schema: Option<JsonSchema>,
}

impl StreamFinalizer {
    /// Accumulate events, forward intermediate deltas, and on end run repair + validate.
    async fn run(mut self) -> Result<FinalArtifact, FinalizeError> {
        let mut assembler = Assembler::new();
        while let Some(event) = self.rx.recv().await {
            match &event {
                ModelEvent::TextDelta { .. } => assembler.append_text(&event),
                ModelEvent::ToolInputDelta { .. } => assembler.append_tool_input(&event),
                ModelEvent::TextEnd { .. } | ModelEvent::ToolInputAvailable { .. } => {
                    // Run repair + schema validation
                    return assembler.finalize(self.schema.as_ref());
                }
                _ => {}
            }
        }
        assembler.finalize(self.schema.as_ref())
    }

    /// Mirrors ResponseStream/get_final_response style from Microsoft Agent Framework.
    async fn get_final_response(&mut self) -> Result<FinalArtifact, FinalizeError> { /* ... */ }
}
```

The finalizer should output:
- `FinalText` (or multi-part content)
- `FinalToolCalls` (validated JSON)
- `FinalUsage`
- `FinalTraceSummary` (optional)
- A "final run state" (`Succeeded`, `Failed`, `Cancelled`, `InterruptedForApproval`)

### Expected Impact vs Naive Streaming

High. Finalization avoids attempting to deserialize incomplete concatenated chunks on every delta, reduces repeated parsing work (avoiding quadratic parse behavior), provides a single validated artifact for downstream logic, and enables actionable repair reporting for operator/debugging workflows. It separates transient UI rendering from authoritative output, improving correctness and debuggability. [A12, B1, B2, C-turn11]

---

## 2. Incremental Structured Output Parsing

**Confidence: HIGH** -- All three reports address this as a critical capability, with complementary depth.

### The Incomplete JSON Problem

Incremental structured output parsing is currently the "sharp edge" of LLM streaming. Tool/function arguments are usually JSON, but during streaming they are *almost always invalid until completion*. Tool arguments arrive as progressively longer prefixes (`{"user`, `{"user":"ali`, ...) that cannot be parsed until enough bytes arrive. [C-turn16]

Providers are starting to acknowledge this explicitly. Amazon Bedrock's documentation for fine-grained tool streaming warns developers may receive invalid or partial JSON tool inputs. [C-turn2] Anthropic's API mixes text content and tool calling, requiring the parser to build the final structure incrementally. [A14]

### Parser Landscape for Rust

All three reports survey the parser ecosystem. The combined landscape:

| Parser / Library | Streaming Capability | Performance / Zero-Copy | Best Use Case |
|:---|:---|:---|:---|
| **`serde_json::StreamDeserializer`** | Native iterator over multiple JSON values [A17]. | High allocations; blocks on incomplete fragments [A15, A2]. | Baseline parsing of complete NDJSON payloads. |
| **`simd-json`** | Tape API and `fill_tape` for incremental consumption [A18]. Two-stage model: Stage 1 scans bytes via SIMD for structural characters, Stage 2 builds parsed representation [A3]. | Multi-GB/s throughput; heavily utilizes `unsafe` SIMD [A19, A3]. | High-throughput, zero-copy parsing of large tool arguments. |
| **`json-stream`** | Parses NDJSON from byte streams (e.g., Axum `BodyStream`) [A20]. | Minimizes memory by deleting deserialized bytes from the buffer [A20]. | Ingesting raw provider SSE streams at the gateway edge. |
| **`oak-json`** | Green/Red Tree architecture for incremental parsing [A21]. | Sub-millisecond latency; shares nodes without copying [A21]. | AST-level manipulation of deeply nested, evolving tool calls. |
| **`actson`** | Push-based feeder with `NeedMoreInput` events and async Tokio readers [B16]. | Minimal allocations; event-driven. | Token-buffered incremental parsing inside finalizer actors. |
| **`json-event-parser`** | SAX-like streaming JSON parsing into events [C-turn20]. | Low overhead; event-based. | Streaming JSON event generation for partial tool-call extraction. |
| **`picojson-rs`** | Zero-allocation slice and stream parsers [B13]. | Zero-allocation; suitable for constrained scenarios [B13]. | Performance-sensitive paths where input is provided as slices. |
| **`simdjson` on-demand** | Forward-only, skip-unused-fields, streaming semantics [B14]. | High throughput; minimal memory. | When skipping unused fields is acceptable. |
| **`streaming_serde_json`** | Incremental serde deserializers for large docs [B15]. | Experimental; not production-ready [B15]. | Research / future consideration. |
| **`json-stream-parser`** | Incremental character-level updates [B12]. | Experimental; not production-ready [B12]. | Research / future consideration. |
| **`deser-incomplete`** | Parse incomplete or broken data with Serde [C-turn20]. | Designed for streaming JSON. | Tolerant parsing when stream is invalid until done. |

### SIMD-Accelerated Partial Validation

Report A provides unique depth on SIMD-accelerated parsing. When using `serde_json::StreamDeserializer` for partial streams, encountering an incomplete JSON fragment causes the deserializer to return the same error in perpetuity (an infinite loop). The mitigation is to use the `byte_offset()` method to manually advance the buffer past the invalid fragment, or deserialize into a generic `Value` when typed deserialization fails. [A2, A16]

**Recommended hybrid approach**: Use `json-stream` for the outer SSE envelope, and `simd-json` for the inner tool-call argument payloads. This combines safe memory profiles for long-running streams with raw speed for token-by-token evaluation. [A-synthesis]

### Push-Based Incremental Parsing

Reports B and C emphasize push-based parsers (actson, json-event-parser) that signal `NeedMoreInput` and emit partial events (start-object, key, value fragment). This approach avoids reparsing whole buffers and enables earlier detection of structural problems. [B16, C-turn20]

### NDJSON as Mitigation

When feasible, using NDJSON/JSON-lines avoids partial-object assembly problems entirely by producing one complete object per line and using a buffered decoder to handle chunk-boundary splits. [B-synthesis]

### Three Failure Classes

Report C uniquely identifies three distinct failure classes that must be handled explicitly:

1. **Incomplete** (waiting for more bytes): common during streaming. [C-turn16, C-turn20]
2. **Invalid/partial by provider semantics** (fine-grained tool streaming may emit invalid JSON): a documented edge case. [C-turn2]
3. **Provider/tooling bugs and truncation** (invalid JSON tool args can break sessions): these show up in real deployments. [C-turn16, C-turn20]

### Streaming Schema Validation (Research Frontier)

Report C identifies academic work on validating streaming JSON against JSON Schema using visibly pushdown automata (VPA)-based approaches, demonstrating that "schema validation while bytes are in flight" is possible but nontrivial. [C-turn16] This is a cutting-edge area that could provide significant advantages but is not yet production-ready.

### Validator Context Accumulation

Report C identifies an insight from Guardrails AI: some validators can declare how much context they need (sentence/paragraph/whole output); the runtime accumulates chunks until validators can run, rather than validating per-token. This is a practical alternative to full streaming validation. [C-turn16]

### Expected Impact

Very high. Incremental structured parsing enables "tool execution starts before the model finishes typing the last brace," which can remove seconds of latency in tool-heavy workflows. It also enables early, user-visible progress (showing which tool is being called and with what partial arguments), a major UX differentiator. It avoids quadratic parse behavior from repeated full deserializations. [A-synthesis, B14, C-turn13]

---

## 3. Reactive Backpressure and Flow Control

**Confidence: HIGH** -- All three reports converge, with complementary depth on different layers.

### In-Process Backpressure: Tokio Bounded Channels

When an LLM produces tokens faster than a consumer can process them (e.g., a slow database write tool), unbounded queues will eventually fill all available memory. Tokio's bounded `mpsc::channel` provides backpressure by forcing the sender to `.await` when the channel reaches capacity. [A22, A23, B9, C3]

**Head-of-line blocking prevention**: Report A uniquely identifies that naive `select!` loops combining socket reads and channel writes can lead to read starvation. If a `write()` call blocks due to backpressure, the entire loop is blocked, inflating the receive socket buffer and pushing TCP backpressure to the remote peer. The solution is Tokio's `OwnedPermit` -- by reserving an `OwnedPermit` *before* pulling the next chunk from the LLM provider, the framework guarantees channel capacity exists, preventing head-of-line blocking and allowing cooperative cancellation. [A23, A24, A25]

### Distributed Flow Control: NATS JetStream

All three reports recommend JetStream for distributed flow control, with complementary analysis:

| Consumer Type | Delivery Mechanism | Backpressure / Flow Control Strategy |
|:---|:---|:---|
| **Push Consumer** | Server actively pushes messages to subscribers [A27]. | Relies entirely on `MaxAckPending` and sliding-window `FlowControl` [A1]. Protocol-level flow control: wire protocol specifies `100 FlowControl Request` messages that must be replied to. [C5] |
| **Pull Consumer** | Client explicitly requests batches via `fetch()` [A28, A27]. | Implicit one-to-one flow control driven by client demand [A1]. Less CPU load on NATS server; scales horizontally without complex rebalancing [A28, A30]. |
| **Ordered Consumer** | Ephemeral, single-threaded dispatch [A29]. | Automatic flow control; recreates consumer if a gap is detected [A29]. |

**Pull Consumers are vastly superior** for Mister Smith's token-heavy pipelines. They allow the agent to fetch messages at its own pace. To prevent redelivery storms when a tool takes too long, configure `MaxAckPending` to limit in-flight messages, and use the `Backoff` sequence to override the static `AckWait` with exponential delays. [A1, A31]

Report C adds that JetStream flow control can stall if not handled correctly -- "pressure bugs" can look like mysterious hangs unless modeled explicitly. [C5]

### Backpressure as a Policy Matrix

Report C uniquely contributes the insight that backpressure should not be binary (block or don't block) but a **per-event-class policy**:

- **Lossless**: tool-call boundaries, step boundaries, finalization events, approvals, errors.
- **Lossy/coalescible**: text deltas, "typing" indicators, intermediate partial JSON deltas (while preserving the final "arguments done"). [C-turn13, C-turn17]

This parallels GStreamer's queue elements, which can block upstream when full or be configured as "leaky" to drop old/new buffers instead of blocking. [C-turn17] And Aeron's `offer` returning a negative code (`-2`) when backpressured, requiring the application to retry/back off. [C-turn17]

### Coalescing as Backpressure Strategy

Report C uniquely identifies coalescing (merging token deltas into larger chunks under pressure) as an alternative to pure blocking. This parallels Akka's discussion of windowed/batching strategies to amortize async-boundary costs. [C3]

### Recommended Backpressure Heuristics

From Report B's practical starting points:

- **Local buffer sizes**: Start with small bounded channels (32-256 events) and observe behavior per workload; use buffered concurrency to limit parallel tool executions.
- **JetStream**: Set `MaxAckPending` to reflect expected concurrency per consumer; enable flow control / `RateLimit` when consumers are slow.
- **SSE/WebSocket frontends**: Apply send timeouts (to close stale connections) and per-client drop policy (`DropOldest` for realtime UIs; `DropNewest` when preserving earlier context matters). [B9, B10, B11]

### Backpressure as Observability

Report C uniquely identifies that backlog depth and "send waited" time should be treated as metrics and trace events -- a proven pattern in actor pipelines. [C-turn19]

### Expected Impact

Very high. Explicit end-to-end backpressure prevents unbounded memory growth, avoids slow-consumer collapse, tail-latency explosions, and "random stream stops" under multi-tool workloads -- failure modes that are common once you move beyond single-call chat. [A22, B9, C-turn19]

---

## 4. Concurrency, Multiplexing, and Fan-Out

**Confidence: HIGH** -- All three reports address this with complementary techniques.

### StreamMap vs SelectAll for Dynamic Fan-In

Report A provides the most detailed analysis of stream aggregation primitives:

When a Planner agent spawns multiple sub-agents, their streaming responses must be merged. Using `futures::stream::SelectAll` is problematic in Rust: it requires uniform stream types, and mapping closures often results in unnameable type mismatches (`expected closure, found a different closure`). Furthermore, `SelectAll` polls streams in round-robin fashion, which degrades as the number of streams grows. [A8, A32]

**`tokio_stream::StreamMap`** is the recommended alternative. It combines multiple streams indexed by unique key (e.g., correlation ID or sub-agent ID). It requires streams to be `Unpin` (often achieved via `Box::pin`), but provides O(1) insertion and removal, making it ideal for dynamic "stream of streams" fan-in where sub-agents spin up and shut down dynamically. [A33, A9]

Report C additionally identifies `futures::stream::flatten_unordered(limit)` as a standard operator for the "outer stream yields per-agent inner streams" pattern. [C4]

### Ordering Strategies

The reports offer complementary ordering strategies:

- **Per-agent ordering**: Tag events with `(agent_id, sequence_number)` so per-agent consumers can enforce local ordering even after global merge. [B-synthesis]
- **Lamport logical clocks**: Include logical clocks or sequence numbers in payloads to preserve causality across merged streams. [A34]
- **Watermark-based merging**: Declare "we believe all events up to time T have arrived," enabling bounded buffering and ordered emission from out-of-order sources (from distributed stream processing). [C-turn9]
- **Step demarcation**: Carry `step_id` in events. Vercel explicitly requires step boundaries to correctly handle "multiple stitched assistant calls." [C-turn13]

Report C adds a critical ordering caveat: Core NATS ordering is **per publisher**, not total ordering across publishers. If multiple agents publish concurrently, you must either (a) accept nondeterministic interleaving, or (b) introduce an ordering key/sequence at the application layer. [C4]

### JetStream for Distributed Fan-Out

For distributed multiplexing across agent processes, JetStream provides a native multiplexing dimension via **subjects**. Publish each agent-run's events to a subject like `ms.<run_id>.<agent_id>.events.*`, and allow per-subscriber consumers (UI, tracing, persistence, tool execution) to filter server-side. Filter-subject semantics are explicitly supported for consumers. [C5]

JetStream features useful for shaping message retention:
- **Rollup**: `AllowRollup` / `Nats-Rollup` headers allow pruning prior messages when only the latest state matters. [B8]
- **Sources/mirrors**: For replicating streams across clusters. [B8]
- **Retention policies**: Per-subject retention for managing distributed fan-out. [B8]

### Expected Impact

High. Good multiplexing enables true multi-agent streaming UX (planner + subagents + tools + observations) without sacrificing correctness or debuggability, supports planners that spawn parallel tool-using agents, prevents head-of-line blocking that naive aggregation causes, and makes NATS/JetStream an architectural advantage rather than just a transport. [A-synthesis, B7, C5]

---

## 5. Stream-as-Actor and Supervision

**Confidence: HIGH** -- All three reports converge on this pattern independently.

### Erlang-Style Dynamic Supervision for Stream Actors

In Erlang/OTP, a `DynamicSupervisor` (formerly `simple_one_for_one`) is optimized to start children dynamically on demand, allowing it to hold millions of transient processes without ordering constraints. Modeling a streaming LLM response as a short-lived actor provides immense benefits: it encapsulates the stream's state, provides a dedicated mailbox, and allows a supervisor to restart the stream upon network failure. [A35, A5, A36]

Report C adds the precedent from Elixir's GenStage, which formalizes demand-driven pipelines between processes: consumers request N events; producers emit at most N, providing built-in backpressure. This "stream = process with a mailbox" intuition maps directly to the actor-per-stream pattern. [C-turn9]

### Rust Actor Framework Comparison

| Rust Actor Framework | Supervision Capabilities | Mailbox & State Management |
|:---|:---|:---|
| **Actix** | Basic `Supervisor` struct; restarts actors on failure [A37, A38]. | Uses mutable self; lacks primitives for non-blocking long-running tasks [A4]. |
| **Ractor** | Pure-Rust Erlang `gen_server` clone; full supervision trees [A4, A39]. | Separate state type; supports `SupervisionEvent` (startup, death, panic) [A4, A39]. |
| **Bastion** | Highly-available, fault-tolerant runtime; dynamic supervision [A40]. | `One-For-One` and `All-For-One` strategies; NUMA-aware executor [A40]. |
| **ractor-supervisor** | Erlang-like supervisors with restart strategies and backoff [B19]. | Implements OTP-style supervision policies. |
| **Supertrees** | Experimental OTP-style supervision ports to Rust [B-turn60]. | Research-stage. |

**Ractor is the most aligned** with Mister Smith's OTP-style requirements. By spawning a `ractor` actor for each LLM stream, Mister Smith can utilize `SupervisionEvent`s to detect stream panics. If a stream fails, the supervisor can apply a `One_For_One` strategy to restart only that specific LLM call, rather than crashing the entire multi-agent workflow. [A4, A39, B18, B19]

### Per-Run Stream Actor Design

Each streamed LLM response should be modeled as a short-lived actor (supervised) that:

1. Reads provider events (SSE or WebSocket)
2. Decodes them into `ModelEvent`
3. Publishes canonical events to NATS subjects (Core for ephemeral UI, JetStream for durable audit)
4. Runs the finalizer to produce final artifacts
5. Enforces backpressure policy on downstream consumers [C-turn19, C5]

The actor owns: (a) provider connection, (b) incremental parsers, (c) finalizer state, (d) outbound routing. [C-turn19]

### Supervisor Policies

- **One-for-one restart** for transient errors (parse failure, network blip)
- **Escalate** after N restarts with exponential backoff
- **Mark artifact failed** and emit `Error` event with `recoverable=false` only after repair attempts exhausted [B17, B19, A35]

### Pooled-Worker Hybrid

Report B uniquely identifies an alternative when per-stream actor overhead is high: use a pool that owns worker tasks and routes stream identifiers through bounded queues with per-stream metadata. [B-synthesis]

### Conceptual Stream Actor Code

```rust
// Conceptual Mister Smith Stream Actor using Ractor
use ractor::{Actor, ActorProcessingErr, ActorRef};

struct LlmStreamActor;
struct StreamState { buffer: Vec<u8>, correlation_id: String }

#[async_trait::async_trait]
impl Actor for LlmStreamActor {
    type Msg = ChunkDelta;
    type State = StreamState;
    type Arguments = String;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(StreamState { buffer: Vec::new(), correlation_id: args })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ChunkDelta::Text(text) => state.buffer.extend_from_slice(text.as_bytes()),
            ChunkDelta::Stop => { /* Finalize and send to NATS */ }
            _ => {}
        }
        Ok(())
    }
}
```

### JetStream for Actor Recovery

JetStream provides durable replay / message redelivery to allow restarted actors to recover unprocessed segments. Restarted actors can rehydrate state from JetStream replay. [B8, B19]

### Expected Impact

High. Actor-per-stream provides clear isolation, fine-grained lifecycle control, and direct supervisory recovery for streaming failures (network blips, parse errors). It simplifies routing partial outputs to other agents. Using supervisors and durable JetStream storage improves resilience and observability. The actor owns completion, making lifecycle edge cases (cancellation, consumer disconnect, timeouts) easier to reason about. [A-synthesis, B17, C-turn19]

---

## 6. Event-Typed Streaming and ModelEvent Taxonomy

**Confidence: HIGH** -- All three reports converge on the need for a rich typed event model.

### State of the Art

Modern AI SDKs have converged on typed streaming events with start/delta/end patterns:

- **OpenAI Responses API**: Typed semantic events (`response.output_text.delta`, `response.completed`, dedicated "arguments delta/done" events for function calls). [C6]
- **Vercel AI SDK**: Multi-part SSE protocol (`text-delta`, `tool-input-delta`, `tool-output-available`, `finish-step`), with explicit `[DONE]` end-of-stream marker. [A10, C-turn13]
- **OpenAI Agents SDK**: Higher-level "run item" events (message generated, tool called, tool output, handoff requested) layered on top of raw token streaming. [C-turn12]

### ModelEvent Design: Convergent Proposals

All three reports propose a `ModelEvent` enum. The synthesized superset across all proposals:

```rust
use serde::Deserialize;
use bytes::Bytes;

/// Canonical streaming event type for all LLM interactions.
/// Uses #[non_exhaustive] for forward compatibility when providers
/// add new event types. Uses #[serde(other)] for graceful fallback.
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ModelEvent {
    // --- Lifecycle events ---
    RunStarted { run_id: String, metadata: EventMeta },
    StepStarted { step_id: String, metadata: EventMeta },
    StepFinished { step_id: String, finish_reason: Option<String> },
    RunCompleted { run_id: String, usage: UsageStats },
    RunFailed { run_id: String, error: ErrorInfo },

    // --- Text streaming ---
    TextStart { id: String, metadata: EventMeta },
    TextDelta { id: String, chunk: Bytes },
    TextEnd { id: String, finish_reason: Option<String> },

    // --- Tool call lifecycle ---
    ToolCallStart { call_id: String, tool_name: String },
    ToolInputDelta { call_id: String, input_text_delta: String },
    ToolInputAvailable { call_id: String, input: serde_json::Value },
    ToolResultAvailable { call_id: String, result: serde_json::Value },

    // --- Structured output fragments ---
    StructuredFragment { schema_id: String, fragment: serde_json::Value },

    // --- Observability ---
    UsageDelta { tokens_in: usize, tokens_out: usize },
    SpanStart { span_id: String, metadata: EventMeta },
    SpanEnd { span_id: String, metadata: EventMeta },

    // --- Error handling ---
    Error { code: u16, message: String, recoverable: bool },

    // --- Heartbeat / keep-alive ---
    Heartbeat,

    // --- Forward compatibility ---
    #[serde(other)]
    Unknown,
}
```

**Report A's emphasis**: `#[non_exhaustive]` ensures the framework never breaks when OpenAI or Anthropic silently update their SSE protocols. `#[serde(other)]` allows the deserializer to gracefully fall back to an `Unknown` variant rather than panicking. [A49, A50]

**Report B's emphasis**: The event shape should include `ToolInputAvailable` (when tool input is assembled) separate from `ToolInputDelta` (incremental chunks), enabling typed handlers and finalizers. Usage updates should be streamable incrementally for near-real-time billing/monitoring. [B20, B21]

**Report C's emphasis**: The event model should be expressive enough for lifecycle events (`RunStarted`, `StepStarted`, `StepFinished`, `RunCompleted`, `RunFailed`) aligned with OpenAI and Vercel patterns. Error events should have a `recoverable` flag to let supervisors decide whether to terminate or resume. [C6, C-turn13, C-turn21]

### Alternative: Event Envelope Design

Report B proposes an alternative envelope-based design for distributed routing:

```rust
pub struct EventEnvelope {
    pub session: SessionId,
    pub offset: u64,        // per-session sequence number
    pub body: EventBody,
    pub trace_ctx: Option<TraceContext>,
}

pub enum EventBody {
    Text(TextEvent),
    Tool(ToolEvent),
    Obs(ObservabilityEvent),
    Err(ErrorEvent),
    // ...
}
```

This separates routing metadata (session, sequence, trace context) from event content, which is advantageous for NATS subject-based routing and JetStream persistence. [B-synthesis]

### Dual-Stream Design

Report C uniquely proposes emitting two streams from the same canonical event log:

1. **Lossless semantic event stream** for correctness/replay (all events preserved)
2. **Best-effort UI stream** that may coalesce/downgrade under pressure (text deltas merged, heartbeats dropped)

This separation is supported by Vercel's protocol already separating fine-grained deltas from "available/done" boundaries. [C-turn13]

### Expected Impact

High. Typed events make composition, supervision, observability, and finalization deterministic and testable. They simplify incremental parsing, tool orchestration, and the ability to route one model's streaming output into another model/tool pipeline in real time. [A10, B20, C-turn13]

---

## 7. Transport Architecture: WebSocket vs SSE vs NATS Native

**Confidence: HIGH** -- All three reports cover this with complementary depth.

### Transport Comparison

| Transport | Bidirectional | Backpressure Handling | Proxy / CDN Compatibility | Best Use Case |
|:---|:---|:---|:---|:---|
| **SSE (HTTP/1.1)** | No (Server-to-Client only) [A44, A45]. | Relies on TCP window; prone to head-of-line blocking [A44]. | High, but requires strict header tuning to prevent buffering [A43, A7]. | Client-facing UI streams with simple server push. |
| **WebSocket** | Yes (Full-duplex) [A45]. | Application-level ping/pong; requires manual reconnection [A45]. | Requires sticky sessions on ALBs [A46]; blocked by some enterprise firewalls [A45]. | Provider connections for tool-heavy agentic loops; interactive UI (cancel, approve). |
| **NATS / JetStream** | Pub/sub (multi-directional) [B23]. | JetStream consumer flow control, MaxAckPending, RateLimit [A1, C5]. | Internal only; not browser-accessible. | Server-to-server durable messaging, distributed fan-out, replay/debug. |

### SSE Proxy Buffering Problem

Report A uniquely identifies a critical production issue: **default NGINX and Cloudflare configurations buffer SSE responses**, destroying token-by-token perceived latency.

- NGINX uses `proxy_buffering on` by default, which batches chunks. [A7, A41]
- Cloudflare buffers SSE responses until approximately 100KB accumulates before flushing. [A6]

**Required mitigations**:
- Inject `X-Accel-Buffering: no` header into application server response. [A42, A7]
- Configure NGINX with `proxy_buffering off`, `proxy_cache off`, and `chunked_transfer_encoding off` for SSE endpoints. [A43]

### WebSocket for Provider Connections

Report C provides unique depth on OpenAI's WebSocket mode for the Responses API:

- Explicitly targets long-running, tool-call-heavy workflows by keeping a persistent connection and continuing each turn with incremental inputs plus `previous_response_id`. [C8]
- Claimed ~40% faster end-to-end execution for rollouts with 20+ tool calls. [C8]
- Operational constraints: runs sequentially (one in-flight response per connection), no multiplexing, 60-minute connection duration limit requiring reconnect/recover strategies. [C8]
- OpenAI Agents JS SDK provides configuration knobs like `setOpenAIResponsesTransport('websocket')`. [C-turn15]

### SSE Reconnection Semantics

Report C uniquely notes that SSE includes standardized reconnection behavior: connection reestablishment and `Last-Event-ID` support are part of the HTML server-sent events model, standardized by WHATWG. This makes SSE more resilient than raw WebSocket for simple delivery scenarios. [C-turn15]

### Hybrid Transport Recommendation

All three reports converge on a hybrid approach:

- **Provider transport**: SSE by default, optional WebSocket for high round-trip workflows (especially when providers support incremental-input continuation semantics like OpenAI's `previous_response_id`).
- **Internal transport**: NATS/JetStream for server-to-server durable messaging, distributed fan-out, and replay.
- **Client transport**: SSE for simple consumption; WebSocket when bidirectional control is needed (cancel, approve tools, interactive debugging).

### Expected Impact

For long tool-call chains, WebSocket mode can materially reduce end-to-end latency. For production SSE, proper proxy configuration is essential. Using JetStream as the internal backbone makes NATS a first-class streaming substrate rather than just transport. [A-synthesis, C8]

---

## 8. Production Readiness: Observability, Benchmarking, and Chaos Engineering

### Tail Latency Amplification

Report A uniquely applies Google's "Tail at Scale" analysis to multi-agent pipelines: a single slow tool call can bottleneck the entire aggregated response if unbounded merges are used. If a planner agent waits for 3 sub-agents, overall latency is dictated by the 99th percentile latency of the slowest agent. [A11]

**Mitigation**: Implement "tied requests" or strict timeouts using `buffer_unordered`. By setting a concurrency limit and allowing items to complete in any order, the system prevents a single slow LLM stream from stalling the event loop. [A11]

### Chaos Engineering Scenarios

Report A defines concrete chaos engineering scenarios:

| Chaos Scenario | Injection Method | Expected System Behavior |
|:---|:---|:---|
| **Broker Failure** | `kill -9` on NATS node [A52]. | JetStream Raft consensus fails over; latency spikes briefly but recovers [A53, A52]. |
| **Slow Consumer** | Artificial `sleep()` in tool execution. | `MaxAckPending` halts LLM ingestion; TCP backpressure applied to provider. |
| **Stream Truncation** | Drop TCP connection mid-SSE. | Actor supervisor detects `Eof` without `Stop` event; triggers `One_For_One` restart [A35]. |

### Observability Integration

Mister Smith must integrate `tokio-metrics` and OpenTelemetry. By tracking `tokio_tasks_spawned`, `tokio_task_poll_duration`, and `tokio_workers_idle`, operators can detect scheduler starvation and queue depth. Distributed tracing spans should carry the `correlation_id` from the initial user request through NATS JetStream headers, allowing full visibility into the exact millisecond a specific tool call stalled the pipeline. [A54]

Report B adds that `SpanStart`/`SpanEnd` events should be part of the `ModelEvent` stream for near-real-time metrics, billing attribution, and correlation with distributed traces. [B21]

### Benchmarking

The `nats bench` CLI tool provides a baseline for JetStream performance, capable of simulating asynchronous publishers and durable pull consumers. [A51]

### Test Scenarios (from Report B)

Minimum three realistic workloads to validate the design:

1. **Short chatty streams with many concurrent agents**: Many short streams emitting small deltas, large concurrency; validate low per-stream latency, bounded memory, and fair scheduling using `buffered(n)` and bounded mpsc.
2. **Long-lived multi-step agentic loops**: Streams that interleave tool calls and model responses over long durations; validate actor supervision, per-stream ordering, and finalizer correctness (including repair of structured outputs).
3. **High-throughput token bursts**: Single or few streams emitting tokens at very high rate; validate backpressure propagation from consumer to provider, JetStream flow control behavior, and system stability under full buffers.

### Measurable Expected Improvements (from Report B)

- **Latency**: Earlier UI rendering retained for incremental tokens; final artifact latency reduced by avoiding quadratic parse behavior from repeated full deserializations.
- **Resource savings**: Bounded channels and JetStream flow control prevent unbounded memory growth; zero-allocation parser options reduce heap usage.
- **Failure recovery**: Actor supervision + durable JetStream persistence reduces lost progress and enables replay on restart; typed `ModelEvent` reduces ambiguity and simplifies recovery logic.
- **Correctness**: Finalizer + repair + schema validation produces a validated final artifact, avoiding brittle downstream behavior caused by partial or malformed JSON.

---

## 9. Synthesized Architecture Recommendation

The following architecture synthesizes the strongest recommendations from all three reports into a unified, implementable design.

### Design Principles

1. **Canonical event log first, presentation stream second**: Define streaming as a typed event stream rather than "text tokens." The `ModelEvent` enum is the internal canonical log. Derive a presentation stream (UI deltas, progress indicators) via coalescing/windowing operators so backpressure policies can differ by event importance.

2. **Stream finalizers as first-class, supervised components**: Implement a Rust `StreamFinalizer` trait as a deterministic reducer over `ModelEvent`. Run this finalizer inside a stream actor to guarantee finalization happens even if downstream consumers stop reading.

3. **Backpressure as a policy matrix**: Define backpressure policy per event class (lossless for boundaries/errors, lossy/coalescible for text deltas). Implement with bounded in-process channels plus coalescing operators, and use JetStream as a durable buffer for debug/replay and distributed fan-out.

4. **Each stream is a supervised actor**: Model each streamed LLM response as a short-lived actor that reads provider events, decodes into `ModelEvent`, publishes to NATS, runs the finalizer, and enforces backpressure policy. OTP-style supervision manages lifecycle, restart, and escalation.

5. **Transport as swappable**: SSE by default for providers, optional WebSocket for tool-heavy loops. NATS/JetStream for all internal routing. SSE or WebSocket for client egress depending on bidirectional requirements.

### End-to-End Data Flow

```
LLM Provider (SSE/WebSocket)
    |
    v
[Provider Adapter] -- decodes SSE/WS into ModelEvent stream
    |
    v
[Stream Actor (supervised)] -- owns: connection, parser, finalizer, routing
    |  - Bounded mpsc channel (OwnedPermit for backpressure)
    |  - Incremental JSON parser (actson/simd-json hybrid)
    |  - StreamFinalizer (accumulate + validate on completion)
    |
    +---> NATS Core subjects (ephemeral UI events)
    +---> JetStream subjects (durable audit/replay)
    +---> StreamMap (fan-in for planner aggregation)
    |
    v
[Downstream Consumers]
    - UI stream (SSE/WS, coalesced under pressure)
    - Tool executor (acts on ToolInputAvailable)
    - Persistence (JetStream -> PostgreSQL)
    - Observability (SpanStart/SpanEnd -> OTel)
```

### Core Primitives

| Primitive | Implementation | Purpose |
|:---|:---|:---|
| `ModelEvent` enum | `#[non_exhaustive]`, `#[serde(other)]` | Typed canonical event log |
| `StreamFinalizer` | `fold`/`try_fold` over `ModelEvent` | Deterministic artifact production |
| `LlmStreamActor` | `ractor` actor, supervised | Lifecycle management, fault isolation |
| Bounded `mpsc` + `OwnedPermit` | Tokio channels | In-process backpressure |
| JetStream Pull Consumer | `MaxAckPending`, `Backoff` | Distributed flow control |
| `StreamMap` | `tokio_stream::StreamMap` | Dynamic fan-in with O(1) ops |
| Incremental parser | `actson` + `simd-json` hybrid | Sub-millisecond partial validation |
| Event envelope | Session ID + sequence + trace context | Distributed routing and ordering |

### Zero-Cost / Low-Allocation Choices

- Use zero-allocation parsers (`picojson-rs`) when inputs can be provided as slices and deterministic memory control is required.
- Use `simd-json` on-demand when skipping unused fields is acceptable to reduce memory and CPU overhead.
- Prefer `bytes::Bytes` / buffer reuse across deltas rather than cloning strings when producing `ModelEvent` chunks.
- Use `json-stream` for the outer SSE envelope to minimize memory by deleting deserialized bytes from the buffer.

---

## 10. Implementation Roadmap

### Phase 1: Prototype (2-4 weeks)

- Implement provider adapters that map SSE/WebSocket chunks into `ModelEvent` `Stream` using `tokio_stream`.
- Implement a `StreamFinalizer` actor that accumulates events, emits intermediate typed deltas, and returns a final artifact on `TextEnd`.
- Use in-memory bounded `mpsc` channels and local supervision via a simple task wrapper.
- Define the `ModelEvent` enum with `#[non_exhaustive]` and `#[serde(other)]`.

### Phase 2: Integration and Load Testing (2-6 weeks)

- Add incremental parser (`actson`/`json-event-parser`) into finalizer.
- Run workloads: short-chatty, long multi-step agentic loops, token bursts.
- Measure memory, latency, parsing CPU; test backpressure with bounded channels.
- Implement `StreamMap` for fan-in of concurrent sub-agent streams.
- Backpressure sweep: measure latency, memory, and throughput across channel buffer sizes (32, 128, 512) and JetStream `MaxAckPending` values under token-burst workloads.
- Parser correctness and latency: feed malformed and partial JSON tool-call fragments to candidate parsers and measure recovery rate, latency, and CPU cost.

### Phase 3: JetStream Persistence and Distributed Rollout (2-4 weeks)

- Persist `ModelEvent` stream into JetStream subjects; configure durable consumers, `MaxAckPending`, and `RateLimit`.
- Test consumer restarts and replay.
- Replace local supervision restarts with supervisors capable of rehydrating state from JetStream replay when actor restarts.
- Implement distributed fan-out with subject-based routing (`ms.<run_id>.<agent_id>.events.*`).

### Phase 4: Production Hardening (ongoing)

- Implement observability (usage events, tracing, `SpanStart`/`SpanEnd`).
- Operational limits on connections (SSE/WebSocket).
- Robust repair heuristics with audit logging.
- Scale worker pools, tune channel sizes, instantiate per-region JetStream clusters as needed.
- Chaos engineering: broker failure, slow consumer, stream truncation scenarios.
- Implement backpressure policy matrix (lossless vs. lossy per event class).
- Add WebSocket transport option for provider connections in tool-heavy workflows.

---

## 11. Evidence Gaps and Required Experiments

### Gaps Identified Across All Reports

1. **Precise numeric backpressure defaults** (channel sizes, `MaxAckPending`) for representative workloads are not specified and require benchmarking across Mister Smith's expected token rates and concurrency. [B9, B10]
2. **Production maturity of streaming-parsing crates** (`json-stream-parser`, `streaming_serde_json`) is flagged as experimental; empirical validation on target workloads is needed before adoption. [B12, B15]
3. **End-to-end producer-side throttling semantics** for SSE/WebSocket in the context of LLM providers (whether the producer can be slowed by client backpressure) is not detailed; behavior varies by provider and transport. [B11, B21]
4. **Exact CPU/latency profiles** for `simd-json` on-demand vs `picojson-rs` vs `streaming_serde_json` vs `actson` within Mister Smith workloads are not present; microbenchmarks are needed to decide parser selection. [B13, B14, B15]
5. **Streaming schema validation** via VPA-based approaches is academically promising but not production-ready. [C-turn16]
6. **JetStream flow control stall behavior** under specific configurations needs empirical testing to avoid "pressure bugs" that look like mysterious hangs. [C5]

### Required Experiments / Benchmarks

1. **Backpressure sweep**: Measure latency, memory, and throughput across channel buffer sizes (32, 128, 512) and JetStream `MaxAckPending` values under token-burst workloads.
2. **Parser correctness and latency**: Feed malformed and partial JSON tool-call fragments to candidate parsers (`actson`, `json-event-parser`, `picojson-rs`, `simd-json`) and measure recovery rate, latency, and CPU cost.
3. **Multiplexing and ordering**: Spawn planners that fan out to N sub-agents streaming concurrently (N=3, 10, 50) and verify per-agent ordering and end-to-end assembly correctness.
4. **Supervisor recovery**: Inject transient network failures and parser panics to verify actor restart, JetStream replays, and final artifact correctness.
5. **Provider-side backpressure**: Test whether real LLM providers (OpenAI, Anthropic) actually slow token generation when client consumption slows, or whether they buffer/drop.
6. **WebSocket vs SSE latency**: Measure actual end-to-end latency difference for tool-heavy workflows (10, 20, 50 tool calls) comparing SSE and WebSocket provider connections.

---

## 12. References

References are deduplicated across all three reports. Prefix indicates the source report: [A] = Report A, [B] = Report B, [C] = Report C.

### NATS and JetStream

- [A1] *Consumers - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream/consumers
- [A26] *JetStream - NATS Docs*. https://docs.nats.io/jetstream
- [A27] *How to Build NATS Consumers*. https://oneuptime.com/blog/post/2026-02-02-nats-consumers/view
- [A28] *Consumer Details - NATS Docs*. https://docs.nats.io/using-nats/developer/develop_jetstream/consumers
- [A29] *Consumers (Concepts) - NATS Docs*. https://docs.nats.io/jetstream/concepts/consumers
- [A30] *Grokking NATS Consumers: Pull-based*. https://www.byronruth.com/grokking-nats-consumers-part-3/
- [A31] *ConsumerConfig in nats::jetstream - Rust*. https://docs.rs/nats/latest/nats/jetstream/struct.ConsumerConfig.html
- [A51] *nats bench | NATS Docs*. https://docs.nats.io/using-nats/nats-tools/nats_cli/natsbench
- [A52] *Chaos Engineering for Streaming Systems | Conduktor*. https://www.conduktor.io/glossary/chaos-engineering-for-streaming-systems
- [A53] *nats-general/SECURITY-SELF-ASSESSMENT.md*. https://github.com/nats-io/nats-general/blob/main/SECURITY-SELF-ASSESSMENT.md
- [B8] *Streams - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream/streams
- [B10] *JetStream Java Client - Consume*. https://nats.io/blog/jetstream-java-client-03-consume/
- [B23] *nats.rs - GitHub*. https://github.com/nats-io/nats.rs

### Tokio and Rust Async

- [A22] *Channels | Tokio Tutorial*. https://tokio.rs/tokio/tutorial/channels
- [A23] *tokio::sync::mpsc - Rust*. https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html
- [A24] *Async Rust with Tokio I/O Streams: Backpressure, Concurrency, and Ergonomics*. https://biriukov.dev/rust-tokio-io/
- [A25] *OwnedPermit in tokio::sync::mpsc - Rust*. https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.OwnedPermit.html
- [A32] *futures::stream - Rust*. https://docs.rs/futures/latest/futures/stream/index.html
- [A33] *tokio_stream - Rust*. https://docs.rs/tokio-stream/latest/tokio_stream/
- [A54] *How to Monitor Tokio Runtime Metrics with OpenTelemetry in Rust*. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view
- [B6] *A Guided Tour of Streams in Rust*. https://www.qovery.com/blog/a-guided-tour-of-streams-in-rust
- [B7] *Infinite Data Streams with Tokio Async Rust*. https://oneuptime.com/blog/post/2026-01-25-infinite-data-streams-tokio-async-rust/view
- [B9] *tokio::sync::mpsc::channel - Rust*. https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html

### JSON Parsing and SIMD

- [A2] *Step past errors in serde_json StreamDeserializer - Rust Forum*. https://users.rust-lang.org/t/step-past-errors-in-serde-json-streamdeserializer/84228
- [A3] *Rust: Why SIMD JSON parsing is different*. https://medium.com/@trivajay259/rust-why-simd-json-parsing-is-different-and-why-simdjson-changed-the-conversation-b170955662b7
- [A15] *Reading JSON sequentially - Rust Forum*. https://users.rust-lang.org/t/reading-json-sequentially/57708
- [A16] *serde-rs/json Issue #70*. https://github.com/serde-rs/json/issues/70
- [A17] *serde_json - Rust docs*. https://docs.rs/serde_json/latest/serde_json/
- [A18] *simd-json - Rust docs*. https://docs.rs/simd-json/latest/simd_json/
- [A19] *simd-json - GitHub*. https://github.com/simd-lite/simd-json
- [A20] *json-stream - crates.io*. https://crates.io/crates/json-stream
- [A21] *oak-json - crates.io*. https://crates.io/crates/oak-json
- [B3] *I Built an API for LLM JSON Validation in Rust*. https://dev.to/mtdevworks/i-built-an-api-for-llm-json-validation-in-rust-heres-what-i-learned-36nc
- [B12] *json-stream-parser - crates.io*. https://crates.io/crates/json-stream-parser
- [B13] *picojson-rs - GitHub*. https://github.com/kaidokert/picojson-rs
- [B14] *simdjson on-demand API*. https://simdjson.org/api/0.6.0/md_doc_ondemand.html
- [B15] *streaming_serde_json - crates.io*. https://crates.io/crates/streaming_serde_json
- [B16] *actson - Rust docs*. https://docs.rs/actson

### Actor Frameworks and Supervision

- [A4] *Ractor: not just another actor framework - Reddit*. https://www.reddit.com/r/rust/comments/113dp70/ractor_not_just_another_actor_framework
- [A5] *DynamicSupervisor behaviour (Elixir)*. https://hexdocs.pm/elixir/DynamicSupervisor.html
- [A35] *Erlang -- Supervisor Behaviour*. https://www.erlang.org/docs/24/design_principles/sup_princ
- [A36] *Erlang -- Processes*. https://erlang.org/doc/reference_manual/processes.html
- [A37] *Supervisor in actix - Rust*. https://docs.rs/actix/latest/actix/struct.Supervisor.html
- [A38] *Odd supervision - actix Issue #204*. https://github.com/actix/actix/issues/204
- [A39] *ractor - GitHub*. https://github.com/slawlor/ractor
- [A40] *bastion - GitHub*. https://github.com/bastion-rs/bastion
- [B17] *Erlang/OTP Design Principles (PDF)*. https://erlang.org/documentation/doc-5.6/pdf/design_principles.pdf
- [B18] *Ractor: not just another actor framework - Reddit*. https://www.reddit.com/r/rust/comments/113dp70/ractor_not_just_another_actor_framework/
- [B19] *ractor-supervisor - Rust docs*. https://docs.rs/ractor-supervisor

### AI SDKs and LLM Streaming APIs

- [A10] *AI SDK UI: Stream Protocols - Vercel*. https://ai-sdk.dev/docs/ai-sdk-ui/stream-protocol
- [A12] *Running Agents | Microsoft Learn*. https://learn.microsoft.com/en-us/agent-framework/agents/running-agents
- [A13] *Producing Structured Output with agents | Microsoft Learn*. https://learn.microsoft.com/en-us/agent-framework/agents/structured-output
- [A14] *Comparing the streaming response structure for different LLM APIs*. https://medium.com/percolation-labs/comparing-the-streaming-response-structure-for-different-llm-apis-2b8645028b41
- [A47] *Realtime API | OpenAI*. https://developers.openai.com/api/docs/guides/realtime/
- [A48] *Realtime Transport Layer | OpenAI Agents SDK*. https://openai.github.io/openai-agents-js/guides/voice-agents/transport/
- [B1] *Semantic Kernel Agent Streaming | Microsoft Learn*. https://learn.microsoft.com/en-us/semantic-kernel/frameworks/agent/agent-streaming
- [B20] *Vercel AI SDK - Message per Response*. https://ably.com/docs/guides/ai-transport/vercel-ai-sdk/vercel-message-per-response
- [B21] *OpenAI Streaming Responses Guide*. https://developers.openai.com/api/docs/guides/streaming-responses/

### Transport and SSE

- [A6] *SSE endpoint breaks after recent update - Cloudflare Community*. https://community.cloudflare.com/t/sse-endpoint-breaks-after-recent-update-cloudflare-buffers-text-event-stream-desp/810790
- [A7] *For Server-Sent Events (SSE) what Nginx proxy configuration is appropriate? - Server Fault*. https://serverfault.com/questions/801628/for-server-sent-events-sse-what-nginx-proxy-configuration-is-appropriate
- [A41] *NGINX proxy_buffering documentation*. http://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_buffering
- [A42] *Using Server Sent Events (SSE) with Cloudflare Proxy*. https://community.cloudflare.com/t/using-server-sent-events-sse-with-cloudflare-proxy/656279
- [A43] *How to Configure Server-Sent Events Through Nginx*. https://oneuptime.com/blog/post/2025-12-16-server-sent-events-nginx/view
- [A44] *Using server-sent events - MDN*. https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
- [A45] *WebSockets vs Server-Sent Events - Ably*. https://ably.com/blog/websockets-vs-sse
- [A46] *Which stickiness strategy is right for you? - AWS Prescriptive Guidance*. https://docs.aws.amazon.com/prescriptive-guidance/latest/load-balancer-stickiness/options.html
- [B11] *Server-Sent Events: A Comprehensive Guide*. https://medium.com/@moali314/server-sent-events-a-comprehensive-guide-e4b15d147576
- [B22] *SSE vs WebSockets: Comparing Real-Time Communication Protocols*. https://softwaremill.com/sse-vs-websockets-comparing-real-time-communication-protocols/

### Distributed Systems Theory

- [A11] *The Tail at Scale - Dean & Barroso*. https://cseweb.ucsd.edu/classes/sp18/cse291-c/post/schedule/p74-dean.pdf
- [A34] *Time, Clocks, and the Ordering of Events in a Distributed System - Lamport*. https://lamport.azurewebsites.net/pubs/time-clocks.pdf

### Reactive and Stream Processing

- [A8] *Merging Disparate Streams using SelectAll - Rust Forum*. https://users.rust-lang.org/t/merging-disparate-streams-using-selectall/31930
- [A9] *Merge-sorting tokio::io::Lines streams using StreamMap - Rust Forum*. https://users.rust-lang.org/t/merge-sorting-tokio-lines-streams-using-streammap-not-compiling/49643
- [B4] *Reactive Stream Processing Patterns*. https://sdpr.rantai.dev/docs/part-vi/chapter-41/
- [B5] *rs2-stream - crates.io*. https://crates.io/crates/rs2-stream

### Rust Type System and Serde

- [A49] *Type system - The Rust Reference*. https://doc.rust-lang.org/reference/attributes/type_system.html
- [A50] *How to ignore unknown enum variant while deserializing? - Stack Overflow*. https://stackoverflow.com/questions/67702612/how-to-ignore-unknown-enum-variant-while-deserializing

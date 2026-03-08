# Zero-Cost LLM Streaming: Architecting Fault-Tolerant Agent Pipelines in Rust and NATS

## Executive Summary

* **[NATS PULL CONSUMERS ENABLE NATIVE BACKPRESSURE]**: While naive SSE chunking risks OOM crashes when LLMs outpace slow tools, NATS JetStream pull consumers using `MaxAckPending` create implicit one-to-one flow control [1]. **Action:** Standardize on JetStream pull consumers rather than push consumers for all inter-agent communication to ensure the system naturally throttles upstream LLM generation when downstream tools stall.
* **[SIMD-ACCELERATED INCREMENTAL PARSING]**: Traditional JSON parsers block until completion, but combining `simd-json` with `serde_json::StreamDeserializer` (using `byte_offset` for error recovery) allows sub-millisecond validation of partial tool-call arguments mid-stream [2] [3]. **Action:** Implement a two-stage parsing pipeline that validates LLM tool arguments incrementally, failing fast on schema hallucinations before the stream even completes.
* **[DYNAMIC SUPERVISION PREVENTS CASCADING FAILURES]**: Borrowing from Erlang's `DynamicSupervisor`, modeling each LLM stream as a transient Rust actor isolates stream panics and network timeouts without crashing the parent planner agent [4] [5]. **Action:** Wrap every LLM stream in a supervised actor (via `ractor` or `bastion`) with a bounded mailbox and an exponential backoff restart policy to guarantee fault isolation.
* **[PROXY BUFFERING DESTROYS SSE LATENCY]**: Default NGINX and Cloudflare configurations buffer SSE responses up to 100KB, completely destroying token-by-token perceived latency for end users [6] [7]. **Action:** Use WebSockets for server-to-server provider links (e.g., OpenAI Realtime API), and strictly configure gateway headers (`X-Accel-Buffering: no`, `proxy_buffering off`) for any client-facing SSE endpoints.
* **[STREAMMAP OUTPERFORMS SELECTALL FOR DYNAMIC FAN-IN]**: When a planner agent spawns multiple sub-agents, Tokio's `SelectAll` suffers from closure type mismatches and allocation overhead, whereas `StreamMap` provides O(1) removal and safe pinning [8] [9]. **Action:** Use `StreamMap` keyed by unique correlation IDs to aggregate concurrent sub-agent streams, ensuring deterministic ordering and fair polling.
* **[TYPED SSE PROTOCOLS PREVENT DESERIALIZATION PANICS]**: Vercel AI SDK's multi-part SSE protocol (`text-delta`, `tool-input-delta`) proves that flat text chunking is insufficient for complex agents [10]. **Action:** Standardize on a rich `ModelEvent` Rust enum using `#[non_exhaustive]` and `#[serde(other)]` to safely ignore unknown future provider features without breaking the deserialization loop.
* **[TAIL LATENCY AMPLIFICATION IN PIPELINES]**: As noted in Google's "Tail at Scale", a 99th percentile slowdown in a single tool-call stream can stall the entire aggregated response if unbounded merges are used [11]. **Action:** Implement `buffer_unordered` with strict timeouts and OpenTelemetry span tracking to shed slow tool responses before they bottleneck the entire multi-agent pipeline.

## 1. Stream Finalization & Incremental Parsing

Transitioning from raw chunks to validated artifacts requires stateful finalizers and SIMD-accelerated partial JSON validation to fail fast on LLM hallucinations.

### Microsoft's `get_final_response` pattern for deterministic stream closure

The Microsoft Agent Framework introduces a dual-consumption pattern for streaming. When an agent runs in streaming mode, it returns a `ResponseStream` object [12]. This object allows developers to asynchronously iterate over chunks for real-time UI updates, and subsequently call `get_final_response()` to retrieve the fully aggregated, schema-validated artifact [12] [13]. This pattern solves the "dangling state" problem by ensuring that the stream's internal finalizer automatically handles structured output parsing once all updates are received [13].

For Mister Smith, this translates to a Rust `Finalizer` struct that wraps a `Stream<Item = ModelEvent>`. As the stream yields `ChunkDelta` variants, the finalizer accumulates state. Upon receiving a `Stop` event, it executes a final `serde` validation pass.

### Sub-millisecond partial JSON validation using `simd-json` and `byte_offset` recovery

Reassembling partial tool call JSON from streaming deltas is notoriously fragile. Anthropic's API, for instance, mixes text content and tool calling, requiring the parser to build the final structure incrementally [14]. Traditional parsers like `serde_json` require the entire JSON text to be read into memory before parsing begins, which spikes memory usage and delays validation [15].

To achieve incremental validation, Mister Smith should leverage `simd-json`, which uses a two-stage parsing model: Stage 1 scans bytes in wide chunks using SIMD instructions to locate structural characters, and Stage 2 builds the parsed representation [3]. However, when using `serde_json::StreamDeserializer` for partial streams, encountering an incomplete JSON fragment causes the deserializer to return the same error in perpetuity (an infinite loop) [2] [16]. The mitigation is to use the `byte_offset()` method to manually advance the buffer past the invalid fragment, or deserialize into a generic `Value` when typed deserialization fails [2].

| Parser / Library | Streaming Capability | Performance / Zero-Copy | Best Use Case in Mister Smith |
| :--- | :--- | :--- | :--- |
| **`serde_json::StreamDeserializer`** | Native iterator over multiple JSON values [17]. | High allocations; blocks on incomplete fragments [15] [2]. | Baseline parsing of complete NDJSON payloads. |
| **`simd-json`** | Tape API and `fill_tape` for incremental consumption [18]. | Multi-GB/s throughput; heavily utilizes `unsafe` SIMD [19] [3]. | High-throughput, zero-copy parsing of large tool arguments. |
| **`json-stream`** | Parses NDJSON from byte streams (e.g., Axum `BodyStream`) [20]. | Minimizes memory by deleting deserialized bytes from the buffer [20]. | Ingesting raw provider SSE streams at the gateway edge. |
| **`oak-json`** | Green/Red Tree architecture for incremental parsing [21]. | Sub-millisecond latency; shares nodes without copying [21]. | AST-level manipulation of deeply nested, evolving tool calls. |

*Key Takeaway*: `simd-json` provides the raw speed needed for token-by-token evaluation, while `json-stream` offers the safest memory profile for long-running streams. Mister Smith should use a hybrid approach: `json-stream` for the outer SSE envelope, and `simd-json` for the inner tool-call argument payloads.

## 2. Reactive Backpressure & Flow Control

True resilience requires end-to-end backpressure, mapping Tokio's bounded channels directly to NATS JetStream's flow control mechanisms.

### Preventing OOM with Tokio bounded `mpsc` and `OwnedPermit`

When an LLM produces tokens faster than a consumer can process them (e.g., a slow database write tool), unbounded queues will eventually fill up all available memory and cause the system to fail unpredictably [22]. Tokio's bounded `mpsc::channel` provides backpressure by forcing the sender to `.await` when the channel reaches capacity [23] [22].

However, naive `select!` loops combining socket reads and channel writes can lead to read starvation. If a `write()` call blocks due to backpressure, the entire loop is blocked, inflating the receive socket buffer and pushing TCP backpressure to the remote peer [24]. To solve this in Mister Smith, use Tokio's `OwnedPermit` [23] [25]. By reserving an `OwnedPermit` *before* pulling the next chunk from the LLM provider, the framework guarantees channel capacity exists, preventing head-of-line blocking and allowing cooperative cancellation.

### NATS JetStream pull consumers with `MaxAckPending` for distributed flow control

While Tokio handles intra-process backpressure, NATS JetStream handles inter-agent distributed flow control. JetStream provides decoupled flow control, meaning publishers are not limited by the slowest consumer [26].

| Consumer Type | Delivery Mechanism | Backpressure / Flow Control Strategy |
| :--- | :--- | :--- |
| **Push Consumer** | Server actively pushes messages to subscribers [27]. | Relies entirely on `MaxAckPending` and sliding-window `FlowControl` [1]. |
| **Pull Consumer** | Client explicitly requests batches via `fetch()` [28] [27]. | Implicit one-to-one flow control driven by client demand [1]. |
| **Ordered Consumer** | Ephemeral, single-threaded dispatch [29]. | Automatic flow control; recreates consumer if a gap is detected [29]. |

*Key Takeaway*: For Mister Smith's token-heavy pipelines, **Pull Consumers** are vastly superior. They allow the agent to fetch messages at its own pace, creating less CPU load on the NATS server and scaling horizontally without complex rebalancing [28] [30]. To prevent redelivery storms when a tool takes too long, configure `MaxAckPending` to limit in-flight messages [1] [31], and use the `Backoff` sequence to override the static `AckWait` with exponential delays [1].

## 3. Concurrency, Multiplexing, & Actor Supervision

Erlang-style supervision trees applied to Rust streams ensure that individual LLM failures do not cascade across the multi-agent orchestration layer.

### O(1) stream aggregation using Tokio `StreamMap` over `SelectAll`

When a Planner agent spawns multiple sub-agents, their streaming responses must be merged. Using `futures::stream::SelectAll` for this is problematic in Rust; it requires uniform stream types, and mapping closures often results in unnameable type mismatches (`expected closure, found a different closure`) [8]. Furthermore, `SelectAll` polls streams in a round-robin fashion [32], which degrades as the number of streams grows.

Instead, Mister Smith should use `tokio_stream::StreamMap` [33]. `StreamMap` combines multiple streams, indexing each with a unique key (e.g., a correlation ID or sub-agent ID) [33]. It requires streams to be `Unpin` (often achieved via `Box::pin`) [9], but provides O(1) insertion and removal, making it ideal for dynamic "stream of streams" fan-in where sub-agents spin up and shut down dynamically. To preserve causality across these merged streams, payloads must include Lamport logical clocks or sequence numbers [34].

### Erlang-style `DynamicSupervisor` for transient stream-actors using `ractor`

In Erlang/OTP, a `DynamicSupervisor` (formerly `simple_one_for_one`) is optimized to start children dynamically on demand, allowing it to hold millions of transient processes without ordering constraints [35] [5]. Modeling a streaming LLM response as a short-lived actor provides immense benefits: it encapsulates the stream's state, provides a dedicated mailbox, and allows a supervisor to restart the stream upon network failure [35] [36].

| Rust Actor Framework | Supervision Capabilities | Mailbox & State Management |
| :--- | :--- | :--- |
| **Actix** | Basic `Supervisor` struct; restarts actors on failure [37] [38]. | Uses mutable self; lacks primitives for non-blocking long-running tasks [4]. |
| **Ractor** | Pure-Rust Erlang `gen_server` clone; full supervision trees [4] [39]. | Separate state type; supports `SupervisionEvent` (startup, death, panic) [4] [39]. |
| **Bastion** | Highly-available, fault-tolerant runtime; dynamic supervision [40]. | `One-For-One` and `All-For-One` strategies; NUMA-aware executor [40]. |

*Key Takeaway*: `Ractor` is the most aligned with Mister Smith's OTP-style requirements. By spawning a `ractor` actor for each LLM stream, Mister Smith can utilize `SupervisionEvent`s to detect stream panics [39]. If a stream fails, the supervisor can apply a `One_For_One` strategy to restart only that specific LLM call, rather than crashing the entire multi-agent workflow [35].

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

 async fn pre_start(&self, _myself: ActorRef<Self::Msg>, args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
 Ok(StreamState { buffer: Vec::new(), correlation_id: args })
 }

 async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, state: &mut Self::State) -> Result<(), ActorProcessingErr> {
 match message {
 ChunkDelta::Text(text) => state.buffer.extend_from_slice(text.as_bytes()),
 ChunkDelta::Stop => { /* Finalize and send to NATS */ }
 _ => {}
 }
 Ok(())
 }
}
```

## 4. Transport Architecture & Event Taxonomy

Extensible, strongly-typed event enums combined with the right transport protocol dictate the reliability of the provider-to-gateway link.

### Bypassing 100KB proxy buffering in SSE via `X-Accel-Buffering: no`

While Server-Sent Events (SSE) are simpler to implement than WebSockets, they suffer from severe intermediary buffering issues. By default, NGINX uses `proxy_buffering on`, which batches chunks and destroys the real-time nature of SSE [7] [41]. Similarly, Cloudflare buffers SSE responses until approximately 100KB accumulates before flushing to the client [6].

To mitigate this, Mister Smith's gateway must explicitly inject the `X-Accel-Buffering: no` header into the application server response [42] [7]. Furthermore, NGINX must be configured with `proxy_buffering off`, `proxy_cache off`, and `chunked_transfer_encoding off` for SSE endpoints [43].

| Transport | Bidirectional | Backpressure Handling | Proxy / CDN Compatibility |
| :--- | :--- | :--- | :--- |
| **SSE (HTTP/1.1)** | No (Server-to-Client only) [44] [45]. | Relies on TCP window; prone to head-of-line blocking [44]. | High, but requires strict header tuning to prevent buffering [43] [7]. |
| **WebSocket** | Yes (Full-duplex) [45]. | Application-level ping/pong; requires manual reconnection [45]. | Requires sticky sessions on ALBs [46]; blocked by some enterprise firewalls [45]. |

*Key Takeaway*: For long-running agentic loops (model <-> tool <-> model), WebSockets are superior for server-to-server communication (e.g., OpenAI Realtime API) because they avoid repeated HTTP handshake overhead and allow bidirectional audio/text streaming [47] [48]. However, for client-facing UI streams, SSE remains the standard, provided the gateway is configured to bypass buffering.

### Future-proofing with `#[non_exhaustive]` Rust enums inspired by Vercel AI SDK

Naive chunk-by-chunk streaming fails when LLMs emit complex metadata. The Vercel AI SDK utilizes a rich Data Stream Protocol over SSE, defining specific event types like `text-delta`, `tool-input-delta`, `tool-output-available`, and `finish-step` [10].

Mister Smith should adopt a similar `ModelEvent` enum in Rust. To ensure forward compatibility when providers add new event types, the enum must use the `#[non_exhaustive]` attribute [49]. Furthermore, using `#[serde(other)]` allows the deserializer to gracefully fallback to an `Unknown` variant rather than panicking when an unrecognized event is received [50].

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ModelEvent {
 TextDelta { id: String, text: String },
 ToolInputDelta { tool_call_id: String, input_text_delta: String },
 FinishStep { finish_reason: String, usage: UsageStats },
 #[serde(other)]
 Unknown,
}
```
Usage updates should be streamed only at completion (`FinishStep`) to save bandwidth, as incremental token counting can be handled client-side [10].

## 5. Production Readiness, Benchmarking, & Observability

Validating the architecture requires chaos engineering and strict SLOs to measure tail latency amplification under heavy concurrent load.

### Mitigating 99th-percentile tail latency amplification in multi-agent pipelines

In distributed agent systems, "Tail at Scale" phenomena dictate that a single slow tool call can bottleneck the entire aggregated response [11]. If a planner agent waits for 3 sub-agents, the overall latency is dictated by the 99th percentile latency of the slowest agent. To combat this, Mister Smith must implement "tied requests" or strict timeouts using `buffer_unordered` [11]. By setting a concurrency limit and allowing items to complete in any order, the system prevents a single slow LLM stream from stalling the event loop.

### Validating throughput with `nats bench` and OpenTelemetry span integration

To prove Mister Smith is architecturally superior, it must be benchmarked rigorously. The `nats bench` CLI tool provides a baseline for JetStream performance, capable of simulating asynchronous publishers and durable pull consumers [51].

| Chaos Engineering Scenario | Injection Method | Expected System Behavior |
| :--- | :--- | :--- |
| **Broker Failure** | `kill -9` on NATS node [52]. | JetStream Raft consensus fails over; latency spikes briefly but recovers [53] [52]. |
| **Slow Consumer** | Artificial `sleep()` in tool execution. | `MaxAckPending` halts LLM ingestion; TCP backpressure applied to provider. |
| **Stream Truncation** | Drop TCP connection mid-SSE. | Actor supervisor detects `Eof` without `Stop` event; triggers `One_For_One` restart [35]. |

To monitor these scenarios, Mister Smith must integrate `tokio-metrics` and OpenTelemetry [54]. By tracking `tokio_tasks_spawned`, `tokio_task_poll_duration`, and `tokio_workers_idle`, operators can detect scheduler starvation and queue depth [54]. Distributed tracing spans should carry the `correlation_id` from the initial user request through the NATS JetStream headers, allowing full visibility into the exact millisecond a specific tool call stalled the pipeline.

## Synthesis: The Optimal Streaming Architecture for Mister Smith

To achieve architectural superiority, Mister Smith must abandon naive SSE chunking in favor of a **Reactive, Actor-Supervised, Pull-Driven Pipeline**:

1. **Base Primitive**: Use Tokio `StreamMap` to aggregate concurrent LLM streams dynamically, avoiding the allocation overhead and type-erasure issues of `SelectAll`.
2. **Distributed Routing**: Utilize NATS JetStream **Pull Consumers** with `MaxAckPending` configured. This translates local Tokio channel backpressure into distributed flow control, naturally slowing down LLM generation when tools stall.
3. **Lifecycle Management**: Wrap every LLM stream in a `ractor` actor. If a stream encounters invalid UTF-8 or a network timeout, the Erlang-style supervisor isolates the panic and restarts the stream using exponential backoff, protecting the parent Planner agent.
4. **Zero-Cost Parsing**: Implement `simd-json` for zero-copy, incremental validation of tool-call arguments, using `byte_offset` to step past incomplete JSON fragments without crashing the deserializer.
5. **Event Taxonomy**: Standardize on a Vercel-inspired `ModelEvent` enum, heavily utilizing `#[non_exhaustive]` and `#[serde(other)]` to ensure the framework never breaks when OpenAI or Anthropic silently update their SSE protocols.

By combining SIMD parsing, OTP supervision, and JetStream pull-mechanics, Mister Smith will deliver predictable p99 latencies and absolute fault isolation under massive concurrent load.

## References

1. *Consumers - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream/consumers
2. *Step past errors in serde_json StreamDeserializer - help - The Rust Programming Language Forum*. https://users.rust-lang.org/t/step-past-errors-in-serde-json-streamdeserializer/84228
3. *Rust : Why SIMD JSON parsing is different (and why simdjson changed the conversation) | by Ajay Kumar | Feb, 2026 | Medium*. https://medium.com/@trivajay259/rust-why-simd-json-parsing-is-different-and-why-simdjson-changed-the-conversation-b170955662b7
4. *Ractor: not just another actor framework : r/rust*. https://www.reddit.com/r/rust/comments/113dp70/ractor_not_just_another_actor_framework
5. *DynamicSupervisor behaviour (Elixir v1.19.5)*. https://hexdocs.pm/elixir/DynamicSupervisor.html
6. *SSE endpoint breaks after recent update – Cloudflare buffers text/event-stream desp - General - Cloudflare Community*. https://community.cloudflare.com/t/sse-endpoint-breaks-after-recent-update-cloudflare-buffers-text-event-stream-desp/810790
7. *For Server-Sent Events (SSE) what Nginx proxy configuration is appropriate? - Server Fault*. https://serverfault.com/questions/801628/for-server-sent-events-sse-what-nginx-proxy-configuration-is-appropriate
8. *Merging Disparate Streams using SelectAll - The Rust Programming Language Forum*. https://users.rust-lang.org/t/merging-disparate-streams-using-selectall/31930
9. *Merge-sorting tokio::io::Lines streams using StreamMap not compiling - help - The Rust Programming Language Forum*. https://users.rust-lang.org/t/merge-sorting-tokio-lines-streams-using-streammap-not-compiling/49643
10. *AI SDK UI: Stream Protocols*. https://ai-sdk.dev/docs/ai-sdk-ui/stream-protocol
11. *The tail at scale*. https://cseweb.ucsd.edu/classes/sp18/cse291-c/post/schedule/p74-dean.pdf
12. *Running Agents | Microsoft Learn*. https://learn.microsoft.com/en-us/agent-framework/agents/running-agents
13. *Producing Structured Output with agents | Microsoft Learn*. https://learn.microsoft.com/en-us/agent-framework/agents/structured-output
14. *Comparing the streaming response structure for different LLM APIs | by Sirsh Amarteifio | Percolation Labs | Medium*. https://medium.com/percolation-labs/comparing-the-streaming-response-structure-for-different-llm-apis-2b8645028b41
15. *Reading JSON sequentially - help - The Rust Programming Language Forum*. https://users.rust-lang.org/t/reading-json-sequentially/57708
16. *Fetched web page*. https://github.com/serde-rs/json/issues/70
17. *Fetched web page*. https://docs.rs/serde_json/latest/serde_json/
18. *Fetched web page*. https://docs.rs/simd-json/latest/simd_json/
19. *Fetched web page*. https://github.com/simd-lite/simd-json
20. *Fetched web page*. https://crates.io/crates/json-stream
21. *oak-json - crates.io: Rust Package Registry*. https://crates.io/crates/oak-json
22. *Channels | Tokio - An asynchronous Rust runtime*. https://tokio.rs/tokio/tutorial/channels
23. *tokio::sync::mpsc - Rust*. https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html
24. *Async Rust with Tokio I/O Streams: Backpressure, Concurrency, and Ergonomics | Viacheslav Biriukov*. https://biriukov.dev/rust-tokio-io/
25. *OwnedPermit in tokio::sync::mpsc - Rust*. https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.OwnedPermit.html
26. *Fetched web page*. https://docs.nats.io/jetstream
27. *How to Build NATS Consumers*. https://oneuptime.com/blog/post/2026-02-02-nats-consumers/view
28. *Consumer Details - NATS Docs*. https://docs.nats.io/using-nats/developer/develop_jetstream/consumers
29. *Fetched web page*. https://docs.nats.io/jetstream/concepts/consumers
30. *Grokking NATS Consumers: Pull-based*. https://www.byronruth.com/grokking-nats-consumers-part-3/
31. *ConsumerConfig in nats::jetstream - Rust*. https://docs.rs/nats/latest/nats/jetstream/struct.ConsumerConfig.html
32. *Fetched web page*. https://docs.rs/futures/latest/futures/stream/index.html
33. *Fetched web page*. https://docs.rs/tokio-stream/latest/tokio_stream/
34. *Time, Clocks, and the Ordering of Events in a Distributed System*. https://lamport.azurewebsites.net/pubs/time-clocks.pdf
35. *Erlang -- Supervisor Behaviour*. https://www.erlang.org/docs/24/design_principles/sup_princ
36. *Fetched web page*. https://erlang.org/doc/reference_manual/processes.html
37. *Supervisor in actix - Rust*. https://docs.rs/actix/latest/actix/struct.Supervisor.html
38. *Odd supervision · Issue #204 · actix/actix · GitHub*. https://github.com/actix/actix/issues/204
39. *Fetched web page*. https://github.com/slawlor/ractor
40. *Fetched web page*. https://github.com/bastion-rs/bastion
41. *Fetched web page*. http://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_buffering
42. *Using Server Sent Events (SSE) with Cloudflare Proxy - Application Performance - Cloudflare Community*. https://community.cloudflare.com/t/using-server-sent-events-sse-with-cloudflare-proxy/656279
43. *How to Configure Server-Sent Events Through Nginx*. https://oneuptime.com/blog/post/2025-12-16-server-sent-events-nginx/view
44. *Fetched web page*. https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
45. *WebSockets vs Server-Sent Events*. https://ably.com/blog/websockets-vs-sse
46. *Which stickiness strategy is right for you? - AWS Prescriptive Guidance*. https://docs.aws.amazon.com/prescriptive-guidance/latest/load-balancer-stickiness/options.html
47. *Realtime API | OpenAI API*. https://developers.openai.com/api/docs/guides/realtime/
48. *Realtime Transport Layer | OpenAI Agents SDK - GitHub Pages*. https://openai.github.io/openai-agents-js/guides/voice-agents/transport/
49. *Type system - The Rust Reference*. https://doc.rust-lang.org/reference/attributes/type_system.html
50. *rust - How to ignore unknown enum variant while deserializing? - Stack Overflow*. https://stackoverflow.com/questions/67702612/how-to-ignore-unknown-enum-variant-while-deserializing
51. *nats bench | NATS Docs*. https://docs.nats.io/using-nats/nats-tools/nats_cli/natsbench
52. *Chaos Engineering for Streaming Systems | Conduktor*. https://www.conduktor.io/glossary/chaos-engineering-for-streaming-systems
53. *nats-general/SECURITY-SELF-ASSESSMENT.md at main · nats-io/nats-general · GitHub*. https://github.com/nats-io/nats-general/blob/main/SECURITY-SELF-ASSESSMENT.md
54. *How to Monitor Tokio Runtime Metrics with OpenTelemetry in Rust*. https://oneuptime.com/blog/post/2026-02-06-monitor-tokio-runtime-metrics-opentelemetry-rust/view

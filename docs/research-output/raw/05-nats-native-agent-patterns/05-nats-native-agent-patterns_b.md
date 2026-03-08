# NATS-Native Architecture for Mister Smith - Phase 9 Research Report

Executive summary: Using NATS (core + JetStream + KV + micro/service patterns) as the primary control plane for LLM-powered agents enables subject-native model routing, low-latency request orchestration, durable conversation memory, live runtime reconfiguration, distributed telemetry, secure multi-tenant isolation, and edge/federated deployment patterns that are difficult to replicate with HTTP alone and that avoid introducing a parallel durable log system in many cases. The recommended end-to-end architecture makes NATS the differentiator for Mister Smith by mapping agent/model/service concepts onto NATS subjects, JetStream streams/consumers, and KV buckets, using queue groups and service discovery for load balancing and failover, and applying JetStream features (replay, retention, republish, rollup) for memory and auditability. Implementation is feasible with async-nats 0.46.0 and existing Phase-4 capabilities; the most significant operational complexity arises from JetStream consumer scale and storage planning. The sections that follow document state of the art, concrete patterns and schemas, async-nats 0.46.0 implementation notes, practical benchmarks and trade-offs versus HTTP and Kafka, an experimental benchmarking plan, an operational/migration playbook, and a prioritized roadmap.

Note on evidence and scope: All factual statements below are drawn from the verified sources listed in References. Where the findings do not specify details (for example Pulsar comparisons or exact LLM tail-latency under specific loads), this report marks those as Evidence Gaps.

---

## 1. NATS for LLM request routing

### State of the art
- NATS uses dot-separated hierarchical subjects for routing and interest management, and supports wildcards for flexible subscriptions and location transparency across routed servers [2], [1].  
- Request-reply in NATS is built on dynamic per-request "inbox" reply subjects; queue groups provide built-in load balancing so only one member receives a request in a group [9], [3].  
- NATS can implement broadcast/scatter-gather by publishing to an inbox and collecting timed replies; this supports speculative/first-response strategies [8].  
- Benchmarks show NATS request-reply microbenchmarks with median microsecond latencies and high throughput; higher-level real-world tests show large advantages over REST in responsiveness and CPU efficiency [14], [15].  
- NATS has patterns and frameworks (micro/service) for service registration and discovery including `$SRV.INFO` and micro stats; service metadata can be attached to registrations for discovery [6], [24], [7].

(Sources for this paragraph: [2], [3], [9], [8], [14], [15], [6], [24], [7].)

### Key techniques and concrete subject patterns
- Model selection via hierarchical subjects: llm.complete.{provider}.{model} where provider instances subscribe to their provider/model subjects; wildcards enable routing to families, e.g., llm.complete.openai.* for all OpenAI models. Use single-level `*` and multi-level `>` wildcards to express flexible interest [1], [34].  
- Queue group load balancing: providers (multiple replicas of a model service) subscribe to llm.complete.openai.gpt-4 queue group to get requests in a balanced, fault-tolerant fashion [3].  
- Dynamic provider registration: providers call the micro service registration ($SRV.INFO / micro) and advertise metadata (tools, GPU capabilities, region, priority) at registration time; discovery clients query `$SRV.INFO` to enumerate available providers and their metadata [6], [24], [7].  
- Speculative (scatter-gather / first-response): implement by publishing the request on a broadcast subject (e.g., llm.complete.broadcast.{request_id}) with a unique reply inbox and aggregating responses for a configured timeout; select the first acceptable response or apply application-level aggregation [8].  
- Geo-affinity and failover: queue groups and NATS supercluster routing route requests locally when responders exist locally, otherwise route to remote regions (geo-affinity) [3], [16].

(Sources for this paragraph: [1], [3], [6], [24], [7], [8], [16].)

### Subject schema example
- llm.complete.{tenant}.{provider}.{model}.{request_type}  
  - Examples: llm.complete.acme.openai.gpt-4.chat, llm.complete.acme.selfhosted.local-llm.stream  
- llm.complete.broadcast.{tenant}.{request_id} - scatter-gather broadcast with reply inbox  
- llm.reply.{tenant}.{request_id} - per-request inbox (system-generated)  
- llm.health.{provider}.{instance} - health pings / status (micro health patterns)

(These patterns use NATS subject hierarchy and wildcard abilities as documented [2], [1], and map to micro/service naming [6].)

### Implementation complexity (async-nats 0.46.0)
- Core request-reply and subscriptions: Low complexity - async-nats provides Connect, Subscribe, Publish, and request APIs suitable for generating reply inboxes and managing subscriptions [10], [11].  
- Queue groups: Low complexity - queue group semantics are implemented server-side and are available via subscribe APIs; async-nats exposes subscriptions as Streams which integrate with Tokio [11].  
- Service registration and metadata: Medium complexity - use NATS micro patterns ($SRV.INFO) via service API building blocks; async-nats provides the primitives but a micro client implementation and metadata encoding are required [24], [10].  
- Scatter-gather aggregation and speculative execution: Medium complexity - requires implementing request aggregation, timeouts and selection logic at the client side (async futures, inbox management) using async-nats request and subscription primitives [9], [11].

(Sources for this paragraph: [10], [11], [24], [9].)

### Expected impact vs HTTP and Kafka
- Latency and CPU: NATS request-reply provides orders-of-magnitude lower request latencies in microbenchmarks and practical comparisons versus REST, enabling sub-millisecond routing overhead for model selection and request dispatching that is negligible compared to LLM TTFT (seconds) [14], [15], [33].  
- Routing expressiveness: subject hierarchies plus wildcards yield more flexible in-network routing than typical HTTP path/host-based systems and avoid creating separate discovery layers. HTTP alternatives require separate load balancers or service discovery systems; Kafka lacks native request-reply and subject semantics [13], [56].

(Sources: [14], [15], [33], [13], [56].)

### Benchmarks & trade-offs to measure
- Measure NATS request-reply p50/p95/p99 and compare to direct HTTP client→provider p50/p95/p99 under identical hardware conditions. Use nats bench and application workloads; nats bench shows microsecond medians in wire tests whereas application-level comparisons versus REST have shown large reductions in median response times [14], [15].  
- Measure speculative execution overhead (scatter→aggregate) by issuing duplicated requests to multiple providers and measuring first-response time and aggregated cost; scatter-gather introduces client aggregation latency proportional to the slowest used responder within the timeout [8].  
- Measure tail latency under client concurrency (simulate many parallel LLM requests) and observe server CPU utilization differences (blogs show NATS lower CPU vs REST for similar workloads) [15].

(Sources: [14], [15], [8].)

---

## 2. JetStream as durable agent memory

### State of the art
- JetStream is NATS’s built-in persistence layer providing durable streams, consumers, replay, retention policies (time, size, count), and multiple replay modes (all, work-queue, interest) [22], [26], [27].  
- JetStream messages include sequence numbers and timestamps and support idempotent publishes via Nats-Msg-Id header; streams may be configured with file or memory storage [16], [22], [38].  
- JetStream consumers can be pull-based with configurable batch sizes, max inflight and PullMaxWaiting settings; managing very large numbers of consumers (100k+) increases Raft/meta-leader load and must be planned [29], [90], [93].  
- The NATS KV store is implemented on top of JetStream streams (KV_<bucket> backing stream) and maps each key to a subject $KV.<bucket>.<key> [99].

(Sources: [22], [16], [26], [29], [93], [99].)

### Modeling conversation logs: stream vs KV
- Append-only conversation log (stream): Model each conversation as messages published to a JetStream stream (e.g., streams.conversations) with subject conversation.{tenant}.{agent}.{conversation_id}. Use JetStream retention policies (max_age, max_bytes, max_msgs) to control storage and use consumers for replay and time-travel debugging [22], [26]. JetStream guarantees ordered delivery within a stream view and supports replay semantics via consumers and durable_name configuration [22], [28].  
- KV for latest state/durable pointers: Use KV buckets for current conversation head, checkpoints, or small durable agent memory entries where immediate-consistency and CAS semantics are needed; KV buckets map to underlying JetStream streams and support watch and history [99], [56], [57]. KV default history size is one (latest value) unless configured otherwise [99].  
- Hybrid pattern (recommended): store each interaction (prompt, response, tokens, metadata) as an append record in JetStream for full audit/tracing, while keeping a KV entry that points to the latest sequence or a compacted summary for fast read/modify operations. The KV pointer (e.g., kv.conversations.latest.{conversation_id} => sequence_id) enables quick random access, while JetStream provides replay and full history [22], [99].

(Sources: [22], [99], [26], [56], [57].)

### Retention, compaction, replay and time-travel debugging
- Retention: configure streams with limits (max_age, max_bytes, max_msgs) and discard policy (Old/New) to control storage growth and retention windows [26], [41].  
- Compaction/rollup: use Nats-Rollup header and AllowRollup option to purge prior messages in a defined scope when a rollup event is published, enabling lightweight compaction semantics [71].  
- Replay/time-travel: create consumers as views on a stream (push or pull) and set durable names to replay from specific sequence numbers or timestamps for debugging and deterministic replay; pull consumers support batched fetch and iteration in async-nats [22], [93].  
- Operational caveat: avoid creating very large numbers of filtered/durable consumers because meta-leader Raft traffic and consumer-info calls can overload the server; consider republish patterns or shared consumers with server-side subject transforms to reduce consumer count [29], [31], [32].

(Sources: [26], [71], [22], [93], [29], [32].)

### Storage / performance / cost implications
- JetStream throughput: memory stores outperform filestores; filestores on some configurations sustain ~250k small messages/sec; synchronous publish (request/ack) increases latency and reduces throughput versus async publishing [99], [40].  
- Idempotency and flow control: asynchronous publish improves throughput but requires flow control to avoid exhausted server resources (limits on in-flight publishes); the telemetry pipeline pattern uses deterministic IDs and retries with dead-letter handling for reliability [99], [66], [68], [69].  
- Cost trade-offs: high retention windows and storing full tokenized interactions will increase disk footprints and JetStream storage; using compaction (rollup) and sensible max_age reduces costs at the expense of longer term full-history availability [26], [22], [69].

(Sources: [99], [40], [66], [68], [69], [26], [22].)

### Implementation complexity (async-nats 0.46.0)
- Creating streams and consumers, publishing and pull consumption are supported by async-nats JetStream APIs; examples include get_or_create_stream, publish, and pull consumer iteration [12], [13], [15], [94].  
- Complexity points: implementing efficient compaction/rollup, deterministic idempotent publishing, and consumer topology that avoids meta-leader overload require operational experience and server configuration tuning; async-nats exposes needed primitives but operators must design consumer counts and republish strategies carefully [29], [99].

(Sources: [12], [13], [15], [94], [29], [99].)

---

## 3. KV store for runtime configuration

### State of the art
- NATS KV is implemented on JetStream and provides Put/Get/Create/Update with metadata including revision numbers, timestamps, and operation types; buckets are backed by streams named KV_<bucket> and keys map to subjects $KV.<bucket>.<key> [99], [21].  
- KV supports watch and watch-all to receive real-time updates; default history size is one unless changed [99], [56], [57].  
- KV provides immediate consistency for monotonic writes/reads, compare-and-swap semantics, and a globally monotonic revision space but direct gets may be served by followers, so read-your-writes is not guaranteed unless reading from the stream leader [99], [5], [6].

(Sources: [99], [56], [57].)

### Key schemas and runtime config patterns
- Provider config bucket: KV bucket `providers` with keys:
  - providers.{tenant}.{provider_id}.manifest -> JSON {models: [...], priority, region, capabilities, health_subject}  
  - providers.{tenant}.{provider_id}.secrets -> encrypted payload with API key transport handled outside NATS (operators decide storage)  
  - providers.{tenant}.routing_policy -> JSON {model_preference_order, weights: {provider:percent}, fallback:[…]}  
- Model mapping bucket: `models.{tenant}.v{version}` -> mapping of logical model alias → provider/model ids; revisions allow safe rollbacks.  
- Checkpoint pointers: `conversations.latest.{tenant}.{conversation_id}` → sequence number (JetStream) for fast resume.

(These schemas map to KV subjects $KV_<bucket>.<key> semantics and standard KV usage patterns [99].)

### Live reconfiguration and consistency guarantees
- KV watches deliver real-time updates to clients subscribed to key or bucket watch streams; operators can use watch events to trigger live reconfiguration without restart [99], [56], [57].  
- Consistency: KV gives immediate consistency for monotonic writes and reads in the logical revision space, but network replication means direct GETs can be served by followers and may not reflect the very latest write unless the client directs requests to the stream leader; design operational flows with revision checks and CAS when strict read-your-writes is required [99], [6].  
- Operator workflows: use staged edits (create new bucket key versions or use versioned model mapping keys), test via watch consumers in staging, then switch routing by updating a single pointer key (atomic Put) and verifying via watches and micro health stats before promoting globally [99], [6].

(Sources: [99], [56], [57], [6].)

### Implementation complexity (async-nats 0.46.0)
- KV primitive support: async-nats exposes JetStream/KeyValue APIs for bucket creation, put/get, watch and history access; building a safe live reconfig pipeline is moderate complexity - KV handles much of the mechanics but operator tooling, encryption/secret handling, and watch handlers must be implemented [12], [99].  
- Operational caveat: to ensure read-your-writes semantics where required, clients may need to target the stream leader or implement revision-based validation around KV Gets; this adds coordination complexity [99].

(Sources: [12], [99].)

---

## 4. NATS for distributed LLM telemetry and auditing

### State of the art
- OpenTelemetry GenAI semantic conventions define attributes for LLM calls (provider, model, token usage metrics) which telemetry systems can emit as spans/metrics [71].  
- JetStream can be used as a durable telemetry ingestion log with append-only storage and acknowledgements; telemetry pipelines using JetStream publish and idempotent deterministic IDs have been implemented in Rust with dead-letter handling and retention windows [22], [66], [67], [68], [69].  
- NATS pub/sub supports fan-out so telemetry events published to subjects can be consumed in real time by aggregation consumers (for metrics) and persisted via JetStream for long-term audit [34], [22].

(Sources: [71], [22], [66], [67], [68], [69], [34].)

### Subject hierarchy and event types
- Suggested subject hierarchy: llm.telemetry.{tenant}.{event_type}.{provider}.{model} where event_type ∈ {request.start, request.end, token.usage, tool.call, error}.  
- For real time metrics aggregation publish lightweight metric events to llm.telemetry.{tenant}.metrics and publish full structured events (request/response bodies, tool calls) to a JetStream-backed subject for durable audit and replay [71], [22], [66].  
- Use deterministic event IDs (UUIDv5 or similar) at publish time to allow idempotent ingestion and deduplication by JetStream [22], [68].

(Sources: [71], [22], [66], [68].)

### Integration with OpenTelemetry/tracing
- Use OpenTelemetry GenAI semantic conventions for span attributes to make LLM traces interoperable across backends; emit spans and also publish telemetry events into NATS for in-network consumption and JetStream persistence where a durable audit trail is required [71], [23].  
- The pipeline approach is complementary to OpenTelemetry exporters: send traces/metrics to OTLP collectors while also publishing events into NATS/JetStream to support in-cluster aggregation and time-travel replay for debugging [71], [22].

(Sources: [71], [22], [23].)

### Implementation complexity (async-nats 0.46.0)
- Publishing telemetry events and structured logs: low complexity - async-nats publish APIs are straightforward; durable JetStream persistence and idempotent publication patterns incur modest complexity (generate deterministic IDs, manage async publish flow control) [12], [99], [68].  
- Aggregation pipelines: medium complexity - building efficient aggregators (token usage, cost rollups) requires consumer topology design and possibly intermediate aggregation services; JetStream consumers with pull batching and appropriate consumer settings support these workloads [93], [90].

(Sources: [12], [99], [68], [93], [90].)

---

## 5. Agent-to-agent streaming and backpressure

### State of the art
- NATS pub/sub allows fan-out delivery and wildcards for flexible routing; JetStream provides durable streaming and consumers with pull/push models and flow control settings such as PullMaxWaiting and max inflight to manage consumer load [34], [93], [90].  
- JetStream async publishing needs flow control to avoid exhausting server resources; consumer configuration and batch/pull settings enable controlled consumption [99], [90].  
- Ordering: NATS and JetStream guarantee ordered delivery within a stream or consumer view; Kafka enforces ordering per partition while NATS orders within stream contexts [72].  
- Real-time pipelines (Kafka/Pulsar) are designed for partitioned throughput and long-term storage; NATS offers a lighter, lower-latency data plane with JetStream providing persistent logs without requiring an entirely separate log cluster [71], [55].

(Sources: [34], [93], [90], [99], [72], [71], [55].)

### Patterns for streaming tokens between agents
- Token streaming subject pattern: llm.stream.{tenant}.{conversation_id}.{producer}.{shard} where producers publish token fragments (small messages) and consumers subscribe with durable pull consumers for replay and ordered processing. For fan-out (pipeline to multiple agents), use pub/sub (non-JetStream) for ephemeral streams or JetStream with interest retention and replicated consumers for durable fan-out [34], [22].  
- Ordering guarantees: use a single stream per conversation (or per producer shard) to ensure ordered delivery; JetStream consumers provide ordered replay while pub/sub (non-persistent) offers best-effort delivery [22], [34].  
- Backpressure strategies:
  - Pull-based consumers: consumers fetch batches on demand, allowing downstream agents to control throughput [93].  
  - Max inflight and PullMaxWaiting: configure JetStream consumer inflight windows and waiting counts to bound resource usage [90].  
  - Async publish flow control: producers should limit in-flight async publishes and monitor PublishAsyncComplete to avoid server overload [99].  
  - WorkQueuePolicy: where parallel processing is required, configure streams with WorkQueuePolicy so messages are removed when acknowledged and work is balanced across consumer instances [22], [15].

(Sources: [34], [22], [93], [90], [99], [15].)

### Comparison with Kafka/Pulsar real-time pipelines
- Throughput vs latency: Kafka partitions provide high throughput and long retention but require partitioning logic for ordering; NATS JetStream delivers ordered views within streams and lower latency for small messages, but scaling extremely large numbers of consumers requires careful design to avoid Raft overhead [72], [29], [40].  
- Backpressure primitives: Kafka is pull-only and uses consumer offsets; JetStream supports both push and pull consumers with configurable inflight semantics and options to implement work queues and replay [53], [22], [93].  
- Practical trade-offs: for low-latency token streaming between agents where message sizes are small and ordering per conversation is required, NATS/JetStream is attractive; for workloads requiring massively partitioned, long-tail storage across many TBs and very large consumer counts, Kafka/Pulsar patterns may be preferable unless JetStream consumer patterns (republish, filtered republish) are used to reduce consumer count [72], [29], [32].

(Sources: [53], [72], [29], [32], [40].)

---

## 6. NATS service mesh and discovery

### State of the art
- The NATS micro framework registers services with NATS for discovery and exposes pings/info/stats subjects; services are discoverable via `$SRV.INFO` and `$SRV.PING.{service}` patterns [24], [6].  
- NATS supports account-based multi-tenant isolation and subject-level authorization, with multiple authentication methods (token, TLS certs, nkeys, JWT) [33], [71].  
- Istio/Linkerd patterns can be mapped to NATS by integrating sidecars for TLS or by mapping mesh identity into NATS authentication and subject mapping for canary/weighted routing [35], [46].

(Sources: [24], [6], [33], [35].)

### Advertising agent capabilities
- Agents register as micro services and include metadata such as tools, supported models, roles, GPU/region tags in service info; discovery clients query `$SRV.INFO` to find candidates and filter by metadata [6], [24], [7].  
- Health checks: use micro PING/INFO and llm.health.{provider}.{instance} subjects to publish liveness and readiness; NATS micro provides nats micro stats for built-in monitoring [24], [7], [88].

(Sources: [6], [24], [7], [88].)

### Service mesh mapping
- Map service mesh concepts to NATS:
  - Service registry/discovery ⇄ micro `$SRV.INFO` and KV registration (services.<service>.<instance>) [6], [86].  
  - Routing policies, canary/weighted routing ⇄ NATS subject mapping and weighted routing config for subject mapping (subject remapping / weights) [98], [99].  
  - Identity/RBAC ⇄ NATS accounts, permission rules and JWTs [33].  
- Escape hatches: sidecars and Istio can be used to enforce mesh-level policies if required, but native NATS accounts + subject permissions provide most service isolation needs [35], [33].

(Sources: [6], [86], [98], [33], [35].)

---

## 7. Edge / federated deployment

### State of the art
- NATS supports adaptive edge deployments with leaf nodes, superclusters and gateways; leaf nodes are lightweight, initiate outbound connections and can continue serving local clients while disconnected, optionally mirroring JetStream streams upstream [21], [74], [75], [74].  
- Leaf nodes support subject remapping via account_mappings enabling site prefixes and local routing semantics [66], [76].  
- Accounts and per-account signing keys provide multi-tenant isolation; credential revocation via JWT revocation lists propagates cluster-wide [21], [77], [28].

(Sources: [21], [74], [75], [66], [76], [77], [28].)

### Federated agent execution patterns
- Deploy local leaf node in each edge site running a small NATS server with local JetStream streams for low-latency interactions and buffering while disconnected; mirror critical telemetry and conversation streams upstream when connectivity allows [21], [23], [22].  
- Multi-tenant isolation: place tenants in separate NATS accounts and map per-tenant subjects and KV buckets to enforce subject boundaries [77].  
- Supercluster topology: connect clusters via gateways for geo distribution and guardrails (geo-affinity routing) so local responders are preferred and cross-region routing occurs only when needed [16], [73].

(Sources: [21], [77], [16], [73], [22].)

### Implementation complexity
- Leaf node and supercluster setups are operational concerns rather than client complexity; clients in async-nats require only correctly configured connection URLs and possible DontRandomize options to prefer low-RTT servers [79], [21]. JetStream mirroring and account mappings necessitate server config and operations expertise [21], [22].

(Sources: [79], [21], [22].)

---

## 8. High-frequency patterns from trading and gaming and their transfer to LLM routing

### State of the art
- HFT systems optimize for microsecond and sub-microsecond message latency using kernel bypass, physical path minimization and specialized hardware; standard OS networking adds 20-50 µs which matters in HFT [36], [37], [35].  
- NATS benchmarks show microsecond median latencies in controlled tests and very high throughput; application tests demonstrate NATS dramatically outperforms REST in request counts and response times [14], [58], [59], [60], [61], [62].  
- Game servers and trading systems rely on similar low-latency message buses, with NATS frequently used where tiny latency and high fan-out are required [17], [70], [71].

(Sources: [36], [37], [35], [14], [58], [59], [60], [61], [62], [17], [70], [71].)

### Transferability to LLM routing
- Difference in timescales: LLM inference calls have TTFT on the order of hundreds of milliseconds to seconds, so routing decisions must be extremely fast (micro/milliseconds) but end-to-end latency is dominated by model inference 
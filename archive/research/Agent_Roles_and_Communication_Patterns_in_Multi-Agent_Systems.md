# Agent Roles and Communication Patterns in Multi-Agent Systems

## Agent Role Taxonomies and Responsibilities

Modern multi-agent frameworks often assign specialized roles to agents to break down complex tasks. A common taxonomy includes:

• **Planner**: Breaks high-level goals into concrete subtasks. For example, a Planner might interpret a user request and formulate a plan or task list.

• **Executor**: Carries out atomic actions or subtasks, such as calling an API, writing code, or retrieving data as directed by the plan.

• **Critic (Evaluator)**: Validates outcomes and critiques results. A Critic agent reviews Executor outputs against the goals or quality criteria, catching errors or suboptimal solutions.

• **Router (Dispatcher)**: Assigns tasks to the appropriate agent(s). This role is essentially a load balancer or coordinator that knows each agent’s specialization and routes subtasks (from the Planner or incoming events) to the right agents. In some designs, the Planner and Router might be combined or the Planner may directly dispatch tasks.

• **Memory (Knowledge Base)**: Stores and retrieves shared knowledge. Not always personified as an “agent,” but many systems include a persistent memory module (or agent) that all other agents can query or update with facts, past states, or intermediate results.

This role specialization has proven efective. Microsoft’s AutoGen framework, for instance, explicitly separates a Planner, multiple Executors, and a Critic in a loop to avoid reasoning bottlenecks.3Anthropic’s open Model-Context-Protocol (MCP) standard similarly defnes planner, executor, critic,memory, and router roles for AI agent collaboration. By having distinct agents “own” planning vs.4doing vs. checking, complex tasks can be decomposed and iteratively refned more reliably. Cornell5researchers note that such task-specialized agents can turn big problems into manageable subtasks,5improving overall performance. In practice, this might look like a Planner agent delegating coding tasks to a Code-Executor agent, which after completion triggers a Test-Executor agent; a Critic agent then verifes outputs, and so on, with a Router ensuring each step goes to the correct specialist.

Real-world example: In an R&D workfow, one could deploy a Researcher agent (Planner) to formulate questions and search strategies, several Searcher agents (Executors) to query diferent data sources in parallel, and a Verifer agent (Critic) to cross-check and consolidate fndings. The system’s Router would distribute each query to an appropriate Searcher and send the collated results back to the Researcher. This approach mirrors how humans divide roles in a team, and it has been reported to yield higher quality results compared to a single monolithic agent.6

## Communication Fabrics Linking Agents

To collaborate, agents need to exchange messages (plans, task results, feedback, etc.). There are several proven communication fabrics for linking agents, each with diferent trade-ofs in coupling, latency, and complexity:

## Direct RPC (Point-to-Point Calls)

In direct messaging, one agent sends a message or invokes a method on another agent directly (e.g. via an HTTP request, gRPC, or a function call if in the same process). This request-reply (RPC) style is straightforward and low-latency between two agents because there’s no intermediary: Agent A knows how to reach Agent B and calls it directly. This pattern is benefcial for real-time needs requiring immediate 7responses – for example, a Planner agent might directly call a Tool agent’s API and wait for the result.Direct RPC is conceptually simple (much like a function call) and often ensures faster point-to-point delivery than going through a middleman.

However, point-to-point RPC tightly couples agents. Each agent must know the address or identity of others,and adding new agents or changing communication patterns can require code changes. There’s also a scalability limitation: in a mesh of many agents, managing numerous direct connections or request pathways becomes complex (the number of possible pairwise connections grows quickly). Reliability can sufer too – if Agent B is down, Agent A’s call fails unless it has retry or fallback logic. There’s no built-in broadcast or group communication; A must send the same message N times to reach N peers. This pattern is essentially synchronous – the caller might block waiting for a response – which can slow down overall throughput if agents form long chains of RPC calls.

Direct RPC Communication: Agent A sends a request directly to Agent B and gets a response (dashed) back. This one-to-one link minimizes hops but requires the caller to know the callee’s address.

In Rust, a simple direct call could be as straightforward as using an HTTP client to call another agent’s REST endpoint, or using an IPC channel if on the same host. For instance, using reqwest :

// Agent A makes a direct HTTP RPC call to Agent B

let response = reqwest::blocking::get("<http://agent-b.local/do_task?param=123>")

.expect("Failed to call Agent B")

.text().unwrap();

println!("Agent A got response: {}", response);

This tightly couples Agent A to Agent B’s URI and protocol. If Agent B is down, this call will fail (unless handled). Despite these drawbacks, direct messaging is low-latency (no broker to go through) and conceptually simple – useful when agent interactions are simple or performance-critical and the network of agents is small/statically defned.

<!-- 2 -->

## Publish/Subscribe Message Bus

A publish/subscribe bus decouples senders and receivers via a mediator (message broker). Agents publish messages to named topics/subjects, and other agents subscribe to those of interest. The bus (e.g. a8NATS server, RabbitMQ, or Kafka) delivers published messages to all subscribers of the topic. This fabric is asynchronous and many-to-many: any agent can broadcast information without needing to know who exactly will consume it. New agents can be added without modifying the senders – they simply subscribe to the relevant topics.

Pub/sub fosters fexibility and scalability. Agents are loosely coupled; they only need to agree on message schemas and topic names. Work can be distributed by topic (for example, a Router agent could publish a "task.planning" message that any idle Planner agent picks up, and Executors could subscribe to "tasks.execute" topics for work in their domain). Because the bus can bufer messages, agents can operate at diferent speeds – a fast publisher won’t overload a slow consumer; messages queue until handled. This model is great for event-driven architectures where agents react to events as they arrive,rather than blocking on each other.

Publish/Subscribe Bus: Multiple agents communicate via a message bus. An agent publishes to a topic, and all subscribed agents receive the message. This decoupling allows broadcast and dynamic agent participation at the cost of an extra network hop.

The trade-ofs: added latency (one extra hop through the broker, and potential routing overhead) and complexity of operating a message broker. Indeed, a distributed pub/sub network can introduce slightly higher end-to-end latency than a direct call because messages must be routed and maybe persisted. On9the other hand, it avoids the explosion of connections in a full mesh and reduces congestion on any single link by spreading communication over the broker’s infrastructure. Fault isolation is improved: if one9subscriber goes down, publishers and other subscribers continue working, unafected by the single failure.The messaging system can also provide durability (saving messages to replay or for agents that were ofine), which direct RPC lacks.

Example: Imagine a centralized Memory agent that updates a knowledge store whenever new info is obtained. Using pub/sub, that Memory agent can subscribe to a topic like "facts.new" ; any agent that learns a fact (Executor, Critic, etc.) simply publishes a message to "facts.new" with the data. They needn’t know or call the Memory agent directly – the bus takes care of delivery. Similarly, a Coordinator agent could broadcast a "task.complete" event that many others hear (updating their state or triggering new actions accordingly).

For autonomous Rust agents, a lightweight bus like NATS is ideal for minimal ceremony. NATS uses subjects (topics) and is extremely fast (sub-millisecond in local networks). Here’s a minimal Rust example using the NATS client to illustrate message fow via pub/sub:

use nats::Connection;

// Connect to a NATS server (could be local or cloud)

let nc: Connection = nats::connect("nats://127.0.0.1:4222").expect("failed to connect");

<!-- 3 -->

// Agent X subscribes to "task.updates" topic

let sub = nc.subscribe("task.updates").expect("subscribe failed");

// Agent Y publishes a message to "task.updates"

nc.publish("task.updates", b"Task42 completed").unwrap();

// Agent X (or any subscriber) can receive the message asynchronously

if let Some(msg) = sub.next() {

println!("Agent X received: {}", String::from_utf8_lossy(&msg.data));

}

In this snippet, Agent Y doesn’t need to know who will pick up the "task.updates" message – any number of agents can be listening. Likewise, Agent X just declares interest and then handles whatever comes. The NATS bus mediates the exchange. This decoupling greatly simplifes multi-agent orchestration:you can add 10 more executor agents and they simply subscribe to the same topics; the publisher (say a Planner) doesn’t change at all. The system becomes event-driven – agents react to published events rather than blocking each other with direct calls. This pattern is recommended for cloud-native deployments,where agents may be separate services or pods: a high-performance broker like NATS or Redis Streams can handle thousands of messages a second with minimal overhead.

### Shared Memory / Blackboard

A blackboard architecture uses a common shared store (a “blackboard”) that all agents can read from and 10write to. Instead of sending messages directly, agents post information, partial results, or requests to this shared memory space. Other agents continuously monitor the blackboard for any data relevant to them, retrieving information or new tasks from it. The blackboard thus serves as a centralized 10coordination hub and single source of truth that every agent can trust for the current state of the problem.

Shared Blackboard Pattern: Agents collaborate via a shared store. Agents A and B post updates to the blackboard,and Agent C retrieves the information. The blackboard acts as a central hub that all agents consult.

This pattern is useful for complex, incremental problem-solving where many agents contribute to a global solution piece by piece. It’s common in AI systems like blackboard problem-solvers or multi-robot systems where, for example, one agent adds a partial solution to the blackboard and another picks it up to refne further. The asynchronous, decoupled nature is similar to pub/sub (in fact, one can implement a blackboard on top of a pub/sub or database), but conceptually the blackboard holds state rather than transient messages. Agents don’t address each other at all – they communicate through the environment (the shared space), a concept akin to “stigmergy” in social insects.

Advantages: This provides maximum decoupling: agents can come and go, or crash, without breaking the chain, as long as the blackboard persists. It also inherently logs history – the blackboard can be a persistent database or even a wiki-like knowledge base that agents update. Fault isolation is decent: a failed agent might leave some data half-written, but other agents can often work around stale data or partial solutions (possibly with a Critic agent fagging inconsistencies).

<!-- 4 -->

Trade-ofs: The blackboard can become a bottleneck. Since it’s centralized, all reads/writes go through it (much like a central coordinator). This can be a scalability limit and a single point of failure if not replicated.Latency can be higher than direct messaging because agents are efectively polling the store or reacting to changes, which might not be instant. Also, implementing synchronization and consistency on a blackboard can be complex – e.g., ensuring two agents don’t both grab the same task from the board, or handling conficts when agents post contradictory data. It’s essentially a global shared memory, so typical distributed systems concerns apply (locking, versioning, etc., if multiple agents write concurrently).

In practice, a shared Redis cache, a distributed flesystem, or an SQL/NoSQL database can serve as a blackboard. For example, agents might all read/write to a specifc database table or a key-value store: a Planner agent writes a list of tasks to the board, and multiple Executor agents poll the board to claim tasks (marking them as in-progress in the store). Upon completion, results are written back. A Monitor or Critic agent might watch the same store for new results to validate. Many workfow systems use this model under the hood, essentially storing the “work queue” in a central durable place.

## Hybrid Models

Real-world systems often combine these communication patterns to get the best of each. Hybrid communication might mean using a direct call for certain interactions and a bus or shared store for others. For instance, an agent could use direct RPC to invoke a compute-intensive tool (to get an immediate result), then publish the result on the bus for other agents to observe. Or a system might use a blackboard for long-lived state (knowledge base, task queue) but also have agents send direct notifcations to each other for time-sensitive triggers.

One common hybrid is a central orchestrator with a pub/sub bus: the orchestrator sends commands via the bus, and workers publish their responses to another topic. This yields a centralized decision-maker (simple logic) but decentralized communication (scalable and fault-tolerant). In fact, the event-driven orchestrator-worker pattern transforms a direct master-worker into a bus-mediated approach for better 1112scalability. Another hybrid approach is hierarchical with pub/sub at each layer: higher-level agents coordinate subagents via topics, and each sub-group of agents might have its own local blackboard or direct RPC calls internally.

Hybrid designs are often necessary “with mixed requirements” – e.g. you need the reliability and decoupling of a bus, but also the low latency of direct calls for specifc high-frequency interactions. Choosing the13mix requires analyzing the task: if every agent needs to share a large context, a central store might be inevitable; if agents mostly work independently but must occasionally synchronize, a pub/sub event (“sync now”) plus local processing may sufce.

The bottom line is that no one fabric is universally best. For cloud-native deployments of autonomous agents (like Rust microservices coordinating Claude-CLI workers), the pub/sub bus approach is often a sweet spot, providing a balance of decoupling and performance. It avoids the tight coupling of direct RPC and the single choke-point of a blackboard. Tools like NATS, Apache Kafka, or RabbitMQ can act as the communication backbone. Meanwhile, a lightweight shared store (even just an in-memory key-value cache or a CRDT-based store) can be layered in for shared memory needs like agent-long-term memory. Minimal ceremony can be achieved by using high-level client libraries for these (as shown in the Rust snippet above for NATS). In sum, hybridizing these patterns lets you tailor the communication topology to the agents’needs – using the bus for most interactions, direct calls for ultra-low-latency tool use, and a shared store for logging and persistence, for example.

<!-- 5 -->

## Coordination Structures: Centralized vs Peer Mesh vs Hierarchical

Beyond message-passing mechanics, how agents are organized (the topology of control and decision making) impacts system latency, fault tolerance, and complexity. The main structures are:

• Centralized Coordination: All agents report to a single central controller/orchestrator, which holds the global state and makes top-level decisions. This resembles a hub-and-spoke model1415– e.g., a master agent assigns tasks to worker agents and integrates their results. The beneft is a global view: the orchestrator can optimize the system’s behavior with full information, and coordination is straightforward since one authority decides and broadcasts commands. Latency for any two agents to indirectly communicate is two hops (Agent -&gt; center -&gt; other Agent), which is usually acceptable and can even be efcient for small systems. In fact, centralized setups often have lower decision latency initially because that single brain can act quickly without negotiation 16. Implementation is simpler (conceptually) – many real multi-agent prototypes start with a central brain because it’s easier to code and reason about.

Drawbacks: This is brittle at scale. The central node is a single point of failure – if the orchestrator 16crashes, the whole system halts. It can become a throughput bottleneck as agent count or message volume grows (every interaction goes through it). Scaling a centralized coordinator usually means vertically scaling that node or sharding responsibilities (which leads toward hierarchical designs anyway). Fault isolation is poor: a bug or overload in the central agent afects everyone. Still, for up to a modest number of agents or very tightly coupled tasks, centralized coordination is often efective. For example, Anthropic’s early multi-agent research system uses a single Lead agent that spawns and manages subagents –17essentially a centralized orchestrator – to ensure the overall research task stays on track.

• Decentralized Peer Mesh: No single leader – every agent is more or less equal, communicating peer-to-peer in a network graph. Decision-making is distributed: each agent makes local decisions based on its view, sometimes via consensus or negotiation protocols with peers. This is akin1819to a team with no manager, or a swarm with emergent coordination. For messaging, this often implies a fully-connected mesh of direct RPC links or a peer-to-peer pub/sub (gossip). The big advantage here is robustness and fault tolerance – there’s no central point to bring down. The20system can continue even if several agents fail, as long as the remaining ones can reroute around them. It’s also naturally scalable in the sense that workload and decision-making load is spread across all agents. If designed well, peer systems can adapt dynamically; they exhibit emergent behavior (e.g., focking, consensus) that is hard-coded in a centralized system.21

Drawbacks: Pure peer meshes are the most complex to design and implement. Without a global controller,ensuring coherent behavior is tricky – you often need sophisticated coordination protocols (voting algorithms, distributed consensus like Raft/Paxos, contract nets for task allocation, etc.). Latency can be22unpredictable: while any two agents could communicate directly in one hop, achieving agreement or broadcasting something to all may require multi-hop message propagation or iterative consensus,increasing latency for global convergence. For instance, a gossip-based peer network reduces central congestion but can increase overall latency for information to reach all nodes. Also, without careful9design, peers can get into conficts or deadlocks if they make inconsistent decisions. Debugging such systems is hard because the logic is emergent from many local interactions.

<!-- 6 -->

In practice, fully decentralized agent swarms are rare in enterprise systems (they’re more common in academic research or specialized domains like blockchain or swarm robotics). That said, some patterns emulate this: e.g., a market-based system where agents bid and negotiate for tasks is efectively peer-to 2324peer coordination via economic principles. This avoids a central scheduler by letting agents fgure out task allocation through an “auction” – improving scalability and fault-isolation, but requiring more complex interaction logic.

• Hierarchical (Tree) Coordination: This is a compromise approach: agents are arranged in a tree or pyramid of authority. A top-level agent oversees high-level goals; it delegates subtasks to mid-level manager agents, which in turn spawn or direct lower-level agents, and so on. Each node in2526the hierarchy may act as an orchestrator for those below it and as a worker (subordinate) to those 27above. This structure is common in human organizations (CEO → managers → staf) and in multi-agent design for complex problems. Latency in a hierarchy is intermediate: communication often travels up or down the tree. Neighboring leaf agents might talk by sending messages up to their common parent and back down, which is two hops – similar to centralized – but not all trafc must go to the root if it’s localized within subtrees. Critical decisions bubble up to higher levels,which may add a bit of delay, but lower-level actions can be taken quickly at the leaves. In terms of fault isolation, a failure in one branch of the tree might be contained to that branch. For example, if a mid-level coordinator dies, only its subtree of agents is afected rather than the whole system. The system can often re-route tasks to a parallel branch or have the parent node spawn a replacement.There’s still some vulnerability at the very top (the root controller), but even that can sometimes be mitigated by redundancy or by having a “backup” agent that can take over if the root fails.

Implementation complexity is moderate: more complex than a single central brain, but simpler than a full peer mesh. Each level deals with a subset of the problem, which makes the logic more manageable. Many multi-agent research systems use a hierarchical approach. For instance, cloud orchestration of agents might have a top-level Orchestrator service, per-team coordinators, and individual task executors.Anthropic’s multi-agent architecture described for their Claude research feature follows this pattern: a Lead Researcher agent spawns multiple Subagent specialists, and even a further specialized CitationAgent for fnal 28processing. The Lead (root) makes global decisions (when to stop researching, how to compile results),while Subagents operate semi-independently on their pieces. This yields a tree of depth 2 or 3.

Hierarchical Coordination Example: Anthropic’s multi-agent research architecture uses a lead agent (top) that delegates to parallel subagents. The lead orchestrator plans and integrates results, while subagents focus on specifc subtasks (e.g., web search, then a citation-fnding agent). This tree-like delegation improves scalability and keeps each agent’s role simple.1728

Hierarchical systems often achieve better scalability than single-hop central control because each layer can handle a portion of the load. They are also easier to scale organizationally: you can improve a subcomponent (say, upgrade the logic of one subgroup of agents) without rewriting the entire system.Latency is typically acceptable because decisions are localized – e.g., low-level agents don’t always need to involve the root for every minor action, only for major re-planning or cross-team coordination.

In summary, centralized vs. decentralized vs. hierarchical involves a classic trade-of triangle between global optimality vs. fault tolerance vs. complexity. Centralized designs give you optimal, globally consistent decisions (and often lower latency per decision) at the cost of resilience – one failure or 16slowdown can derail everything. Fully distributed peer meshes excel in fault tolerance and parallelism (no single bottleneck), but require complex protocols and can sufer higher latencies to reach agreement 9 . Hierarchical designs strike a pragmatic balance: partial centralization at multiple levels contains faults and limits worst-case delays (since not every message goes to a single hub), and the complexity is compartmentalized. Indeed, industry surveys and design guides often recommend hierarchical or hybrid approaches as systems grow, since they “overcome limitations of pure approaches” by combining their strengths.2913

<!-- 7 -->

# Conclusion

When wiring up autonomous agents in practice – for example, Rust-based microservices coordinating large language-model (LLM) tools like Claude through a CLI – these patterns provide a blueprint. A pub/sub bus (e.g. NATS) is often the backbone for cloud-native agent messaging due to its fexibility and performance.Each agent can remain simple (reacting to incoming messages, emitting events) while the bus handles routing and bufering. On top of this, you can introduce hierarchy (a master agent or a few coordinating agents) to impose order when needed, or a shared memory for things like cumulative knowledge that all agents need access to. The key is to match the communication fabric and coordination strategy to your requirements:

• Need ultra-low latency or strict ordering? Direct calls or request-response on the bus (NATS supports request/reply) might be appropriate for those interactions.

• Need loose coupling and scalability? Event streams and pub/sub will serve best, as evidenced by their success in large multi-agent deployments.30

• Need a persistent record of what’s happened or a way for new agents to get up to speed? A shared store (blackboard or event log) provides an authoritative memory that outlives any single agent.

Each agent role (planner, executor, critic, etc.) can be implemented as an independent Rust service or task,all orchestrated through these communication channels. By following proven designs – a centralized orchestrator for high-level decision-making, a hierarchy for scaling out sub-tasks, and a pub/sub bus for the plumbing – you can achieve a robust, fault-tolerant system. Such a system will gracefully handle an agent going down (others keep working via the bus), adapt to new agents being added (just subscribe them to the right topics), and allow each agent to remain focused on its role logic rather than hard-coding network endpoints. Architecture diagrams and logs from real deployments back this up: for instance, Confuent’s event-driven multi-agent patterns showed that switching to a Kafka/NATS-style event log eliminated custom retry logic and made scaling simply a matter of adding consumers. Likewise, developers using NATS1231report that not having to hand-roll messaging (with features like automatic routing and message replay via JetStream) massively simplifed their multi-agent prototypes.3233

In conclusion, by assigning clear roles to agents and linking them with an appropriate communication fabric, we create multi-agent systems that are greater than the sum of their parts. A planner-executor-critic team can iterate towards solutions more reliably than a lone generalist agent, and using a solid messaging backbone (be it pub/sub, RPC, or a blackboard or a mix) ensures these agents remain coordinated with minimal overhead. The evidence from both industry and academia is clear: combining these role taxonomies with the right message-passing patterns leads to systems that are more scalable, lower latency under load, and easier to maintain – key factors when building the next generation of autonomous AI agents.349

<!-- 8 -->

Sources: The insights and patterns above are drawn from recent multi-agent research and industry implementations, including Anthropic’s multi-agent architecture, Microsoft’s AutoGen framework1728, the Model Context Protocol spec, and design guides on event-driven agent systems341253511 . These sources provide real-world evidence of how specialized agent roles and communication models (RPC, pub/sub, blackboards, hybrids) impact system performance and reliability. By referencing these proven approaches, we can confdently design agent orchestration that is both evidence-backed and tuned for autonomous operation.

124 Understanding the AI Agents Through the Lens of MCP | by Eyüp Sercan Uygur | May, 2025 |

Medium

<https://sercanuygur.medium.com/understanding-the-ai-agents-through-the-lens-of-mcp-e62cfdb80950>

36 Agentic RAG systems for enterprise-scale information retrieval

<https://toloka.ai/blog/agentic-rag-systems-for-enterprise-scale-information-retrieval/>

534 Large Language Models – Cornell Statistical Signal Processing Laboratory <https://ssplab.ece.cornell.edu/research/large-language-models/>

7810 Multi-Agent Communication Protocols in Generative AI and Agentic AI: MCP and A2A Protocols Architecture & Governance Magazine

<https://www.architectureandgovernance.com/uncategorized/multi-agent-communication-protocols-in-generative-ai-and-agentic-ai-mcp-and-a2a-protocols/>

9131415161819212229 Centralized vs Distributed AI: Control Paradigms | Galileo <https://galileo.ai/blog/multi-agent-coordination-strategies>

111231 A distributed state of mind: Event-driven multi-agent systems | InfoWorld

<https://www.infoworld.com/article/3808083/a-distributed-state-of-mind-event-driven-multi-agent-systems.html>

1728 How we built our multi-agent research system \ Anthropic

<https://www.anthropic.com/engineering/built-multi-agent-research-system>

20 What is a Multiagent System? | IBM

<https://www.ibm.com/think/topics/multiagent-system>

23242526273035 Four Design Patterns for Event-Driven, Multi-Agent Systems

<https://www.confuent.io/blog/event-driven-multi-agent-systems/>

3233 Go or Rust? Just Listen to the Bots - Cybernetist

<https://cybernetist.com/2024/04/25/go-or-rust-just-listen-to-the-bots/>

<!-- 9 -->

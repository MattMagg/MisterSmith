# Agent Roles and Communication Patterns in Multi-Agent Systems

## Agent Role Taxonomies and Responsibilities

Modern multi-agent frameworks assign specialized roles to agents to break down complex tasks. The standard taxonomy includes:

### Core Agent Roles

**Planner**

- Breaks high-level goals into concrete subtasks
- Interprets user requests and formulates plans/task lists
- Provides strategic direction and task decomposition

**Executor**

- Carries out atomic actions or subtasks
- Calls APIs, writes code, retrieves data as directed by plans
- Implements specific operational capabilities

**Critic (Evaluator)**

- Validates outcomes and critiques results
- Reviews Executor outputs against goals and quality criteria
- Catches errors and identifies suboptimal solutions

**Router (Dispatcher)**

- Assigns tasks to appropriate agents
- Acts as load balancer/coordinator knowing each agent's specialization
- Routes subtasks from Planner or incoming events to correct specialists
- May be combined with Planner in some designs

**Memory (Knowledge Base)**

- Stores and retrieves shared knowledge
- Maintains facts, past states, and intermediate results
- Accessible by all agents for queries and updates
- Not always personified as separate agent

### Framework Examples

**Microsoft AutoGen**: Explicitly separates Planner, multiple Executors, and Critic in loop to avoid reasoning bottlenecks

**Anthropic MCP**: Defines planner, executor, critic, memory, and router roles for AI agent collaboration

**Cornell Research**: Task-specialized agents turn big problems into manageable subtasks, improving overall performance

### Implementation Pattern

Planner → Code-Executor → Test-Executor → Critic → Router (ensures correct specialist handling)

### Real-World Example: R&D Workflow

- **Researcher agent (Planner)**: Formulates questions and search strategies
- **Searcher agents (Executors)**: Query different data sources in parallel
- **Verifier agent (Critic)**: Cross-checks and consolidates findings
- **Router**: Distributes queries to appropriate Searchers, returns collated results

This approach mirrors human team division of roles and yields higher quality results than single monolithic agents.

## Communication Fabrics Linking Agents

To collaborate, agents need to exchange messages (plans, task results, feedback, etc.). There are several proven communication fabrics for linking agents, each with different trade-offs in coupling, latency, and complexity:

### Direct RPC (Point-to-Point Calls)

**Characteristics**

- One agent sends message/invokes method on another agent directly
- Request-reply (RPC) style via HTTP, gRPC, or function calls
- Straightforward and low-latency between two agents
- No intermediary required

**Advantages**

- Conceptually simple (like function call)
- Faster point-to-point delivery than middleman approaches
- Beneficial for real-time needs requiring immediate responses

**Disadvantages**

- Tightly couples agents (each must know others' addresses)
- Adding new agents requires code changes
- Scalability limitation in mesh networks (N² connections)
- No built-in broadcast or group communication
- Synchronous blocking can slow overall throughput
- Single point of failure if target agent is down

**Rust Implementation Example**

```rust
// Agent A makes direct HTTP RPC call to Agent B
let response = reqwest::blocking::get("http://agent-b.local/do_task?param=123")
    .expect("Failed to call Agent B")
    .text().unwrap();
println!("Agent A got response: {}", response);
```

**Use Cases**: Simple agent interactions, performance-critical scenarios, small/static agent networks

### Publish/Subscribe Message Bus

**Characteristics**

- Decouples senders and receivers via message broker
- Agents publish to named topics/subjects, others subscribe to topics of interest
- Asynchronous and many-to-many communication
- Brokers: NATS, RabbitMQ, Kafka

**Advantages**

- Flexibility and scalability through loose coupling
- Work distribution by topic (Router publishes "task.planning", Planners subscribe)
- Message buffering allows different agent speeds
- Event-driven architecture support
- Fault isolation (one subscriber failure doesn't affect others)
- Durability options for message replay

**Disadvantages**

- Added latency (extra hop through broker)
- Complexity of operating message broker
- Potential routing overhead

**NATS Rust Implementation Example**

```rust
use nats::Connection;

// Connect to NATS server
let nc: Connection = nats::connect("nats://127.0.0.1:4222").expect("failed to connect");

// Agent X subscribes to "task.updates" topic
let sub = nc.subscribe("task.updates").expect("subscribe failed");

// Agent Y publishes message to "task.updates"
nc.publish("task.updates", b"Task42 completed").unwrap();

// Agent X receives message asynchronously
if let Some(msg) = sub.next() {
    println!("Agent X received: {}", String::from_utf8_lossy(&msg.data));
}
```

**Use Cases**: Cloud-native deployments, event-driven architectures, scalable agent coordination

### Shared Memory / Blackboard

**Characteristics**

- Common shared store that all agents can read from and write to
- Agents post information, partial results, or requests to shared space
- Other agents monitor blackboard for relevant data
- Centralized coordination hub and single source of truth

**Advantages**

- Maximum decoupling (agents communicate through environment)
- Inherent history logging
- Useful for incremental problem-solving
- Persistent state across agent failures

**Disadvantages**

- Can become bottleneck (centralized reads/writes)
- Single point of failure if not replicated
- Higher latency than direct messaging
- Complex synchronization and consistency requirements
- Global shared memory concerns (locking, versioning)

**Implementation Options**

- Shared Redis cache
- Distributed filesystem
- SQL/NoSQL database
- Key-value store

**Use Cases**: Complex problem-solving, knowledge sharing, work queue management

## Coordination Structures

### Centralized Coordination

**Characteristics**

- Single central controller/orchestrator makes all major decisions
- Hub-and-spoke model with global state view
- Master agent assigns tasks to worker agents

**Advantages**

- Global optimization with full information
- Straightforward coordination
- Lower decision latency initially
- Simple implementation

**Disadvantages**

- Single point of failure
- Throughput bottleneck as system scales
- Poor fault isolation
- Vertical scaling limitations

**Example**: Anthropic's Lead Researcher agent spawning and managing subagents

### Decentralized Peer Mesh

**Characteristics**

- No single leader, agents are equal peers
- Distributed decision-making via consensus/negotiation
- Peer-to-peer communication network

**Advantages**

- Robustness and fault tolerance
- Natural scalability (distributed workload)
- Dynamic adaptation capabilities
- Emergent behavior potential

**Disadvantages**

- Most complex to design and implement
- Sophisticated coordination protocols required
- Unpredictable latency for global convergence
- Potential conflicts and deadlocks
- Difficult debugging

**Use Cases**: Academic research, blockchain, swarm robotics, market-based systems

### Hierarchical (Tree) Coordination

**Characteristics**

- Tree/pyramid structure of authority
- Top-level oversees goals, delegates to mid-level managers
- Each node acts as orchestrator below, worker above

**Advantages**

- Balanced approach between centralized and decentralized
- Fault isolation within branches
- Moderate implementation complexity
- Better scalability than pure centralized
- Localized decision-making

**Disadvantages**

- Still vulnerable at root level
- Communication may require multiple hops
- More complex than centralized approach

**Example**: Anthropic's multi-agent research system with Lead → Subagents → CitationAgent

## Hybrid Models

Real-world systems often combine communication patterns:

**Common Hybrid Patterns**

- Central orchestrator + pub/sub bus
- Direct RPC for tools + bus for coordination
- Blackboard for state + direct notifications for triggers
- Hierarchical with pub/sub at each layer

**Design Considerations**

- Match fabric to requirements (latency vs. reliability vs. scalability)
- Use direct calls for ultra-low-latency interactions
- Use pub/sub for loose coupling and scalability
- Use shared store for persistent memory and late-joining agents

## Conclusion

**Key Principles**

- Assign clear roles to agents (planner, executor, critic, router, memory)
- Link agents with appropriate communication fabric
- Match coordination strategy to system requirements

**Recommended Approach for Rust-based Systems**

- Pub/sub bus (NATS) as backbone for cloud-native messaging
- Hierarchical coordination for scaling
- Shared memory for cumulative knowledge
- Direct calls only for performance-critical interactions

**Benefits**

- Graceful handling of agent failures
- Easy addition of new agents
- Focused agent logic without hard-coded endpoints
- Scalable, fault-tolerant, maintainable systems

The evidence from industry and academia shows that combining role taxonomies with appropriate message-passing patterns leads to systems that are more scalable, lower latency under load, and easier to maintain.

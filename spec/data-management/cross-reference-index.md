# Data Management Cross-Reference Index

## File Relationships

### Core Schema Files

- [core-message-schemas.md](./core-message-schemas.md) - Foundation message schemas
- [workflow-message-schemas.md](./workflow-message-schemas.md) - Task and workflow coordination schemas  
- [system-message-schemas.md](./system-message-schemas.md) - System operation and CLI integration schemas

### Implementation Patterns

- [data-integration-patterns.md](./data-integration-patterns.md) - Message-database integration patterns
- [message-framework.md](./message-framework.md) - Validation and serialization framework
- [agent-communication.md](./agent-communication.md) - Agent communication protocols

### Cross-Directory Dependencies

#### Security Integration

- `../security/` - Authentication and authorization for message handling
- `../security/encryption.md` - Message encryption standards

#### Transport Layer

- `../transport/nats-transport.md` - NATS messaging transport
- `../transport/grpc-transport.md` - gRPC transport protocols

#### Core Architecture

- `../core-architecture/` - System design patterns
- `../core-architecture/agent-lifecycle.md` - Agent lifecycle management

#### Operations

- `../operations/` - Deployment and monitoring
- `../operations/health-monitoring.md` - System health checks

## Message Schema Dependencies

### Schema Inheritance Chain

```
base-message.json (foundation)
    ├── core-message-schemas.md (agent communication)
    ├── workflow-message-schemas.md (task coordination) 
    └── system-message-schemas.md (CLI integration)
```

### Common Type Definitions

All schemas reference `common-types.json` for:

- `agent_id` - Agent identification
- `task_id` - Task tracking  
- `task_status` - Task state enumeration
- `error_details` - Error handling structures
- `execution_metrics` - Performance measurement

## Integration Points

### Workflow Coordination

- Task assignment → Agent communication → Status updates
- Progress tracking → System monitoring → Alert generation
- Error handling → Retry policies → Recovery procedures

### Data Flow

- Message ingestion → Schema validation → Database persistence
- Event sourcing → Read model updates → Query processing
- State synchronization → Conflict resolution → Consistency maintenance

## Navigation Structure

### Sequential Reading Order

1. [Core Message Schemas](./core-message-schemas.md) - Start here for foundational concepts
2. [Workflow Message Schemas](./workflow-message-schemas.md) - Task management patterns
3. [System Message Schemas](./system-message-schemas.md) - CLI and system integration
4. [Message Framework](./message-framework.md) - Implementation framework
5. [Data Integration Patterns](./data-integration-patterns.md) - Advanced integration patterns

### Reference Navigation

- **Core → Workflow**: Task assignment and coordination patterns
- **Workflow → System**: CLI integration for task spawning
- **System → Core**: Health monitoring and agent status
- **Framework → All**: Validation and serialization support
- **Integration → All**: Database persistence for all message types

## Technical Validation

### Schema Validation Chain

```
JSON Schema Definition → Runtime Validation → Type Safety → Serialization
```

### Message Routing Patterns

```
Agent → NATS Subject → Message Handler → Database → Event Store
```

### Error Propagation

```
Validation Error → Error Response → Retry Logic → Compensation Action
```

---

*Cross-Reference Index for Data Management Documentation*

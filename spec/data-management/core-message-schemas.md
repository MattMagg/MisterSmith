# Core Message Schema Definitions

## Agent Framework Message Type System

This document provides essential JSON Schema definitions for core message types in the Mister Smith AI Agent Framework.
It establishes the foundation message patterns for agent communication, task management, and workflow coordination.

## Table of Contents

1. [Foundation Schemas](#1-foundation-schemas)
   - [Base Message Envelope](#11-base-message-envelope)
   - [Common Type Definitions](#12-common-type-definitions)
2. [Agent Communication Messages](#2-agent-communication-messages)
   - [Agent Command Message](#21-agent-command-message)
   - [Agent Status Update Message](#22-agent-status-update-message)
   - [Agent Registration Message](#23-agent-registration-message)

## Integration References

This core message schema specification integrates with:

- [Complete Message Schemas](./message-schemas.md) - Full schema specification
- [Message Framework](./message-framework.md) - Validation and routing framework
- [Transport Layer](../transport/) - NATS and gRPC implementations
- [Agent Lifecycle](../core-architecture/) - Agent management patterns

## 1. Foundation Schemas

### 1.1 Base Message Envelope

All messages in the framework inherit from this base schema to ensure consistent structure and metadata handling. This schema serves as the foundation for:

- [Task and workflow messages](./message-schemas.md#task-management-messages)
- [System operation messages](./message-schemas.md#system-operation-messages)
- [Claude CLI integration messages](./message-schemas.md#claude-cli-integration-messages)

The validation and serialization framework is detailed in [Message Framework](./message-framework.md#validation-framework).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/base-message.json",
  "title": "Base Message Envelope",
  "description": "Universal message envelope for all framework communications",
  "type": "object",
  "required": ["message_id", "timestamp", "schema_version", "message_type"],
  "properties": {
    "message_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique message identifier (UUID v4)"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "Message creation timestamp (ISO 8601 RFC 3339)"
    },
    "schema_version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "description": "Schema version (semantic versioning)",
      "default": "1.0.0"
    },
    "message_type": {
      "type": "string",
      "description": "Discriminator for message type routing and validation"
    },
    "correlation_id": {
      "type": "string",
      "format": "uuid",
      "description": "Optional correlation identifier for request-response patterns"
    },
    "trace_id": {
      "type": "string",
      "format": "uuid",
      "description": "Distributed tracing identifier"
    },
    "source_agent_id": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9_-]+$",
      "description": "Identifier of the sending agent"
    },
    "target_agent_id": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9_-]+$",
      "description": "Identifier of the intended recipient agent"
    },
    "priority": {
      "type": "integer",
      "minimum": 1,
      "maximum": 10,
      "default": 5,
      "description": "Message priority (1=highest, 10=lowest)"
    },
    "reply_to": {
      "type": "string",
      "description": "NATS subject for reply messages"
    },
    "timeout_ms": {
      "type": "integer",
      "minimum": 1000,
      "description": "Message processing timeout in milliseconds"
    },
    "metadata": {
      "type": "object",
      "additionalProperties": {
        "type": "string"
      },
      "description": "Extensible metadata container"
    }
  },
  "additionalProperties": false
}
```

### 1.2 Common Type Definitions

Reusable schema components used across multiple message types. These definitions are referenced extensively in:

- [Workflow orchestration schemas](./message-schemas.md#workflow-orchestration-messages)
- [System health monitoring schemas](./message-schemas.md#system-health-check-message)
- [Agent communication patterns](./message-schemas.md#task-assignment-message)

For implementation guidelines and type generation, see [Code Generation](./message-framework.md#code-generation).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/common-types.json",
  "title": "Common Type Definitions",
  "$defs": {
    "agent_id": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9_-]+$",
      "minLength": 1,
      "maxLength": 64,
      "description": "Agent identifier following naming conventions"
    },
    "task_id": {
      "type": "string",
      "format": "uuid",
      "description": "Task identifier (UUID v4)"
    },
    "agent_status": {
      "type": "string",
      "enum": ["initializing", "idle", "busy", "paused", "error", "stopping", "terminated"],
      "description": "Agent operational status"
    },
    "task_status": {
      "type": "string",
      "enum": ["pending", "assigned", "running", "completed", "failed", "cancelled", "timeout"],
      "description": "Task execution status"
    },
    "capability": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9-]*[a-z0-9]$",
      "description": "Agent capability identifier (kebab-case)"
    },
    "resource_usage": {
      "type": "object",
      "properties": {
        "cpu_percent": {
          "type": "number",
          "minimum": 0,
          "maximum": 100,
          "description": "CPU utilization percentage"
        },
        "memory_mb": {
          "type": "number",
          "minimum": 0,
          "description": "Memory usage in megabytes"
        },
        "disk_mb": {
          "type": "number",
          "minimum": 0,
          "description": "Disk usage in megabytes"
        },
        "network_connections": {
          "type": "integer",
          "minimum": 0,
          "description": "Active network connections"
        }
      },
      "additionalProperties": false
    },
    "error_details": {
      "type": "object",
      "required": ["error_code", "error_message"],
      "properties": {
        "error_code": {
          "type": "string",
          "pattern": "^E\\d{4}$",
          "description": "Standardized error code (E####)"
        },
        "error_message": {
          "type": "string",
          "maxLength": 2048,
          "description": "Human-readable error description"
        },
        "stack_trace": {
          "type": "string",
          "description": "Technical stack trace for debugging"
        },
        "retry_count": {
          "type": "integer",
          "minimum": 0,
          "description": "Number of retry attempts"
        },
        "recoverable": {
          "type": "boolean",
          "description": "Whether the error is potentially recoverable"
        },
        "context": {
          "type": "object",
          "description": "Additional error context"
        }
      },
      "additionalProperties": false
    },
    "execution_metrics": {
      "type": "object",
      "properties": {
        "execution_time_ms": {
          "type": "integer",
          "minimum": 0,
          "description": "Total execution time in milliseconds"
        },
        "cpu_time_ms": {
          "type": "integer",
          "minimum": 0,
          "description": "CPU time consumed in milliseconds"
        },
        "memory_peak_mb": {
          "type": "number",
          "minimum": 0,
          "description": "Peak memory usage during execution"
        },
        "io_operations": {
          "type": "integer",
          "minimum": 0,
          "description": "Number of I/O operations performed"
        },
        "network_bytes_sent": {
          "type": "integer",
          "minimum": 0,
          "description": "Network bytes transmitted"
        },
        "network_bytes_received": {
          "type": "integer",
          "minimum": 0,
          "description": "Network bytes received"
        }
      },
      "additionalProperties": false
    }
  }
}
```

## 2. Agent Communication Messages

### 2.1 Agent Command Message

Schema for commanding agents to perform operations. This message type integrates with:

- [Task assignment workflows](./message-schemas.md#task-assignment-message) for execution commands
- [Claude CLI hook events](./message-schemas.md#hook-event-message) for tool execution
- [Agent lifecycle management](../core-architecture/) for operational commands

For validation rules and error handling, see [Validation Framework](./message-schemas.md#validation-framework).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/agent-command.json",
  "title": "Agent Command Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "agent_command" },
    "payload": {
      "type": "object",
      "required": ["command_type"],
      "properties": {
        "command_type": {
          "type": "string",
          "enum": ["execute", "stop", "pause", "resume", "configure", "shutdown", "restart"],
          "description": "Type of command to execute"
        },
        "command_data": {
          "type": "object",
          "description": "Command-specific data payload"
        },
        "parameters": {
          "type": "object",
          "additionalProperties": true,
          "description": "Command execution parameters"
        },
        "require_confirmation": {
          "type": "boolean",
          "default": false,
          "description": "Whether command requires explicit confirmation"
        },
        "idempotency_key": {
          "type": "string",
          "description": "Key for idempotent command execution"
        }
      },
      "additionalProperties": false,
      "if": {
        "properties": {
          "command_type": { "const": "execute" }
        }
      },
      "then": {
        "properties": {
          "command_data": {
            "type": "object",
            "required": ["task"],
            "properties": {
              "task": { "$ref": "common-types.json#/$defs/task_id" },
              "context": { "type": "object" },
              "deadline": {
                "type": "string",
                "format": "date-time"
              }
            }
          }
        }
      }
    }
  },
  "additionalProperties": false
}
```

### 2.2 Agent Status Update Message

Schema for agent status reporting and health monitoring. This schema connects to:

- [System health check messages](./message-schemas.md#system-health-check-message) for infrastructure monitoring
- [Task progress updates](./message-schemas.md#task-progress-update-message) for execution status
- [Agent operations documentation](../operations/) for operational procedures

Health monitoring patterns are detailed in [System Alert Messages](./message-schemas.md#system-alert-message).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/agent-status.json",
  "title": "Agent Status Update Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "agent_status" },
    "payload": {
      "type": "object",
      "required": ["agent_id", "status", "uptime_seconds"],
      "properties": {
        "agent_id": { "$ref": "common-types.json#/$defs/agent_id" },
        "status": { "$ref": "common-types.json#/$defs/agent_status" },
        "previous_status": { "$ref": "common-types.json#/$defs/agent_status" },
        "uptime_seconds": {
          "type": "integer",
          "minimum": 0,
          "description": "Agent uptime in seconds"
        },
        "capacity": {
          "type": "number",
          "minimum": 0.0,
          "maximum": 1.0,
          "description": "Current capacity utilization (0.0-1.0)"
        },
        "current_tasks": {
          "type": "integer",
          "minimum": 0,
          "description": "Number of currently executing tasks"
        },
        "max_tasks": {
          "type": "integer",
          "minimum": 1,
          "description": "Maximum concurrent task capacity"
        },
        "capabilities": {
          "type": "array",
          "items": { "$ref": "common-types.json#/$defs/capability" },
          "uniqueItems": true,
          "description": "Agent capabilities list"
        },
        "resource_usage": { "$ref": "common-types.json#/$defs/resource_usage" },
        "last_heartbeat": {
          "type": "string",
          "format": "date-time",
          "description": "Last heartbeat timestamp"
        },
        "health_check_results": {
          "type": "object",
          "properties": {
            "overall_health": {
              "type": "string",
              "enum": ["healthy", "degraded", "unhealthy"],
              "description": "Overall health status"
            },
            "component_health": {
              "type": "object",
              "patternProperties": {
                "^[a-zA-Z0-9_-]+$": {
                  "type": "string",
                  "enum": ["healthy", "degraded", "unhealthy"]
                }
              },
              "description": "Per-component health status"
            }
          }
        },
        "error_details": { "$ref": "common-types.json#/$defs/error_details" }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

### 2.3 Agent Registration Message

Schema for agent registration and capability announcement. Registration integrates with:

- [Agent lifecycle management](./agent-lifecycle.md) for registration workflows
- [Hook response messages](./message-schemas.md#hook-response-message) for agent spawning
- [Workflow coordination](./message-schemas.md#workflow-coordination-message) for capability discovery

For implementation patterns, see [Agent Integration](./agent-integration.md) and [Message Framework Security](./message-framework.md#security-considerations).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/agent-registration.json",
  "title": "Agent Registration Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "agent_registration" },
    "payload": {
      "type": "object",
      "required": ["agent_id", "agent_type", "capabilities", "max_tasks"],
      "properties": {
        "agent_id": { "$ref": "common-types.json#/$defs/agent_id" },
        "agent_type": {
          "type": "string",
          "enum": ["supervisor", "worker", "coordinator", "monitor", "planner", "executor", "critic", "router", "memory"],
          "description": "Agent type classification"
        },
        "agent_version": {
          "type": "string",
          "pattern": "^\\d+\\.\\d+\\.\\d+$",
          "description": "Agent implementation version"
        },
        "capabilities": {
          "type": "array",
          "items": { "$ref": "common-types.json#/$defs/capability" },
          "minItems": 1,
          "uniqueItems": true,
          "description": "Agent capabilities list"
        },
        "max_tasks": {
          "type": "integer",
          "minimum": 1,
          "maximum": 1000,
          "description": "Maximum concurrent task capacity"
        },
        "resource_requirements": {
          "type": "object",
          "properties": {
            "min_memory_mb": {
              "type": "integer",
              "minimum": 256,
              "description": "Minimum memory requirement"
            },
            "max_memory_mb": {
              "type": "integer",
              "description": "Maximum memory limit"
            },
            "cpu_cores": {
              "type": "number",
              "minimum": 0.1,
              "description": "CPU core requirement"
            }
          }
        },
        "configuration": {
          "type": "object",
          "description": "Agent-specific configuration parameters"
        },
        "endpoints": {
          "type": "object",
          "properties": {
            "nats_subjects": {
              "type": "array",
              "items": { "type": "string" },
              "description": "NATS subjects the agent subscribes to"
            },
            "grpc_address": {
              "type": "string",
              "description": "gRPC service address"
            },
            "http_address": {
              "type": "string",
              "description": "HTTP API address"
            }
          }
        },
        "supervision": {
          "type": "object",
          "properties": {
            "supervisor_id": { "$ref": "common-types.json#/$defs/agent_id" },
            "restart_policy": {
              "type": "string",
              "enum": ["always", "on-failure", "never"],
              "default": "on-failure"
            },
            "health_check_interval_ms": {
              "type": "integer",
              "minimum": 1000,
              "default": 30000
            }
          }
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

---

## Schema Relationships

### Core Dependencies

- **Foundation for**: [Complete Message Schemas](./message-schemas.md)
- **Extended by**: Task management, health monitoring, and CLI integration schemas
- **Common types used in**: All message schemas throughout the framework

### Integration Points

- **Agent Operations**: [Agent Communication](./agent-communication.md), [Agent Lifecycle](./agent-lifecycle.md)
- **Validation**: [Message Framework](./message-framework.md#validation-framework)
- **Security**: [Framework Security](./message-framework.md#security-considerations)
- **Performance**: [Optimization Strategies](./message-framework.md#performance-optimization)

## Navigation

This file is part of the Message Schema Documentation suite:

1. **[Core Message Schemas](./core-message-schemas.md)** - Foundation schemas and agent communication *(current file)*
2. [Complete Message Schemas](./message-schemas.md) - Full specification with all message types
4. [Message Framework](./message-framework.md) - Validation, serialization, and framework specifications

### Quick Access

- **Task Management**: [Task Assignment](./message-schemas.md#task-assignment-message), [Task Results](./message-schemas.md#task-result-message)
- **System Operations**: [Health Checks](./message-schemas.md#system-health-check-message), [Alerts](./message-schemas.md#system-alert-message)
- **Claude CLI**: [Hook Events](./message-schemas.md#hook-event-message), [Hook Responses](./message-schemas.md#hook-response-message)
- **Framework**: [Validation](./message-framework.md#validation-framework), [Serialization](./message-framework.md#serialization-specifications)

For the complete framework documentation, see the [Data Management Index](./CLAUDE.md).

*Message Schema Definitions v1.0.0 - Mister Smith AI Agent Framework*

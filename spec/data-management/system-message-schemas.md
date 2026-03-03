# System Message Schemas

## Technical Specifications for Claude CLI Integration and System Operations

**🔍 AGENT OPTIMIZATION STATUS**

- **Agent**: Agent 7, Team Beta  
- **Optimization Date**: 2025-07-07
- **Target**: Remove business content, enhance framework integration
- **Status**: OPTIMIZING
- **Focus**: CLI integration patterns, system operation schemas, routing specifications

**Purpose**: This document defines technical message schemas for Claude CLI integration, system operations, and NATS-based message routing within the Mister Smith AI Agent Framework.

## Overview

This file contains schemas for:

- **Claude CLI Integration Messages** - Hook events and responses for CLI integration
- **System Operation Messages** - System alerts and health monitoring
- **Message Routing and Addressing** - NATS subject patterns and routing rules

These schemas enable seamless integration with Claude CLI and provide comprehensive system monitoring capabilities. They build upon:

- [Foundation schemas](./core-message-schemas.md#foundation-schemas) for basic message structure
- [Workflow coordination](./workflow-message-schemas.md#workflow-orchestration-messages) for multi-agent operations
- [Message framework](./message-framework.md) for validation and routing patterns

## 5. Claude CLI Integration Messages

### 5.1 Hook Event Message

Schema for Claude CLI hook events and integration points. Hook events integrate with:

- [Agent command messages](./core-message-schemas.md#agent-command-message) for tool execution
- [Task assignment workflows](./workflow-message-schemas.md#task-assignment-message) for parallel agent spawning
- [Agent registration](./core-message-schemas.md#agent-registration-message) for capability-based spawning
- [Workflow coordination](./workflow-message-schemas.md#workflow-coordination-message) for multi-agent orchestration

For implementation details, see [Agent Integration](./agent-integration.md) and [CLI research](../research/claude-cli-integration/).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/hook-event.json",
  "title": "Claude CLI Hook Event Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "hook_event" },
    "payload": {
      "type": "object",
      "required": ["hook_type", "session_info"],
      "properties": {
        "hook_type": {
          "type": "string",
          "enum": ["startup", "pre_task", "post_task", "on_error", "on_file_change", "shutdown"],
          "description": "Type of hook event"
        },
        "tool_name": {
          "type": "string",
          "description": "Name of the tool being executed (for task hooks)"
        },
        "tool_input": {
          "type": "object",
          "description": "Tool input parameters (for task hooks)"
        },
        "tool_response": {
          "type": "object",
          "description": "Tool execution response (for post-task hooks)"
        },
        "session_info": {
          "type": "object",
          "required": ["session_id", "model", "start_time"],
          "properties": {
            "session_id": {
              "type": "string",
              "format": "uuid",
              "description": "Claude CLI session identifier"
            },
            "model": {
              "type": "string",
              "description": "Claude model being used"
            },
            "start_time": {
              "type": "string",
              "format": "date-time",
              "description": "Session start time"
            },
            "user_id": {
              "type": "string",
              "description": "User identifier"
            },
            "workspace_path": {
              "type": "string",
              "description": "Working directory path"
            }
          },
          "additionalProperties": false
        },
        "context_id": {
          "type": "string",
          "format": "uuid",
          "description": "Contextual grouping identifier"
        },
        "file_changes": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["file_path", "change_type"],
            "properties": {
              "file_path": {
                "type": "string",
                "description": "Path to changed file"
              },
              "change_type": {
                "type": "string",
                "enum": ["created", "modified", "deleted", "renamed"],
                "description": "Type of file change"
              },
              "old_path": {
                "type": "string",
                "description": "Previous path (for renamed files)"
              },
              "content_preview": {
                "type": "string",
                "maxLength": 2048,
                "description": "Preview of file content changes"
              }
            }
          },
          "description": "File changes (for file change hooks)"
        },
        "error_context": {
          "type": "object",
          "properties": {
            "error_message": {
              "type": "string",
              "description": "Error message"
            },
            "stack_trace": {
              "type": "string",
              "description": "Error stack trace"
            },
            "recovery_suggestions": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Suggested recovery actions"
            }
          },
          "description": "Error context (for error hooks)"
        },
        "agent_spawn_request": {
          "type": "object",
          "properties": {
            "requested_agents": {
              "type": "array",
              "items": {
                "type": "object",
                "required": ["agent_type", "capabilities"],
                "properties": {
                  "agent_type": {
                    "type": "string",
                    "enum": ["planner", "executor", "critic", "router", "memory", "coordinator"]
                  },
                  "capabilities": {
                    "type": "array",
                    "items": { "$ref": "common-types.json#/$defs/capability" }
                  },
                  "configuration": { "type": "object" },
                  "priority": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 5
                  }
                }
              },
              "description": "Agents requested for parallel execution"
            },
            "coordination_strategy": {
              "type": "string",
              "enum": ["parallel", "sequential", "pipeline", "hierarchical"],
              "default": "parallel",
              "description": "How agents should be coordinated"
            }
          },
          "description": "Request for parallel agent spawning"
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

### 5.2 Hook Response Message

Schema for responding to Claude CLI hook events. Hook responses coordinate with:

- [Agent status updates](./core-message-schemas.md#agent-status-update-message) for spawned agent reporting
- [Task result messages](./workflow-message-schemas.md#task-result-message) for parallel execution results
- [System health checks](./system-message-schemas.md#system-health-check-message) for infrastructure validation
- [Message transformation](./message-framework.md#message-transformation-patterns) for CLI response formatting

See [Agent Operations](./agent-operations.md) for response handling patterns.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/hook-response.json",
  "title": "Claude CLI Hook Response Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "hook_response" },
    "payload": {
      "type": "object",
      "required": ["hook_event_id", "decision"],
      "properties": {
        "hook_event_id": {
          "type": "string",
          "format": "uuid",
          "description": "ID of the hook event being responded to"
        },
        "decision": {
          "type": "string",
          "enum": ["approve", "block", "continue", "modify"],
          "description": "Hook processing decision"
        },
        "reason": {
          "type": "string",
          "maxLength": 1024,
          "description": "Reason for the decision"
        },
        "continue": {
          "type": "boolean",
          "default": true,
          "description": "Whether to continue normal execution"
        },
        "stop_reason": {
          "type": "string",
          "description": "Reason for stopping execution (if continue=false)"
        },
        "modifications": {
          "type": "object",
          "properties": {
            "tool_input_changes": {
              "type": "object",
              "description": "Modifications to tool input parameters"
            },
            "additional_context": {
              "type": "object",
              "description": "Additional context to inject"
            },
            "environment_changes": {
              "type": "object",
              "description": "Environment variable modifications"
            },
            "configuration_overrides": {
              "type": "object",
              "description": "Configuration parameter overrides"
            }
          },
          "description": "Modifications to apply (for modify decision)"
        },
        "spawned_agents": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["agent_id", "agent_type", "status"],
            "properties": {
              "agent_id": { "$ref": "common-types.json#/$defs/agent_id" },
              "agent_type": {
                "type": "string",
                "enum": ["planner", "executor", "critic", "router", "memory", "coordinator"]
              },
              "status": {
                "type": "string",
                "enum": ["spawning", "ready", "failed"]
              },
              "endpoints": {
                "type": "object",
                "properties": {
                  "nats_subject": { "type": "string" },
                  "grpc_address": { "type": "string" },
                  "http_address": { "type": "string" }
                }
              },
              "error_details": { "$ref": "common-types.json#/$defs/error_details" }
            }
          },
          "description": "Information about spawned agents"
        },
        "execution_context": {
          "type": "object",
          "properties": {
            "parallel_task_results": {
              "type": "object",
              "patternProperties": {
                "^[a-zA-Z0-9_-]+$": {
                  "type": "object",
                  "properties": {
                    "status": { "$ref": "common-types.json#/$defs/task_status" },
                    "result": { "type": "object" },
                    "metrics": { "$ref": "common-types.json#/$defs/execution_metrics" }
                  }
                }
              },
              "description": "Results from parallel agent execution"
            },
            "coordination_summary": {
              "type": "object",
              "properties": {
                "total_agents": { "type": "integer" },
                "successful_agents": { "type": "integer" },
                "failed_agents": { "type": "integer" },
                "total_execution_time_ms": { "type": "integer" }
              }
            }
          },
          "description": "Context from parallel execution"
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

## 6. System Operation Messages

### 6.1 System Alert Message

Schema for system alerts and notifications. System alerts integrate with:

- [Agent status monitoring](./core-message-schemas.md#agent-status-update-message) for health-based alerts
- [Task progress tracking](./workflow-message-schemas.md#task-progress-update-message) for execution alerts
- [Workflow coordination](./workflow-message-schemas.md#workflow-coordination-message) for orchestration failures
- [Error classification](./message-framework.md#error-code-classification) for severity mapping

For alert escalation patterns, see [Security monitoring](../security/security-patterns.md).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/system-alert.json",
  "title": "System Alert Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "system_alert" },
    "payload": {
      "type": "object",
      "required": ["alert_id", "severity", "component", "alert_type", "alert_message"],
      "properties": {
        "alert_id": {
          "type": "string",
          "format": "uuid",
          "description": "Unique alert identifier"
        },
        "severity": {
          "type": "string",
          "enum": ["info", "warning", "error", "critical", "emergency"],
          "description": "Alert severity level"
        },
        "component": {
          "type": "string",
          "description": "System component that generated the alert"
        },
        "alert_type": {
          "type": "string",
          "enum": ["health", "performance", "security", "resource", "operational", "business"],
          "description": "Category of alert"
        },
        "alert_message": {
          "type": "string",
          "maxLength": 2048,
          "description": "Human-readable alert message"
        },
        "details": {
          "type": "object",
          "description": "Additional alert details and context"
        },
        "affected_agents": {
          "type": "array",
          "items": { "$ref": "common-types.json#/$defs/agent_id" },
          "description": "Agents affected by this alert"
        },
        "metrics": {
          "type": "object",
          "patternProperties": {
            "^[a-zA-Z0-9_.-]+$": {
              "type": "number"
            }
          },
          "description": "Relevant metrics at time of alert"
        },
        "resolution_steps": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["step", "description"],
            "properties": {
              "step": { "type": "integer" },
              "description": { "type": "string" },
              "automated": { "type": "boolean" },
              "estimated_time_minutes": { "type": "integer" }
            }
          },
          "description": "Suggested resolution steps"
        },
        "auto_resolution": {
          "type": "object",
          "properties": {
            "enabled": { "type": "boolean" },
            "action": {
              "type": "string",
              "enum": ["restart", "scale", "failover", "throttle", "ignore"]
            },
            "parameters": { "type": "object" }
          },
          "description": "Automatic resolution configuration"
        },
        "escalation": {
          "type": "object",
          "properties": {
            "escalate_after_minutes": { "type": "integer" },
            "escalation_targets": {
              "type": "array",
              "items": { "type": "string" }
            },
            "notification_channels": {
              "type": "array",
              "items": {
                "type": "string",
                "enum": ["email", "slack", "webhook", "sms", "pager"]
              }
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

### 6.2 System Health Check Message

Schema for system health monitoring and reporting. Health monitoring connects to:

- [Agent registration data](./core-message-schemas.md#agent-registration-message) for capacity tracking
- [Task assignment validation](./workflow-message-schemas.md#task-assignment-message) for resource requirements
- [Workflow state sync](./workflow-message-schemas.md#workflow-state-synchronization-message) for infrastructure status
- [Performance optimization](./message-framework.md#performance-optimization) for system tuning

Implementation details are in [Connection Management](./connection-management.md) and [Storage Patterns](./storage-patterns.md).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/system-health-check.json",
  "title": "System Health Check Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "system_health_check" },
    "payload": {
      "type": "object",
      "required": ["check_id", "check_type", "overall_status", "components"],
      "properties": {
        "check_id": {
          "type": "string",
          "format": "uuid",
          "description": "Health check instance identifier"
        },
        "check_type": {
          "type": "string",
          "enum": ["scheduled", "on_demand", "triggered", "startup", "shutdown"],
          "description": "Type of health check"
        },
        "overall_status": {
          "type": "string",
          "enum": ["healthy", "degraded", "unhealthy", "unknown"],
          "description": "Overall system health status"
        },
        "components": {
          "type": "object",
          "patternProperties": {
            "^[a-zA-Z0-9_-]+$": {
              "type": "object",
              "required": ["status"],
              "properties": {
                "status": {
                  "type": "string",
                  "enum": ["healthy", "degraded", "unhealthy", "unknown"]
                },
                "response_time_ms": {
                  "type": "number",
                  "minimum": 0,
                  "description": "Component response time"
                },
                "last_check": {
                  "type": "string",
                  "format": "date-time"
                },
                "error_details": { "$ref": "common-types.json#/$defs/error_details" },
                "metrics": {
                  "type": "object",
                  "properties": {
                    "uptime_seconds": { "type": "integer" },
                    "request_rate": { "type": "number" },
                    "error_rate": { "type": "number" },
                    "latency_p95_ms": { "type": "number" }
                  }
                },
                "dependencies": {
                  "type": "array",
                  "items": { "type": "string" },
                  "description": "Component dependencies"
                }
              }
            }
          },
          "description": "Per-component health status"
        },
        "infrastructure": {
          "type": "object",
          "properties": {
            "nats_cluster": {
              "type": "object",
              "properties": {
                "cluster_size": { "type": "integer" },
                "healthy_nodes": { "type": "integer" },
                "leader_node": { "type": "string" },
                "message_rate": { "type": "number" },
                "storage_usage_percent": { "type": "number" }
              }
            },
            "database": {
              "type": "object",
              "properties": {
                "connection_pool_status": { "type": "string" },
                "active_connections": { "type": "integer" },
                "query_latency_ms": { "type": "number" },
                "storage_usage_percent": { "type": "number" }
              }
            },
            "resources": {
              "type": "object",
              "properties": {
                "cpu_usage_percent": { "type": "number" },
                "memory_usage_percent": { "type": "number" },
                "disk_usage_percent": { "type": "number" },
                "network_utilization_percent": { "type": "number" }
              }
            }
          }
        },
        "sla_compliance": {
          "type": "object",
          "properties": {
            "availability_percent": {
              "type": "number",
              "minimum": 0,
              "maximum": 100
            },
            "response_time_sla_ms": { "type": "number" },
            "error_rate_threshold_percent": { "type": "number" },
            "compliance_status": {
              "type": "string",
              "enum": ["compliant", "at_risk", "breached"]
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

## 9. Message Routing and Addressing

### 9.1 NATS Subject Pattern Schemas

Schema definitions for NATS subject patterns and routing rules. Routing patterns support:

- [Agent communication](./core-message-schemas.md#agent-communication-messages) via agent-specific subjects
- [Task distribution](./workflow-message-schemas.md#task-management-messages) through task-based routing
- [System monitoring](./system-message-schemas.md#system-operation-messages) with severity-based subjects
- [CLI integration](./system-message-schemas.md#claude-cli-integration-messages) using hook-specific patterns

For transport implementation, see [NATS Transport](../transport/nats-transport.md) and [Transport Core](../transport/transport-core.md).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/nats-subject-patterns.json",
  "title": "NATS Subject Pattern Definitions",
  "type": "object",
  "properties": {
    "agent_subjects": {
      "type": "object",
      "properties": {
        "commands": {
          "type": "string",
          "pattern": "^agents\\.[a-zA-Z0-9_-]+\\.commands\\.[a-z_]+$",
          "example": "agents.analyzer-001.commands.execute"
        },
        "status": {
          "type": "string", 
          "pattern": "^agents\\.[a-zA-Z0-9_-]+\\.status$",
          "example": "agents.analyzer-001.status"
        },
        "heartbeat": {
          "type": "string",
          "pattern": "^agents\\.[a-zA-Z0-9_-]+\\.heartbeat$",
          "example": "agents.analyzer-001.heartbeat"
        }
      }
    },
    "task_subjects": {
      "type": "object",
      "properties": {
        "assignment": {
          "type": "string",
          "pattern": "^tasks\\.[a-z_]+\\.assignment$",
          "example": "tasks.analysis.assignment"
        },
        "progress": {
          "type": "string",
          "pattern": "^tasks\\.[0-9a-f-]{36}\\.progress$",
          "example": "tasks.550e8400-e29b-41d4-a716-446655440000.progress"
        },
        "result": {
          "type": "string", 
          "pattern": "^tasks\\.[0-9a-f-]{36}\\.result$",
          "example": "tasks.550e8400-e29b-41d4-a716-446655440000.result"
        }
      }
    },
    "system_subjects": {
      "type": "object",
      "properties": {
        "alerts": {
          "type": "string",
          "pattern": "^system\\.alerts\\.(info|warning|error|critical|emergency)$",
          "example": "system.alerts.critical"
        },
        "health": {
          "type": "string",
          "pattern": "^system\\.health\\.[a-zA-Z0-9_-]+$",
          "example": "system.health.nats_cluster"
        }
      }
    },
    "cli_subjects": {
      "type": "object",
      "properties": {
        "hooks": {
          "type": "string",
          "pattern": "^cli\\.hooks\\.(startup|pre_task|post_task|on_error|on_file_change)\\.[a-zA-Z0-9_-]+$",
          "example": "cli.hooks.pre_task.analyzer-001"
        },
        "responses": {
          "type": "string",
          "pattern": "^cli\\.responses\\.[a-zA-Z0-9_-]+$",
          "example": "cli.responses.analyzer-001"
        }
      }
    }
  }
}
```

### 9.2 Message Correlation Strategies

Correlation strategies integrate with [Message Framework](./message-framework.md) patterns:

```json
{
  "correlation_strategies": {
    "hook_event_response": {
      "pattern": "Correlate CLI hook events with agent responses using correlation_id",
      "timeout_handling": "30-second timeout with exponential backoff (1s, 2s, 4s)",
      "correlation_storage": "In-memory LRU cache with 1-hour TTL",
      "implementation": "CorrelationTracker from message-framework.md",
      "example_flow": [
        "1. Hook event generated with correlation_id",
        "2. Agent processes and includes correlation_id in response", 
        "3. Framework matches request-response using correlation context"
      ]
    },
    "session_tracking": {
      "pattern": "Track CLI session lifecycle using session_id across all messages",
      "state_tracking": "Session state in distributed cache",
      "cleanup": "Automatic session cleanup after 24 hours of inactivity",
      "span_generation": "16-character hex span IDs for distributed tracing"
    },
    "system_health_aggregation": {
      "pattern": "Aggregate health messages by component using component correlation",
      "windowing": "5-second tumbling windows for metrics aggregation",
      "correlation_attributes": ["component", "node_id", "service_type"],
      "aggregation_strategy": "Latest value with timestamp tracking"
    },
    "alert_escalation_chains": {
      "pattern": "Maintain causality chains for alert root cause analysis",
      "causality_tracking": "Directed graph with parent-child relationships",
      "chain_length_limit": "Maximum 10 hops to prevent infinite chains",
      "cleanup": "Automatic cleanup of resolved alert chains after 7 days"
    }
  }
}
```

### Integration with Message Framework Components

System message correlation uses these [Message Framework](./message-framework.md) components:

- **CorrelationContext**: Session and trace tracking for CLI operations
- **CorrelationTracker**: Automatic cleanup and chain management
- **ContentRouter**: Route messages based on correlation metadata
- **EventAggregator**: Aggregate system health data by correlation attributes

For implementation patterns, see [Message Framework Correlation Logic](./message-framework.md#event-correlation-logic) and [Content-Based Routing](./message-framework.md#content-based-routing).

---

## Schema Relationships

### Integration Dependencies

- **Foundation**: [Core Message Schemas](./core-message-schemas.md) for base structures
- **Workflow Integration**: [Workflow Messages](./workflow-message-schemas.md) for orchestration
- **Framework Support**: [Message Framework](./message-framework.md) for validation and routing

### System Integration Points

- **CLI Hooks**: Connect tool execution to agent spawning and coordination
- **Health Monitoring**: Link agent status to system alerts and infrastructure metrics
- **Message Routing**: Enable transport-agnostic communication patterns
- **Error Handling**: Provide comprehensive error reporting and recovery

### External Integrations

- **Transport Layer**: [NATS](../transport/nats-transport.md), [gRPC](../transport/grpc-transport.md), [HTTP](../transport/http-transport.md)
- **Security**: [Security Patterns](../security/security-patterns.md)
- **Operations**: [Agent Operations](./agent-operations.md), [Connection Management](./connection-management.md)

## Navigation

This file is part of the Message Schema Documentation suite:

1. [Core Message Schemas](./core-message-schemas.md) - Foundation schemas and agent communication
2. [Workflow Message Schemas](./workflow-message-schemas.md) - Task management and workflow orchestration
3. **[System Message Schemas](./system-message-schemas.md)** - Claude CLI integration and system operations *(current file)*
4. [Message Framework](./message-framework.md) - Validation, serialization, and framework specifications

### Quick Access

- **CLI Integration**: [Hook Events](#51-hook-event-message), [Hook Responses](#52-hook-response-message)
- **System Operations**: [System Alerts](#61-system-alert-message), [Health Checks](#62-system-health-check-message)
- **Message Routing**: [NATS Patterns](#91-nats-subject-pattern-schemas), [Correlation](#92-message-correlation-strategies)
- **Agent Operations**: [Agent Communication](./agent-communication.md), [Agent Integration](./agent-integration.md)

For the complete framework documentation, see the [Data Management Index](./CLAUDE.md).

*Message Schema Definitions v1.0.0 - Mister Smith AI Agent Framework*

# Workflow Message Schemas

## Task Management and Workflow Orchestration

**Purpose**: This document defines message schemas for task assignment, progress tracking, and multi-agent workflow coordination within the Mister Smith AI Agent Framework.

## Overview

This file contains schemas for:

- **Task Management Messages** - Assignment, results, and progress tracking
- **Workflow Orchestration Messages** - Coordination and state synchronization

These schemas build upon the [Foundation Schemas](./core-message-schemas.md) and integrate with:

- [System operation messages](./system-message-schemas.md) for health monitoring and alerts
- [Claude CLI integration](./system-message-schemas.md#claude-cli-integration-messages) for task spawning
- [Message framework](./message-framework.md) for validation and serialization
- Transport layer specifications for reliable workflow execution

## 3. Task Management Messages

### 3.1 Task Assignment Message

Schema for assigning tasks to agents. This message type coordinates with:

- [Agent command messages](./core-message-schemas.md#agent-command-message) for execution directives
- [Agent registration data](./core-message-schemas.md#agent-registration-message) for capability matching
- [Hook event messages](./system-message-schemas.md#hook-event-message) for CLI-triggered assignments
- [System health checks](./system-message-schemas.md#system-health-check-message) for resource validation

See [Agent Communication](./agent-communication.md) for implementation patterns.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/task-assignment.json",
  "title": "Task Assignment Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "task_assignment" },
    "payload": {
      "type": "object",
      "required": ["task_id", "task_type", "assigned_agent", "created_at"],
      "properties": {
        "task_id": { "$ref": "common-types.json#/$defs/task_id" },
        "task_type": {
          "type": "string",
          "enum": ["analysis", "synthesis", "execution", "validation", "monitoring", "planning", "coordination"],
          "description": "Type of task to be executed"
        },
        "assigned_agent": { "$ref": "common-types.json#/$defs/agent_id" },
        "created_at": {
          "type": "string",
          "format": "date-time",
          "description": "Task creation timestamp"
        },
        "deadline": {
          "type": "string",
          "format": "date-time",
          "description": "Task completion deadline"
        },
        "priority": {
          "type": "integer",
          "minimum": 1,
          "maximum": 10,
          "default": 5,
          "description": "Task priority (1=highest, 10=lowest)"
        },
        "requirements": {
          "type": "object",
          "properties": {
            "capabilities": {
              "type": "array",
              "items": { "$ref": "common-types.json#/$defs/capability" },
              "description": "Required agent capabilities"
            },
            "resources": {
              "type": "object",
              "properties": {
                "cpu_cores": {
                  "type": "number",
                  "minimum": 0.1,
                  "description": "Required CPU cores"
                },
                "memory_mb": {
                  "type": "integer",
                  "minimum": 256,
                  "description": "Required memory in MB"
                },
                "disk_mb": {
                  "type": "integer",
                  "minimum": 100,
                  "description": "Required disk space in MB"
                },
                "network_bandwidth_mbps": {
                  "type": "number",
                  "minimum": 0,
                  "description": "Required network bandwidth"
                }
              }
            },
            "security": {
              "type": "object",
              "properties": {
                "security_level": {
                  "type": "string",
                  "enum": ["public", "internal", "confidential", "restricted"],
                  "default": "internal"
                },
                "required_permissions": {
                  "type": "array",
                  "items": { "type": "string" },
                  "description": "Required security permissions"
                },
                "requires_encryption": {
                  "type": "boolean",
                  "default": false,
                  "description": "Whether task requires encrypted communication"
                }
              }
            }
          }
        },
        "task_data": {
          "type": "object",
          "description": "Task-specific input data and parameters"
        },
        "dependencies": {
          "type": "array",
          "items": { "$ref": "common-types.json#/$defs/task_id" },
          "uniqueItems": true,
          "description": "Task dependencies that must complete first"
        },
        "callback_subjects": {
          "type": "object",
          "properties": {
            "progress": { "type": "string" },
            "completion": { "type": "string" },
            "error": { "type": "string" }
          },
          "description": "NATS subjects for task lifecycle callbacks"
        },
        "retry_policy": {
          "type": "object",
          "properties": {
            "max_retries": {
              "type": "integer",
              "minimum": 0,
              "maximum": 10,
              "default": 3
            },
            "backoff_strategy": {
              "type": "string",
              "enum": ["fixed", "linear", "exponential"],
              "default": "exponential"
            },
            "initial_delay_ms": {
              "type": "integer",
              "minimum": 100,
              "default": 1000
            },
            "max_delay_ms": {
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

### 3.2 Task Result Message

Schema for reporting task execution results. Result reporting integrates with:

- [Agent status updates](./core-message-schemas.md#agent-status-update-message) for capacity management
- [System alerts](./system-message-schemas.md#system-alert-message) for error escalation
- [Hook responses](./system-message-schemas.md#hook-response-message) for CLI feedback
- [Workflow coordination](./workflow-message-schemas.md#workflow-coordination-message) for state updates

For error handling patterns, see [Validation Framework](./message-framework.md#error-code-classification).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/task-result.json",
  "title": "Task Result Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "task_result" },
    "payload": {
      "type": "object",
      "required": ["task_id", "agent_id", "status", "completion_time"],
      "properties": {
        "task_id": { "$ref": "common-types.json#/$defs/task_id" },
        "agent_id": { "$ref": "common-types.json#/$defs/agent_id" },
        "status": { "$ref": "common-types.json#/$defs/task_status" },
        "completion_time": {
          "type": "string",
          "format": "date-time",
          "description": "Task completion timestamp"
        },
        "result_data": {
          "type": "object",
          "description": "Task execution results and output data"
        },
        "partial_results": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "timestamp": {
                "type": "string",
                "format": "date-time"
              },
              "data": { "type": "object" },
              "percentage_complete": {
                "type": "number",
                "minimum": 0,
                "maximum": 100
              }
            }
          },
          "description": "Intermediate results during task execution"
        },
        "error_details": { "$ref": "common-types.json#/$defs/error_details" },
        "metrics": { "$ref": "common-types.json#/$defs/execution_metrics" },
        "artifacts": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["artifact_id", "artifact_type"],
            "properties": {
              "artifact_id": {
                "type": "string",
                "format": "uuid"
              },
              "artifact_type": {
                "type": "string",
                "enum": ["file", "data", "log", "report", "model"]
              },
              "location": {
                "type": "string",
                "description": "Artifact storage location"
              },
              "size_bytes": {
                "type": "integer",
                "minimum": 0
              },
              "checksum": {
                "type": "string",
                "description": "SHA-256 checksum"
              },
              "metadata": {
                "type": "object",
                "description": "Artifact-specific metadata"
              }
            }
          },
          "description": "Generated artifacts and outputs"
        },
        "follow_up_tasks": {
          "type": "array",
          "items": { "$ref": "common-types.json#/$defs/task_id" },
          "description": "Tasks generated as a result of this task"
        },
        "quality_assessment": {
          "type": "object",
          "properties": {
            "score": {
              "type": "number",
              "minimum": 0,
              "maximum": 1,
              "description": "Quality score (0-1)"
            },
            "criteria": {
              "type": "object",
              "patternProperties": {
                "^[a-zA-Z0-9_-]+$": {
                  "type": "number",
                  "minimum": 0,
                  "maximum": 1
                }
              },
              "description": "Individual quality criteria scores"
            },
            "feedback": {
              "type": "string",
              "description": "Qualitative feedback on results"
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

### 3.3 Task Progress Update Message

Schema for reporting task execution progress. Progress updates connect to:

- [Agent health monitoring](./core-message-schemas.md#agent-status-update-message) for resource tracking
- [System performance metrics](./system-message-schemas.md#system-health-check-message) for infrastructure monitoring
- [Workflow state synchronization](./workflow-message-schemas.md#workflow-state-synchronization-message) for coordination

Performance optimization strategies are detailed in [Message Framework](./message-framework.md#performance-optimization).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/task-progress.json",
  "title": "Task Progress Update Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "task_progress" },
    "payload": {
      "type": "object",
      "required": ["task_id", "agent_id", "progress_percentage", "update_time"],
      "properties": {
        "task_id": { "$ref": "common-types.json#/$defs/task_id" },
        "agent_id": { "$ref": "common-types.json#/$defs/agent_id" },
        "progress_percentage": {
          "type": "number",
          "minimum": 0,
          "maximum": 100,
          "description": "Task completion percentage"
        },
        "update_time": {
          "type": "string",
          "format": "date-time",
          "description": "Progress update timestamp"
        },
        "current_phase": {
          "type": "string",
          "description": "Current execution phase or step"
        },
        "phase_description": {
          "type": "string",
          "maxLength": 512,
          "description": "Human-readable phase description"
        },
        "estimated_completion": {
          "type": "string",
          "format": "date-time",
          "description": "Estimated completion time"
        },
        "interim_results": {
          "type": "object",
          "description": "Preliminary results available at this point"
        },
        "resource_usage": { "$ref": "common-types.json#/$defs/resource_usage" },
        "blockers": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["blocker_type", "description"],
            "properties": {
              "blocker_type": {
                "type": "string",
                "enum": ["dependency", "resource", "permission", "data", "external"],
                "description": "Type of blocking issue"
              },
              "description": {
                "type": "string",
                "maxLength": 1024,
                "description": "Blocker description"
              },
              "severity": {
                "type": "string",
                "enum": ["low", "medium", "high", "critical"],
                "description": "Blocker severity level"
              },
              "estimated_resolution": {
                "type": "string",
                "format": "date-time",
                "description": "Estimated blocker resolution time"
              }
            }
          },
          "description": "Current execution blockers"
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

## 4. Workflow Orchestration Messages

### 4.1 Workflow Coordination Message

Schema for coordinating complex multi-agent workflows. Coordination relies on:

- [Common type definitions](./core-message-schemas.md#common-type-definitions) for agent identification
- [System health data](./system-message-schemas.md#system-health-check-message) for participant validation
- [Message transformation patterns](./message-framework.md#message-transformation-patterns) for protocol adaptation

See [Agent Operations](./agent-operations.md) for orchestration implementation details.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/workflow-coordination.json",
  "title": "Workflow Coordination Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "workflow_coordination" },
    "payload": {
      "type": "object",
      "required": ["workflow_id", "coordination_type", "participants"],
      "properties": {
        "workflow_id": {
          "type": "string",
          "format": "uuid",
          "description": "Workflow instance identifier"
        },
        "coordination_type": {
          "type": "string",
          "enum": ["sync", "barrier", "checkpoint", "rollback", "commit", "abort"],
          "description": "Type of coordination action"
        },
        "participants": {
          "type": "array",
          "items": { "$ref": "common-types.json#/$defs/agent_id" },
          "minItems": 1,
          "uniqueItems": true,
          "description": "Participating agents in coordination"
        },
        "coordination_data": {
          "type": "object",
          "description": "Coordination-specific data"
        },
        "timeout_ms": {
          "type": "integer",
          "minimum": 1000,
          "description": "Coordination timeout in milliseconds"
        },
        "required_confirmations": {
          "type": "integer",
          "minimum": 1,
          "description": "Number of required confirmations"
        },
        "compensation_actions": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["action_type", "target_agent"],
            "properties": {
              "action_type": {
                "type": "string",
                "enum": ["undo", "rollback", "cleanup", "notify"]
              },
              "target_agent": { "$ref": "common-types.json#/$defs/agent_id" },
              "action_data": { "type": "object" }
            }
          },
          "description": "Compensation actions for failure scenarios"
        },
        "dependency_graph": {
          "type": "object",
          "patternProperties": {
            "^[a-zA-Z0-9_-]+$": {
              "type": "array",
              "items": { "$ref": "common-types.json#/$defs/agent_id" }
            }
          },
          "description": "Agent dependency relationships"
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

### 4.2 Workflow State Synchronization Message

Schema for synchronizing workflow state across agents. State synchronization coordinates with:

- [Agent status reporting](./core-message-schemas.md#agent-status-update-message) for consistency
- [System monitoring](./system-message-schemas.md#system-health-check-message) for infrastructure state
- [Message routing patterns](./system-message-schemas.md#message-routing-and-addressing) for delivery
- [Version management](./message-framework.md#schema-version-management) for compatibility

For persistence patterns, see [Storage Operations](./persistence-operations.md).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.mister-smith.dev/workflow-state-sync.json",
  "title": "Workflow State Synchronization Message",
  "allOf": [
    { "$ref": "base-message.json" }
  ],
  "properties": {
    "message_type": { "const": "workflow_state_sync" },
    "payload": {
      "type": "object",
      "required": ["workflow_id", "state_version", "state_data"],
      "properties": {
        "workflow_id": {
          "type": "string",
          "format": "uuid",
          "description": "Workflow instance identifier"
        },
        "state_version": {
          "type": "integer",
          "minimum": 1,
          "description": "State version number for conflict resolution"
        },
        "state_data": {
          "type": "object",
          "required": ["workflow_status", "agent_states"],
          "properties": {
            "workflow_status": {
              "type": "string",
              "enum": ["initializing", "running", "suspended", "completed", "failed", "aborted"],
              "description": "Overall workflow status"
            },
            "agent_states": {
              "type": "object",
              "patternProperties": {
                "^[a-zA-Z0-9_-]+$": {
                  "type": "object",
                  "properties": {
                    "status": { "$ref": "common-types.json#/$defs/agent_status" },
                    "current_task": { "$ref": "common-types.json#/$defs/task_id" },
                    "completed_tasks": {
                      "type": "array",
                      "items": { "$ref": "common-types.json#/$defs/task_id" }
                    },
                    "local_state": { "type": "object" }
                  }
                }
              },
              "description": "Per-agent state information"
            },
            "shared_state": {
              "type": "object",
              "description": "Shared workflow state accessible to all agents"
            },
            "execution_context": {
              "type": "object",
              "properties": {
                "started_at": {
                  "type": "string",
                  "format": "date-time"
                },
                "last_checkpoint": {
                  "type": "string",
                  "format": "date-time"
                },
                "estimated_completion": {
                  "type": "string",
                  "format": "date-time"
                }
              }
            }
          }
        },
        "checksum": {
          "type": "string",
          "description": "State data integrity checksum"
        },
        "diff": {
          "type": "object",
          "description": "State changes since last synchronization"
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

## Practical Implementation Examples

### Async Task Coordination with Tokio

#### Task Assignment Handler

```rust
use tokio::sync::mpsc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TaskCoordinator {
    task_senders: Arc<HashMap<String, mpsc::Sender<TaskAssignmentMessage>>>,
    nats_client: Arc<NatsClient>,
}

impl TaskCoordinator {
    pub async fn assign_task(&self, assignment: TaskAssignmentMessage) -> Result<(), Error> {
        // Validate task assignment schema
        let validation_result = self.validate_schema(&assignment).await?;
        if !validation_result.is_valid {
            return Err(Error::InvalidSchema(validation_result.errors));
        }

        // Find available agent
        let agent_id = &assignment.payload.assigned_agent;
        let sender = self.task_senders.get(agent_id)
            .ok_or_else(|| Error::AgentNotFound(agent_id.clone()))?;

        // Send task to agent with timeout
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            sender.send(assignment.clone())
        ).await
        .map_err(|_| Error::TaskAssignmentTimeout)?
        .map_err(|_| Error::AgentChannelClosed)?;

        // Publish assignment event
        let event_subject = format!("task.assignment.{}", assignment.payload.task_id);
        self.nats_client.publish(&event_subject, &assignment.serialize()?).await?;

        Ok(())
    }

    pub async fn handle_task_progress(&self, progress: TaskProgressMessage) -> Result<(), Error> {
        // Update task state
        let task_id = &progress.payload.task_id;
        
        // Publish progress event for interested parties
        let progress_subject = format!("task.progress.{}", task_id);
        self.nats_client.publish(&progress_subject, &progress.serialize()?).await?;

        // Check for workflow coordination needs
        if progress.payload.progress_percentage >= 100.0 {
            self.coordinate_workflow_completion(task_id).await?;
        }

        Ok(())
    }

    async fn coordinate_workflow_completion(&self, task_id: &str) -> Result<(), Error> {
        // Workflow coordination logic
        let coordination_message = WorkflowCoordinationMessage {
            message_type: "workflow_coordination".to_string(),
            payload: WorkflowCoordinationPayload {
                workflow_id: self.get_workflow_for_task(task_id).await?,
                coordination_type: "checkpoint".to_string(),
                participants: self.get_workflow_participants(task_id).await?,
                // ... other fields
            },
            // ... base message fields
        };

        self.coordinate_workflow(coordination_message).await
    }
}
```

#### Multi-Agent Workflow Orchestration

```rust
use tokio::sync::{RwLock, Barrier};
use std::sync::Arc;

pub struct WorkflowOrchestrator {
    active_workflows: Arc<RwLock<HashMap<String, WorkflowState>>>,
    coordination_barriers: Arc<RwLock<HashMap<String, Arc<Barrier>>>>,
}

impl WorkflowOrchestrator {
    pub async fn sync_workflow_state(&self, sync_msg: WorkflowStateSyncMessage) -> Result<(), Error> {
        let workflow_id = &sync_msg.payload.workflow_id;
        
        // Acquire write lock for state update
        let mut workflows = self.active_workflows.write().await;
        
        // Apply state changes with conflict resolution
        let current_state = workflows.get(workflow_id);
        let resolved_state = self.resolve_state_conflicts(current_state, &sync_msg).await?;
        
        workflows.insert(workflow_id.clone(), resolved_state);
        
        // Notify waiting agents
        if let Some(barrier) = self.coordination_barriers.read().await.get(workflow_id) {
            // Signal coordination point completion
            barrier.wait().await;
        }

        Ok(())
    }

    pub async fn coordinate_agents(&self, coord_msg: WorkflowCoordinationMessage) -> Result<(), Error> {
        let workflow_id = &coord_msg.payload.workflow_id;
        let participant_count = coord_msg.payload.participants.len();

        match coord_msg.payload.coordination_type.as_str() {
            "barrier" => {
                // Create synchronization barrier
                let barrier = Arc::new(Barrier::new(participant_count));
                self.coordination_barriers.write().await
                    .insert(workflow_id.clone(), barrier.clone());

                // Wait for all participants
                tokio::time::timeout(
                    std::time::Duration::from_millis(coord_msg.payload.timeout_ms as u64),
                    barrier.wait()
                ).await
                .map_err(|_| Error::CoordinationTimeout)?;
            },
            "checkpoint" => {
                self.create_workflow_checkpoint(workflow_id).await?;
            },
            "rollback" => {
                self.execute_workflow_rollback(workflow_id, &coord_msg.payload.compensation_actions).await?;
            },
            _ => return Err(Error::UnsupportedCoordinationType)
        }

        Ok(())
    }

    async fn resolve_state_conflicts(
        &self, 
        current: Option<&WorkflowState>, 
        incoming: &WorkflowStateSyncMessage
    ) -> Result<WorkflowState, Error> {
        match current {
            Some(state) if state.version >= incoming.payload.state_version => {
                // Current state is newer, keep it
                Ok(state.clone())
            },
            _ => {
                // Apply incoming state
                Ok(WorkflowState::from_sync_message(incoming)?)
            }
        }
    }
}
```

#### Error Handling and Recovery

```rust
pub struct WorkflowErrorHandler {
    retry_policies: HashMap<String, RetryPolicy>,
    compensation_handlers: HashMap<String, CompensationHandler>,
}

impl WorkflowErrorHandler {
    pub async fn handle_task_failure(&self, task_id: &str, error: TaskError) -> Result<(), Error> {
        let retry_policy = self.retry_policies.get(&error.error_type)
            .unwrap_or(&RetryPolicy::default());

        if error.retry_count < retry_policy.max_retries {
            // Schedule retry with backoff
            let delay = self.calculate_backoff_delay(retry_policy, error.retry_count);
            
            tokio::time::sleep(delay).await;
            
            // Republish task assignment
            self.republish_task_assignment(task_id).await?;
        } else {
            // Execute compensation
            if let Some(handler) = self.compensation_handlers.get(&error.task_type) {
                handler.compensate(task_id, &error).await?;
            }
            
            // Mark workflow as failed
            self.fail_workflow(task_id).await?;
        }

        Ok(())
    }

    fn calculate_backoff_delay(&self, policy: &RetryPolicy, attempt: u32) -> std::time::Duration {
        match policy.backoff_strategy {
            BackoffStrategy::Fixed => std::time::Duration::from_millis(policy.initial_delay_ms),
            BackoffStrategy::Linear => {
                std::time::Duration::from_millis(policy.initial_delay_ms * (attempt + 1) as u64)
            },
            BackoffStrategy::Exponential => {
                let delay = policy.initial_delay_ms * 2_u64.pow(attempt);
                std::time::Duration::from_millis(delay.min(policy.max_delay_ms))
            }
        }
    }
}
```

---

## Schema Relationships

### Dependencies

- **Built on**: [Core Message Foundation](./core-message-schemas.md#foundation-schemas)
- **Integrates with**: [System Operations](./system-message-schemas.md), [Claude CLI](./system-message-schemas.md#claude-cli-integration-messages)
- **Validated by**: [Message Framework](./message-framework.md#validation-framework)

### Workflow Integration Points

- **Task Lifecycle**: Assignment → Progress → Results → Coordination
- **Agent Communication**: [Agent Messages](./core-message-schemas.md#agent-communication-messages)
- **System Monitoring**: [Health Checks](./system-message-schemas.md#system-health-check-message), [Alerts](./system-message-schemas.md#system-alert-message)
- **CLI Integration**: [Hook Events](./system-message-schemas.md#hook-event-message), [Responses](./system-message-schemas.md#hook-response-message)

## Navigation

This file is part of the Message Schema Documentation suite:

1. [Core Message Schemas](./core-message-schemas.md) - Foundation schemas and agent communication
2. **[Workflow Message Schemas](./workflow-message-schemas.md)** - Task management and workflow orchestration *(current file)*
3. [System Message Schemas](./system-message-schemas.md) - Claude CLI integration and system operations
4. [Message Framework](./message-framework.md) - Validation, serialization, and framework specifications

### Related Documentation

- **Implementation**: [Agent Communication](./agent-communication.md), [Agent Operations](./agent-operations.md)
- **Storage**: [Persistence Operations](./persistence-operations.md), [Storage Patterns](./storage-patterns.md)
- **Transport**: [NATS Transport](../transport/nats-transport.md), [gRPC Transport](../transport/grpc-transport.md)

For the complete framework documentation, see the [Data Management Index](./CLAUDE.md).

*Message Schema Definitions v1.0.0 - Mister Smith AI Agent Framework*

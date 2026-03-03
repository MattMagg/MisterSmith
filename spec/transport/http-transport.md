---
title: http-transport
type: note
permalink: ms-framework/transport/http-transport
---

## HTTP Transport Specifications

## Agent Framework Implementation Module

> **Source**: Extracted from `transport-layer-specifications.md` sections 4 and 14
> **Technology Stack**: Axum 0.8 (HTTP), Tokio 1.38 (async runtime)

## Overview

This document defines HTTP-based transport patterns for agent communication using the Claude-Flow Rust Stack.
This module focuses on RESTful APIs, WebSocket communication, and complete HTTP protocol specifications.

HTTP transport provides the foundation for web-compatible agent communication with support for:

- REST APIs with comprehensive endpoint specifications
- WebSocket connections for real-time bidirectional communication
- Multiple authentication schemes (Bearer tokens, API keys)
- Standardized error handling with HTTP status codes
- Integration with Axum 0.8 framework patterns

**Technology Reference** (from tech-framework.md):

- Axum 0.8 (HTTP)
- Tokio 1.38 (async runtime)

**Transport Core Dependencies**:

- Section 5: Transport abstraction layer integration and HTTP-specific routing
- Section 7: Connection management patterns for HTTP client and server pools
- Section 8: Error handling standards and HTTP status code mapping
- Section 9: Authentication patterns including JWT, API keys, and OAuth flows

## 1. HTTP API Patterns

### 1.1 RESTful Endpoints

```rust
API_STRUCTURE:
    GET  /agents              # List agents
    POST /agents              # Register agent
    GET  /agents/{id}         # Get agent details
    POST /agents/{id}/message # Send message
    
    GET  /tasks               # List tasks
    POST /tasks               # Create task
    GET  /tasks/{id}          # Get task status
```

### 1.2 WebSocket Communication

```rust
WEBSOCKET_PROTOCOL:
    CONNECTION:
        Client connects to /ws
        Server accepts connection
        
    MESSAGE_FORMAT:
        type: "request" | "response" | "event"
        action: string
        payload: data
        
    PATTERNS:
        - Event notification
        - Real-time updates
        - Bidirectional messaging
```

## 2. HTTP API Specifications

### 2.1 OpenAPI 3.0 Specification

```yaml
openapi: 3.0.3
info:
  title: Agent Framework API
  description: RESTful API for agent communication and management
  version: 1.0.0

servers:
  - url: http://localhost:8080/v1
    description: Development server
  - url: https://api.example.com/v1
    description: Production server template

security:
  - bearerAuth: []
  - apiKeyAuth: []

paths:
  /agents:
    get:
      summary: List all agents
      description: Retrieve a paginated list of all registered agents
      operationId: listAgents
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            minimum: 1
            default: 1
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
        - name: status
          in: query
          schema:
            type: string
            enum: [idle, busy, error, offline, starting, stopping]
        - name: capability
          in: query
          schema:
            type: string
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentListResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '500':
          $ref: '#/components/responses/InternalServerError'

    post:
      summary: Register a new agent
      description: Register a new agent with the framework
      operationId: registerAgent
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AgentRegistration'
      responses:
        '201':
          description: Agent registered successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentRegistrationResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '409':
          $ref: '#/components/responses/Conflict'
        '500':
          $ref: '#/components/responses/InternalServerError'

  /agents/{agentId}:
    get:
      summary: Get agent details
      description: Retrieve detailed information about a specific agent
      operationId: getAgent
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]+$'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Agent'
        '404':
          $ref: '#/components/responses/NotFound'
        '500':
          $ref: '#/components/responses/InternalServerError'

    put:
      summary: Update agent configuration
      description: Update the configuration of an existing agent
      operationId: updateAgent
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]+$'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AgentUpdate'
      responses:
        '200':
          description: Agent updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Agent'
        '400':
          $ref: '#/components/responses/BadRequest'
        '404':
          $ref: '#/components/responses/NotFound'
        '500':
          $ref: '#/components/responses/InternalServerError'

    delete:
      summary: Deregister agent
      description: Remove an agent from the framework
      operationId: deregisterAgent
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]+$'
      responses:
        '204':
          description: Agent deregistered successfully
        '404':
          $ref: '#/components/responses/NotFound'
        '500':
          $ref: '#/components/responses/InternalServerError'

  /agents/{agentId}/commands:
    post:
      summary: Send command to agent
      description: Send a command to a specific agent
      operationId: sendCommand
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]+$'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Command'
      responses:
        '202':
          description: Command accepted
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CommandResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '404':
          $ref: '#/components/responses/NotFound'
        '500':
          $ref: '#/components/responses/InternalServerError'

  /agents/{agentId}/status:
    get:
      summary: Get agent status
      description: Retrieve the current status of an agent
      operationId: getAgentStatus
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]+$'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentStatus'
        '404':
          $ref: '#/components/responses/NotFound'
        '500':
          $ref: '#/components/responses/InternalServerError'

  /tasks:
    get:
      summary: List tasks
      description: Retrieve a paginated list of tasks
      operationId: listTasks
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            minimum: 1
            default: 1
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
        - name: status
          in: query
          schema:
            type: string
            enum: [pending, running, completed, failed, cancelled, timeout]
        - name: type
          in: query
          schema:
            type: string
            enum: [analysis, synthesis, execution, validation, monitoring]
        - name: agent_id
          in: query
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]+$'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TaskListResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '500':
          $ref: '#/components/responses/InternalServerError'

    post:
      summary: Create a new task
      description: Create and assign a new task
      operationId: createTask
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TaskCreation'
      responses:
        '201':
          description: Task created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Task'
        '400':
          $ref: '#/components/responses/BadRequest'
        '500':
          $ref: '#/components/responses/InternalServerError'

  /tasks/{taskId}:
    get:
      summary: Get task details
      description: Retrieve detailed information about a specific task
      operationId: getTask
      parameters:
        - name: taskId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Task'
        '404':
          $ref: '#/components/responses/NotFound'
        '500':
          $ref: '#/components/responses/InternalServerError'

    delete:
      summary: Cancel task
      description: Cancel a pending or running task
      operationId: cancelTask
      parameters:
        - name: taskId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Task cancelled successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TaskCancellation'
        '404':
          $ref: '#/components/responses/NotFound'
        '409':
          $ref: '#/components/responses/Conflict'
        '500':
          $ref: '#/components/responses/InternalServerError'

  /ws:
    get:
      summary: WebSocket endpoint
      description: WebSocket connection for real-time communication
      operationId: websocketConnect
      parameters:
        - name: protocol
          in: query
          schema:
            type: string
            enum: [events, commands, status]
            default: events
      responses:
        '101':
          description: Switching protocols to WebSocket
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
    apiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key

  schemas:
    Agent:
      type: object
      required: [agent_id, status, capabilities, registered_at]
      properties:
        agent_id:
          type: string
          pattern: '^[a-zA-Z0-9_-]+$'
          example: "analyzer-001"
        status:
          type: string
          enum: [idle, busy, error, offline, starting, stopping]
          example: "idle"
        capabilities:
          type: array
          items:
            type: string
          example: ["text-analysis", "data-processing"]
        registered_at:
          type: string
          format: date-time
        last_heartbeat:
          type: string
          format: date-time
        resource_usage:
          $ref: '#/components/schemas/ResourceUsage'
        current_tasks:
          type: integer
          minimum: 0
          example: 2
        max_tasks:
          type: integer
          minimum: 1
          example: 10

    AgentRegistration:
      type: object
      required: [agent_id, capabilities]
      properties:
        agent_id:
          type: string
          pattern: '^[a-zA-Z0-9_-]+$'
        capabilities:
          type: array
          items:
            type: string
          minItems: 1
        max_tasks:
          type: integer
          minimum: 1
          default: 5
        metadata:
          type: object
          additionalProperties: true

    Task:
      type: object
      required: [task_id, type, status, created_at]
      properties:
        task_id:
          type: string
          format: uuid
        type:
          type: string
          enum: [analysis, synthesis, execution, validation, monitoring]
        status:
          type: string
          enum: [pending, running, completed, failed, cancelled, timeout]
        assigned_agent:
          type: string
          pattern: '^[a-zA-Z0-9_-]+$'
        created_at:
          type: string
          format: date-time
        deadline:
          type: string
          format: date-time
        priority:
          type: integer
          minimum: 1
          maximum: 10
        task_data:
          type: object
          additionalProperties: true
        result:
          type: object
          additionalProperties: true

    Command:
      type: object
      required: [command_type]
      properties:
        command_type:
          type: string
          enum: [execute, stop, pause, resume, configure, shutdown]
        payload:
          type: object
          additionalProperties: true
        priority:
          type: integer
          minimum: 1
          maximum: 10
          default: 5
        timeout_ms:
          type: integer
          minimum: 1000
          default: 30000

    ResourceUsage:
      type: object
      properties:
        cpu_percent:
          type: number
          minimum: 0
          maximum: 100
        memory_mb:
          type: number
          minimum: 0
        disk_mb:
          type: number
          minimum: 0

    Error:
      type: object
      required: [error_code, message]
      properties:
        error_code:
          type: string
          example: "INVALID_REQUEST"
        message:
          type: string
          example: "The request payload is invalid"
        details:
          type: array
          items:
            type: object
            properties:
              field:
                type: string
              issue:
                type: string
        trace_id:
          type: string
          format: uuid

  responses:
    BadRequest:
      description: Bad request
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    Unauthorized:
      description: Unauthorized
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    Conflict:
      description: Resource conflict
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    InternalServerError:
      description: Internal server error
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
```

### 2.2 Axum Implementation Patterns

#### 2.2.1 HTTP Router Configuration

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub agent_registry: Arc<AgentRegistry>,
    pub task_manager: Arc<TaskManager>,
    pub config: Arc<HttpConfig>,
}

// Main HTTP router setup
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Agent endpoints
        .route("/agents", get(list_agents).post(register_agent))
        .route("/agents/:agent_id", get(get_agent).put(update_agent).delete(deregister_agent))
        .route("/agents/:agent_id/commands", post(send_command))
        .route("/agents/:agent_id/status", get(get_agent_status))
        
        // Task endpoints
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/:task_id", get(get_task).delete(cancel_task))
        
        // WebSocket upgrade
        .route("/ws", get(websocket_handler))
        
        // Middleware for authentication and logging
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn(logging_middleware))
        .with_state(state)
}

// Agent registration handler
async fn register_agent(
    State(state): State<AppState>,
    Json(payload): Json<AgentRegistration>,
) -> Result<(StatusCode, Json<AgentRegistrationResponse>), HttpError> {
    // Validate agent registration
    if payload.capabilities.is_empty() {
        return Err(HttpError::BadRequest("Agent must have at least one capability".to_string()));
    }
    
    // Register agent with registry
    let agent_id = state.agent_registry.register(payload).await?;
    
    let response = AgentRegistrationResponse {
        agent_id,
        registered_at: chrono::Utc::now(),
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

// Task creation handler
async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<TaskCreation>,
) -> Result<(StatusCode, Json<Task>), HttpError> {
    // Validate task creation request
    if payload.task_type.is_empty() {
        return Err(HttpError::BadRequest("Task type is required".to_string()));
    }
    
    // Create and assign task
    let task = state.task_manager.create_task(payload).await?;
    
    Ok((StatusCode::CREATED, Json(task)))
}

// WebSocket upgrade handler
async fn websocket_handler(
    ws: axum::extract::WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: axum::extract::WebSocket, state: AppState) {
    // WebSocket connection handling logic
    // See WebSocket Protocol Specification below for message format
}
```

#### 2.2.2 Error Handling Patterns

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    pub details: Option<Vec<ErrorDetail>>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub issue: String,
}

#[derive(Debug)]
pub enum HttpError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    InternalServerError(String),
    ValidationError(Vec<ErrorDetail>),
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            HttpError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error_code: "BAD_REQUEST".to_string(),
                    message: msg,
                    details: None,
                    trace_id: Some(generate_trace_id()),
                },
            ),
            HttpError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error_code: "UNAUTHORIZED".to_string(),
                    message: msg,
                    details: None,
                    trace_id: Some(generate_trace_id()),
                },
            ),
            HttpError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error_code: "NOT_FOUND".to_string(),
                    message: msg,
                    details: None,
                    trace_id: Some(generate_trace_id()),
                },
            ),
            HttpError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    error_code: "CONFLICT".to_string(),
                    message: msg,
                    details: None,
                    trace_id: Some(generate_trace_id()),
                },
            ),
            HttpError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error_code: "INTERNAL_SERVER_ERROR".to_string(),
                    message: msg,
                    details: None,
                    trace_id: Some(generate_trace_id()),
                },
            ),
            HttpError::ValidationError(details) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error_code: "VALIDATION_ERROR".to_string(),
                    message: "Request validation failed".to_string(),
                    details: Some(details),
                    trace_id: Some(generate_trace_id()),
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
}

fn generate_trace_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
```

#### 2.2.3 Authentication Middleware

```rust
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    // Extract authentication token
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .or_else(|| {
            headers
                .get("X-API-Key")
                .and_then(|value| value.to_str().ok())
        })
        .ok_or_else(|| HttpError::Unauthorized("Missing authentication token".to_string()))?;

    // Validate token
    let _user_id = state.auth_service.validate_token(token).await
        .map_err(|_| HttpError::Unauthorized("Invalid authentication token".to_string()))?;

    // Continue with request
    Ok(next.run(request).await)
}
```

### 2.3 WebSocket Protocol Specification

```json
WEBSOCKET_PROTOCOL: {
  "connection": {
    "url": "wss://api.mister-smith.dev/v1/ws",
    "protocols": ["events", "commands", "status"],
    "authentication": {
      "type": "bearer_token",
      "header": "Authorization"
    },
    "heartbeat_interval": 30,
    "reconnect_policy": {
      "initial_delay": 1000,
      "max_delay": 30000,
      "backoff_factor": 2.0,
      "max_attempts": 10
    }
  },
  "message_format": {
    "type": "object",
    "required": ["type", "id", "timestamp"],
    "properties": {
      "type": {
        "type": "string",
        "enum": ["event", "command", "response", "heartbeat", "error"]
      },
      "id": {
        "type": "string",
        "format": "uuid"
      },
      "timestamp": {
        "type": "string",
        "format": "date-time"
      },
      "payload": {
        "type": "object"
      },
      "correlation_id": {
        "type": "string",
        "format": "uuid"
      }
    }
  },
  "event_types": {
    "agent_status_changed": {
      "payload": {
        "agent_id": "string",
        "old_status": "string",
        "new_status": "string",
        "timestamp": "date-time"
      }
    },
    "task_assigned": {
      "payload": {
        "task_id": "uuid",
        "agent_id": "string",
        "task_type": "string",
        "priority": "integer"
      }
    },
    "task_completed": {
      "payload": {
        "task_id": "uuid",
        "agent_id": "string",
        "status": "string",
        "execution_time_ms": "integer"
      }
    },
    "system_alert": {
      "payload": {
        "severity": "string",
        "component": "string",
        "message": "string",
        "details": "object"
      }
    }
  }
}
```

### 2.4 WebSocket Implementation Patterns

#### 2.4.1 WebSocket Connection Management

```rust
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: String,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Event,
    Command,
    Response,
    Heartbeat,
    Error,
}

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub agent_id: Option<String>,
    pub protocols: Vec<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

pub struct WebSocketManager {
    connections: RwLock<HashMap<String, WebSocketConnection>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        Self {
            connections: RwLock::new(HashMap::new()),
            broadcast_tx,
        }
    }

    pub async fn handle_connection(
        &self,
        socket: WebSocket,
        connection_id: String,
    ) -> Result<(), WebSocketError> {
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            agent_id: None,
            protocols: vec!["events".to_string()],
            connected_at: chrono::Utc::now(),
        };

        // Register connection
        self.connections.write().await.insert(connection_id.clone(), connection);

        // Set up message streams
        let (mut sender, mut receiver) = socket.split();
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        // Handle incoming messages
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                            // Process incoming message
                            // Handle commands, responses, etc.
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Handle binary messages if needed
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Handle outgoing messages
        let outgoing_task = tokio::spawn(async move {
            while let Ok(msg) = broadcast_rx.recv().await {
                let json_msg = serde_json::to_string(&msg).unwrap();
                if sender.send(Message::Text(json_msg)).await.is_err() {
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        // Clean up connection
        self.connections.write().await.remove(&connection_id);
        Ok(())
    }

    pub async fn broadcast_event(&self, event: WebSocketMessage) -> Result<(), WebSocketError> {
        self.broadcast_tx.send(event)
            .map_err(|_| WebSocketError::BroadcastError)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum WebSocketError {
    ConnectionClosed,
    BroadcastError,
    ParseError(String),
}
```

#### 2.4.2 Event Broadcasting Integration

```rust
use crate::agents::AgentEvent;
use crate::tasks::TaskEvent;

// Integration with agent events
pub async fn broadcast_agent_event(
    ws_manager: &WebSocketManager,
    agent_event: AgentEvent,
) -> Result<(), WebSocketError> {
    let ws_message = WebSocketMessage {
        id: Uuid::new_v4().to_string(),
        message_type: MessageType::Event,
        timestamp: chrono::Utc::now(),
        payload: serde_json::to_value(agent_event)?,
        correlation_id: None,
    };

    ws_manager.broadcast_event(ws_message).await
}

// Integration with task events
pub async fn broadcast_task_event(
    ws_manager: &WebSocketManager,
    task_event: TaskEvent,
) -> Result<(), WebSocketError> {
    let ws_message = WebSocketMessage {
        id: Uuid::new_v4().to_string(),
        message_type: MessageType::Event,
        timestamp: chrono::Utc::now(),
        payload: serde_json::to_value(task_event)?,
        correlation_id: None,
    };

    ws_manager.broadcast_event(ws_message).await
}
```

#### 2.4.3 WebSocket Integration with Transport Core

```rust
use crate::transport::TransportLayer;

impl TransportLayer {
    pub async fn setup_http_websocket(&self) -> Result<(), TransportError> {
        // Initialize WebSocket manager
        let ws_manager = WebSocketManager::new();

        // Connect to agent event stream
        let agent_events = self.agent_registry.subscribe_events().await?;
        
        // Connect to task event stream  
        let task_events = self.task_manager.subscribe_events().await?;

        // Forward agent events to WebSocket
        tokio::spawn(async move {
            while let Ok(event) = agent_events.recv().await {
                if let Err(e) = broadcast_agent_event(&ws_manager, event).await {
                    log::error!("Failed to broadcast agent event: {:?}", e);
                }
            }
        });

        // Forward task events to WebSocket
        tokio::spawn(async move {
            while let Ok(event) = task_events.recv().await {
                if let Err(e) = broadcast_task_event(&ws_manager, event).await {
                    log::error!("Failed to broadcast task event: {:?}", e);
                }
            }
        });

        Ok(())
    }
}
```

## Summary

This HTTP transport module provides comprehensive specifications for:

- RESTful API endpoints with complete OpenAPI 3.0 documentation
- WebSocket real-time communication protocols
- Authentication and security mechanisms
- Error handling and response standards

The specifications are designed for use with Axum 0.8 and provide a complete foundation for HTTP-based agent communication within the Mister Smith framework.

## Navigation

### Transport Module Cross-References

- **[Transport Core](./transport-core.md)** - Core abstractions, connection management, and security patterns
  - Section 5: Transport abstraction layer integration and HTTP-specific routing
  - Section 7: Connection management patterns for HTTP client and server pools
  - Section 8: Error handling standards and HTTP status code mapping
  - Section 9: Authentication patterns including JWT, API keys, and OAuth flows
- **[NATS Transport](./nats-transport.md)** - High-throughput messaging and pub/sub patterns
  - Hierarchical subject structure for agent and task communication
  - JetStream persistence configurations for reliable message delivery
  - Performance benchmarks and comparison with HTTP throughput
- **[gRPC Transport](./grpc-transport.md)** - RPC communication and streaming protocols
  - Protocol Buffers v3 definitions for AgentCommunication services
  - Streaming patterns comparison with HTTP/WebSocket approaches
  - HTTP/2 transport layer shared foundation
- **[Transport Layer Specifications](./transport-layer-specifications.md)** - Complete transport architecture

### Framework Integration Points

- **[Core Architecture](../core-architecture/)** - System integration and async patterns
  - **[Async Patterns](../core-architecture/async-patterns.md)** - Tokio runtime integration with HTTP services
  - **[Component Architecture](../core-architecture/component-architecture.md)** - HTTP service component patterns
  - **[Supervision Trees](../core-architecture/supervision-trees.md)** - HTTP service supervision and restart strategies
- **[Security](../security/)** - Authentication, authorization, and transport security
  - **[Authentication](../security/authentication.md)** - JWT and API key implementation patterns
  - **[Authorization](../security/authorization.md)** - HTTP request authorization middleware
  - **[Security Integration](../security/security-integration.md)** - End-to-end security patterns
- **[Data Management](../data-management/)** - Message schemas and persistence patterns
  - **[Message Schemas](../data-management/message-schemas.md)** - HTTP request/response schema validation
  - **[Agent Communication](../data-management/agent-communication.md)** - Message format integration
  - **[Connection Management](../data-management/connection-management.md)** - HTTP connection pooling patterns

### External References

- **Technology Stack**: `/tech-framework.md` - Canonical technology specifications
- **OpenAPI/Swagger**: For API documentation and client generation
- **Axum Documentation**: Framework-specific implementation guidance
- **Tokio Documentation**: Async runtime patterns and best practices

### Protocol Selection Guidelines

Use HTTP when you need:

- RESTful APIs following web standards
- WebSocket real-time bidirectional communication  
- Integration with web browsers and standard HTTP clients
- OpenAPI/Swagger documentation and tooling
- Simple request/response patterns
- Load balancer and proxy compatibility
- Standard HTTP status codes and headers

**Alternative Protocols:**

- **NATS**: For high-throughput pub/sub messaging (3M+ msgs/sec vs HTTP ~10K req/sec)
- **gRPC**: For typed RPC calls with Protocol Buffers efficiency and streaming support

### HTTP Transport Performance Characteristics

- **Throughput**: ~10,000 requests/second per core (REST API)
- **Latency**: 1-5ms for local requests, 10-50ms for network requests
- **WebSocket**: Real-time bidirectional communication with ~1ms message latency
- **Connection Overhead**: HTTP/1.1 connection pooling, HTTP/2 multiplexing
- **Scalability**: Horizontal scaling with load balancers, connection pooling

### Implementation Notes

- This document provides complete HTTP API specifications with OpenAPI 3.0 schema
- For connection pooling and management, see transport-core.md Section 7
- For security implementation including JWT and API keys, see transport-core.md Section 9
- WebSocket protocols complement HTTP for real-time communication needs
- Integration with transport abstraction layer defined in transport-core.md Section 5

---

*HTTP Transport Specifications - Agent Framework Implementation Module*
*Extracted from transport-layer-specifications.md sections 4 and 14*

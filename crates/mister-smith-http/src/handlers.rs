//! HTTP request handlers for REST API endpoints.
//!
//! All handlers accept `State<AppState>` and return JSON responses.
//! Currently uses placeholder/mock data since there is no backend store yet.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use mister_smith_core::{AgentAvailability, AgentId, AgentType, TaskId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::HttpError;
use crate::server::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall system status.
    pub status: String,
    /// Individual component statuses.
    pub components: Vec<ComponentHealth>,
}

/// Individual component health within the health response.
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    /// Component name.
    pub name: String,
    /// Component status: healthy, degraded, or unhealthy.
    pub status: String,
    /// Optional status message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Agent summary for list responses.
#[derive(Debug, Serialize)]
pub struct AgentSummary {
    /// Agent identifier.
    pub agent_id: AgentId,
    /// Agent type/role.
    pub agent_type: AgentType,
    /// Current availability.
    pub availability: AgentAvailability,
    /// Human-readable name.
    pub name: String,
}

/// Agent detail response.
#[derive(Debug, Serialize)]
pub struct AgentDetail {
    /// Agent identifier.
    pub agent_id: AgentId,
    /// Agent type/role.
    pub agent_type: AgentType,
    /// Current availability.
    pub availability: AgentAvailability,
    /// Human-readable name.
    pub name: String,
    /// Additional metadata.
    pub metadata: serde_json::Value,
}

/// Task submission request.
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    /// Task description.
    pub description: String,
    /// Target agent type for assignment.
    #[serde(default)]
    pub agent_type: Option<AgentType>,
    /// Task priority.
    #[serde(default)]
    pub priority: Option<String>,
}

/// Task submission response (202 Accepted).
#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    /// Assigned task identifier.
    pub task_id: TaskId,
    /// Agent the task was assigned to.
    pub assigned_agent_id: AgentId,
    /// Task status.
    pub status: String,
}

/// Task status response.
#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    /// Task identifier.
    pub task_id: TaskId,
    /// Current status.
    pub status: String,
    /// Task result, if complete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// System configuration response.
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    /// System version.
    pub version: String,
    /// Configuration entries.
    pub config: serde_json::Value,
}

/// Query parameters for listing agents.
#[derive(Debug, Deserialize, Default)]
pub struct AgentListQuery {
    /// Filter by availability.
    #[serde(default)]
    pub availability: Option<String>,
    /// Filter by agent type.
    #[serde(default)]
    pub agent_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/health` — System health with component details.
pub async fn health_check(State(_state): State<AppState>) -> Json<HealthResponse> {
    let components = vec![
        ComponentHealth {
            name: "http_server".to_string(),
            status: "healthy".to_string(),
            message: None,
        },
        ComponentHealth {
            name: "event_bus".to_string(),
            status: "healthy".to_string(),
            message: None,
        },
    ];

    let overall = if components.iter().all(|c| c.status == "healthy") {
        "healthy"
    } else if components.iter().any(|c| c.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    Json(HealthResponse {
        status: overall.to_string(),
        components,
    })
}

/// `GET /api/v1/agents` — List agents with optional filters.
pub async fn list_agents(
    State(_state): State<AppState>,
    Query(query): Query<AgentListQuery>,
) -> Json<Vec<AgentSummary>> {
    // Placeholder: return mock agents, applying filters if provided.
    let agents = mock_agents();
    let filtered: Vec<AgentSummary> = agents
        .into_iter()
        .filter(|a| {
            if let Some(ref avail) = query.availability {
                let agent_avail = format!("{:?}", a.availability);
                if !agent_avail.eq_ignore_ascii_case(avail) {
                    return false;
                }
            }
            if let Some(ref atype) = query.agent_type {
                let agent_type = format!("{:?}", a.agent_type);
                if !agent_type.eq_ignore_ascii_case(atype) {
                    return false;
                }
            }
            true
        })
        .collect();

    Json(filtered)
}

/// `GET /api/v1/agents/{agent_id}` — Single agent detail.
pub async fn get_agent(
    State(_state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetail>, HttpError> {
    // Try to parse agent_id as UUID.
    let uuid = Uuid::parse_str(&agent_id)
        .map_err(|_| HttpError::BadRequest(format!("Invalid agent ID: {agent_id}")))?;

    // Placeholder: check mock agents.
    let agents = mock_agents();
    let found = agents.iter().find(|a| *a.agent_id.as_ref() == uuid);

    match found {
        Some(agent) => Ok(Json(AgentDetail {
            agent_id: agent.agent_id,
            agent_type: agent.agent_type,
            availability: agent.availability,
            name: agent.name.clone(),
            metadata: serde_json::json!({
                "uptime_seconds": 3600,
                "tasks_completed": 42,
            }),
        })),
        None => Err(HttpError::NotFound(format!("Agent {agent_id} not found"))),
    }
}

/// `POST /api/v1/tasks` — Submit a task, returns 202 Accepted.
pub async fn create_task(
    State(_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> (StatusCode, Json<CreateTaskResponse>) {
    let _ = request.description;
    let task_id = TaskId::new();
    let assigned_agent_id = AgentId::new();

    (
        StatusCode::ACCEPTED,
        Json(CreateTaskResponse {
            task_id,
            assigned_agent_id,
            status: "pending".to_string(),
        }),
    )
}

/// `GET /api/v1/tasks/{task_id}` — Task status and result.
pub async fn get_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, HttpError> {
    let uuid = Uuid::parse_str(&task_id)
        .map_err(|_| HttpError::BadRequest(format!("Invalid task ID: {task_id}")))?;

    // Placeholder: return a mock task status.
    Ok(Json(TaskStatusResponse {
        task_id: TaskId::from_uuid(uuid),
        status: "completed".to_string(),
        result: Some(serde_json::json!({
            "output": "Task completed successfully",
        })),
    }))
}

/// `GET /api/v1/config` — System configuration.
pub async fn get_config(State(_state): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        version: "0.1.0".to_string(),
        config: serde_json::json!({
            "runtime": {
                "worker_threads": 4,
                "max_blocking_threads": 512,
            },
            "transport": {
                "http": {
                    "bind_address": "0.0.0.0:8080",
                },
            },
        }),
    })
}

// ---------------------------------------------------------------------------
// Mock data helpers
// ---------------------------------------------------------------------------

/// Generate mock agents for placeholder responses.
fn mock_agents() -> Vec<AgentSummary> {
    vec![
        AgentSummary {
            agent_id: AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            ),
            agent_type: AgentType::Worker,
            availability: AgentAvailability::Idle,
            name: "worker-1".to_string(),
        },
        AgentSummary {
            agent_id: AgentId::from_uuid(
                Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            ),
            agent_type: AgentType::Supervisor,
            availability: AgentAvailability::Busy,
            name: "supervisor-1".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;

    fn test_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    async fn health_check_returns_healthy() {
        let state = test_state();
        let Json(response) = health_check(State(state)).await;
        assert_eq!(response.status, "healthy");
        assert!(!response.components.is_empty());
    }

    #[tokio::test]
    async fn list_agents_returns_agents() {
        let state = test_state();
        let query = Query(AgentListQuery::default());
        let Json(agents) = list_agents(State(state), query).await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn list_agents_filter_by_type() {
        let state = test_state();
        let query = Query(AgentListQuery {
            agent_type: Some("Worker".to_string()),
            availability: None,
        });
        let Json(agents) = list_agents(State(state), query).await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "worker-1");
    }

    #[tokio::test]
    async fn get_agent_found() {
        let state = test_state();
        let result = get_agent(
            State(state),
            Path("00000000-0000-0000-0000-000000000001".to_string()),
        )
        .await;
        assert!(result.is_ok());
        let Json(detail) = result.unwrap();
        assert_eq!(detail.name, "worker-1");
    }

    #[tokio::test]
    async fn get_agent_not_found() {
        let state = test_state();
        let result = get_agent(
            State(state),
            Path("00000000-0000-0000-0000-000000000099".to_string()),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_agent_invalid_id() {
        let state = test_state();
        let result = get_agent(State(state), Path("not-a-uuid".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_task_returns_202_fields() {
        let state = test_state();
        let request = CreateTaskRequest {
            description: "Test task".to_string(),
            agent_type: None,
            priority: None,
        };
        let (_status, Json(response)) = create_task(State(state), Json(request)).await;
        assert_eq!(response.status, "pending");
    }

    #[tokio::test]
    async fn get_task_valid_id() {
        let state = test_state();
        let task_id = Uuid::new_v4().to_string();
        let result = get_task(State(state), Path(task_id)).await;
        assert!(result.is_ok());
        let Json(status) = result.unwrap();
        assert_eq!(status.status, "completed");
    }

    #[tokio::test]
    async fn get_config_returns_version() {
        let state = test_state();
        let Json(config) = get_config(State(state)).await;
        assert_eq!(config.version, "0.1.0");
    }
}

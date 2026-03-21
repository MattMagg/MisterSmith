//! Registry-backed agent inspection service for operator surfaces.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mister_smith_core::{AgentAvailability, AgentId, AgentType};
use mister_smith_http::server::{
    AgentInspectionDetailView, AgentInspectionService, AgentInspectionSummaryView,
};
use mister_smith_persistence::postgres::queries;
use sqlx::PgPool;

/// Registry-backed implementation of the HTTP agent inspection contract.
#[derive(Clone)]
pub(crate) struct RegistryAgentInspectionService {
    pool: PgPool,
}

impl RegistryAgentInspectionService {
    /// Create a new registry-backed service from a shared pool.
    pub(crate) fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl AgentInspectionService for RegistryAgentInspectionService {
    async fn list_agents(&self) -> Result<Vec<AgentInspectionSummaryView>, String> {
        let records = queries::list_agents(&self.pool)
            .await
            .map_err(|error| format!("failed to list agents from registry: {error}"))?;

        records
            .iter()
            .map(build_summary_view)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_agent(
        &self,
        agent_id: AgentId,
    ) -> Result<Option<AgentInspectionDetailView>, String> {
        let record = queries::find_agent(&self.pool, *agent_id.as_ref())
            .await
            .map_err(|error| format!("failed to load agent {agent_id}: {error}"))?;

        record.as_ref().map(build_detail_view).transpose()
    }
}

fn build_summary_view(record: &queries::AgentRecord) -> Result<AgentInspectionSummaryView, String> {
    Ok(AgentInspectionSummaryView {
        agent_id: AgentId::from_uuid(record.agent_id),
        agent_type: parse_agent_type(&record.agent_type)?,
        availability: availability_from_status(&record.status, record.last_heartbeat),
        name: record.agent_name.clone(),
        status: record.status.clone(),
        last_heartbeat: record.last_heartbeat,
    })
}

fn build_detail_view(record: &queries::AgentRecord) -> Result<AgentInspectionDetailView, String> {
    Ok(AgentInspectionDetailView {
        agent_id: AgentId::from_uuid(record.agent_id),
        agent_type: parse_agent_type(&record.agent_type)?,
        availability: availability_from_status(&record.status, record.last_heartbeat),
        name: record.agent_name.clone(),
        status: record.status.clone(),
        last_heartbeat: record.last_heartbeat,
        metadata: record.metadata.clone(),
    })
}

fn parse_agent_type(raw: &str) -> Result<AgentType, String> {
    match raw.to_ascii_lowercase().as_str() {
        "supervisor" => Ok(AgentType::Supervisor),
        "worker" => Ok(AgentType::Worker),
        "coordinator" => Ok(AgentType::Coordinator),
        "monitor" => Ok(AgentType::Monitor),
        "planner" => Ok(AgentType::Planner),
        "executor" => Ok(AgentType::Executor),
        "critic" => Ok(AgentType::Critic),
        "router" => Ok(AgentType::Router),
        "memory" => Ok(AgentType::Memory),
        other => Err(format!("unknown persisted agent type '{other}'")),
    }
}

fn availability_from_status(
    status: &str,
    last_heartbeat: Option<DateTime<Utc>>,
) -> AgentAvailability {
    match status.to_ascii_lowercase().as_str() {
        "initializing" => AgentAvailability::Starting,
        "active" => AgentAvailability::Busy,
        "idle" => AgentAvailability::Idle,
        "error" => AgentAvailability::Error,
        "terminated" | "suspended" => AgentAvailability::Offline,
        "stopping" if last_heartbeat.is_some() => AgentAvailability::Stopping,
        _ => AgentAvailability::Offline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn availability_mapping_uses_persisted_status() {
        assert_eq!(
            availability_from_status("initializing", None),
            AgentAvailability::Starting
        );
        assert_eq!(
            availability_from_status("active", Some(Utc::now())),
            AgentAvailability::Busy
        );
        assert_eq!(
            availability_from_status("idle", None),
            AgentAvailability::Idle
        );
        assert_eq!(
            availability_from_status("error", None),
            AgentAvailability::Error
        );
        assert_eq!(
            availability_from_status("terminated", Some(Utc::now())),
            AgentAvailability::Offline
        );
    }

    #[test]
    fn parse_agent_type_accepts_known_registry_values() {
        assert_eq!(parse_agent_type("worker").unwrap(), AgentType::Worker);
        assert_eq!(
            parse_agent_type("Supervisor").unwrap(),
            AgentType::Supervisor
        );
    }
}

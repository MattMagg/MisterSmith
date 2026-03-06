//! System service — gRPC service for system events, configuration, and metrics.
//!
//! Provides methods matching the `SystemService` proto definition:
//! - `stream_events` — server-streaming system events with severity filter
//! - `get_config` — retrieve current system configuration
//! - `update_config` — update a configuration key
//! - `get_metrics` — retrieve system metrics snapshot

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

use crate::proto::{
    GetConfigRequest, GetConfigResponse, GetMetricsRequest, GetMetricsResponse, Severity,
    StreamEventsRequest, SystemEvent, UpdateConfigRequest, UpdateConfigResponse,
};

/// Type alias for server-streaming event response.
pub type SystemEventStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<SystemEvent, Status>> + Send + 'static>>;

/// Mock configuration store.
#[derive(Debug, Clone)]
struct ConfigStore {
    entries: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for ConfigStore {
    fn default() -> Self {
        let mut entries = HashMap::new();
        entries.insert("runtime.workers".to_string(), "4".to_string());
        entries.insert("runtime.max_blocking".to_string(), "512".to_string());
        entries.insert(
            "transport.max_message_size".to_string(),
            "4194304".to_string(),
        );
        entries.insert(
            "monitoring.health_interval_ms".to_string(),
            "5000".to_string(),
        );

        Self {
            entries: Arc::new(RwLock::new(entries)),
        }
    }
}

/// Implementation of the SystemService gRPC server.
///
/// Provides system-wide operations: event streaming, configuration management,
/// and metrics collection. In production, this would integrate with the
/// `mister-smith-events` `EventBus`, `mister-smith-config`, and
/// `mister-smith-monitoring` crates.
#[derive(Debug, Clone)]
pub struct SystemServiceImpl {
    config: ConfigStore,
}

impl Default for SystemServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemServiceImpl {
    /// Create a new `SystemServiceImpl` with default mock data.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ConfigStore::default(),
        }
    }

    /// Stream system events, optionally filtered by event type and minimum severity.
    pub async fn stream_events(
        &self,
        request: Request<StreamEventsRequest>,
    ) -> Result<Response<SystemEventStream>, Status> {
        let req = request.into_inner();
        let event_type_filter = req.event_types;
        let min_severity = req.min_severity;

        info!(
            event_types = ?event_type_filter,
            min_severity = ?min_severity,
            "starting system event stream"
        );

        let stream = async_stream::try_stream! {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

            // Mock events to emit.
            let mock_events = vec![
                ("agent.started", "supervisor", Severity::Info as i32, "Agent agent-001 started"),
                ("agent.task_completed", "agent-001", Severity::Info as i32, "Task task-123 completed"),
                ("system.memory_warning", "monitor", Severity::Warning as i32, "Memory usage above 80%"),
                ("agent.error", "agent-002", Severity::Error as i32, "Agent encountered processing error"),
                ("system.shutdown", "runtime", Severity::Critical as i32, "System shutting down"),
            ];

            for (event_type, source, severity, message) in mock_events {
                interval.tick().await;

                // Apply severity filter.
                if let Some(min) = min_severity {
                    if severity < min {
                        continue;
                    }
                }

                // Apply event type filter.
                if !event_type_filter.is_empty()
                    && !event_type_filter.iter().any(|f| event_type.starts_with(f.as_str()))
                {
                    continue;
                }

                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default();

                yield SystemEvent {
                    event_type: event_type.to_string(),
                    source: source.to_string(),
                    severity,
                    message: message.to_string(),
                    data: None,
                    timestamp: Some(prost_types::Timestamp {
                        seconds: now.as_secs() as i64,
                        nanos: 0,
                    }),
                };
            }
        };

        Ok(Response::new(Box::pin(stream) as SystemEventStream))
    }

    /// Get current system configuration, optionally filtered by component prefix.
    pub async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let component = request.into_inner().component;
        debug!(component = ?component, "getting config");

        let entries = self.config.entries.read().await;
        let config: HashMap<String, String> = match component {
            Some(ref prefix) if !prefix.is_empty() => entries
                .iter()
                .filter(|(k, _)| k.starts_with(prefix.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            _ => entries.clone(),
        };

        info!(count = config.len(), "returned config entries");
        Ok(Response::new(GetConfigResponse { config }))
    }

    /// Update a configuration key-value pair.
    pub async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        let req = request.into_inner();

        if req.component.is_empty() || req.key.is_empty() {
            return Err(Status::invalid_argument("component and key are required"));
        }

        let component = &req.component;
        let key = &req.key;
        let full_key = format!("{component}.{key}");
        debug!(key = %full_key, value = %req.value, "updating config");

        let mut entries = self.config.entries.write().await;
        let previous_value = entries.insert(full_key.clone(), req.value);

        info!(key = %full_key, had_previous = previous_value.is_some(), "config updated");
        Ok(Response::new(UpdateConfigResponse {
            success: true,
            previous_value,
        }))
    }

    /// Get a snapshot of system metrics, optionally filtered by component.
    pub async fn get_metrics(
        &self,
        request: Request<GetMetricsRequest>,
    ) -> Result<Response<GetMetricsResponse>, Status> {
        let component = request.into_inner().component;
        debug!(component = ?component, "getting metrics");

        // Mock metrics data.
        let all_metrics: HashMap<String, f64> = HashMap::from([
            ("runtime.active_tasks".to_string(), 12.0),
            ("runtime.thread_count".to_string(), 8.0),
            ("transport.messages_sent".to_string(), 1547.0),
            ("transport.messages_received".to_string(), 1523.0),
            ("monitoring.health_checks".to_string(), 342.0),
            ("monitoring.uptime_seconds".to_string(), 3600.0),
        ]);

        let metrics: HashMap<String, f64> = match component {
            Some(ref prefix) if !prefix.is_empty() => all_metrics
                .into_iter()
                .filter(|(k, _)| k.starts_with(prefix.as_str()))
                .collect(),
            _ => all_metrics,
        };

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();

        info!(count = metrics.len(), "returned metrics");
        Ok(Response::new(GetMetricsResponse {
            metrics,
            collected_at: Some(prost_types::Timestamp {
                seconds: now.as_secs() as i64,
                nanos: 0,
            }),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn get_config_all() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .get_config(Request::new(GetConfigRequest { component: None }))
            .await
            .unwrap();
        let config = resp.into_inner().config;
        assert_eq!(config.len(), 4);
        assert_eq!(config["runtime.workers"], "4");
    }

    #[tokio::test]
    async fn get_config_filtered() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .get_config(Request::new(GetConfigRequest {
                component: Some("runtime".to_string()),
            }))
            .await
            .unwrap();
        let config = resp.into_inner().config;
        assert_eq!(config.len(), 2);
        assert!(config.contains_key("runtime.workers"));
        assert!(config.contains_key("runtime.max_blocking"));
    }

    #[tokio::test]
    async fn update_config_new_key() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .update_config(Request::new(UpdateConfigRequest {
                component: "transport".to_string(),
                key: "compression".to_string(),
                value: "gzip".to_string(),
            }))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert!(inner.success);
        assert!(inner.previous_value.is_none());

        // Verify it was stored.
        let config_resp = svc
            .get_config(Request::new(GetConfigRequest {
                component: Some("transport".to_string()),
            }))
            .await
            .unwrap();
        let config = config_resp.into_inner().config;
        assert_eq!(config["transport.compression"], "gzip");
    }

    #[tokio::test]
    async fn update_config_existing_key() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .update_config(Request::new(UpdateConfigRequest {
                component: "runtime".to_string(),
                key: "workers".to_string(),
                value: "8".to_string(),
            }))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert!(inner.success);
        assert_eq!(inner.previous_value, Some("4".to_string()));
    }

    #[tokio::test]
    async fn update_config_empty_component() {
        let svc = SystemServiceImpl::new();
        let err = svc
            .update_config(Request::new(UpdateConfigRequest {
                component: "".to_string(),
                key: "workers".to_string(),
                value: "8".to_string(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn update_config_empty_key() {
        let svc = SystemServiceImpl::new();
        let err = svc
            .update_config(Request::new(UpdateConfigRequest {
                component: "runtime".to_string(),
                key: "".to_string(),
                value: "8".to_string(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn get_metrics_all() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .get_metrics(Request::new(GetMetricsRequest { component: None }))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.metrics.len(), 6);
        assert!(inner.collected_at.is_some());
    }

    #[tokio::test]
    async fn get_metrics_filtered() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .get_metrics(Request::new(GetMetricsRequest {
                component: Some("monitoring".to_string()),
            }))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.metrics.len(), 2);
        assert!(inner.metrics.contains_key("monitoring.health_checks"));
        assert!(inner.metrics.contains_key("monitoring.uptime_seconds"));
    }

    #[tokio::test]
    async fn stream_events_all() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .stream_events(Request::new(StreamEventsRequest {
                event_types: vec![],
                min_severity: None,
            }))
            .await
            .unwrap();

        let mut stream = resp.into_inner();
        let mut events = Vec::new();
        while let Some(item) = stream.next().await {
            events.push(item.unwrap());
        }
        assert_eq!(events.len(), 5);
    }

    #[tokio::test]
    async fn stream_events_severity_filter() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .stream_events(Request::new(StreamEventsRequest {
                event_types: vec![],
                min_severity: Some(Severity::Warning as i32),
            }))
            .await
            .unwrap();

        let mut stream = resp.into_inner();
        let mut events = Vec::new();
        while let Some(item) = stream.next().await {
            events.push(item.unwrap());
        }
        // Warning (1), Error (2), Critical (3) pass the filter.
        assert_eq!(events.len(), 3);
        for event in &events {
            assert!(event.severity >= Severity::Warning as i32);
        }
    }

    #[tokio::test]
    async fn stream_events_type_filter() {
        let svc = SystemServiceImpl::new();
        let resp = svc
            .stream_events(Request::new(StreamEventsRequest {
                event_types: vec!["agent".to_string()],
                min_severity: None,
            }))
            .await
            .unwrap();

        let mut stream = resp.into_inner();
        let mut events = Vec::new();
        while let Some(item) = stream.next().await {
            events.push(item.unwrap());
        }
        // agent.started, agent.task_completed, agent.error = 3 events.
        assert_eq!(events.len(), 3);
        for event in &events {
            assert!(event.event_type.starts_with("agent"));
        }
    }
}

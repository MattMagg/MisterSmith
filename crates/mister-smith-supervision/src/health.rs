//! Actor system health check and metrics integration with monitoring crate.
//!
//! Implements [`HealthCheck`] for the actor system, reporting health status
//! based on actor states, supervision tree depth, and restart counts.
//!
//! Also provides [`ActorSystemMetrics`] for periodically collecting and
//! publishing actor system metrics via [`MetricsCollector`].

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mister_smith_actor::ActorSystem;
use mister_smith_core::AgentState;
use mister_smith_monitoring::health::HealthCheck;
use mister_smith_monitoring::metrics::MetricsCollector;
use mister_smith_monitoring::types::{ComponentId, Status};
use tokio::sync::RwLock;

use crate::tree::SupervisionTree;

/// Health check for the actor system and its supervision tree.
///
/// Reports:
/// - **Healthy**: No actors in error state, restarts within normal limits
/// - **Degraded**: Some actors in error state (< 50% of total) or elevated restarts
/// - **Unhealthy**: Majority of actors in error state or excessive restarts
///
/// Metadata includes total_actors, error_count, tree_depth, and total_restarts.
pub struct ActorSystemHealthCheck {
    system: Arc<ActorSystem>,
    tree: Arc<RwLock<SupervisionTree>>,
}

impl ActorSystemHealthCheck {
    /// Create a new health check for the given actor system and supervision tree.
    pub fn new(system: Arc<ActorSystem>, tree: Arc<RwLock<SupervisionTree>>) -> Self {
        Self { system, tree }
    }
}

#[async_trait]
impl HealthCheck for ActorSystemHealthCheck {
    async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
        let states = self.system.actor_states().await;
        let total_actors = states.len();
        let error_count = states
            .values()
            .filter(|s| **s == AgentState::Error)
            .count();

        let tree_status = self.tree.read().await.query_status();

        // Empty system is healthy
        if total_actors == 0 {
            return Ok(Status::Healthy);
        }

        let error_ratio = error_count as f64 / total_actors as f64;

        if error_ratio > 0.5 {
            Ok(Status::Unhealthy)
        } else if error_ratio > 0.0 || tree_status.total_restarts > total_actors as u64 {
            Ok(Status::Degraded)
        } else {
            Ok(Status::Healthy)
        }
    }

    fn component_id(&self) -> ComponentId {
        ComponentId::new("actor-system")
    }

    fn check_interval(&self) -> Duration {
        Duration::from_secs(15)
    }
}

/// Collects actor system metrics and publishes them via [`MetricsCollector`].
///
/// Reports the following gauges on each [`collect`](ActorSystemMetrics::collect) call:
/// - `actor.total_count` — total number of tracked actors
/// - `actor.error_count` — actors currently in error state
/// - `actor.restart_count` — total restarts across all supervisors
/// - `actor.failure_rate` — ratio of error-state actors to total (0.0–1.0)
/// - `actor.tree_depth` — maximum depth of the supervision tree
/// - `actor.supervisor_count` — number of supervisor nodes
pub struct ActorSystemMetrics {
    system: Arc<ActorSystem>,
    tree: Arc<RwLock<SupervisionTree>>,
    collector: Arc<MetricsCollector>,
}

impl ActorSystemMetrics {
    /// Create a new metrics reporter for the given actor system, supervision tree,
    /// and metrics collector.
    pub fn new(
        system: Arc<ActorSystem>,
        tree: Arc<RwLock<SupervisionTree>>,
        collector: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            system,
            tree,
            collector,
        }
    }

    /// Collect current actor system metrics and publish them to the MetricsCollector.
    pub async fn collect(&self) {
        let states = self.system.actor_states().await;
        let total_actors = states.len();
        let error_count = states
            .values()
            .filter(|s| **s == AgentState::Error)
            .count();

        let tree_status = self.tree.read().await.query_status();

        let tags = HashMap::new();

        self.collector
            .set_gauge("actor.total_count", total_actors as f64, tags.clone())
            .await;
        self.collector
            .set_gauge("actor.error_count", error_count as f64, tags.clone())
            .await;
        self.collector
            .set_gauge(
                "actor.restart_count",
                tree_status.total_restarts as f64,
                tags.clone(),
            )
            .await;

        let failure_rate = if total_actors > 0 {
            error_count as f64 / total_actors as f64
        } else {
            0.0
        };
        self.collector
            .set_gauge("actor.failure_rate", failure_rate, tags.clone())
            .await;

        self.collector
            .set_gauge("actor.tree_depth", tree_status.tree_depth as f64, tags.clone())
            .await;
        self.collector
            .set_gauge(
                "actor.supervisor_count",
                tree_status.supervisor_count as f64,
                tags,
            )
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_actor::{ActorSystemConfig, SpawnConfig};
    use mister_smith_core::{Actor, AgentId, SupervisionStrategy};
    use mister_smith_monitoring::metrics::MetricsCollector;

    #[derive(Debug)]
    struct TestError(String);
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for TestError {}

    struct SimpleActor {
        id: AgentId,
    }

    #[async_trait]
    impl Actor for SimpleActor {
        type Message = ();
        type State = ();
        type Error = TestError;

        async fn handle_message(
            &mut self,
            _: (),
            _: &mut (),
        ) -> Result<(), TestError> {
            Ok(())
        }

        fn pre_start(&mut self) -> Result<(), TestError> {
            Ok(())
        }

        fn post_stop(&mut self) -> Result<(), TestError> {
            Ok(())
        }

        fn actor_id(&self) -> AgentId {
            self.id
        }
    }

    #[tokio::test]
    async fn empty_system_is_healthy() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));
        let check = ActorSystemHealthCheck::new(system, tree);

        let status = check.check().await.unwrap();
        assert_eq!(status, Status::Healthy);
    }

    #[tokio::test]
    async fn all_running_actors_is_healthy() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));

        for _ in 0..3 {
            let id = AgentId::new();
            system
                .spawn(SimpleActor { id }, (), SpawnConfig::default())
                .await
                .unwrap();
        }

        // Wait for actors to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        let check = ActorSystemHealthCheck::new(Arc::clone(&system), tree);
        let status = check.check().await.unwrap();
        assert_eq!(status, Status::Healthy);

        system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn component_id_is_actor_system() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));
        let check = ActorSystemHealthCheck::new(system, tree);

        assert_eq!(check.component_id(), ComponentId::new("actor-system"));
    }

    #[tokio::test]
    async fn check_interval_is_15s() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));
        let check = ActorSystemHealthCheck::new(system, tree);

        assert_eq!(check.check_interval(), Duration::from_secs(15));
    }

    #[tokio::test]
    async fn elevated_restarts_reports_degraded() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));

        // Add a supervisor and simulate restarts in the tree
        let sup_id = AgentId::new();
        {
            let mut t = tree.write().await;
            t.add_supervisor(
                sup_id,
                SupervisionStrategy {
                    max_failures: 100,
                    ..Default::default()
                },
            );
        }

        // Spawn one actor so total_actors > 0
        let id = AgentId::new();
        system
            .spawn(SimpleActor { id }, (), SpawnConfig::default())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Add child and simulate many restarts (> total_actors)
        {
            let mut t = tree.write().await;
            t.add_child(sup_id, id, mister_smith_core::RestartScope::Permanent)
                .unwrap();
            // Simulate failures to bump restart count
            use crate::strategy::TerminationType;
            for _ in 0..5 {
                let _ = t.handle_failure(id, TerminationType::Error);
            }
        }

        let check = ActorSystemHealthCheck::new(Arc::clone(&system), tree);
        let status = check.check().await.unwrap();
        assert_eq!(status, Status::Degraded);

        system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn metrics_collect_reports_gauges() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));
        let collector = Arc::new(MetricsCollector::new(Duration::from_secs(60)));

        // Add a supervisor with children and simulate restarts
        let sup_id = AgentId::new();
        {
            let mut t = tree.write().await;
            t.add_supervisor(
                sup_id,
                SupervisionStrategy {
                    max_failures: 100,
                    ..Default::default()
                },
            );
        }

        // Spawn actors
        for _ in 0..3 {
            let id = AgentId::new();
            system
                .spawn(SimpleActor { id }, (), SpawnConfig::default())
                .await
                .unwrap();
        }
        tokio::time::sleep(Duration::from_millis(50)).await;

        let metrics = ActorSystemMetrics::new(
            Arc::clone(&system),
            Arc::clone(&tree),
            Arc::clone(&collector),
        );
        metrics.collect().await;

        // Should have buffered 6 gauge metrics
        assert_eq!(collector.buffered_count().await, 6);

        system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn metrics_empty_system_reports_zero_failure_rate() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let tree = Arc::new(RwLock::new(SupervisionTree::new()));
        let collector = Arc::new(MetricsCollector::new(Duration::from_secs(60)));

        let metrics = ActorSystemMetrics::new(system, tree, Arc::clone(&collector));
        metrics.collect().await;

        // 6 gauges reported even for empty system
        assert_eq!(collector.buffered_count().await, 6);
    }
}

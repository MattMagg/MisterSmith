//! Supervised actor system: integrates actor lifecycle with supervision trees.
//!
//! Provides [`SupervisedSystem`] which wraps an [`ActorSystem`] and a
//! [`SupervisionTree`], enabling automatic restart of failed actors
//! based on supervision strategies.

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use mister_smith_actor::actor_cell::{SupervisionNotification, TerminationReason};
use mister_smith_actor::{ActorRef, ActorSystem, ActorSystemConfig, SpawnConfig};
use mister_smith_core::{Actor, ActorError, AgentId, EventPublisher, SupervisionError, SupervisionStrategy};
use mister_smith_events::EventBus;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::escalation::escalate;
use crate::events as supervision_events;
use crate::strategy::{SupervisionDecision, TerminationType};
use crate::tree::SupervisionTree;

/// Type-erased trait for restarting actors without knowing their concrete type.
#[async_trait]
trait ActorRestarter: Send + Sync {
    /// Restart the actor by creating a new instance and spawning it.
    async fn restart(&self, system: &ActorSystem) -> Result<AgentId, ActorError>;
}

/// Typed implementation of [`ActorRestarter`] for a specific actor type.
struct TypedRestarter<F, A: Actor> {
    factory: F,
    config: SpawnConfig,
    _phantom: PhantomData<fn() -> A>,
}

#[async_trait]
impl<F, A> ActorRestarter for TypedRestarter<F, A>
where
    F: Fn() -> (A, A::State) + Send + Sync + 'static,
    A: Actor + 'static,
    A::Message: Send + 'static,
    A::State: Send + 'static,
{
    async fn restart(&self, system: &ActorSystem) -> Result<AgentId, ActorError> {
        let (actor, state) = (self.factory)();
        let actor_id = actor.actor_id();
        let _ref = system.spawn(actor, state, self.config.clone()).await?;
        Ok(actor_id)
    }
}

/// A supervised actor system that integrates actor lifecycle management
/// with supervision trees for automatic fault recovery.
///
/// Wraps an [`ActorSystem`] and a [`SupervisionTree`], providing methods
/// to create supervisors, spawn supervised actors with restart factories,
/// and run a background supervision loop that automatically restarts
/// failed actors according to their supervisor's strategy.
pub struct SupervisedSystem {
    system: Arc<ActorSystem>,
    tree: Arc<RwLock<SupervisionTree>>,
    factories: Arc<RwLock<HashMap<AgentId, Box<dyn ActorRestarter>>>>,
    shutting_down: Arc<AtomicBool>,
    event_bus: Option<Arc<EventBus>>,
}

impl SupervisedSystem {
    /// Create a new supervised system with the given configuration.
    pub fn new(config: ActorSystemConfig) -> Self {
        Self {
            system: Arc::new(ActorSystem::new(config)),
            tree: Arc::new(RwLock::new(SupervisionTree::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
            shutting_down: Arc::new(AtomicBool::new(false)),
            event_bus: None,
        }
    }

    /// Create a new supervised system with an EventBus for lifecycle event emission.
    ///
    /// The EventBus is wired into both the ActorSystem (for actor lifecycle events)
    /// and the supervision loop (for restart/escalation events with correlation IDs).
    pub fn with_event_bus(config: ActorSystemConfig, event_bus: Arc<EventBus>) -> Self {
        let publisher: Arc<dyn EventPublisher> = event_bus.clone();
        let system = ActorSystem::new(config).with_event_publisher(publisher);
        Self {
            system: Arc::new(system),
            tree: Arc::new(RwLock::new(SupervisionTree::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
            shutting_down: Arc::new(AtomicBool::new(false)),
            event_bus: Some(event_bus),
        }
    }

    /// Returns a reference to the underlying actor system.
    pub fn system(&self) -> &ActorSystem {
        &self.system
    }

    /// Create a root-level supervisor with the given strategy.
    pub async fn create_supervisor(&self, strategy: SupervisionStrategy) -> AgentId {
        let id = AgentId::new();
        let mut tree = self.tree.write().await;
        tree.add_supervisor(id, strategy);
        id
    }

    /// Create a supervisor as a child of another supervisor.
    pub async fn create_supervisor_under(
        &self,
        parent_id: AgentId,
        strategy: SupervisionStrategy,
    ) -> Result<AgentId, SupervisionError> {
        let id = AgentId::new();
        let mut tree = self.tree.write().await;
        tree.add_supervisor_under(id, parent_id, strategy)?;
        Ok(id)
    }

    /// Spawn an actor under a supervisor with a factory for restart support.
    ///
    /// The factory closure must produce actors with a consistent, fixed `AgentId`.
    /// On restart, the factory is called again to create a fresh actor and state.
    pub async fn spawn_supervised<A, F>(
        &self,
        supervisor_id: AgentId,
        factory: F,
        config: SpawnConfig,
    ) -> Result<ActorRef<A::Message>, ActorError>
    where
        A: Actor + 'static,
        A::Message: Send + 'static,
        A::State: Send + 'static,
        F: Fn() -> (A, A::State) + Send + Sync + 'static,
    {
        let supervisor_exists = self
            .with_tree(|tree| tree.get_node(&supervisor_id).is_some())
            .await;
        if !supervisor_exists {
            return Err(ActorError::StartupFailed(
                std::io::Error::other(format!("Supervisor {supervisor_id} not found")).into(),
            ));
        }

        let (actor, state) = factory();
        let actor_id = actor.actor_id();
        let restart_scope = config.restart_scope;

        let actor_ref = self.system.spawn(actor, state, config.clone()).await?;

        // Register in supervision tree
        let mut tree = self.tree.write().await;
        if let Err(err) = tree.add_child(supervisor_id, actor_id, restart_scope) {
            drop(tree);
            self.system.stop_actor(&actor_id).await;
            self.factories.write().await.remove(&actor_id);
            return Err(ActorError::StartupFailed(
                std::io::Error::other(err.to_string()).into(),
            ));
        }

        // Store factory for restart
        let restarter: Box<dyn ActorRestarter> = Box::new(TypedRestarter {
            factory,
            config,
            _phantom: PhantomData,
        });
        let mut factories = self.factories.write().await;
        factories.insert(actor_id, restarter);

        Ok(actor_ref)
    }

    /// Start the supervision loop as a background task.
    ///
    /// The loop listens for actor termination notifications and applies
    /// the supervision tree's restart policies automatically.
    pub fn start_supervision(&self) -> JoinHandle<()> {
        let system = Arc::clone(&self.system);
        let tree = Arc::clone(&self.tree);
        let factories = Arc::clone(&self.factories);
        let shutting_down = Arc::clone(&self.shutting_down);
        let event_bus = self.event_bus.clone();

        tokio::spawn(async move {
            supervision_loop(system, tree, factories, shutting_down, event_bus).await;
        })
    }

    /// Returns a reference to the supervision tree (for health check integration).
    pub fn tree(&self) -> &Arc<RwLock<SupervisionTree>> {
        &self.tree
    }

    /// Query the current supervision tree status.
    pub async fn tree_status(&self) -> crate::tree::TreeStatus {
        self.tree.read().await.query_status()
    }

    /// Gracefully shut down the supervised system.
    ///
    /// Disables the supervision loop (preventing restarts during shutdown)
    /// then shuts down all actors.
    pub async fn shutdown(&self) -> Result<(), ActorError> {
        self.shutting_down.store(true, Ordering::SeqCst);
        self.system.shutdown().await
    }

    /// Access the supervision tree for inspection (read-only).
    pub async fn with_tree<R>(&self, f: impl FnOnce(&SupervisionTree) -> R) -> R {
        let tree = self.tree.read().await;
        f(&tree)
    }

    /// Access the supervision tree for mutation.
    pub async fn with_tree_mut<R>(&self, f: impl FnOnce(&mut SupervisionTree) -> R) -> R {
        let mut tree = self.tree.write().await;
        f(&mut tree)
    }
}

/// Main supervision loop that processes actor termination notifications.
async fn supervision_loop(
    system: Arc<ActorSystem>,
    tree: Arc<RwLock<SupervisionTree>>,
    factories: Arc<RwLock<HashMap<AgentId, Box<dyn ActorRestarter>>>>,
    shutting_down: Arc<AtomicBool>,
    event_bus: Option<Arc<EventBus>>,
) {
    let rx = system.supervision_rx();
    let mut rx: tokio::sync::MutexGuard<'_, tokio::sync::mpsc::UnboundedReceiver<SupervisionNotification>> = rx.lock().await;

    // Track actors being intentionally stopped for restart
    let mut pending_restarts: HashSet<AgentId> = HashSet::new();

    while let Some(notification) = rx.recv().await {
        // Check if system is shutting down — stop processing restarts
        if shutting_down.load(Ordering::SeqCst) {
            break;
        }

        let actor_id = notification.actor_id;

        // Skip notifications from intentional stops (sibling restarts)
        if pending_restarts.remove(&actor_id) {
            tracing::debug!(actor_id = %actor_id, "Suppressing notification for supervised stop");
            continue;
        }

        // Create a correlation ID for this failure chain
        let correlation_id = Uuid::new_v4();

        // Emit failure event
        let error_msg = match &notification.reason {
            TerminationReason::Normal => None,
            TerminationReason::Failed(e) => Some(e.clone()),
            TerminationReason::Panicked(e) => Some(format!("panic: {e}")),
            TerminationReason::PreStartFailed(e) => Some(format!("pre_start: {e}")),
        };

        let causation_id = if let (Some(ref bus), Some(ref err)) = (&event_bus, &error_msg) {
            Some(
                supervision_events::emit_failure_event(bus, &actor_id, err, correlation_id).await,
            )
        } else {
            None
        };

        let termination_type = match &notification.reason {
            TerminationReason::Normal => TerminationType::Normal,
            TerminationReason::Failed(_)
            | TerminationReason::Panicked(_)
            | TerminationReason::PreStartFailed(_) => TerminationType::Error,
        };

        // Get the supervision decision
        let decision = {
            let mut tree = tree.write().await;
            match tree.handle_failure(actor_id, termination_type) {
                Ok(decision) => decision,
                Err(_) => {
                    // Actor is not supervised — ignore
                    continue;
                }
            }
        };

        handle_decision(
            &system,
            &tree,
            &factories,
            decision,
            actor_id,
            &mut pending_restarts,
            &event_bus,
            correlation_id,
            causation_id.unwrap_or_else(Uuid::new_v4),
        )
        .await;
    }
}

/// Execute a supervision decision, looping on escalation.
#[allow(clippy::too_many_arguments)]
async fn handle_decision(
    system: &ActorSystem,
    tree: &RwLock<SupervisionTree>,
    factories: &RwLock<HashMap<AgentId, Box<dyn ActorRestarter>>>,
    mut decision: SupervisionDecision,
    failed_actor_id: AgentId,
    pending_restarts: &mut HashSet<AgentId>,
    event_bus: &Option<Arc<EventBus>>,
    correlation_id: Uuid,
    causation_id: Uuid,
) {
    loop {
        match decision {
            SupervisionDecision::Restart(ids) => {
                // Find the supervisor for event emission
                let supervisor_id = {
                    let tree_guard = tree.read().await;
                    tree_guard.find_supervisor(&failed_actor_id)
                };

                // Stop sibling actors that need restarting
                // (the failed actor is already stopped)
                for &id in &ids {
                    if id != failed_actor_id {
                        pending_restarts.insert(id);
                        system.stop_actor(&id).await;
                    }
                }

                // Restart all affected actors using their factories
                let factories_guard = factories.read().await;
                for &id in &ids {
                    if let Some(restarter) = factories_guard.get(&id) {
                        match restarter.restart(system).await {
                            Ok(_) => {
                                tracing::info!(actor_id = %id, "Actor restarted successfully");
                                if let (Some(ref bus), Some(sup_id)) =
                                    (event_bus, supervisor_id)
                                {
                                    supervision_events::emit_restart_event(
                                        bus,
                                        &id,
                                        &sup_id,
                                        correlation_id,
                                        causation_id,
                                    )
                                    .await;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    actor_id = %id,
                                    error = %e,
                                    "Failed to restart actor"
                                );
                            }
                        }
                    }
                }
                break;
            }
            SupervisionDecision::Escalate => {
                if let Some(ref bus) = event_bus {
                    let sup_id = {
                        let tree_guard = tree.read().await;
                        tree_guard.find_supervisor(&failed_actor_id)
                    };
                    if let Some(sup_id) = sup_id {
                        supervision_events::emit_escalation_event(
                            bus,
                            &sup_id,
                            correlation_id,
                            causation_id,
                        )
                        .await;
                    }
                }

                let mut tree_guard = tree.write().await;
                if let Some(sup_id) = tree_guard.find_supervisor(&failed_actor_id) {
                    match escalate(&mut tree_guard, sup_id, "budget exhausted") {
                        Ok(new_decision) => {
                            decision = new_decision;
                            drop(tree_guard);
                            continue;
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Escalation failed");
                            break;
                        }
                    }
                } else {
                    tracing::error!(
                        actor_id = %failed_actor_id,
                        "No supervisor found for escalation"
                    );
                    break;
                }
            }
            SupervisionDecision::Shutdown => {
                tracing::error!("Supervision tree shutdown triggered");
                let _ = system.shutdown().await;
                break;
            }
            // Stop/Ignore are terminal only for the current notification.
            // Return explicitly so supervision_loop can continue processing
            // future actor failures.
            SupervisionDecision::Stop(_) | SupervisionDecision::Ignore => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_core::{RestartPolicy, RestartScope};
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    #[derive(Debug)]
    struct TestError(String);
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for TestError {}

    #[derive(Debug)]
    enum TestMsg {
        Ping,
        Fail,
    }

    struct TestActor {
        id: AgentId,
        pre_start_count: Arc<AtomicU32>,
    }

    #[async_trait]
    impl Actor for TestActor {
        type Message = TestMsg;
        type State = u32;
        type Error = TestError;

        async fn handle_message(
            &mut self,
            message: TestMsg,
            state: &mut u32,
        ) -> Result<(), TestError> {
            match message {
                TestMsg::Ping => {
                    *state += 1;
                    Ok(())
                }
                TestMsg::Fail => Err(TestError("intentional failure".into())),
            }
        }

        fn pre_start(&mut self) -> Result<(), TestError> {
            self.pre_start_count.fetch_add(1, Ordering::SeqCst);
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
    async fn supervised_system_basic_lifecycle() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 5,
                ..Default::default()
            })
            .await;

        let starts = Arc::new(AtomicU32::new(0));
        let actor_id = AgentId::new();
        let starts_clone = Arc::clone(&starts);

        let _ref = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: actor_id,
                            pre_start_count: Arc::clone(&starts_clone),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(starts.load(Ordering::SeqCst), 1);

        let status = supervised.tree_status().await;
        assert_eq!(status.total_nodes, 2); // supervisor + 1 child
        assert_eq!(status.supervisor_count, 1);

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn one_for_one_only_failed_child_restarts() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 5,
                ..Default::default()
            })
            .await;

        let a_starts = Arc::new(AtomicU32::new(0));
        let b_starts = Arc::new(AtomicU32::new(0));
        let c_starts = Arc::new(AtomicU32::new(0));

        let a_id = AgentId::new();
        let b_id = AgentId::new();
        let c_id = AgentId::new();

        let a_s = Arc::clone(&a_starts);
        let _ref_a = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: a_id,
                            pre_start_count: Arc::clone(&a_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let b_s = Arc::clone(&b_starts);
        let ref_b = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: b_id,
                            pre_start_count: Arc::clone(&b_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let c_s = Arc::clone(&c_starts);
        let _ref_c = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: c_id,
                            pre_start_count: Arc::clone(&c_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let _handle = supervised.start_supervision();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Kill B
        ref_b.tell(TestMsg::Fail).unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Only B restarted (pre_start called twice)
        assert_eq!(a_starts.load(Ordering::SeqCst), 1);
        assert_eq!(b_starts.load(Ordering::SeqCst), 2);
        assert_eq!(c_starts.load(Ordering::SeqCst), 1);

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn one_for_all_restarts_all_children() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: RestartPolicy::OneForAll,
                max_failures: 5,
                ..Default::default()
            })
            .await;

        let a_starts = Arc::new(AtomicU32::new(0));
        let b_starts = Arc::new(AtomicU32::new(0));
        let c_starts = Arc::new(AtomicU32::new(0));

        let a_id = AgentId::new();
        let b_id = AgentId::new();
        let c_id = AgentId::new();

        let a_s = Arc::clone(&a_starts);
        let _ref_a = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: a_id,
                            pre_start_count: Arc::clone(&a_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let b_s = Arc::clone(&b_starts);
        let ref_b = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: b_id,
                            pre_start_count: Arc::clone(&b_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let c_s = Arc::clone(&c_starts);
        let _ref_c = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: c_id,
                            pre_start_count: Arc::clone(&c_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let _handle = supervised.start_supervision();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Kill B — all should restart
        ref_b.tell(TestMsg::Fail).unwrap();
        // OneForAll stops 2 siblings (each with ~100ms drain timeout) then restarts 3
        tokio::time::sleep(Duration::from_millis(500)).await;

        // All started twice
        assert_eq!(a_starts.load(Ordering::SeqCst), 2);
        assert_eq!(b_starts.load(Ordering::SeqCst), 2);
        assert_eq!(c_starts.load(Ordering::SeqCst), 2);

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn rest_for_one_restarts_failed_and_younger() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: RestartPolicy::RestForOne,
                max_failures: 5,
                ..Default::default()
            })
            .await;

        let a_starts = Arc::new(AtomicU32::new(0));
        let b_starts = Arc::new(AtomicU32::new(0));
        let c_starts = Arc::new(AtomicU32::new(0));

        let a_id = AgentId::new();
        let b_id = AgentId::new();
        let c_id = AgentId::new();

        let a_s = Arc::clone(&a_starts);
        let _ref_a = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: a_id,
                            pre_start_count: Arc::clone(&a_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let b_s = Arc::clone(&b_starts);
        let ref_b = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: b_id,
                            pre_start_count: Arc::clone(&b_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let c_s = Arc::clone(&c_starts);
        let _ref_c = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: c_id,
                            pre_start_count: Arc::clone(&c_s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let _handle = supervised.start_supervision();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Kill B — B and C should restart, A should not
        ref_b.tell(TestMsg::Fail).unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;

        assert_eq!(a_starts.load(Ordering::SeqCst), 1);
        assert_eq!(b_starts.load(Ordering::SeqCst), 2);
        assert_eq!(c_starts.load(Ordering::SeqCst), 2);

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn restarted_actor_has_fresh_state_and_same_id() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 5,
                ..Default::default()
            })
            .await;

        let starts = Arc::new(AtomicU32::new(0));
        let actor_id = AgentId::new();
        let s = Arc::clone(&starts);

        let ref_a = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: actor_id,
                            pre_start_count: Arc::clone(&s),
                        },
                        0, // fresh state each time
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let _handle = supervised.start_supervision();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify initial state
        assert_eq!(ref_a.actor_id(), actor_id);
        assert_eq!(starts.load(Ordering::SeqCst), 1);

        // Trigger failure
        ref_a.tell(TestMsg::Fail).unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;

        // pre_start called again (fresh actor), same ID preserved in tree
        assert_eq!(starts.load(Ordering::SeqCst), 2);

        // The tree still tracks the same actor ID
        let has_child = supervised
            .with_tree(|tree| tree.find_supervisor(&actor_id).is_some())
            .await;
        assert!(has_child);

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn create_supervisor_under_hierarchy() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let root_id = supervised
            .create_supervisor(SupervisionStrategy::default())
            .await;

        let mid_id = supervised
            .create_supervisor_under(root_id, SupervisionStrategy::default())
            .await
            .unwrap();

        let status = supervised.tree_status().await;
        assert_eq!(status.supervisor_count, 2);
        assert_eq!(status.tree_depth, 2);

        // mid is a child of root
        let parent = supervised
            .with_tree(|tree| tree.find_supervisor(&mid_id))
            .await;
        assert_eq!(parent, Some(root_id));

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn transient_child_not_restarted_on_normal_stop() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());

        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 5,
                ..Default::default()
            })
            .await;

        let starts = Arc::new(AtomicU32::new(0));
        let actor_id = AgentId::new();
        let s = Arc::clone(&starts);

        let _ref = supervised
            .spawn_supervised::<TestActor, _>(
                sup_id,
                move || {
                    (
                        TestActor {
                            id: actor_id,
                            pre_start_count: Arc::clone(&s),
                        },
                        0,
                    )
                },
                SpawnConfig {
                    restart_scope: RestartScope::Transient,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let _handle = supervised.start_supervision();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop the actor normally (by stopping it through the system)
        supervised.system().stop_actor(&actor_id).await;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Transient actor should NOT be restarted on normal stop
        assert_eq!(starts.load(Ordering::SeqCst), 1);

        supervised.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn spawn_supervised_fails_atomically_for_missing_supervisor() {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());
        let initial_actor_count = supervised.system().actor_count().await;

        let starts = Arc::new(AtomicU32::new(0));
        let actor_id = AgentId::new();
        let s = Arc::clone(&starts);

        let result = supervised
            .spawn_supervised::<TestActor, _>(
                AgentId::new(),
                move || {
                    (
                        TestActor {
                            id: actor_id,
                            pre_start_count: Arc::clone(&s),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await;

        assert!(matches!(result, Err(ActorError::StartupFailed(_))));
        assert_eq!(supervised.system().actor_count().await, initial_actor_count);
    }
}

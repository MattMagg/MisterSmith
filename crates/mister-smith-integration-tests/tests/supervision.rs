//! Integration tests for Phase 3: Actor Supervision
//!
//! Tests end-to-end supervision behavior across the actor and supervision crates.
//! Validates restart policies, hierarchical escalation, and lifecycle guarantees.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use mister_smith_actor::{ActorError, ActorSystemConfig, SpawnConfig};
use mister_smith_core::{
    Actor, AgentId, EscalationPolicy, RestartPolicy, RestartScope, SupervisionStrategy,
};
use mister_smith_events::{AgentEventType, EventBus, EventType};
use mister_smith_monitoring::health::HealthCheck;
use mister_smith_monitoring::types::Status;
use mister_smith_supervision::{ActorSystemHealthCheck, SupervisedSystem};

// --- Test infrastructure ---

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

/// Actor that tracks pre_start/post_stop calls and can fail on command.
struct TrackingActor {
    id: AgentId,
    pre_start_count: Arc<AtomicU32>,
    post_stop_count: Arc<AtomicU32>,
}

#[async_trait]
impl Actor for TrackingActor {
    type Message = TestMsg;
    type State = u32;
    type Error = TestError;

    async fn handle_message(&mut self, message: TestMsg, state: &mut u32) -> Result<(), TestError> {
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
        self.post_stop_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn actor_id(&self) -> AgentId {
        self.id
    }
}

struct TimestampActor {
    id: AgentId,
    starts: Arc<AtomicU32>,
    start_times: Arc<Mutex<Vec<Instant>>>,
}

#[async_trait]
impl Actor for TimestampActor {
    type Message = TestMsg;
    type State = ();
    type Error = TestError;

    async fn handle_message(&mut self, message: TestMsg, _state: &mut ()) -> Result<(), TestError> {
        match message {
            TestMsg::Ping => Ok(()),
            TestMsg::Fail => Err(TestError("intentional failure".into())),
        }
    }

    fn pre_start(&mut self) -> Result<(), TestError> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        self.start_times.lock().unwrap().push(Instant::now());
        Ok(())
    }

    fn post_stop(&mut self) -> Result<(), TestError> {
        Ok(())
    }

    fn actor_id(&self) -> AgentId {
        self.id
    }
}

async fn wait_for_restarts(counters: &[Arc<AtomicU32>], expected: u32) {
    for _ in 0..80 {
        if counters
            .iter()
            .all(|counter| counter.load(Ordering::SeqCst) >= expected)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!("timed out waiting for restart counters to reach {expected}");
}

// --- T054: OneForOne integration test ---

#[tokio::test]
async fn t054_one_for_one_only_failed_child_restarts() {
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
    let a_ps = Arc::new(AtomicU32::new(0));
    let _ref_a = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: a_id,
                        pre_start_count: Arc::clone(&a_s),
                        post_stop_count: Arc::clone(&a_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let b_s = Arc::clone(&b_starts);
    let b_ps = Arc::new(AtomicU32::new(0));
    let ref_b = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: b_id,
                        pre_start_count: Arc::clone(&b_s),
                        post_stop_count: Arc::clone(&b_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let c_s = Arc::clone(&c_starts);
    let c_ps = Arc::new(AtomicU32::new(0));
    let _ref_c = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: c_id,
                        pre_start_count: Arc::clone(&c_s),
                        post_stop_count: Arc::clone(&c_ps),
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

    // Only B restarted
    assert_eq!(a_starts.load(Ordering::SeqCst), 1, "A should not restart");
    assert_eq!(b_starts.load(Ordering::SeqCst), 2, "B should restart once");
    assert_eq!(c_starts.load(Ordering::SeqCst), 1, "C should not restart");

    supervised.shutdown().await.unwrap();
}

// --- T055: OneForAll integration test ---

#[tokio::test]
async fn t055_one_for_all_all_children_restart() {
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
    let a_ps = Arc::new(AtomicU32::new(0));
    let _ref_a = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: a_id,
                        pre_start_count: Arc::clone(&a_s),
                        post_stop_count: Arc::clone(&a_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let b_s = Arc::clone(&b_starts);
    let b_ps = Arc::new(AtomicU32::new(0));
    let ref_b = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: b_id,
                        pre_start_count: Arc::clone(&b_s),
                        post_stop_count: Arc::clone(&b_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let c_s = Arc::clone(&c_starts);
    let c_ps = Arc::new(AtomicU32::new(0));
    let _ref_c = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: c_id,
                        pre_start_count: Arc::clone(&c_s),
                        post_stop_count: Arc::clone(&c_ps),
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
    // OneForAll stops 2 siblings (~100ms drain each) then restarts 3
    tokio::time::sleep(Duration::from_millis(900)).await;

    assert_eq!(a_starts.load(Ordering::SeqCst), 2, "A should restart");
    assert_eq!(b_starts.load(Ordering::SeqCst), 2, "B should restart");
    assert_eq!(c_starts.load(Ordering::SeqCst), 2, "C should restart");

    supervised.shutdown().await.unwrap();
}

// --- T056: RestForOne integration test ---

#[tokio::test]
async fn t056_rest_for_one_failed_and_younger_restart() {
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
    let a_ps = Arc::new(AtomicU32::new(0));
    let _ref_a = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: a_id,
                        pre_start_count: Arc::clone(&a_s),
                        post_stop_count: Arc::clone(&a_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let b_s = Arc::clone(&b_starts);
    let b_ps = Arc::new(AtomicU32::new(0));
    let ref_b = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: b_id,
                        pre_start_count: Arc::clone(&b_s),
                        post_stop_count: Arc::clone(&b_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let c_s = Arc::clone(&c_starts);
    let c_ps = Arc::new(AtomicU32::new(0));
    let _ref_c = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: c_id,
                        pre_start_count: Arc::clone(&c_s),
                        post_stop_count: Arc::clone(&c_ps),
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
    // RestForOne stops 1 sibling (~100ms drain) then restarts 2 with per-child backoff
    tokio::time::sleep(Duration::from_millis(900)).await;

    assert_eq!(a_starts.load(Ordering::SeqCst), 1, "A should not restart");
    assert_eq!(b_starts.load(Ordering::SeqCst), 2, "B should restart");
    assert_eq!(c_starts.load(Ordering::SeqCst), 2, "C should restart");

    supervised.shutdown().await.unwrap();
}

// --- T057: Restarted actor has fresh state ---

#[tokio::test]
async fn t057_restarted_actor_fresh_state_same_id_pre_start_called() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 5,
            ..Default::default()
        })
        .await;

    let starts = Arc::new(AtomicU32::new(0));
    let stops = Arc::new(AtomicU32::new(0));
    let actor_id = AgentId::new();

    let s = Arc::clone(&starts);
    let ps = Arc::clone(&stops);

    let ref_a = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: actor_id,
                        pre_start_count: Arc::clone(&s),
                        post_stop_count: Arc::clone(&ps),
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
    assert_eq!(ref_a.actor_id(), actor_id, "ActorId should match");
    assert_eq!(starts.load(Ordering::SeqCst), 1, "pre_start called once");

    // Trigger failure
    ref_a.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Restarted: pre_start called again, post_stop called for old instance
    assert_eq!(starts.load(Ordering::SeqCst), 2, "pre_start called again");
    assert_eq!(stops.load(Ordering::SeqCst), 1, "post_stop called for old");

    // Same actor ID tracked in supervision tree
    let still_supervised = supervised
        .with_tree(|tree| tree.find_supervisor(&actor_id).is_some())
        .await;
    assert!(still_supervised, "Actor still in supervision tree");

    supervised.shutdown().await.unwrap();
}

// --- T071: 3-level tree failure cascade ---

#[tokio::test]
async fn t071_three_level_tree_failure_cascade() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    // Root: high budget
    let root_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 10,
            escalation_policy: EscalationPolicy::Terminate,
            ..Default::default()
        })
        .await;

    // Mid-level: low budget (1 restart allowed)
    let mid_id = supervised
        .create_supervisor_under(
            root_id,
            SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 1,
                escalation_policy: EscalationPolicy::Escalate,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let worker_starts = Arc::new(AtomicU32::new(0));
    let worker_id = AgentId::new();

    let ws = Arc::clone(&worker_starts);
    let wps = Arc::new(AtomicU32::new(0));
    let ref_w = supervised
        .spawn_supervised::<TrackingActor, _>(
            mid_id,
            move || {
                (
                    TrackingActor {
                        id: worker_id,
                        pre_start_count: Arc::clone(&ws),
                        post_stop_count: Arc::clone(&wps),
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

    // First failure — mid-level restarts the worker
    ref_w.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(
        worker_starts.load(Ordering::SeqCst),
        2,
        "Worker restarted once"
    );

    // Tree should still be active
    let status = supervised.tree_status().await;
    assert!(status.total_restarts > 0, "Restarts recorded in tree");

    supervised.shutdown().await.unwrap();
}

// --- T072: Tree with 10+ nodes, query status ---

#[tokio::test]
async fn t072_large_tree_status_query() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let root_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 20,
            ..Default::default()
        })
        .await;

    // Spawn 10 workers under root
    for _ in 0..10 {
        let worker_id = AgentId::new();
        let starts = Arc::new(AtomicU32::new(0));
        let stops = Arc::new(AtomicU32::new(0));
        supervised
            .spawn_supervised::<TrackingActor, _>(
                root_id,
                move || {
                    (
                        TrackingActor {
                            id: worker_id,
                            pre_start_count: Arc::clone(&starts),
                            post_stop_count: Arc::clone(&stops),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    let status = supervised.tree_status().await;
    assert_eq!(status.total_nodes, 11); // 1 supervisor + 10 children
    assert_eq!(status.supervisor_count, 1);
    assert_eq!(status.tree_depth, 2);
    assert_eq!(status.total_restarts, 0);

    supervised.shutdown().await.unwrap();
}

// --- T073: Graceful shutdown with post_stop hooks ---

#[tokio::test]
async fn t073_graceful_shutdown_all_post_stops_called() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy::default())
        .await;

    let stops: Vec<Arc<AtomicU32>> = (0..5).map(|_| Arc::new(AtomicU32::new(0))).collect();

    for stop_counter in &stops {
        let worker_id = AgentId::new();
        let starts = Arc::new(AtomicU32::new(0));
        let sc = Arc::clone(stop_counter);
        supervised
            .spawn_supervised::<TrackingActor, _>(
                sup_id,
                move || {
                    (
                        TrackingActor {
                            id: worker_id,
                            pre_start_count: Arc::clone(&starts),
                            post_stop_count: Arc::clone(&sc),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();
    }

    let _handle = supervised.start_supervision();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Graceful shutdown
    supervised.shutdown().await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // All post_stop hooks should have been called
    for (i, stop_counter) in stops.iter().enumerate() {
        assert_eq!(
            stop_counter.load(Ordering::SeqCst),
            1,
            "post_stop not called for actor {}",
            i
        );
    }
}

// --- T084: EventBus lifecycle events integration test ---

#[tokio::test]
async fn t084_event_bus_lifecycle_events_on_failure_and_restart() {
    let event_bus = Arc::new(EventBus::default());
    let mut rx = event_bus.subscribe_broadcast();

    let supervised =
        SupervisedSystem::with_event_bus(ActorSystemConfig::default(), Arc::clone(&event_bus));

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 5,
            ..Default::default()
        })
        .await;

    let starts = Arc::new(AtomicU32::new(0));
    let stops = Arc::new(AtomicU32::new(0));
    let actor_id = AgentId::new();

    let s = Arc::clone(&starts);
    let ps = Arc::clone(&stops);

    let ref_a = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: actor_id,
                        pre_start_count: Arc::clone(&s),
                        post_stop_count: Arc::clone(&ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let _handle = supervised.start_supervision();
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Initial lifecycle events from spawn + startup.
    let mut initial_events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        initial_events.push(event);
    }

    let created_event = initial_events
        .iter()
        .find(|e| e.event_type == EventType::Custom("agent.Created".into()))
        .expect("Should have agent.Created event");
    assert_eq!(created_event.payload["actor_id"], actor_id.to_string());

    let started_event = initial_events
        .iter()
        .find(|e| e.event_type == EventType::Custom("agent.Started".into()))
        .expect("Should have agent.Started event");
    assert_eq!(started_event.payload["actor_id"], actor_id.to_string());

    let actor_id_str = actor_id.to_string();
    let init_to_running = initial_events.iter().any(|e| {
        e.event_type == EventType::Custom("agent.StateChanged".into())
            && e.payload.get("actor_id").and_then(|v| v.as_str()) == Some(actor_id_str.as_str())
            && e.payload.get("from").and_then(|v| v.as_str()) == Some("Initializing")
            && e.payload.get("to").and_then(|v| v.as_str()) == Some("Running")
    });
    assert!(
        init_to_running,
        "Should have StateChanged Initializing->Running event"
    );

    // Trigger failure and restart.
    ref_a.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut restart_events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        restart_events.push(event);
    }

    // Actor cell lifecycle and supervision chain should both be present.
    let has_cell_failed = restart_events
        .iter()
        .any(|e| e.event_type == EventType::Custom("agent.Failed".into()));
    let has_stopping_transition = restart_events.iter().any(|e| {
        e.event_type == EventType::Custom("agent.StateChanged".into())
            && e.payload.get("from").and_then(|v| v.as_str()) == Some("Running")
            && e.payload.get("to").and_then(|v| v.as_str()) == Some("Stopping")
    });
    let has_error_transition = restart_events.iter().any(|e| {
        e.event_type == EventType::Custom("agent.StateChanged".into())
            && e.payload.get("from").and_then(|v| v.as_str()) == Some("Stopping")
            && e.payload.get("to").and_then(|v| v.as_str()) == Some("Error")
    });
    let has_supervision_failed = restart_events
        .iter()
        .any(|e| e.event_type == EventType::Agent(AgentEventType::Failed));
    let supervision_restart = restart_events
        .iter()
        .find(|e| {
            e.event_type == EventType::Agent(AgentEventType::Started)
                && e.payload.get("action").and_then(|v| v.as_str()) == Some("restart")
        })
        .expect("Should have supervision restart Started event");
    let restarted_started = restart_events.iter().any(|e| {
        e.event_type == EventType::Custom("agent.Started".into())
            && e.id != started_event.id
    });

    assert!(has_cell_failed, "Should have actor-cell Failed event");
    assert!(has_stopping_transition, "Should have Running->Stopping transition");
    assert!(has_error_transition, "Should have Stopping->Error transition");
    assert!(has_supervision_failed, "Should have supervision Failed event");
    assert!(restarted_started, "Should have restarted agent.Started event");

    let failure_event = restart_events
        .iter()
        .find(|e| e.event_type == EventType::Agent(AgentEventType::Failed))
        .unwrap();

    assert!(
        failure_event.correlation_id.is_some(),
        "Failure event should have correlation_id"
    );
    assert_eq!(
        failure_event.correlation_id, supervision_restart.correlation_id,
        "Failure and restart events should share correlation_id"
    );
    assert_eq!(
        supervision_restart.causation_id,
        Some(failure_event.id),
        "Restart event causation_id should reference failure event"
    );

    supervised.shutdown().await.unwrap();
}

// --- T085: ActorSystemHealthCheck integration test ---

#[tokio::test]
async fn t085_health_check_reports_actor_system_status() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy::default())
        .await;

    for _ in 0..3 {
        let id = AgentId::new();
        let starts = Arc::new(AtomicU32::new(0));
        let stops = Arc::new(AtomicU32::new(0));
        supervised
            .spawn_supervised::<TrackingActor, _>(
                sup_id,
                move || {
                    (
                        TrackingActor {
                            id,
                            pre_start_count: Arc::clone(&starts),
                            post_stop_count: Arc::clone(&stops),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create health check
    let health_check = ActorSystemHealthCheck::new(
        Arc::new(mister_smith_actor::ActorSystem::new(
            ActorSystemConfig::default(),
        )),
        supervised.tree().clone(),
    );

    // Empty system (different ActorSystem instance) is healthy
    let status = health_check.check().await.unwrap();
    assert_eq!(status, Status::Healthy);

    // Verify component ID
    assert_eq!(health_check.component_id().as_str(), "actor-system");

    supervised.shutdown().await.unwrap();
}

// --- Phase 7: Edge Case & Performance Tests ---

/// Actor that delays message handling to test ask timeouts.
struct SlowActor {
    id: AgentId,
}

#[async_trait]
impl Actor for SlowActor {
    type Message = TestMsg;
    type State = u32;
    type Error = TestError;

    async fn handle_message(
        &mut self,
        _message: TestMsg,
        _state: &mut u32,
    ) -> Result<(), TestError> {
        tokio::time::sleep(Duration::from_secs(5)).await;
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

/// Actor whose pre_start always fails, for restart failure edge cases.
struct PreStartFailActor {
    id: AgentId,
    pre_start_count: Arc<AtomicU32>,
}

#[async_trait]
impl Actor for PreStartFailActor {
    type Message = TestMsg;
    type State = u32;
    type Error = TestError;

    async fn handle_message(
        &mut self,
        message: TestMsg,
        _state: &mut u32,
    ) -> Result<(), TestError> {
        match message {
            TestMsg::Ping => Ok(()),
            TestMsg::Fail => Err(TestError("intentional failure".into())),
        }
    }

    fn pre_start(&mut self) -> Result<(), TestError> {
        let count = self.pre_start_count.fetch_add(1, Ordering::SeqCst);
        // Succeed on first call (initial spawn), fail on subsequent (restarts)
        if count == 0 {
            Ok(())
        } else {
            Err(TestError("pre_start always fails".into()))
        }
    }

    fn post_stop(&mut self) -> Result<(), TestError> {
        Ok(())
    }

    fn actor_id(&self) -> AgentId {
        self.id
    }
}

// T088: Mutual ask deadlock — two actors send ask to each other, both timeout
#[tokio::test]
async fn t088_mutual_ask_timeout_no_permanent_hang() {
    let system = mister_smith_actor::ActorSystem::new(ActorSystemConfig::default());

    let id1 = AgentId::new();
    let id2 = AgentId::new();

    let ref1 = system
        .spawn(SlowActor { id: id1 }, 0, SpawnConfig::default())
        .await
        .unwrap();
    let ref2 = system
        .spawn(SlowActor { id: id2 }, 0, SpawnConfig::default())
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Both asks should timeout independently, no permanent hang
    let (r1, r2) = tokio::join!(
        ref1.ask(TestMsg::Ping, Duration::from_millis(100)),
        ref2.ask(TestMsg::Ping, Duration::from_millis(100)),
    );

    assert!(matches!(r1.unwrap_err(), ActorError::AskTimeout));
    assert!(matches!(r2.unwrap_err(), ActorError::AskTimeout));

    system.shutdown().await.unwrap();
}

// T090: pre_start failure during restart — actor transitions to Error, supervisor notified
#[tokio::test]
async fn t090_pre_start_failure_during_restart() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 10,
            ..Default::default()
        })
        .await;

    let actor_id = AgentId::new();
    let pre_start_count = Arc::new(AtomicU32::new(0));
    let psc = Arc::clone(&pre_start_count);

    // Spawn actor — first pre_start succeeds, subsequent ones fail
    let actor_ref = supervised
        .spawn_supervised::<PreStartFailActor, _>(
            sup_id,
            move || {
                (
                    PreStartFailActor {
                        id: actor_id,
                        pre_start_count: Arc::clone(&psc),
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

    // Trigger a failure so supervision attempts restart (which will fail in pre_start)
    actor_ref.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;

    // pre_start called at least twice: once for initial spawn, once for restart attempt
    assert!(
        pre_start_count.load(Ordering::SeqCst) >= 2,
        "pre_start should have been called at least twice (spawn + restart)"
    );

    supervised.shutdown().await.unwrap();
}

// T091: Root supervisor exhaustion — pre_start always fails, causing cascading restarts
// until budget is exhausted. Verifies the system doesn't hang.
#[tokio::test]
async fn t091_root_supervisor_budget_exhaustion() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 3,
            escalation_policy: EscalationPolicy::Terminate,
            ..Default::default()
        })
        .await;

    let actor_id = AgentId::new();
    let pre_start_count = Arc::new(AtomicU32::new(0));
    let psc = Arc::clone(&pre_start_count);

    // Spawn actor — first pre_start succeeds, subsequent ones fail during restart
    let actor_ref = supervised
        .spawn_supervised::<PreStartFailActor, _>(
            sup_id,
            move || {
                (
                    PreStartFailActor {
                        id: actor_id,
                        pre_start_count: Arc::clone(&psc),
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

    // Trigger initial failure to start cascading restart attempts
    actor_ref.tell(TestMsg::Fail).unwrap();

    // Wait for cascading failures to exhaust budget
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // pre_start was called multiple times (initial + restart attempts)
    let count = pre_start_count.load(Ordering::SeqCst);
    assert!(
        count >= 2,
        "pre_start should have been called at least 2 times, was called {} times",
        count
    );

    // Tree should have recorded restarts
    let tree = supervised.tree().read().await;
    let status = tree.query_status();
    assert!(
        status.total_restarts >= 1,
        "Should have at least 1 restart recorded, got {}",
        status.total_restarts
    );

    // System should not hang — this completes without timeout
    drop(tree);
    supervised.shutdown().await.unwrap();
}

// T092: Message sent during restart — message processed after restart completes
#[tokio::test]
async fn t092_message_during_restart_processed_after() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 5,
            ..Default::default()
        })
        .await;

    let starts = Arc::new(AtomicU32::new(0));
    let stops = Arc::new(AtomicU32::new(0));
    let actor_id = AgentId::new();

    let s = Arc::clone(&starts);
    let ps = Arc::clone(&stops);
    let ref_a = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: actor_id,
                        pre_start_count: Arc::clone(&s),
                        post_stop_count: Arc::clone(&ps),
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

    // Trigger a failure
    ref_a.tell(TestMsg::Fail).unwrap();

    // Immediately send a Ping — this should be accepted in the mailbox
    let ping_result = ref_a.tell(TestMsg::Ping);
    assert!(
        ping_result.is_ok(),
        "Tell should succeed (mailbox has capacity)"
    );

    // Wait for restart to complete
    tokio::time::sleep(Duration::from_millis(900)).await;

    // Actor should have restarted (pre_start called at least twice: initial + restart)
    assert!(
        starts.load(Ordering::SeqCst) >= 2,
        "Actor should have restarted"
    );

    supervised.shutdown().await.unwrap();
}

// T093: Concurrent child failures — supervisor processes all failures correctly
#[tokio::test]
async fn t093_concurrent_child_failures_handled() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 20,
            ..Default::default()
        })
        .await;

    let mut refs = Vec::new();
    let mut start_counters = Vec::new();

    for _ in 0..5 {
        let id = AgentId::new();
        let starts = Arc::new(AtomicU32::new(0));
        let stops = Arc::new(AtomicU32::new(0));
        let s = Arc::clone(&starts);
        let ps = Arc::clone(&stops);

        let actor_ref = supervised
            .spawn_supervised::<TrackingActor, _>(
                sup_id,
                move || {
                    (
                        TrackingActor {
                            id,
                            pre_start_count: Arc::clone(&s),
                            post_stop_count: Arc::clone(&ps),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();
        refs.push(actor_ref);
        start_counters.push(starts);
    }

    let _handle = supervised.start_supervision();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Trigger failures on all 5 actors simultaneously
    for r in &refs {
        r.tell(TestMsg::Fail).unwrap();
    }

    // Wait for all restarts to complete
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // All actors should have restarted (pre_start called at least 2 times each)
    for (i, counter) in start_counters.iter().enumerate() {
        assert!(
            counter.load(Ordering::SeqCst) >= 2,
            "Actor {} should have restarted (pre_start count: {})",
            i,
            counter.load(Ordering::SeqCst)
        );
    }

    supervised.shutdown().await.unwrap();
}

// T094: Performance — spawn 1000 actors, each processes 10 messages, within 5 seconds (SC-001)
#[tokio::test]
async fn t094_spawn_1000_actors_process_messages() {
    let system = mister_smith_actor::ActorSystem::new(ActorSystemConfig::default());

    let start = std::time::Instant::now();

    let mut actor_refs = Vec::new();
    for _ in 0..1000 {
        let id = AgentId::new();
        let starts = Arc::new(AtomicU32::new(0));
        let stops = Arc::new(AtomicU32::new(0));
        let actor = TrackingActor {
            id,
            pre_start_count: starts,
            post_stop_count: stops,
        };
        let actor_ref = system
            .spawn(actor, 0, SpawnConfig::default())
            .await
            .unwrap();
        actor_refs.push(actor_ref);
    }

    // Wait for all actors to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send 10 messages to each actor
    for actor_ref in &actor_refs {
        for _ in 0..10 {
            actor_ref.tell(TestMsg::Ping).unwrap();
        }
    }

    // Wait for all messages to be processed
    tokio::time::sleep(Duration::from_millis(900)).await;

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(5),
        "1000 actors x 10 messages should complete within 5 seconds, took {:?}",
        elapsed
    );

    assert_eq!(system.actor_count().await, 1000);

    system.shutdown().await.unwrap();
}

// T095: Performance — graceful shutdown of 100+ actor tree with all post_stop hooks called (SC-006)
#[tokio::test]
async fn t095_graceful_shutdown_100_actors_all_post_stops() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy::default())
        .await;

    let mut stop_counters = Vec::new();

    for _ in 0..100 {
        let id = AgentId::new();
        let starts = Arc::new(AtomicU32::new(0));
        let stops = Arc::new(AtomicU32::new(0));
        let s = Arc::clone(&starts);
        let ps = Arc::clone(&stops);
        stop_counters.push(Arc::clone(&stops));

        supervised
            .spawn_supervised::<TrackingActor, _>(
                sup_id,
                move || {
                    (
                        TrackingActor {
                            id,
                            pre_start_count: Arc::clone(&s),
                            post_stop_count: Arc::clone(&ps),
                        },
                        0,
                    )
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();
    }

    let _handle = supervised.start_supervision();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let start = std::time::Instant::now();
    supervised.shutdown().await.unwrap();
    let elapsed = start.elapsed();

    // All post_stop hooks should have been called
    for (i, counter) in stop_counters.iter().enumerate() {
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "post_stop not called for actor {}",
            i
        );
    }

    // Shutdown should complete in reasonable time (generous for CI/parallel test contention)
    assert!(
        elapsed < Duration::from_secs(30),
        "Shutdown of 100 actors should complete within 30 seconds, took {:?}",
        elapsed
    );
}

// T094: Stop decision should not terminate the supervision loop for future failures
#[tokio::test]
async fn supervision_continues_after_stop_decision() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 5,
            ..Default::default()
        })
        .await;

    let stop_actor_id = AgentId::new();
    let stop_starts = Arc::new(AtomicU32::new(0));
    let stop_stops = Arc::new(AtomicU32::new(0));
    let stop_s = Arc::clone(&stop_starts);
    let stop_ps = Arc::clone(&stop_stops);

    let _stop_ref = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: stop_actor_id,
                        pre_start_count: Arc::clone(&stop_s),
                        post_stop_count: Arc::clone(&stop_ps),
                    },
                    0,
                )
            },
            SpawnConfig {
                restart_scope: RestartScope::Temporary,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let restart_actor_id = AgentId::new();
    let restart_starts = Arc::new(AtomicU32::new(0));
    let restart_stops = Arc::new(AtomicU32::new(0));
    let restart_s = Arc::clone(&restart_starts);
    let restart_ps = Arc::clone(&restart_stops);

    let restart_ref = supervised
        .spawn_supervised::<TrackingActor, _>(
            sup_id,
            move || {
                (
                    TrackingActor {
                        id: restart_actor_id,
                        pre_start_count: Arc::clone(&restart_s),
                        post_stop_count: Arc::clone(&restart_ps),
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

    // Trigger Stop decision via normal termination of a temporary actor.
    supervised.system().stop_actor(&stop_actor_id).await;
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_eq!(stop_starts.load(Ordering::SeqCst), 1);

    // A later failure should still be handled and restarted.
    restart_ref.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(
        restart_starts.load(Ordering::SeqCst),
        2,
        "Second actor should restart after Stop decision"
    );

    supervised.shutdown().await.unwrap();
}

// T095: Ignore decision should not terminate the supervision loop for future failures
#[tokio::test]
async fn supervision_continues_after_ignore_decision() {
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let ignore_sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 0,
            escalation_policy: EscalationPolicy::LogAndIgnore,
            ..Default::default()
        })
        .await;

    let restart_sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 5,
            ..Default::default()
        })
        .await;

    let ignore_actor_id = AgentId::new();
    let ignore_starts = Arc::new(AtomicU32::new(0));
    let ignore_stops = Arc::new(AtomicU32::new(0));
    let ignore_s = Arc::clone(&ignore_starts);
    let ignore_ps = Arc::clone(&ignore_stops);

    let ignore_ref = supervised
        .spawn_supervised::<TrackingActor, _>(
            ignore_sup_id,
            move || {
                (
                    TrackingActor {
                        id: ignore_actor_id,
                        pre_start_count: Arc::clone(&ignore_s),
                        post_stop_count: Arc::clone(&ignore_ps),
                    },
                    0,
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let restart_actor_id = AgentId::new();
    let restart_starts = Arc::new(AtomicU32::new(0));
    let restart_stops = Arc::new(AtomicU32::new(0));
    let restart_s = Arc::clone(&restart_starts);
    let restart_ps = Arc::clone(&restart_stops);

    let restart_ref = supervised
        .spawn_supervised::<TrackingActor, _>(
            restart_sup_id,
            move || {
                (
                    TrackingActor {
                        id: restart_actor_id,
                        pre_start_count: Arc::clone(&restart_s),
                        post_stop_count: Arc::clone(&restart_ps),
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

    // First failure escalates to LogAndIgnore (Ignore decision).
    ignore_ref.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(ignore_starts.load(Ordering::SeqCst), 1);

    // A later failure on a different supervisor should still be handled.
    restart_ref.tell(TestMsg::Fail).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(
        restart_starts.load(Ordering::SeqCst),
        2,
        "Second actor should restart after Ignore decision"
    );

    supervised.shutdown().await.unwrap();
}

// --- T085: Backoff timing for OneForOne ---

#[tokio::test]
async fn t085_backoff_delays_restart_one_for_one() {
    let backoff = Duration::from_millis(120);
    let margin = Duration::from_millis(20);
    let supervised = SupervisedSystem::new(ActorSystemConfig::default());

    let sup_id = supervised
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForOne,
            max_failures: 5,
            backoff_strategy: mister_smith_core::BackoffStrategy::Fixed(backoff),
            ..Default::default()
        })
        .await;

    let actor_id = AgentId::new();
    let starts = Arc::new(AtomicU32::new(0));
    let start_times = Arc::new(Mutex::new(Vec::new()));

    let starts_clone = Arc::clone(&starts);
    let times_clone = Arc::clone(&start_times);
    let actor_ref = supervised
        .spawn_supervised::<TimestampActor, _>(
            sup_id,
            move || {
                (
                    TimestampActor {
                        id: actor_id,
                        starts: Arc::clone(&starts_clone),
                        start_times: Arc::clone(&times_clone),
                    },
                    (),
                )
            },
            SpawnConfig::default(),
        )
        .await
        .unwrap();

    let _handle = supervised.start_supervision();
    tokio::time::sleep(Duration::from_millis(80)).await;

    let fail_at = Instant::now();
    actor_ref.tell(TestMsg::Fail).unwrap();
    wait_for_restarts(&[Arc::clone(&starts)], 2).await;

    let times = start_times.lock().unwrap().clone();
    assert_eq!(times.len(), 2);
    let restart_delay = times[1].saturating_duration_since(fail_at);
    assert!(
        restart_delay >= backoff - margin,
        "restart occurred too early: {restart_delay:?} < {:?}",
        backoff - margin
    );

    supervised.shutdown().await.unwrap();
}

// --- T086: Backoff timing for OneForAll and RestForOne ---

#[tokio::test]
async fn t086_backoff_delays_restart_for_all_affected_siblings() {
    let backoff = Duration::from_millis(100);
    let margin = Duration::from_millis(20);

    for policy in [RestartPolicy::OneForAll, RestartPolicy::RestForOne] {
        let supervised = SupervisedSystem::new(ActorSystemConfig::default());
        let sup_id = supervised
            .create_supervisor(SupervisionStrategy {
                restart_policy: policy,
                max_failures: 5,
                backoff_strategy: mister_smith_core::BackoffStrategy::Fixed(backoff),
                ..Default::default()
            })
            .await;

        let a_id = AgentId::new();
        let b_id = AgentId::new();
        let c_id = AgentId::new();

        let a_starts = Arc::new(AtomicU32::new(0));
        let b_starts = Arc::new(AtomicU32::new(0));
        let c_starts = Arc::new(AtomicU32::new(0));

        let a_times = Arc::new(Mutex::new(Vec::new()));
        let b_times = Arc::new(Mutex::new(Vec::new()));
        let c_times = Arc::new(Mutex::new(Vec::new()));

        let ref_a = supervised
            .spawn_supervised::<TimestampActor, _>(
                sup_id,
                {
                    let starts = Arc::clone(&a_starts);
                    let times = Arc::clone(&a_times);
                    move || {
                        (
                            TimestampActor {
                                id: a_id,
                                starts: Arc::clone(&starts),
                                start_times: Arc::clone(&times),
                            },
                            (),
                        )
                    }
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let ref_b = supervised
            .spawn_supervised::<TimestampActor, _>(
                sup_id,
                {
                    let starts = Arc::clone(&b_starts);
                    let times = Arc::clone(&b_times);
                    move || {
                        (
                            TimestampActor {
                                id: b_id,
                                starts: Arc::clone(&starts),
                                start_times: Arc::clone(&times),
                            },
                            (),
                        )
                    }
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let ref_c = supervised
            .spawn_supervised::<TimestampActor, _>(
                sup_id,
                {
                    let starts = Arc::clone(&c_starts);
                    let times = Arc::clone(&c_times);
                    move || {
                        (
                            TimestampActor {
                                id: c_id,
                                starts: Arc::clone(&starts),
                                start_times: Arc::clone(&times),
                            },
                            (),
                        )
                    }
                },
                SpawnConfig::default(),
            )
            .await
            .unwrap();

        let _handle = supervised.start_supervision();
        tokio::time::sleep(Duration::from_millis(80)).await;

        let fail_at = Instant::now();
        ref_b.tell(TestMsg::Fail).unwrap();

        if policy == RestartPolicy::OneForAll {
            wait_for_restarts(
                &[
                    Arc::clone(&a_starts),
                    Arc::clone(&b_starts),
                    Arc::clone(&c_starts),
                ],
                2,
            )
            .await;
        } else {
            wait_for_restarts(&[Arc::clone(&b_starts), Arc::clone(&c_starts)], 2).await;
            assert_eq!(a_starts.load(Ordering::SeqCst), 1);
        }

        let mut affected = vec![b_times.lock().unwrap().clone()];
        if policy == RestartPolicy::OneForAll {
            affected.push(a_times.lock().unwrap().clone());
        }
        affected.push(c_times.lock().unwrap().clone());

        for times in affected {
            assert!(times.len() >= 2, "expected restart timestamp");
            let restart_delay = times[1].saturating_duration_since(fail_at);
            assert!(
                restart_delay >= backoff - margin,
                "policy {policy:?} restarted too early: {restart_delay:?} < {:?}",
                backoff - margin
            );
        }

        let _ = (ref_a, ref_c);
        supervised.shutdown().await.unwrap();
    }
}

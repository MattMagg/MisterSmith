//! Internal actor runtime wrapper managing lifecycle and message processing.
//!
//! `ActorCell<A>` owns the actor instance, its state, and the message processing loop.
//! It manages lifecycle transitions (Initializing -> Running -> Stopping -> Terminated)
//! and invokes `pre_start`/`post_stop` hooks.

use std::sync::Arc;

use futures::FutureExt;
use mister_smith_core::{Actor, AgentId, AgentState, EventPublisher, SystemEvent};
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{debug, error, info, warn};

use crate::mailbox::{Envelope, MailboxReceiver};

enum MessageHandlingOutcome<E> {
    Ok,
    Failed(E),
    Panicked(String),
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic with non-string payload".to_string()
    }
}

async fn handle_message_with_panic_capture<A>(
    actor: &mut A,
    message: A::Message,
    state: &mut A::State,
) -> MessageHandlingOutcome<A::Error>
where
    A: Actor,
{
    match std::panic::AssertUnwindSafe(actor.handle_message(message, state))
        .catch_unwind()
        .await
    {
        Ok(Ok(())) => MessageHandlingOutcome::Ok,
        Ok(Err(error)) => MessageHandlingOutcome::Failed(error),
        Err(panic_payload) => MessageHandlingOutcome::Panicked(panic_payload_to_string(panic_payload)),
    }
}

/// Fire-and-forget lifecycle event emission.
async fn emit_lifecycle_event(
    publisher: &Option<Arc<dyn EventPublisher>>,
    event_type: &str,
    payload: serde_json::Value,
) {
    if let Some(ref p) = publisher {
        let event = SystemEvent {
            event_type: event_type.to_string(),
            payload,
        };
        if let Err(e) = p.publish(event).await {
            warn!(error = %e, event_type, "Failed to publish lifecycle event");
        }
    }
}

/// Notification sent from an actor cell to its supervisor when the actor
/// stops or fails.
#[derive(Debug)]
pub struct SupervisionNotification {
    /// The actor that produced this notification.
    pub actor_id: AgentId,
    /// The termination reason.
    pub reason: TerminationReason,
}

/// Why an actor terminated.
#[derive(Debug)]
pub enum TerminationReason {
    /// Normal shutdown (requested or mailbox closed).
    Normal,
    /// Actor's `handle_message` returned an error.
    Failed(String),
    /// Actor panicked during message processing.
    Panicked(String),
    /// Actor's `pre_start` hook failed.
    PreStartFailed(String),
}

/// Runs an actor's message processing loop inside a Tokio task.
///
/// This function is the core of the actor cell. It:
/// 1. Calls `pre_start` on the actor
/// 2. Processes messages sequentially from the mailbox
/// 3. Routes ask replies through oneshot channels
/// 4. Detects panics by spawning message handling in a sub-task
/// 5. Calls `post_stop` on termination
/// 6. Sends a supervision notification on exit
pub async fn run_actor<A>(
    mut actor: A,
    mut state: A::State,
    mut receiver: MailboxReceiver<Envelope<A::Message>>,
    mut stop_rx: mpsc::Receiver<()>,
    state_tx: watch::Sender<AgentState>,
    startup_tx: Option<oneshot::Sender<Result<(), String>>>,
    supervision_tx: Option<mpsc::UnboundedSender<SupervisionNotification>>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
) where
    A: Actor,
    A::Message: Send + 'static,
    A::State: Send + 'static,
{
    let actor_id = actor.actor_id();

    // Phase: pre_start
    let _ = state_tx.send(AgentState::Initializing);
    debug!(actor_id = %actor_id, "Actor initializing");

    if let Err(e) = actor.pre_start() {
        let startup_error = e.to_string();
        error!(actor_id = %actor_id, error = %e, "pre_start failed");
        let _ = state_tx.send(AgentState::Error);
        if let Some(tx) = startup_tx {
            let _ = tx.send(Err(startup_error.clone()));
        }
        emit_lifecycle_event(
            &event_publisher,
            "actor.failed",
            serde_json::json!({
                "actor_id": actor_id.to_string(),
                "error": startup_error.clone(),
                "phase": "pre_start",
            }),
        )
        .await;
        if let Some(ref tx) = supervision_tx {
            let _ = tx.send(SupervisionNotification {
                actor_id,
                reason: TerminationReason::PreStartFailed(startup_error),
            });
        }
        return;
    }

    if let Some(tx) = startup_tx {
        let _ = tx.send(Ok(()));
    }

    // Phase: Running
    let _ = state_tx.send(AgentState::Running);
    info!(actor_id = %actor_id, "Actor started");
    emit_lifecycle_event(
        &event_publisher,
        "actor.started",
        serde_json::json!({"actor_id": actor_id.to_string()}),
    )
    .await;

    // Message processing loop
    let termination_reason = loop {
        tokio::select! {
            biased;

            // Stop signal takes priority
            _ = stop_rx.recv() => {
                debug!(actor_id = %actor_id, "Stop signal received");
                break TerminationReason::Normal;
            }

            // Process next message
            msg = receiver.recv() => {
                match msg {
                    Some(envelope) => {
                        let Envelope { message, reply_tx } = envelope;

                        match handle_message_with_panic_capture(&mut actor, message, &mut state).await {
                            MessageHandlingOutcome::Ok => {
                                if let Some(tx) = reply_tx {
                                    let _ = tx.send(Ok(()));
                                }
                            }
                            MessageHandlingOutcome::Failed(e) => {
                                let err_msg = e.to_string();
                                warn!(actor_id = %actor_id, error = %err_msg, "handle_message failed");
                                if let Some(tx) = reply_tx {
                                    let _ = tx.send(Err(err_msg.clone()));
                                }
                                break TerminationReason::Failed(err_msg);
                            }
                            MessageHandlingOutcome::Panicked(panic_msg) => {
                                error!(actor_id = %actor_id, error = %panic_msg, "handle_message panicked");
                                if let Some(tx) = reply_tx {
                                    let _ = tx.send(Err(format!("actor panicked: {panic_msg}")));
                                }
                                break TerminationReason::Panicked(panic_msg);
                            }
                        }
                    }
                    None => {
                        // All senders dropped — mailbox closed
                        debug!(actor_id = %actor_id, "Mailbox closed");
                        break TerminationReason::Normal;
                    }
                }
            }
        }
    };

    // Phase: Stopping
    let _ = state_tx.send(AgentState::Stopping);
    debug!(actor_id = %actor_id, "Actor stopping");

    // Drain remaining messages in the mailbox
    while let Ok(Some(envelope)) = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        receiver.recv(),
    )
    .await
    {
        let Envelope { message, reply_tx } = envelope;
        match handle_message_with_panic_capture(&mut actor, message, &mut state).await {
            MessageHandlingOutcome::Ok => {
                if let Some(tx) = reply_tx {
                    let _ = tx.send(Ok(()));
                }
            }
            MessageHandlingOutcome::Failed(e) => {
                if let Some(tx) = reply_tx {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
            MessageHandlingOutcome::Panicked(panic_msg) => {
                if let Some(tx) = reply_tx {
                    let _ = tx.send(Err(format!("actor panicked during shutdown: {panic_msg}")));
                }
            }
        }
    }

    // Phase: post_stop
    if let Err(e) = actor.post_stop() {
        warn!(actor_id = %actor_id, error = %e, "post_stop failed");
    }

    // Set terminal state and emit events
    match &termination_reason {
        TerminationReason::Normal => {
            let _ = state_tx.send(AgentState::Terminated);
            info!(actor_id = %actor_id, "Actor terminated normally");
            emit_lifecycle_event(
                &event_publisher,
                "actor.stopped",
                serde_json::json!({"actor_id": actor_id.to_string()}),
            )
            .await;
        }
        TerminationReason::Failed(e) => {
            let _ = state_tx.send(AgentState::Error);
            error!(actor_id = %actor_id, error = %e, "Actor failed");
            emit_lifecycle_event(
                &event_publisher,
                "actor.failed",
                serde_json::json!({
                    "actor_id": actor_id.to_string(),
                    "error": e,
                    "phase": "handle_message",
                }),
            )
            .await;
        }
        TerminationReason::Panicked(e) => {
            let _ = state_tx.send(AgentState::Error);
            error!(actor_id = %actor_id, error = %e, "Actor panicked");
            emit_lifecycle_event(
                &event_publisher,
                "actor.failed",
                serde_json::json!({
                    "actor_id": actor_id.to_string(),
                    "error": e,
                    "phase": "panic",
                }),
            )
            .await;
        }
        TerminationReason::PreStartFailed(_) => {
            // Already set to Error and event emitted above
        }
    }

    // Notify supervisor
    if let Some(tx) = supervision_tx {
        let _ = tx.send(SupervisionNotification {
            actor_id,
            reason: termination_reason,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mailbox::{create_mailbox, Envelope, MailboxConfig};
    use crate::mailbox::SpawnConfig;
    use crate::system::{ActorSystem, ActorSystemConfig};
    use async_trait::async_trait;
    use mister_smith_core::{ActorError, EventError};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::{oneshot, Mutex};

    // --- Test actor definitions ---

    struct CounterActor {
        id: AgentId,
    }

    #[derive(Debug)]
    enum CounterMsg {
        Increment,
        GetCount { reply: oneshot::Sender<u64> },
    }

    #[derive(Debug)]
    struct TestError(String);
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for TestError {}

    #[async_trait]
    impl Actor for CounterActor {
        type Message = CounterMsg;
        type State = u64;
        type Error = TestError;

        async fn handle_message(
            &mut self,
            message: CounterMsg,
            state: &mut u64,
        ) -> Result<(), TestError> {
            match message {
                CounterMsg::Increment => {
                    *state += 1;
                    Ok(())
                }
                CounterMsg::GetCount { reply } => {
                    let _ = reply.send(*state);
                    Ok(())
                }
            }
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

    // Actor that fails on a specific message
    struct FailingActor {
        id: AgentId,
    }

    #[derive(Debug)]
    enum FailMsg {
        #[allow(dead_code)]
        Ok,
        Fail,
    }

    #[async_trait]
    impl Actor for FailingActor {
        type Message = FailMsg;
        type State = ();
        type Error = TestError;

        async fn handle_message(
            &mut self,
            message: FailMsg,
            _state: &mut (),
        ) -> Result<(), TestError> {
            match message {
                FailMsg::Ok => Ok(()),
                FailMsg::Fail => Err(TestError("intentional failure".into())),
            }
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

    // Actor that tracks lifecycle hooks
    struct LifecycleActor {
        id: AgentId,
        pre_start_called: Arc<AtomicBool>,
        post_stop_called: Arc<AtomicBool>,
    }

    struct PanicActor {
        id: AgentId,
        post_stop_called: Arc<AtomicBool>,
    }

    #[derive(Debug)]
    enum PanicMsg {
        Panic,
    }

    #[async_trait]
    impl Actor for PanicActor {
        type Message = PanicMsg;
        type State = ();
        type Error = TestError;

        async fn handle_message(
            &mut self,
            _message: PanicMsg,
            _state: &mut (),
        ) -> Result<(), TestError> {
            panic!("panic requested");
        }

        fn pre_start(&mut self) -> Result<(), TestError> {
            Ok(())
        }

        fn post_stop(&mut self) -> Result<(), TestError> {
            self.post_stop_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn actor_id(&self) -> AgentId {
            self.id
        }
    }

    #[derive(Default)]
    struct TestEventPublisher {
        events: Mutex<Vec<SystemEvent>>,
    }

    #[async_trait]
    impl EventPublisher for TestEventPublisher {
        async fn publish(&self, event: SystemEvent) -> Result<(), EventError> {
            self.events.lock().await.push(event);
            Ok(())
        }
    }

    #[async_trait]
    impl Actor for LifecycleActor {
        type Message = String;
        type State = ();
        type Error = TestError;

        async fn handle_message(
            &mut self,
            _message: String,
            _state: &mut (),
        ) -> Result<(), TestError> {
            Ok(())
        }

        fn pre_start(&mut self) -> Result<(), TestError> {
            self.pre_start_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn post_stop(&mut self) -> Result<(), TestError> {
            self.post_stop_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn actor_id(&self) -> AgentId {
            self.id
        }
    }

    // Actor whose pre_start fails
    struct PreStartFailActor {
        id: AgentId,
    }

    #[async_trait]
    impl Actor for PreStartFailActor {
        type Message = ();
        type State = ();
        type Error = TestError;

        async fn handle_message(
            &mut self,
            _message: (),
            _state: &mut (),
        ) -> Result<(), TestError> {
            Ok(())
        }

        fn pre_start(&mut self) -> Result<(), TestError> {
            Err(TestError("pre_start failure".into()))
        }

        fn post_stop(&mut self) -> Result<(), TestError> {
            Ok(())
        }

        fn actor_id(&self) -> AgentId {
            self.id
        }
    }

    // T029: Spawn actor, process tell message, verify state mutation
    #[tokio::test]
    async fn actor_cell_process_tell() {
        let id = AgentId::new();
        let actor = CounterActor { id };
        let (tx, rx) = create_mailbox::<CounterMsg>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, mut state_rx) = watch::channel(AgentState::Initializing);
        let (sup_tx, mut sup_rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(run_actor(actor, 0u64, rx, stop_rx, state_tx, None, Some(sup_tx), None));

        // Wait for Running state
        while *state_rx.borrow() != AgentState::Running {
            state_rx.changed().await.unwrap();
        }

        // Send increment
        tx.send(Envelope::tell(CounterMsg::Increment)).await.unwrap();

        // Ask for count via the actor's internal reply channel
        let (count_tx, count_rx) = oneshot::channel();
        tx.send(Envelope::tell(CounterMsg::GetCount { reply: count_tx }))
            .await
            .unwrap();

        let count = count_rx.await.unwrap();
        assert_eq!(count, 1);

        // Drop sender to close mailbox
        drop(tx);
        handle.await.unwrap();

        // Should have terminated normally
        let notification = sup_rx.recv().await.unwrap();
        assert!(matches!(notification.reason, TerminationReason::Normal));
    }

    // T018: pre_start and post_stop lifecycle hooks
    #[tokio::test]
    async fn lifecycle_hooks_called() {
        let id = AgentId::new();
        let pre_start = Arc::new(AtomicBool::new(false));
        let post_stop = Arc::new(AtomicBool::new(false));
        let actor = LifecycleActor {
            id,
            pre_start_called: Arc::clone(&pre_start),
            post_stop_called: Arc::clone(&post_stop),
        };
        let (tx, rx) = create_mailbox::<String>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, mut state_rx) = watch::channel(AgentState::Initializing);

        let handle = tokio::spawn(run_actor(actor, (), rx, stop_rx, state_tx, None, None, None));

        // Wait for Running
        while *state_rx.borrow() != AgentState::Running {
            state_rx.changed().await.unwrap();
        }
        assert!(pre_start.load(Ordering::SeqCst));

        // Drop sender to shut down
        drop(tx);
        handle.await.unwrap();

        assert!(post_stop.load(Ordering::SeqCst));
    }

    // T019: Actor failure transitions to Error state
    #[tokio::test]
    async fn actor_failure_transitions_to_error() {
        let id = AgentId::new();
        let actor = FailingActor { id };
        let (tx, rx) = create_mailbox::<FailMsg>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, mut state_rx) = watch::channel(AgentState::Initializing);
        let (sup_tx, mut sup_rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(run_actor(actor, (), rx, stop_rx, state_tx, None, Some(sup_tx), None));

        // Wait for Running
        while *state_rx.borrow() != AgentState::Running {
            state_rx.changed().await.unwrap();
        }

        // Send failure message
        tx.send(Envelope::tell(FailMsg::Fail)).await.unwrap();
        handle.await.unwrap();

        // Should be in Error state
        assert_eq!(*state_rx.borrow(), AgentState::Error);

        let notification = sup_rx.recv().await.unwrap();
        assert!(matches!(notification.reason, TerminationReason::Failed(_)));
    }

    // T030: Ask pattern with reply via envelope
    #[tokio::test]
    async fn ask_pattern_reply() {
        let id = AgentId::new();
        let actor = CounterActor { id };
        let (tx, rx) = create_mailbox::<CounterMsg>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, mut state_rx) = watch::channel(AgentState::Initializing);

        let handle = tokio::spawn(run_actor(actor, 0u64, rx, stop_rx, state_tx, None, None, None));

        while *state_rx.borrow() != AgentState::Running {
            state_rx.changed().await.unwrap();
        }

        // Send increments
        tx.send(Envelope::tell(CounterMsg::Increment)).await.unwrap();
        tx.send(Envelope::tell(CounterMsg::Increment)).await.unwrap();
        tx.send(Envelope::tell(CounterMsg::Increment)).await.unwrap();

        // Ask via envelope with both reply channels
        let (ask_reply_tx, ask_reply_rx) = oneshot::channel();
        let (count_reply_tx, count_reply_rx) = oneshot::channel();
        tx.send(Envelope::ask(
            CounterMsg::GetCount {
                reply: count_reply_tx,
            },
            ask_reply_tx,
        ))
        .await
        .unwrap();

        // Count reply from inside the actor
        let count = count_reply_rx.await.unwrap();
        assert_eq!(count, 3);
        // Ask reply from the envelope routing
        let ask_result = ask_reply_rx.await.unwrap();
        assert!(ask_result.is_ok());

        drop(tx);
        handle.await.unwrap();
    }

    // Stop signal test
    #[tokio::test]
    async fn stop_signal_shuts_down_actor() {
        let id = AgentId::new();
        let post_stop = Arc::new(AtomicBool::new(false));
        let actor = LifecycleActor {
            id,
            pre_start_called: Arc::new(AtomicBool::new(false)),
            post_stop_called: Arc::clone(&post_stop),
        };
        let (_tx, rx) = create_mailbox::<String>(&MailboxConfig::bounded(10));
        let (stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, mut state_rx) = watch::channel(AgentState::Initializing);

        let handle = tokio::spawn(run_actor(actor, (), rx, stop_rx, state_tx, None, None, None));

        while *state_rx.borrow() != AgentState::Running {
            state_rx.changed().await.unwrap();
        }

        // Send stop signal
        stop_tx.send(()).await.unwrap();
        handle.await.unwrap();

        assert!(post_stop.load(Ordering::SeqCst));
        assert_eq!(*state_rx.borrow(), AgentState::Terminated);
    }

    // T087: Ask timeout — actor delays response beyond timeout, caller receives AskTimeout
    struct SlowActor {
        id: AgentId,
    }

    #[async_trait]
    impl Actor for SlowActor {
        type Message = String;
        type State = ();
        type Error = TestError;

        async fn handle_message(
            &mut self,
            _message: String,
            _state: &mut (),
        ) -> Result<(), TestError> {
            // Deliberately delay longer than the ask timeout
            tokio::time::sleep(Duration::from_secs(2)).await;
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
    async fn ask_timeout_with_slow_actor() {
        let system = ActorSystem::new(ActorSystemConfig::default());
        let id = AgentId::new();
        let actor_ref = system
            .spawn(SlowActor { id }, (), SpawnConfig::default())
            .await
            .unwrap();

        // Ask with a very short timeout — actor will still be processing
        let result = actor_ref
            .ask("hello".to_string(), Duration::from_millis(50))
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ActorError::AskTimeout));

        system.shutdown().await.unwrap();
    }

    // T089: pre_start failure on initial spawn — spawn returns startup error and actor is not registered
    #[tokio::test]
    async fn pre_start_failure_on_spawn_transitions_to_error() {
        let system = ActorSystem::new(ActorSystemConfig::default());
        let id = AgentId::new();

        // Spawn an actor whose pre_start always fails
        let result = system
            .spawn(PreStartFailActor { id }, (), SpawnConfig::default())
            .await;

        assert!(matches!(result, Err(ActorError::StartupFailed(_))));

        // Startup failure should keep actor out of registry
        let state = system.get_actor_state(&id).await;
        assert_eq!(state, None);
        assert_eq!(system.actor_count().await, 0);

        system.shutdown().await.unwrap();
    }

    // pre_start failure notification
    #[tokio::test]
    async fn pre_start_failure_notifies_supervisor() {
        let id = AgentId::new();
        let actor = PreStartFailActor { id };
        let (_tx, rx) = create_mailbox::<()>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, state_rx) = watch::channel(AgentState::Initializing);
        let (sup_tx, mut sup_rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(run_actor(actor, (), rx, stop_rx, state_tx, None, Some(sup_tx), None));
        handle.await.unwrap();

        assert_eq!(*state_rx.borrow(), AgentState::Error);
        let notification = sup_rx.recv().await.unwrap();
        assert!(matches!(
            notification.reason,
            TerminationReason::PreStartFailed(_)
        ));
    }

    #[tokio::test]
    async fn panic_transitions_to_error_and_notifies_supervisor() {
        let id = AgentId::new();
        let post_stop_called = Arc::new(AtomicBool::new(false));
        let actor = PanicActor {
            id,
            post_stop_called: Arc::clone(&post_stop_called),
        };
        let (tx, rx) = create_mailbox::<PanicMsg>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, state_rx) = watch::channel(AgentState::Initializing);
        let (sup_tx, mut sup_rx) = mpsc::unbounded_channel();

        let handle = tokio::spawn(run_actor(actor, (), rx, stop_rx, state_tx, None, Some(sup_tx), None));

        tx.send(Envelope::tell(PanicMsg::Panic)).await.unwrap();
        handle.await.unwrap();

        assert_eq!(*state_rx.borrow(), AgentState::Error);
        assert!(post_stop_called.load(Ordering::SeqCst));

        let notification = sup_rx.recv().await.unwrap();
        assert!(matches!(notification.reason, TerminationReason::Panicked(_)));
    }

    #[tokio::test]
    async fn panic_emits_failure_event_with_panic_phase() {
        let id = AgentId::new();
        let actor = PanicActor {
            id,
            post_stop_called: Arc::new(AtomicBool::new(false)),
        };
        let (tx, rx) = create_mailbox::<PanicMsg>(&MailboxConfig::bounded(10));
        let (_stop_tx, stop_rx) = mpsc::channel(1);
        let (state_tx, _state_rx) = watch::channel(AgentState::Initializing);
        let (sup_tx, _sup_rx) = mpsc::unbounded_channel();
        let publisher = Arc::new(TestEventPublisher::default());

        let handle = tokio::spawn(run_actor(
            actor,
            (),
            rx,
            stop_rx,
            state_tx,
            None,
            Some(sup_tx),
            Some(Arc::clone(&publisher) as Arc<dyn EventPublisher>),
        ));

        tx.send(Envelope::tell(PanicMsg::Panic)).await.unwrap();
        handle.await.unwrap();

        let events = publisher.events.lock().await;
        let panic_event = events.iter().find(|event| {
            event.event_type == "actor.failed"
                && event
                    .payload
                    .get("phase")
                    .and_then(serde_json::Value::as_str)
                    == Some("panic")
        });
        assert!(panic_event.is_some());
    }
}

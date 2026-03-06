//! Mailbox implementation: bounded/unbounded FIFO message queues.
//!
//! Wraps Tokio `mpsc` channels with configuration and envelope support.
//! Each actor has exactly one mailbox that receives [`Envelope<M>`] messages.

use mister_smith_core::{ActorError, RestartScope};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

/// Configuration for an actor's mailbox.
#[derive(Debug, Clone)]
pub struct MailboxConfig {
    /// Maximum number of messages the mailbox can hold.
    /// `None` means unbounded.
    pub capacity: Option<usize>,
}

impl Default for MailboxConfig {
    fn default() -> Self {
        Self {
            capacity: Some(1000),
        }
    }
}

impl MailboxConfig {
    /// Create a bounded mailbox config with the given capacity.
    pub fn bounded(capacity: usize) -> Self {
        Self {
            capacity: Some(capacity),
        }
    }

    /// Create an unbounded mailbox config.
    pub fn unbounded() -> Self {
        Self { capacity: None }
    }

    /// Returns true if this mailbox is bounded.
    pub fn is_bounded(&self) -> bool {
        self.capacity.is_some()
    }
}

/// Configuration for spawning an actor.
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// Mailbox configuration for the actor.
    pub mailbox: MailboxConfig,
    /// Restart scope controlling whether this actor is restarted on failure.
    pub restart_scope: RestartScope,
    /// Timeout for graceful shutdown of this actor.
    pub shutdown_timeout: Duration,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            mailbox: MailboxConfig::default(),
            restart_scope: RestartScope::Permanent,
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

/// An envelope wrapping a message with an optional reply channel.
///
/// For tell (fire-and-forget), `reply_tx` is `None`.
/// For ask (request-response), `reply_tx` carries the oneshot sender for the response.
#[derive(Debug)]
pub struct Envelope<M, R> {
    /// The message payload.
    pub message: M,
    /// Optional reply channel for the ask pattern.
    pub reply_tx: Option<oneshot::Sender<Result<R, String>>>,
}

impl<M, R> Envelope<M, R> {
    /// Create a tell envelope (no reply expected).
    pub fn tell(message: M) -> Self {
        Self {
            message,
            reply_tx: None,
        }
    }

    /// Create an ask envelope with a reply channel.
    pub fn ask(message: M, reply_tx: oneshot::Sender<Result<R, String>>) -> Self {
        Self {
            message,
            reply_tx: Some(reply_tx),
        }
    }
}

/// Sender side of a mailbox, supporting both bounded and unbounded variants.
#[derive(Debug)]
pub enum MailboxSender<M> {
    /// Bounded sender with backpressure.
    Bounded(mpsc::Sender<M>),
    /// Unbounded sender that never blocks.
    Unbounded(mpsc::UnboundedSender<M>),
}

impl<M> Clone for MailboxSender<M> {
    fn clone(&self) -> Self {
        match self {
            MailboxSender::Bounded(tx) => MailboxSender::Bounded(tx.clone()),
            MailboxSender::Unbounded(tx) => MailboxSender::Unbounded(tx.clone()),
        }
    }
}

impl<M> MailboxSender<M> {
    /// Send a message, waiting if the mailbox is full (bounded only).
    ///
    /// For unbounded mailboxes, this never blocks.
    /// Returns `ActorError::ActorStopped` if the receiver has been dropped.
    pub async fn send(&self, message: M) -> Result<(), ActorError> {
        match self {
            MailboxSender::Bounded(tx) => {
                tx.send(message).await.map_err(|_| ActorError::ActorStopped)
            }
            MailboxSender::Unbounded(tx) => tx.send(message).map_err(|_| ActorError::ActorStopped),
        }
    }

    /// Try to send a message without waiting.
    ///
    /// Returns `ActorError::MailboxFull` if the bounded mailbox is at capacity.
    /// Returns `ActorError::ActorStopped` if the receiver has been dropped.
    pub fn try_send(&self, message: M) -> Result<(), ActorError> {
        match self {
            MailboxSender::Bounded(tx) => tx.try_send(message).map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => ActorError::MailboxFull,
                mpsc::error::TrySendError::Closed(_) => ActorError::ActorStopped,
            }),
            MailboxSender::Unbounded(tx) => tx.send(message).map_err(|_| ActorError::ActorStopped),
        }
    }

    /// Returns true if the receiving end has been dropped.
    pub fn is_closed(&self) -> bool {
        match self {
            MailboxSender::Bounded(tx) => tx.is_closed(),
            MailboxSender::Unbounded(tx) => tx.is_closed(),
        }
    }
}

/// Receiver side of a mailbox.
pub enum MailboxReceiver<M> {
    /// Bounded receiver.
    Bounded(mpsc::Receiver<M>),
    /// Unbounded receiver.
    Unbounded(mpsc::UnboundedReceiver<M>),
}

impl<M> MailboxReceiver<M> {
    /// Receive the next message, waiting if the mailbox is empty.
    ///
    /// Returns `None` when all senders have been dropped.
    pub async fn recv(&mut self) -> Option<M> {
        match self {
            MailboxReceiver::Bounded(rx) => rx.recv().await,
            MailboxReceiver::Unbounded(rx) => rx.recv().await,
        }
    }

    /// Close the receiver, preventing new messages from being sent.
    pub fn close(&mut self) {
        match self {
            MailboxReceiver::Bounded(rx) => rx.close(),
            MailboxReceiver::Unbounded(rx) => rx.close(),
        }
    }
}

/// Create a mailbox channel pair based on the given configuration.
///
/// Returns a `(MailboxSender, MailboxReceiver)` tuple.
#[allow(clippy::type_complexity)]
pub fn create_mailbox<M, R>(
    config: &MailboxConfig,
) -> (
    MailboxSender<Envelope<M, R>>,
    MailboxReceiver<Envelope<M, R>>,
) {
    match config.capacity {
        Some(capacity) => {
            let (tx, rx) = mpsc::channel(capacity);
            (MailboxSender::Bounded(tx), MailboxReceiver::Bounded(rx))
        }
        None => {
            let (tx, rx) = mpsc::unbounded_channel();
            (MailboxSender::Unbounded(tx), MailboxReceiver::Unbounded(rx))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T014: Mailbox unit tests

    #[tokio::test]
    async fn bounded_send_receive() {
        let (tx, mut rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        tx.send(Envelope::tell("hello".to_string())).await.unwrap();
        let env = rx.recv().await.unwrap();
        assert_eq!(env.message, "hello");
        assert!(env.reply_tx.is_none());
    }

    #[tokio::test]
    async fn unbounded_send_receive() {
        let (tx, mut rx) = create_mailbox::<String, u32>(&MailboxConfig::unbounded());
        tx.send(Envelope::tell("world".to_string())).await.unwrap();
        let env = rx.recv().await.unwrap();
        assert_eq!(env.message, "world");
    }

    #[tokio::test]
    async fn bounded_capacity_rejection() {
        let (tx, _rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(2));
        // Fill the mailbox
        tx.try_send(Envelope::tell(1)).unwrap();
        tx.try_send(Envelope::tell(2)).unwrap();
        // Third message should fail with MailboxFull
        let err = tx.try_send(Envelope::tell(3)).unwrap_err();
        assert!(matches!(err, ActorError::MailboxFull));
    }

    #[tokio::test]
    async fn fifo_ordering() {
        let (tx, mut rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(10));
        for i in 0..5 {
            tx.send(Envelope::tell(i)).await.unwrap();
        }
        for i in 0..5 {
            let env = rx.recv().await.unwrap();
            assert_eq!(env.message, i);
        }
    }

    #[tokio::test]
    async fn send_to_dropped_receiver_returns_actor_stopped() {
        let (tx, rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(10));
        drop(rx);
        let err = tx.send(Envelope::tell(1)).await.unwrap_err();
        assert!(matches!(err, ActorError::ActorStopped));
    }

    #[tokio::test]
    async fn try_send_to_dropped_receiver_returns_actor_stopped() {
        let (tx, rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(10));
        drop(rx);
        let err = tx.try_send(Envelope::tell(1)).unwrap_err();
        assert!(matches!(err, ActorError::ActorStopped));
    }

    #[tokio::test]
    async fn unbounded_never_rejects() {
        let (tx, _rx) = create_mailbox::<u32, ()>(&MailboxConfig::unbounded());
        // Send many messages — unbounded should never fail
        for i in 0..1000 {
            tx.try_send(Envelope::tell(i)).unwrap();
        }
    }

    #[tokio::test]
    async fn envelope_ask_has_reply_channel() {
        let (reply_tx, reply_rx) = oneshot::channel::<Result<String, String>>();
        let env = Envelope::ask("question".to_string(), reply_tx);
        assert!(env.reply_tx.is_some());
        // Send reply through the channel
        env.reply_tx
            .unwrap()
            .send(Ok("answer".to_string()))
            .unwrap();
        let result = reply_rx.await.unwrap();
        assert_eq!(result.unwrap(), "answer");
    }

    #[test]
    fn mailbox_config_defaults() {
        let config = MailboxConfig::default();
        assert_eq!(config.capacity, Some(1000));
        assert!(config.is_bounded());
    }

    #[test]
    fn spawn_config_defaults() {
        let config = SpawnConfig::default();
        assert_eq!(config.mailbox.capacity, Some(1000));
        assert_eq!(config.restart_scope, RestartScope::Permanent);
        assert_eq!(config.shutdown_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn is_closed_after_receiver_drop() {
        let (tx, rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(10));
        assert!(!tx.is_closed());
        drop(rx);
        assert!(tx.is_closed());
    }

    #[tokio::test]
    async fn receiver_close_prevents_new_sends() {
        let (tx, mut rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(10));
        rx.close();
        // Existing messages in buffer can still be received, but new sends fail
        // (since buffer is empty and closed, try_send should fail)
        let err = tx.try_send(Envelope::tell(1)).unwrap_err();
        assert!(matches!(err, ActorError::ActorStopped));
    }
}

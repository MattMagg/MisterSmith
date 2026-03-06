//! Typed actor reference for sending messages to actors.
//!
//! `ActorRef<M, R>` provides `tell` (fire-and-forget) and `ask` (request-response) patterns.
//! It is cheaply cloneable and thread-safe (`Send + Sync`).

use std::time::Duration;

use mister_smith_core::{ActorError, AgentId};
use tokio::sync::oneshot;

use crate::mailbox::{Envelope, MailboxSender};

/// A typed reference to a running actor, used to send messages.
///
/// `ActorRef` wraps a [`MailboxSender`] and provides the public API for
/// communicating with an actor. It is `Clone`, `Send`, and `Sync`.
#[derive(Debug, Clone)]
pub struct ActorRef<M, R> {
    actor_id: AgentId,
    sender: MailboxSender<Envelope<M, R>>,
}

impl<M: Send + 'static, R: Send + 'static> ActorRef<M, R> {
    /// Create a new actor reference.
    pub fn new(actor_id: AgentId, sender: MailboxSender<Envelope<M, R>>) -> Self {
        Self { actor_id, sender }
    }

    /// Send a message without waiting for a reply (fire-and-forget).
    ///
    /// Returns `ActorError::ActorStopped` if the actor has terminated.
    /// Returns `ActorError::MailboxFull` if the bounded mailbox is at capacity.
    pub fn tell(&self, message: M) -> Result<(), ActorError> {
        self.sender.try_send(Envelope::tell(message))
    }

    /// Send a message and wait for a reply with the given timeout.
    ///
    /// Returns `ActorError::AskTimeout` if the actor does not reply within the timeout.
    /// Returns `ActorError::ActorStopped` if the actor has terminated.
    pub async fn ask(&self, message: M, timeout: Duration) -> Result<R, ActorError> {
        let (reply_tx, reply_rx) = oneshot::channel::<Result<R, String>>();
        self.sender.try_send(Envelope::ask(message, reply_tx))?;

        match tokio::time::timeout(timeout, reply_rx).await {
            Ok(Ok(Ok(response))) => Ok(response),
            Ok(Ok(Err(e))) => Err(ActorError::MessageHandlingFailed(e)),
            Ok(Err(_)) => Err(ActorError::ActorStopped),
            Err(_) => Err(ActorError::AskTimeout),
        }
    }

    /// Returns true if the actor's mailbox is still open (actor is alive).
    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Returns the actor's unique identifier.
    pub fn actor_id(&self) -> AgentId {
        self.actor_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mailbox::{create_mailbox, MailboxConfig};

    // T015: ActorRef unit tests

    #[tokio::test]
    async fn tell_to_alive_actor() {
        let id = AgentId::new();
        let (tx, mut rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);

        actor_ref.tell("hello".to_string()).unwrap();
        let env = rx.recv().await.unwrap();
        assert_eq!(env.message, "hello");
    }

    #[tokio::test]
    async fn tell_to_stopped_actor() {
        let id = AgentId::new();
        let (tx, rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);
        drop(rx);

        let err = actor_ref.tell("hello".to_string()).unwrap_err();
        assert!(matches!(err, ActorError::ActorStopped));
    }

    #[tokio::test]
    async fn is_alive_when_running() {
        let id = AgentId::new();
        let (tx, _rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);
        assert!(actor_ref.is_alive());
    }

    #[tokio::test]
    async fn is_alive_when_stopped() {
        let id = AgentId::new();
        let (tx, rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);
        drop(rx);
        assert!(!actor_ref.is_alive());
    }

    #[test]
    fn actor_id_returns_correct_id() {
        let id = AgentId::new();
        let (tx, _rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);
        assert_eq!(actor_ref.actor_id(), id);
    }

    #[tokio::test]
    async fn ask_timeout() {
        let id = AgentId::new();
        let (tx, _rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);

        // Nobody will reply, so this should timeout
        let err = actor_ref
            .ask("question".to_string(), Duration::from_millis(10))
            .await
            .unwrap_err();
        assert!(matches!(err, ActorError::AskTimeout));
    }

    #[tokio::test]
    async fn ask_with_reply() {
        let id = AgentId::new();
        let (tx, mut rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);

        // Spawn a task to reply
        let handle = tokio::spawn(async move {
            let env = rx.recv().await.unwrap();
            if let Some(reply_tx) = env.reply_tx {
                reply_tx.send(Ok(42)).unwrap();
            }
        });

        let result = actor_ref
            .ask("question".to_string(), Duration::from_secs(1))
            .await;
        assert_eq!(result.unwrap(), 42);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn ask_to_stopped_actor() {
        let id = AgentId::new();
        let (tx, rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);
        drop(rx);

        let err = actor_ref
            .ask("question".to_string(), Duration::from_secs(1))
            .await
            .unwrap_err();
        assert!(matches!(err, ActorError::ActorStopped));
    }

    #[tokio::test]
    async fn tell_to_full_mailbox() {
        let id = AgentId::new();
        let (tx, _rx) = create_mailbox::<u32, ()>(&MailboxConfig::bounded(2));
        let actor_ref = ActorRef::new(id, tx);

        actor_ref.tell(1).unwrap();
        actor_ref.tell(2).unwrap();
        let err = actor_ref.tell(3).unwrap_err();
        assert!(matches!(err, ActorError::MailboxFull));
    }

    #[test]
    fn actor_ref_is_clone() {
        let id = AgentId::new();
        let (tx, _rx) = create_mailbox::<String, u32>(&MailboxConfig::bounded(10));
        let actor_ref = ActorRef::new(id, tx);
        let cloned = actor_ref.clone();
        assert_eq!(actor_ref.actor_id(), cloned.actor_id());
    }
}

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use mister_smith_core::AgentId;
use mister_smith_transport::envelope::MessageEnvelope;
use mister_smith_transport::Transport;
use tokio::sync::watch;
use tokio::task::JoinHandle;

/// Emits periodic heartbeat messages on `agents.{id}.heartbeat`.
pub struct HeartbeatEmitter {
    agent_id: AgentId,
    interval: Duration,
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
}

impl HeartbeatEmitter {
    pub fn new(agent_id: AgentId, interval: Duration) -> Self {
        Self {
            agent_id,
            interval,
            handle: None,
            stop_tx: None,
        }
    }

    /// Start emitting heartbeats on a background task.
    pub fn start(&mut self, transport: Arc<dyn Transport>) {
        let agent_id = self.agent_id;
        let interval = self.interval;
        let (stop_tx, mut stop_rx) = watch::channel(false);
        self.stop_tx = Some(stop_tx);

        let handle = tokio::spawn(async move {
            let subject = format!("agents.{agent_id}.heartbeat");
            let mut ticker = tokio::time::interval(interval);

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let payload = serde_json::json!({
                            "agent_id": agent_id.to_string(),
                            "timestamp": Utc::now().to_rfc3339(),
                        });
                        let envelope = MessageEnvelope::builder("heartbeat")
                            .source_agent_id(*agent_id.as_ref())
                            .payload_json(&payload)
                            .ok()
                            .and_then(|b| b.build().ok());

                        if let Some(env) = envelope {
                            let _ = transport.publish(&subject, env).await;
                        }
                    }
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Stop the heartbeat emitter.
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }
}

impl Drop for HeartbeatEmitter {
    fn drop(&mut self) {
        self.stop();
    }
}

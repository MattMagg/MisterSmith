//! KV watcher for change notifications.
//!
//! Wraps [`kv::Store::watch`](async_nats::jetstream::kv::Store::watch) to
//! emit [`StateChange`] events as a tokio stream.
//! Each event includes the key that changed, the operation type, and the
//! revision number for ordering.

use async_nats::jetstream::kv;
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::error::{from_kv_error, PersistenceError};
use crate::kv::state::{Operation, StateChange};

/// Watches a KV bucket for changes matching a key pattern.
///
/// Returns a receiver that emits [`StateChange`] events. The watcher runs
/// in a background task until the receiver is dropped or the bucket is closed.
///
/// # Arguments
///
/// * `store` - The KV bucket to watch.
/// * `pattern` - Key pattern to filter changes (e.g., `">"` for all keys,
///   `"agent_id:*"` for a specific agent's state).
/// * `buffer` - Channel buffer size (use 256 for most cases).
///
/// # Examples
///
/// ```rust,ignore
/// let mut rx = watch_keys(&store, ">", 256).await?;
/// while let Some(change) = rx.recv().await {
///     println!("Key {} changed: {:?}", change.key, change.operation);
/// }
/// ```
pub async fn watch_keys(
    store: &kv::Store,
    pattern: &str,
    buffer: usize,
) -> Result<mpsc::Receiver<StateChange>, PersistenceError> {
    let mut watcher = store.watch(pattern).await.map_err(from_kv_error)?;

    let (tx, rx) = mpsc::channel(buffer);

    tokio::spawn(async move {
        while let Some(entry) = watcher.next().await {
            match entry {
                Ok(entry) => {
                    let operation = match entry.operation {
                        kv::Operation::Put => Operation::Put,
                        kv::Operation::Delete => Operation::Delete,
                        kv::Operation::Purge => Operation::Purge,
                    };

                    let change = StateChange {
                        key: entry.key.clone(),
                        operation,
                        revision: entry.revision,
                    };

                    debug!(
                        key = %entry.key,
                        operation = ?operation,
                        revision = entry.revision,
                        "KV change event"
                    );

                    if tx.send(change).await.is_err() {
                        debug!("Watch receiver dropped, stopping watcher");
                        break;
                    }
                }
                Err(err) => {
                    warn!(error = %err, "KV watch stream error");
                    // Continue watching — transient errors shouldn't kill the watcher
                }
            }
        }
        debug!("KV watch loop ended");
    });

    Ok(rx)
}

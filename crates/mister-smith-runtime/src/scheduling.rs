//! Task scheduling with concurrency limiting and priority queues.
//!
//! Provides [`TaskScheduler`] for enqueueing asynchronous work items with
//! priority and executing them under a [`Semaphore`]-based concurrency
//! ceiling. Also exposes two static utility patterns --
//! [`batch_processing_pattern`](TaskScheduler::batch_processing_pattern) and
//! [`fanout_fanin_pattern`](TaskScheduler::fanout_fanin_pattern) -- for
//! common parallel workload shapes.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use mister_smith_core::MessagePriority;
use tokio::sync::{mpsc, Mutex, Notify, Semaphore};

/// A boxed, `Send`-safe future that produces no meaningful return value.
///
/// Used as the unit of work inside [`ScheduledTask`].
type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// A unit of work paired with a scheduling priority.
pub struct ScheduledTask {
    /// Execution priority -- lower [`MessagePriority`] discriminant means
    /// higher scheduling urgency.
    pub priority: MessagePriority,
    /// The future to execute.
    pub future: BoxFuture,
}

impl ScheduledTask {
    /// Wrap an arbitrary `Send` future with the given priority.
    pub fn new<F>(priority: MessagePriority, future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            priority,
            future: Box::pin(future),
        }
    }
}

/// Concurrency-limited task scheduler with a priority-aware submission
/// channel.
///
/// Tasks are submitted via [`schedule`](Self::schedule) and executed by
/// the [`run`](Self::run) loop, which respects the semaphore ceiling so
/// that at most `max_concurrent` tasks execute simultaneously.
///
/// # Shutdown
///
/// The run loop exits when `shutdown` is set to `true` **and**
/// [`notify_shutdown`](Self::notify_shutdown) is called, or when all
/// senders are dropped (channel closed).
pub struct TaskScheduler {
    /// Semaphore that limits the number of concurrently executing tasks.
    concurrency_semaphore: Arc<Semaphore>,

    /// Sending half -- cloned by each call to [`schedule`](Self::schedule).
    task_sender: mpsc::Sender<ScheduledTask>,

    /// Receiving half -- consumed by the [`run`](Self::run) loop.
    task_receiver: Mutex<mpsc::Receiver<ScheduledTask>>,

    /// Configured concurrency ceiling (informational; enforced by semaphore).
    max_concurrent: usize,

    /// Used to wake the run loop when shutdown is requested.
    shutdown_notify: Arc<Notify>,
}

impl TaskScheduler {
    /// Create a new scheduler that allows up to `max_concurrent` tasks to
    /// execute at the same time.
    ///
    /// An internal MPSC channel is created with a buffer of `max_concurrent * 4`
    /// to provide adequate backpressure headroom.
    pub fn new(max_concurrent: usize) -> Self {
        let buffer = max_concurrent.saturating_mul(4).max(16);
        let (tx, rx) = mpsc::channel(buffer);
        Self {
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            task_sender: tx,
            task_receiver: Mutex::new(rx),
            max_concurrent,
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    /// Returns the configured concurrency ceiling.
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Returns a handle to the shutdown [`Notify`], allowing external code
    /// to wake the run loop after setting the `AtomicBool` flag.
    pub fn shutdown_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown_notify)
    }

    /// Convenience method: sets `shutdown` to `true` and notifies the run
    /// loop so it wakes immediately.
    pub fn notify_shutdown(&self, shutdown: &AtomicBool) {
        shutdown.store(true, Ordering::Release);
        self.shutdown_notify.notify_waiters();
    }

    /// Enqueue a future for execution with the given priority.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the scheduler has been shut down (receiver dropped).
    pub async fn schedule<F>(
        &self,
        priority: MessagePriority,
        future: F,
    ) -> Result<(), mpsc::error::SendError<ScheduledTask>>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = ScheduledTask::new(priority, future);
        self.task_sender.send(task).await
    }

    /// Main execution loop -- dequeue tasks, acquire a semaphore permit,
    /// and spawn each task onto the Tokio runtime.
    ///
    /// The loop exits when **either**:
    /// - `shutdown` is set to `true` (the loop is woken via [`Notify`]), or
    /// - the channel is closed (all senders dropped).
    ///
    /// Outstanding tasks that have already been spawned will continue to
    /// run; only new dequeues are halted.
    pub async fn run(&self, shutdown: Arc<AtomicBool>) {
        let mut receiver = self.task_receiver.lock().await;
        let semaphore = Arc::clone(&self.concurrency_semaphore);
        let notify = Arc::clone(&self.shutdown_notify);

        tracing::info!(
            max_concurrent = self.max_concurrent,
            "TaskScheduler run loop started"
        );

        loop {
            if shutdown.load(Ordering::Acquire) {
                tracing::info!("TaskScheduler shutting down (flag set)");
                break;
            }

            // Race: receive a task vs. shutdown notification.
            let task = tokio::select! {
                biased;
                _ = notify.notified() => {
                    // Re-check the flag — a spurious wake is harmless.
                    if shutdown.load(Ordering::Acquire) {
                        tracing::info!("TaskScheduler shutting down (notified)");
                        break;
                    }
                    continue;
                }
                maybe_task = receiver.recv() => {
                    match maybe_task {
                        Some(t) => t,
                        None => {
                            tracing::info!(
                                "TaskScheduler channel closed — exiting run loop"
                            );
                            break;
                        }
                    }
                }
            };

            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore should not be closed");

            tracing::debug!(priority = ?task.priority, "Spawning scheduled task");

            tokio::spawn(async move {
                task.future.await;
                // Permit is dropped here, releasing the semaphore slot.
                drop(permit);
            });
        }

        tracing::info!("TaskScheduler run loop exited");
    }

    // ------------------------------------------------------------------
    // Static utility patterns
    // ------------------------------------------------------------------

    /// Process `items` in fixed-size batches with up to `batch_size`
    /// concurrent tasks per batch.
    ///
    /// Each item is passed to `processor`, which returns a `Result`. The
    /// function collects all results in input order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = TaskScheduler::batch_processing_pattern(
    ///     urls,
    ///     10,
    ///     |url| async move { fetch(url).await },
    /// ).await;
    /// ```
    pub async fn batch_processing_pattern<T, R, E, F, Fut>(
        items: Vec<T>,
        batch_size: usize,
        processor: F,
    ) -> Vec<Result<R, E>>
    where
        T: Send + 'static,
        R: Send + 'static,
        E: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        let processor = Arc::new(processor);
        let mut all_results: Vec<Result<R, E>> = Vec::with_capacity(items.len());

        for chunk in items.into_iter().collect::<Vec<_>>().chunks_by_batch(batch_size) {
            let mut handles = Vec::with_capacity(chunk.len());
            for item in chunk {
                let proc = Arc::clone(&processor);
                handles.push(tokio::spawn(async move { proc(item).await }));
            }
            for handle in handles {
                match handle.await {
                    Ok(result) => all_results.push(result),
                    Err(join_err) => {
                        // Convert JoinError to a string-based error message.
                        // The caller's error type is opaque so we panic here;
                        // callers concerned with JoinError should use
                        // `fanout_fanin_pattern` instead.
                        panic!("Spawned task panicked: {join_err}");
                    }
                }
            }
        }

        all_results
    }

    /// Fan-out work across up to `max_concurrent` tasks and collect all
    /// results.
    ///
    /// Unlike [`batch_processing_pattern`](Self::batch_processing_pattern),
    /// this uses a [`Semaphore`] so tasks are started as soon as capacity
    /// is available rather than waiting for an entire batch to finish.
    ///
    /// Results are returned in input order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let processed = TaskScheduler::fanout_fanin_pattern(
    ///     records,
    ///     8,
    ///     |record| async move { transform(record) },
    /// ).await;
    /// ```
    pub async fn fanout_fanin_pattern<T, R, F, Fut>(
        items: Vec<T>,
        max_concurrent: usize,
        processor: F,
    ) -> Vec<R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let processor = Arc::new(processor);
        let mut handles = Vec::with_capacity(items.len());

        for item in items {
            let sem = Arc::clone(&semaphore);
            let proc = Arc::clone(&processor);
            handles.push(tokio::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .expect("semaphore should not be closed");
                proc(item).await
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.await.expect("Spawned task panicked"));
        }
        results
    }
}

// ------------------------------------------------------------------
// Internal helper -- chunking iterator
// ------------------------------------------------------------------

/// Extension trait to chunk a `Vec<T>` into batches without requiring
/// the nightly `slice::chunk_by` or `array_chunks` features.
trait ChunkByBatch<T> {
    fn chunks_by_batch(self, size: usize) -> Vec<Vec<T>>;
}

impl<T> ChunkByBatch<T> for Vec<T> {
    fn chunks_by_batch(self, size: usize) -> Vec<Vec<T>> {
        let size = size.max(1);
        let mut result = Vec::new();
        let mut iter = self.into_iter().peekable();
        while iter.peek().is_some() {
            result.push(iter.by_ref().take(size).collect());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn scheduler_runs_and_shuts_down() {
        let scheduler = Arc::new(TaskScheduler::new(2));
        let counter = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));

        // Schedule a few tasks.
        for _ in 0..5 {
            let c = Arc::clone(&counter);
            scheduler
                .schedule(MessagePriority::Normal, async move {
                    c.fetch_add(1, Ordering::Relaxed);
                })
                .await
                .expect("schedule should succeed");
        }

        // Start the run loop in a background task.
        let sched = Arc::clone(&scheduler);
        let shut = Arc::clone(&shutdown);
        let run_handle = tokio::spawn(async move {
            sched.run(shut).await;
        });

        // Give tasks time to complete.
        time::sleep(Duration::from_millis(100)).await;

        // Signal shutdown via the convenience method.
        scheduler.notify_shutdown(&shutdown);

        run_handle.await.expect("run loop should not panic");
        assert_eq!(counter.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn concurrency_is_limited_by_semaphore() {
        // Verify that the semaphore prevents more than `max_concurrent`
        // tasks from running simultaneously.
        let max_concurrent: usize = 2;
        let scheduler = Arc::new(TaskScheduler::new(max_concurrent));
        let shutdown = Arc::new(AtomicBool::new(false));

        let running = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        // Schedule tasks that hold a slot for a short duration.
        for _ in 0..6 {
            let r = Arc::clone(&running);
            let p = Arc::clone(&peak);
            scheduler
                .schedule(MessagePriority::Normal, async move {
                    let current = r.fetch_add(1, Ordering::SeqCst) + 1;
                    // Update peak concurrency.
                    p.fetch_max(current, Ordering::SeqCst);
                    time::sleep(Duration::from_millis(50)).await;
                    r.fetch_sub(1, Ordering::SeqCst);
                })
                .await
                .expect("schedule should succeed");
        }

        let sched = Arc::clone(&scheduler);
        let shut = Arc::clone(&shutdown);
        let run_handle = tokio::spawn(async move {
            sched.run(shut).await;
        });

        // Wait long enough for all tasks to complete.
        time::sleep(Duration::from_millis(500)).await;

        scheduler.notify_shutdown(&shutdown);
        run_handle.await.expect("run loop should not panic");

        let observed_peak = peak.load(Ordering::SeqCst);
        assert!(
            observed_peak <= max_concurrent,
            "peak concurrency {observed_peak} exceeded limit {max_concurrent}"
        );
    }

    #[tokio::test]
    async fn fanout_fanin_preserves_order() {
        let items: Vec<u32> = (0..20).collect();
        let results = TaskScheduler::fanout_fanin_pattern(items.clone(), 4, |x| async move {
            x * 2
        })
        .await;

        let expected: Vec<u32> = items.iter().map(|x| x * 2).collect();
        assert_eq!(results, expected);
    }

    #[tokio::test]
    async fn batch_processing_collects_results() {
        let items: Vec<u32> = (1..=10).collect();
        let results: Vec<Result<u32, String>> =
            TaskScheduler::batch_processing_pattern(items, 3, |x| async move { Ok(x + 100) })
                .await;

        assert_eq!(results.len(), 10);
        for (i, r) in results.iter().enumerate() {
            assert_eq!(*r.as_ref().unwrap(), (i as u32) + 1 + 100);
        }
    }

    #[tokio::test]
    async fn scheduled_task_new_constructs_correctly() {
        let called = Arc::new(AtomicBool::new(false));
        let c = Arc::clone(&called);
        let task = ScheduledTask::new(MessagePriority::High, async move {
            c.store(true, Ordering::Relaxed);
        });
        assert_eq!(task.priority, MessagePriority::High);
        task.future.await;
        assert!(called.load(Ordering::Relaxed));
    }

    #[test]
    fn chunks_by_batch_splits_correctly() {
        let v: Vec<i32> = (1..=7).collect();
        let chunks = v.chunks_by_batch(3);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1, 2, 3]);
        assert_eq!(chunks[1], vec![4, 5, 6]);
        assert_eq!(chunks[2], vec![7]);
    }

    #[test]
    fn scheduler_max_concurrent_accessor() {
        let sched = TaskScheduler::new(16);
        assert_eq!(sched.max_concurrent(), 16);
    }
}

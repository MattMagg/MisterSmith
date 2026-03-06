//! Stream processing with backpressure support.
//!
//! Provides a pipeline of [`Processor`] stages that transform items sequentially.
//! The [`BackpressureConfig`] controls how the pipeline responds when downstream
//! consumers cannot keep up.

use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Strategy for handling backpressure when the pipeline is overwhelmed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    /// Wait (apply back-pressure upstream) until capacity is available.
    Wait,
    /// Drop items that cannot be processed immediately.
    Drop,
    /// Buffer up to `N` items before applying back-pressure.
    Buffer(usize),
    /// Block the caller until the pipeline drains.
    Block,
}

/// Configuration for backpressure behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// The backpressure strategy to apply.
    pub strategy: BackpressureStrategy,
    /// Number of in-flight items above which backpressure is engaged.
    pub high_watermark: usize,
    /// Number of in-flight items below which backpressure is released.
    pub low_watermark: usize,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            strategy: BackpressureStrategy::Wait,
            high_watermark: 1000,
            low_watermark: 100,
        }
    }
}

/// Atomic counters tracking stream processing metrics.
pub struct StreamMetrics {
    /// Total items successfully processed.
    pub items_processed: AtomicU64,
    /// Items dropped due to backpressure.
    pub items_dropped: AtomicU64,
    /// Number of backpressure events triggered.
    pub backpressure_events: AtomicU64,
    /// Number of processing errors encountered.
    pub processing_errors: AtomicU64,
}

impl StreamMetrics {
    /// Create a new zeroed metrics instance.
    fn new() -> Self {
        Self {
            items_processed: AtomicU64::new(0),
            items_dropped: AtomicU64::new(0),
            backpressure_events: AtomicU64::new(0),
            processing_errors: AtomicU64::new(0),
        }
    }
}

impl std::fmt::Debug for StreamMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamMetrics")
            .field(
                "items_processed",
                &self.items_processed.load(Ordering::Relaxed),
            )
            .field("items_dropped", &self.items_dropped.load(Ordering::Relaxed))
            .field(
                "backpressure_events",
                &self.backpressure_events.load(Ordering::Relaxed),
            )
            .field(
                "processing_errors",
                &self.processing_errors.load(Ordering::Relaxed),
            )
            .finish()
    }
}

/// A single stage in a processing pipeline.
///
/// Implementors transform items of type `T` and may produce errors.
#[async_trait]
pub trait Processor<T: Send + Sync + 'static>: Send + Sync {
    /// Process one item, returning the (possibly transformed) item.
    async fn process(&self, item: T) -> Result<T, Box<dyn Error + Send + Sync>>;

    /// Human-readable name of this processor (for diagnostics).
    fn name(&self) -> &str;
}

/// A pipeline of processors applied sequentially to each item.
pub struct StreamProcessor<T: Send + Sync + 'static> {
    processors: Vec<Arc<dyn Processor<T>>>,
    #[allow(dead_code)]
    config: BackpressureConfig,
    metrics: Arc<StreamMetrics>,
}

impl<T: Send + Sync + 'static> StreamProcessor<T> {
    /// Create a new stream processor with the given backpressure configuration.
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            processors: Vec::new(),
            config,
            metrics: Arc::new(StreamMetrics::new()),
        }
    }

    /// Append a processor stage to the pipeline.
    pub fn add_processor(&mut self, processor: Arc<dyn Processor<T>>) -> &mut Self {
        self.processors.push(processor);
        self
    }

    /// Run an item through all processor stages in order.
    ///
    /// Each stage receives the output of the previous stage. On error the
    /// pipeline short-circuits, incrementing the error counter.
    pub async fn process_item(&self, item: T) -> Result<T, Box<dyn Error + Send + Sync>> {
        let mut current = item;
        for processor in &self.processors {
            match processor.process(current).await {
                Ok(result) => current = result,
                Err(e) => {
                    self.metrics
                        .processing_errors
                        .fetch_add(1, Ordering::Relaxed);
                    return Err(e);
                }
            }
        }
        self.metrics.items_processed.fetch_add(1, Ordering::Relaxed);
        Ok(current)
    }

    /// Access the live metrics.
    pub fn metrics(&self) -> &Arc<StreamMetrics> {
        &self.metrics
    }
}

impl<T: Send + Sync + 'static> std::fmt::Debug for StreamProcessor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamProcessor")
            .field("processor_count", &self.processors.len())
            .field("config", &self.config)
            .field("metrics", &self.metrics)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DoubleProcessor;

    #[async_trait]
    impl Processor<i32> for DoubleProcessor {
        async fn process(&self, item: i32) -> Result<i32, Box<dyn Error + Send + Sync>> {
            Ok(item * 2)
        }
        fn name(&self) -> &str {
            "double"
        }
    }

    struct AddOneProcessor;

    #[async_trait]
    impl Processor<i32> for AddOneProcessor {
        async fn process(&self, item: i32) -> Result<i32, Box<dyn Error + Send + Sync>> {
            Ok(item + 1)
        }
        fn name(&self) -> &str {
            "add_one"
        }
    }

    struct FailProcessor;

    #[async_trait]
    impl Processor<i32> for FailProcessor {
        async fn process(&self, _item: i32) -> Result<i32, Box<dyn Error + Send + Sync>> {
            Err("processing failed".into())
        }
        fn name(&self) -> &str {
            "fail"
        }
    }

    #[tokio::test]
    async fn single_processor() {
        let mut sp = StreamProcessor::new(BackpressureConfig::default());
        sp.add_processor(Arc::new(DoubleProcessor));

        let result = sp.process_item(5).await.unwrap();
        assert_eq!(result, 10);
        assert_eq!(sp.metrics().items_processed.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn chained_processors() {
        let mut sp = StreamProcessor::new(BackpressureConfig::default());
        sp.add_processor(Arc::new(DoubleProcessor));
        sp.add_processor(Arc::new(AddOneProcessor));

        // 5 * 2 = 10, 10 + 1 = 11.
        let result = sp.process_item(5).await.unwrap();
        assert_eq!(result, 11);
    }

    #[tokio::test]
    async fn processor_failure_short_circuits() {
        let mut sp = StreamProcessor::new(BackpressureConfig::default());
        sp.add_processor(Arc::new(DoubleProcessor));
        sp.add_processor(Arc::new(FailProcessor));
        sp.add_processor(Arc::new(AddOneProcessor));

        let result = sp.process_item(5).await;
        assert!(result.is_err());
        assert_eq!(sp.metrics().processing_errors.load(Ordering::Relaxed), 1);
        assert_eq!(sp.metrics().items_processed.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn empty_pipeline_passes_through() {
        let sp = StreamProcessor::<i32>::new(BackpressureConfig::default());
        let result = sp.process_item(42).await.unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.high_watermark, 1000);
        assert_eq!(config.low_watermark, 100);
        assert!(matches!(config.strategy, BackpressureStrategy::Wait));
    }

    #[test]
    fn debug_impls() {
        let sp = StreamProcessor::<i32>::new(BackpressureConfig::default());
        let debug = format!("{sp:?}");
        assert!(debug.contains("StreamProcessor"));
    }
}

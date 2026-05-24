//! Dataset pipeline configuration.

use serde::{Deserialize, Serialize};

/// Parallel dataset pipeline configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DatasetPipelineConfig {
    /// Number of parser workers.
    pub workers: usize,
    /// Samples per emitted batch.
    pub batch_size: usize,
    /// Reader chunk size in MiB.
    pub chunk_size_mb: usize,
    /// Whether to shuffle samples.
    pub shuffle: bool,
    /// Optional deterministic shuffle seed.
    pub seed: Option<u64>,
    /// Number of prefetched batches.
    pub prefetch_batches: usize,
}

impl Default for DatasetPipelineConfig {
    fn default() -> Self {
        Self {
            workers: std::thread::available_parallelism().map_or(1, usize::from),
            batch_size: 1024,
            chunk_size_mb: 64,
            shuffle: false,
            seed: None,
            prefetch_batches: 4,
        }
    }
}

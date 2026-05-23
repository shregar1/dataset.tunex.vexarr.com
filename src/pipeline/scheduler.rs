//! Pipeline scheduling options.

use std::time::Duration;

/// Parallel pipeline options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PipelineOptions {
    /// Number of JSONL lines grouped into one parser chunk.
    pub chunk_lines: usize,
    /// Chunk queue capacity.
    pub chunk_queue_capacity: usize,
    /// Parsed batch queue capacity.
    pub batch_queue_capacity: usize,
    /// Producer backpressure timeout.
    pub send_timeout: Duration,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            chunk_lines: 4096,
            chunk_queue_capacity: 4,
            batch_queue_capacity: 4,
            send_timeout: Duration::from_millis(100),
        }
    }
}

/// Chunk scheduling policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChunkScheduler {
    options: PipelineOptions,
}

impl ChunkScheduler {
    /// Creates a scheduler from options.
    #[must_use]
    pub const fn new(options: PipelineOptions) -> Self {
        Self { options }
    }

    /// Returns true when the current chunk should be scheduled.
    #[must_use]
    pub fn should_schedule(self, current_lines: usize) -> bool {
        current_lines >= self.options.chunk_lines
    }
}

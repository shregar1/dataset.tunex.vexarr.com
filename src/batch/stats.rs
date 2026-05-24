//! Batch statistics.

use std::time::Duration;

/// Runtime statistics for dataset batch construction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BatchStats {
    /// Number of batches built.
    pub batches: u64,
    /// Total samples batched.
    pub samples: u64,
    /// Total payload bytes batched.
    pub bytes: u64,
    /// Total time spent building batches.
    pub build_time: Duration,
    /// Time spent waiting for queue operations.
    pub queue_wait_time: Duration,
    /// Estimated allocation count.
    pub allocations: u64,
}

impl BatchStats {
    /// Records one built batch.
    pub fn record_batch(&mut self, samples: usize, bytes: usize, build_time: Duration) {
        self.batches = self.batches.saturating_add(1);
        self.samples = self
            .samples
            .saturating_add(u64::try_from(samples).unwrap_or(u64::MAX));
        self.bytes = self
            .bytes
            .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
        self.build_time += build_time;
    }

    /// Returns average samples per batch.
    #[must_use]
    pub fn samples_per_batch(self) -> Option<f64> {
        (self.batches > 0).then_some(self.samples as f64 / self.batches as f64)
    }

    /// Returns average bytes per batch.
    #[must_use]
    pub fn bytes_per_batch(self) -> Option<f64> {
        (self.batches > 0).then_some(self.bytes as f64 / self.batches as f64)
    }
}

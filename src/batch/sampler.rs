//! Batch sampling policies.

use tunex_core::types::RawSample;

/// Batch sizing mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchSizing {
    /// Emit exactly `samples` except possibly the final batch.
    Fixed { samples: usize },
    /// Emit when either sample or byte threshold is reached.
    Dynamic {
        /// Maximum samples per batch.
        max_samples: usize,
        /// Maximum bytes per batch.
        max_bytes: usize,
    },
}

/// Decides when buffered samples should be emitted as a batch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchSampler {
    sizing: BatchSizing,
}

impl BatchSampler {
    /// Creates a fixed-size sampler.
    #[must_use]
    pub const fn fixed(samples: usize) -> Self {
        Self {
            sizing: BatchSizing::Fixed { samples },
        }
    }

    /// Creates a dynamic-size sampler.
    #[must_use]
    pub const fn dynamic(max_samples: usize, max_bytes: usize) -> Self {
        Self {
            sizing: BatchSizing::Dynamic {
                max_samples,
                max_bytes,
            },
        }
    }

    /// Returns the initial builder capacity.
    #[must_use]
    pub const fn capacity_hint(self) -> usize {
        match self.sizing {
            BatchSizing::Fixed { samples } => samples,
            BatchSizing::Dynamic { max_samples, .. } => max_samples,
        }
    }

    /// Returns true when the current buffered state should be emitted.
    #[must_use]
    pub fn should_emit(self, buffered_samples: usize, buffered_bytes: usize) -> bool {
        match self.sizing {
            BatchSizing::Fixed { samples } => buffered_samples >= samples,
            BatchSizing::Dynamic {
                max_samples,
                max_bytes,
            } => buffered_samples >= max_samples || buffered_bytes >= max_bytes,
        }
    }

    /// Returns true if this sample would exceed dynamic bytes.
    #[must_use]
    pub fn would_exceed(self, buffered_bytes: usize, sample: &RawSample) -> bool {
        match self.sizing {
            BatchSizing::Fixed { .. } => false,
            BatchSizing::Dynamic { max_bytes, .. } => {
                buffered_bytes > 0
                    && buffered_bytes.saturating_add(sample.payload.len()) > max_bytes
            }
        }
    }
}

impl Default for BatchSampler {
    fn default() -> Self {
        Self::fixed(1024)
    }
}

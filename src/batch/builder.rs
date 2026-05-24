//! Dataset batch builder.

use std::time::{Instant, SystemTime};

use bytes::Bytes;
use smallvec::SmallVec;
use tunex_core::types::RawSample;

use super::BatchStats;

/// Lightweight batch metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchMetadata {
    /// Batch sequence id.
    pub batch_id: u64,
    /// Build timestamp.
    pub created_at: SystemTime,
    /// Optional first source sample id.
    pub first_sample_id: Option<String>,
    /// Optional last source sample id.
    pub last_sample_id: Option<String>,
}

/// Throughput-oriented dataset batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetBatch {
    /// Contiguous sample payloads.
    pub payloads: Vec<Bytes>,
    /// Optional ids aligned with payloads.
    pub ids: SmallVec<[Option<String>; 8]>,
    /// Total payload bytes.
    pub bytes: usize,
    /// Batch metadata.
    pub metadata: BatchMetadata,
}

impl DatasetBatch {
    /// Returns sample count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.payloads.len()
    }

    /// Returns currently buffered payload bytes.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// Returns true when batch contains no samples.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payloads.is_empty()
    }
}

/// Reusable batch builder that preserves allocation capacity.
#[derive(Debug)]
pub struct BatchBuilder {
    batch_id: u64,
    payloads: Vec<Bytes>,
    ids: SmallVec<[Option<String>; 8]>,
    bytes: usize,
    stats: BatchStats,
}

impl BatchBuilder {
    /// Creates a builder with reusable capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            batch_id: 0,
            payloads: Vec::with_capacity(capacity),
            ids: SmallVec::with_capacity(capacity.min(8)),
            bytes: 0,
            stats: BatchStats {
                allocations: 2,
                ..BatchStats::default()
            },
        }
    }

    /// Pushes one raw sample into the current batch.
    pub fn push(&mut self, sample: RawSample) {
        self.bytes = self.bytes.saturating_add(sample.payload.len());
        self.ids.push(sample.id);
        self.payloads.push(sample.payload);
    }

    /// Returns current sample count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.payloads.len()
    }

    /// Returns currently buffered payload bytes.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// Returns true when no samples are buffered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payloads.is_empty()
    }

    /// Builds a batch and clears the reusable builder buffers.
    #[must_use]
    pub fn finish(&mut self) -> Option<DatasetBatch> {
        if self.is_empty() {
            return None;
        }

        let started = Instant::now();
        let first_sample_id = self.ids.first().cloned().flatten();
        let last_sample_id = self.ids.last().cloned().flatten();
        let payloads = std::mem::take(&mut self.payloads);
        let ids = std::mem::take(&mut self.ids);
        let bytes = std::mem::take(&mut self.bytes);
        let metadata = BatchMetadata {
            batch_id: self.batch_id,
            created_at: SystemTime::now(),
            first_sample_id,
            last_sample_id,
        };

        self.batch_id = self.batch_id.saturating_add(1);
        self.stats
            .record_batch(payloads.len(), bytes, started.elapsed());

        let capacity = payloads.capacity().max(1);
        self.payloads = Vec::with_capacity(capacity);
        self.ids = SmallVec::with_capacity(capacity.min(8));
        self.stats.allocations = self.stats.allocations.saturating_add(2);

        Some(DatasetBatch {
            payloads,
            ids,
            bytes,
            metadata,
        })
    }

    /// Returns builder statistics.
    #[must_use]
    pub const fn stats(&self) -> BatchStats {
        self.stats
    }
}

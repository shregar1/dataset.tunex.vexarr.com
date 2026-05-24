//! Dataset metric counters and ratios.

use std::time::Duration;

use super::{DatasetLatency, ThroughputSnapshot};

/// Monotonic counters and point-in-time utilization values for dataset work.
#[derive(Clone, Debug, Default)]
pub struct DatasetCounters {
    /// Samples emitted to consumers.
    pub samples_read: u64,
    /// Bytes read from input.
    pub bytes_read: u64,
    /// JSON records parsed successfully.
    pub json_records_parsed: u64,
    /// Batches emitted to consumers.
    pub batches_built: u64,
    /// Cache lookup hits.
    pub cache_hits: u64,
    /// Cache lookup misses.
    pub cache_misses: u64,
    /// CPU utilization in the last sample window.
    pub cpu_utilization: f64,
    /// Memory usage in bytes.
    pub memory_usage_bytes: u64,
    /// Worker utilization in the last sample window.
    pub worker_utilization: f64,
}

impl DatasetCounters {
    /// Records sample and byte progress.
    pub fn record_read(&mut self, samples: u64, bytes: u64) {
        self.samples_read = self.samples_read.saturating_add(samples);
        self.bytes_read = self.bytes_read.saturating_add(bytes);
    }

    /// Records a cache hit.
    pub fn record_cache_hit(&mut self) {
        self.cache_hits = self.cache_hits.saturating_add(1);
    }

    /// Records a cache miss.
    pub fn record_cache_miss(&mut self) {
        self.cache_misses = self.cache_misses.saturating_add(1);
    }

    /// Returns cache hit ratio in the inclusive range 0.0..=1.0.
    #[must_use]
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }

        self.cache_hits as f64 / total as f64
    }

    /// Returns cache miss ratio in the inclusive range 0.0..=1.0.
    #[must_use]
    pub fn cache_miss_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }

        self.cache_misses as f64 / total as f64
    }

    /// Builds throughput metrics for the given elapsed interval.
    #[must_use]
    pub fn throughput(&self, elapsed: Duration) -> ThroughputSnapshot {
        ThroughputSnapshot::new(self.samples_read, self.bytes_read, elapsed)
    }

    /// Emits counters to the active `metrics` recorder.
    pub fn emit(&self, latency: &DatasetLatency) {
        ::metrics::counter!("tunex_dataset_samples_total").absolute(self.samples_read);
        ::metrics::counter!("tunex_dataset_bytes_total").absolute(self.bytes_read);
        ::metrics::counter!("tunex_dataset_json_records_total").absolute(self.json_records_parsed);
        ::metrics::counter!("tunex_dataset_batches_total").absolute(self.batches_built);
        ::metrics::counter!("tunex_dataset_cache_hits_total").absolute(self.cache_hits);
        ::metrics::counter!("tunex_dataset_cache_misses_total").absolute(self.cache_misses);
        ::metrics::gauge!("tunex_dataset_cache_hit_ratio").set(self.cache_hit_ratio());
        ::metrics::gauge!("tunex_dataset_cache_miss_ratio").set(self.cache_miss_ratio());
        ::metrics::gauge!("tunex_dataset_cpu_utilization").set(self.cpu_utilization);
        ::metrics::gauge!("tunex_dataset_memory_usage_bytes").set(self.memory_usage_bytes as f64);
        ::metrics::gauge!("tunex_dataset_worker_utilization").set(self.worker_utilization);
        latency.emit();
    }
}

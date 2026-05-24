//! Dataset latency tracking.

use std::time::{Duration, Instant};

/// Latency totals for major dataset pipeline stages.
#[derive(Clone, Debug, Default)]
pub struct DatasetLatency {
    /// Time spent parsing JSON records.
    pub json_parse: Duration,
    /// Time spent reading memory-mapped data.
    pub mmap_read: Duration,
    /// Time spent decompressing input chunks.
    pub decompression: Duration,
    /// Time spent stalled on queues.
    pub queue_stall: Duration,
    /// Time spent building batches.
    pub batch_build: Duration,
    /// Time spent waiting for workers.
    pub worker_wait: Duration,
}

impl DatasetLatency {
    /// Emits latency histograms to the active `metrics` recorder.
    pub fn emit(&self) {
        emit_duration("tunex_dataset_json_parse_latency_seconds", self.json_parse);
        emit_duration("tunex_dataset_mmap_read_latency_seconds", self.mmap_read);
        emit_duration(
            "tunex_dataset_decompression_latency_seconds",
            self.decompression,
        );
        emit_duration("tunex_dataset_queue_stall_seconds", self.queue_stall);
        emit_duration(
            "tunex_dataset_batch_build_latency_seconds",
            self.batch_build,
        );
        emit_duration("tunex_dataset_worker_wait_seconds", self.worker_wait);
    }
}

/// Small RAII-free timer used by callers that already own the stats object.
#[derive(Clone, Debug)]
pub struct LatencyTimer {
    started_at: Instant,
}

impl LatencyTimer {
    /// Starts a new timer.
    #[must_use]
    pub fn start() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    /// Returns elapsed time without consuming the timer.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Adds elapsed time to a target duration.
    pub fn record_into(self, target: &mut Duration) {
        *target += self.elapsed();
    }
}

fn emit_duration(name: &'static str, duration: Duration) {
    ::metrics::histogram!(name).record(duration.as_secs_f64());
}

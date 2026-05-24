//! Dataset throughput snapshots.

use std::time::{Duration, Instant};

/// Point-in-time throughput measurement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThroughputSnapshot {
    /// Samples processed in the window.
    pub samples: u64,
    /// Bytes processed in the window.
    pub bytes: u64,
    /// Window duration.
    pub elapsed: Duration,
    /// Samples per second.
    pub samples_per_second: f64,
    /// Megabytes per second.
    pub megabytes_per_second: f64,
}

impl ThroughputSnapshot {
    /// Creates a throughput snapshot.
    #[must_use]
    pub fn new(samples: u64, bytes: u64, elapsed: Duration) -> Self {
        let seconds = elapsed.as_secs_f64();
        let (samples_per_second, megabytes_per_second) = if seconds > 0.0 {
            (
                samples as f64 / seconds,
                bytes as f64 / (1024.0 * 1024.0) / seconds,
            )
        } else {
            (0.0, 0.0)
        };

        Self {
            samples,
            bytes,
            elapsed,
            samples_per_second,
            megabytes_per_second,
        }
    }

    /// Emits throughput gauges to the active `metrics` recorder.
    pub fn emit(&self) {
        ::metrics::gauge!("tunex_dataset_samples_per_second").set(self.samples_per_second);
        ::metrics::gauge!("tunex_dataset_megabytes_per_second").set(self.megabytes_per_second);
    }
}

/// Tracks throughput from an initialization point.
#[derive(Clone, Debug)]
pub struct ThroughputMeter {
    started_at: Instant,
    samples: u64,
    bytes: u64,
}

impl ThroughputMeter {
    /// Starts a new throughput meter.
    #[must_use]
    pub fn start() -> Self {
        Self {
            started_at: Instant::now(),
            samples: 0,
            bytes: 0,
        }
    }

    /// Records progress.
    pub fn record(&mut self, samples: u64, bytes: u64) {
        self.samples = self.samples.saturating_add(samples);
        self.bytes = self.bytes.saturating_add(bytes);
    }

    /// Returns the current throughput snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ThroughputSnapshot {
        ThroughputSnapshot::new(self.samples, self.bytes, self.started_at.elapsed())
    }
}

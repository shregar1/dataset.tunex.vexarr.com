//! Metrics exporter contracts.

use crate::DatasetResult;

use super::{DatasetCounters, DatasetLatency, ThroughputSnapshot};

/// Full dataset metrics snapshot.
#[derive(Clone, Debug, Default)]
pub struct DatasetMetricsSnapshot {
    /// Counter values and utilization gauges.
    pub counters: DatasetCounters,
    /// Stage latency totals.
    pub latency: DatasetLatency,
    /// Throughput values.
    pub throughput: ThroughputSnapshot,
}

impl DatasetMetricsSnapshot {
    /// Emits this snapshot through the active process metrics recorder.
    pub fn emit(&self) {
        self.counters.emit(&self.latency);
        self.throughput.emit();
    }
}

/// Contract for exporting dataset metrics to external systems.
pub trait DatasetMetricsExporter: Send + Sync {
    /// Exports a metrics snapshot.
    fn export(&self, snapshot: &DatasetMetricsSnapshot) -> DatasetResult<()>;
}

/// Exporter that only emits to the process-global `metrics` recorder.
#[derive(Clone, Debug, Default)]
pub struct NoopMetricsExporter;

impl DatasetMetricsExporter for NoopMetricsExporter {
    fn export(&self, snapshot: &DatasetMetricsSnapshot) -> DatasetResult<()> {
        snapshot.emit();
        Ok(())
    }
}

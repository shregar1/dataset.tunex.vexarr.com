//! Dataset observability and profiling contracts.

pub mod counters;
pub mod exporter;
pub mod latency;
pub mod throughput;
pub mod tracing;

pub use counters::DatasetCounters;
pub use exporter::{DatasetMetricsExporter, DatasetMetricsSnapshot, NoopMetricsExporter};
pub use latency::{DatasetLatency, LatencyTimer};
pub use throughput::{ThroughputMeter, ThroughputSnapshot};
pub use tracing::{
    batch_build_span, cache_lookup_span, cache_store_span, chunk_read_span, dataset_open_span,
    json_parse_span, worker_wait_span,
};

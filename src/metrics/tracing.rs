//! Tracing span constructors for dataset operations.

use std::path::Path;

use tracing::Span;

/// Span for opening a dataset.
#[must_use]
pub fn dataset_open_span(path: &Path) -> Span {
    tracing::info_span!("dataset_open", path = %path.display())
}

/// Span for reading an input chunk.
#[must_use]
pub fn chunk_read_span(chunk_index: u64) -> Span {
    tracing::debug_span!("chunk_read", chunk_index)
}

/// Span for JSON parsing work.
#[must_use]
pub fn json_parse_span(line: Option<u64>) -> Span {
    tracing::debug_span!("json_parse", line)
}

/// Span for building a batch.
#[must_use]
pub fn batch_build_span(batch_size: usize) -> Span {
    tracing::debug_span!("batch_build", batch_size)
}

/// Span for cache lookup.
#[must_use]
pub fn cache_lookup_span(key: &str) -> Span {
    tracing::debug_span!("cache_lookup", key)
}

/// Span for cache storage.
#[must_use]
pub fn cache_store_span(key: &str, bytes: u64) -> Span {
    tracing::debug_span!("cache_store", key, bytes)
}

/// Span for worker wait time.
#[must_use]
pub fn worker_wait_span(worker_id: &str) -> Span {
    tracing::trace_span!("worker_wait", worker_id)
}

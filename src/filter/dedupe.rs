//! Duplicate text filter.

use std::collections::HashSet;

use tunex_core::types::RawSample;

use super::{length::text, DatasetFilter, FilterDecision, FilterReason};

/// Deterministic exact-text dedupe filter.
#[derive(Clone, Debug, Default)]
pub struct DedupeFilter {
    seen: HashSet<Vec<u8>>,
}

impl DatasetFilter for DedupeFilter {
    fn evaluate(&mut self, sample: &RawSample) -> FilterDecision {
        let bytes = text(sample).as_bytes().to_vec();
        if !self.seen.insert(bytes) {
            return FilterDecision::Reject(FilterReason::DuplicateText);
        }

        FilterDecision::Accept
    }
}

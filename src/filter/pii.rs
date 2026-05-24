//! PII filter placeholder.

use tunex_core::types::RawSample;

use super::{DatasetFilter, FilterDecision};

/// Placeholder for future PII detection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PiiFilter;

impl DatasetFilter for PiiFilter {
    fn evaluate(&mut self, _sample: &RawSample) -> FilterDecision {
        FilterDecision::Accept
    }
}

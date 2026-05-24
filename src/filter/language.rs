//! Language filter placeholder.

use tunex_core::types::RawSample;

use super::{DatasetFilter, FilterDecision};

/// Placeholder for future language detection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageFilter {
    /// Expected language tag.
    pub expected_language: String,
}

impl DatasetFilter for LanguageFilter {
    fn evaluate(&mut self, _sample: &RawSample) -> FilterDecision {
        FilterDecision::Accept
    }
}

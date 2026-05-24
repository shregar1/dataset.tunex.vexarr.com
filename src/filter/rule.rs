//! Filter rule contracts and engine.

use regex::Regex;
use serde::{Deserialize, Serialize};
use tunex_core::types::RawSample;

use crate::{DatasetError, DatasetResult};

use super::{
    dedupe::DedupeFilter,
    language::LanguageFilter,
    length::{LengthFilter, RegexFilter},
    pii::PiiFilter,
    stats::FilterStats,
};

/// Config-driven dataset quality filters.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DatasetFilterConfig {
    /// Minimum allowed UTF-8 character count.
    pub min_chars: Option<usize>,
    /// Maximum allowed UTF-8 character count.
    pub max_chars: Option<usize>,
    /// Reject empty or whitespace-only text.
    pub remove_empty: bool,
    /// Reject duplicate text payloads.
    pub dedupe: bool,
    /// Reject text matching any regex pattern.
    pub reject_patterns: Vec<String>,
    /// Require a language tag later when detection exists.
    pub language: Option<String>,
    /// Enable placeholder PII rejection.
    pub pii: bool,
    /// Stop on first rejected sample when true.
    pub strict: bool,
}

impl DatasetFilterConfig {
    /// Builds a filter engine from config.
    pub fn build_engine(&self) -> DatasetResult<FilterEngine> {
        let mut filters: Vec<Box<dyn DatasetFilter>> = Vec::new();

        if self.remove_empty || self.min_chars.is_some() || self.max_chars.is_some() {
            filters.push(Box::new(LengthFilter {
                min_chars: self.min_chars,
                max_chars: self.max_chars,
                remove_empty: self.remove_empty,
            }));
        }
        if self.dedupe {
            filters.push(Box::new(DedupeFilter::default()));
        }
        for pattern in &self.reject_patterns {
            filters.push(Box::new(RegexFilter {
                pattern: pattern.clone(),
                regex: Regex::new(pattern)
                    .map_err(|error| DatasetError::InvalidConfig(error.to_string()))?,
            }));
        }
        if let Some(language) = &self.language {
            filters.push(Box::new(LanguageFilter {
                expected_language: language.clone(),
            }));
        }
        if self.pii {
            filters.push(Box::new(PiiFilter));
        }

        Ok(FilterEngine {
            filters,
            strict: self.strict,
            stats: FilterStats::default(),
        })
    }
}

/// Reason a sample was rejected.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FilterReason {
    /// Empty or whitespace-only text.
    EmptyText,
    /// Text is shorter than configured minimum.
    TooShort {
        min_chars: usize,
        actual_chars: usize,
    },
    /// Text is longer than configured maximum.
    TooLong {
        max_chars: usize,
        actual_chars: usize,
    },
    /// Text was seen before.
    DuplicateText,
    /// Text matched a reject regex.
    RegexMatched { pattern: String },
    /// Language requirement was not met.
    LanguageRejected { expected: String },
    /// PII rule rejected the text.
    PiiDetected,
}

/// Filter decision for one sample.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FilterDecision {
    /// Sample can continue.
    Accept,
    /// Sample should be removed.
    Reject(FilterReason),
}

/// One deterministic dataset filter rule.
pub trait DatasetFilter: Send {
    /// Evaluates a sample.
    fn evaluate(&mut self, sample: &RawSample) -> FilterDecision;
}

/// Ordered filter engine with aggregate stats.
pub struct FilterEngine {
    filters: Vec<Box<dyn DatasetFilter>>,
    strict: bool,
    stats: FilterStats,
}

impl FilterEngine {
    /// Applies filters to one sample.
    pub fn apply(&mut self, sample: RawSample) -> DatasetResult<Option<RawSample>> {
        for filter in &mut self.filters {
            match filter.evaluate(&sample) {
                FilterDecision::Accept => {}
                FilterDecision::Reject(reason) => {
                    self.stats.record_rejected(reason.clone());
                    if self.strict {
                        return Err(DatasetError::InvalidConfig(format!(
                            "strict filter rejected sample: {reason:?}"
                        )));
                    }
                    return Ok(None);
                }
            }
        }

        self.stats.record_accepted();
        Ok(Some(sample))
    }

    /// Filters an iterator of samples in order.
    pub fn filter_samples(
        &mut self,
        samples: impl IntoIterator<Item = RawSample>,
    ) -> DatasetResult<Vec<RawSample>> {
        let mut accepted = Vec::new();
        for sample in samples {
            if let Some(sample) = self.apply(sample)? {
                accepted.push(sample);
            }
        }
        Ok(accepted)
    }

    /// Returns filter statistics.
    #[must_use]
    pub const fn stats(&self) -> &FilterStats {
        &self.stats
    }
}

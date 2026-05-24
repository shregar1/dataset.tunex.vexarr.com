//! Length, empty text, and regex filters.

use regex::Regex;
use tunex_core::types::RawSample;

use super::{DatasetFilter, FilterDecision, FilterReason};

/// Min/max text length filter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LengthFilter {
    /// Minimum character count.
    pub min_chars: Option<usize>,
    /// Maximum character count.
    pub max_chars: Option<usize>,
    /// Reject whitespace-only text.
    pub remove_empty: bool,
}

impl DatasetFilter for LengthFilter {
    fn evaluate(&mut self, sample: &RawSample) -> FilterDecision {
        let text = text(sample);
        if self.remove_empty && text.trim().is_empty() {
            return FilterDecision::Reject(FilterReason::EmptyText);
        }

        let chars = text.chars().count();
        if let Some(min_chars) = self.min_chars {
            if chars < min_chars {
                return FilterDecision::Reject(FilterReason::TooShort {
                    min_chars,
                    actual_chars: chars,
                });
            }
        }
        if let Some(max_chars) = self.max_chars {
            if chars > max_chars {
                return FilterDecision::Reject(FilterReason::TooLong {
                    max_chars,
                    actual_chars: chars,
                });
            }
        }

        FilterDecision::Accept
    }
}

/// Regex reject filter.
#[derive(Clone, Debug)]
pub struct RegexFilter {
    /// Original pattern.
    pub pattern: String,
    /// Compiled regex.
    pub regex: Regex,
}

impl DatasetFilter for RegexFilter {
    fn evaluate(&mut self, sample: &RawSample) -> FilterDecision {
        if self.regex.is_match(text(sample)) {
            return FilterDecision::Reject(FilterReason::RegexMatched {
                pattern: self.pattern.clone(),
            });
        }

        FilterDecision::Accept
    }
}

pub(crate) fn text(sample: &RawSample) -> &str {
    std::str::from_utf8(&sample.payload).unwrap_or("")
}

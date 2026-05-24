//! Filter statistics.

use std::collections::BTreeMap;

use super::FilterReason;

/// Accepted/rejected sample counters with reason tracking.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FilterStats {
    /// Accepted samples.
    pub accepted: u64,
    /// Rejected samples.
    pub rejected: u64,
    /// Rejection counts grouped by reason label.
    pub rejected_by_reason: BTreeMap<String, u64>,
}

impl FilterStats {
    /// Records an accepted sample.
    pub fn record_accepted(&mut self) {
        self.accepted = self.accepted.saturating_add(1);
    }

    /// Records a rejected sample.
    pub fn record_rejected(&mut self, reason: FilterReason) {
        self.rejected = self.rejected.saturating_add(1);
        let key = reason.label().to_owned();
        *self.rejected_by_reason.entry(key).or_insert(0) += 1;
    }
}

impl FilterReason {
    /// Stable reason label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::EmptyText => "empty_text",
            Self::TooShort { .. } => "too_short",
            Self::TooLong { .. } => "too_long",
            Self::DuplicateText => "duplicate_text",
            Self::RegexMatched { .. } => "regex_matched",
            Self::LanguageRejected { .. } => "language_rejected",
            Self::PiiDetected => "pii_detected",
        }
    }
}

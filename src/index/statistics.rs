//! Dataset statistics collection.

use std::collections::{BTreeMap, HashSet};

use tunex_core::types::RawSample;

/// Dataset-level quality and size statistics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DatasetStatistics {
    /// Average UTF-8 characters per sample.
    pub avg_chars_per_sample: f64,
    /// Average bytes per sample.
    pub avg_bytes_per_sample: f64,
    /// Minimum sample character length.
    pub min_length: usize,
    /// Maximum sample character length.
    pub max_length: usize,
    /// Duplicate text ratio.
    pub duplicate_ratio: f64,
    /// Compression ratio when known.
    pub compression_ratio: Option<f64>,
    /// Placeholder language distribution.
    pub language_distribution: BTreeMap<String, u64>,
}

/// Incremental dataset statistics builder.
#[derive(Clone, Debug, Default)]
pub struct StatisticsBuilder {
    samples: u64,
    total_chars: u64,
    total_bytes: u64,
    min_length: Option<usize>,
    max_length: usize,
    duplicate_count: u64,
    seen_payloads: HashSet<Vec<u8>>,
    language_distribution: BTreeMap<String, u64>,
}

impl StatisticsBuilder {
    /// Adds a sample to the statistics set.
    pub fn record_sample(&mut self, sample: &RawSample) {
        let bytes = sample.payload.len();
        let chars = std::str::from_utf8(&sample.payload).map_or(bytes, |text| text.chars().count());

        self.samples = self.samples.saturating_add(1);
        self.total_chars = self.total_chars.saturating_add(chars as u64);
        self.total_bytes = self.total_bytes.saturating_add(bytes as u64);
        self.min_length = Some(self.min_length.map_or(chars, |current| current.min(chars)));
        self.max_length = self.max_length.max(chars);

        if !self.seen_payloads.insert(sample.payload.to_vec()) {
            self.duplicate_count = self.duplicate_count.saturating_add(1);
        }
    }

    /// Adds a placeholder language count.
    pub fn record_language(&mut self, language: impl Into<String>, count: u64) {
        *self
            .language_distribution
            .entry(language.into())
            .or_insert(0) += count;
    }

    /// Finalizes statistics.
    #[must_use]
    pub fn finish(self) -> DatasetStatistics {
        if self.samples == 0 {
            return DatasetStatistics::default();
        }

        DatasetStatistics {
            avg_chars_per_sample: self.total_chars as f64 / self.samples as f64,
            avg_bytes_per_sample: self.total_bytes as f64 / self.samples as f64,
            min_length: self.min_length.unwrap_or_default(),
            max_length: self.max_length,
            duplicate_ratio: self.duplicate_count as f64 / self.samples as f64,
            compression_ratio: None,
            language_distribution: self.language_distribution,
        }
    }
}

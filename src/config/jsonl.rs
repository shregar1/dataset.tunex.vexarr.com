//! JSONL dataset configuration.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{CompressionKind, DatasetPipelineConfig};
use crate::filter::DatasetFilterConfig;

/// Dataset input format.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFormat {
    /// JSON Lines.
    #[default]
    Jsonl,
}

/// Dataset read mode.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetMode {
    /// Use a buffered streaming reader.
    #[default]
    Stream,
    /// Use a memory-mapped file reader.
    Mmap,
}

/// JSONL dataset source configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct JsonlDatasetConfig {
    /// Dataset path.
    pub path: PathBuf,
    /// Dataset format.
    pub format: DatasetFormat,
    /// Reader mode.
    pub mode: DatasetMode,
    /// Whether memory mapping is enabled.
    pub mmap: bool,
    /// Whether SIMD JSON parsing is enabled.
    pub simd_json: bool,
    /// Compression format.
    pub compression: CompressionKind,
    /// Optional JSON field used as `RawSample.id`.
    pub id_field: Option<String>,
    /// JSON field used as `RawSample` payload text.
    pub text_field: String,
    /// Whether each record should be parsed as JSON before emission.
    pub validate_json: bool,
    /// Optional maximum number of samples to emit.
    pub max_samples: Option<usize>,
    /// Pipeline options.
    pub pipeline: DatasetPipelineConfig,
    /// Dataset quality filters.
    pub filters: DatasetFilterConfig,
}

impl JsonlDatasetConfig {
    /// Creates a JSONL dataset config from a path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }

    /// Uses memory mapping when the input is uncompressed.
    #[must_use]
    pub const fn with_mmap(mut self) -> Self {
        self.mode = DatasetMode::Mmap;
        self.mmap = true;
        self
    }

    /// Sets the maximum number of samples to emit.
    #[must_use]
    pub const fn with_max_samples(mut self, max_samples: usize) -> Self {
        self.max_samples = Some(max_samples);
        self
    }
}

impl Default for JsonlDatasetConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            format: DatasetFormat::Jsonl,
            mode: DatasetMode::Stream,
            mmap: false,
            simd_json: true,
            compression: CompressionKind::Auto,
            id_field: Some("id".to_owned()),
            text_field: "text".to_owned(),
            validate_json: true,
            max_samples: None,
            pipeline: DatasetPipelineConfig::default(),
            filters: DatasetFilterConfig::default(),
        }
    }
}

//! Dataset loader configuration.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

/// Supported compression formats.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionKind {
    /// No compression.
    #[default]
    None,
    /// Zstandard frame compression.
    Zstd,
    /// LZ4 frame compression.
    Lz4,
}

/// JSONL dataset source configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct JsonlDatasetConfig {
    /// Dataset path.
    pub path: PathBuf,
    /// Reader mode.
    pub mode: DatasetMode,
    /// Compression format.
    pub compression: CompressionKind,
    /// Optional JSON field used as `RawSample.id`.
    pub id_field: Option<String>,
    /// Whether each record should be parsed as JSON before emission.
    pub validate_json: bool,
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
}

impl Default for JsonlDatasetConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            mode: DatasetMode::Stream,
            compression: CompressionKind::None,
            id_field: Some("id".to_owned()),
            validate_json: true,
        }
    }
}

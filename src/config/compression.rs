//! Compression configuration.

use serde::{Deserialize, Serialize};

/// Supported compression formats.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionKind {
    /// Detect compression from the path extension.
    Auto,
    /// No compression.
    #[default]
    None,
    /// Zstandard frame compression.
    Zstd,
    /// LZ4 frame compression.
    Lz4,
}

//! Compression format detection.

use std::path::Path;

use crate::CompressionKind;

/// Resolved dataset input compression format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CompressionFormat {
    /// Plain uncompressed input.
    #[default]
    Plain,
    /// Zstandard-compressed input.
    Zstd,
    /// LZ4 frame-compressed input.
    Lz4,
}

impl CompressionFormat {
    /// Resolves a format from config and path extension.
    #[must_use]
    pub fn resolve(path: &Path, configured: CompressionKind) -> Self {
        match configured {
            CompressionKind::Auto => Self::detect(path),
            CompressionKind::None => Self::Plain,
            CompressionKind::Zstd => Self::Zstd,
            CompressionKind::Lz4 => Self::Lz4,
        }
    }

    /// Detects compression format from path extension.
    #[must_use]
    pub fn detect(path: &Path) -> Self {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Self::Plain;
        };

        if name.ends_with(".jsonl.zst") || name.ends_with(".zst") {
            Self::Zstd
        } else if name.ends_with(".jsonl.lz4") || name.ends_with(".lz4") {
            Self::Lz4
        } else {
            Self::Plain
        }
    }

    /// Returns true when the input can be memory-mapped directly.
    #[must_use]
    pub const fn supports_mmap(self) -> bool {
        matches!(self, Self::Plain)
    }
}

//! Dataset module error model.

use std::{io, path::PathBuf};

use thiserror::Error;
use tunex_core::{error::ErrorKind, TunexError};

/// Result alias for dataset operations.
pub type DatasetResult<T> = Result<T, DatasetError>;

/// Errors produced by dataset readers and parsers.
#[derive(Debug, Error)]
pub enum DatasetError {
    /// Underlying I/O error.
    #[error("dataset I/O failed for {path}: {source}")]
    Io {
        /// Path being read.
        path: PathBuf,
        /// Source error.
        #[source]
        source: io::Error,
    },
    /// JSONL row could not be parsed.
    #[error("invalid JSONL record at line {line}: {message}")]
    InvalidJson {
        /// One-based line number.
        line: u64,
        /// Parser message.
        message: String,
    },
    /// JSONL row is missing a string text field.
    #[error("JSONL record at line {line} is missing string field `{field}`")]
    MissingText {
        /// One-based line number.
        line: u64,
        /// Required text field.
        field: String,
    },
    /// The selected compression format is not supported for this reader.
    #[error("unsupported compression format: {0:?}")]
    UnsupportedCompression(crate::config::CompressionKind),
    /// The dataset config is invalid.
    #[error("invalid dataset config: {0}")]
    InvalidConfig(String),
}

impl DatasetError {
    /// Creates an I/O error with path context.
    #[must_use]
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl From<DatasetError> for TunexError {
    fn from(error: DatasetError) -> Self {
        TunexError::new(ErrorKind::Dataset, error.to_string()).recoverable()
    }
}

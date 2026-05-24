//! Memory-mapped JSONL dataset reader.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use memmap2::{Mmap, MmapOptions};
use tunex_core::types::RawSample;

use crate::{
    compression::CompressionFormat, DatasetError, DatasetResult, JsonlDatasetConfig,
    ValidateDatasetConfig,
};

use super::{
    line_index::LineIndex, reader::record_to_sample, slice::LineSlice, JsonlParser, JsonlRecord,
};

struct MmapOwner {
    mmap: Mmap,
}

impl AsRef<[u8]> for MmapOwner {
    fn as_ref(&self) -> &[u8] {
        &self.mmap
    }
}

/// Random-access memory-mapped JSONL dataset.
pub struct MmapJsonlDataset {
    path: PathBuf,
    bytes: Bytes,
    index: LineIndex,
    parser: JsonlParser,
    max_samples: Option<usize>,
}

impl MmapJsonlDataset {
    /// Opens a memory-mapped JSONL dataset from a path using default config.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be opened, mapped, or indexed.
    pub fn open(path: impl AsRef<Path>) -> DatasetResult<Self> {
        Self::open_config(JsonlDatasetConfig::new(path.as_ref().to_path_buf()))
    }

    /// Opens a memory-mapped JSONL dataset from explicit config.
    ///
    /// # Errors
    ///
    /// Returns an error when config is invalid or the file cannot be mapped.
    pub fn open_config(config: JsonlDatasetConfig) -> DatasetResult<Self> {
        validate_config(&config)?;
        let format = CompressionFormat::resolve(&config.path, config.compression);
        if !format.supports_mmap() {
            return Err(DatasetError::UnsupportedCompression(config.compression));
        }

        let file =
            File::open(&config.path).map_err(|error| DatasetError::io(&config.path, error))?;
        let mmap = unsafe {
            // SAFETY: The map is read-only, owned by this dataset, and this type
            // never mutates the underlying file.
            MmapOptions::new()
                .map(&file)
                .map_err(|error| DatasetError::io(&config.path, error))?
        };
        let bytes = Bytes::from_owner(MmapOwner { mmap });
        let index = LineIndex::build(bytes.as_ref());

        Ok(Self {
            path: config.path.clone(),
            bytes,
            index,
            parser: JsonlParser::new(&config),
            max_samples: config.max_samples,
        })
    }

    /// Returns the number of indexed records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.max_samples
            .map_or_else(|| self.index.len(), |limit| self.index.len().min(limit))
    }

    /// Returns true when the dataset has no indexed records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the source path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns a zero-copy line slice by record index.
    ///
    /// # Errors
    ///
    /// Returns an error when the index is out of bounds.
    pub fn line(&self, index: usize) -> DatasetResult<LineSlice> {
        if self.max_samples.is_some_and(|limit| index >= limit) {
            return Err(DatasetError::IndexOutOfBounds {
                index,
                len: self.len(),
            });
        }

        let span = self
            .index
            .get(index)
            .ok_or_else(|| DatasetError::IndexOutOfBounds {
                index,
                len: self.len(),
            })?;

        Ok(LineSlice::new(
            span.line,
            self.bytes.slice(span.start..span.end),
        ))
    }

    /// Returns a parsed JSONL record by record index.
    ///
    /// # Errors
    ///
    /// Returns an error when the index is invalid or parsing fails.
    pub fn record(&self, index: usize) -> DatasetResult<JsonlRecord> {
        let line = self.line(index)?;
        self.parser.parse_bytes(line.line, line.bytes)
    }

    /// Returns a raw sample by record index.
    ///
    /// # Errors
    ///
    /// Returns an error when the index is invalid or parsing fails.
    pub fn get(&self, index: usize) -> DatasetResult<RawSample> {
        self.record(index).map(record_to_sample)
    }
}

fn validate_config(config: &JsonlDatasetConfig) -> DatasetResult<()> {
    let report = config.validate();
    if report.is_valid() {
        return Ok(());
    }

    let message = report
        .errors
        .into_iter()
        .map(|error| format!("{}: {}", error.path, error.message))
        .collect::<Vec<_>>()
        .join("; ");
    Err(DatasetError::InvalidConfig(message))
}

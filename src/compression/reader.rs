//! Dataset input reader with compression stats.

use std::{
    fs::File,
    io::{BufReader, Read, Result as IoResult},
    path::{Path, PathBuf},
};

use crate::{
    compression::{format::CompressionFormat, lz4, zstd},
    CompressionKind, DatasetError, DatasetResult,
};

/// Compression stream statistics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompressionStats {
    /// Resolved compression format.
    pub format: CompressionFormat,
    /// Compressed bytes read from the wrapped stream when observable.
    pub compressed_bytes: u64,
    /// Decompressed bytes emitted to the caller.
    pub decompressed_bytes: u64,
    /// Number of read chunks returned by the stream.
    pub chunks: u64,
}

/// Input reader that handles plain and compressed files.
pub struct DatasetInputReader {
    path: PathBuf,
    stats: CompressionStats,
    reader: Box<dyn Read + Send>,
}

impl DatasetInputReader {
    /// Opens an input reader from path and compression config.
    ///
    /// # Errors
    ///
    /// Returns an error when the file or decoder cannot be opened.
    pub fn open(path: &Path, compression: CompressionKind) -> DatasetResult<Self> {
        let format = CompressionFormat::resolve(path, compression);
        let reader: Box<dyn Read + Send> = match format {
            CompressionFormat::Plain => {
                let file = File::open(path).map_err(|error| DatasetError::io(path, error))?;
                Box::new(BufReader::new(file))
            }
            CompressionFormat::Zstd => zstd::open(path)?,
            CompressionFormat::Lz4 => lz4::open(path)?,
        };

        Ok(Self {
            path: path.to_path_buf(),
            stats: CompressionStats {
                format,
                ..CompressionStats::default()
            },
            reader,
        })
    }

    /// Returns the resolved compression format.
    #[must_use]
    pub const fn format(&self) -> CompressionFormat {
        self.stats.format
    }

    /// Returns current compression stats.
    #[must_use]
    pub const fn stats(&self) -> CompressionStats {
        self.stats
    }

    /// Converts into a boxed reader.
    #[must_use]
    pub fn into_reader(self) -> Box<dyn Read + Send> {
        Box::new(self)
    }

    /// Returns the input path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Read for DatasetInputReader {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        let read = self.reader.read(buffer)?;
        if read > 0 {
            self.stats.decompressed_bytes = self
                .stats
                .decompressed_bytes
                .saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
            self.stats.chunks = self.stats.chunks.saturating_add(1);
        }
        Ok(read)
    }
}

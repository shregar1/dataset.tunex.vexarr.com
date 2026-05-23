//! Memory-mapped JSONL reader.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use memmap2::{Mmap, MmapOptions};

use crate::{
    config::CompressionKind,
    reader::{DatasetReader, JsonlRecord},
    DatasetError, DatasetResult,
};

/// Memory-mapped JSONL reader for uncompressed files.
pub struct MmapJsonlReader {
    path: PathBuf,
    mmap: Mmap,
    offset: usize,
    line: u64,
}

impl MmapJsonlReader {
    /// Opens an uncompressed file as a memory-mapped JSONL reader.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be opened or mapped.
    pub fn open(path: impl AsRef<Path>, compression: CompressionKind) -> DatasetResult<Self> {
        if compression != CompressionKind::None {
            return Err(DatasetError::UnsupportedCompression(compression));
        }

        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|error| DatasetError::io(&path, error))?;
        let mmap = unsafe {
            // SAFETY: The mapping is read-only, tied to this reader, and the file
            // handle is not used for mutation by this type.
            MmapOptions::new()
                .map(&file)
                .map_err(|error| DatasetError::io(&path, error))?
        };

        Ok(Self {
            path,
            mmap,
            offset: 0,
            line: 0,
        })
    }
}

impl DatasetReader for MmapJsonlReader {
    fn next_record(&mut self) -> DatasetResult<Option<JsonlRecord>> {
        while self.offset < self.mmap.len() {
            let start = self.offset;
            let relative_end = self.mmap[start..]
                .iter()
                .position(|byte| *byte == b'\n')
                .unwrap_or(self.mmap.len() - start);
            let end = start + relative_end;
            self.offset = end.saturating_add(1);
            self.line = self.line.saturating_add(1);

            let mut record_end = end;
            if record_end > start && self.mmap[record_end - 1] == b'\r' {
                record_end -= 1;
            }
            if record_end == start {
                continue;
            }

            return Ok(Some(JsonlRecord {
                line: self.line,
                bytes: Bytes::copy_from_slice(&self.mmap[start..record_end]),
            }));
        }

        tracing::trace!(path = %self.path.display(), "mmap jsonl reader exhausted");
        Ok(None)
    }
}

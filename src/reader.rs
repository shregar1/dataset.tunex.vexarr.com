//! JSONL reader contracts and streaming implementation.

use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use bytes::Bytes;

use crate::{compression::open_reader, CompressionKind, DatasetError, DatasetResult};

/// One raw JSONL record with source position metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonlRecord {
    /// One-based line number.
    pub line: u64,
    /// Raw line bytes without the trailing newline.
    pub bytes: Bytes,
}

/// Pull-based dataset reader.
pub trait DatasetReader: Send {
    /// Returns the next record, or `None` at end of file.
    ///
    /// # Errors
    ///
    /// Returns an error when reading fails.
    fn next_record(&mut self) -> DatasetResult<Option<JsonlRecord>>;
}

/// Buffered streaming JSONL reader.
pub struct StreamingJsonlReader {
    path: PathBuf,
    reader: BufReader<Box<dyn Read + Send>>,
    line: u64,
    buffer: Vec<u8>,
}

impl StreamingJsonlReader {
    /// Opens a streaming reader.
    ///
    /// # Errors
    ///
    /// Returns an error when the file or decoder cannot be opened.
    pub fn open(path: impl AsRef<Path>, compression: CompressionKind) -> DatasetResult<Self> {
        let path = path.as_ref().to_path_buf();
        let reader = open_reader(&path, compression)?;
        Ok(Self {
            path,
            reader: BufReader::new(reader),
            line: 0,
            buffer: Vec::new(),
        })
    }
}

impl DatasetReader for StreamingJsonlReader {
    fn next_record(&mut self) -> DatasetResult<Option<JsonlRecord>> {
        loop {
            self.buffer.clear();
            let bytes_read = self
                .reader
                .read_until(b'\n', &mut self.buffer)
                .map_err(|error| DatasetError::io(&self.path, error))?;

            if bytes_read == 0 {
                return Ok(None);
            }

            self.line = self.line.saturating_add(1);
            trim_newline(&mut self.buffer);
            if self.buffer.is_empty() {
                continue;
            }

            return Ok(Some(JsonlRecord {
                line: self.line,
                bytes: Bytes::copy_from_slice(&self.buffer),
            }));
        }
    }
}

fn trim_newline(buffer: &mut Vec<u8>) {
    while matches!(buffer.last(), Some(b'\n' | b'\r')) {
        let _ = buffer.pop();
    }
}

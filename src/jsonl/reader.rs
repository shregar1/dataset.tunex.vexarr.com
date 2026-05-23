//! Buffered JSONL dataset source.

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::Mutex,
};

use async_trait::async_trait;
use tunex_core::{traits::DatasetSource, types::RawSample, TunexResult};

use crate::{DatasetError, DatasetResult, JsonlDatasetConfig};

use super::{JsonlParser, JsonlRecord};

/// Simple buffered JSONL dataset reader.
pub struct JsonlDataset {
    path: std::path::PathBuf,
    reader: BufReader<File>,
    parser: JsonlParser,
    line: u64,
    emitted: usize,
    max_samples: Option<usize>,
    buffer: Vec<u8>,
}

impl JsonlDataset {
    /// Opens a JSONL dataset from a path using default config.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be opened.
    pub fn open(path: impl AsRef<Path>) -> DatasetResult<Self> {
        Self::open_config(JsonlDatasetConfig::new(path.as_ref().to_path_buf()))
    }

    /// Opens a JSONL dataset from explicit config.
    ///
    /// # Errors
    ///
    /// Returns an error when config is invalid or the file cannot be opened.
    pub fn open_config(config: JsonlDatasetConfig) -> DatasetResult<Self> {
        if config.path.as_os_str().is_empty() {
            return Err(DatasetError::InvalidConfig(
                "dataset path must not be empty".to_owned(),
            ));
        }

        let file =
            File::open(&config.path).map_err(|error| DatasetError::io(&config.path, error))?;
        Ok(Self {
            path: config.path.clone(),
            reader: BufReader::new(file),
            parser: JsonlParser::new(&config),
            line: 0,
            emitted: 0,
            max_samples: config.max_samples,
            buffer: Vec::new(),
        })
    }

    /// Returns the next raw sample, or `None` when exhausted.
    ///
    /// # Errors
    ///
    /// Returns an error when reading or parsing fails.
    pub fn next_sample(&mut self) -> DatasetResult<Option<RawSample>> {
        let Some(record) = self.next_record()? else {
            return Ok(None);
        };

        Ok(Some(record_to_sample(record)))
    }

    /// Returns the next parsed JSONL record, or `None` when exhausted.
    ///
    /// # Errors
    ///
    /// Returns an error when reading or parsing fails.
    pub fn next_record(&mut self) -> DatasetResult<Option<JsonlRecord>> {
        if self.max_samples.is_some_and(|limit| self.emitted >= limit) {
            return Ok(None);
        }

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

            let record = self.parser.parse_line(self.line, &self.buffer)?;
            self.emitted = self.emitted.saturating_add(1);
            return Ok(Some(record));
        }
    }
}

/// Core-compatible JSONL dataset source.
pub struct JsonlDatasetSource {
    inner: Mutex<JsonlDataset>,
}

impl JsonlDatasetSource {
    /// Opens a core-compatible JSONL dataset source.
    ///
    /// # Errors
    ///
    /// Returns an error when config is invalid or the file cannot be opened.
    pub fn open(config: JsonlDatasetConfig) -> DatasetResult<Self> {
        Ok(Self {
            inner: Mutex::new(JsonlDataset::open_config(config)?),
        })
    }
}

#[async_trait]
impl DatasetSource for JsonlDatasetSource {
    async fn next_sample(&self) -> TunexResult<Option<RawSample>> {
        let mut inner = self.inner.lock().map_err(|_| {
            tunex_core::TunexError::new(
                tunex_core::error::ErrorKind::Dataset,
                "JSONL dataset mutex poisoned",
            )
            .recoverable()
        })?;

        inner.next_sample().map_err(tunex_core::TunexError::from)
    }
}

fn record_to_sample(record: JsonlRecord) -> RawSample {
    match record.id {
        Some(id) => RawSample::with_id(id, record.text),
        None => RawSample::new(record.text),
    }
}

fn trim_newline(buffer: &mut Vec<u8>) {
    while matches!(buffer.last(), Some(b'\n' | b'\r')) {
        let _ = buffer.pop();
    }
}

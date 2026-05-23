//! JSONL dataset source implementation.

use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::Value;
use tunex_core::{traits::DatasetSource, types::RawSample, TunexResult};

use crate::{
    config::{DatasetMode, JsonlDatasetConfig},
    mmap::MmapJsonlReader,
    reader::{DatasetReader, JsonlRecord, StreamingJsonlReader},
    DatasetError, DatasetResult,
};

/// JSONL-backed dataset source.
pub struct JsonlDatasetSource {
    config: JsonlDatasetConfig,
    reader: Mutex<Box<dyn DatasetReader>>,
}

impl JsonlDatasetSource {
    /// Opens a JSONL dataset source.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured reader cannot be opened.
    pub fn open(config: JsonlDatasetConfig) -> DatasetResult<Self> {
        if config.path.as_os_str().is_empty() {
            return Err(DatasetError::InvalidConfig(
                "dataset path must not be empty".to_owned(),
            ));
        }

        let reader: Box<dyn DatasetReader> = match config.mode {
            DatasetMode::Stream => Box::new(StreamingJsonlReader::open(
                &config.path,
                config.compression,
            )?),
            DatasetMode::Mmap => Box::new(MmapJsonlReader::open(&config.path, config.compression)?),
        };

        Ok(Self {
            config,
            reader: Mutex::new(reader),
        })
    }

    fn sample_from_record(&self, record: JsonlRecord) -> DatasetResult<RawSample> {
        if !self.config.validate_json {
            return Ok(RawSample::new(record.bytes));
        }

        let mut json_bytes = record.bytes.to_vec();
        let value: Value = simd_json::serde::from_slice(&mut json_bytes).map_err(|error| {
            DatasetError::InvalidJson {
                line: record.line,
                message: error.to_string(),
            }
        })?;
        let id = self
            .config
            .id_field
            .as_deref()
            .and_then(|field| value.get(field))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        Ok(match id {
            Some(id) => RawSample::with_id(id, record.bytes),
            None => RawSample::new(record.bytes),
        })
    }
}

#[async_trait]
impl DatasetSource for JsonlDatasetSource {
    async fn next_sample(&self) -> TunexResult<Option<RawSample>> {
        let mut reader = self.reader.lock().map_err(|_| {
            tunex_core::TunexError::new(
                tunex_core::error::ErrorKind::Dataset,
                "dataset reader mutex poisoned",
            )
            .recoverable()
        })?;
        let Some(record) = reader.next_record().map_err(tunex_core::TunexError::from)? else {
            return Ok(None);
        };

        self.sample_from_record(record)
            .map(Some)
            .map_err(tunex_core::TunexError::from)
    }
}

//! Core-compatible dataset source implementation.

use std::sync::Mutex;

use async_trait::async_trait;
use tunex_core::{traits::DatasetSource, types::RawSample, TunexResult};

use crate::{
    compression::CompressionFormat, DatasetResult, JsonlDataset, JsonlDatasetConfig,
    MmapJsonlDataset, ValidateDatasetConfig,
};

enum SourceInner {
    Stream(Mutex<JsonlDataset>),
    Mmap {
        dataset: MmapJsonlDataset,
        cursor: Mutex<usize>,
    },
}

/// TuneX core `DatasetSource` backed by the JSONL dataset engine.
pub struct TunexDatasetSource {
    inner: SourceInner,
}

impl TunexDatasetSource {
    /// Opens a core-compatible dataset source.
    ///
    /// Uncompressed mmap configs use the mmap reader. Compressed inputs fall
    /// back to streaming decompression because compressed files cannot be
    /// memory-mapped directly.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured source cannot be opened.
    pub fn open(config: JsonlDatasetConfig) -> DatasetResult<Self> {
        validate_config(&config)?;
        let format = CompressionFormat::resolve(&config.path, config.compression);
        if (config.mode == crate::DatasetMode::Mmap || config.mmap) && format.supports_mmap() {
            return Ok(Self {
                inner: SourceInner::Mmap {
                    dataset: MmapJsonlDataset::open_config(config)?,
                    cursor: Mutex::new(0),
                },
            });
        }

        Ok(Self {
            inner: SourceInner::Stream(Mutex::new(JsonlDataset::open_config(config)?)),
        })
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
    Err(crate::DatasetError::InvalidConfig(message))
}

#[async_trait]
impl DatasetSource for TunexDatasetSource {
    async fn next_sample(&self) -> TunexResult<Option<RawSample>> {
        match &self.inner {
            SourceInner::Stream(dataset) => dataset
                .lock()
                .map_err(|_| {
                    tunex_core::TunexError::new(
                        tunex_core::error::ErrorKind::Dataset,
                        "dataset source mutex poisoned",
                    )
                    .recoverable()
                })?
                .next_sample()
                .map_err(tunex_core::TunexError::from),
            SourceInner::Mmap { dataset, cursor } => {
                let mut cursor = cursor.lock().map_err(|_| {
                    tunex_core::TunexError::new(
                        tunex_core::error::ErrorKind::Dataset,
                        "dataset mmap cursor mutex poisoned",
                    )
                    .recoverable()
                })?;
                if *cursor >= dataset.len() {
                    return Ok(None);
                }

                let sample = dataset
                    .record(*cursor)
                    .map(|record| match record.id {
                        Some(id) => RawSample::with_id(id, record.text),
                        None => RawSample::new(record.text),
                    })
                    .map_err(tunex_core::TunexError::from)?;
                *cursor = cursor.saturating_add(1);
                Ok(Some(sample))
            }
        }
    }
}

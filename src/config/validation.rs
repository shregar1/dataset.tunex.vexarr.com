//! Dataset config validation.

use std::path::Path;

use crate::{compression::CompressionFormat, CompressionKind, DatasetMode, JsonlDatasetConfig};

/// One config validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigValidationError {
    /// Dot path of the invalid config value.
    pub path: String,
    /// Human-readable validation message.
    pub message: String,
}

impl ConfigValidationError {
    /// Creates a validation error.
    #[must_use]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Dataset config validation report.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigValidationReport {
    /// Validation errors.
    pub errors: Vec<ConfigValidationError>,
}

impl ConfigValidationReport {
    /// Returns true when no validation errors exist.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    fn push(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ConfigValidationError::new(path, message));
    }
}

/// Validation contract for dataset configs.
pub trait ValidateDatasetConfig {
    /// Validates this config.
    fn validate(&self) -> ConfigValidationReport;
}

impl ValidateDatasetConfig for JsonlDatasetConfig {
    fn validate(&self) -> ConfigValidationReport {
        let mut report = ConfigValidationReport::default();

        if !self.path.exists() {
            report.push("path", "dataset file does not exist");
        }
        if self.pipeline.batch_size == 0 {
            report.push("batch_size", "batch size must be greater than zero");
        }
        if self.pipeline.chunk_size_mb == 0 || self.pipeline.chunk_size_mb > 4096 {
            report.push("chunk_size_mb", "chunk size must be between 1 and 4096 MiB");
        }
        if self.pipeline.prefetch_batches == 0 {
            report.push(
                "prefetch_batches",
                "prefetch batches must be greater than zero",
            );
        }
        validate_workers(&mut report, self.pipeline.workers);
        validate_compression(
            &mut report,
            &self.path,
            self.compression,
            self.mmap || self.mode == DatasetMode::Mmap,
        );

        report
    }
}

fn validate_workers(report: &mut ConfigValidationReport, workers: usize) {
    let cpus = std::thread::available_parallelism().map_or(1, usize::from);
    if workers == 0 {
        report.push("workers", "workers must be greater than zero");
    } else if workers > cpus {
        report.push(
            "workers",
            "workers must not exceed available CPU parallelism",
        );
    }
}

fn validate_compression(
    report: &mut ConfigValidationReport,
    path: &Path,
    configured: CompressionKind,
    mmap: bool,
) {
    let detected = CompressionFormat::detect(path);
    let resolved = CompressionFormat::resolve(path, configured);

    if configured != CompressionKind::Auto
        && detected != CompressionFormat::Plain
        && resolved != detected
    {
        report.push(
            "compression",
            "configured compression does not match file extension",
        );
    }

    if mmap && !resolved.supports_mmap() {
        report.push("mmap", "mmap must be disabled for compressed files");
    }
}
